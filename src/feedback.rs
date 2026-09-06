//! Learning from usage without self-confirmation.
//!
//! A feedback loop that reinforces what the engine itself ranked first turns an
//! early mistake into a fixed point. Nothing here is learned from the engine's own
//! results. Two signals only, both external:
//!
//! - **explicit**: someone says "for this query, this note";
//! - **missed**: after a search, a note the search did not show was read. Reading a
//!   note that was already shown teaches nothing.
//!
//! The influence is bounded three times: a capped bonus, applied only to candidates
//! already close to the top, decaying by half every sixty days. The table is a
//! readable JSON file, and the benchmark evaluates it leave-one-out.

use std::collections::HashMap;
use std::path::Path;

/// Two queries are "the same question" from this cosine.
pub const SIMILAR: f32 = 0.85;
/// Bonus per confirmed pair, before decay and cap.
pub const STEP: f32 = 0.03;
/// Cap of the learned bonus for one note.
pub const CAP: f32 = 0.05;
/// The bonus only applies to candidates within this distance of the top score.
pub const WINDOW: f32 = 0.10;
/// Half-life of a feedback pair, in days.
pub const HALF_LIFE_DAYS: f32 = 60.0;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct Pair {
    pub query: String,
    pub path: String,
    /// `explicit` or `missed`.
    pub source: String,
    pub count: u32,
    /// Civil day (seconds since the epoch / 86 400) of the last confirmation.
    pub last_day: u64,
}

#[derive(Default)]
pub struct Table {
    pub pairs: Vec<Pair>,
}

impl Table {
    pub fn load(path: &Path) -> Table {
        let pairs = std::fs::read_to_string(path).ok().and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default();
        Table { pairs }
    }

    pub fn save(&self, path: &Path) -> Result<(), String> {
        let text = serde_json::to_string_pretty(&self.pairs).map_err(|e| e.to_string())?;
        let tmp = path.with_extension("tmp");
        std::fs::write(&tmp, text).map_err(|e| format!("write: {e}"))?;
        std::fs::rename(&tmp, path).map_err(|e| format!("rename: {e}"))
    }

    /// Adds or reinforces a pair. The count rises once per day at most.
    pub fn record(&mut self, query: &str, path: &str, source: &str, day: u64) -> bool {
        let query = query.trim();
        if query.is_empty() || path.is_empty() {
            return false;
        }
        if let Some(p) = self.pairs.iter_mut().find(|p| p.query == query && p.path == path) {
            if p.last_day == day {
                return false;
            }
            p.count += 1;
            p.last_day = day;
            if source == "explicit" {
                p.source = "explicit".into();
            }
            return true;
        }
        self.pairs.push(Pair { query: query.to_string(), path: path.to_string(), source: source.to_string(), count: 1, last_day: day });
        true
    }

    pub fn forget(&mut self, query: &str) -> usize {
        let before = self.pairs.len();
        self.pairs.retain(|p| p.query != query.trim());
        before - self.pairs.len()
    }
}

/// A logged search: time, query, paths shown.
pub struct Searched {
    pub secs: u64,
    pub query: String,
    pub shown: Vec<String>,
}

/// A logged read.
pub struct Read {
    pub secs: u64,
    pub path: String,
}

/// "Missed" pairs: a read within `window` seconds after a search, of a note that
/// search did not show. The latest search before the read is the one that counts.
pub fn missed_pairs(searches: &[Searched], reads: &[Read], window: u64) -> Vec<(String, String, u64)> {
    let mut out = Vec::new();
    for r in reads {
        let Some(s) = searches.iter().filter(|s| s.secs <= r.secs && r.secs - s.secs <= window).max_by_key(|s| s.secs) else {
            continue;
        };
        if s.shown.iter().any(|p| p == &r.path) {
            continue;
        }
        out.push((s.query.clone(), r.path.clone(), r.secs / 86_400));
    }
    out
}

pub fn decay(last_day: u64, today: u64) -> f32 {
    let age = today.saturating_sub(last_day) as f32;
    0.5f32.powf(age / HALF_LIFE_DAYS)
}

/// The learned bonus per note, from past queries close to the current one.
/// `similarity` gives the cosine between the current query and a past one; `skip`
/// leaves one pair out (for the benchmark).
pub fn learned_bonus<F: Fn(&str) -> Option<f32>>(table: &Table, today: u64, similarity: F, skip: Option<&str>) -> HashMap<String, f32> {
    let mut out: HashMap<String, f32> = HashMap::new();
    for p in &table.pairs {
        if skip.is_some_and(|q| q == p.query) {
            continue;
        }
        let Some(sim) = similarity(&p.query) else { continue };
        if sim < SIMILAR {
            continue;
        }
        let strength = (p.count.min(3) as f32) / 3.0;
        let bonus = STEP * sim * strength * decay(p.last_day, today);
        let e = out.entry(p.path.clone()).or_insert(0.0);
        *e = (*e + bonus).min(CAP);
    }
    out
}

/// Keeps the bonus only for notes already within the window of the top score: it
/// breaks ties, it never lifts a distant note.
pub fn within_window(bonus: HashMap<String, f32>, base_scores: &HashMap<String, f32>) -> HashMap<String, f32> {
    let top = base_scores.values().cloned().fold(f32::MIN, f32::max);
    bonus.into_iter().filter(|(path, _)| base_scores.get(path).is_some_and(|s| top - s <= WINDOW)).collect()
}
