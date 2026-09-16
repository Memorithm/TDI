//! Bounded software probes for exact pinned attention APIs, without a model.
//!
//! This is a development/validation operator adapter. It cannot load weights,
//! prompts, checkpoints or a scientific population. Non-softmax mechanisms,
//! Boolean routing, dropout, padding, bias and checkpoint continuation are
//! unsupported, not silently mapped to dense softmax.

use flat_attention::api::v1::{AttentionConfig, AttentionShape, BorrowedAttentionRequest};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

/// Exact upstream source coordinates; these are declarations, not build attestations.
pub const FLAT_SOURCE: &str = "1d5ac64cc87c5dd526e04527c3bb4b78ba0add33";
pub const NNIS_SOURCE: &str = "5436736002834dd6dd7d5ace8c1c044b47aed18f";
/// The process parser refuses a request above this byte budget.
pub const MAX_BYTES: usize = 1024 * 1024;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Domain {
    Development,
    Validation,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub enum Backend {
    #[serde(rename = "flat-reference")]
    FlatReference,
    #[serde(rename = "nnis-cuda-fused")]
    NnisCudaFused,
}

/// Dense row-major BHND dimensions. K/V have physical `kv_heads`, not expanded heads.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Shape {
    pub batch: usize,
    pub q_heads: usize,
    pub kv_heads: usize,
    pub query_len: usize,
    pub kv_len: usize,
    pub head_dim: usize,
    pub query_position_offset: usize,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub schema: u8,
    pub purpose: String,
    pub domain: Domain,
    pub backend: Backend,
    pub mechanism: String,
    pub dtype: String,
    pub layout: String,
    pub mask: String,
    pub shape: Shape,
    pub scale: f32,
    pub q: Vec<f32>,
    pub k: Vec<f32>,
    pub v: Vec<f32>,
}

impl Request {
    fn api_shape(&self) -> AttentionShape {
        let s = &self.shape;
        AttentionShape {
            batch: s.batch,
            q_heads: s.q_heads,
            kv_heads: s.kv_heads,
            query_len: s.query_len,
            kv_len: s.kv_len,
            head_dim: s.head_dim,
            query_position_offset: s.query_position_offset,
        }
    }

    /// Validate before any library execution, device discovery or allocation.
    ///
    /// Bounds: batch <=2, heads <=4, lengths/head dimension <=32, coordinate
    /// offset <=32, finite inputs |x|<=32 and explicit scale in [0.0001,16].
    /// NNIS's current causal API is square and zero-offset only.
    pub fn validate(&self) -> Result<(), String> {
        if self.schema != 1
            || self.purpose != "public-kernel-probe"
            || self.mechanism != "standard-softmax"
            || self.dtype != "f32"
            || self.layout != "BHND"
            || !["none", "causal"].contains(&self.mask.as_str())
        {
            return Err("unsupported public operator semantics".into());
        }
        let s = &self.shape;
        if s.batch > 2
            || s.q_heads > 4
            || s.kv_heads > 4
            || s.query_len > 32
            || s.kv_len > 32
            || s.head_dim > 32
            || s.query_position_offset > 32
            || !self.scale.is_finite()
            || !(0.0001..=16.0).contains(&self.scale)
        {
            return Err("public probe shape/scale budget exceeded".into());
        }
        if self.backend == Backend::NnisCudaFused
            && self.mask == "causal"
            && (s.query_len != s.kv_len || s.query_position_offset != 0)
        {
            return Err(
                "NNIS fused causal API requires square lengths and zero query offset".into(),
            );
        }
        BorrowedAttentionRequest {
            shape: self.api_shape(),
            config: AttentionConfig {
                causal: self.mask == "causal",
                softmax_scale: Some(self.scale),
            },
            q: &self.q,
            k: &self.k,
            v: &self.v,
        }
        .validate()
        .map_err(|e| e.to_string())?;
        if self
            .q
            .iter()
            .chain(&self.k)
            .chain(&self.v)
            .any(|x| x.abs() > 32.0)
        {
            return Err("public probe value budget exceeded".into());
        }
        Ok(())
    }

    /// Describe the actual semantic/backend contract without executing it.
    /// All promotion and scientific authority remains false.
    pub fn contract(&self) -> Result<Value, String> {
        self.validate()?;
        Ok(
            json!({"schema":1,"protocol":"tdi-public-attention-probe/v1","domain":self.domain,
            "backend":self.backend,"mechanism":self.mechanism,"dtype":self.dtype,"layout":self.layout,
            "shape":self.shape,"mask":self.mask,"scale":self.scale,
            "source_commit":if self.backend == Backend::FlatReference { FLAT_SOURCE } else { NNIS_SOURCE },
            "accumulation":if self.backend == Backend::FlatReference {"f32 scalar increasing-dimension multiply/add; online softmax"} else {"f32 explicit FMA chain; block-reduced online softmax; CUDA expf"},
            "query_order":"increasing absolute query position; key position starts at zero",
            "reproducibility":"same implementation/environment only; cross-backend bit identity not asserted",
            "checkpoint":"unsupported","model_execution":false,"scientific_verdict":"not-assessed",
            "performance_claim_authorized":false,"production_routing_authorized":false,
            "nnis_cuda_rust_simt_frontend":"separate unresolved qualification; this optional path uses existing NNIS NVRTC kernels"}),
        )
    }

