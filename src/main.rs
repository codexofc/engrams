//! `engram`: local semantic memory for coding agents.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use engrams::chunking::{budget_for, split};
use engrams::embedder::Embedder;
use engrams::index::{content_hash, Header, Index};
use engrams::note::Note;
use engrams::paths::{self, relative};

const USAGE: &str = "\
engram, local semantic memory for coding agents

Setup
  engram init [<dir>] [--model <hf-repo>] [--no-download]
                          create the notes directory, remember it, download the model
  engram setup <claude-code|opencode|kandev>
                          wire the hook and the MCP server into a tool

Search and read
  engram search <words> [--archives]    five notes at most
  engram answer <question> [-n 3] [--json]   the best passages, bounded
  engram context <topic> [--out <file>]  a markdown brief for an agent
  engram read <name> [--project <p>]     print a whole note

Write
  engram write <family/project> <name> --type <t> --description <d> [--depends-on <x>] [--source <s>] [--force]
                          write a note, body on standard input
  engram append <name>    add a paragraph (standard input), verified today
  engram verify <name>    mark the note verified today
  engram supersede <old> <new> --type <t> --description <d>
                          replace: new note, old one archived, links rewritten
  engram link <a> <b>     cross-reference two notes
  engram learn [<query> <name>] [--show] [--forget <query>]
                          usage feedback: explicit, or derived from reads the search had missed

Maintenance
  engram index            embed changed notes, update the index and the MEMORY.md files
  engram regen [project]  regenerate the MEMORY.md files
  engram check            validate frontmatters, links, bounds, truncation, duplicates
  engram secrets [dir]    exit 1 if any .md looks like it contains a secret
  engram curation         markdown report of what deserves a review
  engram since [days]     what changed (git), per project, 7 days by default
  engram why <name>       provenance: frontmatter, git history, citing notes
  engram list             families, projects and note counts

Integration
  engram hook             Claude Code UserPromptSubmit hook: JSON on stdin, passages on stdout
  engram mcp              MCP server over stdio (search, read, answer, write, append, link, learn)
  engram serve            warm process on a Unix socket, exits after ENGRAM_IDLE s (300)
  engram stop | status    stop or inspect the warm process

Environment
  ENGRAM_ROOT             notes directory (default: the one written by `engram init`, else ~/engram)
  ENGRAM_MODEL            model directory (default ~/.engram/models/<model>)
  ENGRAM_PRECISION        q8 (default) or f32 for the linear layers
  ENGRAM_NO_DAEMON        never start the warm process
  ENGRAM_IDLE, ENGRAM_WATCH   idle timeout and background refresh period of the warm process
  ENGRAM_QUESTIONS_CMD    command writing the questions a paragraph answers (text on stdin, one per line)
  ENGRAM_QUESTIONS_BATCH  paragraphs sent per pass (index: unlimited, warm process: 4)
";

fn main() -> ExitCode {
    // A closed pipe downstream (`engram list | head`) ends the process quietly.
    unsafe {
        libc::signal(libc::SIGPIPE, libc::SIG_DFL);
    }
    let args: Vec<String> = std::env::args().skip(1).collect();
    let started = std::time::Instant::now();
    let outcome = match args.first().map(String::as_str) {
        Some("init") => run_init(&args),
        Some("setup") if args.len() > 1 => run_setup(&args[1]),
        Some("index") => run_index(),
        Some("search") if args.len() > 1 => run_search(&args),
        Some("answer") if args.len() > 1 => run_answer(&args),
        Some("context") if args.len() > 1 => run_context(&args),
        Some("read") if args.len() > 1 => read_note(&args[1], flag_value(&args, "--project")).map(|t| print!("{t}")),
        Some("write") if args.len() > 2 => run_write(&args),
        Some("append") if args.len() > 1 => run_append(&args),
        Some("verify") if args.len() > 1 => run_verify(&args),
        Some("supersede") if args.len() > 2 => run_supersede(&args),
        Some("link") if args.len() > 2 => link_notes(&args[1], &args[2]).map(|t| println!("{t}")),
        Some("learn") => run_learn(&args),
        Some("regen") => run_regen(args.get(1).map(String::as_str)),
        Some("check") => run_check(),
        Some("secrets") => run_secrets(args.get(1).map(PathBuf::from)),
        Some("curation") => run_curation(),
        Some("since") => run_since(args.get(1).map(String::as_str).unwrap_or("7")),
        Some("why") if args.len() > 1 => run_why(&args[1]),
        Some("list") => run_list(),
        Some("hook") => run_hook(),
        Some("mcp") => run_mcp(),
        Some("serve") => run_serve(),
        Some("stop") => {
            use std::io::Write;
            if let Ok(mut st) = std::os::unix::net::UnixStream::connect(socket_path()) {
                let _ = st.write_all(b"stop\n");
            }
            Ok(())
        }
        Some("status") => run_status(),
        Some("version") | Some("--version") | Some("-V") => {
            println!("engram {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
        _ => {
            eprint!("{USAGE}");
            return ExitCode::from(2);
        }
    };
    log_usage(&args, started, outcome.is_ok());
    match outcome {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("engram: {e}");
            ExitCode::FAILURE
        }
    }
}

// ----------------------------------------------------------------------------- paths

fn root() -> PathBuf {
    paths::root()
}

fn model_dir() -> PathBuf {
    paths::model_dir()
}

/// A derived file under `<root>/.engram/`, the directory created on first use.
fn state_file(name: &str) -> PathBuf {
    let dir = paths::state_dir(&root());
    let _ = std::fs::create_dir_all(&dir);
    dir.join(name)
}

fn socket_path() -> PathBuf {
    state_file("engram.sock")
}

/// Seconds since the epoch, without a dependency.
fn now_secs() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

fn today() -> String {
    engrams::hot::ymd(now_secs())
}

fn flag_value<'a>(args: &'a [String], flag: &str) -> Option<&'a str> {
    args.iter().position(|a| a == flag).and_then(|i| args.get(i + 1)).map(String::as_str)
}

/// The words of a command that are neither flags nor their values.
fn positional_query(args: &[String], valued_flags: &[&str]) -> String {
    let mut words = Vec::new();
    let mut skip = false;
    for a in &args[1..] {
        if skip {
            skip = false;
            continue;
        }
        if valued_flags.contains(&a.as_str()) {
            skip = true;
            continue;
        }
        if a.starts_with("--") {
            continue;
        }
        words.push(a.as_str());
    }
    words.join(" ")
}

/// One line per call, to decide on real cadence whether the warm process is worth it.
fn log_usage(args: &[String], started: std::time::Instant, ok: bool) {
    use std::io::Write;
    let Some(command) = args.first() else { return };
    if matches!(command.as_str(), "mcp" | "serve" | "hook") {
        return;
    }
    let line = format!("{}\t{command}\t{}ms\t{}\n", now_secs(), started.elapsed().as_millis(), if ok { "ok" } else { "error" });
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(state_file("usage.log")) {
        let _ = f.write_all(line.as_bytes());
    }
}

fn collect_notes(base: &Path) -> Vec<PathBuf> {
    engrams::hot::notes_of(base)
}

// ----------------------------------------------------------------------------- index

/// What a refresh pass did.
#[derive(Default)]
struct Refresh {
    notes: usize,
    encoded: usize,
    chunks: usize,
    unchanged: usize,
    truncated: usize,
    removed: usize,
    questions: usize,
}

impl Refresh {
    fn changed(&self) -> bool {
        self.encoded > 0 || self.removed > 0 || self.questions > 0
    }
}

/// Opens the index if it was produced under the same conditions, else starts empty.
/// Any header mismatch rebuilds instead of mixing vectors of two models.
fn open_index(model: &Path, embedder: &Embedder) -> Result<(PathBuf, Index), String> {
    let header = Header {
        model: model.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default(),
        weights_hash: weights_fingerprint(model)?,
        dim: embedder.dim(),
        pooling: format!("{:?}", embedder.pooling()).to_lowercase(),
    };
    let path = state_file("index.bin");
    let index = match std::fs::metadata(&path) {
        Err(_) => Index::new(header),
        Ok(_) => match Index::load(&path) {
            Ok(i) if i.matches(&header) => i,
            Ok(_) => {
                eprintln!("engram: index produced by another model, full rebuild");
                Index::new(header)
            }
            Err(e) => {
                eprintln!("engram: unreadable index ({e}), full rebuild");
                Index::new(header)
            }
        },
    };
    Ok((path, index))
}

/// Weights fingerprint, so the index refuses vectors from another model file. The
/// size is enough: this detects a change, it does not defend against tampering.
fn weights_fingerprint(dir: &Path) -> Result<String, String> {
    let meta = std::fs::metadata(dir.join("model.safetensors")).map_err(|e| format!("weights not found: {e}"))?;
    Ok(meta.len().to_string())
}

/// How many paragraphs a pass may send to the questions command. `engram index`
/// has no limit; the warm process takes a few per cycle so a search never waits.
fn questions_budget(default: usize) -> usize {
    std::env::var("ENGRAM_QUESTIONS_BATCH").ok().and_then(|v| v.parse().ok()).unwrap_or(default)
}

