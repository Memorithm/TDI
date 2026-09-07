#!/usr/bin/env bash
set -euo pipefail

fail() {
    echo "TDI-11 bootstrap ERROR: $*" >&2
    exit 1
}

PREREG="docs/TDI-11.0-HALLUCINATION-DYNAMICS-PREREGISTRATION.md"
MANIFEST="docs/TDI-11.0-HALLUCINATION-DYNAMICS-PREREGISTRATION.gitblob"
PROGRAMME="docs/TDI-11-PROGRAMME.md"
SCOPE="docs/TDI-11.0-HALLUCINATION-DYNAMICS-SCOPE.md"
GATE="docs/TDI-11.0-IMPLEMENTATION-GATE.md"
STATUS="docs/TDI-11.0-STATUS.md"

for file in \
    "$PREREG" \
    "$MANIFEST" \
    "$PROGRAMME" \
    "$SCOPE" \
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
    || fail "TDI-11.0 preregistration drifted: $actual_blob != $expected_blob"

# Frozen operational phenomenon and exact scoring boundary.
grep -Fq 'UNSUPPORTED = TRUE_HIDDEN_UNSUPPORTED OR CONTRADICTED OR NONEXISTENT' "$PREREG" \
    || fail "primary unsupported mapping drifted"
grep -Fq 'ASSERT <subject_id> <relation_id> <object_id>' "$PREREG" \
    || fail "structured ASSERT grammar drifted"
grep -Fq 'The first controlled-world evaluator must not use an LLM judge' "$PREREG" \
    || fail "primary no-LLM-judge boundary drifted"
grep -Fq '`4 task families × 3 strata = 12 primary cells`.' "$PREREG" \
    || fail "12-cell primary battery drifted"

# Primary task/control ladder.
for marker in \
    '### F1 — explicit support' \
    '### F2 — derived support' \
    '### F3 — hidden-truth insufficiency' \
    '### F4 — contradiction / override stress' \
    '### B0 — fixed inference' \
    '### B1 — static verification' \
    '### B2 — risk-gated decision' \
    '### B3 — adaptive recovery'; do
    grep -Fq "$marker" "$PREREG" || fail "missing frozen marker: $marker"
done

# Prospective timing and ordered primary decision.
grep -Fq 'A post-hoc detector may be reported but cannot satisfy H11-A.' "$PREREG" \
    || fail "prospective H11-A boundary drifted"
grep -Fq '2. establish coverage non-inferiority;' "$PREREG" \
    || fail "coverage-first control gate drifted"
grep -Fq '3. establish task-success non-inferiority;' "$PREREG" \
    || fail "task-success control gate drifted"
grep -Fq '4. only then test for a material reduction in unsupported emissions;' "$PREREG" \
    || fail "unsupported-risk control gate drifted"

# Hidden-truth leakage and stage protection.
grep -Fq 'The controller must never receive complete-world hidden truth' "$PREREG" \
    || fail "hidden-truth controller boundary missing"
grep -Fq 'No final dataset, final seed list, result payload, or runnable final-confirmation surface may exist during TDI-11.0.' "$PREREG" \
    || fail "TDI-11 final-surface prohibition missing"
grep -Fq '## TDI-11.x bootstrap and stage gate' AGENTS.md \
    || fail "root agent contract lacks TDI-11 stage gate"
grep -Fq 'For TDI-11.x work' .github/copilot-instructions.md \
    || fail "Copilot contract lacks TDI-11 stage gate"

# Executable final surfaces must not exist during TDI-11.0/11.1 bootstrap.
mapfile -t forbidden < <(
    find tdi-ai tdi-bench scripts .github/workflows -type f \
        \( -iname '*tdi11-final*' -o -iname '*tdi11_final*' -o -iname '*tdi11-confirm*' -o -iname '*tdi11_confirm*' -o -iname '*tdi11.2-final*' \) \
        ! -path 'scripts/check-tdi11-bootstrap.sh' \
        -print
)
if ((${#forbidden[@]} != 0)); then
    printf 'Unexpected TDI-11 final/confirmation executable surfaces:\n' >&2
    printf '  %s\n' "${forbidden[@]}" >&2
    fail "TDI-11 final confirmation must not exist at this stage"
fi

# Preserve earlier independent gates.
if test -f scripts/check-tdi8-bootstrap.sh; then
    bash scripts/check-tdi8-bootstrap.sh >/tmp/tdi11-tdi8-bootstrap.log
    rm -f /tmp/tdi11-tdi8-bootstrap.log
fi
if test -f scripts/check-tdi9-bootstrap.sh; then
    bash scripts/check-tdi9-bootstrap.sh >/tmp/tdi11-tdi9-bootstrap.log
    rm -f /tmp/tdi11-tdi9-bootstrap.log
fi

printf 'TDI-11.0 preregistration blob: VERIFIED (%s)\n' "$actual_blob"
printf 'TDI-11 exact controlled-world scorer boundary: PINNED\n'
printf 'TDI-11 F1/F2/F3/F4 × 3-strata primary battery: PINNED\n'
printf 'TDI-11 B0/B1/B2/B3 control ladder: PINNED\n'
printf 'TDI-11 prospective timing and coverage/utility-first verdict: PINNED\n'
printf 'TDI-11 hidden-truth leakage guard: PRESENT\n'
printf 'TDI-11 final confirmation surface: ABSENT\n'
printf 'TDI-8/TDI-9 bootstrap compatibility: PASS\n'
printf 'TDI-11 bootstrap gate: PASS\n'
