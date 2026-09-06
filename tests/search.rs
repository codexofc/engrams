//! Ranking is what the agent sees. A good note in ninth position is buried, not found.

use kept::similarity::rank_notes;
use kept::{cosine, rank, Hit};

#[test]
fn identical_vectors_have_a_cosine_of_one() {
    let v = vec![0.3, 0.4, 0.5];
    assert!((cosine(&v, &v) - 1.0).abs() < 1e-6);
}

#[test]
fn orthogonal_vectors_have_a_null_cosine() {
    assert!(cosine(&[1.0, 0.0], &[0.0, 1.0]).abs() < 1e-6);
}

#[test]
fn magnitude_does_not_change_the_cosine() {
    assert!((cosine(&[1.0, 2.0, 3.0], &[10.0, 20.0, 30.0]) - 1.0).abs() < 1e-6);
}

#[test]
fn a_null_vector_does_not_divide_by_zero() {
    assert_eq!(cosine(&[0.0, 0.0], &[1.0, 1.0]), 0.0);
}

#[test]
fn ranking_orders_from_nearest_to_farthest() {
    let query = vec![1.0, 0.0];
    let corpus = vec![("farthest".to_string(), vec![0.0, 1.0]), ("nearest".to_string(), vec![1.0, 0.1]), ("middle".to_string(), vec![1.0, 1.0])];
    let hits = rank(&query, &corpus, 3);
    let names: Vec<&str> = hits.iter().map(|h| h.name.as_str()).collect();
    assert_eq!(names, vec!["nearest", "middle", "farthest"]);
}

#[test]
fn ranking_stops_at_the_requested_limit() {
    let query = vec![1.0, 0.0];
    let corpus: Vec<(String, Vec<f32>)> = (0..20).map(|i| (format!("n{i}"), vec![1.0, i as f32 / 20.0])).collect();
    assert_eq!(rank(&query, &corpus, 5).len(), 5);
}

#[test]
fn ranking_carries_each_score() {
    let query = vec![1.0, 0.0];
    let corpus = vec![("identical".to_string(), vec![1.0, 0.0])];
    let hits: Vec<Hit> = rank(&query, &corpus, 1);
    assert_eq!(hits[0].name, "identical");
    assert!((hits[0].score - 1.0).abs() < 1e-6);
}

#[test]
fn an_empty_corpus_ranks_to_nothing() {
    assert!(rank(&[1.0, 0.0], &[], 5).is_empty());
}

// Search ranks chunks but must return NOTES: three chunks of one note must not
// take three of the five places.

#[test]
fn three_chunks_of_one_note_take_a_single_place() {
    let query = vec![1.0, 0.0];
    let chunks = vec![
        ("a.md".to_string(), 0, vec![0.9, 0.1]),
        ("a.md".to_string(), 1, vec![1.0, 0.0]),
        ("a.md".to_string(), 2, vec![0.8, 0.2]),
        ("b.md".to_string(), 0, vec![0.5, 0.5]),
    ];
    let notes = rank_notes(&query, &chunks, 5);
    assert_eq!(notes.len(), 2);
    assert_eq!(notes[0].path, "a.md");
}

#[test]
fn a_note_scores_as_high_as_its_best_chunk() {
    // Maximum, not mean: a note whose ONE passage answers exactly is a good answer.
    let query = vec![1.0, 0.0];
    let chunks = vec![("a.md".to_string(), 0, vec![0.0, 1.0]), ("a.md".to_string(), 1, vec![1.0, 0.0])];
    assert!((rank_notes(&query, &chunks, 5)[0].score - 1.0).abs() < 1e-6);
}

#[test]
fn a_note_reports_which_chunk_answered() {
    let query = vec![1.0, 0.0];
    let chunks = vec![("a.md".to_string(), 0, vec![0.0, 1.0]), ("a.md".to_string(), 7, vec![1.0, 0.0])];
    assert_eq!(rank_notes(&query, &chunks, 5)[0].ordinal, 7);
}

#[test]
fn note_ranking_respects_the_limit() {
    let query = vec![1.0, 0.0];
    let chunks: Vec<(String, usize, Vec<f32>)> = (0..20).map(|i| (format!("n{i}.md"), 0, vec![1.0, i as f32 / 20.0])).collect();
    assert_eq!(rank_notes(&query, &chunks, 5).len(), 5);
}

#[test]
fn notes_are_ordered_by_their_best_chunk() {
    let query = vec![1.0, 0.0];
    let chunks = vec![("far.md".to_string(), 0, vec![0.1, 0.9]), ("near.md".to_string(), 0, vec![1.0, 0.0])];
    let notes = rank_notes(&query, &chunks, 5);
    assert_eq!(notes[0].path, "near.md");
    assert_eq!(notes[1].path, "far.md");
}
