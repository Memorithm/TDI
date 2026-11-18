# TDI-22 — Factorized streaming torsor attention (non-final reference)

Status: experimental CPU scalar reference; not a trained model, confirmatory
evaluation, FLAT-ATTENTION integration, GPU implementation or performance claim.

## Exact mathematical reduction

For a key torsor with resultant R, moment M(P) and reduction point P, define

```text
C = M(P) + P x R
M(Q) = C - Q x R
```

At query point Q, a twist (v, omega) supplies the dual score

```text
s_i = v . R_i + omega . M_i(Q)
    = (v + Q x omega) . R_i + omega . C_i
```

The query factorization is computed once per row, and each key's C is
prepared once. Score scaling is explicit and positive.

For the normalized value torsor, linearity gives

```text
R_out = sum_i alpha_i R_i
C_out = sum_i alpha_i C_i
M_out(Q) = C_out - Q x R_out
```

This computes the same mathematical weighted sum as individually transporting
each key's moment to Q, up to finite-precision rounding. Importantly, it does
**not** imply that softmax weights can be replaced by arbitrary averages or
that a block's single average torsor represents the original set's
query-dependent softmax response.

## Streaming algorithm

`tdi-ai/src/tdi22_streaming_attention.rs` implements:

- An opt-in prepared key bank (R, C), with finite-input validation.
- A deterministic online log-sum-exp accumulator for one query row.
- One running weighted sum for R and one for C, with shared weights.
- Full and causal key eligibility; future keys are never scored.
- Reconstruction of M(Q) once after the row, rather than per key.
- A direct, individually transported dense software oracle in its tests.
- Typed failure for empty inputs, invalid scales and non-finite arithmetic.

The existing TDI-22 frozen Stage-0 algebra, non-final evaluator and
preregistration payloads are unchanged. The module is exposed only under
`tdi-ai` feature `experimental`.

## Complexity and memory, without empirical shortcuts

For N stored keys and N query rows:

- Preparation: O(N) time and O(N) prepared storage (six f64 per key).
- Scoring: O(N) six-component score evaluations per full row, O(N²) over N
  rows. Causal mode has N(N+1)/2 admitted pairs.
- Per-row auxiliary scratch: O(1), excluding the prepared key bank.
- No NxN score or probability matrix; output rows still require their
  own storage if the caller retains them.
- Transport cross products during aggregation: one per query row, not one
  per query-key pair. Score factorization likewise moves the query-side
  cross product outside the key loop.

The accompanying Gemini discussion's example labels a standard cache as
`2 L N d` values and a torsor cache as `L N (2d+3)` values. **Those
expressions are approximately equal at large d, not in a 1:2 ratio**. A
real KV-memory comparison must account for the actual K, V, R, C, position,
dtype, storage layout, heads, and any retained raw or compressed tensors.
It cannot be inferred from those contradictory figures.

The hierarchical approximation in the discussion likewise has no guaranteed
O(N log N) worst-case bound, and collapsing a node to one score changes
softmax generally. Adaptive tree criteria must be validated against an
independent dense oracle with error budgets before any claimed speedup or
quality preservation.

## Non-final test command

```sh
cargo test -p tdi-ai --features experimental tdi22_streaming_attention
```

No production promotion or device qualification follows solely from passing
these software tests. Remote physical experiments, if authorized, must use
Memorithm/RemoteOps as the sole distant control plane.
