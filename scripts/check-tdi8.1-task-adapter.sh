#!/usr/bin/env bash
set -euo pipefail

fail() {
    echo "TDI-8.1 task adapter ERROR: $*" >&2
    exit 1
}

SOURCE="tdi-bench/src/task_adapter_v8.rs"
DOC="docs/TDI-8.1-TASK-ADAPTER-FOUNDATION.md"
HARNESS="tdi-bench/tests/tdi8_task_adapter_compile.rs"
LIB="tdi-bench/src/lib.rs"
MANIFEST="tdi-bench/Cargo.toml"
LOCK="Cargo.lock"

for file in "$SOURCE" "$DOC" "$HARNESS" "$LIB" "$MANIFEST" "$LOCK"; do
    test -s "$file" || fail "missing task-adapter surface: $file"
done

grep -Fq 'pub mod task_adapter_v8;' "$LIB" || fail "task_adapter_v8 is not exported by tdi-bench"
grep -Fq 'tdi-ai = { path = "../tdi-ai" }' "$MANIFEST" || fail "tdi-bench does not declare the tdi-ai adapter dependency"
grep -Fq 'pub struct ExactU64Binary64' "$SOURCE" || fail "lossless u64/binary64 codec missing"
grep -Fq 'pub const MIN_TASK_EVENT_INPUT_WIDTH: u64 = 5;' "$SOURCE" || fail "leakage-safe lossless event-frame minimum missing"
grep -Fq 'pub struct TaskAdapterLayout' "$SOURCE" || fail "caller-supplied adapter layout missing"
grep -Fq 'if recurrent_input_width < MIN_TASK_EVENT_INPUT_WIDTH' "$SOURCE" || fail "minimum input-width guard missing"
grep -Fq 'input.resize(width, 0.0);' "$SOURCE" || fail "deterministic zero-padding rule missing"
grep -Fq 'pub enum A0TaskAction' "$SOURCE" || fail "A0 task action mapping missing"
grep -Fq 'pub struct TaskEventPlan' "$SOURCE" || fail "shared event schedule missing"
grep -Fq 'pub fn build_task_adapter_plan' "$SOURCE" || fail "adapter plan builder missing"
grep -Fq 'select_distractor_read_key' "$SOURCE" || fail "distractor logical-hit guard missing"
grep -Fq 'pub fn audit_associative_projection' "$SOURCE" || fail "physical associative-projection audit missing"
grep -Fq 'generator_class_reuses' "$SOURCE" || fail "generator-side class reuse accounting missing"
grep -Fq 'physical_replacement_collisions' "$SOURCE" || fail "physical collision accounting missing"
grep -Fq 'query targets are evaluator-owned and are never encoded into an arm input' "$DOC" || fail "query-target leakage boundary missing"
grep -Fq 'collision_class' "$DOC" || fail "generator collision metadata boundary missing"
grep -Fq 'provides no experimental default' "$DOC" || fail "adapter dimension non-freeze statement missing"
grep -Fq 'creates no final holdout' "$DOC" || fail "future-holdout non-creation statement missing"

if grep -Fq 'fill_pair(&mut input, 3, key.collision_class());' "$SOURCE"; then
    fail "generator collision class leaked into recurrent input"
fi
if grep -Fq 'fill_pair(&mut input, 5, target.code());' "$SOURCE"; then
    fail "query target leaked into recurrent input"
fi
if grep -Fq 'fill_pair(&mut input, 7, source_index);' "$SOURCE"; then
    fail "source index leaked into recurrent input"
fi

for test_name in \
    exact_u64_codec_round_trips_edge_values \
    decoder_rejects_noncanonical_coordinates \
    layout_only_enforces_minimum_lossless_width \
    symbolic_t1_maps_to_shared_recurrent_schedule_and_exact_a0_targets \
    recurrent_frames_exclude_targets_collision_classes_and_source_indices \
    t2_writes_and_queries_use_identical_derived_memory_keys \
    distractor_read_key_is_outside_logical_write_set \
    projection_audit_separates_generator_classes_from_physical_collisions \
    source_event_order_is_preserved_exactly; do
    grep -Fq "fn ${test_name}()" "$SOURCE" || fail "required task-adapter oracle test missing: $test_name"
done

grep -Fq 'fn downstream_crate_can_build_lossless_shared_task_schedule()' "$HARNESS" || fail "public task-adapter compile/smoke test missing"
grep -Fq 'fn downstream_crate_can_measure_physical_projection_separately_from_t3_classes()' "$HARNESS" || fail "public physical-collision audit regression missing"

cargo test -p tdi-bench --locked 'task_adapter_v8::tests'
cargo test -p tdi-bench --locked --test tdi8_task_adapter_compile

printf 'TDI-8.1 lossless symbolic adapter: VERIFIED\n'
printf 'TDI-8.1 symbolic-execution leakage boundary: VERIFIED\n'
printf 'TDI-8.1 shared A0/A1/A2/A3 schedule: VERIFIED\n'
printf 'TDI-8.1 distractor logical-key separation: VERIFIED\n'
printf 'TDI-8.1 generator-class/physical-collision separation: VERIFIED\n'
printf 'TDI-8.2 executable/token surface: NOT CREATED BY THIS TRANCHE\n'
printf 'TDI-8.1 task-adapter gate: PASS\n'
