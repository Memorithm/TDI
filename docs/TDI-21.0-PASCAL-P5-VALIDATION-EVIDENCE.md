# TDI-21 Pascal P5 Validation evidence

Status: completed preregistered Validation-only realized-cost slice. This is non-final evidence; no protected/final population was opened, generated, or executed.

## Frozen execution identity

- Execution workspace HEAD at launch: `c3bd0b1248a4196e4076de26b3a50cebcf2794bc`.
- Remote Validation preflight commit: `f879bbf743e40925e63f689c025d9dd837ba0570`.
- The local Development evidence commit `c3bd0b1248a4196e4076de26b3a50cebcf2794bc` and the remote alias `83443a1904d333a424e96abe081c73a6cd8ae0ec` have the identical Git tree `f4dd48e572060777b3972c9a29d32dadb9a40e4e`. The different commit IDs are retained rather than collapsed.
- Validation binary SHA-256: `b5b3376e4635ec3e7e640cf4c2c7bdc20cc6cd11545b36f5175c60ff8b88649b`.
- Validation case-plan SHA-256: `3d36b2c852f8ac6f4df097474e55430d06dde8eb65efa73b7e89774e5c3c1f06`.
- Remote preflight-freeze SHA-256: `c7d703e55dfeefbb200934ee12d612d78714c27188b9aa024a4fae141d6ef67d`.
- Execution-side freeze SHA-256: `b2da44227bfc854cac21dc6f2f0a2219741e302961fcee188d7e0228751fdf64`.
- Raw Validation TSV SHA-256: `c7e08d317606897d8c3ec7f3cab4bf5e88e264a6213f254a04c21c7adb104a3d`.
- Captured stderr is empty; SHA-256 `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`.

The preflight-freeze files differ only in which byte-identical Development commit alias is named. Both are retained separately. No raw timing row was changed.

## Exact Validation result

The frozen grid contains 54 cells at `n=15`. Each cell retains six timing rounds for each of the Pascal, direct ANF, and generic full-materialization arms: 972 raw timing measurements in total.

Semantic controls are exact: 0 Pascal mismatches, 0 generic mismatches, and 0 pairwise-token comparisons.

Against `DIRECT_ANF_QUERY`, `delta_direct = direct - Pascal` is positive in 6 cells, zero in 0, and negative in 48. The median signed delta is -166875.5 ns.

Against `GENERIC_FULL_MATERIALIZE`, Pascal is faster in 54/54 cells, with median signed delta +2008360.5 ns.

There are 24/54 sign disagreements between the P4 direct-cost proxy and P5 realized direct timing. Those disagreements are retained as evidence and are not filtered or reclassified.

## Interpretation boundary

This slice validates only the preregistered non-final P5 timing grid on the frozen host/toolchain. It does not establish confirmatory model utility and does not authorize promotion into SBG, MOR, Delta-KV, context memory, KV cache, or model architecture. SML-GENIUS remains the owner of model-side Pascal primitives.