#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

fail() {
    echo "ERROR: $*" >&2
    exit 1
}

test -f docs/TDI-7.5-FLAT-SEMANTIC-PREREGISTRATION.md \
    || fail "missing TDI-7.5 preregistration"
test -f docs/TDI-7.5-POPULATION-DECISION.toml \
    || fail "missing TDI-7.5 population decision record"
test -f docs/TDI-7.5-SELECTION-DECISION.toml \
    || fail "missing TDI-7.5 seed-selection decision record"
test -f docs/TDI-7.5-REJECTION-POLICY.toml \
    || fail "missing TDI-7.5 rejection-policy decision record"

for record in \
    docs/TDI-7.5-FLAT-SEMANTIC-PREREGISTRATION.md \
    docs/TDI-7.5-POPULATION-DECISION.toml \
    docs/TDI-7.5-SELECTION-DECISION.toml \
    docs/TDI-7.5-REJECTION-POLICY.toml; do
    grep -Fqi 'frozen' "$record" \
        || fail "$record is not properly frozen"
done

for record in \
    docs/TDI-7.5-POPULATION-DECISION.toml \
    docs/TDI-7.5-SELECTION-DECISION.toml \
    docs/TDI-7.5-REJECTION-POLICY.toml; do
    grep -Fq 'authorization_state = "NOT_AUTHORIZED"' "$record" \
        || fail "$record is not explicitly unauthorized"
done

printf 'TDI-7.5 preregistration: PRESENT\n'
printf 'TDI-7.5 population decision: PRESENT / FROZEN / NOT AUTHORIZED\n'
printf 'TDI-7.5 seed-selection decision: PRESENT / FROZEN / NOT AUTHORIZED\n'
printf 'TDI-7.5 rejection-policy decision: PRESENT / FROZEN / NOT AUTHORIZED\n'
printf 'TDI-7.5 final holdout: BLOCKED / NOT ACCESSED\n'
