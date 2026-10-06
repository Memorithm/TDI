# TDI-21.0 Pascal P2 preregistration

Status: Development/Validation only. No confirmatory or final execution is authorised.

P2 extends the TDI Pascal/ANF gate-bank bench to wider non-final domains and freezes paired sensitivity analysis before execution. It remains an execution-method study only; model-side Pascal ownership remains with SML-GENIUS and no downstream architecture promotion is authorised.

## Frozen design

Arms are matched per cell: PASCAL_ZETA_BANK, DIRECT_ANF_QUERY, GENERIC_FULL_MATERIALIZE, plus RESIDENT_TABLE as a diagnostic lifecycle reference.

Development widths: n={9,11}, K={512,2048}.
Validation widths: n={13,14}, K={8192,16384}.
Density: d={4,32,256}.
Query load: q={16,64,K}.
Reuse: r={1,8}.
Schedules: AFFINE, LOW_WEIGHT_FIRST, HIGH_WEIGHT_FIRST.

The complete fixed grid contains 216 cells. The bank generator and schedule definitions are inherited unchanged from P1. No cell may be removed after observation.

## Paired/sensitivity analysis

For every cell retain signed contrasts:
- delta_direct = work(DIRECT_ANF_QUERY) - work(PASCAL_ZETA_BANK)
- delta_generic = work(GENERIC_FULL_MATERIALIZE) - work(PASCAL_ZETA_BANK)

Positive, zero and negative values are all retained. The 216-cell grid itself is the frozen sensitivity analysis. No adaptive search or post-hoc threshold tuning is allowed. Forge is therefore not used as an optimizer in P2. Hub/orchestration may be used only for transport/provenance while binding the exact protocol, source and result identities.

## Required evidence

Each cell records split, width, density, schedule, query load, reuse, exact output checksums, oracle mismatch counts, packed XORs, direct/generic term tests, lookups, representation bytes/bits, signed paired contrasts, and pairwise-token-comparison count. The retained dossier also binds the exact Git commit, protocol hash, implementation-source hashes and canonical result-byte SHA-256.

## Gates

G1: zero output mismatch in every Development/Validation cell.
G2: all accounting is finite and internally consistent.
G3: no prohibited attention primitive or pairwise token comparison is introduced.
G4: all 216 signed paired contrasts are persisted, including zero and negative values.
G5: Validation executes without changing the frozen design after Development is observed.

If G1-G3 fail, reject the implementation/protocol. If G1-G3 hold but G4 or G5 fails, mark the evidence inconclusive. If G1-G5 hold, retain only a non-final paired boundary map; this does not authorise confirmation, final evaluation or downstream model/runtime promotion.

No P2 execution may be represented as a fresh replication of P1.
