//! ModernBERT encoder, written after the reference implementation.
//!
//! The library implementation hard-codes a GELU activation in the MLP; the models
//! served here declare `silu`, which was enough to push the cosine with the
//! reference down to 0.85. This graph reads the activation from the configuration,
//! keeps the embedding table in the mapped file (rows gathered on demand), and can
//! quantise the linear layers to Q8_0 like the XLM-RoBERTa graph. Alternating
//! global and local (sliding window) attention with two rotary bases, pre-norm
//! layers without bias, gated MLP.

use candle_core::quantized::{GgmlDType, QMatMul, QTensor};
use candle_core::safetensors::MmapedSafetensors;
use candle_core::{DType, Device, Module, Result, Tensor, D};
use candle_nn::{ops::softmax_last_dim, Activation, LayerNorm, VarBuilder};
use std::path::Path;
use std::sync::Arc;

use crate::xlm_roberta::Precision;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Config {
    pub hidden_size: usize,
    pub intermediate_size: usize,
    pub num_attention_heads: usize,
    pub num_hidden_layers: usize,
    pub vocab_size: usize,
    pub hidden_activation: Activation,
    pub norm_eps: f64,
    pub global_attn_every_n_layers: usize,
    pub global_rope_theta: f64,
    pub local_attention: usize,
    pub local_rope_theta: f64,
    #[serde(default)]
    pub attention_bias: bool,
    #[serde(default)]
    pub mlp_bias: bool,
    #[serde(default)]
    pub norm_bias: bool,
}

fn msg(s: String) -> candle_core::Error {
    candle_core::Error::Msg(s)
}

/// Embedding table read on demand from the mapped file.
struct LazyTable {
    file: Arc<MmapedSafetensors>,
    name: String,
    hidden: usize,
    vocab: usize,
    device: Device,
}

impl LazyTable {
    fn forward(&self, ids: &[u32]) -> Result<Tensor> {
        if let Some(bad) = ids.iter().find(|&&i| i as usize >= self.vocab) {
            return Err(msg(format!("token id {bad} outside the table ({})", self.vocab)));
        }
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
                "BF16" => out.extend(row.chunks_exact(2).map(|c| half::bf16::from_bits(u16::from_le_bytes([c[0], c[1]])).to_f32())),
                "F16" => out.extend(row.chunks_exact(2).map(|c| half::f16::from_bits(u16::from_le_bytes([c[0], c[1]])).to_f32())),
                _ => out.extend(row.chunks_exact(4).map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))),
            }
        }
        Tensor::from_vec(out, (1, ids.len(), self.hidden), &self.device)
    }
}

enum Dense {
    Full(candle_nn::Linear),
    Quant { weight: QMatMul, bias: Option<Tensor> },
}

