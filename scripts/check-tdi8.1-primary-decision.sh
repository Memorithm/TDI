#!/usr/bin/env bash
set -euo pipefail

fail() {
    echo "TDI-8.1 primary decision ERROR: $*" >&2
    exit 1
}

SOURCE="tdi-bench/src/decision_v8.rs"
LIB="tdi-bench/src/lib.rs"
DOC="docs/TDI-8.1-PRIMARY-DECISION-RULES.md"
HARNESS="tdi-bench/tests/tdi8_primary_decision_compile.rs"

for file in "$SOURCE" "$LIB" "$DOC" "$HARNESS"; do
    test -s "$file" || fail "missing TDI-8 primary decision surface: $file"
done

grep -Fq 'pub mod decision_v8;' "$LIB" \
    || fail "decision_v8 is not exported by tdi-bench"
grep -Fq 'pub const TDI8_PRIMARY_CELL_COUNT: usize = 9;' "$SOURCE" \
    || fail "exact nine-cell primary count missing"
grep -Fq 'pub const TDI8_EQUIVALENCE_MARGIN: f64 = 0.02;' "$SOURCE" \
    || fail "frozen delta=0.02 missing"
grep -Fq 'if interval.lower() > delta {' "$SOURCE" \
    || fail "strict Beneficial lower-bound rule missing"
grep -Fq '} else if interval.upper() < -delta {' "$SOURCE" \
    || fail "strict Harmful upper-bound rule missing"
grep -Fq '} else if interval.lower() >= -delta && interval.upper() <= delta {' "$SOURCE" \
    || fail "closed equivalence-band rule missing"
grep -Fq 'if baseline_deficit == 0.0 {' "$SOURCE" \
    || fail "zero-baseline branch missing"
grep -Fq 'MissingOrRejected' "$SOURCE" \
    || fail "missing/rejected primary-cell disposition missing"
grep -Fq '[PrimaryCellDisposition; TDI8_PRIMARY_CELL_COUNT]' "$SOURCE" \
    || fail "fixed nine-cell hypothesis input missing"

for test_name in \
    zero_baseline_branch_never_divides_and_never_becomes_beneficial \
    nonzero_baseline_classifier_uses_exact_frozen_precedence_and_boundaries \
    missing_interval_is_inconclusive_not_favorable \
    invalid_numeric_inputs_fail_closed \
    nine_cell_aggregation_matches_the_frozen_closed_rule; do
    grep -Fq "fn ${test_name}()" "$SOURCE" \
        || fail "required frozen-decision regression test missing: $test_name"
done

cargo test -p tdi-bench --locked 'decision_v8::tests'
cargo test -p tdi-bench --locked --test tdi8_primary_decision_compile

printf 'TDI-8.1 delta=0.02: VERIFIED\n'
printf 'TDI-8.1 four-way primary-cell classifier: VERIFIED\n'
printf 'TDI-8.1 zero-baseline branch: VERIFIED\n'
printf 'TDI-8.1 exact nine-cell hypothesis gate: VERIFIED\n'
printf 'TDI-8.1 primary decision gate: PASS\n'
