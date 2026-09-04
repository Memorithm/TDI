#!/usr/bin/env bash
set -euo pipefail

fail() {
    echo "TDI-9.1 reference policies ERROR: $*" >&2
    exit 1
}

bash scripts/check-tdi9-bootstrap.sh
bash scripts/check-tdi9.1-foundation.sh
bash scripts/check-tdi9.1-task-generators.sh
bash scripts/check-tdi9.1-reference-execution.sh

MODULE="tdi-ai/src/adaptive_policies.rs"
TEST="tdi-ai/tests/tdi9_reference_policies_compile.rs"
DOC="docs/TDI-9.1-REFERENCE-POLICIES.md"

for file in "$MODULE" "$TEST" "$DOC"; do
    test -s "$file" || fail "missing reference-policy surface: $file"
done

# Frozen arm identities and structural information boundaries.
grep -Fq 'pub struct C0FixedPolicy' "$MODULE" || fail "C0 reference policy missing"
grep -Fq 'pub fn decide(self, step_index: u64)' "$MODULE" || fail "fixed/static step-only decision surface missing"
grep -Fq 'pub struct C1StaticPolicy' "$MODULE" || fail "C1 reference policy missing"
grep -Fq 'pub const fn plan(self, family: AdaptiveTaskFamily)' "$MODULE" || fail "C1 family-only planning surface missing"
grep -Fq 'pub struct C2AdaptivePolicy' "$MODULE" || fail "C2 reference policy missing"
grep -Fq 'validate_for_arm(PolicyArm::C2AdaptiveStopping)' "$MODULE" || fail "C2 observation boundary missing"
grep -Fq 'pub struct C3RecoveryPolicy' "$MODULE" || fail "C3 reference policy missing"
grep -Fq 'validate_for_arm(PolicyArm::C3VerificationRecovery)' "$MODULE" || fail "C3 observation boundary missing"
grep -Fq 'Some(VerifierSignal::Violated) if checkpoint_available => InferenceAction::Backtrack' "$MODULE" \
    || fail "C3 verifier-gated BACKTRACK policy missing"
grep -Fq 'UnrecoverableVerifiedViolation' "$MODULE" || fail "C3 terminal violation does not fail closed"

# Every decision carries an explicit charge and adaptive predicates are evaluated
# without short-circuiting so the declared logical cost is path invariant.
grep -Fq 'pub struct PolicyCharge' "$MODULE" || fail "policy charge type missing"
grep -Fq 'pub struct PolicyDecision' "$MODULE" || fail "policy decision type missing"
grep -Fq 'const C2_DECISION_OPS: u64 = 10;' "$MODULE" || fail "C2 logical operation contract drifted"
grep -Fq 'const C3_DECISION_OPS: u64 = 17;' "$MODULE" || fail "C3 logical operation contract drifted"
grep -Fq 'let adaptive_stop = residual_small & delta_small & margin_large;' "$MODULE" \
    || fail "C2/C3 adaptive predicate no longer uses complete evaluation"
grep -Fq 'let cadence_due = verification_threshold_met & on_verification_cadence;' "$MODULE" \
    || fail "C3 cadence predicate no longer uses complete evaluation"

# Policy implementation must not accept evaluator-owned or hidden generator data.
if grep -E -n 'Evaluator(Target|Record|Oracle)|DifficultyStratum|\.target\(\)|\.stratum\(\)|\.seed\(\)' "$MODULE" >/tmp/tdi91-policy-leak.log; then
    cat /tmp/tdi91-policy-leak.log >&2
    fail "reference policy module accepts evaluator-owned metadata"
fi
rm -f /tmp/tdi91-policy-leak.log

# Required qualification scenarios.
for test_name in \
    c0_is_a_fixed_schedule_without_observation_input \
    c1_plan_depends_on_family_identity_only \
    c2_rejects_c3_only_observation_fields \
    c2_adaptive_stop_has_path_invariant_charge \
    c3_verifier_state_machine_has_path_invariant_charge \
    c3_unrecoverable_terminal_violation_fails_closed \
    invalid_policy_configuration_fails_closed \
    p3_c2_fails_while_c3_reference_policy_recovers_with_paid_decisions; do
    grep -Fq "fn $test_name" "$TEST" || fail "missing qualification test: $test_name"
done

# TDI-9.2/final executable material remains forbidden during this tranche.
if find tdi-ai tdi-bench scripts .github/workflows -type f \
    \( -iname '*tdi9.2*' -o -iname '*tdi9_2*' -o -iname '*tdi9-final*' -o -iname '*tdi9_final*' \) \
    ! -path 'scripts/check-tdi9-bootstrap.sh' \
    ! -path 'scripts/check-tdi9.1-foundation.sh' \
    ! -path 'scripts/check-tdi9.1-task-generators.sh' \
    ! -path 'scripts/check-tdi9.1-reference-execution.sh' \
    ! -path 'scripts/check-tdi9.1-reference-policies.sh' \
    -print -quit | grep -q .; then
    fail "TDI-9.2/final executable surface exists during TDI-9.1"
fi

rustfmt --edition 2024 --check "$MODULE" "$TEST"
cargo clippy -p tdi-ai --test tdi9_reference_policies_compile --locked -- -D warnings
cargo test -p tdi-ai --test tdi9_reference_policies_compile --locked

printf 'TDI-9.1 C0 fixed reference policy: PRESENT\n'
printf 'TDI-9.1 C1 family-only static policy: PRESENT\n'
printf 'TDI-9.1 C2 observation-conditioned policy: PRESENT\n'
printf 'TDI-9.1 C3 verification/recovery policy: PRESENT\n'
printf 'TDI-9.1 path-invariant policy accounting: PRESENT\n'
printf 'TDI-9.2/final executable surface: ABSENT\n'
printf 'TDI-9.1 reference policy gate: PASS\n'