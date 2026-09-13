#!/usr/bin/env bash
# Fail-closed TDI-9.3.0 representation-calibration gate.
# Verifies exhaustive C2 truth-table equivalence tests and non-pinning docs.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

fail() { echo "TDI-9.3.0 calibration ERROR: $*" >&2; exit 1; }

test -s docs/TDI-9.3-BOOLEAN-POLICY-SYNTHESIS.md \
  || fail "missing TDI-9.3 design doc"
grep -Fq '## Relationship to TDI-9.1' docs/TDI-9.3-BOOLEAN-POLICY-SYNTHESIS.md \
  || fail "TDI-9.3 doc must keep Relationship to TDI-9.1 boundary"
grep -Fq 'does **not** freeze observation-to-predicate' docs/TDI-9.3-BOOLEAN-POLICY-SYNTHESIS.md \
  || fail "TDI-9.3.0 section must state non-pinning boundary"
grep -Fq 'reference_c2_stop_expression' tdi-ai/src/boolean_policy_synthesis.rs \
  || fail "missing reference_c2_stop_expression fixture"
grep -Fq 'reference_c2_expression_matches_hand_formula_on_exhaustive_table' \
  tdi-ai/src/boolean_policy_synthesis.rs \
  || fail "missing exhaustive truth-table unit test"
grep -Fq 'REFERENCE_C2_PREDICATE_COUNT' tdi-ai/src/boolean_policy_synthesis.rs \
  || fail "missing REFERENCE_C2_PREDICATE_COUNT"

# Must not invent 9.1 pins or arm 9.2 from this surface.
python3 scripts/check-tdi9.1-configuration-freeze.py \
  || fail "9.1 freeze checker failed (9.3 must remain non-authorizing)"

cargo test -p tdi-ai --all-features --lib boolean_policy_synthesis \
  || fail "boolean_policy_synthesis tests failed"

echo "TDI-9.3.0 representation calibration: PASS"
