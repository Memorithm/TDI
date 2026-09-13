#!/usr/bin/env bash
# TDI-11.2 always-on readiness / fail-closed execution gate.
# Passes while the model/observation freeze remains unresolved_blocking,
# but refuses armed execution, invented pins, and forbidden surfaces.
set -euo pipefail

cd "$(dirname "$0")/.."

fail() {
    echo "TDI-11.2 readiness ERROR: $*" >&2
    exit 1
}

FREEZE="docs/tdi11.2-model-observation-freeze.json"
TEMPLATE="docs/tdi11.2-model-observation-freeze.template.json"
PREARM="docs/tdi11.2-prearm.yaml"
LEDGER="docs/TDI-11.2-UNRESOLVED-LEDGER.md"
DIGEST="docs/tdi11.2-model-observation-freeze.sha256"
STATUS="docs/TDI-11.2-STATUS.md"
GATE="docs/TDI-11.2-IMPLEMENTATION-GATE.md"

for file in \
    "$FREEZE" \
    "$TEMPLATE" \
    "$PREARM" \
    "$LEDGER" \
    "$DIGEST" \
    "$STATUS" \
    "$GATE" \
    scripts/check-tdi11.2-readiness.py \
    scripts/check-tdi11.2-model-observation-freeze.py \
    scripts/check-tdi11.2-freeze-schema.py \
    docs/tdi11.2-blocker-evidence-classes.json \
    docs/tdi-freeze-progress-summary.json \
    scripts/emit-tdi-freeze-progress-summary.py \
    scripts/check-tdi11.2-prearm.sh; do
    test -s "$file" || fail "missing required readiness surface: $file"
done

printf '\n===== TDI-11.2 PRE-ARM BOUNDARY =====\n'
bash scripts/check-tdi11.2-prearm.sh

printf '\n===== TDI-11.2 MODEL/OBSERVATION FREEZE =====\n'
python3 scripts/check-tdi11.2-model-observation-freeze.py

printf '\n===== TDI-11.2 TEMPLATE SCHEMA SELF-TEST =====\n'
python3 scripts/check-tdi11.2-freeze-schema.py "$TEMPLATE" --self-test

printf '\n===== TDI-11.2 FAIL-CLOSED READINESS =====\n'
python3 scripts/check-tdi11.2-readiness.py


printf '\n===== FREEZE PROGRESS SUMMARY + LEDGER DIGEST =====\n'
python3 scripts/emit-tdi-freeze-progress-summary.py --check
DIGEST="$(python3 -c 'import hashlib; from pathlib import Path; print(hashlib.sha256(Path("docs/tdi11.2-model-observation-freeze.json").read_bytes()).hexdigest())')"
grep -Fq "$DIGEST" docs/TDI-11.2-UNRESOLVED-LEDGER.md \
    || fail "unresolved ledger lost freeze digest"
grep -Fq "$DIGEST" docs/tdi11.2-model-observation-freeze.sha256 \
    || fail "digest sidecar lost freeze digest"
grep -Fq "$DIGEST" docs/tdi-freeze-progress-summary.json \
    || fail "freeze progress summary must carry verified 11.2 ledger digest"
grep -Fq 'required_evidence_class' docs/tdi11.2-blocker-evidence-classes.json \
    || fail "11.2 evidence-class inventory must declare required_evidence_class entries"
printf 'TDI-11.2 readiness integrity gate: PASS\n'
