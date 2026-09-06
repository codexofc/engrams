//! A lean XLM-RoBERTa, derived from candle-transformers 0.9 (Apache-2.0 / MIT).
//!
//! Two differences from the original, both for a resident process that must stay
//! small. The embedding table (250 002 x 768, 768 MB in F32, 70 % of the weights)
//! is never loaded: a query only reads the rows of its tokens, gathered from the
//! memory-mapped file and converted on the fly, bit-identical result. The linear
//! layers can be quantised to Q8_0 with F32 activations. The rest of the graph is
//! the original, without cross-attention or key cache, which an encoder never uses.

use candle_core::quantized::{GgmlDType, QMatMul, QTensor};
use candle_core::safetensors::MmapedSafetensors;
use candle_core::{DType, Device, Module, Result, Tensor};
use candle_nn::{embedding, layer_norm, ops::softmax_last_dim, Activation, Embedding, LayerNorm, VarBuilder};
use std::path::Path;
use std::sync::Arc;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Config {
    pub hidden_size: usize,
    pub layer_norm_eps: f64,
    pub num_attention_heads: usize,
    pub intermediate_size: usize,
    pub hidden_act: Activation,
    pub num_hidden_layers: usize,
    pub vocab_size: usize,
    pub max_position_embeddings: usize,
    pub type_vocab_size: usize,
    pub pad_token_id: u32,
}

/// Precision of the linear layers. The embedding table is never resident.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Precision {
    F32,
    Q8,
}

fn msg(s: String) -> candle_core::Error {
    candle_core::Error::Msg(s)
}

/// Embedding table read on demand from the memory-mapped file.
struct LazyTable {
    file: Arc<MmapedSafetensors>,
    name: String,
    hidden: usize,
    vocab: usize,
    device: Device,
}

impl LazyTable {
    fn rows(&self, ids: &[u32]) -> Result<Vec<f32>> {
        let tensors = self.file.tensors();
        let (_, view) = tensors.iter().find(|(n, _)| *n == self.name).ok_or_else(|| msg(format!("tensor {} missing from the weights", self.name)))?;
        let data = view.data();
        let dtype = format!("{:?}", view.dtype());
        let width = match dtype.as_str() {
            "BF16" | "F16" => 2,
            "F32" => 4,
            other => return Err(msg(format!("embedding table in {other}, unsupported"))),
        };
        let mut out = Vec::with_capacity(ids.len() * self.hidden);
        for &id in ids {
            let start = id as usize * self.hidden * width;
            let row = &data[start..start + self.hidden * width];
            match dtype.as_str() {
                "BF16" => out.extend(row.as_chunks::<2>().0.iter().map(|c| half::bf16::from_bits(u16::from_le_bytes(*c)).to_f32())),
                "F16" => out.extend(row.as_chunks::<2>().0.iter().map(|c| half::f16::from_bits(u16::from_le_bytes(*c)).to_f32())),
                _ => out.extend(row.as_chunks::<4>().0.iter().map(|c| f32::from_le_bytes(*c))),
            }
        }
        Ok(out)
    }

    fn forward(&self, input_ids: &Tensor) -> Result<Tensor> {
        let (b, n) = input_ids.dims2()?;
        let ids: Vec<u32> = input_ids.flatten_all()?.to_vec1()?;
        if let Some(bad) = ids.iter().find(|&&i| i as usize >= self.vocab) {
            return Err(msg(format!("token id {bad} outside the table ({})", self.vocab)));
        }
        Tensor::from_vec(self.rows(&ids)?, (b, n, self.hidden), &self.device)
    }
}

/// Linear layer in F32 or Q8_0 (weights only, F32 activations).
enum Dense {
    Full(candle_nn::Linear),
    Quant { weight: QMatMul, bias: Option<Tensor> },
}

