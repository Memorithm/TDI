# TDI-22.1 — Normative Seed and Dyadic Generation Contract

Status: **part of the TDI-22.1 freeze; non-final only**.

## Stream identity

Primary cell ids are exactly `P1=1`, `P2=2`, `P3=3`, `P4=4`, `P5=5`.
Episode indices must be `< 2^48`.

The packed selector is exactly:

`selector = (cell_id << 56) | episode_index`.

Bits 48..55 are therefore zero. Invalid cell ids or out-of-range episode indices fail closed before packing.

The initial state is:

`state_0 = split_domain XOR selector`.

Development domain: `0x5444493232444556`.
Validation domain: `0x544449323256414c`.

## SplitMix64

Every random scalar consumes exactly one `next_u64()` word. Structural ids/indices consume no random words.

For each call:

```text
x = x + 0x9e3779b97f4a7c15
z = x
z = (z ^ (z >> 30)) * 0xbf58476d1ce4e5b9
z = (z ^ (z >> 27)) * 0x94d049bb133111eb
z = z ^ (z >> 31)
```

All additions and multiplications use wrapping `u64`; final `x` is the next internal state.

## Scalar mappings

Base content component:

`half_step(w) = ((w mod 33) - 16) / 2`.

The subtraction is signed integer arithmetic before binary64 conversion. This yields exactly the 33 half-integers `-8..+8`.

Supplied-coordinate component:

`quarter_step(w) = ((w mod 17) - 8) / 4`.

The subtraction is signed integer arithmetic before binary64 conversion. This yields exactly the 17 quarter-integers `-2..+2`.

## Retry semantics

An evaluator-owned ambiguous target does not rewind the stream. The complete query/candidate content is regenerated from the current forward state. At most 32 attempts are allowed per query; exhaustion is `RetryBudgetExhausted`.

For F3, nuisance geometry is generated only after a unique target exists, so ambiguous-target attempts consume no nuisance-position words.

A T0 tie used by T1 is **not** a generator retry condition. It produces the T1-only rejection `AmbiguousT1Top`, as frozen by `TDI-22.1-FREEZE.md`.

No final/confirmatory seed domain is defined here.
