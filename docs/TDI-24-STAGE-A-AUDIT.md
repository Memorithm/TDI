# TDI-24 Stage-A audit and freeze candidate

Status: **pre-merge candidate for Slice 10/50; non-confirmatory**.

This artifact audits the exact-semantics/control layer built by TDI-24 Slices 01-09. It does not evaluate a task population, compare model quality, use protected/final data, or establish a performance/novelty claim.

## Audited semantic surfaces

Stage A binds the following versioned contracts:

- `tdi24-mirror-coupled-chiral-v1` — C6 carrier plus `M`/`J` algebra;
- `tdi24-matched-vector6-v1` — independent V6 control carrier;
- `tdi24-channel-decomposition-v1` — unaggregated `s/m/chi` channels;
- `tdi24-enantiomorphic-score-pair-v1` — R/L score branches;
- `tdi24-parity-recombination-v1` — even/odd R/L recombination;
- `tdi24-masked-softmax-reference-v1` — deterministic masked normalizer;
- `tdi24-attention-mask-reference-v1` — full/causal mask semantics;
- `tdi24-reference-accounting-v1` — source-level operation/logical-storage accounting;
- aggregate identity `tdi24-stage-a-v1`.

## Audit checks

The Stage-A gate requires:

1. exact SHA-256 agreement for the scientific programme and Stage-A Rust surfaces;
2. Rust formatting and Clippy with warnings denied;
3. targeted tests for chiral algebra, V6 control, masking/normalization, accounting and aggregate Stage-A contracts;
4. exact version strings for every admitted semantic surface;
5. absence of `todo!` and `unimplemented!` in the Stage-A Rust modules;
6. no protected/final execution and no interpretation of this software audit as evidence that C6 is superior to V6.

## Promotion boundary

This candidate must be regenerated and rechecked after Slices 06-09 are individually merged and exact-head-qualified. Any semantic source change after the final Stage-A manifest is frozen requires an explicit versioned amendment; silently rewriting the frozen manifest is forbidden.

Stage A authorizes progression to task-population construction only after the final Slice-10 PR itself is qualified and merged. It does not authorize a confirmatory/final run.