impl Dense {
    fn new(in_dim: usize, out_dim: usize, vb: VarBuilder, precision: Precision) -> Result<Self> {
        match precision {
            Precision::F32 => Ok(Dense::Full(candle_nn::linear(in_dim, out_dim, vb)?)),
            Precision::Q8 => {
                let w = vb.get((out_dim, in_dim), "weight")?;
                let bias = vb.get(out_dim, "bias").ok();
                let q = QTensor::quantize(&w, GgmlDType::Q8_0)?;
                Ok(Dense::Quant { weight: QMatMul::from_qtensor(q)?, bias })
            }
        }
    }

    fn forward(&self, x: &Tensor) -> Result<Tensor> {
        match self {
            Dense::Full(l) => l.forward(x),
            Dense::Quant { weight, bias } => {
                let y = weight.forward(x)?;
                match bias {
                    Some(b) => y.broadcast_add(b),
                    None => Ok(y),
                }
            }
        }
    }
}

struct Embeddings {
    word: LazyTable,
    position: Embedding,
    token_type: Embedding,
    layer_norm: LayerNorm,
    padding_idx: u32,
    /// BERT counts positions from zero; RoBERTa from `padding_idx + 1`.
    bert_positions: bool,
}

impl Embeddings {
    fn load(vb: VarBuilder, cfg: &Config, file: Arc<MmapedSafetensors>, device: &Device, bert_positions: bool) -> Result<Self> {
        Ok(Self {
            word: LazyTable {
                file,
                name: {
                    let prefix = vb.prefix();
                    if prefix.is_empty() {
                        "word_embeddings.weight".to_string()
                    } else {
                        format!("{prefix}.word_embeddings.weight")
                    }
                },
                hidden: cfg.hidden_size,
                vocab: cfg.vocab_size,
                device: device.clone(),
            },
            position: embedding(cfg.max_position_embeddings, cfg.hidden_size, vb.pp("position_embeddings"))?,
            token_type: embedding(cfg.type_vocab_size, cfg.hidden_size, vb.pp("token_type_embeddings"))?,
            layer_norm: layer_norm(cfg.hidden_size, cfg.layer_norm_eps, vb.pp("LayerNorm"))?,
            padding_idx: cfg.pad_token_id,
            bert_positions,
        })
    }

    fn forward(&self, input_ids: &Tensor, token_type_ids: &Tensor) -> Result<Tensor> {
        let words = self.word.forward(input_ids)?;
        let mut e = (&words + self.token_type.forward(token_type_ids)?)?;
        let position_ids = if self.bert_positions {
            let (b, n) = input_ids.dims2()?;
            Tensor::arange(0u32, n as u32, words.device())?.unsqueeze(0)?.repeat((b, 1))?
        } else {
            // RoBERTa positions: counted from padding_idx + 1, padding keeps padding_idx.
            let mask = input_ids.ne(self.padding_idx)?.to_dtype(DType::F32)?;
            (mask.cumsum(1)? * &mask)?.broadcast_add(&Tensor::new(self.padding_idx as f32, words.device())?)?.to_dtype(DType::U32)?
        };
        e = e.broadcast_add(&self.position.forward(&position_ids)?)?;
        self.layer_norm.forward(&e)
    }
}

struct SelfAttention {
    heads: usize,
    head_size: usize,
    query: Dense,
    key: Dense,
    value: Dense,
}

impl SelfAttention {
    fn new(cfg: &Config, vb: VarBuilder, p: Precision) -> Result<Self> {
        let head_size = cfg.hidden_size / cfg.num_attention_heads;
        let all = cfg.num_attention_heads * head_size;
        Ok(Self {
            heads: cfg.num_attention_heads,
            head_size,
            query: Dense::new(cfg.hidden_size, all, vb.pp("query"), p)?,
            key: Dense::new(cfg.hidden_size, all, vb.pp("key"), p)?,
            value: Dense::new(cfg.hidden_size, all, vb.pp("value"), p)?,
        })
    }

    fn split_heads(&self, x: &Tensor) -> Result<Tensor> {
        let (b, n, _) = x.dims3()?;
        x.reshape((b, n, self.heads, self.head_size))?.permute((0, 2, 1, 3))?.contiguous()
    }

