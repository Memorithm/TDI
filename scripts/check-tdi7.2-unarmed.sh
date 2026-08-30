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
test -f tdi-ai/examples/tdi7_arming_decision.rs \
    || fail "missing TDI-7.2 population decision validator"

grep -Fq 'Status: **unarmed design contract**' docs/TDI-7.2-ARMING-PROTOCOL.md \
    || fail "TDI-7.2 arming protocol is not explicitly unarmed"
grep -Fq 'A real final-holdout generator/runner must not be added' docs/TDI-7.2-ARMING-PROTOCOL.md \
    || fail "TDI-7.2 runner prohibition is missing"
grep -Fq 'authorization_state = "NOT_AUTHORIZED"' docs/TDI-7.2-FINAL-HOLDOUT-DECISION.toml \
    || fail "final population decision record is not explicitly unauthorized"

mapfile -t forbidden < <(
    find tdi-ai/examples tdi-bench/src/bin scripts -type f \
        \( -iname '*v72*' -o -iname '*tdi7.2*' -o -iname '*tdi7_2*' \) \
        ! -path 'scripts/check-tdi7.2-unarmed.sh' \
        -print
)

if ((${#forbidden[@]} != 0)); then
    printf 'Unexpected TDI-7.2 executable surfaces:\n' >&2
    printf '  %s\n' "${forbidden[@]}" >&2
    fail "TDI-7.2 must remain unarmed until the population decision and arming transition are separately reviewed"
fi

printf 'TDI-7.2 arming contract: PRESENT\n'
printf 'TDI-7.2 population decision record: PRESENT / NOT AUTHORIZED\n'
printf 'TDI-7.2 executable runner: ABSENT\n'
printf 'TDI-7.2 final holdout: UNARMED / NOT ACCESSED\n'
