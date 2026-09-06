//! The hot index: strata by type, archived notes excluded and counted, compaction
//! under the bound, shared section of the family.
use engrams::hot::{projects, render, BOUND};
use std::path::PathBuf;

fn corpus(name: &str) -> PathBuf {
    let base = std::env::temp_dir().join(format!("engram-hot-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(base.join("fam/proj")).unwrap();
    std::fs::create_dir_all(base.join("fam/common")).unwrap();
    base
}

fn note(base: &std::path::Path, rel: &str, name: &str, kind: &str, status: &str, verified: &str, body: &str) {
    let text = format!("---\nname: {name}\ndescription: Description of {name}\ntype: {kind}\nstatus: {status}\nverified: {verified}\n---\n\n{body}\n");
    std::fs::write(base.join(rel), text).unwrap();
}

#[test]
fn strata_archived_and_shared_notes_are_rendered_in_order() {
    let base = corpus("strata");
    note(&base, "fam/proj/rule.md", "rule", "feedback", "active", "2026-09-01", "body");
    note(&base, "fam/proj/ref.md", "ref", "reference", "active", "2026-09-01", "body");
    note(&base, "fam/proj/work.md", "work", "project", "active", "2026-09-01", "body");
    note(&base, "fam/proj/old.md", "old", "project", "archived", "2026-01-01", "body");
    note(&base, "fam/common/shared.md", "shared", "reference", "active", "2026-09-01", "body");
    let text = render(&base, "fam/proj");
    let pos = |s: &str| text.find(s).unwrap_or_else(|| panic!("{s} missing in:\n{text}"));
    assert!(pos("## Durable knowledge") < pos("## Ways of working"));
    assert!(pos("## Ways of working") < pos("## Projects"));
    assert!(pos("## Projects") < pos("## Shared by fam"));
    assert!(text.contains("- [shared](../common/shared.md): Description of shared"));
    assert!(!text.contains("old.md"), "an archived note is not in the index");
    assert!(text.contains("<!-- 1 archived note(s)"));
    assert_eq!(projects(&base), vec!["fam/common".to_string(), "fam/proj".to_string()]);
    assert!(!render(&base, "fam/common").contains("## Shared"), "the shared project does not include itself");
    std::fs::remove_dir_all(&base).unwrap();
}

#[test]
fn an_overflowing_index_drops_the_oldest_project_notes_first() {
    let base = corpus("bound");
    for i in 0..200 {
        let name = format!("work-{i:03}");
        let verified = format!("2026-{:02}-{:02}", 1 + i % 12, 1 + i % 28);
        let text = format!("---\nname: {name}\ndescription: {}\ntype: project\nstatus: active\nverified: {verified}\n---\n\nbody\n", "x".repeat(150));
        std::fs::write(base.join("fam/proj").join(format!("{name}.md")), text).unwrap();
    }
    note(&base, "fam/proj/rule.md", "rule", "feedback", "active", "2026-09-01", "body");
    let text = render(&base, "fam/proj");
    assert!(text.len() <= BOUND, "index at {} bytes", text.len());
    assert!(text.contains("left out of the hot index"));
    assert!(text.contains("[rule](rule.md)"), "a way of working never leaves");
    assert!(text.contains("work-011") || text.contains("work-023"), "a recent note must stay");
    assert!(!text.contains("[work-000]"), "the oldest must leave first");
    std::fs::remove_dir_all(&base).unwrap();
}