/// Brings the index level with the notes: embeds what changed, removes what
/// disappeared, keeps the rest. The same pass serves `index` and `search`.
fn refresh_index(embedder: &Embedder, index: &mut Index, base: &Path, max_generate: usize) -> Result<Refresh, String> {
    let files = collect_notes(base);
    let budget = budget_for(embedder.window());
    let mut stats = Refresh { notes: files.len(), ..Refresh::default() };
    let (mut keys, mut texts): (Vec<(String, u64)>, Vec<String>) = (Vec::new(), Vec::new());
    let mut expected: std::collections::HashSet<String> = std::collections::HashSet::new();

    // Generated questions: enabled by ENGRAM_QUESTIONS_CMD, cached by paragraph
    // fingerprint, indexed as `path#ordinal?k`.
    let questions_cmd = std::env::var("ENGRAM_QUESTIONS_CMD").ok().filter(|c| !c.trim().is_empty());
    let mut store = engrams::questions::Store::load(&state_file("questions.json"));
    let mut live_hashes: std::collections::HashSet<u64> = std::collections::HashSet::new();
    // (paragraph fingerprint, text, path, ordinal, note fingerprint)
    let mut pending: Vec<(u64, String, String, usize, u64)> = Vec::new();

    for file in &files {
        let Ok(content) = std::fs::read_to_string(file) else { continue };
        let path = relative(file, base);
        let fingerprint = content_hash(&content);
        let note = Note::parse(&content);
        let pieces = split(note.body(), budget);
        if pieces.is_empty() {
            continue;
        }
        let name = note.field("name").unwrap_or_default();
        let description = note.field("description").unwrap_or_default();
        let mut touched = false;
        // One entry per CHUNK: a single vector for a long note represents its
        // dominant topic, not its details.
        for chunk in &pieces {
            let text = chunk.with_context(name, description);
            let key = format!("{path}#{}", chunk.ordinal);
            expected.insert(key.clone());
            if !index.is_fresh(&key, fingerprint) {
                keys.push((key, fingerprint));
                texts.push(text.clone());
                touched = true;
            }
            let hash = content_hash(&text);
            live_hashes.insert(hash);
            match store.get(hash) {
                Some(questions) => {
                    for (i, q) in questions.iter().enumerate() {
                        let qkey = format!("{path}#{}?{i}", chunk.ordinal);
                        expected.insert(qkey.clone());
                        if !index.is_fresh(&qkey, fingerprint) {
                            keys.push((qkey, fingerprint));
                            texts.push(q.clone());
                            touched = true;
                        }
                    }
                }
                None if questions_cmd.is_some() && pending.len() < max_generate => {
                    pending.push((hash, text, path.clone(), chunk.ordinal, fingerprint));
                }
                None => {}
            }
        }
        if touched {
            stats.encoded += 1;
        } else {
            stats.unchanged += 1;
        }
    }

    if let (Some(cmd), false) = (&questions_cmd, pending.is_empty()) {
        // Batched, cache saved after each batch: a whole-corpus generation is an
        // hour of remote calls and must survive an interruption.
        for batch in pending.chunks(8) {
            let prompts: Vec<String> = batch.iter().map(|p| p.1.clone()).collect();
            let results = engrams::questions::generate_many(cmd, &prompts, 4);
            for ((hash, _, path, ordinal, fingerprint), result) in batch.iter().zip(results) {
                match result {
                    Ok(questions) => {
                        for (i, q) in questions.iter().enumerate() {
                            let qkey = format!("{path}#{ordinal}?{i}");
                            expected.insert(qkey.clone());
                            keys.push((qkey, *fingerprint));
                            texts.push(q.clone());
                        }
                        stats.questions += questions.len();
                        store.insert(*hash, questions);
                    }
                    Err(e) => eprintln!("engram: {path}#{ordinal}: {e}"),
                }
            }
            store.save()?;
        }
    }
    store.prune(&live_hashes);
    store.save()?;
    stats.chunks = keys.len();

    // Chunks of a deleted or shortened note do not survive: a ghost keeps a
    // plausible score and nothing would flag it.
    stats.removed = index.retain(|key| expected.contains(key));

    if !keys.is_empty() {
        let threads = std::thread::available_parallelism().map_or(1, |n| n.get());
        let vectors = embedder.encode_many_checked(&texts, threads)?;
        for ((key, fingerprint), (vector, truncated)) in keys.iter().zip(&vectors) {
            if *truncated {
                stats.truncated += 1;
            }
            index.try_push(key, *fingerprint, vector)?;
        }
    }
    Ok(stats)
}

fn run_index() -> Result<(), String> {
    let root = root();
    let model = model_dir();
    let embedder = Embedder::load(&model)?;
    let (path, mut index) = open_index(&model, &embedder)?;
    let started = std::time::Instant::now();
    let stats = refresh_index(&embedder, &mut index, &root, questions_budget(usize::MAX))?;
    if stats.changed() {
        index.save(&path)?;
    }
    for p in engrams::hot::projects(&root) {
        engrams::hot::write(&root, &p)?;
    }
    println!(
        "{} notes, {} embedded as {} chunks, {} unchanged, {} chunks removed, {} questions generated, in {:.1} s",
        stats.notes,
        stats.encoded,
        stats.chunks,
        stats.unchanged,
        stats.removed,
        stats.questions,
        started.elapsed().as_secs_f32()
    );
    if stats.truncated > 0 {
        println!("{} of {} chunks exceed the model window: paragraphs longer than the budget, kept whole then cut by the model", stats.truncated, stats.chunks);
    }
    Ok(())
}

// ----------------------------------------------------------------------------- engine

/// The loaded engine: model, index, feedback. What a warm process keeps between two
/// requests, and what an isolated call builds then drops.
struct Engine {
    embedder: Embedder,
    index: Index,
    index_path: PathBuf,
    base: PathBuf,
    feedback: engrams::feedback::Table,
    /// Vectors of the feedback queries, cached in `feedback.bin`.
    feedback_vectors: std::collections::HashMap<String, Vec<f32>>,
}

/// A passage ready to be read, as `answer` and `context` return it.
#[derive(serde::Serialize, serde::Deserialize)]
struct Passage {
    path: String,
    name: String,
    score: f32,
    verified: Option<String>,
    text: String,
}

type Chunks = Vec<(String, usize, Vec<f32>)>;

impl Engine {
    fn open() -> Result<Self, String> {
        let root = root();
        let model = model_dir();
        let embedder = Embedder::load(&model)?;
        let (index_path, index) = open_index(&model, &embedder)?;
        let mut engine = Engine {
            embedder,
            index,
            index_path,
            base: root,
            feedback: engrams::feedback::Table::load(&state_file("feedback.json")),
            feedback_vectors: std::collections::HashMap::new(),
        };
        engine.load_feedback_vectors()?;
        Ok(engine)
    }

    /// Query vectors of the feedback table, from the cache plus the new ones. The
    /// cache carries the index header, so it rebuilds with it if the model changes.
    fn load_feedback_vectors(&mut self) -> Result<(), String> {
        let path = state_file("feedback.bin");
        let header = self.index.header().clone();
        let mut cache = Index::load(&path).ok().filter(|i| i.matches(&header)).unwrap_or_else(|| Index::new(header));
        let mut added = false;
        for p in &self.feedback.pairs {
            if cache.vector(&p.query).is_none() {
                let v = self.embedder.encode(&p.query)?;
                cache.try_push(&p.query, 0, &v)?;
                added = true;
            }
        }
        if added {
            cache.save(&path)?;
        }
        self.feedback_vectors = cache.iter().map(|(k, v)| (k.to_string(), v.to_vec())).collect();
        Ok(())
    }

    fn learn_from_logs(&mut self) -> Result<usize, String> {
        let n = learn_from_logs(&mut self.feedback)?;
        if n > 0 {
            self.load_feedback_vectors()?;
        }
        Ok(n)
    }

    fn chunks(&self) -> Chunks {
        self.index
            .iter()
            .map(|(key, v)| {
                let (path, ordinal) = Index::split_key(key);
                (path.to_string(), ordinal, v.to_vec())
            })
            .collect()
    }

    /// Learned bonus: past queries close to this one lift the note that had to be
    /// read. Bounded, decaying, and applied only within the window of the top score.
    fn learned_bonus(&self, query: &[f32]) -> std::collections::HashMap<String, f32> {
        let today = now_secs() / 86_400;
        engrams::feedback::learned_bonus(&self.feedback, today, |q| self.feedback_vectors.get(q).map(|v| engrams::similarity::cosine(query, v)), None)
    }

    /// Every bonus of a query: lexical on identifiers, learned from usage.
    fn bonuses(&self, query: &str, vector: &[f32], chunks: &Chunks) -> std::collections::HashMap<String, f32> {
        let step: f32 = std::env::var("ENGRAM_ID_BONUS").ok().and_then(|v| v.parse().ok()).unwrap_or(0.04);
        let mut bonus = engrams::similarity::lexical_bonus(&self.base, &engrams::similarity::identifiers(query), step);
        if std::env::var("ENGRAM_LEARN").map_or(true, |v| v != "0") {
            let base_scores: std::collections::HashMap<String, f32> =
                engrams::similarity::rank_notes(vector, chunks, 50).into_iter().map(|h| (h.path, h.score)).collect();
            for (p, b) in engrams::feedback::within_window(self.learned_bonus(vector), &base_scores) {
                *bonus.entry(p).or_insert(0.0) += b;
            }
        }
        bonus
    }

    fn journal_search(&self, query: &str, shown: &[String]) {
        use std::io::Write;
        if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(state_file("searches.log")) {
            let _ = writeln!(f, "{}\t{}\t{}", now_secs(), query.replace(['\t', '\n'], " "), shown.join(","));
        }
    }

    /// Brings the index level with the notes before serving. No question generation
    /// here: it would wait on a remote model while a search waits.
    fn refresh(&mut self) -> Result<Refresh, String> {
        self.refresh_with(0)
    }

    /// Refreshes, allowing `questions_budget` paragraphs of generation, and
    /// regenerates the hot indexes when something moved.
    fn refresh_with(&mut self, questions_budget: usize) -> Result<Refresh, String> {
        let stats = refresh_index(&self.embedder, &mut self.index, &self.base, questions_budget)?;
        if stats.changed() {
            self.index.save(&self.index_path)?;
            if questions_budget > 0 {
                for p in engrams::hot::projects(&self.base) {
                    engrams::hot::write(&self.base, &p)?;
                }
            }
        }
        if questions_budget > 0 {
            let _ = self.learn_from_logs();
        }
        Ok(stats)
    }

