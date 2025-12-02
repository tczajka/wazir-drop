use std::{
    error::Error,
    fs::File,
    io::{BufWriter, Write},
    path::Path,
};

use crate::model::{EvalModel, Export};
use extra::base128_encoder::Base128Encoder;
use serde::Deserialize;
use tch::{
    Tensor,
    nn::{self, Module},
};
use wazir_drop::Features;

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    embedding_size: i64,
    hidden_sizes: Vec<i64>,
    hidden_weight_bits: u32,
    value_scale: f64,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LearnConfig {
    max_embedding: f64,
}

#[derive(Debug)]
pub struct NnueModel<F: Features> {
    _features: F,
    config: Config,
    embedding_weights: Tensor,
    embedding_bias: Tensor,
    hidden: Vec<nn::Linear>,
    final_layer: nn::Linear,
    max_hidden_weight: f64,
    max_last_layer_weight: f64,
    activations: Vec<Tensor>,
}

impl<F: Features> NnueModel<F> {
    fn encode_tensor(&self, encoder: &mut Base128Encoder, tensor: &Tensor, multiplier: f64) {
        let weights = tensor.flatten(0, -1) * multiplier;
        let max: f64 = weights.abs().max().try_into().unwrap();
        log::info!("max scaled |weight| = {max:.1}");
        let weights: Vec<i32> = weights.round().try_into().expect("out of range");
        for &w in &weights {
            encoder.encode_varint(w);
        }
    }
}

impl<F: Features> EvalModel for NnueModel<F> {
    type Features = F;
    type Config = Config;
    type LearnConfig = LearnConfig;

    fn new(features: F, vs: nn::Path, config: &Config) -> Self {
        let limit = (2.0 / features.approximate_avg_set()).sqrt();
        let embedding_path = &vs / "embedding";
        let embedding_weights = embedding_path.var(
            "weights",
            &[features.count() as i64, config.embedding_size],
            nn::Init::Uniform {
                lo: -limit,
                up: limit,
            },
        );
        let embedding_bias =
            embedding_path.var("bias", &[config.embedding_size], nn::Init::Const(0.0));

        let mut last_size = 2 * config.embedding_size;

        let mut hidden = Vec::with_capacity(config.hidden_sizes.len());
        for (index, &hidden_size) in config.hidden_sizes.iter().enumerate() {
            let limit = (2.0 / last_size as f64).sqrt();
            let layer = nn::linear(
                &vs / "hidden" / index,
                last_size,
                hidden_size,
                nn::LinearConfig {
                    ws_init: nn::Init::Uniform {
                        lo: -limit,
                        up: limit,
                    },
                    bs_init: Some(nn::Init::Const(0.0)),
                    bias: true,
                },
            );
            hidden.push(layer);
            last_size = hidden_size;
        }

        let limit = (2.0 / last_size as f64).sqrt();
        let final_layer = nn::linear(
            &vs / "final",
            last_size,
            1,
            nn::LinearConfig {
                ws_init: nn::Init::Uniform {
                    lo: -limit,
                    up: limit,
                },
                bs_init: Some(nn::Init::Const(0.0)),
                bias: true,
            },
        );

        let max_hidden_weight = 127.0 / (1u32 << config.hidden_weight_bits) as f64;
        let max_last_layer_weight = 127.0 * 127.0 / config.value_scale;

        Self {
            _features: features,
            config: config.clone(),
            embedding_weights,
            embedding_bias,
            hidden,
            final_layer,
            max_hidden_weight,
            max_last_layer_weight,
            activations: Vec::new(),
        }
    }

