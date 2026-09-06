//! The hot index: one generated `MEMORY.md` per project.
//!
//! It is the file a session loads at start, so it is bounded: past `BOUND` bytes it
//! dilutes attention instead of guiding it. When it overflows, project notes leave
//! from the oldest verified to the newest until it fits, and one line says how many
//! are missing. Durable knowledge and ways of working never leave.
//!
//! A project named `common` inside a family is listed in the hot index of every
//! sibling project.

use crate::note::Note;
use std::path::{Path, PathBuf};

/// Past this size the index loaded at every session dilutes attention.
pub const BOUND: usize = 17408;

/// Name of the project shared by a whole family.
pub const COMMON: &str = "common";

/// Loading order: durable knowledge first, ongoing projects last.
const STRATA: [(&str, &str); 4] = [("reference", "Durable knowledge"), ("feedback", "Ways of working"), ("user", "About the user"), ("project", "Projects")];

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Entry {
    name: String,
    rel: String,
    description: String,
    date: String,
}

/// The notes under a directory, sorted by file name, indexes excluded.
pub fn notes_of(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        let Ok(listing) = std::fs::read_dir(&d) else { continue };
        for e in listing.flatten() {
            let p = e.path();
            if p.is_dir() {
                if p.file_name().is_some_and(|n| n.to_string_lossy().starts_with('.')) {
                    continue;
                }
                stack.push(p);
            } else if p.extension().is_some_and(|x| x == "md") && p.file_name().is_some_and(|n| !n.to_string_lossy().starts_with("MEMORY")) {
                out.push(p);
            }
        }
    }
    out.sort();
    out
}

/// The projects, as `family/project`, sorted.
pub fn projects(base: &Path) -> Vec<String> {
    let mut out = Vec::new();
    let Ok(families) = std::fs::read_dir(base) else { return out };
    let mut families: Vec<PathBuf> =
        families.flatten().map(|e| e.path()).filter(|p| p.is_dir() && !p.file_name().is_some_and(|n| n.to_string_lossy().starts_with('.'))).collect();
    families.sort();
    for fam in families {
        let Ok(projs) = std::fs::read_dir(&fam) else { continue };
        let mut projs: Vec<PathBuf> = projs.flatten().map(|e| e.path()).filter(|p| p.is_dir()).collect();
        projs.sort();
        for p in projs {
            out.push(format!("{}/{}", fam.file_name().unwrap_or_default().to_string_lossy(), p.file_name().unwrap_or_default().to_string_lossy()));
        }
    }
    out
}

pub fn display_name(path: &Path, note: &Note) -> String {
    note.field("name").map(str::to_string).unwrap_or_else(|| path.file_stem().unwrap_or_default().to_string_lossy().into_owned())
}

fn date_of(path: &Path, note: &Note) -> String {
    if let Some(d) = note.field("verified").or_else(|| note.field("modified")) {
        return d.to_string();
    }
    let secs = std::fs::metadata(path).and_then(|m| m.modified()).ok().and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok()).map_or(0, |d| d.as_secs());
    ymd(secs)
}

/// Civil date (UTC) of a timestamp, without a dependency (Howard Hinnant's algorithm).
pub fn ymd(secs: u64) -> String {
    let days = (secs / 86_400) as i64;
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!("{y:04}-{m:02}-{d:02}")
}

fn line(e: &Entry) -> String {
    if e.description.is_empty() {
        format!("- [{}]({})", e.name, e.rel)
    } else {
        format!("- [{}]({}): {}", e.name, e.rel, e.description)
    }
}

