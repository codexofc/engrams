//! Tokenizer stage: load time and resident memory of the native SentencePiece
//! model against tokenizer.json, on the same vocabulary.
use std::time::Instant;

fn rss_mb() -> f64 {
    let out = std::process::Command::new("ps").args(["-o", "rss=", "-p", &std::process::id().to_string()]).output().unwrap();
    String::from_utf8_lossy(&out.stdout).trim().parse::<f64>().unwrap_or(0.0) / 1024.0
}

fn main() {
    let dir = engrams::paths::model_dir();
    let which = std::env::args().nth(1).unwrap_or_else(|| "native".into());
    let before = rss_mb();
    let t = Instant::now();
    let tok = match which.as_str() {
        "json" => engrams::tokenizer::Unigram::from_file(&dir.join("tokenizer.json"), 512).unwrap(),
        _ => engrams::tokenizer::Unigram::from_sentencepiece(&dir.join("sentencepiece.bpe.model"), 512).unwrap(),
    };
    let load = t.elapsed();
    let n = tok.encode("how are the databases isolated between agents, port 3334, release-v2").ids.len();
    println!("{which:<6} load {:>4} ms  rss +{:.0} MB  ({} pieces, {n} tokens)", load.as_millis(), rss_mb() - before, tok.vocab_size());
}
