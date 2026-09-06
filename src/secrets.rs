//! Refuses content that looks like a secret. Deliberately blunt: a false positive
//! is lifted by hand, a real token pushed to a remote host is not taken back.

use regex::Regex;
use std::path::Path;

pub struct Hit {
    pub path: String,
    pub line: usize,
    pub kind: &'static str,
}

const PATTERNS: [(&str, &str); 7] = [
    ("GitLab token", r"glpat-[A-Za-z0-9_\-]{15,}"),
    ("Anthropic key", r"sk-ant-[A-Za-z0-9_\-]{20,}"),
    ("GitHub token", r"gh[pousr]_[A-Za-z0-9]{30,}"),
    ("AWS key", r"AKIA[0-9A-Z]{16}"),
    ("private key", r"-----BEGIN [A-Z ]*PRIVATE KEY-----"),
    ("YouTrack token", r"perm[-:][A-Za-z0-9._\-]{30,}"),
    ("long bearer token", r"Bearer\s+[A-Za-z0-9._\-]{40,}"),
];

/// Secrets found in a text, with their line number.
pub fn scan_text(path: &str, text: &str) -> Vec<Hit> {
    let mut out = Vec::new();
    for (kind, pat) in PATTERNS {
        let re = Regex::new(pat).expect("valid pattern");
        for m in re.find_iter(text) {
            out.push(Hit { path: path.to_string(), line: text[..m.start()].matches('\n').count() + 1, kind });
        }
    }
    // Plain password: `password = value`. The regex crate has no look-behind, so the
    // context is checked by hand around each match.
    let pw = Regex::new(r#"(?i)(password|passwd|mdp)\s*[=:]\s*["']?([^\s"'<>{}&]{6,})"#).expect("valid pattern");
    for c in pw.captures_iter(text) {
        let whole = c.get(0).unwrap();
        let value = c.get(2).unwrap().as_str();
        let before = &text[..whole.start()];
        // OAuth documentation, elided or placeholder values are not secrets.
        if before.ends_with("grant_type=") || value.starts_with('<') || value.starts_with("...") {
            continue;
        }
        let upper: String = value.chars().take_while(|ch| ch.is_ascii_uppercase() || *ch == '_').collect();
        let boundary = value[upper.len()..].chars().next().is_none_or(|ch| !(ch.is_alphanumeric() || ch == '_'));
        if upper.trim_matches('_').len() >= 4 && boundary {
            continue;
        }
        out.push(Hit { path: path.to_string(), line: before.matches('\n').count() + 1, kind: "password" });
    }
    out
}

/// Scans every `.md` under `root`.
pub fn scan_dir(root: &Path) -> Vec<Hit> {
    let mut files = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(d) = stack.pop() {
        let Ok(listing) = std::fs::read_dir(&d) else { continue };
        for e in listing.flatten() {
            let p = e.path();
            if p.is_dir() {
                if p.file_name().is_some_and(|n| n == ".git" || n == ".souvenance" || n == "models" || n == "target") {
                    continue;
                }
                stack.push(p);
            } else if p.extension().is_some_and(|x| x == "md") {
                files.push(p);
            }
        }
    }
    files.sort();
    let mut out = Vec::new();
    for f in files {
        let Ok(text) = std::fs::read_to_string(&f) else { continue };
        out.extend(scan_text(&crate::paths::relative(&f, root), &text));
    }
    out
}
