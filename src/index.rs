//! Vector storage.
//!
//! The index is derived and disposable: losing it costs a few minutes of rebuild,
//! losing the notes would cost years. Vectors live in one flat allocation, which
//! serialises trivially and could be memory-mapped later without touching callers.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Signature of the binary format; the digit is its version.
const MAGIC: &[u8] = b"MEMIDX1\n";

/// What identifies how the index was produced. Any mismatch triggers a full rebuild:
/// vectors from two models give plausible scores and a ranking that is only noise.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Header {
    pub model: String,
    pub weights_hash: String,
    pub dim: usize,
    pub pooling: String,
}

/// One entry: the chunk key and the fingerprint of its note's content.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Entry {
    path: String,
    hash: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Index {
    header: Header,
    entries: Vec<Entry>,
    /// Flat matrix, `entries.len() * header.dim` floats.
    data: Vec<f32>,
    /// Key to row, derived from `entries`, not serialised.
    #[serde(skip)]
    lookup: HashMap<String, usize>,
}

impl Index {
    pub fn new(header: Header) -> Self {
        Index { header, entries: Vec::new(), data: Vec::new(), lookup: HashMap::new() }
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn header(&self) -> &Header {
        &self.header
    }

    /// Adds or replaces a vector.
    ///
    /// # Panics
    /// If the dimension does not match the header. Use `try_push` to handle it.
    pub fn push(&mut self, path: &str, hash: u64, v: &[f32]) {
        self.try_push(path, hash, v).expect("dimension mismatch");
    }

    pub fn try_push(&mut self, path: &str, hash: u64, v: &[f32]) -> Result<(), String> {
        if v.iter().any(|x| !x.is_finite()) {
            return Err(format!("non-finite vector for {path}: a NaN serialises as null and makes the index unreadable"));
        }
        if v.len() != self.header.dim {
            return Err(format!("vector of {} dimensions for an index of {}", v.len(), self.header.dim));
        }
        match self.lookup.get(path).copied() {
            Some(row) => {
                let start = row * self.header.dim;
                self.data[start..start + self.header.dim].copy_from_slice(v);
                self.entries[row].hash = hash;
            }
            None => {
                self.lookup.insert(path.to_string(), self.entries.len());
                self.entries.push(Entry { path: path.to_string(), hash });
                self.data.extend_from_slice(v);
            }
        }
        Ok(())
    }

    pub fn vector(&self, path: &str) -> Option<&[f32]> {
        let row = *self.lookup.get(path)?;
        let start = row * self.header.dim;
        Some(&self.data[start..start + self.header.dim])
    }

    /// Whether the stored vector still matches the file content.
    pub fn is_fresh(&self, path: &str, hash: u64) -> bool {
        self.lookup.get(path).is_some_and(|&row| self.entries[row].hash == hash)
    }

    pub fn matches(&self, h: &Header) -> bool {
        &self.header == h
    }

    /// Splits a key `path#ordinal` or `path#ordinal?k` (question k of the chunk) into
    /// the note path and the chunk ordinal.
    pub fn split_key(key: &str) -> (&str, usize) {
        let (path, rest) = key.rsplit_once('#').unwrap_or((key, "0"));
        let ordinal = rest.split('?').next().unwrap_or("0").parse().unwrap_or(0);
        (path, ordinal)
    }

    /// Whether the key names a generated question rather than a text chunk.
    pub fn is_question_key(key: &str) -> bool {
        key.rsplit_once('#').is_some_and(|(_, rest)| rest.contains('?'))
    }

    /// Iterates over the vectors without copying.
    pub fn iter(&self) -> impl Iterator<Item = (&str, &[f32])> {
        self.entries.iter().enumerate().map(move |(row, e)| {
            let start = row * self.header.dim;
            (e.path.as_str(), &self.data[start..start + self.header.dim])
        })
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(self).expect("serialisable index")
    }

    /// Writes the index atomically: a temporary file, then a rename. The format is a
    /// JSON header (header and entries) followed by little-endian floats.
    pub fn save(&self, path: &std::path::Path) -> Result<(), String> {
        let tmp = path.with_extension("tmp");
        let meta = serde_json::json!({ "header": self.header, "entries": self.entries });
        let meta = serde_json::to_vec(&meta).map_err(|e| e.to_string())?;
        let mut out = Vec::with_capacity(MAGIC.len() + 4 + meta.len() + self.data.len() * 4);
        out.extend_from_slice(MAGIC);
        out.extend_from_slice(&(meta.len() as u32).to_le_bytes());
        out.extend_from_slice(&meta);
        for x in &self.data {
            out.extend_from_slice(&x.to_le_bytes());
        }
        std::fs::write(&tmp, out).map_err(|e| format!("write: {e}"))?;
        std::fs::rename(&tmp, path).map_err(|e| format!("rename: {e}"))
    }

    /// Reads an index written by `save`. The older JSON format is still accepted.
    pub fn load(path: &std::path::Path) -> Result<Self, String> {
        let bytes = std::fs::read(path).map_err(|e| format!("read: {e}"))?;
        if !bytes.starts_with(MAGIC) {
            let raw = String::from_utf8(bytes).map_err(|_| "index is neither binary nor JSON".to_string())?;
            return Self::from_json(&raw);
        }
        let mut pos = MAGIC.len();
        let take = |pos: &mut usize, n: usize| -> Result<&[u8], String> {
            let end = *pos + n;
            let slice = bytes.get(*pos..end).ok_or("truncated index")?;
            *pos = end;
            Ok(slice)
        };
        let len = u32::from_le_bytes(take(&mut pos, 4)?.try_into().unwrap()) as usize;
        #[derive(Deserialize)]
        struct Meta {
            header: Header,
            entries: Vec<Entry>,
        }
        let meta: Meta = serde_json::from_slice(take(&mut pos, len)?).map_err(|e| format!("header: {e}"))?;
        let rest = &bytes[pos..];
        let expected = meta.entries.len() * meta.header.dim;
        if rest.len() != expected * 4 {
            return Err(format!("inconsistent index: {} bytes of vectors for {} entries of {} dimensions", rest.len(), meta.entries.len(), meta.header.dim));
        }
        let data: Vec<f32> = rest.chunks_exact(4).map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]])).collect();
        let lookup = meta.entries.iter().enumerate().map(|(row, e)| (e.path.clone(), row)).collect();
        Ok(Index { header: meta.header, entries: meta.entries, data, lookup })
    }

    /// Keeps only the entries whose key satisfies `keep`; returns how many were
    /// removed. This purges ghost chunks of deleted or shortened notes.
    pub fn retain<F: Fn(&str) -> bool>(&mut self, keep: F) -> usize {
        let dim = self.header.dim;
        let before = self.entries.len();
        let mut entries = Vec::with_capacity(before);
        let mut data = Vec::with_capacity(self.data.len());
        for (row, e) in self.entries.iter().enumerate() {
            if keep(&e.path) {
                entries.push(e.clone());
                data.extend_from_slice(&self.data[row * dim..(row + 1) * dim]);
            }
        }
        self.entries = entries;
        self.data = data;
        self.lookup = self.entries.iter().enumerate().map(|(row, e)| (e.path.clone(), row)).collect();
        before - self.entries.len()
    }

    pub fn from_json(s: &str) -> Result<Self, String> {
        let mut idx: Index = serde_json::from_str(s).map_err(|e| format!("unreadable index: {e}"))?;
        let expected = idx.entries.len() * idx.header.dim;
        if idx.data.len() != expected {
            return Err(format!("inconsistent index: {} floats for {} entries of {} dimensions", idx.data.len(), idx.entries.len(), idx.header.dim));
        }
        idx.lookup = idx.entries.iter().enumerate().map(|(row, e)| (e.path.clone(), row)).collect();
        Ok(idx)
    }
}

/// Content fingerprint, to detect that a note changed. Not cryptographic, and it
/// does not need to be.
pub fn content_hash(content: &str) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    content.hash(&mut h);
    h.finish()
}
