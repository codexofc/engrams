//! Where things live.
//!
//! The root is the directory of notes. Everything derived from them sits in
//! `<root>/.kept/`, so a single ignore rule keeps it out of version control. The
//! model is shared across roots under `~/.kept/models/`.

use std::path::{Path, PathBuf};

/// Hugging Face repository of the default model.
pub const DEFAULT_MODEL_REPO: &str = "ibm-granite/granite-embedding-278m-multilingual";

pub fn home() -> PathBuf {
    PathBuf::from(std::env::var("HOME").unwrap_or_default())
}

/// The user-level configuration directory, `~/.kept`.
pub fn config_dir() -> PathBuf {
    home().join(".kept")
}

/// The notes directory: `KEPT_ROOT`, else the path written by `kept init` in
/// `~/.kept/root`, else `~/kept`.
pub fn root() -> PathBuf {
    if let Ok(r) = std::env::var("KEPT_ROOT") {
        return PathBuf::from(r);
    }
    if let Ok(pointer) = std::fs::read_to_string(config_dir().join("root")) {
        let p = pointer.trim();
        if !p.is_empty() {
            return PathBuf::from(p);
        }
    }
    home().join("kept")
}

/// Derived state of a root: index, caches, logs, socket.
pub fn state_dir(root: &Path) -> PathBuf {
    root.join(".kept")
}

/// The model directory: `KEPT_MODEL`, else `~/.kept/models/<default model>`.
pub fn model_dir() -> PathBuf {
    std::env::var("KEPT_MODEL")
        .map(PathBuf::from)
        .unwrap_or_else(|_| config_dir().join("models").join(DEFAULT_MODEL_REPO.rsplit('/').next().unwrap_or(DEFAULT_MODEL_REPO)))
}

/// Path of a note relative to the root. This is its identity, never its `name`
/// field, which two projects may share.
pub fn relative(file: &Path, root: &Path) -> String {
    file.strip_prefix(root).unwrap_or(file).to_string_lossy().into_owned()
}

/// Hugging Face repository of the long-context multilingual ModernBERT model.
pub const ALT_MODEL_REPO: &str = "ibm-granite/granite-embedding-97m-multilingual-r2";

/// Directory of a model repository under `~/.kept/models/`.
pub fn model_dir_of(repo: &str) -> PathBuf {
    config_dir().join("models").join(repo.rsplit('/').next().unwrap_or(repo))
}

/// The long-context ModernBERT model, for tests and the model choice.
pub fn alt_model_dir() -> PathBuf {
    std::env::var("KEPT_MODEL_ALT").map(PathBuf::from).unwrap_or_else(|_| model_dir_of(ALT_MODEL_REPO))
}