    fn search(&self, query: &str, with_archives: bool) -> Result<String, String> {
        if self.index.is_empty() {
            return Err(format!("no indexable note under {}", self.base.display()));
        }
        let vector = self.embedder.encode(query)?;
        let chunks = self.chunks();
        let bonus = self.bonuses(query, &vector, &chunks);
        let mut out = String::new();
        let mut shown = Vec::new();
        let mut hidden = 0usize;
        for hit in engrams::similarity::rank_notes_with_bonus(&vector, &chunks, 25, &bonus) {
            if shown.len() == 5 {
                break;
            }
            let content = std::fs::read_to_string(self.base.join(&hit.path)).unwrap_or_default();
            if !with_archives && !Note::parse(&content).is_active() {
                hidden += 1;
                continue;
            }
            shown.push(hit);
        }
        self.journal_search(query, &shown.iter().map(|h| h.path.clone()).collect::<Vec<_>>());
        for hit in shown {
            let content = std::fs::read_to_string(self.base.join(&hit.path)).unwrap_or_default();
            let note = Note::parse(&content);
            // Freshness is read from the frontmatter at search time, never from the
            // index, which would otherwise become a second source of truth.
            let state = if note.is_active() { "" } else { " [archived]" };
            let verified = note.field("verified").map(|d| format!(" verified {d}")).unwrap_or_default();
            out.push_str(&format!("{:.3}  {}{state}{verified}\n       {}\n", hit.score, hit.path, note.field("description").unwrap_or("(no description)")));
            if let Some(chunk) = split(note.body(), budget_for(self.embedder.window())).into_iter().find(|c| c.ordinal == hit.ordinal) {
                let excerpt: String = chunk.text.chars().take(180).collect();
                out.push_str(&format!("       \"{}\"\n", excerpt.replace('\n', " ")));
            }
        }
        if hidden > 0 {
            out.push_str(&format!("({hidden} archived note(s) hidden, `--archives` shows them)\n"));
        }
        Ok(out)
    }

    /// The `n` best passages, bounded to `max_chars` in total.
    fn answer(&self, query: &str, n: usize, max_chars: usize) -> Result<Vec<Passage>, String> {
        let vector = self.embedder.encode(query)?;
        let chunks = self.chunks();
        let bonus = self.bonuses(query, &vector, &chunks);
        let mut out = Vec::new();
        let mut used = 0usize;
        let ranked = engrams::similarity::rank_notes_with_bonus(&vector, &chunks, 25, &bonus);
        self.journal_search(query, &ranked.iter().take(5).map(|h| h.path.clone()).collect::<Vec<_>>());
        for hit in ranked {
            if out.len() == n {
                break;
            }
            let Ok(content) = std::fs::read_to_string(self.base.join(&hit.path)) else { continue };
            let note = Note::parse(&content);
            if !note.is_active() {
                continue;
            }
            let Some(chunk) = split(note.body(), budget_for(self.embedder.window())).into_iter().find(|c| c.ordinal == hit.ordinal) else { continue };
            let room = max_chars.saturating_sub(used);
            if room < 200 {
                break;
            }
            let text: String = chunk.text.chars().take(room).collect();
            used += text.chars().count();
            out.push(Passage {
                name: engrams::hot::display_name(&self.base.join(&hit.path), &note),
                path: hit.path,
                score: hit.score,
                verified: note.field("verified").map(str::to_string),
                text,
            });
        }
        Ok(out)
    }
}

fn render_passages(query: &str, passages: &[Passage], json: bool) -> String {
    if json {
        return serde_json::to_string_pretty(passages).unwrap_or_default() + "\n";
    }
    let mut out = format!("# What the memory says about: {query}\n\n");
    for p in passages {
        let v = p.verified.as_deref().map(|d| format!(", verified {d}")).unwrap_or_default();
        out.push_str(&format!("## {} ({}{v}, {:.2})\n\n{}\n\n", p.name, p.path, p.score, p.text.trim()));
    }
    if passages.is_empty() {
        out.push_str("(nothing close)\n");
    }
    out
}

// ----------------------------------------------------------------------------- warm process

/// Modification time of the binary: a warm process started by an older version
/// steps aside for the new one.
fn exe_stamp() -> String {
    std::env::current_exe()
        .and_then(std::fs::metadata)
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_secs().to_string())
        .unwrap_or_default()
}

fn ask_daemon(request: &str) -> Option<String> {
    use std::io::{Read, Write};
    let mut stream = std::os::unix::net::UnixStream::connect(socket_path()).ok()?;
    stream.set_read_timeout(Some(std::time::Duration::from_secs(60))).ok()?;
    stream.write_all(request.as_bytes()).ok()?;
    stream.shutdown(std::net::Shutdown::Write).ok()?;
    let mut reply = String::new();
    stream.read_to_string(&mut reply).ok()?;
    let (status, body) = reply.split_once('\n')?;
    (status == "ok").then(|| body.to_string())
}

fn ask_daemon_search(query: &str, with_archives: bool) -> Option<String> {
    ask_daemon(&format!("search\t{}\t{}\t{}\n", exe_stamp(), u8::from(with_archives), query.replace(['\n', '\t'], " ")))
}

fn ask_daemon_answer(query: &str, n: usize, max_chars: usize, json: bool) -> Option<String> {
    ask_daemon(&format!("answer\t{}\t{n}:{max_chars}:{}\t{}\n", exe_stamp(), u8::from(json), query.replace(['\n', '\t'], " ")))
}

fn spawn_daemon() {
    if std::env::var_os("ENGRAM_NO_DAEMON").is_some() {
        return;
    }
    if let Ok(exe) = std::env::current_exe() {
        let _ = std::process::Command::new(exe)
            .arg("serve")
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn();
    }
}

