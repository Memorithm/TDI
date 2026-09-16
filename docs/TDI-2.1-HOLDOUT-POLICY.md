# TDI-2.1 — Holdout opening policy

Date frozen: 2026-09-16

## Preconditions

`PrimaryHoldout` must remain unopened until all of the following are committed on `main`:

- hypotheses;
- task/generator specification;
- experience-store configuration;
- baselines;
- candidate architecture/configuration;
- confidence definition and calibration method;
- arbitration threshold and utility parameters;
- metric/statistical analysis plan;
- falsification criteria;
- exact source revision and runnable command.

## One-opening rule

The primary confirmatory analysis is executed once on the frozen revision. If execution fails before producing any target-dependent aggregate, a technical correction may be made only if it is documented, does not inspect target-dependent results and preserves all scientific parameters.

If protected target-dependent aggregates were produced, the holdout is considered opened. Subsequent changes are post-hoc and must use a new series or a newly generated holdout.

## Transfer holdout

`TransferHoldout` is opened only after the primary holdout report is immutable. The transfer population must differ by the preregistered structural shift, not by a hand-selected subset discovered from errors.

## Blinding record

The result artifact must record:

- commit SHA used for execution;
- split manifest digest;
- command line;
- timestamp;
- environment fingerprint;
- whether any protected aggregate had been produced before an interruption;
- correction record if applicable.

## Prohibited responses to failure

After holdout opening do not:

- retune confidence or routing thresholds;
- remove difficult strata;
- alter loss or utility penalties;
- change random seeds to seek a favorable result;
- redefine the primary endpoint;
- replace the baseline because it performed better than expected.
