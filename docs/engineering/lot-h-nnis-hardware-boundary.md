# Lot H — TDI → NNIS hardware-qualification boundary

## Status

**Current-main requalification candidate / contract only.** The #286 boundary is present on `main`; this refresh re-runs its dedicated exact-head contract gate on the integrated implementation. Even a green TDI contract gate does not qualify NNIS execution, CUDA-Rust SIMT, a GPU device, serving performance, production routing, or any scientific result. The upstream manifest state described below remains authoritative and fail-closed.

The audited NNIS source for this slice is `5436736002834dd6dd7d5ace8c1c044b47aed18f`. Its `docs/cuda-rust-simt-qualification.json` is currently `unresolved_blocking`; the TDI contract preserves that state exactly rather than converting it into success.

## Ownership

- **NNIS owns** runtime/kernel semantics, device/toolchain qualification, correction oracles, environment-compatible performance evidence, real-model end-to-end requalification, and final runtime promotion.
- **TDI owns** scientific identities, admissible evidence references, split/holdout boundaries, scientific stage authorization and verdict semantics.
- **scirust-hub owns** orchestration, component admission, leases/fencing, transport, physical artifacts and authoritative publication.

TDI must not reimplement NNIS CUDA execution, infer hardware from a hostname, select a kernel, or promote a runtime path.

## Audited NNIS source surface

The contract pins:

- repository: `Memorithm/NNIS`;
- source: `5436736002834dd6dd7d5ace8c1c044b47aed18f`;
- NNIS Rust MSRV: `1.77`;
- manifest: `docs/cuda-rust-simt-qualification.json`, Git blob `a777fa47f8fe4af068ef6a14c808d91e95161f58`;
- manifest schema: `nnis-cuda-rust-simt-qualification-v1`;
- manifest state at the audited source: `unresolved_blocking`;
- frontend contract: `CUDA_RUST_SIMT_PTX`;
- validator: `scripts/validate_cuda_rust_simt_qualification.py`, Git blob `ef7af3f8cc840b6f3a8053e8e5b33bf38d29f143`;
- evidence schema expected by that validator: `nnis-cuda-rust-simt-evidence-v1`.

The TDI adapter schema identities are deterministically derived from these exact coordinates. They are TDI interchange identities, not NNIS-published digests.

## NNIS upstream fail-closed boundary

The audited NNIS validator requires, before `qualified_nonproduction` can be accepted:

- exact toolchain source commit;
- exact PTX SHA-256;
- exact device/CUDA identity;
- independent vector-add correction oracle;
- negative fail-closed tests;
- a content-addressed evidence bundle;
- exact NNIS revision binding.

It also requires `production_routing_authorized = false` and `performance_claim_authorized = false`. Successful manifest validation still does not promote the frontend.

Because the audited manifest is presently `unresolved_blocking`, `compile_nnis_qualification_request()` emits `upstream_qualification_resolved = false`. A later qualified NNIS source must be audited and source-pinned explicitly; TDI does not reinterpret or mutate the current upstream state.

## TDI contract

`NnisQualificationContract/v1` binds:

1. an exact common `PartnerAdapter/v1` for `Memorithm/NNIS`;
2. an exact G3-admitted, non-executing `tdi.prepare` Hub step;
3. the audited NNIS source/manifest/validator tuple above;
4. one Development or Validation candidate artifact;
5. one bounded review scope: `cuda-rust-simt-contract` or `cuda-rust-simt-evidence-review`;
6. mandatory independent NNIS validation, exact NNIS revision binding, exact device/CUDA identity, correction-before-performance and real-device evidence for any device claim.

All authority flags are fixed to false, including NNIS execution qualification, hardware-qualification authorization, device-performance qualification, performance claims, production routing, protected-holdout access, scientific stage/verdict authority and runtime actuation.

## Non-claims

This slice provides no measured latency, throughput, energy, memory, TTFT/TPOT or kernel speedup. It is not a CUDA correction result and does not qualify the current unresolved NNIS manifest. It does not infer GPU model, compute capability, driver, CUDA version, power mode or topology. It does not turn synthetic or vector-add correction evidence into real-model serving performance.

Any later performance claim remains subject to NNIS's own exact-head, same-model/representation/workload/device and environment-fingerprint requirements and to the applicable NNIS agent roadmaps.
