//! The compact tokenizer must return exactly the ids of the reference, on a whole
//! corpus, every benchmark query and edge cases. A different id makes a different
//! vector without any signal.
use engrams::chunking::{budget_for, split};
use engrams::note::Note;
use engrams::tokenizer::Unigram;
use std::path::{Path, PathBuf};

fn model_dir() -> Option<PathBuf> {
    let d = engrams::paths::model_dir();
    d.join("tokenizer.json").exists().then_some(d)
}

fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
    for e in std::fs::read_dir(dir).into_iter().flatten().flatten() {
        let p = e.path();
        if p.is_dir() {
            if !p.file_name().is_some_and(|n| n.to_string_lossy().starts_with('.')) {
                walk(&p, out);
            }
        } else if p.extension().is_some_and(|x| x == "md") && p.file_name().is_some_and(|n| !n.to_string_lossy().starts_with("MEMORY")) {
            out.push(p);
        }
    }
}

fn edge_cases() -> Vec<String> {
    vec![
        "".into(),
        " ".into(),
        "a".into(),
        "Hello world".into(),
        "règle sur les signatures dans les commits".into(),
        "DB_HOST=db_agents port 3334, database unit_tests; see release-v2".into(),
        "Ünïcödé — « quotes » … and emojis 🚀🧪, 中文, русский, العربية".into(),
        "https://example.test/api/issues?fields=idReadable&x=1".into(),
        "fn main() { let x: Vec<u32> = vec![1, 2]; }".into(),
        "  multiple   spaces \t tabs \n returns\n\nlines".into(),
        "a literal <unk> and <s> too </s> then <mask> and <pad>".into(),
        "▁ a real low line character in the text ▁word".into(),
        "ﬁ ligature, ½, ①, ｆｕｌｌｗｉｄｔｈ, ẛ, K (kelvin), µ".into(),
        "x".repeat(3000),
    ]
}

/// The whole corpus under the root, chunk by chunk and whole note, plus the
/// benchmark queries when present.
fn corpus_texts() -> Vec<String> {
    let mut texts = Vec::new();
    let root = engrams::paths::root();
    let mut files = Vec::new();
    walk(&root, &mut files);
    for f in &files {
        let Ok(content) = std::fs::read_to_string(f) else { continue };
        let note = Note::parse(&content);
        let (name, desc) = (note.field("name").unwrap_or_default(), note.field("description").unwrap_or_default());
        for c in split(note.body(), budget_for(512)) {
            texts.push(c.with_context(name, desc));
        }
        texts.push(content);
    }
    let bench = std::env::var("ENGRAM_BENCH").map(PathBuf::from).unwrap_or_else(|_| engrams::paths::state_dir(&root).join("bench-queries.json"));
    if let Ok(raw) = std::fs::read_to_string(bench) {
        let cases: std::collections::BTreeMap<String, Vec<serde_json::Value>> = serde_json::from_str(&raw).unwrap_or_default();
        for list in cases.values() {
            for c in list {
                if let Some(q) = c["query"].as_str() {
                    texts.push(q.to_string());
                }
            }
        }
    }
    texts
}

#[test]
fn compact_tokenizer_matches_the_reference_everywhere() {
    let Some(dir) = model_dir() else {
        eprintln!("tokenizer.json absent, test skipped");
        return;
    };
    let ours = Unigram::from_file(&dir.join("tokenizer.json"), 512).expect("compact tokenizer");
    let mut reference = tokenizers::Tokenizer::from_file(dir.join("tokenizer.json")).expect("reference");
    reference.with_truncation(Some(tokenizers::TruncationParams { max_length: 512, ..Default::default() })).unwrap();

    let mut texts = edge_cases();
    texts.extend(corpus_texts());
    let mut mismatches = 0;
    for t in &texts {
        let a = ours.encode(t);
        let b = reference.encode(t.as_str(), true).unwrap();
        let same = a.ids == b.get_ids() && a.truncated != b.get_overflowing().is_empty();
        if !same {
            mismatches += 1;
            if mismatches <= 3 {
                let first = a.ids.iter().zip(b.get_ids()).position(|(x, y)| x != y);
                eprintln!(
                    "mismatch on {:?}: ours {} ids, reference {} ids, first difference at {:?}",
                    t.chars().take(60).collect::<String>(),
                    a.ids.len(),
                    b.get_ids().len(),
                    first
                );
            }
        }
    }
    assert_eq!(mismatches, 0, "{mismatches} mismatch(es) on {} texts", texts.len());
}

/// The native SentencePiece model must produce exactly the same tokenizer as
/// `tokenizer.json`.
#[test]
fn native_sentencepiece_matches_tokenizer_json() {
    let Some(dir) = model_dir() else {
        eprintln!("tokenizer.json absent, test skipped");
        return;
    };
    let native = dir.join("sentencepiece.bpe.model");
    if !native.exists() {
        eprintln!("sentencepiece.bpe.model absent, test skipped");
        return;
    }
    let a = Unigram::from_sentencepiece(&native, 512).expect("native model");
    let b = Unigram::from_file(&dir.join("tokenizer.json"), 512).expect("tokenizer.json");
    assert_eq!(a.vocab_size(), b.vocab_size(), "vocabulary size");
    let mut texts = edge_cases();
    texts.extend(corpus_texts());
    let mut mismatches = 0;
    for t in &texts {
        let (x, y) = (a.encode(t), b.encode(t));
        if x.ids != y.ids || x.truncated != y.truncated {
            mismatches += 1;
            if mismatches <= 3 {
                eprintln!(
                    "mismatch on {:?}\n  native {:?}\n  json   {:?}",
                    t.chars().take(60).collect::<String>(),
                    &x.ids[..x.ids.len().min(20)],
                    &y.ids[..y.ids.len().min(20)]
                );
            }
        }
    }
    assert_eq!(mismatches, 0, "{mismatches} mismatch(es) on {} texts", texts.len());
}
