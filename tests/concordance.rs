//! The most important test of the project.
//!
//! It compares this binary's output with frozen reference vectors produced once by
//! an independent implementation. It is the only test that catches at once a wrong
//! pooling, a forgotten layer, a missing normalisation, a different truncation or a
//! tokenizer drift: each of those yields plausible, wrong vectors.
//!
//! Skipped when the model is absent, so the suite runs on a bare machine and in CI.

use kept::embedder::Embedder;
use kept::similarity::cosine;
use std::path::PathBuf;

fn model_path() -> Option<PathBuf> {
    let d = kept::paths::model_dir();
    d.join("model.safetensors").exists().then_some(d)
}

struct Case {
    text: String,
    vector: Vec<f32>,
}

fn reference() -> std::collections::BTreeMap<String, Case> {
    let raw = include_str!("fixtures/reference-granite-278m.json");
    serde_json::from_str(raw).expect("unreadable reference fixture")
}

impl<'de> serde::Deserialize<'de> for Case {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(serde::Deserialize)]
        struct Raw {
            text: String,
            vector: Vec<f32>,
        }
        let r = Raw::deserialize(d)?;
        Ok(Case { text: r.text, vector: r.vector })
    }
}

#[test]
fn encoding_matches_the_reference_implementation() {
    let Some(dir) = model_path() else {
        eprintln!("model absent, test skipped; run `kept init`");
        return;
    };
    let embedder = Embedder::load(&dir).expect("model");
    for (name, case) in reference() {
        let got = embedder.encode(&case.text).expect("encoding");
        assert_eq!(got.len(), case.vector.len(), "case {name}: unexpected dimension");
        let c = cosine(&got, &case.vector);
        assert!(c > 0.999, "case {name}: cosine {c:.6} with the reference, below 0.999. A pooling, a layer or the tokenizer differs.");
    }
}

#[test]
fn encoding_is_deterministic() {
    let Some(dir) = model_path() else { return };
    let embedder = Embedder::load(&dir).expect("model");
    let text = "Token rotation happens every ninety days.";
    assert_eq!(embedder.encode(text).unwrap(), embedder.encode(text).unwrap());
}

#[test]
fn encoding_returns_a_normalised_vector() {
    let Some(dir) = model_path() else { return };
    let embedder = Embedder::load(&dir).expect("model");
    let v = embedder.encode("Token rotation happens every ninety days.").unwrap();
    let norm: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt();
    assert!((norm - 1.0).abs() < 1e-4, "norm {norm}, expected 1.0");
}

#[test]
fn truncation_is_detected_on_a_text_beyond_the_window() {
    let Some(dir) = model_path() else { return };
    let embedder = Embedder::load(&dir).expect("model");
    assert!(embedder.would_truncate(&"word ".repeat(3000)));
}

#[test]
fn a_short_text_is_not_reported_as_truncated() {
    let Some(dir) = model_path() else { return };
    let embedder = Embedder::load(&dir).expect("model");
    assert!(!embedder.would_truncate("A short note."));
}

/// The ModernBERT graph, written after the reference implementation, must match
/// its reference vectors too. The library graph hard-codes a GELU activation and
/// stalled at 0.85 on this model, which declares SiLU.
#[test]
fn the_modernbert_model_matches_its_reference() {
    let dir = kept::paths::alt_model_dir();
    if !dir.join("model.safetensors").exists() {
        eprintln!("ModernBERT model absent, test skipped; run `kept init --model {}`", kept::paths::ALT_MODEL_REPO);
        return;
    }
    let embedder = Embedder::load(&dir).expect("model");
    let raw = include_str!("fixtures/reference-granite-97m-multilingual-r2.json");
    let reference: std::collections::BTreeMap<String, Case> = serde_json::from_str(raw).expect("unreadable fixture");
    for (name, case) in reference {
        let got = embedder.encode(&case.text).expect("encoding");
        let c = cosine(&got, &case.vector);
        assert!(c > 0.999, "case {name}: cosine {c:.6}, below 0.999");
    }
    assert!(!embedder.would_truncate(&"word ".repeat(1500)), "the long-context model keeps 1 500 words");
}
