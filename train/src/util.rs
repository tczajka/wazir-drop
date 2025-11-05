use tch::{Tensor, kind};

pub fn sparse_1d_tensor(
    features: impl Iterator<Item = (usize, i32)>,
    num_features: usize,
) -> Tensor {
    let mut keys: Vec<i64> = Vec::new();
    let mut values: Vec<f32> = Vec::new();
    for (key, value) in features {
        keys.push(key as i64);
        values.push(value as f32);
    }
    Tensor::sparse_coo_tensor_indices_size(
        &Tensor::from_slice(&keys).unsqueeze(0),
        &Tensor::from_slice(&values),
        [num_features as i64],
        kind::FLOAT_CPU,
        false,
    )
}
