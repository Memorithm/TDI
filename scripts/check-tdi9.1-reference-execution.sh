#!/usr/bin/env bash
set -euo pipefail

fail() {
    echo "TDI-9.1 reference execution ERROR: $*" >&2
    exit 1
}

bash scripts/check-tdi9-bootstrap.sh
bash scripts/check-tdi9.1-foundation.sh
bash scripts/check-tdi9.1-task-generators.sh

MODULE="tdi-ai/src/adaptive_execution.rs"
TEST="tdi-ai/tests/tdi9_reference_execution_compile.rs"
DOC="docs/TDI-9.1-REFERENCE-EXECUTION.md"

for file in "$MODULE" "$TEST" "$DOC"; do
    test -s "$file" || fail "missing reference-execution surface: $file"
done

grep -Fq 'pub struct ReferenceExecution' "$MODULE" || fail "executor type missing"
grep -Fq 'pub fn continue_step' "$MODULE" || fail "CONTINUE execution missing"
grep -Fq 'pub fn verify' "$MODULE" || fail "VERIFY execution missing"
grep -Fq 'pub fn backtrack' "$MODULE" || fail "BACKTRACK execution missing"
grep -Fq 'pub fn stop' "$MODULE" || fail "STOP execution missing"
grep -Fq 'pub fn evaluate_stopped' "$MODULE" || fail "post-STOP evaluator boundary missing"
grep -Fq 'ComputeComponent::Replay' "$MODULE" || fail "replay accounting missing"
grep -Fq 'CheckpointTraffic' "$MODULE" || fail "checkpoint byte traffic missing"
grep -Fq 'P3_CHECKPOINT_BYTES' "$MODULE" || fail "canonical checkpoint size missing"
grep -Fq 'BacktrackRequiresViolation' "$MODULE" || fail "verifier-gated backtrack missing"
grep -Fq 'charge_policy_decision' "$MODULE" || fail "future policy accounting hook missing"

# Evaluator target access is permitted only in the post-STOP scoring function.
TARGET_CALLS=$(grep -Fc 'evaluator.target()' "$MODULE" || true)
[[ "$TARGET_CALLS" == "1" ]] || fail "expected exactly one post-STOP evaluator.target() access, found $TARGET_CALLS"

# Do not hand a future policy the complete sequential task through the executor.
if grep -Fq 'pub fn task(' "$MODULE"; then
    fail "live executor exposes complete PolicyTask"
fi

# No policy implementation belongs in this tranche.
if grep -E -n 'struct (C0|C1|C2|C3).*Policy|trait .*Policy|fn choose_action' "$MODULE" "$TEST" >/tmp/tdi91-reference-policy-scope.log; then
    cat /tmp/tdi91-reference-policy-scope.log >&2
    fail "reference execution silently expanded into policy-choice logic"
fi
rm -f /tmp/tdi91-reference-policy-scope.log

# TDI-9.2 remains nonexistent while TDI-9.1 is active.
if find tdi-ai tdi-bench scripts .github/workflows -type f \
    \( -iname '*tdi9.2*' -o -iname '*tdi9_2*' -o -iname '*tdi9-final*' -o -iname '*tdi9_final*' \) \
    ! -path 'scripts/check-tdi9-bootstrap.sh' \
    ! -path 'scripts/check-tdi9.1-foundation.sh' \
    ! -path 'scripts/check-tdi9.1-task-generators.sh' \
    ! -path 'scripts/check-tdi9.1-reference-execution.sh' \
    -print -quit | grep -q .; then
    fail "TDI-9.2/final executable surface exists during TDI-9.1"
fi

rustfmt --edition 2024 --check "$MODULE" "$TEST"
cargo clippy -p tdi-ai --test tdi9_reference_execution_compile --locked -- -D warnings
cargo test -p tdi-ai --test tdi9_reference_execution_compile --locked

printf 'TDI-9.1 deterministic solver transitions: PRESENT\n'
printf 'TDI-9.1 independent verifier: PRESENT\n'
printf 'TDI-9.1 checkpoint store/restore bytes: PRESENT\n'
printf 'TDI-9.1 replay operation accounting: PRESENT\n'
printf 'TDI-9.1 post-STOP evaluator boundary: PRESENT\n'
printf 'TDI-9.2/final executable surface: ABSENT\n'
printf 'TDI-9.1 reference execution gate: PASS\n'
