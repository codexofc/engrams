//! Compact Unigram (SentencePiece) tokenizer for XLM-RoBERTa.
//!
//! The reference crate holds the 250 000 pieces in a trie of hash maps and takes
//! 380 MB resident. Here the table is one hash map piece -> (id, score), about
//! 30 MB, and prefix lookup slices the word at most sixteen characters ahead. The
//! output must be identical to the reference: a parity test enforces it on a whole
//! corpus, because a different id makes a different vector without any signal.
//!
//! Pipeline: special tokens cut out of the raw text, `Precompiled` normalisation
//! (the same crate as the reference), whitespace split, `▁` prefix per word, Viterbi
//! per word with merged `<unk>`, `<s> … </s>` template, truncation keeping `</s>`.

use spm_precompiled::Precompiled;
use std::collections::HashMap;
use std::path::Path;

const UNK_PENALTY: f64 = 10.0;
const SPACE: &str = "\u{2581}";

pub struct Encoding {
    pub ids: Vec<u32>,
    /// The text exceeded the window: the end was cut before `</s>`.
    pub truncated: bool,
}

pub struct Unigram {
    normalizer: Precompiled,
    pieces: HashMap<String, (u32, f64)>,
    max_piece_bytes: usize,
    unk_id: u32,
    unk_score: f64,
    bos: u32,
    eos: u32,
    /// Added tokens, matched verbatim in the raw text, longest first.
    specials: Vec<(String, u32)>,
    max_tokens: usize,
}

impl std::fmt::Debug for Unigram {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Unigram").field("pieces", &self.pieces.len()).finish_non_exhaustive()
    }
}

#[derive(Clone, Copy)]
struct Node {
    score: f64,
    start: Option<usize>,
    id: u32,
}

impl Unigram {
    /// Reads a Hugging Face `tokenizer.json` of type Unigram.
    pub fn from_file(path: &Path, max_tokens: usize) -> Result<Self, String> {
        let raw = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
        let json: serde_json::Value = serde_json::from_str(&raw).map_err(|e| format!("unreadable tokenizer.json: {e}"))?;
        let model = &json["model"];
        if model["type"] != "Unigram" {
            return Err(format!("unsupported tokenizer model {}, expected Unigram", model["type"]));
        }
        let charsmap = json["normalizer"]["precompiled_charsmap"].as_str().ok_or("Precompiled normalizer expected")?;
        let bytes = base64::decode(charsmap).map_err(|e| format!("charsmap: {e}"))?;
        let normalizer = Precompiled::from(&bytes).map_err(|e| format!("charsmap: {e:?}"))?;

        let vocab = model["vocab"].as_array().ok_or("missing vocabulary")?;
        let mut pieces = HashMap::with_capacity(vocab.len());
        let mut min_score = f64::INFINITY;
        let mut max_piece_bytes = 0;
        for (id, entry) in vocab.iter().enumerate() {
            let piece = entry[0].as_str().ok_or("non-text piece")?;
            let score = entry[1].as_f64().ok_or("non-numeric score")?;
            min_score = min_score.min(score);
            max_piece_bytes = max_piece_bytes.max(piece.len());
            // Like the reference: on a duplicate, the last id wins.
            pieces.insert(piece.to_string(), (id as u32, score));
        }
        let unk_id = model["unk_id"].as_u64().ok_or("missing unk_id")? as u32;

        let mut specials: Vec<(String, u32)> = json["added_tokens"]
            .as_array()
            .map(|a| {
                a.iter()
                    .filter(|t| t["special"].as_bool().unwrap_or(false))
                    .filter_map(|t| Some((t["content"].as_str()?.to_string(), t["id"].as_u64()? as u32)))
                    .collect()
            })
            .unwrap_or_default();
        specials.sort_by_key(|(content, _)| std::cmp::Reverse(content.len()));
        let find = |name: &str| specials.iter().find(|(c, _)| c == name).map(|(_, i)| *i);
        let bos = find("<s>").ok_or("missing <s> token")?;
        let eos = find("</s>").ok_or("missing </s> token")?;

        Ok(Self { normalizer, pieces, max_piece_bytes, unk_id, unk_score: min_score - UNK_PENALTY, bos, eos, specials, max_tokens })
    }

