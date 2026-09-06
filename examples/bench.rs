//! Retrieval benchmark on the delivered code path.
//!
//! Families are reported SEPARATELY and never merged into one score. The metric
//! that counts is whether the expected NOTE is among the five returned, since that
//! is what `kept search` shows; the second says whether the right passage was
//! the one put forward for that note.
//!
//! Queries come from a JSON file (`KEPT_BENCH`, default `<root>/.kept/bench-queries.json`)
//! of the form `{"family": [{"path": "...", "ordinal": 0, "query": "..."}]}`. Write
//! them blind: the author sees only the target passage, never the title.
//!
//! Variants, by environment variable, measured on the same index:
//! `KEPT_NO_QUESTIONS=1` drops the question vectors, `KEPT_ID_BONUS=0` cuts the
//! lexical bonus, `KEPT_LEARN=0` cuts the learned bonus, `KEPT_LEXICAL=1` runs a
//! words-only baseline, `KEPT_CENTER=1` recenters vectors, `KEPT_MMR=<lambda>`
//! reranks by maximal marginal relevance.

use kept::index::Index;
use kept::similarity::rank_notes;
use std::path::PathBuf;

#[derive(serde::Deserialize)]
struct Case {
    path: String,
    ordinal: usize,
    query: String,
}

fn main() {
    let root = kept::paths::root();
    let model = kept::paths::model_dir();
    let state = kept::paths::state_dir(&root);
    let index = Index::load(&state.join("index.bin")).expect("index missing, run `kept index`");
    let embedder = kept::embedder::Embedder::load(&model).expect("model");

    let no_questions = std::env::var("KEPT_NO_QUESTIONS").is_ok();
    let chunks: Vec<(String, usize, Vec<f32>)> = index
        .iter()
        .filter(|(key, _)| !(no_questions && Index::is_question_key(key)))
        .map(|(key, v)| {
            let (path, ordinal) = Index::split_key(key);
            (path.to_string(), ordinal, v.to_vec())
        })
        .collect();

    let center = std::env::var("KEPT_CENTER").is_ok();
    let mmr: Option<f32> = std::env::var("KEPT_MMR").ok().and_then(|v| v.parse().ok());
    let lexical = std::env::var("KEPT_LEXICAL").is_ok();
    let dim = chunks.first().map(|c| c.2.len()).unwrap_or(0);
    let mut mean = vec![0f32; dim];
    for (_, _, v) in &chunks {
        for (m, x) in mean.iter_mut().zip(v) {
            *m += x / chunks.len() as f32;
        }
    }
    let recenter = |v: &[f32]| -> Vec<f32> {
        if !center {
            return v.to_vec();
        }
        let c: Vec<f32> = v.iter().zip(&mean).map(|(x, m)| x - m).collect();
        let n = c.iter().map(|x| x * x).sum::<f32>().sqrt().max(1e-9);
        c.into_iter().map(|x| x / n).collect()
    };
    let chunks: Vec<(String, usize, Vec<f32>)> = chunks.into_iter().map(|(p, o, v)| (p, o, recenter(&v))).collect();
    if center || mmr.is_some() || no_questions || lexical {
        println!("# variants: center={center} mmr={mmr:?} no_questions={no_questions} lexical={lexical}\n");
    }

    let bench = std::env::var("KEPT_BENCH").map(PathBuf::from).unwrap_or_else(|_| state.join("bench-queries.json"));
    let raw = std::fs::read_to_string(&bench).unwrap_or_else(|e| panic!("{}: {e}", bench.display()));
    let cases: std::collections::BTreeMap<String, Vec<Case>> = serde_json::from_str(&raw).expect("unreadable queries");

    println!("# model {}, {} vectors\n", model.file_name().unwrap().to_string_lossy(), chunks.len());
    println!("{:<12} {:>6} {:>8} {:>8} {:>10} {:>12}", "family", "cases", "top 1", "top 5", "passage", "top 5 tol.");

    let id_bonus: f32 = std::env::var("KEPT_ID_BONUS").ok().and_then(|v| v.parse().ok()).unwrap_or(0.04);
    // Link resolution, by name and by file stem, for the tolerant column.
    let mut by_key: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    for f in kept::hot::notes_of(&root) {
        let rel = kept::paths::relative(&f, &root);
        if let Ok(content) = std::fs::read_to_string(&f) {
            if let Some(name) = kept::note::Note::parse(&content).field("name") {
                by_key.insert(kept::check::link_key(name), rel.clone());
            }
        }
        by_key.insert(kept::check::link_key(&f.file_stem().unwrap_or_default().to_string_lossy()), rel);
    }

    // Usage feedback: applied as in production, leaving out the evaluated pair.
    // The table also yields a "real" family: the pairs themselves.
    let learn = std::env::var("KEPT_LEARN").map_or(true, |v| v != "0");
    let table = kept::feedback::Table::load(&state.join("feedback.json"));
    let today = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() / 86_400;
    let mut past: std::collections::HashMap<String, Vec<f32>> = std::collections::HashMap::new();
    if learn {
        for p in &table.pairs {
            if !past.contains_key(&p.query) {
                past.insert(p.query.clone(), recenter(&embedder.encode_query(&p.query).expect("encoding")));
            }
        }
    }
    let learned = |v: &[f32], query: &str, base: &[kept::similarity::NoteHit]| -> std::collections::HashMap<String, f32> {
        if !learn {
            return Default::default();
        }
        let b = kept::feedback::learned_bonus(&table, today, |q| past.get(q).map(|pv| kept::similarity::cosine(v, pv)), Some(query));
        let scores = base.iter().map(|h| (h.path.clone(), h.score)).collect();
        kept::feedback::within_window(b, &scores)
    };
    let mut cases = cases;
    if learn && !table.pairs.is_empty() {
        cases.insert("real".into(), table.pairs.iter().map(|p| Case { path: p.path.clone(), ordinal: 0, query: p.query.clone() }).collect());
    }

    for (family, list) in &cases {
        let (mut first, mut top5, mut right_chunk, mut top5_tolerant) = (0usize, 0usize, 0usize, 0usize);
        let mut misses: Vec<&str> = Vec::new();
        for case in list {
            let v = recenter(&embedder.encode_query(&case.query).expect("encoding"));
            let mut bonus = kept::similarity::lexical_bonus(&root, &kept::similarity::identifiers(&case.query), id_bonus);
            for (p, b) in learned(&v, &case.query, &rank_notes(&v, &chunks, 50)) {
                *bonus.entry(p).or_insert(0.0) += b;
            }
            let hits = if lexical {
                lexical_hits(&root, &case.query)
            } else {
                match mmr {
                    None => kept::similarity::rank_notes_with_bonus(&v, &chunks, 5, &bonus),
                    Some(lambda) => mmr_rerank(&v, &chunks, lambda),
                }
            };

            // Tolerant ground truth: the expected note, or one it cross-references or
            // that replaces it. Those are right answers to the same question.
            let accepted = accepted_paths(&root, &case.path, &by_key);
            if hits.iter().any(|h| accepted.contains(&h.path)) {
                top5_tolerant += 1;
            }
            match hits.iter().position(|h| h.path == case.path) {
                Some(0) => {
                    first += 1;
                    top5 += 1;
                }
                Some(_) => top5 += 1,
                None => misses.push(&case.query),
            }
            if let Some(h) = hits.iter().find(|h| h.path == case.path) {
                if h.ordinal == case.ordinal {
                    right_chunk += 1;
                }
            }
        }
        let n = list.len();
        println!(
            "{family:<12} {n:>6} {:>7.0}% {:>7.0}% {:>9.0}% {:>11.0}%",
            first as f32 * 100.0 / n as f32,
            top5 as f32 * 100.0 / n as f32,
            right_chunk as f32 * 100.0 / n as f32,
            top5_tolerant as f32 * 100.0 / n as f32,
        );
        for m in &misses {
            println!("             missed: {m}");
        }
    }
}

