#!/usr/bin/env bash
# TDI-8.1 integrity / pre-confirmatory readiness gate.
# Passes while the configuration freeze remains unresolved_blocking, but
# refuses any TDI-8.2 surface and refuses a false frozen_nonfinal claim.
set -euo pipefail

cd "$(dirname "$0")/.."

fail() {
    echo "TDI-8.1 readiness ERROR: $*" >&2
    exit 1
}

FREEZE="docs/tdi8.1-configuration-freeze.json"
STATUS="docs/TDI-8.1-STATUS.md"
PLAN="docs/TDI-8.1-FREEZE-RESOLUTION-PLAN.md"

for file in \
    "$FREEZE" \
    "$STATUS" \
    "$PLAN" \
    docs/TDI-8.1-A3-ADAPTER-PREFLIGHT.md \
    docs/TDI-8.1-SYMBOLIC-REJECTIONS.md \
    scripts/check-tdi8.1-configuration-freeze.py \
    scripts/check-tdi8.1-a3-adapter.sh \
    scripts/check-tdi8.1-symbolic-rejections.sh \
    tdi-ai/src/task_adapters/a3.rs \
    tdi-ai/src/task_adapters.rs; do
    test -s "$file" || fail "missing required readiness surface: $file"
done

printf '\n===== TDI-8 BOOTSTRAP =====\n'
bash scripts/check-tdi8-bootstrap.sh

printf '\n===== CONFIGURATION FREEZE CONTRACT =====\n'
python3 scripts/check-tdi8.1-configuration-freeze.py

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
    grep -Fq 'tdi8.1-configuration-freeze.json' "$STATUS" \
        || fail "STATUS must point at the configuration freeze contract"
fi

# Require the two currently authorized policy pins to remain present and non-empty.
python3 - "$FREEZE" <<'PY'
import json, sys
data = json.load(open(sys.argv[1], encoding="utf-8"))
required = {
    "a3_event_store_read_cleanup_policy": "tdi8.1-a3-qualified-adapter-v1",
    "closed_rejection_taxonomy": "SymbolicRejectionCode",
}
fields = data["fields"]
for name, marker in required.items():
    record = fields[name]
    if record["status"] != "pinned":
        raise SystemExit(f"{name} must remain pinned once introduced")
    value = record["value"]
    blob = json.dumps(value, sort_keys=True)
    if marker not in blob:
        raise SystemExit(f"{name} pin missing required marker {marker!r}")
print("authorized policy pins: PRESENT")
PY

printf '\n===== TDI-8.2 / TDI-7.2 SURFACE EXCLUSION =====\n'
mapfile -t forbidden < <(
    find tdi-ai tdi-bench scripts docs -type f \
        \( -iname '*tdi8.2*' -o -iname '*tdi8_2*' -o -iname '*tdi8-final*' -o -iname '*tdi8_final*' \) \
        ! -path 'scripts/check-tdi8-bootstrap.sh' \
        ! -path 'scripts/check-tdi8.1-foundation.sh' \
        ! -path 'scripts/check-tdi8.1-readiness.sh' \
        ! -path 'docs/TDI-8.1-STATUS.md' \
        ! -path 'docs/TDI-8.1-FREEZE-RESOLUTION-PLAN.md' \
        ! -path 'docs/tdi8.1-configuration-freeze.json' \
        -print
)
if ((${#forbidden[@]} != 0)); then
    printf 'Unexpected TDI-8.2 surfaces:\n' >&2
    printf '  %s\n' "${forbidden[@]}" >&2
    fail "TDI-8.2 must remain absent during TDI-8.1"
fi

# Scan readiness surfaces only. Do not scan this checker itself: it necessarily
# contains the forbidden TDI-7 token name as the literal pattern used to detect leakage.
if grep -R -n -F 'TDI7_CONFIRM_FINAL_HOLDOUT' \
    docs/TDI-8.1-STATUS.md \
    docs/TDI-8.1-FREEZE-RESOLUTION-PLAN.md \
    docs/TDI-8.1-A3-ADAPTER-PREFLIGHT.md \
    docs/TDI-8.1-SYMBOLIC-REJECTIONS.md \
    docs/tdi8.1-configuration-freeze.json \
    >/tmp/tdi81-readiness-token-scan.log; then
    cat /tmp/tdi81-readiness-token-scan.log >&2
    fail "TDI-8.1 readiness surfaces must not carry the TDI-7.2 confirmation token"
fi
rm -f /tmp/tdi81-readiness-token-scan.log

printf 'TDI-8.1 pinned freeze fields: %s\n' "$pinned_count"
printf 'TDI-8.1 unresolved freeze fields: %s\n' "$unresolved_count"
printf 'TDI-8.1 scientific_status: %s\n' "$scientific_status"
if [[ "$scientific_status" == "frozen_nonfinal" ]]; then
    printf 'TDI-8.1 confirmatory readiness: FREEZE COMPLETE — TDI-8.2 STILL UNAUTHORIZED\n'
else
    printf 'TDI-8.1 confirmatory readiness: NOT READY (freeze incomplete)\n'
fi
printf 'TDI-8.2 executable/token/result surface: ABSENT\n'
printf 'TDI-8.1 readiness integrity gate: PASS\n'
