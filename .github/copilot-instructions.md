# TDI repository agent instructions

Before repository changes, fetch and read the persistent off-main scientific/ecosystem roadmap:

```bash
git fetch origin agent/ecosystem-roadmap && \
git show origin/agent/ecosystem-roadmap:.agent/TDI_ECOSYSTEM_ROADMAP.yaml
```

Treat root `AGENTS.md` as mandatory bootstrap policy.

Autonomous agents must never supply confirmatory full-run tokens or initiate final holdout execution. Roadmap availability does not authorize a confirmatory experiment.

Preserve preregistration, untouched holdouts, frozen negative results, calibration limitations, and the separation between finite-state TDI evidence and attention/Transformer hypotheses.

If the roadmap is unavailable, fail closed for major scientific-promotion, holdout, preregistration, cross-repository, or merge decisions.