/// Warm process: model and index stay loaded, requests arrive on a Unix socket, the
/// process exits after ENGRAM_IDLE seconds without a request. Each connection is
/// served in its own thread under a read lock; refreshes take the write lock.
fn run_serve() -> Result<(), String> {
    use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
    use std::sync::{Arc, RwLock};
    let path = socket_path();
    let _ = std::fs::remove_file(&path);
    let listener = std::os::unix::net::UnixListener::bind(&path).map_err(|e| format!("socket {}: {e}", path.display()))?;
    listener.set_nonblocking(true).map_err(|e| e.to_string())?;
    let idle_limit = std::env::var("ENGRAM_IDLE").ok().and_then(|v| v.parse().ok()).map_or(std::time::Duration::from_secs(300), std::time::Duration::from_secs);
    let watch = std::env::var("ENGRAM_WATCH").ok().and_then(|v| v.parse().ok()).map_or(std::time::Duration::from_secs(30), std::time::Duration::from_secs);
    let engine = Arc::new(RwLock::new(Engine::open()?));
    let stamp = exe_stamp();
    let started = std::time::Instant::now();
    let last_activity = Arc::new(AtomicU64::new(0));
    let served = Arc::new(AtomicUsize::new(0));
    let refreshed = Arc::new(AtomicUsize::new(0));
    let stop = Arc::new(AtomicBool::new(false));
    let mut last_refresh = std::time::Instant::now();
    loop {
        if stop.load(Ordering::Relaxed) {
            break;
        }
        match listener.accept() {
            Ok((stream, _)) => {
                last_activity.store(started.elapsed().as_secs(), Ordering::Relaxed);
                let _ = stream.set_nonblocking(false);
                let (engine, stamp, served, refreshed, stop) =
                    (Arc::clone(&engine), stamp.clone(), Arc::clone(&served), Arc::clone(&refreshed), Arc::clone(&stop));
                let (uptime, since_refresh) = (started.elapsed().as_secs(), last_refresh.elapsed().as_secs());
                std::thread::spawn(move || {
                    serve_one(stream, &engine, &stamp, &served, &refreshed, &stop, uptime, since_refresh);
                });
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                let idle = started.elapsed().as_secs().saturating_sub(last_activity.load(Ordering::Relaxed));
                if idle > idle_limit.as_secs() {
                    break;
                }
                if last_refresh.elapsed() > watch {
                    if let Ok(mut e) = engine.write() {
                        let _ = e.refresh_with(questions_budget(4));
                    }
                    last_refresh = std::time::Instant::now();
                    refreshed.fetch_add(1, Ordering::Relaxed);
                }
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
            Err(e) => return Err(format!("accept: {e}")),
        }
    }
    let _ = std::fs::remove_file(&path);
    Ok(())
}

/// One connection: a request line, a reply, then the socket closes.
#[allow(clippy::too_many_arguments)]
fn serve_one(
    stream: std::os::unix::net::UnixStream,
    engine: &std::sync::RwLock<Engine>,
    stamp: &str,
    served: &std::sync::atomic::AtomicUsize,
    refreshed: &std::sync::atomic::AtomicUsize,
    stop: &std::sync::atomic::AtomicBool,
    uptime: u64,
    since_refresh: u64,
) {
    use std::io::{BufRead, Write};
    use std::sync::atomic::Ordering;
    let mut reader = std::io::BufReader::new(&stream);
    let mut line = String::new();
    if reader.read_line(&mut line).is_err() {
        return;
    }
    let mut w = &stream;
    let fields: Vec<&str> = line.trim_end_matches('\n').splitn(4, '\t').collect();
    fn poisoned<T>(_: T) -> String {
        "poisoned lock".to_string()
    }
    let reply = match fields.as_slice() {
        ["search", client_stamp, archives, query] if *client_stamp == stamp => {
            served.fetch_add(1, Ordering::Relaxed);
            let fresh = engine.write().map_err(poisoned).and_then(|mut e| e.refresh().map(|_| ()));
            let answer = fresh.and_then(|_| engine.read().map_err(poisoned).and_then(|e| e.search(query, *archives == "1")));
            match answer {
                Ok(text) => format!("ok\n{text}"),
                Err(e) => format!("error\n{e}\n"),
            }
        }
        ["answer", client_stamp, params, query] if *client_stamp == stamp => {
            served.fetch_add(1, Ordering::Relaxed);
            let mut p = params.split(':');
            let n: usize = p.next().and_then(|v| v.parse().ok()).unwrap_or(3);
            let max_chars: usize = p.next().and_then(|v| v.parse().ok()).unwrap_or(1800);
            let json = p.next() == Some("1");
            let fresh = engine.write().map_err(poisoned).and_then(|mut e| e.refresh().map(|_| ()));
            let answer = fresh.and_then(|_| engine.read().map_err(poisoned).and_then(|e| e.answer(query, n, max_chars)));
            match answer {
                Ok(passages) => format!("ok\n{}", render_passages(query, &passages, json)),
                Err(e) => format!("error\n{e}\n"),
            }
        }
        ["search", ..] | ["answer", ..] => {
            // The client is another version: step aside, it will do the work itself
            // and start an up-to-date process.
            let _ = w.write_all(b"stale\n");
            stop.store(true, Ordering::Relaxed);
            return;
        }
        ["stop", ..] => {
            let _ = w.write_all(b"ok\n");
            stop.store(true, Ordering::Relaxed);
            return;
        }
        ["status", ..] => {
            let entries = engine.read().map(|e| e.index.len()).unwrap_or(0);
            format!(
                "ok\npid\t{}\nuptime_s\t{uptime}\nrequests\t{}\nrefreshes\t{}\nrss_kb\t{}\nvectors\t{entries}\nlast_refresh_s\t{since_refresh}\n",
                std::process::id(),
                served.load(Ordering::Relaxed),
                refreshed.load(Ordering::Relaxed),
                rss_kb()
            )
        }
        _ => "error\nunknown request\n".to_string(),
    };
    let _ = w.write_all(reply.as_bytes());
}

/// Resident memory of the current process in kB, via `ps`: portable between macOS
/// and Linux without a dependency.
fn rss_kb() -> u64 {
    std::process::Command::new("ps")
        .args(["-o", "rss=", "-p", &std::process::id().to_string()])
        .output()
        .ok()
        .and_then(|o| String::from_utf8_lossy(&o.stdout).trim().parse().ok())
        .unwrap_or(0)
}

fn run_status() -> Result<(), String> {
    use std::io::{Read, Write};
    match std::os::unix::net::UnixStream::connect(socket_path()) {
        Ok(mut s) => {
            let _ = s.set_read_timeout(Some(std::time::Duration::from_secs(5)));
            let _ = s.write_all(b"status\n");
            let _ = s.shutdown(std::net::Shutdown::Write);
            let mut reply = String::new();
            let _ = s.read_to_string(&mut reply);
            println!("warm process: running");
            for line in reply.lines().skip(1) {
                if let Some((k, v)) = line.split_once('\t') {
                    let v = if k == "rss_kb" { format!("{} MB", v.parse::<u64>().unwrap_or(0) / 1024) } else { v.to_string() };
                    println!("  {k:<16} {v}");
                }
            }
        }
        Err(_) => println!("warm process: none (the next search starts one)"),
    }
    println!("root: {}", root().display());
    println!("model: {}", model_dir().display());
    match std::fs::metadata(state_file("index.bin")) {
        Ok(m) => {
            let age = m.modified().ok().and_then(|t| t.elapsed().ok()).map_or(0, |d| d.as_secs());
            println!("index: {} MB, updated {} s ago", m.len() / 1_000_000, age);
        }
        Err(_) => println!("index: none"),
    }
    if let Ok(log) = std::fs::read_to_string(state_file("usage.log")) {
        let now = now_secs();
        let (mut total, mut day) = (0usize, 0usize);
        let mut durations: Vec<u64> = Vec::new();
        for line in log.lines() {
            let f: Vec<&str> = line.split('\t').collect();
            if f.len() < 3 || f[1] != "search" {
                continue;
            }
            total += 1;
            if f[0].parse::<u64>().is_ok_and(|t| now.saturating_sub(t) < 86_400) {
                day += 1;
            }
            if let Some(ms) = f[2].strip_suffix("ms").and_then(|v| v.parse().ok()) {
                durations.push(ms);
            }
        }
        durations.sort_unstable();
        let median = durations.get(durations.len() / 2).copied().unwrap_or(0);
        println!("searches: {total} total, {day} in 24 h, median {median} ms");
    }
    Ok(())
}

// ----------------------------------------------------------------------------- search

/// The search text, from the warm process when there is one, else computed here.
fn search_text(query: &str, with_archives: bool) -> Result<String, String> {
    if let Some(text) = ask_daemon_search(query, with_archives) {
        return Ok(text);
    }
    if !model_dir().join("model.safetensors").exists() {
        return lexical_search(query).map(|t| format!("(lexical search: no model in {})\n{t}", model_dir().display()));
    }
    spawn_daemon();
    let mut engine = Engine::open()?;
    engine.refresh()?;
    engine.search(query, with_archives)
}

fn run_search(args: &[String]) -> Result<(), String> {
    let with_archives = args.iter().any(|a| a == "--archives");
    let query = args[1..].iter().filter(|a| *a != "--archives").cloned().collect::<Vec<_>>().join(" ");
    print!("{}", search_text(&query, with_archives)?);
    Ok(())
}

/// Without a model: one line per note touched, and the mode announced.
fn lexical_search(query: &str) -> Result<String, String> {
    let base = root();
    let words: Vec<String> = query.split_whitespace().map(|w| w.to_lowercase()).filter(|w| w.len() >= 3).collect();
    let mut out = String::new();
    let mut hits = 0;
    for f in engrams::hot::notes_of(&base) {
        let Ok(content) = std::fs::read_to_string(&f) else { continue };
        let note = Note::parse(&content);
        if let Some((i, l)) = note.body().lines().enumerate().find(|(_, l)| {
            let low = l.to_lowercase();
            words.iter().any(|w| low.contains(w))
        }) {
            out.push_str(&format!(
                "{}:{}\t{}\t{}\n",
                relative(&f, &base),
                i + 1,
                engrams::hot::display_name(&f, &note),
                l.trim().chars().take(110).collect::<String>()
            ));
            hits += 1;
        }
    }
    if hits > 15 {
        out.push_str(&format!("# {hits} notes touched, the query is too broad to be useful\n"));
    }
    Ok(out)
}

fn answer_text(query: &str, n: usize, max_chars: usize, json: bool) -> Result<String, String> {
    if let Some(text) = ask_daemon_answer(query, n, max_chars, json) {
        return Ok(text);
    }
    spawn_daemon();
    let mut engine = Engine::open()?;
    engine.refresh()?;
    let passages = engine.answer(query, n, max_chars)?;
    Ok(render_passages(query, &passages, json))
}

fn run_answer(args: &[String]) -> Result<(), String> {
    let n: usize = flag_value(args, "-n").and_then(|v| v.parse().ok()).unwrap_or(3);
    let json = args.iter().any(|a| a == "--json");
    let max_chars: usize = std::env::var("ENGRAM_ANSWER_CHARS").ok().and_then(|v| v.parse().ok()).unwrap_or(1800);
    let query = positional_query(args, &["-n"]);
    if query.is_empty() {
        return Err("missing question".into());
    }
    print!("{}", answer_text(&query, n, max_chars, json)?);
    Ok(())
}

fn run_context(args: &[String]) -> Result<(), String> {
    let out_path = flag_value(args, "--out").map(PathBuf::from);
    let query = positional_query(args, &["--out"]);
    if query.is_empty() {
        return Err("missing topic".into());
    }
    let text = answer_text(&query, 5, 4000, false)?;
    match out_path {
        Some(p) => {
            std::fs::write(&p, &text).map_err(|e| format!("{}: {e}", p.display()))?;
            println!("written: {}", p.display());
        }
        None => print!("{text}"),
    }
    Ok(())
}

// ----------------------------------------------------------------------------- read and write

/// The single note whose `name` or file stem is `name`, under `project` if given.
/// Two homonyms are an error to raise, not one to guess.
fn find_note(name: &str, project: Option<&str>) -> Result<PathBuf, String> {
    let base = root();
    let dir = project.map_or_else(|| base.clone(), |p| base.join(p));
    let wanted = engrams::check::link_key(name);
    let mut found: Vec<PathBuf> = Vec::new();
    for f in engrams::hot::notes_of(&dir) {
        let stem = f.file_stem().unwrap_or_default().to_string_lossy().into_owned();
        let by_stem = engrams::check::link_key(&stem) == wanted;
        let by_name = std::fs::read_to_string(&f).ok().and_then(|c| Note::parse(&c).field("name").map(engrams::check::link_key)).is_some_and(|k| k == wanted);
        if by_stem || by_name {
            found.push(f);
        }
    }
    match found.len() {
        0 => Err(format!("not found: {name}")),
        1 => Ok(found.remove(0)),
        _ => Err(format!("several notes are named {name}: {}; add --project", found.iter().map(|p| relative(p, &base)).collect::<Vec<_>>().join(", "))),
    }
}

/// The note text. A read that follows a search is usage feedback (see `learn`).
fn read_note(name: &str, project: Option<&str>) -> Result<String, String> {
    use std::io::Write;
    let path = find_note(name, project)?;
    let rel = relative(&path, &root());
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(state_file("reads.log")) {
        let _ = writeln!(f, "{}\t{rel}", now_secs());
    }
    std::fs::read_to_string(&path).map_err(|e| e.to_string())
}

fn read_stdin_body() -> Result<String, String> {
    use std::io::Read;
    let mut body = String::new();
    std::io::stdin().read_to_string(&mut body).map_err(|e| e.to_string())?;
    let body = body.trim().to_string();
    if body.is_empty() {
        return Err("nothing on standard input, the body is required".into());
    }
    Ok(body)
}

/// Refuses a body that looks like a secret: better never written than caught by a
/// commit hook an hour later.
fn refuse_secrets(body: &str) -> Result<(), String> {
    let hits = engrams::secrets::scan_text("(input)", body);
    if hits.is_empty() {
        Ok(())
    } else {
        Err(format!("the body contains what looks like a secret ({}): remove it first", hits.iter().map(|h| h.kind).collect::<Vec<_>>().join(", ")))
    }
}

/// The closest active note to a text, when above `threshold`.
fn nearest_active_note(text: &str, threshold: f32) -> Result<Option<(String, f32)>, String> {
    let model = model_dir();
    if !model.join("model.safetensors").exists() {
        return Ok(None);
    }
    let Ok(index) = Index::load(&state_file("index.bin")) else { return Ok(None) };
    let embedder = Embedder::load(&model)?;
    let vector = embedder.encode(text)?;
    let base = root();
    let chunks: Chunks = index
        .iter()
        .filter(|(key, _)| !Index::is_question_key(key))
        .map(|(key, v)| {
            let (path, ordinal) = Index::split_key(key);
            (path.to_string(), ordinal, v.to_vec())
        })
        .collect();
    for hit in engrams::similarity::rank_notes(&vector, &chunks, 5) {
        let active = std::fs::read_to_string(base.join(&hit.path)).map(|c| Note::parse(&c).is_active()).unwrap_or(false);
        if active {
            return Ok((hit.score >= threshold).then_some((hit.path, hit.score)));
        }
    }
    Ok(None)
}

fn dup_threshold() -> f32 {
    std::env::var("ENGRAM_DUP").ok().and_then(|v| v.parse().ok()).unwrap_or(0.90)
}

fn quote_if_needed(v: &str) -> String {
    if v.contains(':') && !v.starts_with('"') {
        format!("\"{}\"", v.replace('"', "\\\""))
    } else {
        v.to_string()
    }
}

struct NewNote<'a> {
    project: &'a str,
    name: &'a str,
    kind: &'a str,
    description: &'a str,
    depends_on: Option<&'a str>,
    source: Option<&'a str>,
    body: &'a str,
    force: bool,
}

