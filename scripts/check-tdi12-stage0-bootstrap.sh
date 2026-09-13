#!/usr/bin/env bash
set -euo pipefail

fail() {
    echo "TDI-12 Stage-0 bootstrap ERROR: $*" >&2
    exit 1
}

PROGRAMME="docs/TDI-12-PROGRAMME.md"
SCOPE="docs/TDI-12.0-SCOPE.md"
STATUS="docs/TDI-12.0-STATUS.md"
ORDINAL_DOC="docs/tdi12/TDI-12.0-ORDINAL-RANKING.md"
FREEZE_TEMPLATE="docs/tdi12/tdi12.0-stage0-freeze.template.json"
ORDINAL_SRC="tdi-operator/src/ordinal.rs"
ORDINAL_TEST="tdi-operator/tests/ordinal_stage0_bootstrap.rs"

for file in \
    "$PROGRAMME" \
    "$SCOPE" \
    "$STATUS" \
    "$ORDINAL_DOC" \
    "$FREEZE_TEMPLATE" \
    "$ORDINAL_SRC" \
    "$ORDINAL_TEST" \
    AGENTS.md \
    .github/copilot-instructions.md; do
    test -s "$file" || fail "missing required Stage-0 surface: $file"
done

# Programme / scope markers.
grep -Fq 'active Stage 0 bootstrap; not frozen; no confirmatory execution authorised' \
    "$PROGRAMME" || fail "programme Stage-0 status drifted"
grep -Fq 'Stage-0 bootstrap: declare freeze-template fields' "$PROGRAMME" \
    || fail "programme stage map missing TDI-12.0 bootstrap wording"
grep -Fq 'Holdout / execution boundary' "$PROGRAMME" \
    || fail "programme holdout boundary section missing"

grep -Fq 'ACTIVE STAGE-0 BOOTSTRAP / NON-FINAL' "$SCOPE" \
    || fail "scope status drifted"
grep -Fq 'EXACT claim 1 — average ranks for ties' "$ORDINAL_DOC" \
    || fail "ordinal doc missing EXACT claim 1"
grep -Fq 'EXACT claim 4 — strictly increasing affine invariance' "$ORDINAL_DOC" \
    || fail "ordinal doc missing EXACT claim 4"
grep -Fq 'No soft-edge / double-scaling statement' "$ORDINAL_DOC" \
    || fail "ordinal doc missing soft-edge non-claim"

# Freeze template must keep execution flags false and fields unresolved.
python3 - <<'PY'
import json
import sys
from pathlib import Path

path = Path("docs/tdi12/tdi12.0-stage0-freeze.template.json")
data = json.loads(path.read_text())
errors = []
if data.get("status") != "template_unfrozen":
    errors.append(f"status={data.get('status')!r}")
if data.get("confirmatory_execution_authorized") is not False:
    errors.append("confirmatory_execution_authorized must be false")
if data.get("final_execution_authorized") is not False:
    errors.append("final_execution_authorized must be false")
freeze = data.get("freeze")
if not isinstance(freeze, dict) or not freeze:
    errors.append("freeze object missing")
else:
    for name, field in freeze.items():
        if not isinstance(field, dict):
            errors.append(f"{name}: not an object")
            continue
        if field.get("status") != "unresolved_blocking":
            errors.append(f"{name}: status={field.get('status')!r}")
        if field.get("value") is not None:
            errors.append(f"{name}: value must be null in Stage-0 template")
if errors:
    print("TDI-12 Stage-0 freeze template rejected:", file=sys.stderr)
    for err in errors:
        print(f"  - {err}", file=sys.stderr)
    sys.exit(1)
print("TDI-12 Stage-0 freeze template: all fields unresolved_blocking; execution flags false")
PY

# Agent contracts must encode the Stage-0 gate.
grep -Fq '## TDI-12.x Stage-0 bootstrap and stage gate' AGENTS.md \
    || fail "AGENTS.md lacks TDI-12 Stage-0 gate"
grep -Fq 'For TDI-12.x work' .github/copilot-instructions.md \
    || fail "copilot-instructions lack TDI-12 Stage-0 gate"

# Scientific-boundary gate: reject positive overclaims (negations are allowed).
if grep -Eiq \
    'confirmatory population materialized|Riemann hypothesis proved|proves a soft-edge theorem|ordinal universality (is|has been) established|TDI-7\.2 token|authorized TDI-8\.2 runner|materializes TDI-9\.2 final' \
    "$PROGRAMME" "$SCOPE" "$STATUS" "$ORDINAL_DOC" "$ORDINAL_SRC" "$ORDINAL_TEST"; then
    fail "scientific-boundary gate rejected an unsupported claim"
fi

# Forbidden executable confirmatory surfaces must not exist.
mapfile -t forbidden < <(
    find tdi-operator tdi-ai tdi-bench scripts .github/workflows -type f \
        \( -iname '*tdi12-final*' -o -iname '*tdi12_final*' \
           -o -iname '*tdi12-confirm*' -o -iname '*tdi12_confirm*' \
           -o -iname '*tdi12.4*' \) \
        2>/dev/null | sort
)
if ((${#forbidden[@]} > 0)); then
    printf 'TDI-12 Stage-0 forbids confirmatory surfaces:\n' >&2
    printf '  %s\n' "${forbidden[@]}" >&2
    exit 1
fi

cargo +1.97.1 fmt --all -- --check
cargo +1.97.1 clippy -p tdi-operator --all-targets -- -D warnings
cargo +1.97.1 test -p tdi-operator --test ordinal_stage0_bootstrap
cargo +1.97.1 test -p tdi-operator

echo "TDI-12 Stage-0 bootstrap checks passed"
