//! What reaches the model's context to answer a question: the engine against a
//! plain grep, on the benchmark queries, and the hot index against its unbounded
//! equivalent. Sizes are characters, and tokens are estimated at four characters per
//! token, the usual rule for English text.
//!
//! Two ways to reach the note that answers a query, both run for real:
//!
//! - **grep**: one `grep -rli <word> <root>` per word of the query (three characters
//!   or more), whose lists of files enter the context, the notes ranked by the number
//!   of words they contain (the words-only baseline of the benchmark), then read in
//!   that order until the expected note is reached, five notes at most.
//! - **engram**: `engram search`, whose five lines enter the context, then `engram
//!   read` of the expected note when the search returned it. And `engram answer`
//!   alone, the passages ready to cite, which is what the hook and the MCP tool give.
//!
//! Queries come from the same file as `bench` (`ENGRAM_BENCH`, default
//! `<root>/.engram/bench-queries.json`). `ENGRAM_BIN` names the binary (`engram`).

use engrams::note::Note;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(serde::Deserialize)]
struct Case {
    path: String,
    query: String,
}

fn tokens(chars: usize) -> usize {
    chars.div_ceil(4)
}

fn run(bin: &str, args: &[&str]) -> String {
    let out = Command::new(bin).args(args).env("ENGRAM_NO_DAEMON", "1").output().expect("running the binary");
    String::from_utf8_lossy(&out.stdout).into_owned()
}

fn read_note(root: &Path, rel: &str) -> String {
    std::fs::read_to_string(root.join(rel)).unwrap_or_default()
}

/// The words of a query as grep would take them, three characters or more.
fn words(query: &str) -> Vec<String> {
    query.split(|c: char| !c.is_alphanumeric() && c != '_' && c != '-').map(str::to_lowercase).filter(|w| w.chars().count() >= 3).collect()
}

struct Tally {
    /// What the tool printed.
    output: usize,
    /// The notes read afterwards.
    notes: usize,
    reached: usize,
}

impl Tally {
    const ZERO: Tally = Tally { output: 0, notes: 0, reached: 0 };
}

fn main() {
    let root = engrams::paths::root();
    let bin = std::env::var("ENGRAM_BIN").unwrap_or_else(|_| "engram".into());
    let file = std::env::var("ENGRAM_BENCH").map(PathBuf::from).unwrap_or_else(|_| engrams::paths::state_dir(&root).join("bench-queries.json"));
    let raw = std::fs::read_to_string(&file).unwrap_or_else(|e| panic!("{}: {e}", file.display()));
    let families: BTreeMap<String, Vec<Case>> = serde_json::from_str(&raw).expect("bench-queries.json");
    let cases: Vec<&Case> = families.values().flatten().collect();
    println!("# {} queries, root {}\n", cases.len(), root.display());

    let (mut grep, mut search, mut answer) = (Tally::ZERO, Tally::ZERO, Tally::ZERO);
    let expected_of = |case: &Case| root.join(&case.path).display().to_string();
    for case in &cases {
        // grep: one list of files per word, the notes ranked by distinct words matched.
        let mut per_file: BTreeMap<String, usize> = BTreeMap::new();
        for word in words(&case.query) {
            let out = run("grep", &["-rli", "--include=*.md", "--exclude-dir=.engram", "-e", &word, &root.display().to_string()]);
            grep.output += out.len();
            for file in out.lines() {
                *per_file.entry(file.to_string()).or_default() += 1;
            }
        }
        let mut ranked: Vec<(usize, String)> = per_file.into_iter().map(|(f, n)| (n, f)).collect();
        ranked.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
        let expected = expected_of(case);
        for (_, file) in ranked.iter().take(5) {
            grep.notes += std::fs::read_to_string(file).map(|s| s.len()).unwrap_or(0);
            if *file == expected {
                grep.reached += 1;
                break;
            }
        }

        // engram search, then the note if the search returned it.
        let out = run(&bin, &["search", &case.query]);
        let hit = out.lines().any(|l| l.contains(&case.path));
        search.output += out.len();
        if hit {
            search.notes += read_note(&root, &case.path).len();
            search.reached += 1;
        }

        // engram answer: passages only, nothing read afterwards.
        let out = run(&bin, &["answer", &case.query]);
        answer.output += out.len();
        answer.reached += usize::from(out.contains(&case.path));
    }
    let n = cases.len();
    println!("| path to the answer | tool output | notes read | total per query | tokens (est.) | expected note reached |");
    println!("|---|---|---|---|---|---|");
    for (name, t) in [
        ("grep per word, then the notes in grep order (5 at most)", &grep),
        ("engram search, then engram read", &search),
        ("engram answer, passages only", &answer),
    ] {
        let total = (t.output + t.notes) / n;
        println!("| {name} | {} | {} | {total} | {} | {} / {n} |", t.output / n, t.notes / n, tokens(total), t.reached);
    }

    // The hot index: what a session loads, bounded, against the same list unbounded
    // and against every note.
    let mut bounded = 0usize;
    let mut unbounded = 0usize;
    let mut all_notes = 0usize;
    let mut projects = 0usize;
    for project in engrams::hot::projects(&root) {
        let dir = root.join(&project);
        let index = dir.join("MEMORY.md");
        if !index.exists() {
            continue;
        }
        projects += 1;
        bounded += std::fs::metadata(&index).map(|m| m.len() as usize).unwrap_or(0);
        for f in engrams::hot::notes_of(&dir) {
            let Ok(content) = std::fs::read_to_string(&f) else { continue };
            all_notes += content.len();
            let note = Note::parse(&content);
            if note.is_active() {
                let rel = f.strip_prefix(&dir).unwrap_or(&f).to_string_lossy().into_owned();
                unbounded +=
                    format!("- [{}]({}): {}\n", engrams::hot::display_name(&f, &note), rel, note.field("description").unwrap_or_default().trim()).len();
            }
        }
    }
    println!("\n| what a session could load | chars | tokens (est.) |");
    println!("|---|---|---|");
    println!("| the hot indexes as generated, bounded at {} bytes each, {projects} projects | {bounded} | {} |", engrams::hot::BOUND, tokens(bounded));
    println!("| one line per active note, no bound | {unbounded} | {} |", tokens(unbounded));
    println!("| every note of the corpus | {all_notes} | {} |", tokens(all_notes));
}
