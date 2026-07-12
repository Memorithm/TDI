# TDI — Dynamic Information Theory

TDI is a deterministic Rust research project for studying whether the structure of accessible futures contains predictive information that is not preserved by a scalar entropy baseline.

## What TDI-1 currently proves

On the finite synthetic systems tested here:

- systems can have the same Shannon block entropy but different recovery behaviour;
- an exact prospective return profile separates thousands of same-entropy, opposite-outcome pairs;
- the TDI return profile improves prediction on an untouched 4,000-system holdout set;
- paired deterministic bootstrap confidence intervals remain strictly positive.

These results are limited to the synthetic finite-state families and perturbation protocols in this repository. They do not yet establish a universal physical or information-theoretic law.

## Workspace

- `tdi-core`: exact finite-state dynamics, prospective exploration, signatures, orbits and perturbation recovery.
- `tdi-bench`: adversarial examples, exhaustive scans, leave-one-out evaluation and independent holdout validation.
- `docs/TDI-1-RESULTS.md`: exact protocol, results, confidence intervals and limits.
- `results/`: captured reference outputs.

## Reproduce TDI-1

```bash
./scripts/reproduce-tdi1.sh
```

The script runs formatting checks, tests, Clippy, the adversarial counterexample, the exhaustive scan, the leave-one-out evaluation and the independent holdout evaluation.

## Direct commands

```bash
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo run --release -p tdi-bench --bin tdi-holdout
```
