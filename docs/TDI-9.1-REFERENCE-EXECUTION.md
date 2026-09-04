# TDI-9.1 — Reference execution semantics

Status: **non-final development/validation infrastructure**.

This tranche implements the deterministic execution layer required between the
merged TDI-9.1 task generators and future C0/C1/C2/C3 policies. It does not arm
or create TDI-9.2.

## Scope

The reference executor receives only a `PolicyTask`, a `PolicyArm`, and a common
`ResourceEnvelope`. Evaluator metadata is not part of live execution state.

Implemented semantics:

- P1 sequential signed-evidence accumulation;
- P2 sequential solution of the generated unit-lower-triangular GF(2) system;
- P3 branch commitment under local evidence;
- independent explicit verifier semantics;
- one C3-only P3 checkpoint stored immediately before the public `ChoicePoint`;
- verifier-gated P3 backtracking;
- replay accounting after restoration;
- exact logical checkpoint store/restore byte traffic;
- atomic compute and memory rejection;
- a policy-decision accounting hook without policy choice logic;
- post-STOP evaluator scoring through a distinct `StoppedCandidate` type.

## P1

The solver accumulates the observed ±1 evidence in fixed order. The policy
observation exposes current signed score margin and remaining event count but no
hidden decisive-step label. A C2 policy can therefore learn or implement a
stopping rule from current trajectory information only.

The explicit verifier returns `Indeterminate` until all P1 evidence has actually
been observed. It never reads future P1 evidence merely because the immutable
task object exists in the executor.

## P2

The solver processes one triangular parity row at a time and deterministically
sets the current pivot bit. The explicit verifier independently scans the full
public constraint system and returns only `Satisfied` or `Violated`; it never
receives the evaluator target bit-vector.

Verifier compute is charged for the complete fixed-order scan rather than an
early exit. This avoids turning verifier runtime/remaining-budget variation into
an accidental constraint-position side channel.

## P3

The first local evidence after the public `ChoicePoint` commits the solver to a
branch. The generated pre-contradiction evidence therefore commits the ordinary
solver to the decoy. Later `EliminateBranch` evidence does not silently repair
that commitment.

For C3 only, the executor automatically stores one canonical checkpoint
immediately before processing the first `ChoicePoint`. Checkpoint creation is
not a hidden fifth policy action; it is deterministic C3 execution semantics and
its cost is always paid.

The P3 verifier scans only events already processed. If it confirms that the
committed branch has been explicitly eliminated, it returns `Violated` and
records that live branch as local recovery information. `BACKTRACK` is legal
only after this verifier result. Restoration keeps only the fact that the
rejected branch was contradicted; it does not read or preserve the evaluator
answer. On replay, the solver cannot recommit to that refuted binary branch.

## Accounting

Reference compute is divided into the already-frozen components:

- solver operations;
- verifier operations;
- policy-decision operations;
- checkpoint copy/restore operations;
- replayed solver operations.

The P3 checkpoint has a canonical packed logical size of 27 bytes:

`cursor:u64 + left:i64 + right:i64 + eliminated:u8 + committed:u8 + forbidden:u8`.

One byte-copy operation is charged per stored or restored checkpoint byte.
Store and restore bytes are also recorded separately in `CheckpointTraffic`.

Arm memory uses a deterministic logical model rather than compiler object
layout. Immutable task input and evaluator instrumentation counters are common
infrastructure and excluded from arm working memory. Solver state, live
execution metadata, policy state, checkpoint storage, transactional shadow
state, and action scratch are charged through `ResourceMeter`.

Every action is transactionally preflighted on copied logical state. If any
compute, memory, arithmetic, or accounting bound fails, solver state,
checkpoint state, traffic counters, and committed resource usage remain
unchanged.

## Leakage boundary

The live executor has no public accessor returning the stored `PolicyTask`, so a
future policy cannot obtain future sequential events by being handed an
executor reference. Policy-visible data is emitted through `PolicyObservation`
and the current solver candidate only.

The evaluator target is accessed only by `evaluate_stopped(StoppedCandidate,
EvaluatorRecord)`. The dedicated gate requires exactly one `evaluator.target()`
call in the implementation.

## Deliberately not frozen here

This tranche does not implement or choose:

- C0/C1/C2/C3 policy logic;
- concrete Shallow/Intermediate/Deep primary-cell parameters;
- common primary resource envelope values;
- policy search or Forge integration;
- uncertainty intervals or nine-cell aggregation;
- development/validation split sizes;
- final entropy source/event/derivation;
- TDI-9.2 runner, seeds, dataset, result payload, or provenance record.

Those remain separate TDI-9.1 gates.