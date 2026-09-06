//! Questions indexed next to paragraphs.
//!
//! A query is short and interrogative, a technical paragraph is long and
//! declarative. For each paragraph, an external command (a language model chosen by
//! `KEPT_QUESTIONS_CMD`) writes the questions it answers; they are embedded and
//! indexed as extra entries pointing back to the paragraph. The cache maps a
//! paragraph fingerprint to its questions, so an unchanged paragraph never goes
//! through the model twice. The engine knows no model: it runs a command, feeds it
//! the text, and reads what comes back.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

pub const MAX_QUESTIONS: usize = 3;

pub struct Store {
    path: PathBuf,
    map: HashMap<String, Vec<String>>,
    dirty: bool,
}

impl Store {
    pub fn load(path: &Path) -> Store {
        let map = std::fs::read_to_string(path).ok().and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default();
        Store { path: path.to_path_buf(), map, dirty: false }
    }

    pub fn get(&self, hash: u64) -> Option<&Vec<String>> {
        self.map.get(&hash.to_string())
    }

    pub fn insert(&mut self, hash: u64, questions: Vec<String>) {
        self.map.insert(hash.to_string(), questions);
        self.dirty = true;
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// Forgets the questions of paragraphs that no longer exist as such.
    pub fn prune(&mut self, live: &HashSet<u64>) -> usize {
        let before = self.map.len();
        self.map.retain(|k, _| k.parse::<u64>().is_ok_and(|h| live.contains(&h)));
        if self.map.len() != before {
            self.dirty = true;
        }
        before - self.map.len()
    }

    /// Atomic write, only when something changed. Keys are sorted so the file diffs
    /// cleanly under version control.
    pub fn save(&mut self) -> Result<(), String> {
        if !self.dirty {
            return Ok(());
        }
        let mut entries: Vec<(&String, &Vec<String>)> = self.map.iter().collect();
        entries.sort();
        let ordered: serde_json::Map<String, serde_json::Value> = entries.into_iter().map(|(k, v)| (k.clone(), serde_json::json!(v))).collect();
        let text = serde_json::to_string_pretty(&ordered).map_err(|e| e.to_string())?;
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("create {}: {e}", parent.display()))?;
        }
        let tmp = self.path.with_extension("tmp");
        std::fs::write(&tmp, text).map_err(|e| format!("write: {e}"))?;
        std::fs::rename(&tmp, &self.path).map_err(|e| format!("rename: {e}"))?;
        self.dirty = false;
        Ok(())
    }
}

/// The prompt given to the command. Two questions in the language of the excerpt
/// and one in English, because queries come in both.
pub fn prompt(text: &str) -> String {
    format!(
        "Here is an excerpt from personal technical notes. Write exactly three short questions \
         that this excerpt answers, one per line, without numbering, bullets, comments or \
         introduction. The first two in the language of the excerpt, the third in English (all \
         three in English if the excerpt is in English). The questions must be askable by \
         someone looking for this information without knowing the text.\n\n---\n{text}\n---"
    )
}

/// The command output, cleaned: escape codes removed, header or decoration lines
/// dropped, at most `MAX_QUESTIONS` questions of reasonable length.
pub fn parse_output(raw: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for line in strip_ansi(raw).lines() {
        let mut l = line.trim();
        l = l.trim_start_matches(|c: char| c.is_ascii_digit() || matches!(c, '.' | ')' | '-' | '*' | '•' | ' '));
        let l = l.trim().trim_matches('"').trim();
        if l.is_empty() || l.starts_with('>') || l.starts_with('#') || l.starts_with("---") {
            continue;
        }
        let n = l.chars().count();
        if !(8..=220).contains(&n) || out.iter().any(|q| q == l) {
            continue;
        }
        out.push(l.to_string());
        if out.len() == MAX_QUESTIONS {
            break;
        }
    }
    out
}

fn strip_ansi(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\u{1b}' {
            if chars.peek() == Some(&'[') {
                chars.next();
                for d in chars.by_ref() {
                    if d.is_ascii_alphabetic() {
                        break;
                    }
                }
            }
            continue;
        }
        out.push(c);
    }
    out
}

/// Runs the command with the prompt on standard input and returns the questions.
pub fn generate(cmd: &str, text: &str) -> Result<Vec<String>, String> {
    use std::io::Write;
    let mut child = std::process::Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map_err(|e| format!("questions command: {e}"))?;
    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(prompt(text).as_bytes());
    }
    let out = child.wait_with_output().map_err(|e| e.to_string())?;
    if !out.status.success() {
        return Err(format!("questions command: exit {}", out.status));
    }
    let questions = parse_output(&String::from_utf8_lossy(&out.stdout));
    if questions.is_empty() {
        return Err("questions command: no readable question in the output".into());
    }
    Ok(questions)
}

/// Several paragraphs at once, `parallel` commands in flight. Model commands spend
/// their time waiting on the network, so four in flight divide the time by four.
pub fn generate_many(cmd: &str, texts: &[String], parallel: usize) -> Vec<Result<Vec<String>, String>> {
    let parallel = parallel.clamp(1, texts.len().max(1));
    let per = texts.len().div_ceil(parallel);
    let parts: Vec<Vec<Result<Vec<String>, String>>> = std::thread::scope(|scope| {
        let handles: Vec<_> = texts.chunks(per).map(|part| scope.spawn(move || part.iter().map(|t| generate(cmd, t)).collect())).collect();
        handles.into_iter().map(|h| h.join().unwrap_or_default()).collect()
    });
    parts.into_iter().flatten().collect()
}
