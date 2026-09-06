//! The configuration decides which architecture to load and where the window ends.

use engrams::model::{Architecture, ModelConfig};

const XLMR: &str = r#"{"architectures":["XLMRobertaModel"],"max_position_embeddings":514,"hidden_size":768}"#;

#[test]
fn reads_the_context_window() {
    // Two positions are reserved for the start and end markers.
    assert_eq!(ModelConfig::parse(XLMR).unwrap().max_tokens, 512);
}

#[test]
fn reads_the_hidden_size() {
    assert_eq!(ModelConfig::parse(XLMR).unwrap().hidden_size, 768);
}

#[test]
fn recognises_xlm_roberta() {
    assert_eq!(ModelConfig::parse(XLMR).unwrap().architecture, Architecture::XlmRoberta);
}

#[test]
fn recognises_modernbert() {
    let conf = r#"{"architectures":["ModernBertModel"],"max_position_embeddings":8192,"hidden_size":384}"#;
    assert_eq!(ModelConfig::parse(conf).unwrap().architecture, Architecture::ModernBert);
}

#[test]
fn an_unsupported_architecture_is_refused() {
    let conf = r#"{"architectures":["LlamaForCausalLM"],"max_position_embeddings":4096,"hidden_size":4096}"#;
    assert!(ModelConfig::parse(conf).is_err());
}

#[test]
fn a_short_text_is_not_truncated() {
    let conf = ModelConfig { architecture: Architecture::ModernBert, max_tokens: 8192, hidden_size: 384 };
    assert!(!conf.would_truncate(1200));
}

#[test]
fn a_text_beyond_the_window_is_reported_as_truncated() {
    let conf = ModelConfig { architecture: Architecture::XlmRoberta, max_tokens: 512, hidden_size: 768 };
    assert!(conf.would_truncate(913));
}
