#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

fail() {
    echo "ERROR: $*" >&2
    exit 1
}

FROZEN_RESULT="artifacts/tdi7_final_holdout_2026-09-01.txt"
RETIRED_RUNNER="tdi-ai/examples/tdi7_final_holdout.rs"

test -f docs/TDI-7.2-ARMING-PROTOCOL.md \
    || fail "missing historical TDI-7.2 arming protocol"
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

# These documents are immutable historical inputs. Their original pre-access
# wording is expected and must not be rewritten after the confirmatory run.
grep -Fq 'Status: **unarmed design contract**' docs/TDI-7.2-ARMING-PROTOCOL.md \
    || fail "historical TDI-7.2 arming protocol drifted"
grep -Fq 'A real final-holdout generator/runner must not be added' docs/TDI-7.2-ARMING-PROTOCOL.md \
    || fail "historical TDI-7.2 runner prohibition is missing"
for record in \
    docs/TDI-7.2-FINAL-HOLDOUT-DECISION.toml \
    docs/TDI-7.2-FINAL-HOLDOUT-SELECTION.toml \
    docs/TDI-7.2-FINAL-HOLDOUT-REJECTION-POLICY.toml; do
    grep -Fq 'authorization_state = "NOT_AUTHORIZED"' "$record" \
        || fail "$record no longer preserves its historical unauthorized state"
done

# The frozen result record is existence-only evidence for this guard. The gate
# deliberately does not inspect, parse or reinterpret the scientific payload.
test -s "$FROZEN_RESULT" \
    || fail "missing frozen TDI-7.2 result record"

# The exact runner used for the accessed experiment remains recoverable from
# Git history. It must not remain executable in the current tree after access.
test ! -e "$RETIRED_RUNNER" \
    || fail "TDI-7.2 final-holdout runner must be retired after frozen access"

mapfile -t rerun_surfaces < <(
    find tdi-ai/examples tdi-bench/src/bin scripts -type f \
        \( -iname '*v72*' -o -iname '*tdi7.2*' -o -iname '*tdi7_2*' \) \
        ! -path 'scripts/check-tdi7.2-unarmed.sh' \
        -print
)
if ((${#rerun_surfaces[@]} != 0)); then
    printf 'Unexpected TDI-7.2 executable rerun surfaces:\n' >&2
    printf '  %s\n' "${rerun_surfaces[@]}" >&2
    fail "TDI-7.2 rerun surface must remain closed after frozen access"
fi

printf 'TDI-7.2 historical arming contract: PRESENT / PRESERVED\n'
printf 'TDI-7.2 population decision record: PRESENT / FROZEN / HISTORICALLY NOT AUTHORIZED\n'
printf 'TDI-7.2 seed-selection decision record: PRESENT / FROZEN / HISTORICALLY NOT AUTHORIZED\n'
printf 'TDI-7.2 rejection-policy decision record: PRESENT / FROZEN / HISTORICALLY NOT AUTHORIZED\n'
printf 'TDI-7.2 frozen result record: PRESENT (payload not inspected by this gate)\n'
printf 'TDI-7.2 final-holdout executable runner: RETIRED / ABSENT\n'
printf 'TDI-7.2 rerun surface: CLOSED\n'
