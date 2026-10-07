# TDI-21 Pascal P5 Validation evidence

Status: completed preregistered Validation-only realized-cost slice; the Development-vs-Validation timing comparison is inconclusive under G7 (Validation CPU affinity not recorded, see below). This is non-final evidence; no protected/final population was opened, generated, or executed.

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

## G7 timing-procedure gap: Development-vs-Validation timing comparison is inconclusive

The Development freeze (`results/tdi21_pascal_p5/freeze.json`) pins `cpu_affinity` to `cpu0`, and the Validation runner amendment requires the Validation timing procedure to match Development. No committed artifact records the CPU affinity (or the pinning invocation) used for the Validation execution: the execution-side freeze records host, kernel, governor, toolchain and binary identities, but not affinity, and no such record exists elsewhere. No affinity value is inferred or added here.

Gate G7 (unchanged timing procedure between Development and Validation) therefore cannot be established. Under the preregistration rule that a G7 failure makes the affected evidence inconclusive, the **Development-vs-Validation timing comparison is classified as inconclusive** because the Validation CPU affinity was not recorded. `validation-summary.json` records this as `gates.g7_validation_timing_procedure_matches_development = false` and `development_validation_timing_comparison.classification = "inconclusive"`.

Unaffected: the semantic gates (exact Pascal/generic outputs, zero pairwise-token operations) and the completeness of the raw record (all six rounds per arm, every signed contrast retained). The raw Validation timing rows above are retained unchanged as measured on this host, but they are not certified as procedure-matched to Development and support no cross-split timing claim.

## Interpretation boundary

This slice validates only the preregistered non-final P5 timing grid on the frozen host/toolchain. It does not establish confirmatory model utility and does not authorize promotion into SBG, MOR, Delta-KV, context memory, KV cache, or model architecture. SML-GENIUS remains the owner of model-side Pascal primitives.
