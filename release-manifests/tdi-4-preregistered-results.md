# TDI-4 — Preregistered Target Geometry Results

Official historical release for the annotated tag `tdi-4-preregistered-results`, which resolves to frozen scientific commit `fad29965f057e368ef453bea96e6f3b36dee8da3`.

## Preregistered verdict

- **TDI-4A — failed**
- **TDI-4B — failed**

These verdicts are final for the frozen protocol and are preserved without retrospective reinterpretation.

## What the experiment established

TDI-4 tested a target-geometry formulation with a binary exact-recovery head and a continuous deficit component.

- No exact recovery was observed at horizon 6 in the evaluated populations, so the binary target was constant and the Brier-improvement requirement could not be satisfied.
- The full two-head preregistered protocol therefore failed formally.
- The continuous deficit geometry nevertheless showed strong predictive signal in the tested regime: approximately 25.82% loss reduction on width 3, 23.86% on width 4, 25.31% on the combined in-distribution population, and 50.03% on width-5 OOD.
- The OOD continuous component also improved ranking, reconstructed error, explained variance and bias relative to the baseline in the frozen evaluation.

The bounded conclusion is that the binary exact-recovery head was not identifiable in this regime, while the continuous deficit geometry remained informative. This does not change the negative preregistered verdict.

## Frozen evidence bundle

The attached archive is built directly from the historical tag and contains:

- `docs/TDI-4-TARGET-GEOMETRY-PREREGISTRATION.md`
- `docs/TDI-4-TARGET-GEOMETRY-PREREGISTRATION.sha256`
- `docs/TDI-4-TARGET-GEOMETRY-EVALUATOR.sha256`
- `docs/TDI-4-SCIENTIFIC-CODE.sha256`
- `docs/TDI-4-TARGET-GEOMETRY-RESULTS.md`
- `docs/TDI-4-TARGET-GEOMETRY-RESULTS.sha256`
- `docs/TDI-4-CI-INTEGRITY-CORRECTION.md`

A SHA-256 file for the archive is published alongside it.

## Scientific status

**Completed historical experiment.** Negative preregistered verdict with a degenerate binary target and a reproducible continuous-geometry signal.
