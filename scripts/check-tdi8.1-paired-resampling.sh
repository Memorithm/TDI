#!/usr/bin/env bash
set -euo pipefail

fail() {
    echo "TDI-8.1 paired resampling ERROR: $*" >&2
    exit 1
}

SOURCE="tdi-bench/src/paired_resampling_v8.rs"
LIB="tdi-bench/src/lib.rs"
DOC="docs/TDI-8.1-PAIRED-RESAMPLING-FOUNDATION.md"
HARNESS="tdi-bench/tests/tdi8_paired_resampling_compile.rs"

for file in "$SOURCE" "$LIB" "$DOC" "$HARNESS"; do
    test -s "$file" || fail "missing TDI-8 paired-resampling surface: $file"
done

grep -Fq 'pub mod paired_resampling_v8;' "$LIB" \
    || fail "paired_resampling_v8 is not exported by tdi-bench"
grep -Fq 'pub const TDI8_FAMILY_ALPHA: f64 = 0.05;' "$SOURCE" \
    || fail "frozen family alpha missing"
grep -Fq 'TDI8_FAMILY_ALPHA / TDI8_PRIMARY_CELL_COUNT as f64' "$SOURCE" \
    || fail "Bonferroni alpha=0.05/9 allocation missing"
grep -Fq 'pub struct PairedDeficitObservation' "$SOURCE" \
    || fail "paired observation type missing"
grep -Fq 'pub struct PairedResamplingPlan' "$SOURCE" \
    || fail "caller-supplied resampling plan missing"
grep -Fq 'let threshold = upper.wrapping_neg() % upper;' "$SOURCE" \
    || fail "unbiased bounded rejection sampler missing"
grep -Fq 'let observation = observations[rng.bounded(observations.len())?];' "$SOURCE" \
    || fail "paired index resampling missing"
grep -Fq 'zero_zero_replicates' "$SOURCE" \
    || fail "zero/zero replicate accounting missing"
grep -Fq 'zero_positive_replicates' "$SOURCE" \
    || fail "zero/positive replicate accounting missing"
grep -Fq 'validate_complete_accounting' "$SOURCE" \
    || fail "complete replicate accounting guard missing"

# This foundation must not silently freeze a specific interval estimator,
# replicate count, or resampling seed. Concrete values remain later TDI-8.1
# non-final protocol choices.
if grep -Eq 'TDI8_.*(BOOTSTRAP|RESAMPLING)_(REPLICATES|SEED)[[:space:]]*:' "$SOURCE"; then
    fail "paired-resampling foundation must not freeze replicate count or seed"
fi
for forbidden in 'percentile(' 'BCa' 'studentized interval' 'normal approximation interval'; do
    if grep -Fq "$forbidden" "$SOURCE"; then
        fail "paired-resampling foundation prematurely selects interval method: $forbidden"
    fi
done

for test_name in \
    frozen_bonferroni_allocation_is_exposed_without_interval_method_freeze \
    paired_inputs_and_plan_fail_closed \
    point_estimate_matches_frozen_relative_and_zero_baseline_branches \
    bootstrap_is_deterministic_for_identical_plan \
    bootstrap_resamples_baseline_and_candidate_as_indivisible_pairs \
    zero_baseline_replicates_are_counted_not_silently_dropped; do
    grep -Fq "fn ${test_name}()" "$SOURCE" \
        || fail "required paired-resampling regression test missing: $test_name"
done

cargo test -p tdi-bench --locked 'paired_resampling_v8::tests'
cargo test -p tdi-bench --locked --test tdi8_paired_resampling_compile

printf 'TDI-8.1 generator-level pairing substrate: VERIFIED\n'
printf 'TDI-8.1 Bonferroni alpha allocation: VERIFIED\n'
printf 'TDI-8.1 zero-baseline replicate accounting: VERIFIED\n'
printf 'TDI-8.1 interval/replicate/seed freeze: NOT YET SELECTED\n'
printf 'TDI-8.1 paired resampling gate: PASS\n'