    /// Execute the declared public operator. CUDA requires both the build
    /// feature and explicit caller opt-in; failures never fall back to CPU.
    ///
    /// Returns O, optional FLAT LSE and available device identity, with no
    /// timing/memory claim. The caller bounds process time and persists evidence.
    pub fn run(&self, allow_cuda: bool) -> Result<Value, String> {
        let contract = self.contract()?;
        let (output, lse, device) = match self.backend {
            Backend::FlatReference => {
                let result = flat_attention::forward_reference_grouped_asymmetric(
                    &self.q,
                    &self.k,
                    &self.v,
                    self.api_shape()
                        .to_core_shape()
                        .map_err(|e| e.to_string())?,
                    AttentionConfig {
                        causal: self.mask == "causal",
                        softmax_scale: Some(self.scale),
                    }
                    .to_core_config(),
                )
                .map_err(|e| e.to_string())?;
                (result.output, Some(result.lse), Value::Null)
            }
            Backend::NnisCudaFused => {
                if !allow_cuda {
                    return Err("CUDA execution requires explicit --allow-cuda".into());
                }
                nnis_run(self)?
            }
        };
        if output.len() != self.q.len()
            || output.iter().any(|x| !x.is_finite())
            || lse
                .as_ref()
                .is_some_and(|x| x.iter().any(|v| !v.is_finite()))
        {
            return Err("backend returned an invalid or nonfinite output".into());
        }
        Ok(
            json!({"schema":1,"status":"observed","contract":contract,"output":output,"lse":lse,
            "device":device,"timing":"not-measured","memory":"not-measured",
            "scientific_verdict":"not-assessed"}),
        )
    }
}

type Observed = (Vec<f32>, Option<Vec<f32>>, Value);

#[cfg(not(feature = "nnis-cuda"))]
fn nnis_run(_: &Request) -> Result<Observed, String> {
    Err("NNIS CUDA feature is not built; no CPU fallback".into())
}

#[cfg(feature = "nnis-cuda")]
fn nnis_run(r: &Request) -> Result<Observed, String> {
    use nnis_jit::JitCompiler;
    use nnis_kernels::{AttentionMask, F32Attention};
    use nnis_rt::{Context, Device, DeviceBuffer, Stream};
    let execute = || -> Result<Observed, Box<dyn std::error::Error>> {
        // An explicit single visible device prevents accidental selection of
        // an unrelated accelerator. No environment variable is rewritten here.
        if Device::count()? != 1 {
            return Err("exactly one visible CUDA device required".into());
        }
        let device = Device::first()?;
        let context = Context::new(&device)?;
        let props = context.props();
        let uuid = props.uuid.ok_or("CUDA device UUID unavailable")?;
        let driver = nnis_sys::driver::driver_version().ok_or("CUDA driver version unavailable")?;
        let nvrtc = nnis_sys::nvrtc::version().ok_or("NVRTC version unavailable")?;
        let metadata = json!({"name":props.name,"uuid":format!("{uuid:?}"),"compute_capability":props.compute_capability,
            "driver_version":driver,"nvrtc_version":nvrtc});
        let stream = Stream::new(&context)?;
        let attention = F32Attention::load(&context, &JitCompiler::new())?;
        let s = &r.shape;
        if !attention.fused_available(s.head_dim, s.head_dim) {
            return Err("NNIS fused attention unavailable for shape".into());
        }
        let mask = if r.mask == "causal" {
            AttentionMask::Causal
        } else {
            AttentionMask::None
        };
        let q_stride = s.query_len * s.head_dim;
        let kv_stride = s.kv_len * s.head_dim;
        let group = s.q_heads / s.kv_heads;
        let mut output = vec![0.0; r.q.len()];
        for b in 0..s.batch {
            for kv_head in 0..s.kv_heads {
                let kb = (b * s.kv_heads + kv_head) * kv_stride;
                // Reuse each physical K/V head for its query-head group.
                let k = DeviceBuffer::from_host(&context, &stream, &r.k[kb..kb + kv_stride])?;
                let v = DeviceBuffer::from_host(&context, &stream, &r.v[kb..kb + kv_stride])?;
                for h in kv_head * group..(kv_head + 1) * group {
                    let qb = (b * s.q_heads + h) * q_stride;
                    let q = DeviceBuffer::from_host(&context, &stream, &r.q[qb..qb + q_stride])?;
                    let o = DeviceBuffer::<f32>::new(&context, q_stride)?;
                    attention.attention_fused(
                        &stream,
                        &q,
                        &k,
                        &v,
                        &o,
                        s.query_len,
                        s.head_dim,
                        s.kv_len,
                        s.head_dim,
                        r.scale,
                        mask,
                    )?;
                    output[qb..qb + q_stride].copy_from_slice(&o.to_vec(&stream)?);
                }
            }
        }
        Ok((output, None, metadata))
    };
    execute().map_err(|e| e.to_string())
}

/// Strict typed decoding rejects unknown/duplicate fields and invalid floats.
/// It validates the complete contract before returning the request.
pub fn parse(raw: &[u8]) -> Result<Request, String> {
    if raw.len() > MAX_BYTES {
        return Err("request exceeds 1 MiB".into());
    }
    let request: Request = serde_json::from_slice(raw).map_err(|e| e.to_string())?;
    request.validate()?;
    Ok(request)
}
