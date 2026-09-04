#!/usr/bin/env bash
set -euo pipefail

fail() {
    echo "TDI-9.1 task generators ERROR: $*" >&2
    exit 1
}

bash scripts/check-tdi9-bootstrap.sh
bash scripts/check-tdi9.1-foundation.sh

MODULE="tdi-ai/src/adaptive_task_generators.rs"
TEST="tdi-ai/tests/tdi9_adaptive_task_generators_compile.rs"
DOC="docs/TDI-9.1-TASK-GENERATORS.md"

for file in "$MODULE" "$TEST" "$DOC"; do
    test -s "$file" || fail "missing task-generator surface: $file"
done

grep -Fq 'pub enum AdaptiveTaskFamily' "$MODULE" || fail "task-family vocabulary missing"
grep -Fq 'StagedEvidenceAccumulation' "$MODULE" || fail "P1 family missing"
grep -Fq 'VerificationSensitiveInference' "$MODULE" || fail "P2 family missing"
grep -Fq 'RecoverableDeceptiveFork' "$MODULE" || fail "P3 family missing"
grep -Fq 'pub enum DifficultyStratum' "$MODULE" || fail "difficulty-stratum vocabulary missing"
grep -Fq 'Shallow' "$MODULE" || fail "Shallow stratum missing"
grep -Fq 'Intermediate' "$MODULE" || fail "Intermediate stratum missing"
grep -Fq 'Deep' "$MODULE" || fail "Deep stratum missing"
grep -Fq 'pub enum PolicyTask' "$MODULE" || fail "policy task boundary missing"
grep -Fq 'pub struct EvaluatorRecord' "$MODULE" || fail "evaluator-only record missing"
grep -Fq 'pub struct GeneratedTask' "$MODULE" || fail "generated task envelope missing"
grep -Fq 'earliest_decisive_step' "$MODULE" || fail "P1 decisive-prefix oracle missing"
grep -Fq 'ParityConstraint' "$MODULE" || fail "P2 parity constraints missing"
grep -Fq 'EliminateBranch' "$MODULE" || fail "P3 contradiction event missing"
grep -Fq '0u64..64' "$TEST" || fail "bounded multi-seed P1 qualification missing"
grep -Fq 'solutions, vec![target]' "$TEST" || fail "P2 unique-solution qualification missing"

# Hidden evaluator metadata must not become fields of the policy task envelope.
POLICY_BLOCK="$(sed -n '/pub enum PolicyTask {/,/^}/p' "$MODULE")"
printf '%s\n' "$POLICY_BLOCK" | grep -Fq 'seed' && fail "PolicyTask leaks generator seed"
printf '%s\n' "$POLICY_BLOCK" | grep -Fq 'stratum' && fail "PolicyTask leaks hidden difficulty stratum"
printf '%s\n' "$POLICY_BLOCK" | grep -Fq 'target' && fail "PolicyTask leaks evaluator target"

# TDI-9.2 remains nonexistent while task semantics are qualified on non-final domains.
if find tdi-ai tdi-bench scripts .github/workflows -type f \
    \( -iname '*tdi9.2*' -o -iname '*tdi9_2*' -o -iname '*tdi9-final*' -o -iname '*tdi9_final*' \) \
    ! -path 'scripts/check-tdi9-bootstrap.sh' \
    ! -path 'scripts/check-tdi9.1-foundation.sh' \
    ! -path 'scripts/check-tdi9.1-task-generators.sh' \
    -print -quit | grep -q .; then
    fail "TDI-9.2/final executable surface exists during TDI-9.1 task qualification"
fi

# This slice owns generation only; solver/verifier/checkpoint/policy-search
# execution semantics remain separate future TDI-9.1 work.
if grep -R -n -E 'trait .*Solver|struct .*Solver|trait .*Verifier|struct .*Verifier|trait .*PolicySearch|struct .*PolicySearch' \
    "$MODULE" "$TEST" >/tmp/tdi91-task-scope.log; then
    cat /tmp/tdi91-task-scope.log >&2
    fail "task-generator slice silently expanded into execution/search semantics"
fi
rm -f /tmp/tdi91-task-scope.log

rustfmt --edition 2024 --check "$MODULE" "$TEST"
cargo test -p tdi-ai --test tdi9_adaptive_task_generators_compile --locked

printf 'TDI-9.1 P1/P2/P3 deterministic generators: PRESENT\n'
printf 'TDI-9.1 policy/evaluator metadata separation: PRESENT\n'
printf 'TDI-9.1 concrete final sizes/seeds/populations: UNFROZEN\n'
printf 'TDI-9.2/final executable surface: ABSENT\n'
printf 'TDI-9.1 task-generator gate: PASS\n'
