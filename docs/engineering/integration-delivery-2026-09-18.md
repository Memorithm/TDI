# Final industrial integration handoff — 2026-09-18

## Delivered

The consolidated TDI research-engine candidate is published in PR [#490](https://github.com/Memorithm/TDI/pull/490), rebased onto current `main` at `1e7f27a11374de34b06459478fb1c638517d2442`.

The candidate contains the execution, search, sensitivity, reporting, evidence, operational and qualification slices previously prepared for #413. The local source-bound qualification recorded 70/70 public tests, zero skips/expected failures, and negative preflight coverage.

## Dependency pins

The aggregate integration workflow is pinned to the verified merged upstream commits:

- SciRust: `be7fcca3b31cedf722d71a2a56db8f6d088037cf` (PR #1452).
- Forge: `28067ab0aa1d52a2260d9bb2bf35a346547a292a` (PR #39).

## Release gate

GitHub exact-head checks are authoritative. At publication time, 57 workflow runs were created for head `9c7a94da069011f47b6fcc196c8886e1110c30b9`; one had completed successfully, 11 were in progress and 45 were queued. The engine is not declared merged, released, CUDA-qualified or production-routed until all required checks pass and the PR is merged.

The former PR #413 was closed without merge and is superseded by #490.