/// Writes a new note. `force` skips the duplicate check. Returns the relative path.
fn create_note(n: &NewNote) -> Result<String, String> {
    if !engrams::check::TYPES.contains(&n.kind) {
        return Err(format!("invalid type `{}`, expected one of {:?}", n.kind, engrams::check::TYPES));
    }
    let name = n.name;
    if !name.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-') || name.starts_with('-') || name.ends_with('-') || name.contains("--") {
        return Err("name outside the convention: lowercase ASCII, digits and dashes only".into());
    }
    if n.project.split('/').count() != 2 {
        return Err("project must be `family/project`".into());
    }
    refuse_secrets(n.body)?;
    let base = root();
    let dir = base.join(n.project);
    let path = dir.join(format!("{name}.md"));
    if path.exists() {
        return Err(format!("{} already exists: `engram append {name}` to complete it, `engram supersede {name} <new>` to replace it", relative(&path, &base)));
    }
    if !n.force {
        if let Some((near, score)) = nearest_active_note(&format!("{name}\n{}\n\n{}", n.description, n.body), dup_threshold())? {
            return Err(format!("an active note already says this at {score:.2} cosine: {near}. `engram append` to complete it, `engram supersede` to replace it, --force to override"));
        }
    }
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let mut head = format!("---\nname: {name}\ndescription: {}\ntype: {}\nstatus: active\nverified: {}\n", quote_if_needed(n.description), n.kind, today());
    if let Some(d) = n.depends_on {
        head.push_str(&format!("depends_on: {}\n", quote_if_needed(d)));
    }
    if let Some(src) = n.source {
        head.push_str(&format!("source: {src}\n"));
    }
    head.push_str("---\n\n");
    std::fs::write(&path, format!("{head}{}\n", n.body)).map_err(|e| e.to_string())?;
    let size = engrams::hot::write(&base, n.project)?;
    if size > engrams::hot::BOUND {
        eprintln!("warning: the hot index of {} exceeds the bound ({size} bytes)", n.project);
    }
    Ok(relative(&path, &base))
}

fn run_write(args: &[String]) -> Result<(), String> {
    let body = read_stdin_body()?;
    let rel = create_note(&NewNote {
        project: &args[1],
        name: &args[2],
        kind: flag_value(args, "--type").ok_or("--type is required")?,
        description: flag_value(args, "--description").ok_or("--description is required")?,
        depends_on: flag_value(args, "--depends-on"),
        source: flag_value(args, "--source"),
        body: &body,
        force: args.iter().any(|a| a == "--force"),
    })?;
    println!("written: {rel}");
    Ok(())
}

fn regen_project_of(path: &Path) -> Result<(), String> {
    let base = root();
    let rel = relative(path, &base);
    let mut parts = rel.split('/');
    if let (Some(fam), Some(proj)) = (parts.next(), parts.next()) {
        if parts.next().is_some() {
            engrams::hot::write(&base, &format!("{fam}/{proj}"))?;
        }
    }
    Ok(())
}

/// Appends a paragraph to a note and marks it verified today.
fn append_note(name: &str, project: Option<&str>, paragraph: &str) -> Result<String, String> {
    let path = find_note(name, project)?;
    refuse_secrets(paragraph)?;
    let content = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let updated = engrams::lifecycle::set_field(&engrams::lifecycle::append_paragraph(&content, paragraph), "verified", &today())?;
    std::fs::write(&path, updated).map_err(|e| e.to_string())?;
    regen_project_of(&path)?;
    Ok(relative(&path, &root()))
}

fn run_append(args: &[String]) -> Result<(), String> {
    let body = read_stdin_body()?;
    println!("completed: {}", append_note(&args[1], flag_value(args, "--project"), &body)?);
    Ok(())
}

fn run_verify(args: &[String]) -> Result<(), String> {
    let path = find_note(&args[1], flag_value(args, "--project"))?;
    let content = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    std::fs::write(&path, engrams::lifecycle::set_field(&content, "verified", &today())?).map_err(|e| e.to_string())?;
    println!("verified today: {}", relative(&path, &root()));
    regen_project_of(&path)
}

/// Replaces a note: the new one is written in the old one's project, the old one is
/// archived with `superseded_by`, and every link to it is rewritten.
fn run_supersede(args: &[String]) -> Result<(), String> {
    let old_path = find_note(&args[1], flag_value(args, "--project"))?;
    let base = root();
    let old_rel = relative(&old_path, &base);
    let project = old_rel.rsplit_once('/').map(|(p, _)| p.to_string()).ok_or("note outside a project")?;
    let new_name = &args[2];
    let body = read_stdin_body()?;
    let new_rel = create_note(&NewNote {
        project: &project,
        name: new_name,
        kind: flag_value(args, "--type").ok_or("--type is required")?,
        description: flag_value(args, "--description").ok_or("--description is required")?,
        depends_on: flag_value(args, "--depends-on"),
        source: flag_value(args, "--source"),
        body: &body,
        force: true,
    })?;
    let new_path = base.join(&new_rel);
    let old_content = std::fs::read_to_string(&old_path).map_err(|e| e.to_string())?;
    let old_note = Note::parse(&old_content);
    let old_stem = old_path.file_stem().unwrap_or_default().to_string_lossy().into_owned();
    let old_keys: Vec<String> = [Some(old_stem.as_str()), old_note.field("name")].into_iter().flatten().map(engrams::check::link_key).collect();
    let archived =
        engrams::lifecycle::set_field(&engrams::lifecycle::set_field(&old_content, "status", "archived")?, "superseded_by", &format!("[[{new_name}]]"))?;
    std::fs::write(&old_path, archived).map_err(|e| e.to_string())?;
    let mut relinked = 0;
    for f in engrams::hot::notes_of(&base) {
        if f == old_path || f == new_path {
            continue;
        }
        let Ok(c) = std::fs::read_to_string(&f) else { continue };
        let mut text = c.clone();
        let mut n = 0;
        for k in &old_keys {
            let (t, m) = engrams::lifecycle::relink(&text, k, new_name);
            text = t;
            n += m;
        }
        if n > 0 {
            std::fs::write(&f, text).map_err(|e| e.to_string())?;
            relinked += n;
        }
    }
    println!("replaced: {old_rel} -> {new_rel} ({relinked} link(s) rewritten)");
    run_regen(None)
}

/// A cross-reference in both directions.
fn link_notes(a: &str, b: &str) -> Result<String, String> {
    let a = find_note(a, None)?;
    let b = find_note(b, None)?;
    let base = root();
    for (from, to) in [(&a, &b), (&b, &a)] {
        let content = std::fs::read_to_string(from).map_err(|e| e.to_string())?;
        let to_content = std::fs::read_to_string(to).map_err(|e| e.to_string())?;
        let to_note = Note::parse(&to_content);
        let to_name = engrams::hot::display_name(to, &to_note);
        if engrams::lifecycle::relink(&content, &engrams::check::link_key(&to_name), &to_name).1 > 0 {
            continue;
        }
        let project = relative(to, &base).split('/').nth(1).unwrap_or("").to_string();
        let updated = engrams::lifecycle::append_paragraph(&content, &format!("See also [[{to_name}]], the same topic seen from {project}."));
        std::fs::write(from, updated).map_err(|e| e.to_string())?;
    }
    Ok(format!("linked: {} and {}", relative(&a, &base), relative(&b, &base)))
}

// ----------------------------------------------------------------------------- learning

fn read_search_log() -> Vec<engrams::feedback::Searched> {
    std::fs::read_to_string(state_file("searches.log"))
        .unwrap_or_default()
        .lines()
        .filter_map(|l| {
            let mut f = l.split('\t');
            let secs = f.next()?.parse().ok()?;
            let query = f.next()?.to_string();
            let shown = f.next().unwrap_or("").split(',').filter(|s| !s.is_empty()).map(str::to_string).collect();
            Some(engrams::feedback::Searched { secs, query, shown })
        })
        .collect()
}

fn read_read_log() -> Vec<engrams::feedback::Read> {
    std::fs::read_to_string(state_file("reads.log"))
        .unwrap_or_default()
        .lines()
        .filter_map(|l| {
            let (secs, path) = l.split_once('\t')?;
            Some(engrams::feedback::Read { secs: secs.parse().ok()?, path: path.to_string() })
        })
        .collect()
}