    fn forward(&mut self, features: &Tensor, offsets: &Tensor) -> Tensor {
        self.activations.clear();
        let (mut embedding, _, _, _) = Tensor::embedding_bag::<&Tensor>(
            &self.embedding_weights,
            features,
            &offsets.reshape([-1]),
            false, /* scale_grad_by_freq */
            0,     /* mode = sum */
            false, /* sparse */
            None,  /* per_sample_weights */
            false, /* include_last_offset */
        );
        // add bias
        embedding += &self.embedding_bias;
        embedding = embedding.clamp(0.0, 1.0);
        self.activations.push(embedding.shallow_clone());

        // embedding: [batch_size * 2, embedding_size]
        let mut x = embedding.reshape([-1, 2 * self.config.embedding_size]);
        // x: [batch_size, 2 * embedding_size]
        for hidden in &self.hidden {
            x = hidden.forward(&x);
            x = x.clamp(0.0, 1.0);
            self.activations.push(x.shallow_clone());
        }
        x = self.final_layer.forward(&x);
        // x: [batch_size, 1]
        x.squeeze_dim(1)
    }

    fn fixup(&mut self, learn_config: &Self::LearnConfig) {
        let _guard = tch::no_grad_guard();
        _ = self
            .embedding_weights
            .clamp_(-learn_config.max_embedding, learn_config.max_embedding);

        for hidden in &mut self.hidden {
            _ = hidden
                .ws
                .clamp_(-self.max_hidden_weight, self.max_hidden_weight);
        }

        _ = self
            .final_layer
            .ws
            .clamp_(-self.max_last_layer_weight, self.max_last_layer_weight);
    }

    fn num_layers(&self) -> usize {
        1 + self.hidden.len() + 1
    }

    fn layer_weights(&self, layer: usize) -> Tensor {
        if layer == 0 {
            self.embedding_weights.flatten(0, -1)
        } else if layer < 1 + self.hidden.len() {
            self.hidden[layer - 1].ws.flatten(0, -1)
        } else if layer == 1 + self.hidden.len() {
            self.final_layer.ws.flatten(0, -1)
        } else {
            panic!("layer out of range");
        }
    }

    fn activations(&self, layer: usize) -> Tensor {
        self.activations[layer].shallow_clone()
    }
}

impl<F: Features> Export for NnueModel<F> {
    type ExportConfig = ();

    fn export(&self, output: &Path, _export_config: &()) -> Result<(), Box<dyn Error>> {
        let _guard = tch::no_grad_guard();

        let mut f = BufWriter::new(File::create(output)?);
        writeln!(
            f,
            "pub const SCALE: f32 = {scale:.1};",
            scale = self.config.value_scale
        )?;
        writeln!(
            f,
            "pub const EMBEDDING_SIZE: usize = {};",
            self.config.embedding_size
        )?;
        write!(
            f,
            "pub const HIDDEN_SIZES: [usize; {}] = [",
            self.config.hidden_sizes.len()
        )?;
        for &size in &self.config.hidden_sizes {
            write!(f, "{size}, ")?;
        }
        writeln!(f, "];")?;
        writeln!(
            f,
            "pub const HIDDEN_WEIGHT_BITS: u32 = {};",
            self.config.hidden_weight_bits
        )?;
        let mut encoder = Base128Encoder::new();
        self.encode_tensor(&mut encoder, &self.embedding_weights, 127.0);
        self.encode_tensor(&mut encoder, &self.embedding_bias, 127.0);
        let weight_multiplier = f64::from(1u32 << self.config.hidden_weight_bits);
        for hidden in &self.hidden {
            self.encode_tensor(&mut encoder, &hidden.ws, weight_multiplier);
            self.encode_tensor(
                &mut encoder,
                hidden.bs.as_ref().unwrap(),
                127.0 * weight_multiplier,
            );
        }
        self.encode_tensor(
            &mut encoder,
            &self.final_layer.ws,
            self.config.value_scale / 127.0,
        );
        self.encode_tensor(
            &mut encoder,
            self.final_layer.bs.as_ref().unwrap(),
            self.config.value_scale,
        );
        let weights_str = encoder.finish();
        writeln!(f, "pub const WEIGHTS: &str = r\"{weights_str}\";")?;

        log::info!("Encoded weights in {} bytes", weights_str.len());
        log::info!("Exported NNUE to file {}", output.display());

        Ok(())
    }
}
