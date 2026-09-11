# TDI holdouts — do not touch

This file is the fail-closed contract for Orchestrator and any other autonomous agent.

## Frozen

- Series 1–6.7 evidence already collected stays archived; do not rerun to shop a better number.
- Freeze 11.2 fields: resolve or mark BLOQUÉ with a date. Do not silently widen the field list.
- Labeled holdouts (EXACT / PROVED / NUMERICAL partitions that the suite marks held-out) are unreadable for training and unwritable for "improvements".

## Canon

Owner of the TDI operator is this repository (`tdi-core`, `tdi-bench`, `tdi-ai`, `tdi-operator`). `scirust-tdi` is a facade, not a second series.

## Orchestrator

AUTO_MERGE is denied for this repository (see Memorithm/orchestrator `docs/HOLDOUTS.md`). Documentation-only PRs that restate this contract are allowed. Feature PRs against holdout paths are not.

Related: issue #186, historical #151.
