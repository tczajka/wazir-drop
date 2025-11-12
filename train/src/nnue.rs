use crate::model::EvalModel;
use serde::Deserialize;
use tch::{
    TchError, Tensor,
    nn::{self, Module, OptimizerConfig},
};
use wazir_drop::Features;

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    learning_rate: f64,
    embedding_size: i64,
    hidden_sizes: Vec<i64>,
    max_embedding: f64,
    hidden_weight_bits: u32,
}

#[derive(Debug)]
pub struct NnueModel<F: Features> {
    _features: F,
    config: Config,
    embedding: nn::Linear,
    hidden: Vec<nn::Linear>,
    final_layer: nn::Linear,
    max_hidden_weight: f64,
}

impl<F: Features> EvalModel for NnueModel<F> {
    type Features = F;
    type Config = Config;

    fn new(features: F, vs: nn::Path, config: &Config) -> Self {
        let limit = (2.0 / features.approximate_avg_set()).sqrt();
        let embedding = nn::linear(
            &vs / "embedding",
            features.count() as i64,
            config.embedding_size,
            nn::LinearConfig {
                ws_init: nn::Init::Uniform {
                    lo: -limit,
                    up: limit,
                },
                bs_init: Some(nn::Init::Const(0.0)),
                bias: true,
            },
        );

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

        Self {
            _features: features,
            config: config.clone(),
            embedding,
            hidden,
            final_layer,
            max_hidden_weight,
        }
    }

    fn forward(&self, features: &Tensor, offsets: &Tensor) -> Tensor {
        let (embedding, _, _, _) = Tensor::embedding_bag::<&Tensor>(
            &self.embedding.ws,
            features,
            &offsets.reshape([-1]),
            false, /* scale_grad_by_freq */
            0,     /* mode = sum */
            false, /* sparse */
            None,  /* per_sample_weights */
            false, /* include_last_offset */
        );
        // embedding: [batch_size * 2, embedding_size]
        let embedding = embedding.reshape([-1, 2, self.config.embedding_size]);
        // embedding: [batch_size, 2, embedding_size]
        let mut x = &embedding + self.embedding.bs.as_ref().unwrap();
        for hidden in &self.hidden {
            x = hidden.forward(&x);
        }
        x = self.final_layer.forward(&x);
        // x: [batch_size, 1]
        x.unsqueeze(1)
    }

    fn optimizer(&self, vs: &nn::VarStore) -> Result<nn::Optimizer, TchError> {
        nn::Adam::default().build(vs, self.config.learning_rate)
    }

    fn fixup(&mut self) {
        let _guard = tch::no_grad_guard();
        _ = self
            .embedding
            .ws
            .clamp_(-self.config.max_embedding, self.config.max_embedding);

        for hidden in &mut self.hidden {
            _ = hidden
                .ws
                .clamp_(-self.max_hidden_weight, self.max_hidden_weight);
        }
    }
}
