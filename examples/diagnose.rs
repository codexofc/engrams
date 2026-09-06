//! Why a benchmark target is not found, without extra computation.
//!
//! Compare the target's score with the first result's. At the same level but left
//! out: an aggregation or identity defect. Clearly below while results share the
//! topic: dilution. At background level: the query or the language is the cause.

use souvenance::index::Index;
use souvenance::similarity::{cosine, rank_notes};
use std::path::PathBuf;

#[derive(serde::Deserialize)]
struct Case {
    path: String,
    ordinal: usize,
    query: String,
}

fn main() {
    let root = souvenance::paths::root();
    let state = souvenance::paths::state_dir(&root);
    let index = Index::load(&state.join("index.bin")).expect("index missing, run `souvenance index`");
    let embedder = souvenance::embedder::Embedder::load(&souvenance::paths::model_dir()).expect("model");
    let chunks: Vec<(String, usize, Vec<f32>)> = index
        .iter()
        .map(|(key, v)| {
            let (path, ordinal) = Index::split_key(key);
            (path.to_string(), ordinal, v.to_vec())
        })
        .collect();
    let bench = std::env::var("SOUVENANCE_BENCH").map(PathBuf::from).unwrap_or_else(|_| state.join("bench-queries.json"));
    let cases: std::collections::BTreeMap<String, Vec<Case>> = serde_json::from_str(&std::fs::read_to_string(bench).expect("queries")).unwrap();

    println!("{:<12} {:>5} {:>8} {:>8} {:>7} {:>6}  query", "family", "found", "target", "top", "gap", "chars");
    for (family, list) in &cases {
        for case in list {
            let q = embedder.encode_query(&case.query).expect("encoding");
            let hits = rank_notes(&q, &chunks, 5);
            let found = hits.iter().any(|h| h.path == case.path);
            let target = chunks.iter().find(|(p, o, _)| *p == case.path && *o == case.ordinal).map(|(_, _, v)| cosine(&q, v)).unwrap_or(f32::NAN);
            let top = hits.first().map(|h| h.score).unwrap_or(0.0);
            let target_len = std::fs::read_to_string(root.join(&case.path))
                .ok()
                .and_then(|content| {
                    let note = souvenance::note::Note::parse(&content);
                    souvenance::chunking::split(note.body(), souvenance::chunking::budget_for(embedder.window()))
                        .into_iter()
                        .find(|c| c.ordinal == case.ordinal)
                        .map(|c| c.text.chars().count())
                })
                .unwrap_or(0);
            println!(
                "{family:<12} {:>5} {target:>8.3} {top:>8.3} {:>7.3} {target_len:>6}  {}",
                if found { "yes" } else { "NO" },
                top - target,
                case.query.chars().take(52).collect::<String>()
            );
        }
    }
}
