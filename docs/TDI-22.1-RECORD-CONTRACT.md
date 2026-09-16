# TDI-22.1 — Canonical Evaluator Record Encoding

Status: **part of the TDI-22.1 freeze; non-final only**.

Schema version is exactly `tdi22-eval-record-v1`.

Each evidence record is exactly one UTF-8 line terminated by one LF byte (`0x0a`), with no CR. The line consists of `name=value` pairs separated by exactly one ASCII TAB byte (`0x09`). No spaces are permitted around `=` or delimiters.

Field order is exactly:

1. `schema_version`
2. `split`
3. `cell`
4. `episode_index`
5. `query_index`
6. `arm`
7. `target_identity`
8. `selected_identity`
9. `exact_success`
10. `target_score_bits`
11. `runner_up_score_bits`
12. `target_margin_bits`
13. `rejection_reason`
14. `query_bits`
15. `key_bits`
16. `value_bits`
17. `position_bits`
18. `dynamic_state_bits`
19. `static_parameter_bits`
20. `temporary_slots`
21. `add_count`
22. `mul_count`
23. `cross_count`
24. `dot_lane_count`
25. `comparison_count`

## Scalar encoding

- unsigned integers: minimal base-10 ASCII, no sign, no leading zeros except `0`;
- binary64 score scalar: `0x` followed by exactly 16 lowercase hex digits from `to_bits()`;
- boolean: `0` or `1` only;
- absent selected identity: `none`; otherwise unsigned integer;
- no rejection: `none`.

## Resource-field semantics

`query_bits`, `key_bits`, `value_bits`, `position_bits`, `dynamic_state_bits`, and `static_parameter_bits` are **unsigned integer bit counts**. They are not payload dumps or lists of floating-point values. They quantify candidate-visible/retained storage according to the frozen resource ledger and are always present, including when zero.

`temporary_slots`, `add_count`, `mul_count`, `cross_count`, `dot_lane_count`, and `comparison_count` are unsigned integer semantic operation/storage counts. They are not wall-clock measurements.

## Enum tokens

`split`: `development` or `validation`.

`cell`: `P1`, `P2`, `P3`, `P4`, `P5`.

`arm`: `T0`, `T1`, `T3`, `T4`.

`rejection_reason` is one of:

`none`, `malformed_episode`, `non_finite_input`, `non_finite_derived`, `invalid_geometry`, `out_of_bounds`, `missing_target`, `duplicate_target`, `length_mismatch`, `unresolved_protocol_parameter`, `arithmetic_overflow`, `unexpected_extra_output`, `ambiguous_target`, `retry_budget_exhausted`, `ambiguous_t1_top`.

No implementation-defined enum formatting is permitted. A conforming encoder must be byte-deterministic for identical semantic records.
