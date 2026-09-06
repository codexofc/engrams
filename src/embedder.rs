//! Model loading and encoding.
//!
//! No download, at build time or at run time: the model directory is given, or
//! loading fails naming the missing file. CPU only, F32 activations, Q8 weights on
//! the linear layers by default. Measured on the reference corpus: a Metal F16 path
//! compiled its kernels for 9 s per process and produced NaN vectors on real
//! paragraphs, so it was removed.

use crate::model::{Architecture, ModelConfig};
use crate::models::Prompts;
use crate::modernbert::{Config as ModernConfig, ModernBertModel};
use crate::pooling::{pool, Pooling};
use crate::tokenizer::AnyTokenizer;
use crate::xlm_roberta::{Config as XlmConfig, Precision, XLMRobertaModel};
use candle_core::{DType, Device, Tensor};
use candle_nn::VarBuilder;
use std::path::Path;

/// A normalised vector and the flag "the text was cut by the window".
pub type Encoded = (Vec<f32>, bool);

enum Graph {
    XlmRoberta(Box<XLMRobertaModel>),
    ModernBert(Box<ModernBertModel>),
}

pub struct Embedder {
    model: Graph,
    tokenizer: AnyTokenizer,
    pooling: Pooling,
    device: Device,
    config: ModelConfig,
    prompts: Prompts,
}

impl std::fmt::Debug for Embedder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Embedder").field("pooling", &self.pooling).finish_non_exhaustive()
    }
}

impl Embedder {
    /// Loads a model from a local directory. Configuration and pooling are read
    /// before the weights, so an incomplete directory names what is missing.
    pub fn load(dir: &Path) -> Result<Self, String> {
        let config = read_text(dir, "config.json")?;
        let mut config = ModelConfig::parse(&config)?;
        // sentence-transformers caps the sequence below the position table.
        if let Ok(sbert) = std::fs::read_to_string(dir.join("sentence_bert_config.json")) {
            if let Some(n) = serde_json::from_str::<serde_json::Value>(&sbert).ok().and_then(|v| v["max_seq_length"].as_u64()) {
                config.max_tokens = config.max_tokens.min(n as usize);
            }
        }
        let pooling = read_text(dir, "1_Pooling/config.json")?;
        let pooling = Pooling::from_config(&pooling)?;

        if !dir.join("sentencepiece.bpe.model").exists() && !dir.join("tokenizer.json").exists() {
            return Err(format!("neither sentencepiece.bpe.model nor tokenizer.json in {}", dir.display()));
        }
        if !dir.join("model.safetensors").exists() {
            return Err(format!("model.safetensors missing from {}", dir.display()));
        }

        // Truncation happens inside the tokenizer, markers included.
        let tokenizer = AnyTokenizer::from_model_dir(dir, config.max_tokens, config.vocab_size)?;
        let prompts = crate::models::prompts_for(dir);

        let weights = dir.join("model.safetensors");
        let (device, dtype) = (Device::Cpu, DType::F32);
        // SAFETY: the mapped file is read-only and nobody writes it during a call.
        let vb =
            unsafe { VarBuilder::from_mmaped_safetensors(std::slice::from_ref(&weights), dtype, &device).map_err(|e| format!("unreadable weights: {e}"))? };

        let precision = match std::env::var("KEPT_PRECISION").as_deref() {
            Ok("f32") => Precision::F32,
            _ => Precision::Q8,
        };
        let model = match config.architecture {
            Architecture::XlmRoberta | Architecture::Bert => {
                let raw = read_text(dir, "config.json")?;
                let cfg: XlmConfig = serde_json::from_str(&raw).map_err(|e| format!("config incompatible with the BERT graph: {e}"))?;
                // Q8_0 on the linear layers by default: weights only, F32 activations,
                // cosine 0.9999 with F32 on real paragraphs. `KEPT_PRECISION=f32`
                // restores full precision.
                let bert = config.architecture == Architecture::Bert;
                Graph::XlmRoberta(Box::new(XLMRobertaModel::new(&cfg, vb, &weights, precision, &device, bert).map_err(|e| format!("graph: {e}"))?))
            }
            Architecture::ModernBert => {
                let raw = read_text(dir, "config.json")?;
                let cfg: ModernConfig = serde_json::from_str(&raw).map_err(|e| format!("config incompatible with ModernBERT: {e}"))?;
                // Some exports carry a `model.` prefix, some do not.
                let vb = vb.rename_f(|name: &str| name.strip_prefix("model.").unwrap_or(name).to_string());
                Graph::ModernBert(Box::new(ModernBertModel::new(&cfg, vb, &weights, precision, &device).map_err(|e| format!("graph: {e}"))?))
            }
        };

        Ok(Embedder { model, tokenizer, pooling, device, config, prompts })
    }