    fn forward(&self, hidden: &Tensor, mask: &Tensor) -> Result<Tensor> {
        let q = self.split_heads(&self.query.forward(hidden)?)?;
        let k = self.split_heads(&self.key.forward(hidden)?)?;
        let v = self.split_heads(&self.value.forward(hidden)?)?;
        let scores = (q.matmul(&k.transpose(2, 3)?)? * (1f64 / (self.head_size as f64).sqrt()))?;
        let scores = scores.broadcast_add(mask)?;
        let probs = softmax_last_dim(&scores)?;
        let context = probs.matmul(&v)?.permute((0, 2, 1, 3))?.contiguous()?;
        let (b, n, _, _) = context.dims4()?;
        context.reshape((b, n, self.heads * self.head_size))
    }
}

struct Residual {
    dense: Dense,
    layer_norm: LayerNorm,
}

impl Residual {
    fn new(in_dim: usize, cfg: &Config, vb: VarBuilder, p: Precision) -> Result<Self> {
        Ok(Self {
            dense: Dense::new(in_dim, cfg.hidden_size, vb.pp("dense"), p)?,
            layer_norm: layer_norm(cfg.hidden_size, cfg.layer_norm_eps, vb.pp("LayerNorm"))?,
        })
    }

    fn forward(&self, x: &Tensor, input: &Tensor) -> Result<Tensor> {
        self.layer_norm.forward(&(self.dense.forward(x)? + input)?)
    }
}

struct Layer {
    attention: SelfAttention,
    attention_out: Residual,
    intermediate: Dense,
    act: Activation,
    output: Residual,
}

impl Layer {
    fn new(cfg: &Config, vb: VarBuilder, p: Precision) -> Result<Self> {
        Ok(Self {
            attention: SelfAttention::new(cfg, vb.pp("attention.self"), p)?,
            attention_out: Residual::new(cfg.hidden_size, cfg, vb.pp("attention.output"), p)?,
            intermediate: Dense::new(cfg.hidden_size, cfg.intermediate_size, vb.pp("intermediate.dense"), p)?,
            act: cfg.hidden_act,
            output: Residual::new(cfg.intermediate_size, cfg, vb.pp("output"), p)?,
        })
    }

    fn forward(&self, hidden: &Tensor, mask: &Tensor) -> Result<Tensor> {
        let a = self.attention_out.forward(&self.attention.forward(hidden, mask)?, hidden)?;
        let i = self.act.forward(&self.intermediate.forward(&a)?)?;
        self.output.forward(&i, &a)
    }
}

pub struct XLMRobertaModel {
    embeddings: Embeddings,
    layers: Vec<Layer>,
}

impl XLMRobertaModel {
    /// `weights` is the safetensors file itself: the embedding table is read from
    /// it on demand. `vb` serves the remaining tensors.
    pub fn new(cfg: &Config, vb: VarBuilder, weights: &Path, precision: Precision, device: &Device, bert_positions: bool) -> Result<Self> {
        // SAFETY: the model file is read-only for the lifetime of the process.
        let file = Arc::new(unsafe { MmapedSafetensors::new(weights)? });
        let layers = (0..cfg.num_hidden_layers).map(|i| Layer::new(cfg, vb.pp(format!("encoder.layer.{i}")), precision)).collect::<Result<Vec<_>>>()?;
        let embeddings = Embeddings::load(vb.pp("embeddings"), cfg, file, device, bert_positions)?;
        Ok(Self { embeddings, layers })
    }

    pub fn forward(&self, input_ids: &Tensor, attention_mask: &Tensor, token_type_ids: &Tensor) -> Result<Tensor> {
        let mut hidden = self.embeddings.forward(input_ids, token_type_ids)?;
        // Additive mask (0 for a token, -inf for padding), broadcast over the heads.
        let (b, n) = attention_mask.dims2()?;
        let mask = ((1.0 - attention_mask.to_dtype(DType::F32)?)? * (f32::MIN as f64))?.reshape((b, 1, 1, n))?;
        for layer in &self.layers {
            hidden = layer.forward(&hidden, &mask)?;
        }
        Ok(hidden)
    }
}
