#!/usr/bin/env bash
set -euo pipefail

fail() {
    echo "TDI-8.1 A0 reference ERROR: $*" >&2
    exit 1
}

SOURCE="tdi-ai/src/full_history_reference.rs"
DOC="docs/TDI-8.1-A0-FULL-HISTORY-REFERENCE.md"
HARNESS="tdi-ai/tests/tdi8_a0_full_history_compile.rs"
LIB="tdi-ai/src/lib.rs"

for file in "$SOURCE" "$DOC" "$HARNESS" "$LIB"; do
    test -s "$file" || fail "missing A0 reference surface: $file"
done

grep -Fq 'pub mod full_history_reference;' "$LIB" \
    || fail "A0 full-history reference is not exported by tdi-ai"
grep -Fq 'pub struct FullHistoryLayout' "$SOURCE" \
    || fail "A0 full-history layout missing"
grep -Fq 'pub struct A0Reference' "$SOURCE" \
    || fail "A0 public reference missing"
grep -Fq 'pub struct A0Readout' "$SOURCE" \
    || fail "A0 readout/coefficient surface missing"
grep -Fq 'let squared = difference * difference;' "$SOURCE" \
    || fail "fixed-order squared-L2 content score missing"
grep -Fq 'if distance <= best_distance {' "$SOURCE" \
    || fail "declared most-recent exact-tie rule missing"
grep -Fq 'coefficients[best_index] = 1.0;' "$SOURCE" \
    || fail "deterministic one-hot read coefficients missing"
grep -Fq '.with_cumulative_history(StorageBits::new(cumulative_history))' "$SOURCE" \
    || fail "A0 cumulative-history accounting missing"
grep -Fq 'HISTORY_COUNT_METADATA_BITS' "$SOURCE" \
    || fail "A0 history-count metadata accounting missing"
grep -Fq 'READOUT_SCALAR_TEMP_BITS' "$SOURCE" \
    || fail "A0 read temporary accounting missing"
grep -Fq 'use tdi_ai::full_history_reference::' "$HARNESS" \
    || fail "downstream smoke test does not consume the public A0 API"

for test_name in \
    layout_rejects_zero_widths \
    append_retains_complete_history_in_order \
    rejected_append_cannot_partially_mutate_history \
    hard_content_read_selects_nearest_complete_history_item \
    exact_distance_ties_select_the_most_recent_item \
    read_rejects_empty_or_non_finite_inputs_without_mutation \
    fixed_order_distance_fails_closed_on_non_finite_intermediate \
    accounting_grows_with_history_and_reports_peak_read_temporaries \
    snapshot_and_clear_cover_the_complete_persistent_history; do
    grep -Fq "fn ${test_name}()" "$SOURCE" \
        || fail "required A0 oracle test missing: $test_name"
done

cargo test -p tdi-ai --locked 'full_history_reference::tests'
cargo test -p tdi-ai --locked --test tdi8_a0_full_history_compile

printf 'TDI-8.1 A0 public API: VERIFIED\n'
printf 'TDI-8.1 A0 complete-history retention: VERIFIED\n'
printf 'TDI-8.1 A0 deterministic hard content attention: VERIFIED\n'
printf 'TDI-8.1 A0 read coefficients/tie rule: VERIFIED\n'
printf 'TDI-8.1 A0 exact history/temporary accounting: VERIFIED\n'
printf 'TDI-8.1 A0 reference gate: PASS\n'