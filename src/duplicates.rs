//! Near-duplicate notes, read from the index. Two active notes whose chunks are
//! almost identical in vector space say the same thing twice. The index is enough,
//! the model is not loaded.

use std::collections::{HashMap, HashSet};

use crate::index::Index;
use crate::similarity::cosine;

/// A pair of notes and the maximal cosine between their chunks.
pub type Pair = (f32, String, String);

/// Pairs of notes among `active` with two chunks above `threshold`, closest first.
/// Also returns how many chunks were compared, so an empty result can be told from
/// an index that does not cover the corpus.
pub fn near_duplicates(index: &Index, active: &HashSet<String>, threshold: f32) -> (usize, Vec<Pair>) {
    // Generated questions look alike by nature: only text chunks say whether two
    // notes say the same thing.
    let chunks: Vec<(&str, &[f32])> = index
        .iter()
        .filter(|(key, _)| !Index::is_question_key(key))
        .map(|(key, v)| (Index::split_key(key).0, v))
        .filter(|(p, _)| active.contains(*p))
        .collect();
    let mut best: HashMap<(&str, &str), f32> = HashMap::new();
    for (i, (pa, va)) in chunks.iter().enumerate() {
        for (pb, vb) in &chunks[i + 1..] {
            if pa == pb {
                continue;
            }
            let s = cosine(va, vb);
            if s < threshold {
                continue;
            }
            let key = if pa < pb { (*pa, *pb) } else { (*pb, *pa) };
            let e = best.entry(key).or_insert(0.0);
            if s > *e {
                *e = s;
            }
        }
    }
    let mut out: Vec<Pair> = best.into_iter().map(|((a, b), s)| (s, a.to_string(), b.to_string())).collect();
    out.sort_by(|x, y| y.0.total_cmp(&x.0).then_with(|| x.1.cmp(&y.1)));
    (chunks.len(), out)
}
