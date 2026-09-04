# TDI-9.1 — Deterministic P1/P2/P3 Task Generators

Status: **NON-FINAL DEVELOPMENT/VALIDATION FOUNDATION**

This slice implements only the three frozen TDI-9.0 task-family generators. It does not freeze final sizes, final seed domains, final populations, solver semantics, verifier semantics, checkpoint semantics, policy logic, uncertainty rules or any TDI-9.2 surface.

## Structural leakage boundary

Every generated instance is split into two typed surfaces:

- `PolicyTask`: ordinary task information that may be consumed by solver/policy execution;
- `EvaluatorRecord`: generator seed, hidden difficulty stratum, exact target and generator-side construction oracle.

`PolicyTask` has no seed, hidden stratum or evaluator-target field. The evaluator record must remain outside adaptive-policy inputs. This type boundary is necessary but not sufficient by itself; later execution harnesses must preserve it.

## P1 — staged evidence accumulation

P1 emits an odd-length sequence of exact `+1/-1` evidence. The caller supplies a non-final `decisive_step`, defined as the first one-based prefix for which the final majority is mathematically irreversible even if every remaining evidence item opposed the current majority.

Generation verifies that the requested decisive step is exactly achieved. The policy sees only the evidence sequence. The evaluator record retains the exact binary target and the earliest irreversible prefix for construction validation.

This generator intentionally does not expose a hidden difficulty label or the requested decisive step to the policy.

## P2 — verification-sensitive inference

P2 generates a bounded GF(2) parity system. For width `w`, the `w` constraints form a unit-lower-triangular binary system: row `i` contains pivot bit `i` and optionally lower-index bits. The system therefore has one exact satisfying bit-vector.

The policy sees the parity constraints and width as ordinary task input. The evaluator record stores the exact target bit-vector. A later independent verifier may test whether a candidate satisfies the public constraints, but verifier execution is not implemented by this slice.

## P3 — recoverable deceptive fork

P3 emits:

1. a public choice point;
2. bounded local evidence favoring a decoy branch;
3. an ordinary task event that explicitly eliminates that decoy branch;
4. bounded recovery evidence favoring the surviving branch.

The policy sees these events as ordinary task evidence. The evaluator record retains the exact target, decoy identity and contradiction-event index. Checkpoint creation, restoration, replay and backtracking policy semantics remain separate TDI-9.1 work.

## Determinism

Generation uses domain-separated SplitMix64 derivation from caller-supplied non-final seeds, checked event-count arithmetic and fallible allocation. No nondeterministic iteration is used.

## Qualification

`scripts/check-tdi9.1-task-generators.sh`:

- reruns the frozen TDI-9.0 bootstrap and TDI-9.1 foundation gate;
- checks the P1/P2/P3 and Shallow/Intermediate/Deep vocabularies;
- checks typed policy/evaluator separation;
- rejects TDI-9.2/final executable surfaces;
- rejects silent expansion into solver/verifier/policy-search semantics;
- checks Rust formatting;
- runs a targeted compile/integration fixture.

The fixture validates P1 across all legal decisive positions for odd lengths 3 through 21 and 64 non-final seeds per configuration, validates unique P2 solutions on a bounded instance, and validates the P3 decoy/contradiction structure.

## Explicitly unresolved TDI-9.1 decisions

Before TDI-9.2 can exist, later TDI-9.1 work must still freeze the concrete task parameters for each primary cell, exact solver/verifier/checkpoint semantics, exact policy observation vector, concrete matched resource envelopes, paired uncertainty implementation, non-final/final domain derivation, final future-entropy contract, rejection taxonomy, evaluator manifests and result schema.
