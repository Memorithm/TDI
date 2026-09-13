#!/usr/bin/env bash
set -euo pipefail

fail() {
    echo "TDI-11.2 model-observation accounting ERROR: $*" >&2
    exit 1
}

MODULE="tdi-ai/src/hallucination_model_observation_accounting.rs"
REJECTIONS="tdi-ai/src/hallucination_model_observation_rejections.rs"
TEST="tdi-ai/tests/tdi11_model_observation_accounting.rs"
DOC="docs/TDI-11.2-MODEL-OBSERVATION-RESOURCE-ACCOUNTING.md"
FREEZE="docs/tdi11.2-model-observation-freeze.json"
STATUS="docs/TDI-11.2-STATUS.md"

for file in "$MODULE" "$REJECTIONS" "$TEST" "$DOC" "$FREEZE" "$STATUS"; do
    test -s "$file" || fail "missing surface: $file"
done

grep -Fq 'pub struct ModelObservationResourceUsage' "$MODULE" || fail "usage meter missing"
grep -Fq 'pub struct ModelObservationResourceEnvelope' "$MODULE" || fail "envelope missing"
grep -Fq 'CANDIDATE_RESOURCE_ACCOUNTING_CONTRACT' "$MODULE" || fail "non-authorizing candidate missing"
grep -Fq 'RESOURCE_ACCOUNTING_COMPONENT_KEYS' "$MODULE" || fail "component taxonomy missing"
grep -Fq 'checked_add' "$MODULE" || fail "fail-closed checked_add missing"
grep -Fq 'AccountingOverflow = 0x0601' "$REJECTIONS" || fail "0x0601 overflow code missing"
grep -Fq 'AccountingEnvelopeExceeded = 0x0602' "$REJECTIONS" || fail "0x0602 envelope code missing"
grep -Fq 'NOT A PIN' "$DOC" || fail "docs must declare NOT A PIN"
grep -Fq 'ModelObservationResourceAccountingContract' "$DOC" || fail "docs missing candidate id"
grep -Fq 'unlimited_for_development' "$DOC" || fail "docs must declare non-transfer from TDI-11.1"

# Must not promote into stable tdi-ai API.
if grep -Eq 'pub mod hallucination_model_observation_accounting|mod hallucination_model_observation_accounting' tdi-ai/src/lib.rs; then
    fail "model-observation accounting layer was promoted into stable tdi-ai API"
fi

# Must not invent freeze pins or arm execution.
python3 - <<'PY'
import json
from pathlib import Path
freeze = json.loads(Path("docs/tdi11.2-model-observation-freeze.json").read_text(encoding="utf-8"))
if freeze.get("model_execution_authorized") is not False:
    raise SystemExit("model_execution_authorized must remain false")
if freeze.get("final_execution_authorized") is not False:
    raise SystemExit("final_execution_authorized must remain false")
fields = freeze["fields"]
pinned = [name for name, entry in fields.items() if entry.get("status") == "pinned"]
if pinned:
    raise SystemExit(f"invented pins forbidden in this scaffolding slice: {pinned}")
unresolved = [name for name, entry in fields.items() if entry.get("status") == "unresolved_blocking"]
if len(unresolved) != 12:
    raise SystemExit(f"expected 12 unresolved_blocking fields, found {len(unresolved)}")
rac = fields["resource_accounting_contract"]
if rac.get("status") != "unresolved_blocking" or rac.get("value") is not None:
    raise SystemExit("resource_accounting_contract must remain unresolved_blocking with null value")
print("TDI-11.2 freeze pins remain 0/12; resource_accounting_contract unresolved; execution flags hard-false")
PY

grep -Eq '0/12' "$STATUS" || fail "STATUS must still report 0/12 pinned"

for test_name in \
    candidate_identifier_is_stable_and_non_pinning \
    component_taxonomy_is_exact_eleven_keys \
    charge_helpers_accumulate_exactly_and_frame_canonical_usage \
    overflow_fail_closes_to_0x0601 \
    envelope_admission_fail_closes_to_0x0602 \
    unbounded_caller_supplied_admits_finite_usage_but_is_not_a_pin \
    accounting_codes_are_present_unique_and_stable_in_rejection_vocabulary \
    tdi11_1_unlimited_development_label_does_not_transfer_as_11_2_pin; do
    grep -Fq "fn $test_name" "$TEST" || fail "missing qualification test: $test_name"
done

rustfmt --edition 2024 --check "$MODULE" "$REJECTIONS" "$TEST"
cargo clippy -p tdi-ai --test tdi11_model_observation_accounting --locked -- -D warnings
cargo test -p tdi-ai --test tdi11_model_observation_accounting --locked
# Existing rejection vocabulary must remain green after 0x06xx extension.
cargo test -p tdi-ai --test tdi11_model_observation_rejections --locked

printf 'TDI-11.2 model-observation resource accounting taxonomy: PRESENT (non-pinning)\n'
printf 'TDI-11.2 resource_accounting_contract: unresolved_blocking (0/12 pins)\n'
printf 'TDI-11.2 execution authorization: UNAUTHORIZED\n'
