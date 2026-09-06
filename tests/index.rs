//! The index is derived and disposable. What must be solid is that it refuses to
//! serve vectors produced by another model or for a file that changed.

use engrams::index::{Header, Index};

fn header() -> Header {
    Header { model: "model-a".into(), weights_hash: "abc123".into(), dim: 3, pooling: "cls".into(), prompts: String::new() }
}

#[test]
fn a_new_index_is_empty() {
    assert_eq!(Index::new(header()).len(), 0);
}

#[test]
fn a_pushed_vector_is_retrievable_by_its_path() {
    // A note's identity is its PATH, not its `name`: two projects may share a name.
    let mut idx = Index::new(header());
    idx.push("fam/one/a.md", 42, &[1.0, 0.0, 0.0]);
    assert_eq!(idx.vector("fam/one/a.md"), Some(&[1.0, 0.0, 0.0][..]));
}

#[test]
fn two_notes_with_the_same_name_in_different_projects_coexist() {
    let mut idx = Index::new(header());
    idx.push("fam/one/shared.md", 1, &[1.0, 0.0, 0.0]);
    idx.push("fam/two/shared.md", 2, &[0.0, 1.0, 0.0]);
    assert_eq!(idx.len(), 2);
    assert_eq!(idx.vector("fam/two/shared.md"), Some(&[0.0, 1.0, 0.0][..]));
}

#[test]
fn pushing_the_same_path_twice_replaces_the_vector() {
    let mut idx = Index::new(header());
    idx.push("a.md", 1, &[1.0, 0.0, 0.0]);
    idx.push("a.md", 2, &[0.0, 1.0, 0.0]);
    assert_eq!(idx.len(), 1);
    assert_eq!(idx.vector("a.md"), Some(&[0.0, 1.0, 0.0][..]));
}

#[test]
fn a_vector_of_the_wrong_dimension_is_refused() {
    let mut idx = Index::new(header());
    assert!(idx.try_push("a.md", 1, &[1.0, 0.0]).is_err());
}

#[test]
fn an_unchanged_file_is_recognised_as_fresh() {
    let mut idx = Index::new(header());
    idx.push("a.md", 4242, &[1.0, 0.0, 0.0]);
    assert!(idx.is_fresh("a.md", 4242));
}

#[test]
fn a_changed_file_is_not_fresh() {
    let mut idx = Index::new(header());
    idx.push("a.md", 4242, &[1.0, 0.0, 0.0]);
    assert!(!idx.is_fresh("a.md", 9999));
}

#[test]
fn an_unknown_file_is_not_fresh() {
    assert!(!Index::new(header()).is_fresh("never-seen.md", 1));
}

#[test]
fn an_index_survives_a_round_trip() {
    let mut idx = Index::new(header());
    idx.push("fam/one/a.md", 7, &[0.6, 0.8, 0.0]);
    idx.push("personal/b.md", 8, &[0.0, 0.0, 1.0]);
    let reloaded = Index::from_json(&idx.to_json()).unwrap();
    assert_eq!(reloaded.len(), 2);
    assert_eq!(reloaded.vector("fam/one/a.md"), Some(&[0.6, 0.8, 0.0][..]));
    assert!(reloaded.is_fresh("personal/b.md", 8));
}

#[test]
fn an_index_built_with_another_model_is_refused() {
    // Mixing vectors of two models is the worst case: scores stay in range and
    // the ranking is noise.
    let idx = Index::new(header());
    let other = Header { model: "model-b".into(), ..header() };
    assert!(!Index::from_json(&idx.to_json()).unwrap().matches(&other));
}

#[test]
fn an_index_built_with_another_pooling_is_refused() {
    let idx = Index::new(header());
    let other = Header { pooling: "mean".into(), ..header() };
    assert!(!Index::from_json(&idx.to_json()).unwrap().matches(&other));
}

#[test]
fn an_index_built_with_the_same_header_is_accepted() {
    let idx = Index::new(header());
    assert!(Index::from_json(&idx.to_json()).unwrap().matches(&header()));
}

#[test]
fn save_leaves_no_temporary_file_and_round_trips() {
    let dir = std::env::temp_dir().join(format!("engram-index-test-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("index.bin");
    let mut idx = Index::new(header());
    idx.push("a.md#0", 1, &[1.0, 0.0, 0.0]);
    idx.save(&path).unwrap();
    assert!(path.exists());
    assert!(!path.with_extension("tmp").exists(), "the temporary file must have been renamed");
    let back = Index::load(&path).unwrap();
    assert_eq!(back.vector("a.md#0"), Some(&[1.0, 0.0, 0.0][..]));
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn retain_drops_ghost_entries_and_keeps_the_others_addressable() {
    let mut idx = Index::new(header());
    idx.push("a.md#0", 1, &[1.0, 0.0, 0.0]);
    idx.push("a.md#1", 1, &[0.0, 1.0, 0.0]);
    idx.push("b.md#0", 2, &[0.0, 0.0, 1.0]);
    let removed = idx.retain(|k| k != "a.md#1");
    assert_eq!(removed, 1);
    assert_eq!(idx.len(), 2);
    assert_eq!(idx.vector("a.md#1"), None);
    assert_eq!(idx.vector("b.md#0"), Some(&[0.0, 0.0, 1.0][..]));
    assert!(idx.is_fresh("b.md#0", 2));
}

#[test]
fn a_non_finite_vector_is_refused() {
    let mut idx = Index::new(header());
    let err = idx.try_push("a.md#0", 1, &[1.0, f32::NAN, 0.0]).unwrap_err();
    assert!(err.contains("non-finite"), "{err}");
    assert!(idx.is_empty());
}

#[test]
fn the_old_json_format_is_still_readable() {
    let dir = std::env::temp_dir().join(format!("engram-index-json-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("index.json");
    let mut idx = Index::new(header());
    idx.push("a.md#0", 7, &[0.0, 1.0, 0.0]);
    std::fs::write(&path, idx.to_json()).unwrap();
    let back = Index::load(&path).unwrap();
    assert!(back.is_fresh("a.md#0", 7));
    std::fs::remove_dir_all(&dir).unwrap();
}
