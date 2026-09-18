# Public attention operator probe

This optional process adapter calls exact pinned FLAT-ATTENTION and NNIS APIs.
It accepts only small f32 arrays; it has no model, tokenizer, data population,
checkpoint, learned routing or final-stage execution interface. It does not
change the older non-executing partner qualification envelopes.

The separate Cargo workspace requires Rust 1.89. TDI's main Rust workspace and
NNIS's upstream Rust 1.77 policy are unchanged. `Cargo.lock` and exact Git revisions
bind dependencies; an executable hash plus a declared source is not a build
attestation.

```sh
cargo build --locked --manifest-path integrations/attention-probe/Cargo.toml
PYTHONPATH=scripts python3 -c 'import json; from tdi_attention_fixture import fixture_request; print(json.dumps(fixture_request(0)))' > public-attention.json
integrations/attention-probe/target/debug/tdi-attention-probe validate < public-attention.json
integrations/attention-probe/target/debug/tdi-attention-probe run < public-attention.json
```

`validate` never discovers a device or invokes a numerical backend. `run` uses
the explicit request backend; unknown/duplicate fields, nonfinite numbers, mixed
layout/dtype, unsupported semantics or exceeded bounds fail with exit 21 and a
versioned JSON rejection. Input is at most 1 MiB. The TDI process client adds wall,
input/output and child-lifetime bounds; invoking the binary directly does not add
a watchdog around a hung GPU driver. Arrays contain at most 2 batches, 4 query/KV
heads, 32 query/KV rows and 32 features; values satisfy `abs(x)<=32`, with explicit
scale in `[0.0001,16]`. Q/O are `[B,Hq,Nq,D]`, K/V `[B,Hkv,Nkv,D]`, LSE `[B,Hq,Nq]`.
Query heads map contiguously to KV groups. All arrays are dense row-major.

| Property | FLAT reference | Optional NNIS fused |
| --- | --- | --- |
| Mechanism | Standard scaled softmax | Standard scaled softmax |
| Storage/accumulation | f32; scalar multiply/add in dimension order | f32; explicit FMA chain, block reductions and CUDA `expf` |
| Mask | none or causal with absolute query offset | none, or causal square lengths with zero offset |
| Grouping | MHA/GQA/MQA through the real asymmetric API | explicit head loop reuses each K/V allocation across its query group |
| Result | O and LSE | O; LSE explicitly unavailable |
| Device | host CPU, no GPU claim | exactly one visible CUDA device; UUID/driver/NVRTC identity required |
| Fallback | no other backend | errors never select a CPU or composed path |
| Checkpoint/backward | unsupported | unsupported |
| Timing/memory | not measured inside the kernel adapter | not measured inside the kernel adapter |

Boolean admission, Boolean similarity, recurrent, structured and hybrid mechanisms
are distinct semantics and are rejected by this adapter. They are not implicitly
declared equivalent to softmax. Cross-backend bitwise reproducibility is not
claimed. The public software verifier uses a separate two-pass f64 oracle and
predeclared tolerances: FLAT abs/rel `3e-6`; NNIS abs `1e-3`, rel `1e-4` (maximum
of the absolute and relative bounds). These tiny controls cannot establish model
quality, serving throughput, memory savings or a new scientific semantic.

## Catalogue and independent verification

After starting the authenticated local Hub as described in
[the operational guide](../../docs/engineering/operational-engine.md):

```sh
python3 scripts/tdi_engine.py --hub http://127.0.0.1:8477 --allow-loopback-http \
  attention-fixture-plan --worker integrations/attention-probe/target/debug/tdi-attention-probe \
  --trials 6 --output attention-plan.json
python3 scripts/tdi_engine.py --hub http://127.0.0.1:8477 --allow-loopback-http submit attention-plan.json
# Use the campaign ID returned above:
python3 scripts/tdi_engine.py --hub http://127.0.0.1:8477 --allow-loopback-http run CAMPAIGN_ID
```

The six actual fixtures cover MHA/GQA/MQA, two batches, rectangular lengths,
causal prefixes and FLAT's offset single-query contract. Each run publishes its
observation before a separate Hub verification stage reads it. A failure retains
its exit status and process costs in the run artifact while verification fails.
No cache reuse is allowed. Restart, bundle export/verification and consultation
use the existing engine commands. New plan/output files are required; commands
refuse accidental overwrite.

## Optional physical NNIS path

Compile and lint the optional path separately:

```sh
cargo build --locked --manifest-path integrations/attention-probe/Cargo.toml --features nnis-cuda
cargo clippy --locked --manifest-path integrations/attention-probe/Cargo.toml --all-targets --all-features -- -D warnings
```

Execution needs an exact software-qualified build, one visible CUDA device, a
compatible CUDA Driver API and NVRTC installation discoverable by NNIS, sufficient
shared memory, a trusted local Hub and an operator-authorized non-final software
profile. The adapter validates shape/mask before device access. No library paths
or GPU selection are silently rewritten. On the eligible device use a new plan:

```sh
python3 scripts/tdi_engine.py --hub http://127.0.0.1:8477 --allow-loopback-http \
  attention-fixture-plan --worker integrations/attention-probe/target/debug/tdi-attention-probe \
  --backend nnis-cuda-fused --allow-cuda --trials 6 --output nnis-attention-plan.json
```

Then explicitly submit/run as above. NNIS case 3 is square causal; it is not the
FLAT offset contract. The process timeout is 30 seconds including initialization
and JIT; a timeout is failed evidence. The resulting wall/CPU/wait4 RSS metrics
include the complete process and are not CUDA-event kernel latency or VRAM.
Physical CUDA execution and parity were **not qualified** in the current CPU
environment. Compiling the optional path is not hardware evidence.

NNIS's existing NVRTC attention path is separate from its unresolved
`CUDA_RUST_SIMT_PTX` frontend qualification. Review the latter using the exact
upstream validator without a GPU:

```sh
python3 scripts/tdi_engine.py nnis-qualification-review /path/to/exact-nnis-checkout
```

The checkout must be at `5436736002834dd6dd7d5ace8c1c044b47aed18f`; the two known
manifest/validator Git blobs must match exactly. Other checkout files are not
inspected. The actual NNIS validator currently accepts the structurally valid
`unresolved_blocking` manifest and says qualification remains unresolved. TDI
retains that status and all execution/promotion authority stays false. Future
qualification requires separately audited source/evidence pins, not a Boolean
override in this adapter.

The integration suite exercises actual FLAT, actual NNIS-owned validation, the
real authenticated Hub and independent output checking. It rejects altered
outputs, future causal influence, unsupported NNIS causal offsets, missing CUDA
opt-in, malformed input and changed source blobs. Synthetic presentation or
malformed-input cases are never used as device-performance evidence.
