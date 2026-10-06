# TDI-21.0 — Pascal/ANF gate-bank Development protocol P1

Status: **preregistered non-final Development/Validation protocol; no confirmatory or final execution authorised**.

This protocol is a TDI-owned evaluation of an exact Boolean execution method for
existing TDI-21 B4 algebraic-normal-form (ANF/Zhegalkin) programs. It does not
import SML-GENIUS code or results, does not alter TDI-21 task semantics, and does
not promote a model/runtime architecture. SML-GENIUS remains the owner of
model-side Pascal primitives.

## Question

For a bank of exact Boolean ANF gates, can a bit-sliced subset-zeta transform
(the Pascal triangle modulo 2 / Boolean-lattice zeta transform) materialize the
same gate outputs with lower preregistered semantic work than competent
TDI-native controls in declared non-final regions?

The candidate is an execution method, not a learned relation. A positive result
does not establish language-model quality, attention replacement, SBG/MOR/
Delta-KV value, or hardware superiority.

## Arms

All arms consume the same generated ANF coefficient bank and the same ordered
query addresses.

- **PASCAL_ZETA_BANK**: pack all eight gate coefficients into one `u8` lane per
  monomial address, then apply the in-place subset-zeta transform over GF(2).
  One table XOR counts as one packed bank XOR.
- **DIRECT_ANF_QUERY**: evaluate each gate independently from its declared ANF
  monomials at each requested assignment.
- **GENERIC_FULL_MATERIALIZE**: evaluate every gate independently at every
  assignment, materialize the complete table, then service the same queries.
- **RESIDENT_TABLE**: lookup-only lifecycle reference after an already
  materialized table exists. It is diagnostic and excluded from materialization
  superiority decisions.

No arm may use Q/K/V, pairwise token scoring, softmax, an attention fallback, or
a history scan. The TDI-21 pairwise-comparison counter remains zero.

## Frozen deterministic bank generator

Each cell has exactly eight gates. For variable count `n`, domain size
`K = 2^n`, density `d`, gate index `g in 0..7`, and term index
`t in 0..d-1`, the enabled non-constant monomial mask is

`1 + ((257*g + 73*t) mod (K - 1))`.

The step 73 is coprime to every frozen `K-1`, so masks are unique within a
gate. Gate constant is `g mod 2 == 1`. The implementation must reject any
duplicate mask rather than silently deduplicate.

This generator is fixed before observing P1 outcomes and is not derived from
SML-GENIUS PB results.

## Non-final splits

Only the following variable-count split is authorised:

- **Development**: `n in {6, 8}` (`K in {64, 256}`);
- **Validation**: `n in {10, 12}` (`K in {1024, 4096}`);
- **Final/confirmatory**: **absent and forbidden**.

Density is frozen per width as

`d in {4, min(32, K-1), min(256, K-1)}`.

Query load is

`q in {min(16,K), min(64,K), K}`.

Reuse is `r in {1, 8}`.

The three query schedules are deterministic and result-independent:

1. **AFFINE** — addresses `(17 + 37*i) mod K` in sequence order;
2. **LOW_WEIGHT_FIRST** — all addresses sorted by
   `(popcount(address), address)`;
3. **HIGH_WEIGHT_FIRST** — all addresses sorted by
   `(Reverse(popcount(address)), address)`.

For `q < K`, take the first `q` addresses from the schedule. For `q = K`,
use the complete schedule. Reuse repeats the same ordered query list; it does
not regenerate or tune addresses.

## Required evidence

Every cell must record:

- split, `n`, `K`, density, schedule, `q`, reuse;
- exact output checksum for every arm;
- pairwise output mismatch counts against an independently written direct oracle;
- packed-zeta XOR count;
- direct monomial-term tests;
- generic materialization monomial-term tests;
- query lookup count;
- coefficient bytes, materialized table bytes and ANF semantic bits;
- pairwise-token-comparison count, which must remain zero.

Wall-clock timing may be collected later only under a separately declared
physical benchmark panel. P1's scientific decision is based on exact semantics
and deterministic work accounting, not noisy CI timing.

## Gates

- **P1-G1 exactness**: every non-final cell has zero output mismatch across
  PASCAL_ZETA_BANK, DIRECT_ANF_QUERY and GENERIC_FULL_MATERIALIZE.
- **P1-G2 representation accounting**: all semantic byte/bit and operation
  counters are finite, checked and internally consistent.
- **P1-G3 TDI-21 prohibition**: pairwise token comparisons remain zero and no
  prohibited attention primitive is introduced.
- **P1-G4 Development structural utility**: for each Development width at
  density >= 32, full-domain `q=K`, reuse >= 1, packed-zeta work is lower than
  both DIRECT_ANF_QUERY and GENERIC_FULL_MATERIALIZE under the frozen counters.
- **P1-G5 Validation generalization**: P1-G4 holds unchanged for both Validation
  widths without changing generator, schedules, densities, query loads or
  accounting after Development is observed.
- **P1-G6 sparse boundary retention**: any cell where a competent direct control
  has lower work is retained and reported; it is not excluded from the matrix.

## Decision

- If P1-G1, G2 or G3 fails: `REJECT_IMPLEMENTATION_OR_PROTOCOL`.
- If G1-G3 hold, G4 holds, but G5 fails:
  `RETAIN_DEVELOPMENT_ONLY_NO_GENERALIZATION`.
- If G1-G3 and G4-G5 hold:
  `RETAIN_NONFINAL_PASCAL_GATE_BANK_UTILITY`.
- Independently, G6 is mandatory reporting and cannot be used to erase a
  negative/sparse region.

No P1 outcome authorises TDI-21.3 confirmation, a production route, SciRust
promotion, FLAT-ATTENTION integration, or any SML-GENIUS model-path change.

## Reproducibility and provenance

The implementation must be Rust, dependency-free inside the TDI workspace,
deterministic across repeated runs, and wired into
`scripts/check-tdi21-development.sh`. The retained evidence must bind the exact
Git commit, protocol path, source hashes and deterministic output bytes.
Development and Validation may execute autonomously. No protected/final
population or confirmation surface may be created by this protocol.
