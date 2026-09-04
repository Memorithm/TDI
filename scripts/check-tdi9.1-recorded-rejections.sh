#!/usr/bin/env bash
set -euo pipefail

fail() {
    echo "TDI-9.1 recorded-rejection ERROR: $*" >&2
    exit 1
}

SOURCE="tdi-ai/src/adaptive_recording.rs"
TEST="tdi-ai/tests/tdi9_recorded_rejections_compile.rs"
DOC="docs/TDI-9.1-RECORDED-REJECTIONS.md"

for file in "$SOURCE" "$TEST" "$DOC"; do
    test -s "$file" || fail "missing recorded rejection qualification surface: $file"
done

grep -Fq 'pub enum AdaptiveInferenceRejectionCode' "$SOURCE" \
    || fail "adaptive-inference rejection code missing"
grep -Fq 'pub enum ReferenceExecutionRejectionCode' "$SOURCE" \
    || fail "execution rejection code missing"
grep -Fq 'pub enum ReferencePolicyRejectionCode' "$SOURCE" \
    || fail "policy rejection code missing"
grep -Fq 'pub enum ReferenceRejectionCode' "$SOURCE" \
    || fail "top-level evaluator rejection code missing"
grep -Fq 'pub struct ReferenceRejectionRecord' "$SOURCE" \
    || fail "immutable rejection provenance record missing"
grep -Fq 'pub enum ReferenceRecordedOutcome' "$SOURCE" \
    || fail "completed/rejected outcome split missing"
grep -Fq 'pub fn evaluate_generated_task_recorded(' "$SOURCE" \
    || fail "recorded evaluator entry point missing"
grep -Fq 'match evaluate_generated_task(generated, policy, envelope, runtime_decision_limit)' "$SOURCE" \
    || fail "recording layer no longer delegates to qualified evaluator"
grep -Fq 'ReferenceRecordedOutcome::Completed(record)' "$SOURCE" \
    || fail "normal completed result path missing"
grep -Fq 'ReferenceRecordedOutcome::Rejected(ReferenceRejectionRecord' "$SOURCE" \
    || fail "technical rejection recording path missing"
grep -Fq 'stratum: evaluator.stratum()' "$SOURCE" \
    || fail "evaluator-owned stratum provenance missing from rejection record"
grep -Fq 'seed: evaluator.seed()' "$SOURCE" \
    || fail "evaluator-owned seed provenance missing from rejection record"

for forbidden in 'Beneficial' 'Equivalent' 'Harmful' 'success: false' 'success: true'; do
    if grep -Fq "$forbidden" "$SOURCE"; then
        fail "rejection layer contains forbidden scientific reinterpretation marker: $forbidden"
    fi
done

grep -Fq 'decision_guard_exhaustion_is_not_reinterpreted_as_stop_or_failure' "$TEST" \
    || fail "decision-limit rejection regression missing"
grep -Fq 'memory_envelope_exhaustion_remains_nested_execution_rejection' "$TEST" \
    || fail "resource-envelope rejection regression missing"
grep -Fq 'policy_origin_and_inference_payload_are_preserved_losslessly' "$TEST" \
    || fail "policy/inference lossless mapping regression missing"
grep -Fq 'completed_recording_is_bit_identical_to_normal_evaluator_result' "$TEST" \
    || fail "completed-path compatibility regression missing"
grep -Fq 'TDI-9.2 executable' "$DOC" \
    || fail "TDI-9.2 absence boundary missing"

cargo test --locked -p tdi-ai --test tdi9_recorded_rejections_compile

printf 'TDI-9.1 technical rejection taxonomy: LOSSLESS_TYPED\n'
printf 'TDI-9.1 completed/rejected split: EXPLICIT\n'
printf 'TDI-9.1 rejection quality reinterpretation: FORBIDDEN\n'
printf 'TDI-9.1 evaluator provenance on rejection: RECORDED_POST_POLICY\n'
printf 'TDI-9.2 executable/seed/result surface: ABSENT\n'
printf 'TDI-9.1 recorded rejection gate: PASS\n'
