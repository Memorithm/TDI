#!/usr/bin/env bash
set -euo pipefail

fail() {
    echo "TDI-9 bootstrap ERROR: $*" >&2
    exit 1
}

PREREG="docs/TDI-9.0-ADAPTIVE-INFERENCE-PREREGISTRATION.md"
MANIFEST="docs/TDI-9.0-ADAPTIVE-INFERENCE-PREREGISTRATION.gitblob"
PROGRAMME="docs/TDI-9-PROGRAMME.md"
GATE="docs/TDI-9.0-IMPLEMENTATION-GATE.md"
STATUS="docs/TDI-9.0-STATUS.md"

for file in \
    "$PREREG" \
    "$MANIFEST" \
    "$PROGRAMME" \
    "$GATE" \
    "$STATUS" \
    AGENTS.md \
    .github/copilot-instructions.md; do
    test -s "$file" || fail "missing required bootstrap surface: $file"
done

read -r expected_blob expected_path < "$MANIFEST"
[[ "$expected_path" == "$PREREG" ]] || fail "preregistration manifest path drifted"
actual_blob="$(git hash-object "$PREREG")"
[[ "$actual_blob" == "$expected_blob" ]] \
    || fail "TDI-9.0 preregistration drifted: $actual_blob != $expected_blob"

# Frozen scientific ladder and primary contrasts.
grep -Fq '`C2 observation-conditioned CONTINUE/STOP` versus `C0 fixed compute`.' "$PREREG" \
    || fail "H9-A C2 vs C0 contrast drifted"
grep -Fq '`C3 CONTINUE/VERIFY/BACKTRACK/STOP` versus' "$PREREG" \
    || fail "H9-B C3 vs C2 contrast drifted"
grep -Fq 'Each primary hypothesis therefore has exactly nine primary cells:' "$PREREG" \
    || fail "nine-cell primary set drifted"
grep -Fq '`delta_q = 0.02`' "$PREREG" \
    || fail "quality non-inferiority margin drifted"
grep -Fq '`delta_k = 0.05`' "$PREREG" \
    || fail "compute materiality margin drifted"
grep -Fq '`alpha = 0.05 / 18`.' "$PREREG" \
    || fail "18-interval Bonferroni reference drifted"
grep -Fq '1. `Harmful` iff `L_Q > +delta_q`;' "$PREREG" \
    || fail "quality-first primary classifier drifted"
grep -Fq '`Beneficial` iff all nine cells are in `{Beneficial, Equivalent}`' "$PREREG" \
    || fail "nine-cell hypothesis aggregation drifted"

# Agent-first confirmation contract.
grep -Fq 'TDI-9 uses no human confirmation token.' "$PREREG" \
    || fail "autonomous TDI-9 confirmation rule missing"
grep -Fq 'public randomness beacon or equivalent immutable external entropy source' "$PREREG" \
    || fail "future-derived entropy rule missing"
grep -Fq 'Agents and CI must not select another value,' "$PREREG" \
    || fail "no-retry future entropy rule missing"
grep -Fq 'TDI-9.1 may begin only after this TDI-9.0 preregistration is merged' "$PREREG" \
    || fail "TDI-9.0 -> TDI-9.1 gate missing"
grep -Fq 'TDI-9 is a separately preregistered exception' AGENTS.md \
    || fail "root agent contract lacks TDI-9 autonomous exception"
grep -Fq 'TDI-7.2 and TDI-8.2 retain their existing human-only/forbidden-agent boundaries.' AGENTS.md \
    || fail "legacy confirmatory protections were weakened"
grep -Fq 'TDI-9 is a separately preregistered exception' .github/copilot-instructions.md \
    || fail "copilot contract lacks TDI-9 autonomous exception"

# TDI-9.0 may contain documentation about TDI-9.2, but no executable/final
# surface may exist yet.
mapfile -t forbidden < <(
    find tdi-ai tdi-bench scripts .github/workflows -type f \
        \( -iname '*tdi9.2*' -o -iname '*tdi9_2*' -o -iname '*tdi9-final*' -o -iname '*tdi9_final*' -o -iname '*tdi92*' \) \
        ! -path 'scripts/check-tdi9-bootstrap.sh' \
        -print
)
if ((${#forbidden[@]} != 0)); then
    printf 'Unexpected TDI-9.2 executable/code surfaces:\n' >&2
    printf '  %s\n' "${forbidden[@]}" >&2
    fail "TDI-9.2 must not exist during TDI-9.0"
fi

# Human confirmation tokens are intentionally absent from TDI-9. This scan is
# narrowly scoped so historical TDI-7 contracts remain untouched.
if grep -R -n -E 'TDI9_(CONFIRM|FULL|FINAL_TOKEN|HUMAN_TOKEN)' \
    docs/TDI-9* AGENTS.md .github/copilot-instructions.md .github/workflows \
    >/tmp/tdi9-token-scan.log; then
    cat /tmp/tdi9-token-scan.log >&2
    fail "TDI-9 must not introduce a human/full-run confirmation token"
fi
rm -f /tmp/tdi9-token-scan.log

# Preserve the independently frozen earlier-series gate.
if test -f scripts/check-tdi8-bootstrap.sh; then
    bash scripts/check-tdi8-bootstrap.sh >/tmp/tdi9-tdi8-bootstrap.log
    rm -f /tmp/tdi9-tdi8-bootstrap.log
fi

printf 'TDI-9.0 preregistration blob: VERIFIED (%s)\n' "$actual_blob"
printf 'TDI-9.0 -> TDI-9.1 implementation gate: PRESENT\n'
printf 'TDI-9 policy ladder and primary contrasts: PINNED\n'
printf 'TDI-9 quality/compute primary rule: PINNED\n'
printf 'TDI-9 autonomous future-entropy contract: PRESENT\n'
printf 'TDI-9 human confirmation token: ABSENT\n'
printf 'TDI-9.2 executable/final surface: ABSENT\n'
printf 'TDI-8 bootstrap compatibility: PASS\n'
printf 'TDI-9 bootstrap gate: PASS\n'