/// The expected note, plus those it cross-references ("See also") and the one that
/// replaces it.
fn accepted_paths(base: &std::path::Path, path: &str, by_key: &std::collections::HashMap<String, String>) -> std::collections::HashSet<String> {
    let mut out = std::collections::HashSet::new();
    out.insert(path.to_string());
    let Ok(content) = std::fs::read_to_string(base.join(path)) else { return out };
    let note = kept::note::Note::parse(&content);
    let mut linked: Vec<&str> = Vec::new();
    if let Some(sup) = note.field("superseded_by") {
        linked.push(sup);
    }
    for line in note.body().lines().filter(|l| l.starts_with("See also") || l.starts_with("Voir aussi")) {
        linked.push(line);
    }
    for text in linked {
        let mut rest = text;
        while let Some(start) = rest.find("[[") {
            let after = &rest[start + 2..];
            let Some(end) = after.find("]]") else { break };
            let target = after[..end].split(['|', '#']).next().unwrap_or("").trim();
            if let Some(p) = by_key.get(&kept::check::link_key(target)) {
                out.insert(p.clone());
            }
            rest = &after[end + 2..];
        }
    }
    out
}

/// Maximal marginal relevance over the twenty best notes.
fn mmr_rerank(query: &[f32], chunks: &[(String, usize, Vec<f32>)], lambda: f32) -> Vec<kept::similarity::NoteHit> {
    let pool = rank_notes(query, chunks, 20);
    let vec_of = |path: &str, ordinal: usize| chunks.iter().find(|(p, o, _)| p == path && *o == ordinal).map(|c| c.2.as_slice());
    let mut chosen: Vec<kept::similarity::NoteHit> = Vec::new();
    while chosen.len() < 5 && chosen.len() < pool.len() {
        let mut best: Option<(f32, usize)> = None;
        for (i, cand) in pool.iter().enumerate() {
            if chosen.iter().any(|c| c.path == cand.path) {
                continue;
            }
            let cv = vec_of(&cand.path, cand.ordinal).unwrap_or(&[]);
            let redundancy = chosen.iter().filter_map(|c| vec_of(&c.path, c.ordinal)).map(|v| kept::similarity::cosine(cv, v)).fold(0f32, f32::max);
            let score = lambda * cand.score - (1.0 - lambda) * redundancy;
            if best.is_none_or(|(b, _)| score > b) {
                best = Some((score, i));
            }
        }
        match best {
            Some((_, i)) => chosen.push(pool[i].clone()),
            None => break,
        }
    }
    chosen
}

/// Words-only baseline: a note scores the number of distinct query words it
/// contains (three characters or more, case-insensitive), then total occurrences.
fn lexical_hits(base: &std::path::Path, query: &str) -> Vec<kept::similarity::NoteHit> {
    let words: Vec<String> =
        query.split(|c: char| !c.is_alphanumeric() && c != '_' && c != '-').map(|w| w.to_lowercase()).filter(|w| w.chars().count() >= 3).collect();
    let mut scored: Vec<(usize, usize, String)> = Vec::new();
    for f in kept::hot::notes_of(base) {
        let Ok(content) = std::fs::read_to_string(&f) else { continue };
        let low = content.to_lowercase();
        let distinct = words.iter().filter(|w| low.contains(w.as_str())).count();
        if distinct == 0 {
            continue;
        }
        let total: usize = words.iter().map(|w| low.matches(w.as_str()).count()).sum();
        scored.push((distinct, total, kept::paths::relative(&f, base)));
    }
    scored.sort_by(|a, b| b.0.cmp(&a.0).then(b.1.cmp(&a.1)).then(a.2.cmp(&b.2)));
    scored.into_iter().take(5).map(|(d, _, path)| kept::similarity::NoteHit { path, ordinal: 0, score: d as f32 }).collect()
}
