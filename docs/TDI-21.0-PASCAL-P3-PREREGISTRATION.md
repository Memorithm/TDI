# TDI-21.0 Pascal P3 preregistration

Status: prospective Development/Validation-only successor to the blocked P2 protocol. No P3 scientific cell has been executed under this protocol. No confirmatory or final execution is authorised.

P2 Development preflight retained two protocol/implementation blockers before Validation: the inherited term generator is non-unique for part of the frozen P2 grid, and the preregistered `work(...)` aggregation was not defined. P3 corrects only those two defects prospectively. P3 is a new protocol version, not a rerun or fresh replication of P2.

Model-side Pascal primitives remain owned by SML-GENIUS. This protocol authorises no promotion into SBG, MOR, Delta-KV, context memory, or model architecture.

## Frozen research question

Across a fixed non-final sensitivity grid, when does exact Pascal/subset-zeta materialisation reduce a declared primitive-operation work proxy relative to matched direct-ANF querying and generic full materialisation, while preserving exact outputs?

The proxy is an operation-accounting comparison only. It is not a wall-clock, GPU-throughput, energy, latency, or model-quality claim.

## Frozen generator

For each gate and term index:

```text
mask = 1 + ((257 * gate + 71 * term) mod (2^n - 1))
```

The step 71 is fixed before execution. For every preregistered width `n={9,11,13,14}`, preflight must verify `gcd(71, 2^n - 1) = 1` and must verify exactly `density` unique non-zero masks per gate for every declared density.

Any uniqueness failure stops P3 before scientific execution and is retained as negative implementation/protocol evidence. The generator must not be changed in-place after observing P3 results.

## Frozen matched design

Arms per cell:

- `PASCAL_ZETA_BANK`
- `DIRECT_ANF_QUERY`
- `GENERIC_FULL_MATERIALIZE`
- `RESIDENT_TABLE` as a diagnostic lifecycle reference only

Development widths: `n={9,11}`, `K={512,2048}`.
Validation widths: `n={13,14}`, `K={8192,16384}`.
Density: `d={4,32,256}`.
Query load: `q={16,64,K}`.
Reuse: `r={1,8}`.
Schedules: `AFFINE`, `LOW_WEIGHT_FIRST`, `HIGH_WEIGHT_FIRST`.

The complete fixed grid contains 216 cells: 108 Development and 108 Validation. No cell may be removed, added, relabelled, or selectively omitted after any P3 observation.

## Frozen primitive-work accounting

The exact work proxy is fixed before execution:

```text
W_pascal  = pascal_zeta_xors + query_lookups
W_direct  = direct_term_tests
W_generic = generic_term_tests + query_lookups

delta_direct  = W_direct  - W_pascal
delta_generic = W_generic - W_pascal
```

All counters are unsigned exact primitive counts; signed deltas use a representation that cannot wrap. A positive delta favours Pascal under this proxy, zero is a null result, and a negative delta favours the matched control. Positive, null, and negative values are all retained.

`RESIDENT_TABLE` is diagnostic and is excluded from the two primary signed contrasts.

## Sensitivity and paired analysis

The full 216-cell grid is the preregistered sensitivity analysis. Each cell is paired because all arms consume the same frozen gate bank, schedule, query addresses, query load, and reuse setting.

No adaptive search, post-hoc threshold tuning, or result-driven cell selection is permitted. Forge must not optimise the P3 grid. Forge may be used only for non-adaptive verification utilities that cannot alter the frozen case set or decision rule.

Hub/orchestration may transport jobs or evidence only when it binds the exact protocol identity, source commit, implementation identities, and result bytes. Hub scheduling must not alter case membership or scientific interpretation.

## Execution order

1. Generator/work-accounting preflight.
2. Full Development execution and persistence.
3. Freeze Development evidence and verify the protocol/implementation/source identities remain unchanged.
4. Full Validation execution with the identical frozen design.
5. Paired analysis over all retained cells.

Validation must not execute if preflight fails or if the protocol/implementation is changed after Development begins.

## Required exact evidence

For every cell retain:

- split, width, domain size, density, schedule, query load, and reuse;
- exact output checksums for all matched arms;
- oracle mismatch counts;
- Pascal zeta XORs;
- direct and generic term-test counts;
- query lookup count;
- representation bytes/bits;
- `W_pascal`, `W_direct`, and `W_generic`;
- signed `delta_direct` and `delta_generic`;
- pairwise-token-comparison count.

The dossier must bind:

- exact Git source commit;
- SHA-256 of this protocol;
- SHA-256 of all implementation sources used by P3;
- canonical complete result bytes and their SHA-256;
- deterministic replay comparison of the canonical result bytes.

No completed P2 evidence may be relabelled as P3 evidence.

## Gates

G1: generator preflight proves uniqueness for every declared width/density before scientific execution.

G2: zero output mismatch in every executed Development/Validation cell and equality of matched output checksums.

G3: all primitive accounting is finite, exact, and internally consistent; no signed-delta overflow or wrap is possible.

G4: no prohibited attention primitive or pairwise token comparison is introduced.

G5: all 216 signed paired contrasts are persisted, including positive, null, and negative values.

G6: Validation executes only after Development evidence is frozen and without changing protocol, generator, accounting, implementation identity, or case geometry.

If G1 fails, stop before scientific execution and retain the blocker. If G2-G4 fail, reject the implementation/protocol while retaining exact evidence. If G5 or G6 fails, mark the campaign inconclusive. If G1-G6 hold, retain only a non-final paired boundary map under the declared primitive-work proxy.

Passing P3 does not authorise confirmation, final evaluation, or downstream model/runtime promotion.

## Protected/final boundary

P3 contains Development and Validation only. It defines no protected/final fixture, no final population identity, and no final execution path. Protected/final populations must not be opened, generated, inspected, or executed under this protocol.
