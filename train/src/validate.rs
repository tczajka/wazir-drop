use crate::{
    config::FeaturesConfig,
    data::{DatasetConfig, DatasetIterator},
    linear::LinearModel,
    model::EvalModel,
    nnue::{self, NnueModel},
};
use extra::PSFeatures;
use plotters::{
    backend::SVGBackend,
    chart::ChartBuilder,
    drawing::IntoDrawingArea,
    element::Circle,
    series::LineSeries,
    style::{BLUE, Color, WHITE},
};
use serde::Deserialize;
use std::{
    error::Error,
    fs,
    path::{Path, PathBuf},
    time::Instant,
};
use tch::{Device, Reduction, Tensor, nn};
use wazir_drop::{Features, WPSFeatures};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    dataset: DatasetConfig,
    weights: PathBuf,
    model: ModelConfig,
    graph_dir: PathBuf,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelConfig {
    Linear,
    Nnue { config: nnue::Config },
}

pub fn run(config_dir: &Path, config: &Config) -> Result<(), Box<dyn Error>> {
    match config.dataset.features {
        FeaturesConfig::PS => run_with_features(PSFeatures, config_dir, config),
        FeaturesConfig::WPS => run_with_features(WPSFeatures, config_dir, config),
    }
}

fn run_with_features<F: Features>(
    features: F,
    config_dir: &Path,
    config: &Config,
) -> Result<(), Box<dyn Error>> {
    match &config.model {
        ModelConfig::Linear => run_with_model::<LinearModel<F>>(features, config_dir, config, &()),
        ModelConfig::Nnue {
            config: nnue_config,
        } => run_with_model::<NnueModel<F>>(features, config_dir, config, nnue_config),
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

    let mut activation_stats: Vec<ActivationStats> = (0..model.num_layers() - 1)
        .map(|_| ActivationStats::new())
        .collect();

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

        for (layer, act_stats) in activation_stats.iter_mut().enumerate() {
            let _guard = tch::no_grad_guard();
            act_stats.update(model.activations(layer));
        }
    }
    let elapsed_time = start_time.elapsed().as_secs_f64();
    log::info!(
        "samples={num_samples} time={elapsed_time:.2}s \
        samples/s={samples_per_second:.0} loss={loss:.6}",
        samples_per_second = num_samples as f64 / elapsed_time,
        loss = total_loss / num_samples as f64,
    );

    let _guard = tch::no_grad_guard();
    let graph_dir = config_dir.join(&config.graph_dir);
    fs::create_dir_all(&graph_dir)?;
    for layer in 0..model.num_layers() {
        let filename = graph_dir.join(format!("weights_{layer}.svg"));
        let weights = model.layer_weights(layer);
        plot_weights(&weights, &filename)?;
    }
    for (layer, act_stats) in activation_stats.iter().enumerate() {
        let filename = graph_dir.join(format!("activations_{layer}.svg"));
        plot_activations(act_stats, &filename)?;
    }

    Ok(())
}

fn plot_weights(weights: &Tensor, filename: &Path) -> Result<(), Box<dyn Error>> {
    let mut weights: Vec<f64> = weights.try_into().unwrap();
    weights.sort_by(f64::total_cmp);

    let root = SVGBackend::new(filename, (1024, 384)).into_drawing_area();
    root.fill(&WHITE)?;

    let mut chart_context = ChartBuilder::on(&root)
        .margin(5)
        .set_all_label_area_size(50)
        .caption("Weight CDF", ("sans-serif", 40))
        .build_cartesian_2d(
            *weights.first().unwrap()..*weights.last().unwrap(),
            0.0..1.0,
        )?;

    chart_context
        .configure_mesh()
        .x_labels(20)
        .y_labels(10)
        .draw()?;

    let step_by = (weights.len() / 10000).max(1);
    _ = chart_context.draw_series(LineSeries::new(
        (0..weights.len())
            .step_by(step_by)
            .map(|index| (weights[index], index as f64 / (weights.len() - 1) as f64)),
        &BLUE,
    ))?;

    root.present()?;
    log::info!("Written chart {}", filename.display());
    Ok(())
}

struct ActivationStats {
    num_samples: i64,
    mean: Tensor,
    // sum of (x-mean)^2
    m2: Tensor,
}

impl ActivationStats {
    fn new() -> Self {
        Self {
            num_samples: 0,
            mean: Tensor::new(),
            m2: Tensor::new(),
        }
    }

    fn update(&mut self, activations: Tensor) {
        let n = activations.size()[0];
        let mean = activations.mean_dim(0, false, tch::Kind::Float);
        let m2 = activations.var_dim(0, false, false) * (n - 1) as f64;

        if self.num_samples == 0 {
            self.num_samples = n;
            self.mean = mean;
            self.m2 = m2;
            return;
        }

        let delta = &mean - &self.mean;
        let total_samples = self.num_samples + n;

        self.mean += &delta * (n as f64 / total_samples as f64);
        self.m2 += &m2;
        self.m2 += &delta.pow_tensor_scalar(2)
            * (self.num_samples as f64 * n as f64 / total_samples as f64);
        self.num_samples = total_samples;
    }
}

fn plot_activations(act_stats: &ActivationStats, filename: &Path) -> Result<(), Box<dyn Error>> {
    let means: Vec<f64> = (&act_stats.mean).try_into().unwrap();
    let stddevs: Vec<f64> = (&act_stats.m2 / (act_stats.num_samples - 1) as f64)
        .sqrt()
        .try_into()
        .unwrap();
    let mut mean_stddev: Vec<(f64, f64)> = means.into_iter().zip(stddevs).collect();
    mean_stddev.sort_by(|(m1, _), (m2, _)| m1.total_cmp(m2));

    let root = SVGBackend::new(filename, (1024, 768)).into_drawing_area();
    root.fill(&WHITE)?;
    let (upper, lower) = root.split_vertically(768 / 2);

    let mut chart_context = ChartBuilder::on(&upper)
        .margin(5)
        .set_all_label_area_size(50)
        .caption("Activations", ("sans-serif", 40))
        .build_cartesian_2d(0.0..1.0, 0.0..0.5)?;

    chart_context
        .configure_mesh()
        .x_labels(20)
        .x_desc("mean")
        .y_labels(10)
        .y_desc("stddev")
        .draw()?;

    _ = chart_context.draw_series(
        mean_stddev
            .iter()
            .map(|&(mean, stddev)| Circle::new((mean, stddev), 2, BLUE.filled())),
    )?;

    let mut chart_context = ChartBuilder::on(&lower)
        .margin(5)
        .set_all_label_area_size(50)
        .caption("Mean CDF", ("sans-serif", 40))
        .build_cartesian_2d(0.0..1.0, 0.0..1.0)?;

    chart_context
        .configure_mesh()
        .x_labels(20)
        .y_labels(10)
        .draw()?;

    _ = chart_context.draw_series(LineSeries::new(
        mean_stddev
            .iter()
            .enumerate()
            .map(|(index, &(mean, _))| (mean, index as f64 / (mean_stddev.len() - 1) as f64)),
        &BLUE,
    ))?;

    root.present()?;
    log::info!("Written chart {}", filename.display());
    Ok(())
}
