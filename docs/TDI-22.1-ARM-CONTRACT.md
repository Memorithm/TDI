# TDI-22.1 — Normative Arm Semantics

Status: **part of the TDI-22.1 preregistration; no execution authorization**.

This companion contract removes any ambiguity between the same-width vector and non-torsor controls. It is normative wherever the summary preregistration is less explicit.

For a key torsor stored at reference point `P`, let its raw local six-component representation be `(R, M(P))` and its origin-reduced representation be `(R, C)` with `C = M(P) + P x R`. For query twist `(v, omega)` at `Q`:

- **T0 vector reference**: `score_T0 = v.R + omega.M(P)`. T0 treats the stored local six-vector as ordinary numeric channels; it does not transport the moment and does not use `P` in scoring.
- **T1 torsor value only**: uses exactly `score_T0`; its value payload is `(R,C)` and its readout rule is frozen separately.
- **T3 full torsor**: `score_T3 = (v + Q x omega).R + omega.C`, exactly equivalent to the frozen direct pairing `v.R + omega.M(Q)`.
- **T4 matched non-torsor control**: `score_T4 = v.R + omega.C`. T4 uses the same six query lanes `(v,omega)` and the same reduced key lanes `(R,C)` but deliberately omits `Q x omega`.

Therefore the exact T3-minus-T4 difference is

`score_T3 - score_T4 = (Q x omega).R`.

This makes T3 versus T4 the intended test of the query-side Varignon transport contribution under matched six-component storage/reduction width. T0 remains a conventional raw-local-vector context baseline and is not the torsor-specific control.

Operation counts are still recorded separately: T4 is dimension/storage matched, not asserted to be arithmetic-cost identical to T3.
