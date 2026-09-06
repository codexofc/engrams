//! Byte-level BPE tokenizer, as used by ModernBERT-based embedders.
//!
//! Reads a Hugging Face `tokenizer.json` of type BPE with a byte-level
//! pre-tokenizer. The split pattern of those tokenizers contains a lookahead
//! (`\s+(?!\S)`) that the `regex` crate does not support, so the whitespace
//! alternatives are handled by hand and the rest of the pattern is compiled as is.
//! Merges are applied with the same priority order as the reference (lowest rank
//! first, leftmost position on a tie), and a word present in the vocabulary is taken
//! whole when the model says `ignore_merges`. A parity test against the reference
//! crate enforces identical ids.

use crate::tokenizer::Encoding;
use regex::Regex;
use std::collections::HashMap;
use std::path::Path;

pub struct Bpe {
    /// Unicode normalisation form applied before splitting, when the model asks for one.
    nfc: bool,
    vocab: HashMap<String, u32>,
    merges: HashMap<(String, String), usize>,
    ignore_merges: bool,
    /// Byte value to the printable character the byte-level scheme maps it to.
    byte_char: [char; 256],
    split: Regex,
    specials: Vec<(String, u32)>,
    cls: u32,
    sep: u32,
    max_tokens: usize,
}

impl std::fmt::Debug for Bpe {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Bpe").field("vocab", &self.vocab.len()).field("merges", &self.merges.len()).finish_non_exhaustive()
    }
}

/// The GPT-2 byte to unicode table: printable bytes map to themselves, the others
/// to code points from 256 upwards.
fn byte_to_unicode() -> [char; 256] {
    let mut table = ['\0'; 256];
    let printable = |b: u8| (b'!'..=b'~').contains(&b) || (0xa1..=0xac).contains(&b) || (0xae..=0xff).contains(&b);
    let mut next = 256u32;
    for b in 0..=255u8 {
        table[b as usize] = if printable(b) {
            char::from_u32(b as u32).unwrap()
        } else {
            let c = char::from_u32(next).unwrap();
            next += 1;
            c
        };
    }
    table
}

impl Bpe {
    pub fn from_file(path: &Path, max_tokens: usize) -> Result<Self, String> {
        let raw = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
        let json: serde_json::Value = serde_json::from_str(&raw).map_err(|e| format!("unreadable tokenizer.json: {e}"))?;
        let model = &json["model"];
        if model["type"] != "BPE" {
            return Err(format!("unsupported tokenizer model {}, expected BPE", model["type"]));
        }
        let vocab: HashMap<String, u32> =
            model["vocab"].as_object().ok_or("missing vocabulary")?.iter().filter_map(|(k, v)| Some((k.clone(), v.as_u64()? as u32))).collect();
        let mut merges = HashMap::new();
        for (rank, m) in model["merges"].as_array().ok_or("missing merges")?.iter().enumerate() {
            let pair = match m {
                serde_json::Value::Array(p) if p.len() == 2 => (p[0].as_str().unwrap_or("").to_string(), p[1].as_str().unwrap_or("").to_string()),
                serde_json::Value::String(s) => {
                    let (a, b) = s.split_once(' ').ok_or("malformed merge")?;
                    (a.to_string(), b.to_string())
                }
                _ => return Err("malformed merge".into()),
            };
            merges.entry(pair).or_insert(rank);
        }
        let ignore_merges = model["ignore_merges"].as_bool().unwrap_or(false);
        let nfc = json["normalizer"]["type"] == "NFC" || json["normalizer"]["normalizers"].as_array().is_some_and(|a| a.iter().any(|n| n["type"] == "NFC"));

        // The split pattern: explicit in a `Split` pre-tokenizer, else the GPT-2 one when
        // the byte-level pre-tokenizer says `use_regex`. Its trailing whitespace
        // alternatives carry a lookahead, so they are dropped here and done by hand.
        let pre = &json["pre_tokenizer"];
        let explicit =
            pre["pretokenizers"].as_array().and_then(|a| a.iter().find_map(|p| p["pattern"]["Regex"].as_str())).or_else(|| pre["pattern"]["Regex"].as_str());
        let byte_level_regex = pre["use_regex"].as_bool() == Some(true)
            || pre["pretokenizers"].as_array().is_some_and(|a| a.iter().any(|p| p["type"] == "ByteLevel" && p["use_regex"].as_bool() == Some(true)));
        let pattern = match explicit {
            Some(p) => p.to_string(),
            None if byte_level_regex => r"'s|'t|'re|'ve|'m|'ll|'d| ?\p{L}+| ?\p{N}+| ?[^\s\p{L}\p{N}]+|\s+(?!\S)|\s+".to_string(),
            None => return Err("byte-level tokenizer without a split pattern".into()),
        };
        let body = pattern.strip_suffix(r"|\s+(?!\S)|\s+").ok_or("split pattern without the expected whitespace alternatives")?;
        let split = Regex::new(&format!("^(?:{body})")).map_err(|e| format!("split pattern: {e}"))?;

        // Every added token, special or not, is cut out before the split pattern runs:
        // ModernBERT tokenizers register runs of spaces this way.
        let mut specials: Vec<(String, u32)> = json["added_tokens"]
            .as_array()
            .map(|a| a.iter().filter_map(|t| Some((t["content"].as_str()?.to_string(), t["id"].as_u64()? as u32))).collect())
            .unwrap_or_default();
        specials.sort_by_key(|(content, _)| std::cmp::Reverse(content.len()));
        // The template: the first and last special tokens of the single-sequence form.
        let template = json["post_processor"]["single"].as_array().ok_or("missing post-processor template")?;
        let special_id = |entry: &serde_json::Value| -> Option<u32> {
            let id = entry["SpecialToken"]["id"].as_str()?;
            json["post_processor"]["special_tokens"][id]["ids"][0].as_u64().map(|v| v as u32)
        };
        let cls = template.first().and_then(special_id).ok_or("template without a leading special token")?;
        let sep = template.last().and_then(special_id).ok_or("template without a trailing special token")?;

        Ok(Bpe { nfc, vocab, merges, ignore_merges, byte_char: byte_to_unicode(), split, specials, cls, sep, max_tokens })
    }

