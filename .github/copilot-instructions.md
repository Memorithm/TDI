# TDI repository agent instructions

Before repository changes, fetch and read the persistent off-main scientific/ecosystem roadmap:

```bash
git fetch origin agent/ecosystem-roadmap && \
git show origin/agent/ecosystem-roadmap:.agent/TDI_ECOSYSTEM_ROADMAP.yaml
```

Treat root `AGENTS.md` as mandatory bootstrap policy.

The roadmap is persistent coordination state, not authority to override merged protocols, immutable manifests, frozen result records, or current default-branch evidence. If those sources conflict, fail closed for the affected scientific decision and repair the roadmap separately. Do not inspect a forbidden holdout merely to reconcile roadmap drift.

For TDI-8.x work, read `docs/TDI-8-PROGRAMME.md`, `docs/TDI-8.0-SCOPE.md`, `docs/TDI-8.0-ASSR-PREREGISTRATION.md`, and `docs/TDI-8.0-IMPLEMENTATION-GATE.md` before making evaluator or scientific changes.

TDI-8.1 evaluator implementation is forbidden until TDI-8.0 is merged and frozen. During TDI-8.0, keep changes limited to preregistration, bootstrap, integrity, review, and other non-evaluator preparation surfaces.

Keep all TDI-8 work isolated from TDI-7.2 final-holdout data, seeds, authorization state, and confirmation surfaces. No TDI-8.2 runner, token, seed range, result surface, or equivalent confirmatory execution path may exist during TDI-8.0 or TDI-8.1.

When `scripts/check-tdi8-bootstrap.sh` exists, run it before a TDI-8 pull request or merge decision.

Autonomous agents must never supply confirmatory full-run tokens or initiate final holdout execution. Roadmap availability does not authorize a confirmatory experiment.

Preserve preregistration, untouched holdouts, frozen negative/null results, calibration limitations, explicit matched-budget controls, and the separation between scientific reference evidence and downstream optimization. TDI-8 architecture labels are hypotheses, not novelty or performance claims.

Do not claim Transformer replacement, superiority to a named architecture, strict end-to-end O(N), constant total memory, tokenizer elimination, VSA dimension-level interpretability, or GPU/Jetson speedup without separate direct evidence.

If the roadmap is unavailable, fail closed for major scientific-promotion, holdout, preregistration, cross-repository, or merge decisions.
