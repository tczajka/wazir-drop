use std::{error::Error, fs, path::{Path, PathBuf}, time::Instant};
use extra::PSFeatures;
use plotters::{backend::SVGBackend, chart::ChartBuilder, drawing::IntoDrawingArea, series::LineSeries, style::{BLUE, WHITE}};
use serde::Deserialize;
use tch::{Device, Reduction, Tensor, nn};
use wazir_drop::{Features, WPSFeatures};
use crate::{config::{FeaturesConfig, ModelConfig}, data::{DatasetConfig, DatasetIterator}, linear::LinearModel, model::EvalModel, nnue::NnueModel};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    dataset: DatasetConfig,
    weights: PathBuf,
    model: ModelConfig,
    graph_dir: PathBuf,
}

pub fn run(config_dir: &Path, config: &Config) -> Result<(), Box<dyn Error>> {
    match config.dataset.features {
        FeaturesConfig::PS => run_with_features(PSFeatures, config_dir, config),
        FeaturesConfig::WPS => run_with_features(WPSFeatures, config_dir, config),
    }
}

fn run_with_features<F: Features>(features: F, config_dir: &Path, config: &Config) -> Result<(), Box<dyn Error>> {
    match &config.model {
        ModelConfig::Linear(c) => run_with_model::<LinearModel<F>>(features, config_dir, config, c),
        ModelConfig::Nnue(c) => run_with_model::<NnueModel<F>>(features, config_dir, config, c),
    }
}

fn run_with_model<M: EvalModel>(
    features: M::Features,
    config_dir: &Path,
    config: &Config,
    model_config: &M::Config,
) -> Result<(), Box<dyn Error>> {
    let device = Device::cuda_if_available();
    log::info!("Validating using device: {device:?}");
    let mut vs = nn::VarStore::new(device);
    let mut model = M::new(features, vs.root(), model_config);
    vs.load(&config.weights)?;

    let mut num_samples = 0;
    let mut total_loss: f64 = 0.0;
    let start_time = Instant::now();
    let mut dataset_iterator = DatasetIterator::new(&config.dataset)?;
    while let Some(batch) = dataset_iterator.next_batch()? {
        let batch = batch.to_device(device);
        let values = model.forward(&batch.features, &batch.offsets);
        let loss = values.binary_cross_entropy_with_logits::<Tensor>(
            &batch.outputs,
            None,
            None,
            Reduction::Mean,
        );
        num_samples += batch.size;
        total_loss += batch.size as f64 * f64::try_from(&loss).unwrap();
    }
    let elapsed_time = start_time.elapsed().as_secs_f64();
    log::info!(
        "samples={num_samples} time={elapsed_time:.2}s \
        samples/s={samples_per_second:.0} loss={loss:.6}",
        samples_per_second = num_samples as f64 / elapsed_time,
        loss = total_loss / num_samples as f64,
    );

    let graph_dir = config_dir.join(&config.graph_dir);
    fs::create_dir_all(&graph_dir)?;
    plot_weights(&model, &graph_dir)?;

    Ok(())
}

fn plot_weights<M: EvalModel>(model: &M, graph_dir: &Path) -> Result<(), Box<dyn Error>> {
    let _guard = tch::no_grad_guard();

    for layer in 0..model.num_layers() {
        let weights = model.layer_weights(layer);
        let mut weights: Vec<f64> = weights.try_into().unwrap();
        weights.sort_by(f64::total_cmp);

        let filename = graph_dir.join(format!("weights_{layer}.svg"));
        let root = SVGBackend::new(&filename, (640, 480)).into_drawing_area();
        root.fill(&WHITE)?;

        let mut chart_context = ChartBuilder::on(&root)
            .margin(5)
            .set_all_label_area_size(50)
            .build_cartesian_2d(*weights.first().unwrap()..*weights.last().unwrap(), 0.0..1.0)?;

        chart_context.configure_mesh()
            .x_labels(20)
            .y_labels(10)
            .draw()?;

        let step_by = (weights.len()/10000).max(1);
        _ = chart_context.draw_series(LineSeries::new(
            (0..weights.len()).step_by(step_by).map(
                |index| (weights[index], index as f64 / (weights.len() - 1) as f64)
            ),
            &BLUE
        ))?;

        root.present()?;
        log::info!("Written chart {}", filename.display());
    }

    Ok(())
}