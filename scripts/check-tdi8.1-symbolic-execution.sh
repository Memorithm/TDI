#!/usr/bin/env bash
set -euo pipefail

fail() {
    echo "TDI-8.1 symbolic execution ERROR: $*" >&2
    exit 1
}

SOURCE="tdi-ai/src/task_execution.rs"
LIB="tdi-ai/src/lib.rs"
DOC="docs/TDI-8.1-SYMBOLIC-EXECUTION-CONTRACT.md"
HARNESS="tdi-ai/tests/tdi8_symbolic_execution_compile.rs"

for file in "$SOURCE" "$LIB" "$DOC" "$HARNESS"; do
    test -s "$file" || fail "missing symbolic-execution surface: $file"
done

grep -Fq 'pub mod task_execution;' "$LIB" \
    || fail "task_execution is not exported by tdi-ai"
grep -Fq 'pub trait SymbolicTaskAdapter' "$SOURCE" \
    || fail "SymbolicTaskAdapter contract missing"
grep -Fq 'fn associate(&mut self, key_code: u64, value: TaskSymbol)' "$SOURCE" \
    || fail "association adapter surface no longer exposes only stable key code/value"
grep -Fq 'fn payload(&mut self, value: TaskSymbol)' "$SOURCE" \
    || fail "payload adapter surface drifted"
grep -Fq 'fn distractor(&mut self, token: TaskSymbol)' "$SOURCE" \
    || fail "distractor adapter surface drifted"
grep -Fq 'fn query_association(&mut self, key_code: u64)' "$SOURCE" \
    || fail "association query surface drifted or exposes extra metadata"
grep -Fq 'fn query_payload(&mut self, position: u64)' "$SOURCE" \
    || fail "payload query surface drifted"

# Exact target, source index and generator collision class must stay in the
# runner-owned match/record path and must never become extra query arguments.
if grep -Eq 'query_association\([^)]*(target|source_index|collision_class)' "$SOURCE"; then
    fail "association adapter query leaks runner-owned target/provenance metadata"
fi
if grep -Eq 'associate\([^)]*(source_index|collision_class)' "$SOURCE"; then
    fail "association adapter write leaks generator-only provenance metadata"
fi
grep -Fq 'adapter.query_association(key.code())' "$SOURCE" \
    || fail "runner no longer passes only stable key code to association query"
grep -Fq 'collision_class: key.collision_class(),' "$SOURCE" \
    || fail "runner no longer preserves collision class for analysis"
grep -Fq 'source_index,' "$SOURCE" \
    || fail "runner no longer preserves source index for analysis"
grep -Fq 'target,' "$SOURCE" \
    || fail "runner no longer records the generator-owned exact target"
grep -Fq 'adapter.reset().map_err(TaskExecutionError::AdapterReset)?;' "$SOURCE" \
    || fail "per-instance adapter reset missing"
grep -Fq 'ensure_arm(adapter, expected_arm, Some(event_index))?;' "$SOURCE" \
    || fail "mid-instance arm identity guard missing"
grep -Fq 'QueryCountMismatch' "$SOURCE" \
    || fail "query-count consistency guard missing"

for test_name in \
    runner_preserves_exact_event_order_and_targets_without_exposing_labels \
    delayed_copy_exposes_query_position_but_not_exact_target \
    t3_collision_class_and_source_index_remain_runner_owned_metadata \
    adapter_failure_is_typed_with_exact_source_event_index \
    adapter_arm_identity_cannot_drift_mid_instance; do
    grep -Fq "fn ${test_name}()" "$SOURCE" \
        || fail "required symbolic-execution regression test missing: $test_name"
done

cargo test -p tdi-ai --locked 'task_execution::tests'
cargo test -p tdi-ai --locked --test tdi8_symbolic_execution_compile

printf 'TDI-8.1 symbolic event order: VERIFIED\n'
printf 'TDI-8.1 exact target leakage into adapter API: ABSENT\n'
printf 'TDI-8.1 source-index/collision-class leakage into adapter API: ABSENT\n'
printf 'TDI-8.1 per-instance reset and arm identity: VERIFIED\n'
printf 'TDI-8.1 vector encoding/configuration freeze: NOT SELECTED\n'
printf 'TDI-8.1 symbolic execution gate: PASS\n'
