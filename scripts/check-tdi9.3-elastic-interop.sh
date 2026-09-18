#!/usr/bin/env bash
# Deterministic, non-final TDI-9.3 C3 representation export for ElasticXxx.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"
fail() { echo "TDI-9.3 ElasticXxx interop ERROR: $*" >&2; exit 1; }
FIXTURE="interop/elasticxxx/tdi9.3-c3-carrier-v1.tsv"
test -s "$FIXTURE" || fail "missing versioned interop fixture"
test -s docs/TDI-9.3-ELASTICXXX-INTEROP.md || fail "missing interop contract doc"
grep -Fq 'non-final-representation-only' "$FIXTURE" || fail "fixture lost non-final claim boundary"
grep -Fq 'missing_predicate_semantics=not-represented-by-TDI-binary-carrier' "$FIXTURE" \
  || fail "fixture must not invent TDI Unknown semantics"
grep -Fq 'does not authorize TDI-9.1 or TDI-9.2' docs/TDI-9.3-ELASTICXXX-INTEROP.md \
  || fail "interop doc lost non-authorizing boundary"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT
cargo run --quiet --locked -p tdi-ai --features experimental --example boolean_c3_elastic_interop \
  > "$TMP/generated.tsv" || fail "interop exporter failed"
cmp "$FIXTURE" "$TMP/generated.tsv" || fail "checked-in interop fixture differs from exporter"
python3 - "$FIXTURE" <<'PY'
import sys
from pathlib import Path
path = Path(sys.argv[1])
rows = [line.rstrip("\n").split("\t") for line in path.read_text().splitlines() if not line.startswith("#")]
if len(rows) != 512:
    raise SystemExit(f"expected 512 rows, got {len(rows)}")
counts = {"ACTION": 0, "UNRECOVERABLE": 0, "INVALID": 0}
actions = {"CONTINUE": 0, "VERIFY": 0, "BACKTRACK": 0, "STOP": 0}
for expected_index, fields in enumerate(rows):
    if len(fields) != 4:
        raise SystemExit(f"row {expected_index}: expected 4 fields")
    row, bits, classification, action = fields
    if int(row) != expected_index or len(bits) != 9 or set(bits) - {"0", "1"}:
        raise SystemExit(f"row {expected_index}: malformed identity/bits")
    if classification not in counts:
        raise SystemExit(f"row {expected_index}: unknown classification {classification}")
    counts[classification] += 1
    if classification == "ACTION":
        if action not in actions:
            raise SystemExit(f"row {expected_index}: unknown action {action}")
        actions[action] += 1
    elif action != "-":
        raise SystemExit(f"row {expected_index}: rejected row carries an action")
if counts != {"ACTION": 120, "UNRECOVERABLE": 8, "INVALID": 384}:
    raise SystemExit(f"unexpected partition {counts}")
if actions != {"CONTINUE": 48, "VERIFY": 16, "BACKTRACK": 16, "STOP": 40}:
    raise SystemExit(f"unexpected action counts {actions}")
print("TDI-9.3 ElasticXxx interop fixture: 512 rows / partition and action counts verified")
PY