/// Derives the "missed" pairs from the logs and adds them to the table, saved when
/// it changed. Returns the number of new or reinforced pairs.
fn learn_from_logs(table: &mut engrams::feedback::Table) -> Result<usize, String> {
    let searches = read_search_log();
    let reads = read_read_log();
    let mut n = 0;
    for (query, path, day) in engrams::feedback::missed_pairs(&searches, &reads, 600) {
        if table.record(&query, &path, "missed", day) {
            n += 1;
        }
    }
    if n > 0 {
        table.save(&state_file("feedback.json"))?;
    }
    Ok(n)
}

/// Records an explicit pair. Only the table is written: query vectors are completed
/// at the next engine start, so this needs no model.
fn learn_pair(query: &str, name: &str, project: Option<&str>) -> Result<String, String> {
    let table_path = state_file("feedback.json");
    let mut table = engrams::feedback::Table::load(&table_path);
    let note = find_note(name, project)?;
    let rel = relative(&note, &root());
    if table.record(query, &rel, "explicit", now_secs() / 86_400) {
        table.save(&table_path)?;
        Ok(format!("learned: \"{query}\" -> {rel}"))
    } else {
        Ok("already learned today".into())
    }
}

fn run_learn(args: &[String]) -> Result<(), String> {
    let table_path = state_file("feedback.json");
    let mut table = engrams::feedback::Table::load(&table_path);
    let today = now_secs() / 86_400;
    if args.iter().any(|a| a == "--show") {
        let mut pairs = table.pairs.clone();
        pairs.sort_by_key(|p| std::cmp::Reverse(p.last_day));
        println!("{} learned pair(s)", pairs.len());
        for p in pairs {
            println!("  {:<8} x{:<2} weight {:.2}  \"{}\" -> {}", p.source, p.count, engrams::feedback::decay(p.last_day, today), p.query, p.path);
        }
        return Ok(());
    }
    if let Some(q) = flag_value(args, "--forget") {
        let n = table.forget(q);
        table.save(&table_path)?;
        println!("{n} pair(s) forgotten");
        return Ok(());
    }
    if args.len() >= 3 {
        println!("{}", learn_pair(&args[1], &args[2], flag_value(args, "--project"))?);
        return Ok(());
    }
    let n = learn_from_logs(&mut table)?;
    println!("{n} pair(s) learned from the logs, {} in total", table.pairs.len());
    Ok(())
}

// ----------------------------------------------------------------------------- maintenance

fn run_regen(project: Option<&str>) -> Result<(), String> {
    let base = root();
    let projects = match project {
        Some(p) => vec![p.to_string()],
        None => engrams::hot::projects(&base),
    };
    let mut over = 0;
    for p in &projects {
        let size = engrams::hot::write(&base, p)?;
        let state = if size <= engrams::hot::BOUND { "OK  " } else { "OVER" };
        println!("{state}\t{p}\t{size} bytes");
        over += usize::from(size > engrams::hot::BOUND);
    }
    if over > 0 {
        return Err(format!("{over} index(es) above the bound despite compaction"));
    }
    Ok(())
}

fn run_list() -> Result<(), String> {
    let base = root();
    let mut current_family = String::new();
    for p in engrams::hot::projects(&base) {
        let family = p.split('/').next().unwrap_or("").to_string();
        if family != current_family {
            println!("{family}");
            current_family = family;
        }
        println!("  {p}\t{} note(s)", engrams::hot::notes_of(&base.join(&p)).len());
    }
    Ok(())
}

/// Installed version of a tool, to compare with pinned `depends_on` fields.
fn installed_version(tool: &str) -> Option<String> {
    let out = std::process::Command::new(tool).arg("--version").output().ok()?;
    let text = String::from_utf8_lossy(&out.stdout);
    let mut it = text.split(|c: char| !(c.is_ascii_digit() || c == '.')).filter(|s| s.contains('.') && s.chars().next().is_some_and(|c| c.is_ascii_digit()));
    it.next().map(|s| s.trim_matches('.').to_string())
}

fn run_secrets(dir: Option<PathBuf>) -> Result<(), String> {
    let dir = dir.unwrap_or_else(root);
    let hits = engrams::secrets::scan_dir(&dir);
    for h in &hits {
        println!("  {}:{}  {}", h.path, h.line, h.kind);
    }
    if hits.is_empty() {
        Ok(())
    } else {
        Err(format!("{} probable secret(s), see above", hits.len()))
    }
}

fn run_check() -> Result<(), String> {
    let base = root();
    let files = collect_notes(&base);
    let mut corpus: Vec<(String, Note)> = Vec::with_capacity(files.len());
    for file in &files {
        let Ok(content) = std::fs::read_to_string(file) else { continue };
        corpus.push((relative(file, &base), Note::parse(&content)));
    }

    let mut findings: Vec<engrams::check::Finding> = corpus.iter().flat_map(|(path, note)| engrams::check::check_note(path, note)).collect();
    findings.extend(engrams::check::check_corpus(&corpus));

    for file in &files {
        let Ok(content) = std::fs::read_to_string(file) else { continue };
        let path = relative(file, &base);
        let head = content.split("\n---").next().unwrap_or("");
        if head.lines().any(|l| l.starts_with("metadata:")) {
            findings.push(engrams::check::Finding::new(&path, "nested `metadata:` block, flatten it to the frontmatter root"));
        }
        let note = Note::parse(&content);
        if let Some(status) = note.field("status") {
            if !["active", "archived"].contains(&status) {
                findings.push(engrams::check::Finding::new(&path, format!("unknown status `{status}`, expected active or archived")));
            }
        }
        if let Some(dep) = note.field("depends_on") {
            for pin in dep.split(',') {
                let mut parts = pin.trim().splitn(2, ' ');
                if let (Some(tool), Some(pinned)) = (parts.next(), parts.next()) {
                    let pinned = pinned.trim().trim_end_matches(".x");
                    if let Some(current) = installed_version(tool) {
                        if !current.starts_with(pinned) {
                            findings.push(engrams::check::Finding::new(&path, format!("pinned on {tool} {pinned}, installed {current}, re-verify")));
                        }
                    }
                }
            }
        }
    }
    for p in engrams::hot::projects(&base) {
        let size = engrams::hot::render(&base, &p).len();
        if size > engrams::hot::BOUND {
            findings.push(engrams::check::Finding::new(
                &format!("{p}/MEMORY.md"),
                format!("index at {size} bytes, above the {} byte bound despite compaction", engrams::hot::BOUND),
            ));
        }
    }

    // Truncation needs the model, so it is optional: `check` stays useful on a
    // machine without one.
    let mut truncated = Vec::new();
    if let Ok(embedder) = Embedder::load(&model_dir()) {
        let budget = budget_for(embedder.window());
        for (path, note) in &corpus {
            let name = note.field("name").unwrap_or_default();
            let description = note.field("description").unwrap_or_default();
            let cut = split(note.body(), budget).iter().filter(|c| embedder.would_truncate(&c.with_context(name, description))).count();
            if cut > 0 {
                truncated.push(format!("{path} ({cut} paragraph(s))"));
            }
        }
    }

    for f in &findings {
        println!("{}\t{}", f.path, f.message);
    }
    if !truncated.is_empty() {
        println!("\n{} of {} notes have a paragraph longer than the model window, cut at encoding:", truncated.len(), corpus.len());
        for p in truncated.iter().take(10) {
            println!("  {p}");
        }
        if truncated.len() > 10 {
            println!("  ... and {} more", truncated.len() - 10);
        }
    }
    // Near-duplicates from the index alone. ENGRAM_DUP sets the threshold (0.90).
    let loaded = Index::load(&state_file("index.bin")).map_err(|e| format!("index missing or unreadable ({e}), run `engram index`"));
    if let Err(e) = &loaded {
        println!("\nDuplicates: not checked, {e}");
    }
    if let Ok(index) = loaded {
        let threshold = dup_threshold();
        let active: std::collections::HashSet<String> = corpus.iter().filter(|(_, n)| n.is_active()).map(|(p, _)| p.clone()).collect();
        let (compared, pairs) = engrams::duplicates::near_duplicates(&index, &active, threshold);
        println!("\nDuplicates: {} chunks of active notes compared, {} pair(s) above {threshold:.2}.", compared, pairs.len());
        for (score, a, b) in pairs.iter().take(30) {
            println!("  {score:.3}  {a}  <->  {b}");
        }
        if pairs.len() > 30 {
            println!("  ... and {} more", pairs.len() - 30);
        }
    }
    println!("\n{} note(s) checked, {} problem(s).", corpus.len(), findings.len());
    if findings.is_empty() {
        Ok(())
    } else {
        Err(format!("{} problem(s), see above", findings.len()))
    }
}

/// Civil days since the epoch for a `YYYY-MM-DD` date, without a dependency.
fn days_from_ymd(date: &str) -> Option<i64> {
    let mut it = date.split('-');
    let (y, m, d): (i64, i64, i64) = (it.next()?.parse().ok()?, it.next()?.parse().ok()?, it.next()?.parse().ok()?);
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let mp = (m + 9) % 12;
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    Some(era * 146_097 + doe - 719_468)
}

