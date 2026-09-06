//! A note: its frontmatter and its body.
//!
//! The frontmatter is the contract of the memory. It carries what decides freshness
//! (`verified`), visibility (`status`) and relevance (`description`).

use std::collections::BTreeMap;

/// Frontmatter fields and body of a note. Owned strings: a note outlives the buffer
/// it was parsed from.
#[derive(Debug, Clone, Default)]
pub struct Note {
    fields: BTreeMap<String, String>,
    body: String,
}

impl Note {
    /// Parses a note. A file without frontmatter is still a note, with no fields:
    /// refusing it would make it invisible, which is worse.
    pub fn parse(raw: &str) -> Self {
        let Some(rest) = raw.strip_prefix("---\n") else {
            return Self::without_header(raw);
        };
        let Some(end) = rest.find("\n---") else {
            return Self::without_header(raw);
        };

        let mut fields = BTreeMap::new();
        for line in rest[..end].lines() {
            let Some((key, value)) = parse_field(line) else {
                continue;
            };
            // A root-level field wins over a nested homonym.
            let root = !line.starts_with([' ', '\t']);
            if root || !fields.contains_key(&key) {
                fields.insert(key, value);
            }
        }
        let body = rest[end..].trim_start_matches("\n---").trim_start_matches('\n').to_string();

        Note { fields, body }
    }

    fn without_header(raw: &str) -> Self {
        Note { fields: BTreeMap::new(), body: raw.to_string() }
    }

    pub fn field(&self, key: &str) -> Option<&str> {
        self.fields.get(key).map(String::as_str)
    }

    pub fn body(&self) -> &str {
        &self.body
    }

    /// A note without `status` is active.
    pub fn is_active(&self) -> bool {
        self.field("status") != Some("archived")
    }
}

/// Reads one `key: value` line. Nested lines are read too, flattened to one level,
/// so a `type` under a `metadata:` block is still found. A key without a value is
/// not a field.
fn parse_field(line: &str) -> Option<(String, String)> {
    if line.trim_start().starts_with('#') {
        return None;
    }
    let (key, value) = line.split_once(':')?;
    let key = key.trim();
    if key.is_empty() || !key.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
        return None;
    }
    let value = value.trim().trim_matches(['"', '\'']);
    if value.is_empty() {
        return None;
    }
    Some((key.to_string(), value.to_string()))
}
