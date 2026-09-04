#!/usr/bin/env bash
set -euo pipefail

fail() {
    echo "TDI-9.1 reference evaluator ERROR: $*" >&2
    exit 1
}

bash scripts/check-tdi9.1-reference-policies.sh

MODULE="tdi-ai/src/adaptive_evaluator.rs"
TEST="tdi-ai/tests/tdi9_reference_evaluator_compile.rs"
DOC="docs/TDI-9.1-REFERENCE-EVALUATOR-INTEGRATION.md"

for file in "$MODULE" "$TEST" "$DOC"; do
    test -s "$file" || fail "missing reference-evaluator surface: $file"
done

# Required composition boundaries.
grep -Fq 'pub enum ReferencePolicy' "$MODULE" || fail "typed C0/C1/C2/C3 policy carrier missing"
grep -Fq 'pub fn evaluate_generated_task(' "$MODULE" || fail "integrated evaluator entry point missing"
grep -Fq 'let (policy_task, evaluator) = generated.into_parts();' "$MODULE" \
    || fail "GeneratedTask is not split at the evaluator boundary"
grep -Fq 'ReferenceExecution::new(arm, policy_task, envelope)?' "$MODULE" \
    || fail "live execution is not constructed from policy-visible task only"
grep -Fq 'execution.charge_policy_decision(charge.operations(), charge.memory_bits())?' "$MODULE" \
    || fail "runtime policy decisions are not explicitly charged"
grep -Fq 'let charge = policy.planning_charge();' "$MODULE" \
    || fail "C1 pre-inference planning charge missing"
grep -Fq 'let success = evaluate_stopped(stopped, evaluator)?;' "$MODULE" \
    || fail "post-STOP evaluator boundary missing"
grep -Fq 'DecisionLimitExceeded' "$MODULE" \
    || fail "caller-supplied technical decision guard missing"

# Evaluator target/oracle access must remain delegated to evaluate_stopped after STOP.
if grep -E -n 'evaluator\.(target|oracle)\(\)' "$MODULE" >/tmp/tdi91-evaluator-leak.log; then
    cat /tmp/tdi91-evaluator-leak.log >&2
    fail "integrator directly reads evaluator target/oracle"
fi
rm -f /tmp/tdi91-evaluator-leak.log

# Keep this qualification module outside the stable library API.
if grep -Eq 'pub mod adaptive_evaluator|mod adaptive_evaluator' tdi-ai/src/lib.rs; then
    fail "adaptive evaluator was promoted into stable tdi-ai API during qualification"
fi

for test_name in \
    c0_and_c1_integrate_without_observation_adaptation \
    c2_full_forward_evaluation_uses_post_stop_target_only \
    p3_c2_c3_contrast_survives_complete_evaluator_integration \
    caller_supplied_decision_guard_rejects_without_evaluating; do
    grep -Fq "fn $test_name" "$TEST" || fail "missing evaluator qualification test: $test_name"
done

# No TDI-9.2/final executable or result surface may appear.
if find tdi-ai tdi-bench scripts .github/workflows -type f \
    \( -iname '*tdi9.2*' -o -iname '*tdi9_2*' -o -iname '*tdi9-final*' -o -iname '*tdi9_final*' \) \
    ! -path 'scripts/check-tdi9-bootstrap.sh' \
    ! -path 'scripts/check-tdi9.1-foundation.sh' \
    ! -path 'scripts/check-tdi9.1-task-generators.sh' \
    ! -path 'scripts/check-tdi9.1-reference-execution.sh' \
    ! -path 'scripts/check-tdi9.1-reference-policies.sh' \
    ! -path 'scripts/check-tdi9.1-reference-evaluator.sh' \
    -print -quit | grep -q .; then
    fail "TDI-9.2/final executable surface exists during TDI-9.1"
fi

rustfmt --edition 2024 --check "$MODULE" "$TEST"
cargo clippy -p tdi-ai --test tdi9_reference_evaluator_compile --locked -- -D warnings
cargo test -p tdi-ai --test tdi9_reference_evaluator_compile --locked

printf 'TDI-9.1 task/policy/execution composition: PRESENT\n'
printf 'TDI-9.1 policy decision charging: PRESENT\n'
printf 'TDI-9.1 C1 planning charging: PRESENT\n'
printf 'TDI-9.1 post-STOP evaluator boundary: PRESENT\n'
printf 'TDI-9.1 caller decision guard: FAIL_CLOSED\n'
printf 'TDI-9.2/final executable surface: ABSENT\n'
printf 'TDI-9.1 reference evaluator gate: PASS\n'
