//! `souvenance check` gives every frontmatter field a reader, and defines by what it
//! refuses the YAML subset the memory accepts.

use souvenance::check::{check_corpus, check_note};
use souvenance::note::Note;

fn note(raw: &str) -> Note {
    Note::parse(raw)
}

fn messages(findings: &[souvenance::check::Finding]) -> String {
    findings.iter().map(|f| f.message.as_str()).collect::<Vec<_>>().join(" | ")
}

#[test]
fn a_complete_note_raises_nothing() {
    let n = note("---\nname: x\ndescription: A complete note\ntype: reference\nstatus: active\n---\n\nbody\n");
    assert!(check_note("x.md", &n).is_empty());
}

#[test]
fn a_missing_name_is_reported() {
    let n = note("---\ndescription: No name\n---\n\nbody\n");
    assert!(messages(&check_note("a.md", &n)).contains("name"));
}

#[test]
fn a_missing_description_is_reported() {
    let n = note("---\nname: x\n---\n\nbody\n");
    assert!(messages(&check_note("a.md", &n)).contains("description"));
}

#[test]
fn an_unknown_type_is_reported() {
    let n = note("---\nname: x\ndescription: d\ntype: task\n---\n\nbody\n");
    assert!(messages(&check_note("a.md", &n)).contains("type"));
}

#[test]
fn an_archived_note_without_a_successor_is_reported() {
    let n = note("---\nname: x\ndescription: d\nstatus: archived\n---\n\nbody\n");
    assert!(messages(&check_note("a.md", &n)).contains("superseded_by"));
}

#[test]
fn an_archived_note_with_a_successor_is_accepted() {
    let n = note("---\nname: x\ndescription: d\nstatus: archived\nsuperseded_by: \"[[y]]\"\n---\n\nbody\n");
    assert!(check_note("x.md", &n).is_empty());
}

#[test]
fn a_folded_scalar_is_refused() {
    let n = note("---\nname: x\ndescription: >\n  a folded value\n---\n\nbody\n");
    assert!(messages(&check_note("a.md", &n)).contains("folded"));
}

#[test]
fn a_windows_line_ending_is_refused() {
    let n = note("---\r\nname: x\r\ndescription: d\r\n---\r\n\r\nbody\r\n");
    assert!(messages(&check_note("a.md", &n)).contains("Windows"));
}

#[test]
fn a_dangling_link_is_reported() {
    let corpus = vec![("a.md".to_string(), note("---\nname: a\ndescription: d\n---\n\nsee [[never-written]]\n"))];
    assert!(messages(&check_corpus(&corpus)).contains("never-written"));
}

#[test]
fn a_link_to_an_existing_note_is_accepted() {
    let corpus = vec![
        ("a.md".to_string(), note("---\nname: a\ndescription: d\n---\n\nsee [[b]]\n")),
        ("b.md".to_string(), note("---\nname: b\ndescription: d\n---\n\nbody\n")),
    ];
    assert!(check_corpus(&corpus).is_empty());
}

#[test]
fn two_notes_sharing_a_name_are_reported() {
    let corpus = vec![
        ("fam/a.md".to_string(), note("---\nname: twin\ndescription: d\n---\n\nbody\n")),
        ("personal/b.md".to_string(), note("---\nname: twin\ndescription: d\n---\n\nbody\n")),
    ];
    assert!(messages(&check_corpus(&corpus)).contains("twin"));
}

#[test]
fn a_finding_carries_the_path_of_the_note() {
    let n = note("---\ndescription: no name\n---\n\nbody\n");
    assert_eq!(check_note("fam/one/a.md", &n)[0].path, "fam/one/a.md");
}

#[test]
fn a_link_by_file_stem_or_with_underscores_is_not_dangling() {
    // Notes are written by hand: links use the file stem as often as the name
    // field, with underscores where the name has dashes.
    let target = note("---\nname: project-audit-v2\ndescription: target\ntype: project\n---\n\nbody\n");
    let source = note("---\nname: src\ndescription: source\ntype: project\n---\n\nSee [[project_audit_v2]], [[Project-Audit-V2|the audit]] and [[project_audit_v2#section]]. But [[nowhere]] is missing.\n");
    let corpus = vec![("p/project_audit_v2.md".to_string(), target), ("p/src.md".to_string(), source)];
    let msgs = messages(&check_corpus(&corpus));
    assert!(msgs.contains("[[nowhere]]"), "{msgs}");
    assert_eq!(msgs.matches("dangling link").count(), 1, "{msgs}");
}

#[test]
fn a_file_named_differently_from_its_name_or_with_a_type_prefix_is_reported() {
    let n = note("---\nname: audit-v2\ndescription: d\ntype: project\nstatus: active\n---\n\nbody\n");
    assert!(check_note("p/audit-v2.md", &n).is_empty());
    let msgs = messages(&check_note("p/project_audit_v2.md", &n));
    assert!(msgs.contains("named after its name"), "{msgs}");
    assert!(msgs.contains("outside the convention"), "{msgs}");
    let n2 = note("---\nname: project-x\ndescription: d\ntype: project\nstatus: active\n---\n\nbody\n");
    assert!(messages(&check_note("p/project-x.md", &n2)).contains("type prefix"));
}
