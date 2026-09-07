# TDI-3 — Preregistered Inter-Width Results

Official historical release for the annotated tag `tdi-3-preregistered-results`, which resolves to frozen scientific commit `b3266406c2a40b73da05ee7b0c54d9d6584d2785`.

## Preregistered verdict

- **TDI-3A — failed**
- **TDI-3B — failed**

These verdicts are preserved exactly. This release does not relabel the experiment as a success.

## What the experiment established

TDI-3 tested whether the predictive information exposed by the earlier TDI programme transported reliably across system widths.

- On width 3, the TDI-3 feature block produced a clear predictive improvement, including a relative MSE reduction of about 13.98%.
- On width 4, MSE and MAE degraded despite an improvement in rank correlation, so the preregistered per-width requirement failed.
- The combined width-3/4 holdout was favourable in aggregate, but the frozen criterion required improvement in each width and therefore remained failed.
- On the width-5 out-of-distribution holdout, absolute error and bias improved substantially, including about 37.03% relative MSE reduction, while R² and Spearman remained negative. This is error reduction without successful structural transfer.

The bounded conclusion is therefore that the tested TDI-3 signal is width-sensitive and does not validate a universal inter-width representation under the preregistered criteria.

## Frozen evidence bundle

The attached archive is built directly from the historical tag and contains:

- `docs/TDI-3-INTERWIDTH-PREREGISTRATION.md`
- `docs/TDI-3-INTERWIDTH-PREREGISTRATION.sha256`
- `docs/TDI-3-INTERWIDTH-EVALUATOR.sha256`
- `docs/TDI-3-SCIENTIFIC-CODE.sha256`
- `docs/TDI-3-INTERWIDTH-RESULTS.md`
- `docs/TDI-3-NUMERICAL-CORRECTION.md`

A SHA-256 file for the archive is published alongside it.

## Scientific status

**Completed historical experiment.** Negative preregistered verdict with informative width-dependent signal and explicit transfer limitations.
