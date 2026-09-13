#!/usr/bin/env bash
# Fail-closed TDI-9.3.0 representation-calibration gate.
# Verifies exhaustive C2 + well-formed C3 truth-table equivalence tests,
# synthesis envelopes, fail-closed mutation, C2↔C3 joint invariants,
# complexity dominance, and non-pinning docs.
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
grep -Fq 'does **not** pin TDI-9.1 `agent_search_safe_policy_mutation_contract`' \
  docs/TDI-9.3-BOOLEAN-POLICY-SYNTHESIS.md \
  || fail "TDI-9.3.0 must state mutation surface does not pin 9.1 mutation contract"
grep -Fq 'reference_c2_stop_expression' tdi-ai/src/boolean_policy_synthesis.rs \
  || fail "missing reference_c2_stop_expression fixture"
grep -Fq 'reference_c2_expression_matches_hand_formula_on_exhaustive_table' \
  tdi-ai/src/boolean_policy_synthesis.rs \
  || fail "missing exhaustive truth-table unit test"
grep -Fq 'REFERENCE_C2_PREDICATE_COUNT' tdi-ai/src/boolean_policy_synthesis.rs \
  || fail "missing REFERENCE_C2_PREDICATE_COUNT"

grep -Fq 'reference_c3_policy' tdi-ai/src/boolean_policy_synthesis.rs \
  || fail "missing reference_c3_policy fixture"
grep -Fq 'reference_c3_policy_matches_hand_on_well_formed_action_rows' \
  tdi-ai/src/boolean_policy_synthesis.rs \
  || fail "missing C3 well-formed action-table unit test"
grep -Fq 'REFERENCE_C3_PREDICATE_COUNT' tdi-ai/src/boolean_policy_synthesis.rs \
  || fail "missing REFERENCE_C3_PREDICATE_COUNT"

grep -Fq 'SynthesisSearchEnvelope' tdi-ai/src/boolean_policy_synthesis.rs \
  || fail "missing SynthesisSearchEnvelope"
grep -Fq 'mutate_boolean_policy' tdi-ai/src/boolean_policy_synthesis.rs \
  || fail "missing mutate_boolean_policy"
grep -Fq 'reference_c2_stop_projects_to_c3_absent_stop' \
  tdi-ai/src/boolean_policy_synthesis.rs \
  || fail "missing C2↔C3 joint projection helper"
grep -Fq 'weakly_dominates' tdi-ai/src/boolean_policy_synthesis.rs \
  || fail "missing complexity weakly_dominates"
grep -Fq 'c2_c3_joint_complexity_components_are_pareto_incomparable' \
  tdi-ai/src/boolean_policy_synthesis.rs \
  || fail "missing C2/C3 Pareto-incomparability unit test"

# Must not invent 9.1 pins or arm 9.2 from this surface.
python3 scripts/check-tdi9.1-configuration-freeze.py \
  || fail "9.1 freeze checker failed (9.3 must remain non-authorizing)"

cargo test -p tdi-ai --all-features --lib boolean_policy_synthesis \
  || fail "boolean_policy_synthesis tests failed"

echo "TDI-9.3.0 representation calibration: PASS"
