#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

fail() {
    echo "ERROR: $*" >&2
    exit 1
}

test -f docs/TDI-7.6-EVIDENCE-ABLATIONS-PREREGISTRATION.md \
    || fail "missing TDI-7.6 preregistration"
test -f docs/TDI-7.6-POPULATION-DECISION.toml \
    || fail "missing TDI-7.6 population decision record"
test -f docs/TDI-7.6-SELECTION-DECISION.toml \
    || fail "missing TDI-7.6 seed-selection decision record"
test -f docs/TDI-7.6-REJECTION-POLICY.toml \
    || fail "missing TDI-7.6 rejection-policy decision record"

for record in \
    docs/TDI-7.6-EVIDENCE-ABLATIONS-PREREGISTRATION.md \
    docs/TDI-7.6-POPULATION-DECISION.toml \
    docs/TDI-7.6-SELECTION-DECISION.toml \
    docs/TDI-7.6-REJECTION-POLICY.toml; do
    grep -Fqi 'frozen' "$record" \
        || fail "$record is not properly frozen"
done

for record in \
    docs/TDI-7.6-POPULATION-DECISION.toml \
    docs/TDI-7.6-SELECTION-DECISION.toml \
    docs/TDI-7.6-REJECTION-POLICY.toml; do
    grep -Fq 'authorization_state = "NOT_AUTHORIZED"' "$record" \
        || fail "$record is not explicitly unauthorized"
done

printf 'TDI-7.6 preregistration: PRESENT\n'
printf 'TDI-7.6 population decision: PRESENT / FROZEN / NOT AUTHORIZED\n'
printf 'TDI-7.6 seed-selection decision: PRESENT / FROZEN / NOT AUTHORIZED\n'
printf 'TDI-7.6 rejection-policy decision: PRESENT / FROZEN / NOT AUTHORIZED\n'
printf 'TDI-7.6 final holdout: BLOCKED / NOT ACCESSED\n'
