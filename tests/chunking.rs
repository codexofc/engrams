//! Splitting follows the markdown structure and never cuts inside a paragraph.

use souvenance::chunking::{split, Chunk};

fn note(body: &str) -> Vec<Chunk> {
    split(body, 300)
}

#[test]
fn a_short_note_yields_a_single_chunk() {
    let chunks = note("A short paragraph.\n");
    assert_eq!(chunks.len(), 1);
    assert_eq!(chunks[0].text.trim(), "A short paragraph.");
}

#[test]
fn an_empty_note_yields_nothing() {
    assert!(note("").is_empty());
    assert!(note("   \n\n  \n").is_empty());
}

#[test]
fn a_heading_starts_a_new_chunk() {
    // A heading marks a change of topic.
    let chunks = note("Before the heading.\n\n## A heading\n\nAfter the heading.\n");
    assert_eq!(chunks.len(), 2);
    assert!(chunks[0].text.contains("Before"));
    assert!(chunks[1].text.contains("A heading"));
    assert!(chunks[1].text.contains("After"));
}

#[test]
fn paragraphs_merge_until_the_budget_is_reached() {
    let short = "Short.\n\nShort.\n\nShort.\n";
    assert_eq!(split(short, 300).len(), 1);
}

#[test]
fn a_paragraph_is_never_split_even_when_it_exceeds_the_budget() {
    // The budget is a target, not a hard limit: a halved idea gives two vectors that
    // mean nothing.
    let long = format!("{}\n", "word ".repeat(500));
    assert_eq!(split(&long, 50).len(), 1, "a lone paragraph is never cut");
}

#[test]
fn a_chunk_knows_where_it_starts_in_the_note() {
    let chunks = note("First.\n\n## Heading\n\nSecond.\n");
    assert_eq!(chunks[0].start, 0);
    assert!(chunks[1].start > 0);
    assert!(chunks[1].start < "First.\n\n## Heading\n\nSecond.\n".len());
}

#[test]
fn chunks_are_numbered_in_reading_order() {
    let chunks = note("A.\n\n## T1\n\nB.\n\n## T2\n\nC.\n");
    let order: Vec<usize> = chunks.iter().map(|c| c.ordinal).collect();
    assert_eq!(order, vec![0, 1, 2]);
}

#[test]
fn a_chunk_carries_its_note_context_when_asked() {
    // The frontmatter IS the context: a paragraph about `DB_HOST` that never names
    // its note stays findable once it gets its title back.
    let c = Chunk { ordinal: 0, start: 0, text: "The worktree points DB_HOST at the agents database.".into() };
    let prefixed = c.with_context("agent-pipeline", "Agent pipeline on two repositories");
    assert!(prefixed.starts_with("agent-pipeline"));
    assert!(prefixed.contains("Agent pipeline"));
    assert!(prefixed.contains("DB_HOST"));
}

#[test]
fn a_note_without_description_still_gets_its_name() {
    let c = Chunk { ordinal: 0, start: 0, text: "body".into() };
    let prefixed = c.with_context("name-only", "");
    assert!(prefixed.starts_with("name-only"));
    assert!(prefixed.contains("body"));
}

#[test]
fn the_budget_is_derived_from_the_model_window() {
    assert_eq!(souvenance::chunking::budget_for(512), 450);
    assert_eq!(souvenance::chunking::budget_for(8192), 8130);
}

#[test]
fn a_tiny_window_still_leaves_room_for_content() {
    assert!(souvenance::chunking::budget_for(64) > 0);
}
