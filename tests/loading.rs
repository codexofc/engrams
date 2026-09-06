//! Loading a model from a local directory. No download: the directory is given, or
//! loading fails naming the missing file.

use souvenance::embedder::Embedder;
use std::fs;
use std::path::PathBuf;

fn scratch_dir(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("souvenance-test-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&d);
    fs::create_dir_all(&d).unwrap();
    d
}

const CONFIG: &str = r#"{"architectures":["ModernBertModel"],"max_position_embeddings":8192,"hidden_size":384}"#;
const POOLING: &str = r#"{"pooling_mode_cls_token": true, "pooling_mode_mean_tokens": false}"#;

#[test]
fn an_empty_directory_names_the_missing_file() {
    let d = scratch_dir("empty");
    let error = Embedder::load(&d).unwrap_err();
    assert!(error.contains("config.json"), "the error must name the missing file, got: {error}");
}

#[test]
fn a_missing_pooling_config_is_named() {
    let d = scratch_dir("no-pooling");
    fs::write(d.join("config.json"), CONFIG).unwrap();
    let error = Embedder::load(&d).unwrap_err();
    assert!(error.contains("1_Pooling"), "got: {error}");
}

#[test]
fn a_missing_tokenizer_is_named() {
    let d = scratch_dir("no-tokenizer");
    fs::write(d.join("config.json"), CONFIG).unwrap();
    fs::create_dir_all(d.join("1_Pooling")).unwrap();
    fs::write(d.join("1_Pooling/config.json"), POOLING).unwrap();
    let error = Embedder::load(&d).unwrap_err();
    assert!(error.contains("tokenizer.json"), "got: {error}");
}

#[test]
fn a_missing_weights_file_is_named() {
    let d = scratch_dir("no-weights");
    fs::write(d.join("config.json"), CONFIG).unwrap();
    fs::create_dir_all(d.join("1_Pooling")).unwrap();
    fs::write(d.join("1_Pooling/config.json"), POOLING).unwrap();
    fs::write(d.join("tokenizer.json"), "{}").unwrap();
    let error = Embedder::load(&d).unwrap_err();
    assert!(error.contains("safetensors"), "got: {error}");
}

#[test]
fn an_unsupported_architecture_is_refused_before_reading_the_weights() {
    let d = scratch_dir("unknown-arch");
    fs::write(d.join("config.json"), r#"{"architectures":["LlamaForCausalLM"],"max_position_embeddings":4096,"hidden_size":4096}"#).unwrap();
    let error = Embedder::load(&d).unwrap_err();
    assert!(error.contains("architecture"), "got: {error}");
}
