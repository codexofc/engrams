//! Where things live.
//!
//! The root is the directory of notes. Everything derived from them sits in
//! `<root>/.engram/`, so a single ignore rule keeps it out of version control. The
//! model is shared across roots under `~/.engram/models/`.

use std::path::{Path, PathBuf};

/// Hugging Face repository of the default model.
pub const DEFAULT_MODEL_REPO: &str = "ibm-granite/granite-embedding-278m-multilingual";

pub fn home() -> PathBuf {
    PathBuf::from(std::env::var("HOME").unwrap_or_default())
}

/// The user-level configuration directory, `~/.engram`.
pub fn config_dir() -> PathBuf {
    home().join(".engram")
}

/// The notes directory: `ENGRAM_ROOT`, else the path written by `engram init` in
/// `~/.engram/root`, else `~/engram`.
pub fn root() -> PathBuf {
    if let Ok(r) = std::env::var("ENGRAM_ROOT") {
        return PathBuf::from(r);
    }
    if let Ok(pointer) = std::fs::read_to_string(config_dir().join("root")) {
        let p = pointer.trim();
        if !p.is_empty() {
            return PathBuf::from(p);
        }
    }
    home().join("engram")
}

/// Derived state of a root: index, caches, logs, socket.
pub fn state_dir(root: &Path) -> PathBuf {
    root.join(".engram")
}

/// The model directory: `ENGRAM_MODEL`, else `~/.engram/models/<default model>`.
pub fn model_dir() -> PathBuf {
    std::env::var("ENGRAM_MODEL")
        .map(PathBuf::from)
        .unwrap_or_else(|_| config_dir().join("models").join(DEFAULT_MODEL_REPO.rsplit('/').next().unwrap_or(DEFAULT_MODEL_REPO)))
}

/// Path of a note relative to the root. This is its identity, never its `name`
/// field, which two projects may share.
pub fn relative(file: &Path, root: &Path) -> String {
    file.strip_prefix(root).unwrap_or(file).to_string_lossy().into_owned()
}

/// Hugging Face repository of the long-context English model.
pub const ALT_MODEL_REPO: &str = "ibm-granite/granite-embedding-small-english-r2";

/// Directory of a model repository under `~/.engram/models/`.
pub fn model_dir_of(repo: &str) -> PathBuf {
    config_dir().join("models").join(repo.rsplit('/').next().unwrap_or(repo))
}

/// The long-context English model, for tests and the model choice.
pub fn alt_model_dir() -> PathBuf {
    std::env::var("ENGRAM_MODEL_ALT").map(PathBuf::from).unwrap_or_else(|_| model_dir_of(ALT_MODEL_REPO))
}
