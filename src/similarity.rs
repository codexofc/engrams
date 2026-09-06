//! Ranking by proximity. Rank matters as much as presence: a good note in ninth
//! position is not found, it is buried.

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Hit {
    pub name: String,
    pub score: f32,
}

/// Cosine similarity. Returns 0 rather than NaN on a zero vector.
pub fn cosine(a: &[f32], b: &[f32]) -> f32 {
    let (mut dot, mut norm_a, mut norm_b) = (0.0f32, 0.0f32, 0.0f32);
    for (x, y) in a.iter().zip(b) {
        dot += x * y;
        norm_a += x * x;
        norm_b += y * y;
    }
    let denom = norm_a.sqrt() * norm_b.sqrt();
    if denom == 0.0 {
        0.0
    } else {
        dot / denom
    }
}

/// Ranks a corpus by decreasing proximity and keeps the `limit` first.
pub fn rank(query: &[f32], corpus: &[(String, Vec<f32>)], limit: usize) -> Vec<Hit> {
    let mut hits: Vec<Hit> = corpus.iter().map(|(name, v)| Hit { name: name.clone(), score: cosine(query, v) }).collect();
    // `total_cmp`: a NaN must not decide the order.
    hits.sort_unstable_by(|a, b| b.score.total_cmp(&a.score));
    hits.truncate(limit);
    hits
}

/// A result aggregated at the note level.
#[derive(Debug, Clone)]
pub struct NoteHit {
    pub path: String,
    pub score: f32,
    pub ordinal: usize,
}

/// Ranks chunks, then aggregates per note by MAXIMUM: a note whose single passage
/// answers the question is a good answer even if the rest is about something else.
pub fn rank_notes(query: &[f32], chunks: &[(String, usize, Vec<f32>)], limit: usize) -> Vec<NoteHit> {
    rank_notes_with_bonus(query, chunks, limit, &HashMap::new())
}

/// Like `rank_notes`, with a per-note bonus added to the cosine of its chunks. The
/// bonus carries the lexical signal on identifiers and the learned usage signal.
pub fn rank_notes_with_bonus(query: &[f32], chunks: &[(String, usize, Vec<f32>)], limit: usize, bonus: &HashMap<String, f32>) -> Vec<NoteHit> {
    let mut best: HashMap<&str, NoteHit> = HashMap::new();
    for (path, ordinal, vector) in chunks {
        let score = cosine(query, vector) + bonus.get(path.as_str()).copied().unwrap_or(0.0);
        match best.get(path.as_str()) {
            Some(previous) if previous.score >= score => {}
            _ => {
                best.insert(path.as_str(), NoteHit { path: path.clone(), score, ordinal: *ordinal });
            }
        }
    }
    let mut out: Vec<NoteHit> = best.into_values().collect();
    // Tie-break on the path, so the output is reproducible.
    out.sort_unstable_by(|a, b| b.score.total_cmp(&a.score).then_with(|| a.path.cmp(&b.path)));
    out.truncate(limit);
    out
}

/// Words of a query that look like identifiers rather than language: ticket keys
/// (`ABC-213`), names with underscores (`DB_HOST`), letter-digit mixes (`v2`, `rc23`),
/// file names, camelCase. An exact match on those is a strong signal.
pub fn identifiers(query: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for raw in query.split(|c: char| c.is_whitespace() || matches!(c, ',' | ';' | '(' | ')' | '«' | '»' | '"' | '\'' | '?' | '!' | ':')) {
        let word = raw.trim_matches(|c: char| c == '.' && !raw.ends_with(".md") && !raw.ends_with(".sh"));
        if word.chars().count() < 3 || word.chars().all(|c| !c.is_alphanumeric()) {
            continue;
        }
        let has_digit = word.chars().any(|c| c.is_ascii_digit());
        let has_alpha = word.chars().any(|c| c.is_alphabetic());
        let ticket = word
            .split_once('-')
            .is_some_and(|(a, b)| a.len() >= 2 && a.chars().all(|c| c.is_ascii_uppercase()) && !b.is_empty() && b.chars().all(|c| c.is_ascii_digit()));
        let underscore = word.contains('_');
        let file = word.contains('.')
            && word
                .rsplit('.')
                .next()
                .is_some_and(|ext| matches!(ext, "md" | "sh" | "rs" | "php" | "dart" | "json" | "yml" | "yaml" | "toml" | "sql" | "py" | "ts" | "js"));
        let camel =
            word.chars().any(|c| c.is_ascii_lowercase()) && word.chars().skip(1).any(|c| c.is_ascii_uppercase()) && word.chars().all(|c| c.is_alphanumeric());
        if ticket || underscore || file || camel || (has_digit && has_alpha) {
            let w = word.to_string();
            if !out.contains(&w) {
                out.push(w);
            }
        }
    }
    out
}

/// Per-note bonus for identifiers present verbatim in its text, case-sensitive.
/// `step` per identifier found, capped at two.
pub fn lexical_bonus(base: &std::path::Path, ids: &[String], step: f32) -> HashMap<String, f32> {
    let mut out = HashMap::new();
    if ids.is_empty() || step <= 0.0 {
        return out;
    }
    for f in crate::hot::notes_of(base) {
        let Ok(content) = std::fs::read_to_string(&f) else { continue };
        let hits = ids.iter().filter(|id| content.contains(id.as_str())).count();
        if hits > 0 {
            out.insert(crate::paths::relative(&f, base), step * hits.min(2) as f32);
        }
    }
    out
}
