//! Corpus validation.
//!
//! Every frontmatter field has a reader here. It also defines, by what it refuses,
//! the YAML subset the memory accepts: no folded scalars, no lists, no CRLF. A small,
//! known format beats a complete, approximate one.

use crate::note::Note;
use std::collections::{BTreeMap, BTreeSet};

/// Recognised note types. They decide the stratum of the hot index.
pub const TYPES: [&str; 4] = ["user", "feedback", "project", "reference"];

/// A problem found on a note.
#[derive(Debug, Clone)]
pub struct Finding {
    pub path: String,
    pub message: String,
}

impl Finding {
    pub fn new(path: &str, message: impl Into<String>) -> Self {
        Finding { path: path.to_string(), message: message.into() }
    }
}

/// Checks that only need one note.
pub fn check_note(path: &str, note: &Note) -> Vec<Finding> {
    let mut out = Vec::new();

    // Naming: the file is named after its `name`, in lowercase ASCII, digits and
    // dashes, without a type prefix. A name that drifts ends up linked under two
    // spellings and searched under three.
    let stem = path.rsplit('/').next().unwrap_or(path);
    let stem = stem.strip_suffix(".md").unwrap_or(stem);
    if let Some(name) = note.field("name") {
        if name != stem {
            out.push(Finding::new(path, format!("file must be named after its name field ({name}.md)")));
        }
    }
    let kebab = !stem.is_empty()
        && stem.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        && !stem.starts_with('-')
        && !stem.ends_with('-')
        && !stem.contains("--");
    if !kebab {
        out.push(Finding::new(path, "name outside the convention: lowercase ASCII, digits and dashes only"));
    }
    if ["project-", "feedback-", "reference-", "user-"].iter().any(|p| stem.starts_with(p)) {
        out.push(Finding::new(path, "type prefix in the name: the type belongs in the frontmatter"));
    }

    if note.field("name").is_none() {
        out.push(Finding::new(path, "missing `name` field"));
    }
    // The description decides relevance in the hot index.
    if note.field("description").is_none() {
        out.push(Finding::new(path, "missing `description` field"));
    }

    if let Some(kind) = note.field("type") {
        if !TYPES.contains(&kind) {
            out.push(Finding::new(path, format!("unknown type `{kind}`, expected one of {TYPES:?}")));
        }
    }

    // Facts are replaced, never lost: an archived note names its successor.
    if note.field("status") == Some("archived") && note.field("superseded_by").is_none() {
        out.push(Finding::new(path, "archived without `superseded_by`: the fact leaves the index with no successor"));
    }

    out.extend(check_yaml_subset(path, note));
    out
}

/// Refuses what the parser cannot read rather than reading it halfway.
fn check_yaml_subset(path: &str, note: &Note) -> Vec<Finding> {
    let mut out = Vec::new();
    for key in ["name", "description"] {
        if matches!(note.field(key), Some(">") | Some("|")) {
            out.push(Finding::new(path, format!("`{key}` is a folded scalar, unsupported: put the value on one line")));
        }
    }
    if note.body().contains('\r') || note.field("name").is_some_and(|v| v.contains('\r')) {
        out.push(Finding::new(path, "Windows line endings, unsupported: convert the file to LF"));
    }
    out
}

/// Checks that need the whole corpus.
pub fn check_corpus(notes: &[(String, Note)]) -> Vec<Finding> {
    let mut out = Vec::new();

    // The name is the target of `[[links]]`, so two homonyms make every link to it
    // ambiguous.
    let mut by_name: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for (path, note) in notes {
        if let Some(name) = note.field("name") {
            by_name.entry(name).or_default().push(path);
        }
    }
    for (name, paths) in &by_name {
        if paths.len() > 1 {
            out.push(Finding::new(paths[0], format!("name `{name}` shared with {}", paths[1..].join(", "))));
        }
    }

    // A link may target the `name` field or the file stem, with `_` or `-`, in any
    // case: they are the same notes.
    let mut known: BTreeSet<String> = BTreeSet::new();
    for (path, note) in notes {
        if let Some(name) = note.field("name") {
            known.insert(link_key(name));
        }
        let stem = path.rsplit('/').next().unwrap_or(path);
        known.insert(link_key(stem.strip_suffix(".md").unwrap_or(stem)));
    }
    for (path, note) in notes {
        for target in wiki_links(note.body()) {
            if !known.contains(&link_key(target)) {
                out.push(Finding::new(path, format!("dangling link: [[{target}]]")));
            }
        }
    }
    out
}

/// Comparable form of a note name: lowercase letters and digits only.
pub fn link_key(name: &str) -> String {
    name.chars().filter(|c| c.is_alphanumeric()).flat_map(char::to_lowercase).collect()
}

/// Targets of `[[name]]`, `[[name|alias]]` and `[[name#section]]` links.
fn wiki_links(body: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut rest = body;
    while let Some(open) = rest.find("[[") {
        let after = &rest[open + 2..];
        match after.find("]]") {
            Some(close) => {
                let raw = &after[..close];
                out.push(raw.split(['|', '#']).next().unwrap_or(raw).trim());
                rest = &after[close + 2..];
            }
            None => break,
        }
    }
    out
}
