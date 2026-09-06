//! Fidelity of Q8 to F32 on real paragraphs of the corpus: cosine between the two
//! vectors, per length bucket. This is the measurement that justifies Q8 by default.
use engrams::chunking::{budget_for, split};
use engrams::embedder::Embedder;
use engrams::note::Note;
use engrams::similarity::cosine;

fn main() {
    let root = engrams::paths::root();
    let model = engrams::paths::model_dir();
    std::env::set_var("ENGRAM_PRECISION", "f32");
    let f32e = Embedder::load(&model).unwrap();
    std::env::set_var("ENGRAM_PRECISION", "q8");
    let q8e = Embedder::load(&model).unwrap();
    let step: usize = std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(8);
    let mut texts = Vec::new();
    for f in engrams::hot::notes_of(&root) {
        let Ok(content) = std::fs::read_to_string(&f) else { continue };
        let note = Note::parse(&content);
        let (name, desc) = (note.field("name").unwrap_or_default(), note.field("description").unwrap_or_default());
        for c in split(note.body(), budget_for(512)) {
            texts.push(c.with_context(name, desc));
        }
    }
    let sample: Vec<&String> = texts.iter().step_by(step).collect();
    let mut buckets: Vec<(usize, usize, Vec<f32>)> = vec![(0, 64, vec![]), (64, 128, vec![]), (128, 256, vec![]), (256, 513, vec![])];
    for t in &sample {
        let n = f32e.token_count(t);
        let c = cosine(&f32e.encode(t).unwrap(), &q8e.encode(t).unwrap());
        if let Some(b) = buckets.iter_mut().find(|(lo, hi, _)| n >= *lo && n < *hi) {
            b.2.push(c);
        }
    }
    println!("{} paragraphs of {} (one in {step})", sample.len(), texts.len());
    println!("{:<12} {:>4} {:>9} {:>9} {:>9}", "tokens", "n", "min", "median", "mean");
    let mut all = Vec::new();
    for (lo, hi, mut v) in buckets {
        if v.is_empty() {
            continue;
        }
        v.sort_by(|a, b| a.total_cmp(b));
        let mean = v.iter().sum::<f32>() / v.len() as f32;
        println!("{:<12} {:>4} {:>9.5} {:>9.5} {:>9.5}", format!("{lo}..{}", hi - 1), v.len(), v[0], v[v.len() / 2], mean);
        all.extend(v);
    }
    all.sort_by(|a, b| a.total_cmp(b));
    println!("{:<12} {:>4} {:>9.5} {:>9.5} {:>9.5}", "all", all.len(), all[0], all[all.len() / 2], all.iter().sum::<f32>() / all.len() as f32);
}