    /// Reads the native SentencePiece model (`sentencepiece.bpe.model`, a protobuf
    /// `ModelProto`) without a library: the pieces (field 1) and the precompiled
    /// normalisation table (field 3 then 2). Ids follow the fairseq layout of XLM-R,
    /// the one the embedding table expects: `<s>` 0, `<pad>` 1, `</s>` 2, `<unk>` 3,
    /// then the model pieces from the third one shifted by one, `<mask>` last.
    pub fn from_sentencepiece(path: &Path, max_tokens: usize) -> Result<Self, String> {
        let raw = std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
        let mut spm_pieces: Vec<(String, f64)> = Vec::with_capacity(250_000);
        let mut charsmap: Option<Vec<u8>> = None;
        let mut top = proto::Reader::new(&raw);
        while let Some((field, wire)) = top.field()? {
            match (field, wire) {
                (1, 2) => {
                    let mut p = proto::Reader::new(top.bytes()?);
                    let (mut piece, mut score) = (String::new(), 0f64);
                    while let Some((f, w)) = p.field()? {
                        match (f, w) {
                            (1, 2) => piece = String::from_utf8(p.bytes()?.to_vec()).map_err(|e| format!("non-UTF-8 piece: {e}"))?,
                            (2, 5) => score = p.f32()? as f64,
                            _ => p.skip(w)?,
                        }
                    }
                    spm_pieces.push((piece, score));
                }
                (3, 2) => {
                    let mut n = proto::Reader::new(top.bytes()?);
                    while let Some((f, w)) = n.field()? {
                        match (f, w) {
                            (2, 2) => charsmap = Some(n.bytes()?.to_vec()),
                            _ => n.skip(w)?,
                        }
                    }
                }
                _ => top.skip(wire)?,
            }
        }
        let charsmap = charsmap.ok_or("normalisation table missing from the SentencePiece model")?;
        let normalizer = Precompiled::from(&charsmap).map_err(|e| format!("charsmap: {e:?}"))?;
        if spm_pieces.len() < 4 {
            return Err("SentencePiece model without pieces".into());
        }

        let mut pieces = HashMap::with_capacity(spm_pieces.len() + 2);
        let mut min_score = f64::INFINITY;
        let mut max_piece_bytes = 0;
        let mut id: u32 = 0;
        let mut insert = |pieces: &mut HashMap<String, (u32, f64)>, piece: &str, score: f64, id: &mut u32| {
            min_score = min_score.min(score);
            max_piece_bytes = max_piece_bytes.max(piece.len());
            pieces.insert(piece.to_string(), (*id, score));
            *id += 1;
        };
        for s in ["<s>", "<pad>", "</s>", "<unk>"] {
            insert(&mut pieces, s, 0.0, &mut id);
        }
        for (piece, score) in &spm_pieces[3..] {
            insert(&mut pieces, piece, *score, &mut id);
        }
        insert(&mut pieces, "<mask>", 0.0, &mut id);
        let mask_id = id - 1;
        let mut specials: Vec<(String, u32)> =
            vec![("<s>".into(), 0), ("<pad>".into(), 1), ("</s>".into(), 2), ("<unk>".into(), 3), ("<mask>".into(), mask_id)];
        specials.sort_by_key(|(content, _)| std::cmp::Reverse(content.len()));

        Ok(Self { normalizer, pieces, max_piece_bytes, unk_id: 3, unk_score: min_score - UNK_PENALTY, bos: 0, eos: 2, specials, max_tokens })
    }

    /// The native model when present, else `tokenizer.json`.
    pub fn from_model_dir(dir: &Path, max_tokens: usize) -> Result<Self, String> {
        let native = dir.join("sentencepiece.bpe.model");
        if native.exists() {
            Self::from_sentencepiece(&native, max_tokens)
        } else {
            Self::from_file(&dir.join("tokenizer.json"), max_tokens)
        }
    }

    pub fn vocab_size(&self) -> usize {
        self.pieces.len()
    }

