# TDI-21.0 — Relational address learning coverage control

Status: **development-only learning control; stacked on relational binding prototype; not frozen; not confirmatory**.

This slice asks a narrower question than full relational reasoning: can a bounded Boolean rule learned only from Development recover the symbolic address function

```text
(relation, subject) -> packed address
```

and transfer unchanged to renamed Validation identifiers?

The candidate rule family is deliberately tiny and auditable. Each of the eight output address bits is selected independently from:

```text
0, 1, x_i, NOT x_i
```

for the eight input bits formed by the four relation bits followed by the four subject bits. Validation never participates in fitting.

## Negative coverage control

The fixed Development relational family yields five unique packed inputs:

```text
17, 21, 34, 51, 68
```

or, in binary:

```text
00010001
00010101
00100010
00110011
01000100
```

This sparse sample is insufficient to identify the full identity map. In particular:

- entity high bit 3 is never active;
- relation high bit 7 is never active;
- on the observed rows, relation output bit 4 is perfectly aliased with input bit 0;
- relation output bit 5 is perfectly aliased with input bit 1.

A deterministic minimum-complexity learner can therefore achieve zero Development bit errors with a spurious program that uses constants for unseen high bits and cross-field literals for some relation bits.

The renamed Validation family yields:

```text
153, 157, 170, 187, 204
```

which activates the unseen high entity/relation bits. The preregistered software expectation for this constructed control is therefore **failure on all five Validation addresses**, with two wrong bits per address. That result would diagnose Development feature coverage, not a general failure of Boolean relational addressing.

## Positive coverage control

A second Development set contains input zero plus the eight one-hot basis assignments. For an identity target, that is sufficient for this unary rule family to distinguish every output bit from constants, inverted literals and the other variables.

The selected rules must then be exactly:

```text
y_i = x_i, i = 0..7
```

and must transfer with zero mismatch to all 256 possible eight-bit Validation inputs.

## Functional routing control

Exact-address reconstruction and task success are evaluated separately. Under the sparse Development fit, the renamed Validation addresses are all represented incorrectly relative to the literal packed-identity target. However, the learned mapping can still canonically map the renamed namespace onto the Development namespace and remain internally consistent across a multi-hop episode.

The functional harness therefore executes the selected Development-only encoder inside bounded relational memory on the unchanged Validation episodes. A passing relational episode is not re-labelled as exact address recovery.

A separate alias control stores one Development identity and its renamed Validation counterpart simultaneously. Under the sparse under-covered encoder they map to the same learned key, so the second write overwrites the first. This exposes the non-injectivity that split-local task success can hide.

The basis-covered positive control must recover the true eight-bit identity map, eliminate this cross-namespace alias, and preserve both facts simultaneously.

## Evidence boundary

These controls test rule identifiability and renamed-ID transfer for a deliberately simple address function. They do not establish semantic relation learning, natural-language binding, model quality, hardware speed, or attention replacement.

A later learned relational candidate must keep the same causal task/evaluator boundary, account for search work, and be evaluated on relation structures and identifiers not used during selection.
