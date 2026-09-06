//! Cleaning a model command's output, and the index keys of questions.
use engrams::index::Index;
use engrams::questions::{parse_output, Store, MAX_QUESTIONS};

#[test]
fn model_output_is_cleaned_into_at_most_three_questions() {
    let raw = "\u{1b}[0m\n> build · some-model\n\u{1b}[0m\n1. Which database does the worktree use?\n- How does it connect to the database?\n\"Which port does the database listen on?\"\nA fourth question too many?\nshort\n";
    let q = parse_output(raw);
    assert_eq!(q.len(), MAX_QUESTIONS, "{q:?}");
    assert_eq!(q[0], "Which database does the worktree use?");
    assert_eq!(q[1], "How does it connect to the database?");
    assert_eq!(q[2], "Which port does the database listen on?");
}

#[test]
fn question_keys_point_to_their_chunk() {
    assert_eq!(Index::split_key("p/a.md#3"), ("p/a.md", 3));
    assert_eq!(Index::split_key("p/a.md#3?1"), ("p/a.md", 3));
    assert!(Index::is_question_key("p/a.md#3?1"));
    assert!(!Index::is_question_key("p/a.md#3"));
    assert_eq!(Index::split_key("no-hash"), ("no-hash", 0));
}

#[test]
fn the_store_round_trips_and_prunes() {
    let dir = std::env::temp_dir().join(format!("engram-questions-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("questions.json");
    let mut s = Store::load(&path);
    s.insert(42, vec!["Q1?".into(), "Q2?".into()]);
    s.insert(7, vec!["old?".into()]);
    s.save().unwrap();
    let mut back = Store::load(&path);
    assert_eq!(back.get(42).map(Vec::len), Some(2));
    let removed = back.prune(&[42u64].into_iter().collect());
    assert_eq!(removed, 1);
    assert!(back.get(7).is_none());
    std::fs::remove_dir_all(&dir).unwrap();
}