    /// Encodes a text: `<s>`, the pieces, `</s>`, cut to the model window.
    pub fn encode(&self, text: &str) -> Encoding {
        let mut ids = vec![self.bos];
        let mut rest = text;
        while !rest.is_empty() {
            // The earliest special token in the text; on a tie, the longest.
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
        ids.push(self.eos);
        Encoding { ids, truncated }
    }

    fn encode_segment(&self, segment: &str, out: &mut Vec<u32>) {
        if segment.is_empty() {
            return;
        }
        let normalized = self.normalizer.normalize_string(segment);
        for word in normalized.split(char::is_whitespace).filter(|w| !w.is_empty()) {
            let mut w = String::with_capacity(word.len() + 3);
            if !word.starts_with(SPACE) {
                w.push_str(SPACE);
            }
            w.push_str(word);
            self.encode_word(&w, out);
        }
    }

    /// Viterbi on one word: best path left to right, then backtrack.
    fn encode_word(&self, word: &str, out: &mut Vec<u32>) {
        let size = word.len();
        let mut best = vec![Node { score: 0.0, start: None, id: 0 }; size + 1];
        let mut s = 0;
        while s < size {
            let base = best[s].score;
            let mblen = word[s..].chars().next().map_or(1, char::len_utf8);
            let mut has_single = false;
            let mut end = s;
            for ch in word[s..].chars() {
                end += ch.len_utf8();
                if end - s > self.max_piece_bytes {
                    break;
                }
                if let Some(&(id, score)) = self.pieces.get(&word[s..end]) {
                    let cand = score + base;
                    let node = &mut best[end];
                    if node.start.is_none() || cand > node.score {
                        *node = Node { score: cand, start: Some(s), id };
                    }
                    if end - s == mblen {
                        has_single = true;
                    }
                }
            }
            if !has_single {
                let cand = self.unk_score + base;
                let node = &mut best[s + mblen];
                if node.start.is_none() || cand > node.score {
                    *node = Node { score: cand, start: Some(s), id: self.unk_id };
                }
            }
            s += mblen;
        }
        let mut reversed = Vec::new();
        let mut end = size;
        let mut in_unk = false;
        while end > 0 {
            let node = best[end];
            let start = node.start.expect("complete path");
            if node.id == self.unk_id {
                // Consecutive <unk> merge into one, like the reference.
                if !in_unk {
                    reversed.push(self.unk_id);
                    in_unk = true;
                }
            } else {
                reversed.push(node.id);
                in_unk = false;
            }
            end = start;
        }
        out.extend(reversed.into_iter().rev());
    }
}

/// The bare minimum of the protobuf wire format: varint, length-delimited, fixed32,
/// and skipping the fields we do not read.
mod proto {
    pub struct Reader<'a> {
        buf: &'a [u8],
        pos: usize,
    }

    impl<'a> Reader<'a> {
        pub fn new(buf: &'a [u8]) -> Self {
            Reader { buf, pos: 0 }
        }

        fn varint(&mut self) -> Result<u64, String> {
            let (mut v, mut shift) = (0u64, 0);
            loop {
                let b = *self.buf.get(self.pos).ok_or("truncated protobuf")?;
                self.pos += 1;
                v |= u64::from(b & 0x7f) << shift;
                if b & 0x80 == 0 {
                    return Ok(v);
                }
                shift += 7;
                if shift > 63 {
                    return Err("varint too long".into());
                }
            }
        }

        /// Number and wire type of the next field, `None` at the end of the buffer.
        pub fn field(&mut self) -> Result<Option<(u32, u8)>, String> {
            if self.pos >= self.buf.len() {
                return Ok(None);
            }
            let key = self.varint()?;
            Ok(Some(((key >> 3) as u32, (key & 7) as u8)))
        }

        pub fn bytes(&mut self) -> Result<&'a [u8], String> {
            let len = self.varint()? as usize;
            let end = self.pos.checked_add(len).filter(|&e| e <= self.buf.len()).ok_or("truncated protobuf")?;
            let out = &self.buf[self.pos..end];
            self.pos = end;
            Ok(out)
        }

        pub fn f32(&mut self) -> Result<f32, String> {
            let end = self.pos + 4;
            let b = self.buf.get(self.pos..end).ok_or("truncated protobuf")?;
            self.pos = end;
            Ok(f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
        }

        pub fn skip(&mut self, wire: u8) -> Result<(), String> {
            match wire {
                0 => self.varint().map(|_| ()),
                1 => {
                    self.pos += 8;
                    Ok(())
                }
                2 => self.bytes().map(|_| ()),
                5 => {
                    self.pos += 4;
                    Ok(())
                }
                w => Err(format!("unknown protobuf wire type {w}")),
            }
        }
    }
}
