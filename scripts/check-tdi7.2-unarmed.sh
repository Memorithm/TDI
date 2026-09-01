#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

fail() {
    echo "ERROR: $*" >&2
    exit 1
}

test -f docs/TDI-7.2-ARMING-PROTOCOL.md \
    || fail "missing TDI-7.2 arming protocol"
test -f docs/TDI-7.2-FINAL-HOLDOUT-DECISION.toml \
    || fail "missing TDI-7.2 final population decision record"
test -f docs/TDI-7.2-FINAL-HOLDOUT-SELECTION.toml \
    || fail "missing TDI-7.2 final seed-selection decision record"
test -f docs/TDI-7.2-FINAL-HOLDOUT-REJECTION-POLICY.toml \
    || fail "missing TDI-7.2 final rejection-policy decision record"
test -f tdi-ai/examples/tdi7_arming_decision.rs \
    || fail "missing TDI-7.2 population decision validator"
test -f tdi-ai/examples/tdi7_seed_selection_decision.rs \
    || fail "missing TDI-7.2 seed-selection decision validator"
test -f tdi-ai/examples/tdi7_rejection_policy_decision.rs \
    || fail "missing TDI-7.2 rejection-policy decision validator"

grep -Fq 'Status: **unarmed design contract**' docs/TDI-7.2-ARMING-PROTOCOL.md \
    || fail "TDI-7.2 arming protocol is not explicitly unarmed"
grep -Fq 'A real final-holdout generator/runner must not be added' docs/TDI-7.2-ARMING-PROTOCOL.md \
    || fail "TDI-7.2 runner prohibition is missing"
for record in \
    docs/TDI-7.2-FINAL-HOLDOUT-DECISION.toml \
    docs/TDI-7.2-FINAL-HOLDOUT-SELECTION.toml \
    docs/TDI-7.2-FINAL-HOLDOUT-REJECTION-POLICY.toml; do
    grep -Fq 'authorization_state = "NOT_AUTHORIZED"' "$record" \
        || fail "$record is not explicitly unauthorized"
done

mapfile -t other_surfaces < <(
    find tdi-ai/examples tdi-bench/src/bin scripts -type f \
        \( -iname '*v72*' -o -iname '*tdi7.2*' -o -iname '*tdi7_2*' \) \
        ! -path 'scripts/check-tdi7.2-unarmed.sh' \
        -print
)
if ((${#other_surfaces[@]} != 0)); then
    printf 'Unexpected TDI-7.2 executable surfaces:\n' >&2
    printf '  %s\n' "${other_surfaces[@]}" >&2
    fail "only the reviewed final-holdout runner may exist"
fi

# The arming transition added exactly one reviewed runner; it must refuse to
# run without the human-supplied confirmation variable.
RUNNER="tdi-ai/examples/tdi7_final_holdout.rs"
test -f "$RUNNER" || fail "missing armed TDI-7.2 final-holdout runner"
if env -u TDI7_CONFIRM_FINAL_HOLDOUT \
    cargo run --quiet -p tdi-ai --example tdi7_final_holdout \
    >/tmp/tdi72-runner-refusal.log 2>&1; then
    cat /tmp/tdi72-runner-refusal.log >&2
    fail "final-holdout runner ran without human authorization"
fi
grep -Fq 'BLOCKED: final holdout requires the human-supplied confirmation variable' \
    /tmp/tdi72-runner-refusal.log \
    || fail "runner refusal reason was not explicit"
rm -f /tmp/tdi72-runner-refusal.log

printf 'TDI-7.2 arming contract: PRESENT\n'
printf 'TDI-7.2 population decision record: PRESENT / FROZEN / NOT AUTHORIZED\n'
printf 'TDI-7.2 seed-selection decision record: PRESENT / FROZEN / NOT AUTHORIZED\n'
printf 'TDI-7.2 rejection-policy decision record: PRESENT / FROZEN / NOT AUTHORIZED\n'
printf 'TDI-7.2 final-holdout runner: PRESENT (authorization-only)\n'
printf 'TDI-7.2 final holdout: ARMED / STILL NOT ACCESSED (human token required)\n'
