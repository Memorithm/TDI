# TDI-21 Pascal P5 realized-cost preregistration

Status: prospective Development/Validation-only research slice. No confirmatory,
protected, or final execution is authorized.

P5 follows the frozen P4 crossover dossier but measures a new endpoint:
single-process realized CPU elapsed time for the same three matched semantic
arms. It is not a fresh replication of P3 or P4, does not reinterpret their
primitive-work counters as latency, and does not authorize model-side promotion.

## Question

Under a fixed host/toolchain and a prospectively fixed paired timing procedure,
where does the realized CPU-time crossover lie between:

- PASCAL_ZETA_BANK;
- DIRECT_ANF_QUERY;
- GENERIC_FULL_MATERIALIZE?

The purpose is to test whether the directional P4 primitive-work boundary is
visible in a realized-cost measurement. P5 does not assume that the P4 direction
must reproduce.

## Frozen non-final geometry

P5 reuses the already-declared P4 synthetic semantic geometry only as a
benchmark workload. Reuse of the workload is not counted as a new P4
replication.

Development widths: n={10,12}.
Validation width: n={15}.

For every declared width:

- gate count: 8;
- density: d={4,16,64};
- schedules: AFFINE, LOW_WEIGHT_FIRST, HIGH_WEIGHT_FIRST;
- load regions: LOW, CROSS, HIGH using the P4 frozen Lstar rule;
- reuse: r={1,8}.

This yields 108 Development cells and 54 Validation cells. No cell may be
removed, added, or reclassified after observing timings.

## Matched arms and exactness

Each cell uses identical generated ANF semantics and identical query addresses
across the three arms. Before a timing observation is accepted, all three arms
must produce identical output checksums and zero semantic mismatch.

The timed work for each arm is:

- Pascal: build the Pascal/subset-zeta materialized table, then perform the
  declared queries/reuse;
- Direct: evaluate the ANF directly for the declared queries/reuse;
- Generic: materialize the full truth table by direct ANF evaluation, then
  perform the declared queries/reuse.

Allocation and materialization that are intrinsic to an arm remain inside that
arm's timed region. Case-plan construction, result serialization, hashing, and
report writing stay outside timed regions.

## Frozen timing procedure

Use one process and one OS thread for a complete split. Record exact host,
kernel, CPU model, rustc version, build profile, Git commit, protocol SHA-256,
implementation SHA-256 set, and case-plan SHA-256 before Development execution.

Build once with the frozen release profile. For every cell:

1. execute one unrecorded warm-up of each arm;
2. execute six measured paired rounds;
3. rotate arm order deterministically by round:
   - round 0: Pascal, Direct, Generic;
   - round 1: Direct, Generic, Pascal;
   - round 2: Generic, Pascal, Direct;
   - rounds 3-5 repeat rounds 0-2;
4. use a monotonic clock and record integer nanoseconds for every arm/round;
5. use `black_box` or the repository-equivalent barrier so outputs cannot be
   optimized away.

No timing sample may be deleted as an outlier. Interrupted cells are retained
as interrupted/inconclusive and are not silently rerun as fresh evidence.

For each arm, the cell summary is the median of its six recorded nanosecond
values, defined as the integer average of the third and fourth values after
sorting. Persist every raw round as well as the median.

## Paired analysis

For every cell retain signed realized-cost contrasts:

- delta_ns_direct = median_ns_direct - median_ns_pascal;
- delta_ns_generic = median_ns_generic - median_ns_pascal.

Positive, zero, and negative values are all retained. The complete fixed grid is
the sensitivity analysis. No adaptive threshold, favorable-cell selection, or
post-hoc outlier rule is permitted.

Also retain the frozen P4 primitive-work contrasts for the identical cell as a
descriptive covariate only. P5 must report disagreements between primitive-work
sign and realized-time sign rather than resolving them by changing accounting.

Forge may independently recompute frozen summaries but must not optimize or
select the P5 grid. Hub may be used only for provenance/transport while binding
the exact protocol, case plan, implementation, host identity, raw timing bytes,
and summaries.

## Execution order

1. commit this protocol;
2. implement the three separately timed matched arms and timing preflight;
3. freeze exact case plan, implementation identities, host/toolchain identity,
   and canonical raw-output format;
4. run preflight without scientific timing claims;
5. execute Development only;
6. commit the complete Development raw timing dossier;
7. execute unchanged Validation only if Development exactness/provenance gates
   hold;
8. commit the complete paired dossier.

Validation must not alter Development design or interpretation.

## Gates

G1: every declared bank is collision-free.
G2: matched-arm semantic outputs are exactly equal for every accepted cell.
G3: all six timing rounds per arm are present as integer nanoseconds.
G4: timing order is exactly the frozen rotating order and no sample is filtered.
G5: all signed positive/null/negative realized-time contrasts are retained.
G6: zero prohibited attention or pairwise-token operation is introduced.
G7: Validation starts only after Development evidence is frozen with unchanged
protocol, case geometry, implementation identities, and timing procedure.

Failure of G1-G2 or G6 rejects the implementation/protocol. Failure of G3-G5 or
G7 makes the affected evidence inconclusive.

## Interpretation boundary

P5 can establish only a host/toolchain-specific realized CPU-time boundary for
the frozen synthetic workloads. It does not establish GPU throughput, energy,
memory efficiency, end-to-end model latency, or model quality, and it is not a
confirmatory/final study.

No protected/final population is created, opened, inspected, or executed.
SML-GENIUS remains the owner of model-side Pascal primitives. P5 authorizes no
promotion into SBG, MOR, Delta-KV, context memory, or model architecture.
