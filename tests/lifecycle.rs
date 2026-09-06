//! Life-cycle operations keep everything they do not touch.
use souvenance::lifecycle::{append_paragraph, relink, remove_field, set_field};

const NOTE: &str = "---\nname: alpha\ndescription: \"Alpha: a note\"\ntype: project\nstatus: active\n---\n\nFirst paragraph.\n\nSee [[beta]] and [[Beta_Two|that one]] and [[gamma#section]].\n";

#[test]
fn set_field_replaces_or_adds_without_touching_the_rest() {
    let s = set_field(NOTE, "verified", "2026-09-06").unwrap();
    assert!(s.contains("\nverified: 2026-09-06\n"));
    assert!(s.contains("description: \"Alpha: a note\""));
    assert!(s.ends_with("[[gamma#section]].\n"));
    let s2 = set_field(&s, "status", "archived").unwrap();
    assert!(s2.contains("\nstatus: archived\n") && !s2.contains("status: active"));
    assert_eq!(s2.matches("verified:").count(), 1);
    let s3 = set_field(&s2, "superseded_by", "[[alpha-2]]").unwrap();
    assert!(s3.contains("superseded_by: [[alpha-2]]"), "{s3}");
    assert!(remove_field(&s3, "superseded_by").unwrap().lines().all(|l| !l.starts_with("superseded_by")));
}

#[test]
fn append_keeps_one_blank_line_between_paragraphs() {
    let s = append_paragraph(NOTE, "Addition.");
    assert!(s.ends_with("[[gamma#section]].\n\nAddition.\n"), "{s}");
}

#[test]
fn relink_rewrites_every_spelling_of_the_old_name_and_keeps_aliases() {
    let (s, n) = relink(NOTE, "betatwo", "beta-three");
    assert_eq!(n, 1);
    assert!(s.contains("[[beta-three|that one]]"), "{s}");
    assert!(s.contains("[[beta]]") && s.contains("[[gamma#section]]"));
    let (s, n) = relink(NOTE, "beta", "beta-2");
    assert_eq!(n, 1);
    assert!(s.contains("[[beta-2]] and [[Beta_Two|that one]]"), "{s}");
}
