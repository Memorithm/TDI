#!/usr/bin/env bash
set -euo pipefail

fail() {
    echo "TDI-9.1 reference rejection ERROR: $*" >&2
    exit 1
}

bash scripts/check-tdi9.1-reference-evaluator.sh

MODULE="tdi-ai/src/adaptive_rejections.rs"
TEST="tdi-ai/tests/tdi9_reference_rejections_compile.rs"
DOC="docs/TDI-9.1-REFERENCE-REJECTIONS.md"

for file in "$MODULE" "$TEST" "$DOC"; do
    test -s "$file" || fail "missing reference rejection surface: $file"
done

grep -Fq '#[repr(u16)]' "$MODULE" || fail "stable numeric rejection representation missing"
grep -Fq 'pub enum ReferenceRejectionCode' "$MODULE" || fail "typed rejection code missing"
grep -Fq 'pub struct ReferenceRejectionRecord' "$MODULE" || fail "immutable rejection record missing"
grep -Fq 'pub enum ReferenceEvaluationOutcome' "$MODULE" || fail "completed/rejected outcome split missing"
grep -Fq 'pub fn evaluate_generated_task_recorded(' "$MODULE" \
    || fail "recorded evaluator entry point missing"
grep -Fq 'match evaluate_generated_task(generated, policy, envelope, runtime_decision_limit)' "$MODULE" \
    || fail "recorded evaluator does not wrap the qualified compatibility API"
grep -Fq 'code: ReferenceRejectionCode::from_error(error)' "$MODULE" \
    || fail "typed evaluator error is not mapped to a stable code"
grep -Fq 'error: ReferenceEvaluatorError' "$MODULE" \
    || fail "original typed rejection error is not retained"

# Rejection provenance may use evaluator-side stratum/seed after the generated
# task boundary is fixed, but target/oracle access remains forbidden here.
if grep -E -n '\.(target|oracle)\(\)' "$MODULE" >/tmp/tdi91-reference-rejection-leak.log; then
    cat /tmp/tdi91-reference-rejection-leak.log >&2
    fail "rejection layer reads evaluator target/oracle"
fi
rm -f /tmp/tdi91-reference-rejection-leak.log

# Keep qualification-only surfaces outside the stable tdi-ai module API.
if grep -Eq 'pub mod adaptive_rejections|mod adaptive_rejections' tdi-ai/src/lib.rs; then
    fail "adaptive rejection layer was promoted into stable tdi-ai API"
fi

for test_name in \
    recorded_completion_remains_a_normal_evaluation \
    decision_limit_rejection_retains_evaluator_side_provenance \
    resource_exhaustion_is_rejection_not_quality_failure \
    typed_mapping_keeps_execution_policy_and_inference_paths_distinct \
    rejection_numeric_codes_are_exact_and_stable; do
    grep -Fq "fn $test_name" "$TEST" || fail "missing rejection qualification test: $test_name"
done

rustfmt --edition 2024 --check "$MODULE" "$TEST"
cargo clippy -p tdi-ai --test tdi9_reference_rejections_compile --locked -- -D warnings
cargo test -p tdi-ai --test tdi9_reference_rejections_compile --locked

printf 'TDI-9.1 recorded evaluator compatibility API: PRESERVED\n'
printf 'TDI-9.1 typed rejection code coverage: PRESENT\n'
printf 'TDI-9.1 original typed rejection diagnostics: PRESERVED\n'
printf 'TDI-9.1 resource/decision rejection quality reinterpretation: FORBIDDEN\n'
printf 'TDI-9.2/final executable surface: ABSENT\n'
printf 'TDI-9.1 reference rejection gate: PASS\n'