/// The index text of a project (`family/project`), compacted under the bound.
pub fn render(base: &Path, project: &str) -> String {
    let dir = base.join(project);
    let mut by_type: std::collections::BTreeMap<String, Vec<Entry>> = Default::default();
    let mut archived = 0usize;
    for f in notes_of(&dir) {
        let Ok(content) = std::fs::read_to_string(&f) else { continue };
        let note = Note::parse(&content);
        if !note.is_active() {
            archived += 1;
            continue;
        }
        let rel = f.strip_prefix(&dir).unwrap_or(&f).to_string_lossy().into_owned();
        by_type.entry(note.field("type").unwrap_or("project").to_string()).or_default().push(Entry {
            name: display_name(&f, &note),
            rel,
            description: note.field("description").unwrap_or_default().trim().to_string(),
            date: date_of(&f, &note),
        });
    }
    let known: Vec<&str> = STRATA.iter().map(|(t, _)| *t).collect();
    let mut strata: Vec<(String, String)> = STRATA.iter().map(|(t, l)| (t.to_string(), l.to_string())).collect();
    for t in by_type.keys() {
        if !known.contains(&t.as_str()) {
            strata.push((t.clone(), t.clone()));
        }
    }

    // Shared section of the family, for projects other than the shared one.
    let (family, project_name) = project.split_once('/').unwrap_or((project, ""));
    let mut common_lines: Vec<String> = Vec::new();
    let common_dir = base.join(family).join(COMMON);
    if !project_name.is_empty() && project_name != COMMON && common_dir.is_dir() {
        for f in notes_of(&common_dir) {
            let Ok(content) = std::fs::read_to_string(&f) else { continue };
            let note = Note::parse(&content);
            if !note.is_active() {
                continue;
            }
            let rel = format!("../{COMMON}/{}", f.strip_prefix(&common_dir).unwrap_or(&f).to_string_lossy());
            let desc = note.field("description").unwrap_or_default().trim();
            let head = format!("- [{}]({rel})", display_name(&f, &note));
            common_lines.push(if desc.is_empty() { head } else { format!("{head}: {desc}") });
        }
        common_lines.sort();
    }

    let render_with = |masked: &std::collections::BTreeSet<Entry>| -> String {
        let mut out = vec![format!("# Memory index, {project}"), String::new()];
        for (t, label) in &strata {
            let Some(entries) = by_type.get(t) else { continue };
            let mut visible: Vec<&Entry> = entries.iter().filter(|e| !masked.contains(e)).collect();
            visible.sort();
            if visible.is_empty() && t != "project" {
                continue;
            }
            out.push(format!("## {label}"));
            out.push(String::new());
            for e in visible {
                out.push(line(e));
            }
            if t == "project" && !masked.is_empty() {
                out.push(format!("- {} older project notes left out of the hot index, `souvenance search` finds them", masked.len()));
            }
            out.push(String::new());
        }
        if !common_lines.is_empty() {
            out.push(format!("## Shared by {family}"));
            out.push(String::new());
            out.extend(common_lines.iter().cloned());
            out.push(String::new());
        }
        if archived > 0 {
            out.push(format!("<!-- {archived} archived note(s), outside the index, reachable with souvenance search --archives -->"));
        }
        let mut text = out.join("\n");
        while text.ends_with('\n') {
            text.pop();
        }
        text.push('\n');
        text
    };

    // Compaction: project notes leave from the oldest to the newest.
    let mut candidates: Vec<Entry> = by_type.get("project").cloned().unwrap_or_default();
    candidates.sort_by(|a, b| (&a.date, &a.name).cmp(&(&b.date, &b.name)));
    let mut masked = std::collections::BTreeSet::new();
    let mut text = render_with(&masked);
    while text.len() > BOUND && masked.len() < candidates.len() {
        masked.insert(candidates[masked.len()].clone());
        text = render_with(&masked);
    }
    text
}

/// Writes a project's `MEMORY.md` if it changed; returns its size in bytes.
pub fn write(base: &Path, project: &str) -> Result<usize, String> {
    let text = render(base, project);
    let path = base.join(project).join("MEMORY.md");
    let previous = std::fs::read_to_string(&path).ok();
    if previous.as_deref() != Some(text.as_str()) {
        std::fs::write(&path, &text).map_err(|e| format!("{}: {e}", path.display()))?;
    }
    Ok(text.len())
}