    /// The prefixes this model expects; empty for most.
    pub fn prompts(&self) -> &Prompts {
        &self.prompts
    }

    /// A document text as the model wants to see it.
    pub fn document_text(&self, text: &str) -> String {
        format!("{}{text}", self.prompts.document)
    }

    /// A query text as the model wants to see it.
    pub fn query_text(&self, text: &str) -> String {
        format!("{}{text}", self.prompts.query)
    }

    /// Encodes a query, prefix included.
    pub fn encode_query(&self, text: &str) -> Result<Vec<f32>, String> {
        self.encode(&self.query_text(text))
    }

    pub fn dim(&self) -> usize {
        self.config.hidden_size
    }

    /// The model window in tokens. Chunk size derives from it.
    pub fn window(&self) -> usize {
        self.config.max_tokens
    }

    pub fn pooling(&self) -> Pooling {
        self.pooling
    }

    /// Whether this text gets cut by the model window.
    pub fn would_truncate(&self, text: &str) -> bool {
        self.tokenizer.encode(text).truncated
    }

    /// Token count of the text, markers included, after truncation.
    pub fn token_count(&self, text: &str) -> usize {
        self.tokenizer.encode(text).ids.len()
    }

    /// Encodes a text into a normalised vector, so the cosine reduces to a dot
    /// product at search time.
    pub fn encode(&self, text: &str) -> Result<Vec<f32>, String> {
        self.encode_checked(text).map(|(v, _)| v)
    }

    /// Encodes and says whether the text was cut, with a single tokenisation.
    pub fn encode_checked(&self, text: &str) -> Result<Encoded, String> {
        let enc = self.tokenizer.encode(text);
        let truncated = enc.truncated;
        let ids = &enc.ids;
        let n = ids.len();
        let input = Tensor::from_slice(ids, (1, n), &self.device).map_err(to_msg)?;
        // One text at a time, no padding: the mask is full.
        let mask = Tensor::ones((1, n), DType::U32, &self.device).map_err(to_msg)?;
        let token_types = Tensor::zeros((1, n), DType::U32, &self.device).map_err(to_msg)?;
        let hidden = match &self.model {
            Graph::XlmRoberta(m) => m.forward(&input, &mask, &token_types).map_err(to_msg)?,
            Graph::ModernBert(m) => m.forward(&input).map_err(to_msg)?,
        };
        let v = pool(&hidden, &mask, self.pooling).map_err(to_msg)?;
        let v: Vec<f32> = v.squeeze(0).map_err(to_msg)?.to_dtype(DType::F32).map_err(to_msg)?.to_vec1().map_err(to_msg)?;

        // A NaN must never reach the index: it serialises as null and makes the
        // index unreadable.
        if v.iter().any(|x| !x.is_finite()) {
            return Err(format!("non-finite vector for a text of {} characters, encoding refused", text.chars().count()));
        }
        let norm = v.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm == 0.0 {
            return Ok((v, truncated));
        }
        Ok((v.into_iter().map(|x| x / norm).collect(), truncated))
    }

    /// Encodes several texts in parallel and returns the vectors in input order.
    /// The model is shared read-only between threads; the output does not depend on
    /// the thread count.
    pub fn encode_many(&self, texts: &[String], threads: usize) -> Result<Vec<Vec<f32>>, String> {
        Ok(self.encode_many_checked(texts, threads)?.into_iter().map(|(v, _)| v).collect())
    }

    /// Like `encode_many`, with the truncation flag of each text.
    pub fn encode_many_checked(&self, texts: &[String], threads: usize) -> Result<Vec<Encoded>, String> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }
        let threads = threads.clamp(1, texts.len());
        let per_thread = texts.len().div_ceil(threads);
        let parts: Vec<Result<Vec<Encoded>, String>> = std::thread::scope(|scope| {
            let handles: Vec<_> = texts.chunks(per_thread).map(|part| scope.spawn(move || part.iter().map(|t| self.encode_checked(t)).collect())).collect();
            handles.into_iter().map(|h| h.join().unwrap_or_else(|_| Err("an encoding thread panicked".into()))).collect()
        });
        let mut out = Vec::with_capacity(texts.len());
        for part in parts {
            out.extend(part?);
        }
        Ok(out)
    }
}

fn read_text(dir: &Path, name: &str) -> Result<String, String> {
    std::fs::read_to_string(dir.join(name)).map_err(|_| format!("{name} missing from {}", dir.display()))
}

fn to_msg(e: candle_core::Error) -> String {
    e.to_string()
}
