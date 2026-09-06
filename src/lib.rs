//! Local semantic memory for coding agents.
//!
//! Markdown notes are the source of truth. Everything else (vectors, hot indexes,
//! caches) is derived and disposable. No server, no network, no database: one binary,
//! a directory of notes, and a rebuildable index.

pub mod check;
pub mod chunking;
pub mod duplicates;
pub mod embedder;
pub mod feedback;
pub mod hot;
pub mod index;
pub mod lifecycle;
pub mod model;
pub mod note;
pub mod paths;
pub mod pooling;
pub mod questions;
pub mod secrets;
pub mod similarity;
pub mod tokenizer;
pub mod xlm_roberta;

pub use note::Note;
pub use similarity::{cosine, rank, Hit};
