#!/usr/bin/env bash
set -euo pipefail

fail() {
    echo "TDI-8 bootstrap ERROR: $*" >&2
    exit 1
}

PREREG="docs/TDI-8.0-ASSR-PREREGISTRATION.md"
MANIFEST="docs/TDI-8.0-ASSR-PREREGISTRATION.gitblob"
GATE="docs/TDI-8.0-IMPLEMENTATION-GATE.md"

for file in \
    "$PREREG" \
    "$MANIFEST" \
    docs/TDI-8-PROGRAMME.md \
    docs/TDI-8.0-SCOPE.md \
    "$GATE" \
    docs/TDI-8.0-REVIEW-CHECKLIST.md \
    docs/TDI-8.0-STATUS.md \
    docs/TDI-8.0-README.md \
    docs/TDI-8.0-NEXT.md \
    AGENTS.md \
    .github/copilot-instructions.md; do
    test -s "$file" || fail "missing required bootstrap surface: $file"
done

read -r expected_blob expected_path < "$MANIFEST"
[[ "$expected_path" == "$PREREG" ]] || fail "preregistration manifest path drifted"
actual_blob="$(git hash-object "$PREREG")"
[[ "$actual_blob" == "$expected_blob" ]] \
    || fail "TDI-8.0 preregistration drifted: $actual_blob != $expected_blob"

grep -Fq 'Primary contrast:' "$PREREG" \
    || fail "primary contrast declaration missing"
grep -Fq '`A2 recurrent + associative memory` versus `A1 recurrent state only`.' "$PREREG" \
    || fail "A2 vs A1 primary contrast drifted"
grep -Fq '`A3 recurrent + associative memory + VSA workspace` versus' "$PREREG" \
    || fail "A3 vs A2 primary contrast drifted"
grep -Fq 'The primary A1/A2/A3 comparison uses an identical total dynamic-memory budget' "$PREREG" \
    || fail "matched dynamic-memory budget rule drifted"
grep -Fq 'Each hypothesis has exactly nine primary cells' "$PREREG" \
    || fail "nine-cell primary set drifted"
grep -Fq 'The relative statistic is never evaluated with a zero denominator.' "$PREREG" \
    || fail "zero-baseline rule missing"
grep -Fq 'frozen at `delta = 0.02`' "$PREREG" \
    || fail "primary decision margin drifted"
grep -Fq 'Bonferroni allocation of `alpha = 0.05 / 9`' "$PREREG" \
    || fail "family-wise coverage rule drifted"
grep -Fq '1. `Beneficial` iff `L > +delta`;' "$PREREG" \
    || fail "four-way classifier drifted"
grep -Fq '`Beneficial` iff all nine cells are in `{Beneficial, Equivalent}`' "$PREREG" \
    || fail "hypothesis aggregation gate drifted"
grep -Fq 'TDI-8.2 — confirmatory holdout' "$PREREG" \
    || fail "TDI-8.2 human-only boundary missing"
grep -Fq 'Autonomous agents must' "$PREREG" \
    || fail "autonomous holdout prohibition missing"

grep -Fq 'TDI-8.1 may begin only after the TDI-8.0 preregistration is merged' \
    "$GATE" \
    || fail "TDI-8.1 merge gate missing"
grep -Fq 'Each hypothesis retains exactly nine primary task × horizon cells.' "$GATE" \
    || fail "implementation gate lost nine-cell rule"
grep -Fq 'The primary relative-effect margin remains `delta = 0.02`.' "$GATE" \
    || fail "implementation gate lost decision margin"
grep -Fq 'using the frozen Bonferroni allocation `alpha = 0.05 / 9`.' "$GATE" \
    || fail "implementation gate lost family-wise coverage"
grep -Fq 'TDI-8 work must not read, generate, reuse or modify TDI-7.2 final-holdout data' \
    docs/TDI-8-PROGRAMME.md \
    || fail "TDI-7.2 isolation rule missing"
grep -Fq 'TDI-8.0 must be merged and frozen before TDI-8.1 evaluator implementation' AGENTS.md \
    || fail "agent bootstrap does not enforce the TDI-8.0 -> TDI-8.1 gate"

mapfile -t forbidden < <(
    find tdi-ai/examples tdi-bench/src/bin scripts -type f \
        \( -iname '*tdi8.2*' -o -iname '*tdi8_2*' -o -iname '*tdi8-final*' -o -iname '*tdi8_final*' \) \
        ! -path 'scripts/check-tdi8-bootstrap.sh' \
        -print
)
if ((${#forbidden[@]} != 0)); then
    printf 'Unexpected TDI-8.2 executable surfaces:\n' >&2
    printf '  %s\n' "${forbidden[@]}" >&2
    fail "TDI-8.2 must not exist during TDI-8.0"
fi

# Scan the bootstrap surfaces consumed by agents and CI. Do not scan this
# checker itself: it necessarily contains the forbidden TDI-7 token name as
# the literal pattern used to detect leakage.
if grep -R -n -F 'TDI7_CONFIRM_FINAL_HOLDOUT' \
    docs/TDI-8* AGENTS.md .github/copilot-instructions.md .github/workflows/public-ci.yml \
    >/tmp/tdi8-bootstrap-token-scan.log; then
    cat /tmp/tdi8-bootstrap-token-scan.log >&2
    fail "TDI-8 bootstrap must not carry the TDI-7.2 confirmation surface"
fi
rm -f /tmp/tdi8-bootstrap-token-scan.log

printf 'TDI-8.0 preregistration blob: VERIFIED (%s)\n' "$actual_blob"
printf 'TDI-8.0 -> TDI-8.1 implementation gate: PRESENT\n'
printf 'TDI-8 primary verdict rule: PINNED\n'
printf 'TDI-8.2 executable surface: ABSENT\n'
printf 'TDI-7.2 confirmation surface in TDI-8 bootstrap: ABSENT\n'
printf 'TDI-8 bootstrap gate: PASS\n'
