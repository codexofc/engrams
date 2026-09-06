//! The frontmatter is the contract of the memory: if it is misread, everything
//! derived from it lies.

use souvenance::Note;

#[test]
fn reads_top_level_fields() {
    let raw = "---\nname: token-rotation\ndescription: How a token gets renewed\ntype: reference\n---\n\nThe body of the fact.\n";
    let note = Note::parse(raw);
    assert_eq!(note.field("name"), Some("token-rotation"));
    assert_eq!(note.field("description"), Some("How a token gets renewed"));
    assert_eq!(note.field("type"), Some("reference"));
}

#[test]
fn separates_body_from_frontmatter() {
    let raw = "---\nname: x\n---\n\nThe body of the fact.\nOn two lines.\n";
    assert_eq!(Note::parse(raw).body(), "The body of the fact.\nOn two lines.\n");
}

#[test]
fn tolerates_a_file_without_frontmatter() {
    let raw = "Just text, no header.\n";
    let note = Note::parse(raw);
    assert_eq!(note.field("name"), None);
    assert_eq!(note.body(), "Just text, no header.\n");
}

#[test]
fn flattens_nested_keys_one_level() {
    // Tools that write notes often nest `type` under `metadata:`.
    let raw = "---\nname: x\nmetadata:\n  type: project\n  origin: session\n---\n\nbody\n";
    let note = Note::parse(raw);
    assert_eq!(note.field("name"), Some("x"));
    assert_eq!(note.field("type"), Some("project"));
    assert_eq!(note.field("origin"), Some("session"));
}

#[test]
fn a_top_level_key_wins_over_a_nested_one() {
    let raw = "---\ntype: reference\nmetadata:\n  type: project\n---\n\nbody\n";
    assert_eq!(Note::parse(raw).field("type"), Some("reference"));
}

#[test]
fn the_container_key_itself_is_not_a_field() {
    let raw = "---\nmetadata:\n  type: project\n---\n\nbody\n";
    assert_eq!(Note::parse(raw).field("metadata"), None);
}

#[test]
fn strips_quotes_around_a_value() {
    let raw = "---\ndescription: \"A quoted value\"\n---\n\nbody\n";
    assert_eq!(Note::parse(raw).field("description"), Some("A quoted value"));
}

#[test]
fn a_missing_status_means_active() {
    assert!(Note::parse("---\nname: x\n---\n\nbody\n").is_active());
}

#[test]
fn an_archived_status_makes_the_note_inactive() {
    assert!(!Note::parse("---\nname: x\nstatus: archived\n---\n\nbody\n").is_active());
}
