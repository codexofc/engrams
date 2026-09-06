//! Two notes that say the same thing come out as one pair, never two chunks of one
//! note, never an archived note.
use kept::duplicates::near_duplicates;
use kept::index::{Header, Index};
use std::collections::HashSet;

fn header() -> Header {
    Header { model: "t".into(), weights_hash: "0".into(), dim: 3, pooling: "cls".into(), prompts: String::new() }
}

#[test]
fn near_identical_chunks_of_two_notes_make_one_pair() {
    let mut idx = Index::new(header());
    idx.push("a.md#0", 1, &[1.0, 0.0, 0.0]);
    idx.push("a.md#1", 1, &[0.99, 0.1, 0.0]); // close to a#0: same note, ignored
    idx.push("b.md#0", 2, &[0.98, 0.15, 0.0]); // close to a: one pair (a, b)
    idx.push("c.md#0", 3, &[0.0, 0.0, 1.0]); // far from everything
    idx.push("z.md#0", 4, &[1.0, 0.0, 0.0]); // identical to a, but archived
    let active: HashSet<String> = ["a.md", "b.md", "c.md"].iter().map(|s| s.to_string()).collect();
    let (compared, pairs) = near_duplicates(&idx, &active, 0.9);
    assert_eq!(compared, 4, "z.md is archived, its chunks do not count");
    assert_eq!(pairs.len(), 1);
    assert_eq!((pairs[0].1.as_str(), pairs[0].2.as_str()), ("a.md", "b.md"));
    assert!(pairs[0].0 > 0.98);
}
