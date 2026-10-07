# TDI-21 Pascal P4 crossover preregistration

Status: prospective Development/Validation-only research slice. No confirmatory,
protected, or final execution is authorized.

P4 follows the frozen P3 non-final paired map. It tests whether the P3
Pascal-versus-direct work crossover generalizes to previously unused synthetic
widths. It does not reinterpret P3 as model evidence and does not promote any
Pascal primitive into a model architecture.

## Frozen matched arms

Each cell evaluates identical gate-bank semantics with:

- PASCAL_ZETA_BANK;
- DIRECT_ANF_QUERY;
- GENERIC_FULL_MATERIALIZE as the exact-output/accounting control.

All queried outputs must match exactly. Positive, null, and negative paired
contrasts are retained without filtering.

## Non-final splits

Development: n={10,12}.
Validation: n={15}.

Gate count: 8.
Density: d={4,16,64}.
Schedules: AFFINE, LOW_WEIGHT_FIRST, HIGH_WEIGHT_FIRST.

For each width and density, define the prospective semantic crossover load

Lstar = n*K / (2*(8*d - 1)), where K=2^n.

Freeze three effective-load regions before execution:

- LOW: nearest valid integer to Lstar/2;
- CROSS: nearest valid integer to Lstar;
- HIGH: nearest valid integer to 2*Lstar.

Each region is represented by two lifecycle decompositions when valid:
(q=L,r=1) and (q=round(L/8),r=8). Exact integer q values must be emitted and
committed by the preflight before Development begins. No cell may be added or
removed after observing results.

The intended grid is 108 Development cells and 54 Validation cells.

## Frozen accounting

Use the P3 definitions unchanged:

W_pascal = pascal_zeta_xors + query_lookups.
W_direct = direct_term_tests.
W_generic = generic_term_tests + query_lookups.

Persist signed contrasts W_direct-W_pascal and W_generic-W_pascal for every
cell, plus exact checksums, mismatch counts, representation accounting, and
pairwise-token-comparison count.

The full fixed grid is the sensitivity analysis. Forge must not adapt the grid
or select favorable cells. Forge may independently recompute frozen summaries.
Hub may be used only for provenance/transport while binding exact protocol,
plan, source, implementation, and result identities.

## Execution order

1. commit protocol and exact case plan;
2. pass generator/accounting/equivalence preflight;
3. execute Development only;
4. commit complete Development evidence;
5. execute unchanged Validation only if the Development software gates hold;
6. commit the complete paired dossier.

Validation may not influence Development design.

## Gates

G1: generated terms are collision-free for every declared bank.
G2: zero output mismatch across all matched arms.
G3: accounting is exact, finite, and uses the frozen P3 definitions.
G4: zero prohibited attention or pairwise-token operation is introduced.
G5: every signed positive, null, and negative contrast is retained.
G6: Validation starts only after Development evidence is frozen with unchanged
protocol, case geometry, and implementation.

Failure of G1-G4 rejects the implementation/protocol. Failure of G5-G6 makes
the evidence inconclusive.

P4 is not a fresh replication of P3. It contains no final population. SML-GENIUS
remains the owner of model-side Pascal primitives; P4 authorizes no promotion
into SBG, MOR, Delta-KV, context memory, or model architecture.
