#!/usr/bin/env bash
# TDI-9.1 integrity / pre-confirmatory readiness gate.
# Passes while the configuration freeze remains unresolved_blocking, but
# refuses any TDI-9.2 surface and refuses a false frozen_nonfinal claim.
set -euo pipefail

cd "$(dirname "$0")/.."

fail() {
    echo "TDI-9.1 readiness ERROR: $*" >&2
    exit 1
}

FREEZE="docs/tdi9.1-configuration-freeze.json"
STATUS="docs/TDI-9.1-STATUS.md"
PLAN="docs/TDI-9.1-FREEZE-RESOLUTION-PLAN.md"

for file in \
    "$FREEZE" \
    "$STATUS" \
    "$PLAN" \
    docs/tdi9.1-blocker-evidence-classes.json \
    docs/tdi-freeze-progress-summary.json \
    docs/TDI-9.1-REFERENCE-REJECTIONS.md \
    docs/TDI-9.1-REFERENCE-POLICIES.md \
    scripts/check-tdi9.1-configuration-freeze.py \
    scripts/check-tdi9.1-reference-rejections.sh \
    tdi-ai/src/adaptive_inference.rs \
    tdi-ai/src/adaptive_rejections.rs; do
    test -s "$file" || fail "missing required readiness surface: $file"
done

printf '\n===== TDI-9 BOOTSTRAP =====\n'
bash scripts/check-tdi9-bootstrap.sh

printf '\n===== CONFIGURATION FREEZE CONTRACT =====\n'
python3 scripts/check-tdi9.1-configuration-freeze.py

scientific_status="$(python3 - "$FREEZE" <<'PY'
import json, sys
data = json.load(open(sys.argv[1], encoding="utf-8"))
print(data["scientific_status"])
PY
)"

pinned_count="$(python3 - "$FREEZE" <<'PY'
import json, sys
data = json.load(open(sys.argv[1], encoding="utf-8"))
print(sum(1 for r in data["fields"].values() if r["status"] == "pinned"))
PY
)"

unresolved_count="$(python3 - "$FREEZE" <<'PY'
import json, sys
data = json.load(open(sys.argv[1], encoding="utf-8"))
print(sum(1 for r in data["fields"].values() if r["status"] == "unresolved_blocking"))
PY
)"

if [[ "$scientific_status" == "frozen_nonfinal" && "$unresolved_count" != "0" ]]; then
    fail "scientific_status is frozen_nonfinal while unresolved fields remain"
fi

if [[ "$scientific_status" == "unresolved_blocking" ]]; then
    grep -Fq 'unresolved_blocking' "$STATUS" \
        || fail "STATUS must still declare unresolved_blocking while the freeze is incomplete"
    grep -Fq 'tdi9.1-configuration-freeze.json' "$STATUS" \
        || fail "STATUS must point at the configuration freeze contract"
fi

# Require the authorized rejection-taxonomy pin. All other fields stay unresolved.
python3 - "$FREEZE" <<'PY'
import json, sys
data = json.load(open(sys.argv[1], encoding="utf-8"))
required = {
    "closed_rejection_taxonomy": "ReferenceRejectionCode",
}
fields = data["fields"]
for name, marker in required.items():
    record = fields[name]
    if record["status"] != "pinned":
        raise SystemExit(f"{name} must remain pinned once introduced")
    blob = json.dumps(record["value"], sort_keys=True)
    if marker not in blob:
        raise SystemExit(f"{name} pin missing required marker {marker!r}")
unauthorized = [
    name
    for name, record in fields.items()
    if name not in required and record["status"] != "unresolved_blocking"
]
if unauthorized:
    raise SystemExit(
        "unauthorized freeze pins (evidence required before pin): "
        + ", ".join(sorted(unauthorized))
    )
print("authorized policy pins: PRESENT")
print("unauthorized fields remain unresolved_blocking: YES")
PY