    pub fn vocab_size(&self) -> usize {
        self.vocab.len()
    }

    /// Encodes a text: leading special token, the pieces, trailing special token,
    /// cut to the window keeping the trailing token.
    pub fn encode(&self, text: &str) -> Encoding {
        let normalized: std::borrow::Cow<str> = if self.nfc {
            use unicode_normalization::UnicodeNormalization;
            std::borrow::Cow::Owned(text.nfc().collect())
        } else {
            std::borrow::Cow::Borrowed(text)
        };
        let mut ids = vec![self.cls];
        let mut rest: &str = &normalized;
        while !rest.is_empty() {
            let next = self
                .specials
                .iter()
                .filter_map(|(s, id)| rest.find(s.as_str()).map(|pos| (pos, s.len(), *id)))
                .min_by_key(|&(pos, len, _)| (pos, std::cmp::Reverse(len)));
            match next {
                Some((pos, len, id)) => {
                    self.encode_segment(&rest[..pos], &mut ids);
                    ids.push(id);
                    rest = &rest[pos + len..];
                }
                None => {
                    self.encode_segment(rest, &mut ids);
                    rest = "";
                }
            }
        }
        let limit = self.max_tokens.saturating_sub(1).max(1);
        let truncated = ids.len() > limit;
        if truncated {
            ids.truncate(limit);
        }
        ids.push(self.sep);
        Encoding { ids, truncated }
    }

    fn encode_segment(&self, segment: &str, out: &mut Vec<u32>) {
        for word in self.pretokenize(segment) {
            let mapped: String = word.bytes().map(|b| self.byte_char[b as usize]).collect();
            self.encode_word(&mapped, out);
        }
    }

    /// The split pattern: the compiled alternatives first, then the whitespace
    /// alternatives `\s+(?!\S)` and `\s+` by hand.
    fn pretokenize<'a>(&self, text: &'a str) -> Vec<&'a str> {
        let mut out = Vec::new();
        let mut pos = 0;
        while pos < text.len() {
            let rest = &text[pos..];
            if let Some(m) = self.split.find(rest) {
                if m.end() > 0 {
                    out.push(&rest[..m.end()]);
                    pos += m.end();
                    continue;
                }
            }
            let run = rest.char_indices().find(|(_, c)| !c.is_whitespace()).map_or(rest.len(), |(i, _)| i);
            if run == 0 {
                // Not whitespace and not matched: one character on its own.
                let c = rest.chars().next().unwrap();
                out.push(&rest[..c.len_utf8()]);
                pos += c.len_utf8();
                continue;
            }
            let followed = run < rest.len();
            let chars = rest[..run].chars().count();
            let take = if followed && chars >= 2 {
                // `\s+(?!\S)`: the run gives its last character to the next token.
                let last = rest[..run].char_indices().last().map_or(run, |(i, _)| i);
                last
            } else {
                run
            };
            out.push(&rest[..take]);
            pos += take;
        }
        out
    }

    /// Merges the characters of a word by rank, lowest first, leftmost on a tie.
    fn encode_word(&self, word: &str, out: &mut Vec<u32>) {
        if word.is_empty() {
            return;
        }
        if self.ignore_merges {
            if let Some(&id) = self.vocab.get(word) {
                out.push(id);
                return;
            }
        }
        let mut symbols: Vec<String> = word.chars().map(|c| c.to_string()).collect();
        loop {
            let mut best: Option<(usize, usize)> = None;
            for i in 0..symbols.len().saturating_sub(1) {
                if let Some(&rank) = self.merges.get(&(symbols[i].clone(), symbols[i + 1].clone())) {
                    if best.is_none_or(|(r, _)| rank < r) {
                        best = Some((rank, i));
                    }
                }
            }
            let Some((_, i)) = best else { break };
            let merged = format!("{}{}", symbols[i], symbols[i + 1]);
            symbols[i] = merged;
            symbols.remove(i + 1);
        }
        for s in symbols {
            match self.vocab.get(&s) {
                Some(&id) => out.push(id),
                // Every single byte is in the vocabulary; a missing symbol means a
                // merge result outside it, which the reference would also refuse.
                None => {
                    for c in s.chars() {
                        if let Some(&id) = self.vocab.get(&c.to_string()) {
                            out.push(id);
                        }
                    }
                }
            }
        }
    }
}
