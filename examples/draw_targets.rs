//! Draws benchmark targets and exposes ONLY the target passage.
//!
//! Blindness keeps the benchmark honest: an author who sees the title slips the
//! topic into the query and measures the heading again. Two families, drawn
//! separately: "note" targets the topic of a note, "detail" targets a passage in
//! the second half of its note, where an index by heading never looked.
//! Output: a JSON skeleton to fill in the `query` fields by hand.

use kept::chunking::{budget_for, split};
use kept::note::Note;
use std::collections::BTreeMap;

/// Reproducible generator (xorshift64): a draw that changes at every run compares nothing.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    fn below(&mut self, n: usize) -> usize {
        (self.next() % n.max(1) as u64) as usize
    }
}

fn main() {
    let root = kept::paths::root();
    let budget = budget_for(512);
    let per_family: usize = std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(30);
    let seed: u64 = std::env::args().nth(2).and_then(|s| s.parse().ok()).unwrap_or(0x5EED_1234_ABCD_0001);

    let mut notes: Vec<(String, String)> = Vec::new();
    let mut details: Vec<(String, usize, String)> = Vec::new();
    for file in kept::hot::notes_of(&root) {
        let Ok(content) = std::fs::read_to_string(&file) else { continue };
        let path = kept::paths::relative(&file, &root);
        let note = Note::parse(&content);
        let chunks = split(note.body(), budget);
        if chunks.is_empty() {
            continue;
        }
        notes.push((path.clone(), chunks[0].text.clone()));
        let half = chunks.len() / 2;
        for c in chunks.iter().skip(half.max(1)) {
            if c.text.len() > 200 {
                details.push((path.clone(), c.ordinal, c.text.clone()));
            }
        }
    }
    eprintln!("pools: {} notes, {} second-half passages", notes.len(), details.len());

    let mut rng = Rng(seed);
    let mut out: BTreeMap<String, Vec<serde_json::Value>> = BTreeMap::new();
    let draw = |rng: &mut Rng, n: usize, len: usize| -> Vec<usize> {
        let mut seen = std::collections::BTreeSet::new();
        let mut drawn = Vec::new();
        while drawn.len() < n.min(len) {
            let i = rng.below(len);
            if seen.insert(i) {
                drawn.push(i);
            }
        }
        drawn
    };
    let picked = draw(&mut rng, per_family, notes.len());
    out.insert(
        "note".into(),
        picked
            .iter()
            .map(|&i| serde_json::json!({"path": notes[i].0, "ordinal": 0, "passage": notes[i].1.chars().take(700).collect::<String>(), "query": ""}))
            .collect(),
    );
    let picked = draw(&mut rng, per_family, details.len());
    out.insert("detail".into(), picked.iter().map(|&i| serde_json::json!({"path": details[i].0, "ordinal": details[i].1, "passage": details[i].2.chars().take(700).collect::<String>(), "query": ""})).collect());
    println!("{}", serde_json::to_string_pretty(&out).unwrap());
}
