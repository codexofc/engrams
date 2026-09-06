//! Parallel encoding must return exactly the sequential vectors, in order: the core
//! count of a machine must not change the index.
use kept::embedder::Embedder;
use std::path::PathBuf;

fn model_dir() -> Option<PathBuf> {
    let d = kept::paths::model_dir();
    d.join("model.safetensors").exists().then_some(d)
}

#[test]
fn parallel_encoding_matches_sequential_and_keeps_order() {
    let Some(dir) = model_dir() else {
        eprintln!("model absent, test skipped; run `kept init`");
        return;
    };
    let embedder = Embedder::load(&dir).expect("model");
    let texts: Vec<String> = (0..16)
        .map(|i| format!("Paragraph {i}: the agents database listens on port 3334, and each worktree reaches it through DB_HOST set in the compose file."))
        .collect();
    let sequential = embedder.encode_many(&texts, 1).expect("sequential");
    let parallel = embedder.encode_many(&texts, 8).expect("parallel");
    assert_eq!(sequential.len(), 16);
    assert_eq!(parallel.len(), 16);
    for (a, b) in sequential.iter().zip(&parallel) {
        assert_eq!(a.len(), b.len());
        for (x, y) in a.iter().zip(b) {
            assert!((x - y).abs() < 1e-6, "{x} versus {y}");
        }
    }
    assert!(embedder.encode_many(&[], 4).expect("empty").is_empty());
}

#[test]
fn a_long_text_gives_a_finite_unit_vector() {
    // F16 weights returned an all-NaN vector past three hundred tokens while passing
    // the concordance on short sentences. This test keeps a real-sized paragraph.
    let Some(dir) = model_dir() else { return };
    let embedder = Embedder::load(&dir).expect("model");
    let long: String =
        (0..60).map(|i| format!("Paragraph {i}: the worktree points DB_HOST at the agents database on port 3334, database unit_tests. ")).collect();
    let (v, truncated) = embedder.encode_checked(&long).expect("encoding");
    assert!(v.iter().all(|x| x.is_finite()), "non-finite vector");
    let norm: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt();
    assert!((norm - 1.0).abs() < 1e-3, "norm {norm}");
    assert!(truncated, "sixty paragraphs exceed the window");
}
