# TDI-9.3.3 — Exact local sensitivity of C3 policies

This is a non-final representation audit, not a trajectory experiment, model robustness result, TDI-9.1 observation freeze or TDI-9.2 authorization.

## Contract

`c3_policy_sensitivity(policy)` admits the policy to the existing reference C3 synthesis envelope, enumerates the existing validated nine-predicate carrier, and measures action changes on a precisely defined local graph. It does not mutate, select or promote a policy.

For each of the 120 action-bearing rows, propose five single-bit changes (BASE_STOP, VERIFY_BEFORE_STOP, CADENCE_DUE, CHECKPOINT_AVAILABLE, REMAINING_WORK) and three substitutions of the one-hot verifier state. A categorical substitution changes two encoding bits and must not be described as single-bit noise. The same validated carrier is used for sources and destinations.

Expected exact accounting, independently enumerated before the Rust regression run: 960 proposed directed changes, 920 admitted, 40 rejected. The initial carrier still distinguishes 384 invalid encodings and eight unrecoverable violations. Rejected neighbours are not counted as stable/unchanged actions.

The full hand-written reference has the following regression targets:

| Axis | Admitted | Action changed | Rejected |
| --- | ---: | ---: | ---: |
| BASE_STOP | 120 | 24 | 0 |
| VERIFY_BEFORE_STOP | 120 | 16 | 0 |
| CADENCE_DUE | 120 | 16 | 0 |
| CHECKPOINT_AVAILABLE | 112 | 16 | 8 |
| REMAINING_WORK | 112 | 0 | 8 |
| Verifier-state substitution | 336 | 284 | 24 |

The complete directed action-transition matrix is reported in CONTINUE, VERIFY, BACKTRACK, STOP order. Reverse edges exist for every admitted pair, so the matrix must be symmetric. This is a graph-calibration invariant, not an assertion that real solver transitions are reversible or that these combinations are reachable.

## Stability is not quality

The report also counts disagreement with the existing independent hand-written action oracle over the 120 valid rows. The constant-CONTINUE negative control has zero action changes but disagrees on 72 rows. Fewer action changes must never be presented as an improvement without the separate correctness/task-quality constraint.

The example reports the 27 existing single-rule baselines, full C3 reference, and constant-CONTINUE negative control. It is fixed, accepts no external data/arguments, and emits every axis and all matrix rows, including zeros. No elapsed-time or controller-cost conclusion is derived from graph counts.

## Reproduce

```bash
cargo test --locked -p tdi-ai --features experimental --lib boolean_policy_sensitivity
cargo run --locked -p tdi-ai --features experimental --example boolean_c3_sensitivity
bash scripts/check-tdi9.3-representation-calibration.sh
```

The existing experimental-contract gate runs the four regression tests and compares two report invocations byte-for-byte. Existing calibration, carrier, bootstrap and freeze-integrity checks remain enabled.

## Relationship to BooleanLab

BL-14 distinguishes a truth-table function, the topology it induces, and the resulting numerical quality. TDI-9.3.3 makes the corresponding distinction for action policies: behaviour sensitivity alone is neither correctness nor useful adaptive inference. The shared lesson is an evaluation contract, not a copied production runtime. TDI-specific action order, carrier constraints, observation lineage and protected/final boundaries remain here.
