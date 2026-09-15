# TDI-21.0 — B4 bounded ANF synthesis and algebraic substitution

Status: **development-only Stage-0 engineering; not TDI-21.3, not frozen, no confirmatory execution**.

This increment starts from the merged B0/B1/B2/B3 development stack and adds the first executable B4 surface without changing the public task semantics. The purpose is isolation: establish that an explicit GF(2) / algebraic-normal-form (Zhegalkin) composition can be synthesized, executed and accounted inside the non-attention candidate path before asking whether a different synthesized relation improves a task.

## Exact synthesis

For `n` Boolean variables, a complete truth table of `2^n` outputs is transformed into canonical algebraic normal form by the in-place Möbius transform over GF(2). Assignment index bits are the Boolean variables. The resulting program is

`f(x) = c XOR m_1(x) XOR ... XOR m_k(x)`,

where every monomial `m_i` is a product of selected Boolean variables. Terms are emitted in ascending bit-mask order, so equal truth tables synthesize byte-stable programs.

The development synthesizer permits 1..12 variables and at most 4095 non-constant terms. Those are software safety/search bounds, not scientific hyperparameters and not claims about useful model-scale Boolean width. Truth-table length mismatches, assignments using undeclared variables and allocation failures reject explicitly.

Exhaustive regression enumerates all 256 Boolean functions of three variables and checks the synthesized program on every one of the eight assignments. This validates the finite transform implementation; it is not a learned generalization result.

## First B4 substitution control

The existing B2/B3 write-admission rule is

`x0 AND NOT x1`,

where `x0` and `x1` are the two declared marker bits. Its canonical Zhegalkin form is

`x0 XOR (x0 * x1)`.

`AlgebraicBooleanStream` synthesizes this predicate from the complete four-row truth table rather than hard-coding those ANF terms. On every valid write, it evaluates the synthesized ANF, records the monomial-evaluation work, maps the result into the already-declared B3 marker contract, and then delegates to the unchanged bounded two-way B3 memory.

This deliberate semantic equivalence gives a clean ablation: B4 adds explicit algebraic relation composition while preserving B3 addressing, storage, collision and replacement behavior. A difference in task answers at this stage is therefore a bug, not an expected scientific effect.

Invalid marker values bypass ANF evaluation and reach the existing B3 validator unchanged, so a rejected API call does not accrue apparently successful algebraic work or mutate state.

## Prohibition and accounting boundary

B4 remains a TDI-21 Boolean candidate. The new module introduces no Q/K/V projection, dot product, cosine score, softmax, Hamming/POPCOUNT all-pairs score, dense token-token matrix, attention fallback or history scan for addressing. Its pairwise-comparison counter remains zero.

ANF term evaluations are reported separately from B3 Boolean-word work, route derivations and memory operations. The program representation reports semantic bits for its variable-count byte, constant bit and 64-bit monomial masks. That number excludes Rust object layout, allocator metadata, code, evaluator state and process memory. It must not be compared to B0/B1 or B2/B3 as a matched total-resource budget.

## Development regressions

The B4 contract tests include:

- all 256 three-variable Boolean functions reconstructed exactly;
- the canonical marker polynomial `x0 XOR x0*x1`;
- fail-closed variable/truth-table/assignment bounds;
- rejection of a direct-memory substrate because this first B4 control is specifically B3 + ANF;
- all `8^4 = 4096` four-event streams over writes, inhibited writes, recalls, conjunction and ignore, checking B4 output and B3 native counters for exact parity;
- explicit ANF term accounting with zero pairwise comparisons;
- atomic invalid-marker rejection;
- reset preserving the synthesized program and reserved substrate while clearing episode counters.

The existing `scripts/check-tdi21-development.sh` runs this suite in debug and release in addition to all previous TDI-21 regressions. Executed outcomes belong to exact-head CI logs; source assertions alone are not evidence of execution.

## What this does not establish

This increment does **not** show that ANF routing improves B3, replaces attention at model scale, learns semantic relations, reduces end-to-end compute, or runs faster on hardware. The first B4 program is intentionally equivalent to the existing admission rule.

The next meaningful B4 step is a bounded synthesis/search contract over explicitly declared non-final development predicates, with separate Development/Validation families and a search-cost budget. Only after that contract exists should TDI-21 test whether synthesized algebraic relations outperform hand-written relations under a declared matched comparison. TDI-21.1/21.2/21.3 remain unfrozen and unauthorized by this Stage-0 engineering slice.