/// A markdown list of what deserves a review, without the model and without a
/// decision: stale project notes, long notes, near-duplicates, undated notes,
/// hot indexes that mask notes.
fn run_curation() -> Result<(), String> {
    let base = root();
    let today = today();
    let today_days = days_from_ymd(&today).unwrap_or(0);
    let mut stale: Vec<(i64, String)> = Vec::new();
    let mut long: Vec<(usize, String)> = Vec::new();
    let mut undated: Vec<String> = Vec::new();
    let mut corpus: Vec<(String, Note)> = Vec::new();
    for f in collect_notes(&base) {
        let Ok(content) = std::fs::read_to_string(&f) else { continue };
        let rel = relative(&f, &base);
        let note = Note::parse(&content);
        if !note.is_active() {
            corpus.push((rel, note));
            continue;
        }
        match note.field("verified") {
            None => undated.push(rel.clone()),
            Some(v) => {
                if note.field("type").unwrap_or("project") == "project" {
                    if let Some(days) = days_from_ymd(v) {
                        let age = today_days - days;
                        if age > 60 {
                            stale.push((age, rel.clone()));
                        }
                    }
                }
            }
        }
        // Past two thousand tokens, a note is a journal that took the place of a note.
        let tokens = note.body().chars().count() / 4;
        if tokens > 2000 {
            long.push((tokens, rel.clone()));
        }
        corpus.push((rel, note));
    }
    stale.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.cmp(&b.1)));
    long.sort_by_key(|(tokens, _)| std::cmp::Reverse(*tokens));

    println!("# Curation, {today}\n");
    println!("This report lists candidates, it decides nothing. Tick what was handled.\n");
    println!("## Active project notes not verified for more than sixty days ({})\n", stale.len());
    println!("Re-verify (`engram verify`) or replace (`engram supersede`).\n");
    for (age, rel) in &stale {
        println!("- [ ] `{rel}` ({age} days)");
    }
    println!("\n## Notes longer than two thousand tokens ({})\n", long.len());
    println!("A journal that took the place of a note: split into facts, or archive.\n");
    for (tokens, rel) in &long {
        println!("- [ ] `{rel}` (~{tokens} tokens)");
    }
    println!("\n## Active notes without a verification date ({})\n", undated.len());
    for rel in &undated {
        println!("- [ ] `{rel}`");
    }
    if let Ok(index) = Index::load(&state_file("index.bin")) {
        let active: std::collections::HashSet<String> = corpus.iter().filter(|(_, n)| n.is_active()).map(|(p, _)| p.clone()).collect();
        let (_, pairs) = engrams::duplicates::near_duplicates(&index, &active, dup_threshold());
        println!("\n## Pairs of active notes above {:.2} cosine ({})\n", dup_threshold(), pairs.len());
        println!("Merge, or cross-reference if they are two angles of one topic.\n");
        for (s, a, b) in pairs.iter().take(30) {
            println!("- [ ] {s:.3} `{a}` and `{b}`");
        }
    }
    let mut masked_total = 0usize;
    let mut masked_lines = Vec::new();
    for p in engrams::hot::projects(&base) {
        let text = engrams::hot::render(&base, &p);
        if let Some(line) = text.lines().find(|l| l.contains("left out of the hot index")) {
            let n: usize = line.trim_start_matches("- ").split(' ').next().and_then(|v| v.parse().ok()).unwrap_or(0);
            masked_total += n;
            masked_lines.push(format!("- `{p}`: {n} notes left out of the hot index"));
        }
    }
    println!("\n## Hot indexes that leave notes out ({masked_total} in total)\n");
    for l in &masked_lines {
        println!("{l}");
    }
    Ok(())
}

/// What changed in the memory, per project, read from git.
fn run_since(days: &str) -> Result<(), String> {
    let root = root();
    let days: u32 = days.parse().map_err(|_| "expected a number of days".to_string())?;
    let out = std::process::Command::new("git")
        .args([
            "-C",
            &root.to_string_lossy(),
            "log",
            &format!("--since={days} days ago"),
            "--name-status",
            "--relative",
            "--date=short",
            "--format=%x01%ad%x09%s",
            "--",
            ".",
        ])
        .output()
        .map_err(|e| format!("git: {e}"))?;
    let text = String::from_utf8_lossy(&out.stdout);
    let mut by_project: std::collections::BTreeMap<String, Vec<String>> = std::collections::BTreeMap::new();
    let mut date = String::new();
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix('\u{1}') {
            date = rest.split('\t').next().unwrap_or("").to_string();
            continue;
        }
        let mut f = line.split('\t');
        let (Some(status), Some(path)) = (f.next(), f.next()) else { continue };
        let rel = f.next().unwrap_or(path); // rename: R100\told\tnew
        if rel.ends_with("MEMORY.md") || rel.starts_with(".engram/") || !rel.ends_with(".md") {
            continue;
        }
        let project = rel.rsplit_once('/').map_or("", |(p, _)| p).to_string();
        let verb = match status.chars().next() {
            Some('A') => "added",
            Some('D') => "deleted",
            Some('R') => "renamed",
            _ => "modified",
        };
        by_project.entry(project).or_default().push(format!("{date}  {verb:<9} {}", rel.rsplit('/').next().unwrap_or(rel)));
    }
    if by_project.is_empty() {
        println!("nothing changed in the last {days} day(s)");
    }
    for (project, lines) in &by_project {
        println!("{project}");
        let mut seen = std::collections::HashSet::new();
        for l in lines.iter().rev() {
            if seen.insert(l.clone()) {
                println!("  {l}");
            }
        }
    }
    Ok(())
}

/// Provenance of a note: frontmatter, git history, citing notes.
fn run_why(name: &str) -> Result<(), String> {
    let base = root();
    let path = find_note(name, None)?;
    let rel = relative(&path, &base);
    let content = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let note = Note::parse(&content);
    println!("{rel}");
    for k in ["name", "type", "status", "verified", "depends_on", "superseded_by", "source"] {
        if let Some(v) = note.field(k) {
            println!("  {k:<14} {v}");
        }
    }
    let out = std::process::Command::new("git")
        .args(["-C", &base.to_string_lossy(), "log", "--follow", "--date=short", "--format=%ad  %s", "--", &rel])
        .output()
        .map_err(|e| format!("git: {e}"))?;
    let history = String::from_utf8_lossy(&out.stdout);
    println!("\ngit history ({} commit(s))", history.lines().count());
    for l in history.lines().take(12) {
        println!("  {l}");
    }
    let key = engrams::check::link_key(&engrams::hot::display_name(&path, &note));
    let stem_key = engrams::check::link_key(&path.file_stem().unwrap_or_default().to_string_lossy());
    let mut citing = Vec::new();
    for f in engrams::hot::notes_of(&base) {
        if f == path {
            continue;
        }
        let Ok(c) = std::fs::read_to_string(&f) else { continue };
        if engrams::lifecycle::relink(&c, &key, "x").1 > 0 || engrams::lifecycle::relink(&c, &stem_key, "x").1 > 0 {
            citing.push(relative(&f, &base));
        }
    }
    println!("\ncited by {} note(s)", citing.len());
    for c in citing {
        println!("  {c}");
    }
    Ok(())
}

// ----------------------------------------------------------------------------- integrations

/// Claude Code `UserPromptSubmit` hook: reads the hook JSON on stdin, prints the
/// passages close to the prompt as context. Silent when there is nothing to say.
/// `ENGRAM_HOOK_MIN` (0.60) drops distant passages, `ENGRAM_HOOK_CHARS` (700)
/// bounds each passage, `ENGRAM_HOOK_LEN` (30) ignores shorter prompts.
fn run_hook() -> Result<(), String> {
    use std::io::Read;
    let mut input = String::new();
    let _ = std::io::stdin().read_to_string(&mut input);
    let value: serde_json::Value = serde_json::from_str(&input).unwrap_or(serde_json::Value::Null);
    let prompt = value["prompt"].as_str().unwrap_or("").trim();
    let min_len: usize = std::env::var("ENGRAM_HOOK_LEN").ok().and_then(|v| v.parse().ok()).unwrap_or(30);
    if prompt.chars().count() < min_len || prompt.starts_with('/') || prompt.starts_with('!') {
        return Ok(());
    }
    let min_score: f32 = std::env::var("ENGRAM_HOOK_MIN").ok().and_then(|v| v.parse().ok()).unwrap_or(0.60);
    let max_chars: usize = std::env::var("ENGRAM_HOOK_CHARS").ok().and_then(|v| v.parse().ok()).unwrap_or(700);
    // A long prompt is summarised by its first lines: the model reads 512 tokens.
    let query: String = prompt.lines().take(12).collect::<Vec<_>>().join(" ").chars().take(1200).collect();
    let Ok(json) = answer_text(&query, 2, max_chars, true) else { return Ok(()) };
    let passages: Vec<Passage> = serde_json::from_str(&json).unwrap_or_default();
    let kept: Vec<&Passage> = passages.iter().filter(|p| p.score >= min_score).collect();
    if kept.is_empty() {
        return Ok(());
    }
    println!("<working-memory>");
    println!("Passages from the user's working memory close to this request (from `engram answer`). They may be off topic: use them only if they answer, and read the whole note with `engram read <name>` before relying on it.\n");
    for p in kept {
        let v = p.verified.as_deref().map(|d| format!(", verified {d}")).unwrap_or_default();
        println!("## {} ({}{v}, {:.2})\n{}\n", p.name, p.path, p.score, p.text.trim());
    }
    println!("</working-memory>");
    Ok(())
}

/// MCP server over stdio: one JSON-RPC message per line. Tools reuse the CLI code.
fn run_mcp() -> Result<(), String> {
    use std::io::{BufRead, Write};
    let stdin = std::io::stdin();
    let mut out = std::io::stdout().lock();
    let mut reply = |v: serde_json::Value| {
        let _ = writeln!(out, "{v}");
        let _ = out.flush();
    };
    for line in stdin.lock().lines() {
        let Ok(line) = line else { break };
        if line.trim().is_empty() {
            continue;
        }
        let Ok(msg) = serde_json::from_str::<serde_json::Value>(&line) else { continue };
        let id = msg.get("id").cloned();
        let method = msg["method"].as_str().unwrap_or("");
        let Some(id) = id else { continue }; // a notification needs no reply
        let result = match method {
            "initialize" => Ok(serde_json::json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {"tools": {}},
                "serverInfo": {"name": "engram", "version": env!("CARGO_PKG_VERSION")}
            })),
            "ping" => Ok(serde_json::json!({})),
            "tools/list" => Ok(serde_json::json!({"tools": mcp_tools()})),
            "tools/call" => {
                let name = msg["params"]["name"].as_str().unwrap_or("");
                let args = msg["params"]["arguments"].clone();
                Ok(match mcp_call(name, &args) {
                    Ok(text) => serde_json::json!({"content": [{"type": "text", "text": text}]}),
                    Err(e) => serde_json::json!({"content": [{"type": "text", "text": e}], "isError": true}),
                })
            }
            other => Err(format!("method not found: {other}")),
        };
        match result {
            Ok(r) => reply(serde_json::json!({"jsonrpc": "2.0", "id": id, "result": r})),
            Err(e) => reply(serde_json::json!({"jsonrpc": "2.0", "id": id, "error": {"code": -32601, "message": e}})),
        }
    }
    Ok(())
}

