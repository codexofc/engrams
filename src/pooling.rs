//! How token vectors become one document vector.
//!
//! Pooling is read from the model, never guessed: mean-pooling a model that
//! declares `cls` silently costs fourteen points of recall.

use candle_core::{IndexOp, Result, Tensor};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pooling {
    /// The first token carries the document.
    Cls,
    /// Mean of the real tokens, padding excluded.
    Mean,
}

impl Pooling {
    /// Reads `1_Pooling/config.json`. Refuses an unknown configuration rather than
    /// falling back to a default.
    pub fn from_config(json: &str) -> std::result::Result<Self, String> {
        let v: serde_json::Value = serde_json::from_str(json).map_err(|e| format!("unreadable pooling config: {e}"))?;
        let is_true = |key: &str| v.get(key).and_then(serde_json::Value::as_bool) == Some(true);
        match (is_true("pooling_mode_cls_token"), is_true("pooling_mode_mean_tokens")) {
            (true, false) => Ok(Pooling::Cls),
            (false, true) => Ok(Pooling::Mean),
            _ => Err("unsupported pooling: the model declares neither cls nor mean alone".into()),
        }
    }
}

/// Reduces `[batch, tokens, dim]` to `[batch, dim]`. The mask matters: without it,
/// padding pulls every vector towards zero.
pub fn pool(hidden: &Tensor, mask: &Tensor, how: Pooling) -> Result<Tensor> {
    match how {
        Pooling::Cls => hidden.i((.., 0))?.contiguous(),
        Pooling::Mean => {
            let mask = mask.to_dtype(hidden.dtype())?.unsqueeze(2)?;
            let sum = hidden.broadcast_mul(&mask)?.sum(1)?;
            // Floor at 1: a fully masked sequence would give NaN.
            let count = mask.sum(1)?.maximum(1.0)?;
            sum.broadcast_div(&count)
        }
    }
}
