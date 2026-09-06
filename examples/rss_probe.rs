//! What weighs what in a warm process: resident memory after each step.
fn rss_mb() -> f64 {
    let out = std::process::Command::new("ps").args(["-o", "rss=", "-p", &std::process::id().to_string()]).output().unwrap();
    String::from_utf8_lossy(&out.stdout).trim().parse::<f64>().unwrap_or(0.0) / 1024.0
}

fn main() {
    let root = souvenance::paths::root();
    let model = souvenance::paths::model_dir();
    let mut prev = rss_mb();
    let mut step = |label: &str| {
        let now = rss_mb();
        println!("{label:<44} {now:>7.0} MB  (+{:.0})", now - prev);
        prev = now;
    };
    step("start");
    let e = souvenance::embedder::Embedder::load(&model).unwrap();
    step("model loaded (tokenizer, Q8 layers)");
    let _ = e.encode("a medium sized text to see the activations, with DB_HOST and release-v2 in it").unwrap();
    step("after one encoding");
    let long: String = (0..60).map(|i| format!("Paragraph {i}: the worktree points DB_HOST at the agents database on port 3334. ")).collect();
    let _ = e.encode(&long).unwrap();
    step("after a long encoding (512 tokens)");
    let idx = souvenance::index::Index::load(&souvenance::paths::state_dir(&root).join("index.bin")).expect("index missing, run `souvenance index`");
    step("index loaded");
    let chunks: Vec<(String, usize, Vec<f32>)> = idx.iter().map(|(k, v)| (k.to_string(), 0, v.to_vec())).collect();
    step("chunk copy for ranking");
    drop(chunks);
    step("copy released");
}
