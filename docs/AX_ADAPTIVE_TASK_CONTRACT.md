# AX-inspired adaptive research task boundary

TDI studies inference dynamics and recovery. It may consume a declarative task
envelope, but it remains the authority for its preregistered scientific
semantics and evidence.

## Task inputs

A TDI experiment binds:

- an immutable protocol/revision;
- reference and candidate arm identities;
- task/workspace source digests;
- fixed compute, memory and horizon bounds;
- permitted observations and actions;
- seed-domain and holdout derivation rules;
- evidence schema and rejection taxonomy.

The task declaration is an execution boundary, not a way to modify a frozen
hypothesis after observing results.

## Lifecycle

```text
DECLARE → FREEZE → ADMIT → EXECUTE → RECORD → QUALIFY
```

Adaptive actions such as CONTINUE, VERIFY, BACKTRACK and STOP are part of the
declared protocol. They cannot access labels, future events or final holdout
material unless the frozen protocol explicitly permits it.

## Boundary

SciRust Hub orchestrates; RemoteOps enforces host/runtime controls; ElasticXxx
selects admissible resource plans. TDI owns scientific interpretation and
promotion decisions. No AX-derived orchestration abstraction is evidence of
scientific improvement.
