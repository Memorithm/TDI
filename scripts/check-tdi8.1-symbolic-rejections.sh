#!/usr/bin/env bash
set -euo pipefail

fail() {
    echo "TDI-8.1 symbolic rejection ERROR: $*" >&2
    exit 1
}

MODULE="tdi-ai/src/task_rejections.rs"
TEST="tdi-ai/tests/tdi8_symbolic_rejections_compile.rs"
DOC="docs/TDI-8.1-SYMBOLIC-REJECTIONS.md"

for file in "$MODULE" "$TEST" "$DOC"; do
    test -s "$file" || fail "missing symbolic rejection qualification surface: $file"
done

grep -Fq '#[repr(u16)]' "$MODULE" \
    || fail "stable numeric rejection representation missing"
grep -Fq 'pub enum SymbolicRejectionCode' "$MODULE" \
    || fail "typed symbolic rejection code missing"
grep -Fq 'pub struct SymbolicTaskRejectionRecord<E>' "$MODULE" \
    || fail "typed rejection provenance record missing"
grep -Fq 'pub enum RecordedSymbolicTaskOutcome<E>' "$MODULE" \
    || fail "Completed/Rejected outcome split missing"
grep -Fq 'pub fn execute_symbolic_task_recorded<A>' "$MODULE" \
    || fail "recorded symbolic executor entry point missing"
grep -Fq 'match execute_symbolic_task(instance, adapter)' "$MODULE" \
    || fail "recorded wrapper does not preserve qualified executor path"
grep -Fq 'code: SymbolicRejectionCode::from_error(&error)' "$MODULE" \
    || fail "executor rejection is not mapped to stable code"
grep -Fq 'error: TaskExecutionError<E>' "$MODULE" \
    || fail "original typed executor error is not retained"
grep -Fq 'generator_seed: u64' "$MODULE" \
    || fail "generator-seed provenance missing"

# The wrapper may retain family/seed and arm identity but must not inspect exact
# query targets or oracle fields itself.
if grep -E -n '\.(target|oracle)\(\)' "$MODULE" >/tmp/tdi81-symbolic-rejection-leak.log; then
    cat /tmp/tdi81-symbolic-rejection-leak.log >&2
    fail "rejection layer reads evaluator target/oracle"
fi
rm -f /tmp/tdi81-symbolic-rejection-leak.log

# Keep this qualification layer outside the stable tdi-ai API until concrete
# A0/A1/A2/A3 adapters are promoted from preflight binaries.
if grep -Eq 'pub mod task_rejections|mod task_rejections' tdi-ai/src/lib.rs; then
    fail "symbolic rejection layer was prematurely promoted into stable tdi-ai API"
fi

for test_name in \
    rejection_numeric_codes_are_exact_and_stable \
    invalid_prediction_remains_completed_quality_failure \
    adapter_event_rejection_retains_typed_error_and_provenance \
    reset_failure_is_rejection_not_quality_record; do
    grep -Fq "fn $test_name" "$TEST" || fail "missing rejection qualification test: $test_name"
done

for code in 0x0101 0x0102 0x0201 0x0202 0x0301 0x0302; do
    grep -Fq "$code" "$TEST" || fail "stable rejection code test missing: $code"
done

grep -Fq 'TaskPrediction::Invalid' "$DOC" \
    || fail "invalid-prediction quality boundary is undocumented"
grep -Fq 'TDI-8.2 seeds, runner, result payload, confirmation token or final holdout' "$DOC" \
    || fail "TDI-8.2 absence boundary missing"

rustfmt --edition 2024 --check "$MODULE" "$TEST"
cargo clippy -p tdi-ai --test tdi8_symbolic_rejections_compile --locked -- -D warnings
cargo test -p tdi-ai --test tdi8_symbolic_rejections_compile --locked

printf 'TDI-8.1 Completed/Rejected quality boundary: VERIFIED\n'
printf 'TDI-8.1 stable symbolic rejection code coverage: PRESENT\n'
printf 'TDI-8.1 original typed rejection diagnostics: PRESERVED\n'
printf 'TDI-8.1 evaluator-side arm/family/seed provenance: PRESERVED\n'
printf 'TDI-8.1 target/oracle access in rejection layer: ABSENT\n'
printf 'TDI-8.2 executable/token/result surface: ABSENT\n'
printf 'TDI-8.1 symbolic rejection gate: PASS\n'