if (( pinned_count < 1 )); then
    fail "pinned-count floor is 1; found $pinned_count"
fi

# Cross-check STATUS-reported N/M pinned against freeze JSON (fail-closed).
python3 - "$FREEZE" "$STATUS" "$pinned_count" <<'PY'
import json, re, sys
freeze_path, status_path, pinned_s = sys.argv[1], sys.argv[2], sys.argv[3]
data = json.load(open(freeze_path, encoding="utf-8"))
fields = data["fields"]
json_pinned = sum(1 for r in fields.values() if r["status"] == "pinned")
json_total = len(fields)
pinned = int(pinned_s)
if pinned != json_pinned:
    raise SystemExit(f"internal pinned_count mismatch: shell={pinned} json={json_pinned}")
status = open(status_path, encoding="utf-8").read()
matches = re.findall(r"(\d+)/(\d+)\s+pinned", status)
if not matches:
    raise SystemExit("STATUS must report an N/M pinned freeze-progress count")
# Require every reported N/M to agree with the freeze JSON.
for n_s, m_s in matches:
    n, m = int(n_s), int(m_s)
    if n != json_pinned or m != json_total:
        raise SystemExit(
            f"STATUS pin-count {n}/{m} disagrees with freeze JSON "
            f"{json_pinned}/{json_total}"
        )
print(
    f"TDI-9.1 STATUS<->freeze pin-count cross-check: "
    f"{json_pinned}/{json_total} OK ({len(matches)} STATUS mention(s))"
)
PY
grep -Fq 'permitted_observation_vector' "$PLAN" \
    || fail "resolution plan must still list permitted_observation_vector"
grep -Fq 'TDI-9.3 boundary (non-pinning)' "$STATUS" \
    || fail "STATUS must declare the TDI-9.3 non-pinning boundary"
grep -Fq 'does **not** pin any TDI-9.1 freeze field' "$STATUS" \
    || fail "STATUS must state that TDI-9.3 does not pin TDI-9.1 freeze fields"
grep -Fq '## Relationship to TDI-9.1' docs/TDI-9.3-BOOLEAN-POLICY-SYNTHESIS.md \
    || fail "TDI-9.3 doc must keep the Relationship to TDI-9.1 boundary section"
grep -Fq '`tdi9_2_execution_authorized`: **false**' "$STATUS" \
    || fail "STATUS must keep tdi9_2_execution_authorized hard-false"

# Refuse silent STATUS upgrades while the freeze remains incomplete.
if [[ "$scientific_status" == "unresolved_blocking" ]]; then
    if grep -E '^- Status:[[:space:]]+\*\*(frozen_nonfinal|frozen|complete|authorized)' "$STATUS"; then
        fail "STATUS silently upgraded while scientific_status is unresolved_blocking"
    fi
    if grep -Fq 'scientific_status` is `frozen_nonfinal' "$STATUS" \
        || grep -Fq 'scientific_status` remains `frozen_nonfinal' "$STATUS"; then
        fail "STATUS claims frozen_nonfinal while the freeze is still unresolved_blocking"
    fi
    if grep -Eqi 'TDI-9\.2[^[:space:]]* (is |now )?(authorized|ready|armed)' "$STATUS"; then
        fail "STATUS must not claim TDI-9.2 authorization while freeze is incomplete"
    fi
fi

