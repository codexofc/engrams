//! Pooling is read from the model and tested: mean-pooling a `cls` model silently
//! costs fourteen points of recall.

use candle_core::{Device, Tensor};
use souvenance::pooling::{pool, Pooling};

fn hidden(values: &[[f32; 3]]) -> Tensor {
    let flat: Vec<f32> = values.iter().flatten().copied().collect();
    Tensor::from_vec(flat, (1, values.len(), 3), &Device::Cpu).unwrap()
}

fn mask(bits: &[u32]) -> Tensor {
    Tensor::from_vec(bits.to_vec(), (1, bits.len()), &Device::Cpu).unwrap()
}

#[test]
fn cls_pooling_takes_the_first_token() {
    let v = pool(&hidden(&[[1.0, 2.0, 3.0], [9.0, 9.0, 9.0]]), &mask(&[1, 1]), Pooling::Cls).unwrap();
    assert_eq!(v.to_vec2::<f32>().unwrap()[0], vec![1.0, 2.0, 3.0]);
}

#[test]
fn mean_pooling_averages_the_tokens() {
    let v = pool(&hidden(&[[0.0, 0.0, 0.0], [2.0, 4.0, 6.0]]), &mask(&[1, 1]), Pooling::Mean).unwrap();
    assert_eq!(v.to_vec2::<f32>().unwrap()[0], vec![1.0, 2.0, 3.0]);
}

#[test]
fn mean_pooling_ignores_padding() {
    let v = pool(&hidden(&[[2.0, 4.0, 6.0], [100.0, 100.0, 100.0]]), &mask(&[1, 0]), Pooling::Mean).unwrap();
    assert_eq!(v.to_vec2::<f32>().unwrap()[0], vec![2.0, 4.0, 6.0]);
}

#[test]
fn a_fully_masked_sequence_does_not_divide_by_zero() {
    let v = pool(&hidden(&[[1.0, 1.0, 1.0]]), &mask(&[0]), Pooling::Mean).unwrap();
    assert!(v.to_vec2::<f32>().unwrap()[0].iter().all(|x| x.is_finite()));
}

#[test]
fn pooling_is_read_from_the_model_configuration() {
    let conf = r#"{"pooling_mode_cls_token": true, "pooling_mode_mean_tokens": false}"#;
    assert_eq!(Pooling::from_config(conf).unwrap(), Pooling::Cls);
}

#[test]
fn mean_pooling_is_read_from_the_model_configuration() {
    let conf = r#"{"pooling_mode_cls_token": false, "pooling_mode_mean_tokens": true}"#;
    assert_eq!(Pooling::from_config(conf).unwrap(), Pooling::Mean);
}

#[test]
fn an_unknown_pooling_configuration_is_refused() {
    assert!(Pooling::from_config(r#"{"pooling_mode_lasttoken": true}"#).is_err());
}