fn mcp_tools() -> serde_json::Value {
    let s = |d: &str| serde_json::json!({"type": "string", "description": d});
    serde_json::json!([
        {"name": "search", "description": "Semantic search over the user's working memory. Returns up to five notes with score, path, description and the passage that matched.",
         "inputSchema": {"type": "object", "properties": {"query": s("Topic or question, in any language"), "archives": {"type": "boolean", "description": "Include archived notes"}}, "required": ["query"]}},
        {"name": "answer", "description": "The passages of the memory that best answer a question, bounded in size. Prefer this over search when you need the fact itself.",
         "inputSchema": {"type": "object", "properties": {"question": s("The question"), "n": {"type": "integer", "description": "Number of passages, default 3"}}, "required": ["question"]}},
        {"name": "read", "description": "Read a whole note by name. Use it before relying on a passage.",
         "inputSchema": {"type": "object", "properties": {"name": s("Note name or file stem"), "project": s("family/project, to disambiguate")}, "required": ["name"]}},
        {"name": "write", "description": "Write a new durable note. Refuses secrets and near-duplicates of an active note.",
         "inputSchema": {"type": "object", "properties": {"project": s("family/project"), "name": s("kebab-case name"), "type": {"type": "string", "enum": ["user", "feedback", "project", "reference"]}, "description": s("One line"), "body": s("Markdown body"), "depends_on": s("Pinned tool version, e.g. `tool 1.2`"), "source": s("Ticket or merge request"), "force": {"type": "boolean", "description": "Skip the duplicate check"}}, "required": ["project", "name", "type", "description", "body"]}},
        {"name": "append", "description": "Add a paragraph to an existing note and mark it verified today.",
         "inputSchema": {"type": "object", "properties": {"name": s("Note name"), "text": s("Paragraph to append"), "project": s("family/project, to disambiguate")}, "required": ["name", "text"]}},
        {"name": "link", "description": "Cross-reference two notes in both directions.",
         "inputSchema": {"type": "object", "properties": {"a": s("First note name"), "b": s("Second note name")}, "required": ["a", "b"]}},
        {"name": "learn", "description": "Confirm that a note is the right answer to a query, so future similar queries rank it higher.",
         "inputSchema": {"type": "object", "properties": {"query": s("The query"), "name": s("Note name")}, "required": ["query", "name"]}}
    ])
}

fn mcp_call(name: &str, a: &serde_json::Value) -> Result<String, String> {
    let str_arg = |k: &str| a[k].as_str().map(str::trim).filter(|s| !s.is_empty());
    let need = |k: &str| str_arg(k).ok_or_else(|| format!("missing argument `{k}`"));
    match name {
        "search" => search_text(need("query")?, a["archives"].as_bool().unwrap_or(false)),
        "answer" => answer_text(need("question")?, a["n"].as_u64().unwrap_or(3) as usize, 1800, false),
        "read" => read_note(need("name")?, str_arg("project")),
        "write" => create_note(&NewNote {
            project: need("project")?,
            name: need("name")?,
            kind: need("type")?,
            description: need("description")?,
            depends_on: str_arg("depends_on"),
            source: str_arg("source"),
            body: need("body")?,
            force: a["force"].as_bool().unwrap_or(false),
        })
        .map(|rel| format!("written: {rel}")),
        "append" => append_note(need("name")?, str_arg("project"), need("text")?).map(|rel| format!("completed: {rel}")),
        "link" => link_notes(need("a")?, need("b")?),
        "learn" => learn_pair(need("query")?, need("name")?, None),
        other => Err(format!("unknown tool: {other}")),
    }
}

/// Reads a JSON file as an object, or an empty object when absent.
fn read_json_object(path: &Path) -> Result<serde_json::Map<String, serde_json::Value>, String> {
    match std::fs::read_to_string(path) {
        Ok(text) => serde_json::from_str::<serde_json::Value>(&text)
            .map_err(|e| format!("{}: {e}", path.display()))?
            .as_object()
            .cloned()
            .ok_or_else(|| format!("{} is not a JSON object", path.display())),
        Err(_) => Ok(serde_json::Map::new()),
    }
}

fn write_json_object(path: &Path, obj: &serde_json::Map<String, serde_json::Value>) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("{}: {e}", parent.display()))?;
    }
    if path.exists() {
        let backup = path.with_extension(format!("json.bak.{}", now_secs()));
        std::fs::copy(path, &backup).map_err(|e| format!("backup: {e}"))?;
    }
    let text = serde_json::to_string_pretty(obj).map_err(|e| e.to_string())?;
    std::fs::write(path, text + "\n").map_err(|e| format!("{}: {e}", path.display()))
}

/// Wires the hook and the MCP server into a tool's configuration.
fn run_setup(tool: &str) -> Result<(), String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?.to_string_lossy().into_owned();
    match tool {
        "claude-code" => {
            let path = paths::home().join(".claude/settings.json");
            let mut settings = read_json_object(&path)?;
            let hook_cmd = format!("{exe} hook");
            let hooks = settings.entry("hooks").or_insert_with(|| serde_json::json!({}));
            let list = hooks.as_object_mut().ok_or("`hooks` is not an object")?.entry("UserPromptSubmit").or_insert_with(|| serde_json::json!([]));
            let already = list.to_string().contains("engram") && list.to_string().contains(" hook");
            if !already {
                list.as_array_mut()
                    .ok_or("`UserPromptSubmit` is not an array")?
                    .push(serde_json::json!({"hooks": [{"type": "command", "command": hook_cmd, "timeout": 15}]}));
                write_json_object(&path, &settings)?;
                println!("hook added to {}", path.display());
            } else {
                println!("hook already present in {}", path.display());
            }
            match std::process::Command::new("claude").args(["mcp", "add", "--scope", "user", "engram", "--", &exe, "mcp"]).output() {
                Ok(o) if o.status.success() => println!("MCP server registered with `claude mcp add`"),
                _ => println!("register the MCP server yourself:\n  claude mcp add --scope user engram -- {exe} mcp"),
            }
            Ok(())
        }
        "opencode" => {
            let path = paths::home().join(".config/opencode/opencode.json");
            let mut config = read_json_object(&path)?;
            let mcp = config.entry("mcp").or_insert_with(|| serde_json::json!({}));
            mcp.as_object_mut()
                .ok_or("`mcp` is not an object")?
                .insert("engram".into(), serde_json::json!({"type": "local", "command": [exe, "mcp"], "enabled": true}));
            write_json_object(&path, &config)?;
            println!("MCP server added to {}", path.display());
            Ok(())
        }
        "kandev" => {
            println!("Kandev reads MCP servers from its settings (Settings > MCP) or the `update_mcp_config` tool. Add:");
            println!("{}", serde_json::to_string_pretty(&serde_json::json!({"mcpServers": {"engram": {"command": exe, "args": ["mcp"]}}})).unwrap_or_default());
            println!("Cards run Claude Code and opencode with the user's configuration, so `engram setup claude-code` and `engram setup opencode` also apply to them.");
            Ok(())
        }
        other => Err(format!("unknown tool `{other}`, expected claude-code, opencode or kandev")),
    }
}

/// Creates the notes directory, remembers it, and downloads the model.
fn run_init(args: &[String]) -> Result<(), String> {
    let dir = args.get(1).filter(|a| !a.starts_with("--")).map(PathBuf::from).unwrap_or_else(root);
    let dir = if dir.is_absolute() { dir } else { std::env::current_dir().map_err(|e| e.to_string())?.join(dir) };
    std::fs::create_dir_all(paths::state_dir(&dir)).map_err(|e| format!("{}: {e}", dir.display()))?;
    let pointer = paths::config_dir().join("root");
    std::fs::create_dir_all(paths::config_dir()).map_err(|e| e.to_string())?;
    std::fs::write(&pointer, format!("{}\n", dir.display())).map_err(|e| format!("{}: {e}", pointer.display()))?;
    let ignore = dir.join(".gitignore");
    if !ignore.exists() {
        std::fs::write(&ignore, ".engram/*\n!.engram/questions.json\n!.engram/feedback.json\n").map_err(|e| e.to_string())?;
    }
    println!("root: {} (remembered in {})", dir.display(), pointer.display());

    if args.iter().any(|a| a == "--no-download") {
        return Ok(());
    }
    let repo = flag_value(args, "--model").unwrap_or(paths::DEFAULT_MODEL_REPO);
    let model = match std::env::var("ENGRAM_MODEL") {
        Ok(m) => PathBuf::from(m),
        Err(_) => paths::config_dir().join("models").join(repo.rsplit('/').next().unwrap_or(repo)),
    };
    std::fs::create_dir_all(model.join("1_Pooling")).map_err(|e| e.to_string())?;
    for file in ["config.json", "1_Pooling/config.json", "sentencepiece.bpe.model", "model.safetensors"] {
        let target = model.join(file);
        if std::fs::metadata(&target).is_ok_and(|m| m.len() > 0) {
            println!("present: {file}");
            continue;
        }
        let url = format!("https://huggingface.co/{repo}/resolve/main/{file}");
        println!("downloading {file}");
        let status = std::process::Command::new("curl")
            .args(["-L", "--fail", "--progress-bar", "-o"])
            .arg(&target)
            .arg(&url)
            .status()
            .map_err(|e| format!("curl: {e}"))?;
        if !status.success() {
            let _ = std::fs::remove_file(&target);
            return Err(format!("download failed: {url}"));
        }
    }
    println!("model: {}", model.display());
    println!("next: write notes under {}/<family>/<project>/, then `engram index`", dir.display());
    Ok(())
}