impl Dense {
    fn new(in_dim: usize, out_dim: usize, bias: bool, vb: VarBuilder, precision: Precision) -> Result<Self> {
        match precision {
            Precision::F32 => Ok(Dense::Full(if bias { candle_nn::linear(in_dim, out_dim, vb)? } else { candle_nn::linear_no_bias(in_dim, out_dim, vb)? })),
            Precision::Q8 => {
                let w = vb.get((out_dim, in_dim), "weight")?;
                let bias = if bias { vb.get(out_dim, "bias").ok() } else { None };
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

fn norm(size: usize, eps: f64, bias: bool, vb: VarBuilder) -> Result<LayerNorm> {
    if bias {
        candle_nn::layer_norm(size, eps, vb)
    } else {
        candle_nn::layer_norm_no_bias(size, eps, vb)
    }
}

struct Attention {
    heads: usize,
    head_dim: usize,
    wqkv: Dense,
    wo: Dense,
    theta: f64,
    window: Option<usize>,
}

impl Attention {
    fn forward(&self, x: &Tensor, mask: &Tensor) -> Result<Tensor> {
        let (b, n, hidden) = x.dims3()?;
        let qkv = self.wqkv.forward(x)?.reshape((b, n, 3, self.heads, self.head_dim))?;
        let q = qkv.narrow(2, 0, 1)?.squeeze(2)?.transpose(1, 2)?.contiguous()?;
        let k = qkv.narrow(2, 1, 1)?.squeeze(2)?.transpose(1, 2)?.contiguous()?;
        let v = qkv.narrow(2, 2, 1)?.squeeze(2)?.transpose(1, 2)?.contiguous()?;
        let (cos, sin) = rotary(n, self.head_dim, self.theta, x.device())?;
        let q = apply_rotary(&q, &cos, &sin)?;
        let k = apply_rotary(&k, &cos, &sin)?;
        let scores = (q.matmul(&k.transpose(2, 3)?)? * (1f64 / (self.head_dim as f64).sqrt()))?;
        let scores = scores.broadcast_add(mask)?;
        let probs = softmax_last_dim(&scores)?;
        let context = probs.matmul(&v)?.transpose(1, 2)?.contiguous()?.reshape((b, n, hidden))?;
        self.wo.forward(&context)
    }
}

/// Cosine and sine tables of the rotary embedding, `[1, 1, n, head_dim]`.
fn rotary(n: usize, dim: usize, theta: f64, device: &Device) -> Result<(Tensor, Tensor)> {
    let half = dim / 2;
    let mut cos = Vec::with_capacity(n * dim);
    let mut sin = Vec::with_capacity(n * dim);
    for pos in 0..n {
        let mut angles = Vec::with_capacity(dim);
        for i in 0..half {
            let inv = 1f32 / theta.powf(2.0 * i as f64 / dim as f64) as f32;
            angles.push(pos as f32 * inv);
        }
        let all: Vec<f32> = angles.iter().chain(angles.iter()).copied().collect();
        cos.extend(all.iter().map(|a| a.cos()));
        sin.extend(all.iter().map(|a| a.sin()));
    }
    Ok((Tensor::from_vec(cos, (1, 1, n, dim), device)?, Tensor::from_vec(sin, (1, 1, n, dim), device)?))
}

/// `x * cos + rotate_half(x) * sin`, with `rotate_half(x) = [-x2, x1]`.
fn apply_rotary(x: &Tensor, cos: &Tensor, sin: &Tensor) -> Result<Tensor> {
    let dim = x.dim(D::Minus1)?;
    let half = dim / 2;
    let x1 = x.narrow(D::Minus1, 0, half)?;
    let x2 = x.narrow(D::Minus1, half, half)?;
    let rotated = Tensor::cat(&[&x2.neg()?, &x1], D::Minus1)?;
    x.broadcast_mul(cos)?.add(&rotated.broadcast_mul(sin)?)
}

struct Mlp {
    wi: Dense,
    wo: Dense,
    act: Activation,
}

impl Mlp {
    fn forward(&self, x: &Tensor) -> Result<Tensor> {
        let both = self.wi.forward(x)?;
        let width = both.dim(D::Minus1)? / 2;
        let input = both.narrow(D::Minus1, 0, width)?;
        let gate = both.narrow(D::Minus1, width, width)?;
        self.wo.forward(&(self.act.forward(&input)? * gate)?)
    }
}

struct Layer {
    attn_norm: Option<LayerNorm>,
    attn: Attention,
    mlp_norm: LayerNorm,
    mlp: Mlp,
}

pub struct ModernBertModel {
    embeddings: LazyTable,
    embed_norm: LayerNorm,
    layers: Vec<Layer>,
    final_norm: LayerNorm,
}

impl ModernBertModel {
    /// `weights` is the safetensors file itself, for the lazily read embedding table.
    pub fn new(cfg: &Config, vb: VarBuilder, weights: &Path, precision: Precision, device: &Device) -> Result<Self> {
        // SAFETY: the model file is read-only for the lifetime of the process.
        let file = Arc::new(unsafe { MmapedSafetensors::new(weights)? });
        let head_dim = cfg.hidden_size / cfg.num_attention_heads;
        let mut layers = Vec::with_capacity(cfg.num_hidden_layers);
        for i in 0..cfg.num_hidden_layers {
            let vl = vb.pp(format!("layers.{i}"));
            let global = i % cfg.global_attn_every_n_layers == 0;
            layers.push(Layer {
                attn_norm: if i == 0 { None } else { Some(norm(cfg.hidden_size, cfg.norm_eps, cfg.norm_bias, vl.pp("attn_norm"))?) },
                attn: Attention {
                    heads: cfg.num_attention_heads,
                    head_dim,
                    wqkv: Dense::new(cfg.hidden_size, 3 * cfg.hidden_size, cfg.attention_bias, vl.pp("attn.Wqkv"), precision)?,
                    wo: Dense::new(cfg.hidden_size, cfg.hidden_size, cfg.attention_bias, vl.pp("attn.Wo"), precision)?,
                    theta: if global { cfg.global_rope_theta } else { cfg.local_rope_theta },
                    window: if global { None } else { Some(cfg.local_attention / 2) },
                },
                mlp_norm: norm(cfg.hidden_size, cfg.norm_eps, cfg.norm_bias, vl.pp("mlp_norm"))?,
                mlp: Mlp {
                    wi: Dense::new(cfg.hidden_size, 2 * cfg.intermediate_size, cfg.mlp_bias, vl.pp("mlp.Wi"), precision)?,
                    wo: Dense::new(cfg.intermediate_size, cfg.hidden_size, cfg.mlp_bias, vl.pp("mlp.Wo"), precision)?,
                    act: cfg.hidden_activation,
                },
            });
        }
        Ok(Self {
            embeddings: LazyTable {
                file,
                name: {
                    let prefix = vb.prefix();
                    if prefix.is_empty() {
                        "embeddings.tok_embeddings.weight".to_string()
                    } else {
                        format!("{prefix}.embeddings.tok_embeddings.weight")
                    }
                },
                hidden: cfg.hidden_size,
                vocab: cfg.vocab_size,
                device: device.clone(),
            },
            embed_norm: norm(cfg.hidden_size, cfg.norm_eps, cfg.norm_bias, vb.pp("embeddings.norm"))?,
            layers,
            final_norm: norm(cfg.hidden_size, cfg.norm_eps, cfg.norm_bias, vb.pp("final_norm"))?,
        })
    }

    /// One sequence, no padding: `input_ids` is `[1, n]`.
    pub fn forward(&self, input_ids: &Tensor) -> Result<Tensor> {
        let ids: Vec<u32> = input_ids.flatten_all()?.to_vec1()?;
        let n = ids.len();
        let device = input_ids.device();
        let mut hidden = self.embed_norm.forward(&self.embeddings.forward(&ids)?)?;
        let full = Tensor::zeros((1, 1, n, n), DType::F32, device)?;
        let mut local: Option<Tensor> = None;
        for layer in &self.layers {
            let mask = match layer.attn.window {
                None => &full,
                Some(w) => {
                    if local.is_none() {
                        local = Some(window_mask(n, w, device)?);
                    }
                    local.as_ref().unwrap()
                }
            };
            let normed = match &layer.attn_norm {
                Some(ln) => ln.forward(&hidden)?,
                None => hidden.clone(),
            };
            hidden = (&hidden + layer.attn.forward(&normed, mask)?)?;
            hidden = (&hidden + layer.mlp.forward(&layer.mlp_norm.forward(&hidden)?)?)?;
        }
        self.final_norm.forward(&hidden)
    }
}

/// Additive mask of the sliding window: zero within `w` positions, minus infinity
/// beyond, `[1, 1, n, n]`.
fn window_mask(n: usize, w: usize, device: &Device) -> Result<Tensor> {
    let mut data = vec![0f32; n * n];
    for i in 0..n {
        for j in 0..n {
            if i.abs_diff(j) > w {
                data[i * n + j] = f32::MIN;
            }
        }
    }
    Tensor::from_vec(data, (1, 1, n, n), device)
}