printf '\n===== TDI-9.2 / HOLDOUT SURFACE EXCLUSION =====\n'
mapfile -t forbidden < <(
    find tdi-ai tdi-bench scripts docs .github/workflows -type f \
        \( -iname '*tdi9.2*' -o -iname '*tdi9_2*' -o -iname '*tdi9-final*' -o -iname '*tdi9_final*' \) \
        ! -path 'scripts/check-tdi9-bootstrap.sh' \
        ! -path 'scripts/check-tdi9.1-foundation.sh' \
        ! -path 'scripts/check-tdi9.1-readiness.sh' \
        ! -path 'scripts/check-tdi9.1-configuration-freeze.py' \
        ! -path 'docs/TDI-9.1-STATUS.md' \
        ! -path 'docs/TDI-9.1-FREEZE-RESOLUTION-PLAN.md' \
        ! -path 'docs/tdi9.1-configuration-freeze.json' \
        -print
)
if ((${#forbidden[@]} != 0)); then
    printf 'Unexpected TDI-9.2 surfaces:\n' >&2
    printf '  %s\n' "${forbidden[@]}" >&2
    fail "TDI-9.2 must remain absent during TDI-9.1"
fi

if grep -R -n -E 'TDI9_(CONFIRM|FULL|FINAL_TOKEN|HUMAN_TOKEN)' \
    docs/TDI-9.1-STATUS.md \
    docs/TDI-9.1-FREEZE-RESOLUTION-PLAN.md \
    docs/TDI-9.1-REFERENCE-REJECTIONS.md \
    docs/tdi9.1-configuration-freeze.json \
    >/tmp/tdi91-readiness-token-scan.log; then
    cat /tmp/tdi91-readiness-token-scan.log >&2
    fail "TDI-9.1 readiness surfaces must not carry a human confirmation token"
fi
rm -f /tmp/tdi91-readiness-token-scan.log

if grep -R -n -F 'TDI7_CONFIRM_FINAL_HOLDOUT' \
    docs/TDI-9.1-STATUS.md \
    docs/TDI-9.1-FREEZE-RESOLUTION-PLAN.md \
    docs/TDI-9.1-REFERENCE-REJECTIONS.md \
    docs/tdi9.1-configuration-freeze.json \
    >/tmp/tdi91-readiness-t72-scan.log; then
    cat /tmp/tdi91-readiness-t72-scan.log >&2
    fail "TDI-9.1 readiness surfaces must not carry the TDI-7.2 confirmation token"
fi
rm -f /tmp/tdi91-readiness-t72-scan.log

printf 'TDI-9.1 pinned freeze fields: %s\n' "$pinned_count"
printf 'TDI-9.1 unresolved freeze fields: %s\n' "$unresolved_count"
printf 'TDI-9.1 scientific_status: %s\n' "$scientific_status"
if [[ "$scientific_status" == "frozen_nonfinal" ]]; then
    printf 'TDI-9.1 confirmatory readiness: FREEZE COMPLETE — TDI-9.2 STILL UNAUTHORIZED\n'
else
    printf 'TDI-9.1 confirmatory readiness: NOT READY (freeze incomplete)\n'
fi
printf 'TDI-9.2 executable/token/result surface: ABSENT\n'

printf '\n===== FREEZE PROGRESS SUMMARY + EVIDENCE-CLASS INVENTORY =====\n'
test -s docs/tdi9.1-blocker-evidence-classes.json || fail "missing blocker evidence-class inventory"
test -s docs/tdi-freeze-progress-summary.json || fail "missing freeze progress summary"
python3 scripts/emit-tdi-freeze-progress-summary.py --check

grep -Fq 'docs/TDI-9.3-BOOLEAN-POLICY-SYNTHESIS.md' docs/tdi9.1-blocker-evidence-classes.json \
    || fail "9.1 evidence-class inventory must list TDI-9.3 as a non-authorizing surface"
grep -Fq 'non_authorizing_rule' docs/tdi9.1-blocker-evidence-classes.json \
    || fail "9.1 evidence-class inventory must declare non_authorizing_rule"
grep -Fq 'experimental_observation_subset_selection' docs/tdi9.1-blocker-evidence-classes.json \
    || fail "9.1 inventory must keep permitted_observation_vector evidence class"
grep -Fq 'required_evidence_class' docs/tdi9.1-blocker-evidence-classes.json \
    || fail "evidence-class inventory must declare required_evidence_class entries"
printf 'TDI-9.1 readiness integrity gate: PASS\n'
