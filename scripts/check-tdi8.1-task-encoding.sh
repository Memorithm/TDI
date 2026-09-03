#!/usr/bin/env bash
set -euo pipefail

fail() {
    echo "TDI-8.1 task encoding ERROR: $*" >&2
    exit 1
}

SOURCE="tdi-ai/src/task_encoding.rs"
RUNNER="tdi-ai/src/bin/tdi8-task-encoding-preflight/main.rs"
DOC="docs/TDI-8.1-TASK-ENCODING-PREFLIGHT.md"
SYMBOLIC="tdi-ai/src/task_execution.rs"
LIB="tdi-ai/src/lib.rs"

for file in "$SOURCE" "$RUNNER" "$DOC" "$SYMBOLIC" "$LIB"; do
    test -s "$file" || fail "missing task-encoding preflight surface: $file"
done

# This tranche is intentionally evaluator-local. Promotion into the tdi-ai
# public library remains a later reviewed decision after non-final qualification.
if grep -Fq 'pub mod task_encoding;' "$LIB"; then
    fail "task_encoding was promoted into the public tdi-ai API before qualification"
fi

grep -Fq 'pub const MIN_TASK_INPUT_WIDTH: u64 = 5;' "$SOURCE" \
    || fail "leakage-safe lossless minimum width is missing"
grep -Fq 'pub struct ExactU64Binary64' "$SOURCE" \
    || fail "exact u64 binary64 codec is missing"
grep -Fq 'value.to_bits() == (-0.0f64).to_bits()' "$SOURCE" \
    || fail "canonical exact-u64 decoder no longer rejects negative zero"
grep -Fq 'pub struct LosslessTaskEncoder' "$SOURCE" \
    || fail "lossless task encoder is missing"
grep -Fq 'pub fn association(' "$SOURCE" \
    || fail "association encoding surface is missing"
grep -Fq 'pub fn payload(self, value: TaskSymbol)' "$SOURCE" \
    || fail "payload encoding no longer mirrors the leakage-safe symbolic contract"
grep -Fq 'pub fn query_association(self, key_code: u64)' "$SOURCE" \
    || fail "association-query encoding no longer excludes runner-owned target metadata"
grep -Fq 'pub fn query_payload(self, position: u64)' "$SOURCE" \
    || fail "payload-query encoding no longer excludes runner-owned target metadata"
grep -Fq 'pub struct PayloadKeyCursor' "$SOURCE" \
    || fail "call-order payload-key cursor is missing"
grep -Fq 'pub fn distractor_read_key_for_instance' "$SOURCE" \
    || fail "distractor no-write key guard is missing"
grep -Fq 'pub fn audit_associative_projection' "$SOURCE" \
    || fail "physical projection audit is missing"
grep -Fq 'generator_class_reuses' "$SOURCE" \
    || fail "generator-side class diagnostic is missing"
grep -Fq 'physical_replacement_collisions' "$SOURCE" \
    || fail "physical replacement diagnostic is missing"
grep -Fq 'negative zero' "$DOC" \
    || fail "canonical negative-zero rejection is not documented"

# The upstream executor is the authority on what arm adapters may observe.
grep -Fq 'fn query_association(&mut self, key_code: u64)' "$SYMBOLIC" \
    || fail "symbolic executor query-association leakage boundary changed"
grep -Fq 'fn query_payload(&mut self, position: u64)' "$SYMBOLIC" \
    || fail "symbolic executor query-payload leakage boundary changed"
grep -Fq 'fn payload(&mut self, value: TaskSymbol)' "$SYMBOLIC" \
    || fail "symbolic executor payload-order leakage boundary changed"

for test_name in \
    exact_u64_codec_round_trips_edge_values \
    decoder_rejects_noncanonical_coordinates \
    layout_enforces_only_the_leakage_safe_lossless_minimum \
    query_frames_contain_request_only_and_zero_padding \
    association_frame_contains_only_key_and_value_after_tag \
    payload_frame_has_no_source_position_feature \
    a0_namespaces_task_items_and_queries_without_targets \
    payload_cursor_reconstructs_generator_positions_from_call_order \
    distractor_read_key_is_outside_complete_logical_write_set \
    projection_audit_separates_generator_classes_from_physical_collisions; do
    grep -Fq "fn ${test_name}()" "$SOURCE" \
        || fail "required task-encoding oracle test missing: $test_name"
done

cargo test --locked -p tdi-ai --bin tdi8-task-encoding-preflight
OUTPUT="$(cargo run --quiet --locked -p tdi-ai --bin tdi8-task-encoding-preflight)"
printf '%s\n' "$OUTPUT"

for required in \
    'TDI-8.1 task encoding preflight: PASS' \
    'scope=bounded_preflight_only' \
    'query_target_arm_surface=ABSENT' \
    'source_index_arm_surface=ABSENT' \
    'collision_class_arm_feature=ABSENT' \
    'physical_projection_audit=SEPARATE_RUNNER_DIAGNOSTIC' \
    'final_holdout=DOES_NOT_EXIST' \
    'tdi8_2_surface=ABSENT'; do
    grep -Fxq "$required" <<<"$OUTPUT" || fail "missing preflight status line: $required"
done

printf 'TDI-8.1 lossless binary64 encoding: VERIFIED\n'
printf 'TDI-8.1 canonical negative-zero rejection: VERIFIED\n'
printf 'TDI-8.1 symbolic target/provenance leakage exclusion: VERIFIED\n'
printf 'TDI-8.1 physical projection diagnostics: VERIFIED SEPARATELY\n'
printf 'TDI-8.1 public API promotion: NOT PERFORMED\n'
printf 'TDI-8.1 task-encoding gate: PASS\n'
