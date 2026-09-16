# TDI-22.1A — T1 Value-Reconstruction Evidence Amendment

Status: **pre-execution amendment candidate; no TDI-22.2 campaign result has been observed or persisted**.

## 1. Purpose

The frozen TDI-22.1 preregistration requires T1 to report exact/declared-tolerance reconstruction of the evaluator-owned target value `(R,C)`. The frozen base schema `tdi22-eval-record-v1` records selected/target identities and ranking scores, but it does not retain the selected and target `(R,C)` components. Because two distinct candidate identities can in principle carry equal `(R,C)` values, value reconstruction is not logically equivalent to identity retrieval.

This amendment closes that evidence-retention gap **before bounded Development/Validation campaign execution**. It does not alter the 25-field `tdi22-eval-record-v1` schema, any task population, any seed stream, any arm score, any target rule, or any decision threshold.

## 2. Companion schema

Every T1 base record must have exactly one companion line with schema version:

`tdi22-t1-value-record-v1`

Each line is UTF-8, terminated by exactly one LF byte, with `name=value` pairs separated by exactly one ASCII TAB. Field order is exactly:

1. `schema_version`
2. `split`
3. `cell`
4. `episode_index`
5. `query_index`
6. `target_identity`
7. `selected_identity`
8. `reconstruction_success`
9. `target_r_x_bits`
10. `target_r_y_bits`
11. `target_r_z_bits`
12. `target_c_x_bits`
13. `target_c_y_bits`
14. `target_c_z_bits`
15. `selected_r_x_bits`
16. `selected_r_y_bits`
17. `selected_r_z_bits`
18. `selected_c_x_bits`
19. `selected_c_y_bits`
20. `selected_c_z_bits`
21. `rejection_reason`

Integer, enum, boolean, binary64 and `none` encodings are identical to `docs/TDI-22.1-RECORD-CONTRACT.md`.

## 3. Reconstruction rule

The target value is the evaluator-owned target candidate's `(R,C)`. The selected value is the one-hot T1 readout candidate's `(R,C)`.

For each of the six components, use the already frozen tolerance

`tol(a,b) = 512 * EPSILON * max(1, abs(a), abs(b))`.

`reconstruction_success=1` iff all six selected components satisfy `abs(selected-target) <= tol(selected,target)`. Otherwise it is `0`.

This metric is deliberately independent of identity equality. A different identity carrying the same `(R,C)` may therefore reconstruct successfully while the base retrieval record remains an identity miss.

## 4. Rejection behavior

If T1 is rejected as `AmbiguousT1Top`, `selected_identity` and all six selected-value fields are `none`, `reconstruction_success=0`, and `rejection_reason=ambiguous_t1_top`. Target identity/value fields remain present when the evaluator target is known.

For any future pre-selection failure where the target is unavailable, target identity/value fields are `none`. No rejected record is dropped.

## 5. Pairing and auditability

The tuple `(split, cell, episode_index, query_index)` is the unique join key to the corresponding T1 `tdi22-eval-record-v1` line. Exactly one companion line must exist for every T1 base record and no companion line may exist for T0, T3 or T4.

The companion line is evidence only; its construction is evaluator-side work and does not alter T1 resource accounting.

## 6. Scientific boundary

This amendment is not a result, does not change T1 scoring, does not add T2, does not authorize final/confirmatory execution, does not alter FLAT-ATTENTION, and creates no model-quality or hardware claim. It must be merged and integrity-frozen before any TDI-22.2 Development/Validation campaign evidence is persisted.
