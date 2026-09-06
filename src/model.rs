//! Model configuration: which architecture to load, and where the window ends.

use serde_json::Value;

/// Supported architectures. Anything else is refused: loading weights into the
/// wrong graph yields plausible, wrong vectors, which is worse than an error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Architecture {
    XlmRoberta,
    ModernBert,
}

impl Architecture {
    fn from_name(name: &str) -> Option<Self> {
        match name {
            "XLMRobertaModel" | "XLMRobertaForMaskedLM" => Some(Self::XlmRoberta),
            "ModernBertModel" | "ModernBertForMaskedLM" => Some(Self::ModernBert),
            _ => None,
        }
    }
}

/// What the model configuration imposes on the encoder.
#[derive(Debug, Clone)]
pub struct ModelConfig {
    pub architecture: Architecture,
    /// Tokens the model can encode, sequence markers excluded.
    pub max_tokens: usize,
    pub hidden_size: usize,
}

impl ModelConfig {
    /// Reads the `config.json` shipped with the model.
    pub fn parse(json: &str) -> Result<Self, String> {
        let v: Value = serde_json::from_str(json).map_err(|e| format!("unreadable config: {e}"))?;
        let names = v.get("architectures").and_then(Value::as_array).ok_or("config without `architectures`")?;
        let architecture =
            names.iter().filter_map(Value::as_str).find_map(Architecture::from_name).ok_or_else(|| format!("unsupported architecture: {names:?}"))?;
        let positions = v.get("max_position_embeddings").and_then(Value::as_u64).ok_or("config without `max_position_embeddings`")? as usize;
        let hidden_size = v.get("hidden_size").and_then(Value::as_u64).ok_or("config without `hidden_size`")? as usize;
        Ok(ModelConfig {
            architecture,
            // Two positions go to the start and end markers.
            max_tokens: positions.saturating_sub(2),
            hidden_size,
        })
    }

    /// Whether a text of this many tokens gets cut by the window.
    pub fn would_truncate(&self, tokens: usize) -> bool {
        tokens > self.max_tokens
    }
}
