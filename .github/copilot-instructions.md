# TDI repository agent instructions

Before repository changes, fetch and read the persistent off-main scientific/ecosystem roadmap:

```bash
git fetch origin agent/ecosystem-roadmap && \
git show origin/agent/ecosystem-roadmap:.agent/TDI_ECOSYSTEM_ROADMAP.yaml
```

Treat root `AGENTS.md` as mandatory bootstrap policy.

The roadmap is persistent coordination state, not authority to override merged protocols, immutable manifests, frozen result records, or current default-branch evidence. If those sources conflict, fail closed for the affected scientific decision and repair the roadmap separately. Do not inspect a forbidden historical holdout merely to reconcile roadmap drift.

For TDI-8.x work, read `docs/TDI-8-PROGRAMME.md`, `docs/TDI-8.0-SCOPE.md`, `docs/TDI-8.0-ASSR-PREREGISTRATION.md`, and `docs/TDI-8.0-IMPLEMENTATION-GATE.md` before evaluator or scientific changes.

TDI-8.1 evaluator implementation is forbidden until TDI-8.0 is merged and frozen. Keep all TDI-8 work isolated from TDI-7.2 final-holdout data, seeds, authorization state, and confirmation surfaces. No TDI-8.2 runner, token, seed range, result surface, or equivalent confirmatory execution path may exist during TDI-8.0 or TDI-8.1.

When `scripts/check-tdi8-bootstrap.sh` exists, run it before a TDI-8 pull request or merge decision.

For TDI-9.x work, read `docs/TDI-9-PROGRAMME.md`, `docs/TDI-9.0-ADAPTIVE-INFERENCE-PREREGISTRATION.md`, `docs/TDI-9.0-IMPLEMENTATION-GATE.md`, and `docs/TDI-9.0-STATUS.md` before evaluator, policy, or scientific changes.

TDI-9.1 implementation is forbidden until TDI-9.0 is merged, blob-pinned and bootstrap-valid. TDI-9 development is agent-first and may autonomously search/falsify policies on non-final development/validation surfaces.

TDI-7.2 and TDI-8.2 retain their existing human-only/forbidden-agent execution boundaries. TDI-9 is a separately preregistered exception: it has no human confirmation token and may use autonomous final confirmation only after its complete evaluator and future-entropy derivation contract are frozen. The future entropy value must have been unknowable at freeze time, and agents must never select, retry, replace or skip the frozen event based on results.

No final TDI-9.2 seed list, dataset, result payload, or runnable confirmation surface may exist during TDI-9.0. During TDI-9.1, creation of a final runner remains blocked until the exact public future-entropy source/event, canonical encoding, deterministic seed derivation, final population, rejection rules, evaluator manifests and provenance schema are frozen.

When `scripts/check-tdi9-bootstrap.sh` exists, run it before a TDI-9 pull request or merge decision.

Preserve preregistration, untouched or future-derived final-evaluation lineage, frozen negative/null results, calibration limitations, explicit matched-resource controls, and the separation between scientific reference evidence and downstream optimization. Architecture and policy labels are hypotheses, not novelty or performance claims.

Do not claim proprietary architecture reconstruction, Transformer replacement, superiority to a named commercial model, strict end-to-end O(N), constant total memory, tokenizer elimination, cognitive transparency, or GPU/Jetson speedup without separate direct evidence.

If the roadmap is unavailable, fail closed for major scientific-promotion, final-evaluation, preregistration, cross-repository, or merge decisions.
