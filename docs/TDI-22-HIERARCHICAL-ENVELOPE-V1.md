# TDI-22 — Bounded hierarchical torsor attention (development-only)

## Scope and relation to the merged streaming reference

This is an **experimental Rust scalar implementation**, built on the
TDI-22 sign convention and the previously merged factorized streaming row.
It is deliberately isolated under tdi-ai's feature "experimental".
No frozen TDI-22.0 artefact, evaluation population, confirmation protocol,
FLAT-ATTENTION execution route, or GPU kernel is changed.

Input keys are ordered by sequence index in a **binary segment tree**, not
in an Octree that implicitly assumes physical R^3 language-token geometry.
Each node stores (i) a count, (ii) a mean factorized torsor (R,C),
(iii) coordinate-wise extrema for R and C, and (iv) its contiguous token
range. Construction is linear in the key count, up to the explicit bound
65,536, and consumes at most 2N-1 tree nodes.

For a query twist (v,omega) at position Q:

    C_i = M_i(P_i) + P_i x R_i
    a = v + Q x omega
    s_i = scale * (a . R_i + omega . C_i)

The score of an averaged block is the arithmetic mean of its member scores,
in exact arithmetic. A block of K keys substituted by one average key is
assigned a **logit log K mass correction**:

    log_mass(block) = scale * (a . mean_R + omega . mean_C) + log K

The R and C values are aggregated using the same online-softmax weights.
Only once at the end is M_out(Q) reconstructed as C_out - Q x R_out.
Without the log K term, a large block would have the same attention mass as
one token even if all underlying scores were identical; that is incorrect.

## Acceptance, causal masking and exact fallback

For each candidate block calculate the conservative component-extrema
score spread around the stored mean:

    delta_block = scale *
       (sum_h |a_h| max(|minR_h - meanR_h|, |maxR_h - meanR_h|)
      + sum_h |omega_h| max(|minC_h - meanC_h|, |maxC_h - meanC_h|))

If delta_block <= caller_tolerance, the entire eligible block is replaced
by its summary and its multiplicity. Otherwise its children are visited.
A leaf is always scored exactly. A block containing both eligible and
causally masked future keys is always split; no future key is scored.

Tolerance zero therefore gives an algebraically exact reference limit:
only score-homogeneous blocks can collapse. With nonzero tolerance,
the result is *approximate* and the maximum actual accepted delta is
returned. The algorithm is deterministic, but finite-precision roundoff
means that a true machine-verified numerical enclosure is **not** claimed.

## Mathematical error guarantee (real arithmetic only)

For every key the surrogate score differs from the true score by at most
Delta = max delta of accepted blocks. The unnormalized partition functions
therefore satisfy

    exp(-Delta) <= Z_exact/Z_surrogate <= exp(Delta)

and thus |log Z_exact - log Z_surrogate| <= Delta. The true and surrogate
normalized token distributions have total variation <= tanh(Delta), which
follows from the bound exp(-2Delta) <= p_i/q_i <= exp(2Delta).

For any coordinate of a torsor value lying in [min,max]:

    |E_p[value] - E_q[value]| <= (max - min) * tanh(Delta).

The implementation returns per-coordinate bounds on R and C, and propagates
them to the transported moment M(Q) using the triangle inequality.
Only extrema from fully eligible accepted blocks contribute to the error
envelope; masked future values never participate in its range calculation.
When the actual accepted score deviation is zero, both component-error
bounds are returned as zero without subtracting potentially overflowing
but finite extrema. Floats are checked for nonfinite intermediates.

These analytical envelopes are not IEEE-754 directed-rounding interval
certificates. They must never be reported as formal machine-verified
enclosures or as model-quality guarantees.

## Complexity, applicability and controls

- Build cost and storage: O(N) nodes with up to 2N-1 entries.
- Query traversal: O(B) visited nodes for B selected or rejected segments.
  The worst case is O(N); zero tolerance is not guaranteed to accelerate
  arbitrary keys.
- Staging scratch: O(log N) traversal stack for the balanced binary tree.
- If all keys have the same scores for the given query, the root can be
  reduced to one score evaluation, even if values differ in invisible
  directions. This is a **logical operation-count** reduction only.
- Causal selection never accepts a partially visible node; a prefix boundary
  requires descending into its children.
- No claim of universal O(log N) query cost, O(N log N) total time,
  physical bandwidth savings or physical acceleration is made.
- Scoring remains a six-channel torsor pairing. The matched T4 non-torsor
  scientific control remains required before an architectural claim.
- The experimental tolerance is a declared software knob, not permission
  to change an existing frozen scientific evaluation arm after observing
  results.
- Before GPU/FLAT graduation: independently compare dense references,
  causal masks, score distributions, realistic workloads, error budgets,
  output quality, physical latency and effective bytes moved.

## Reproduction commands

For the bounded unit suite:

    cargo test -p tdi-ai --features experimental tdi22_hierarchical_attention

For an independent dense direct-Varignon comparison over 256 synthetic keys,
eight full and causal query indices and fixed tolerances:

    cargo run -p tdi-ai --features experimental --example tdi22_hierarchical_sweep

The sweep prints logical score evaluations, collapsed blocks and maximum
deviations, and fails if the independently computed binary64 dense output
exceeds the declared analytical envelope plus explicit numerical slack.
The synthetic sweep is Development-only, is not confirmation, and reports
no timings. Any physical qualification must be scheduled through
Memorithm/RemoteOps as the sole remote control plane.
