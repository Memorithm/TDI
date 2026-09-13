#!/usr/bin/env bash
set -euo pipefail

fail() {
    echo "TDI-11.2 model-observation registry ERROR: $*" >&2
    exit 1
}

MODULE="tdi-ai/src/hallucination_model_observation_registry.rs"
REJECTIONS="tdi-ai/src/hallucination_model_observation_rejections.rs"
TEST="tdi-ai/tests/tdi11_model_observation_registry.rs"
DOC="docs/TDI-11.2-MODEL-OBSERVATION-REGISTRY.md"
FREEZE="docs/tdi11.2-model-observation-freeze.json"
STATUS="docs/TDI-11.2-STATUS.md"

for file in "$MODULE" "$REJECTIONS" "$TEST" "$DOC" "$FREEZE" "$STATUS"; do
    test -s "$file" || fail "missing surface: $file"
done

grep -Fq 'pub struct ModelObservationChannelRegistry' "$MODULE" || fail "registry missing"
grep -Fq 'pub struct ModelObservationTimingContract' "$MODULE" || fail "timing contract missing"
grep -Fq 'pub struct ModelObservationH11AEligibilityRule' "$MODULE" || fail "H11-A rule missing"
grep -Fq 'CANDIDATE_OBSERVATION_REGISTRY' "$MODULE" || fail "registry candidate missing"
grep -Fq 'CANDIDATE_OBSERVATION_TIMING_CONTRACT' "$MODULE" || fail "timing candidate missing"
grep -Fq 'CANDIDATE_H11A_ELIGIBILITY_RULE' "$MODULE" || fail "eligibility candidate missing"
grep -Fq 'INHERITED_OBSERVATION_SOURCE_CLASS_KEYS' "$MODULE" || fail "inherited class taxonomy missing"
grep -Fq 'RegistryEmpty = 0x0701' "$REJECTIONS" || fail "0x0701 registry code missing"
grep -Fq 'EligibilityPrimaryRequiresStrictPreAssertion = 0x070B' "$REJECTIONS" || fail "0x070B eligibility code missing"
grep -Fq 'NOT A PIN' "$DOC" || fail "docs must declare NOT A PIN"
grep -Fq 'ModelObservationChannelRegistry' "$DOC" || fail "docs missing registry candidate"
grep -Fq 'ModelObservationTimingContract' "$DOC" || fail "docs missing timing candidate"
grep -Fq 'ModelObservationH11AEligibilityRule' "$DOC" || fail "docs missing eligibility candidate"

# Must not promote into stable tdi-ai API.
if grep -Eq 'pub mod hallucination_model_observation_registry|mod hallucination_model_observation_registry' tdi-ai/src/lib.rs; then
    fail "model-observation registry layer was promoted into stable tdi-ai API"
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
for key in (
    "exact_observation_registry",
    "exact_observation_timing",
    "primary_pre_assertion_eligibility",
):
    entry = fields[key]
    if entry.get("status") != "unresolved_blocking" or entry.get("value") is not None:
        raise SystemExit(f"{key} must remain unresolved_blocking with null value")
print("TDI-11.2 freeze pins remain 0/12; registry/timing/eligibility unresolved; execution flags hard-false")
PY

grep -Eq '0/12' "$STATUS" || fail "STATUS must still report 0/12 pinned"

for test_name in \
    candidate_identifiers_are_stable_distinct_and_non_pinning \
    inherited_source_class_taxonomy_is_exact_eight_keys_matching_prearm \
    registry_construction_and_lookup_are_exact_and_fail_closed \
    timing_contract_exact_coverage_and_admission_rules \
    h11a_eligibility_exact_primary_posthoc_and_fail_closed_boundary \
    registry_timing_eligibility_codes_are_present_unique_and_stable \
    inherited_class_coverage_helper_does_not_pin_registry; do
    grep -Fq "fn $test_name" "$TEST" || fail "missing qualification test: $test_name"
done

rustfmt --edition 2024 --check "$MODULE" "$REJECTIONS" "$TEST"
cargo clippy -p tdi-ai --test tdi11_model_observation_registry --locked -- -D warnings
cargo test -p tdi-ai --test tdi11_model_observation_registry --locked
# Existing rejection / accounting vocabulary must remain green after 0x07xx extension.
cargo test -p tdi-ai --test tdi11_model_observation_rejections --locked
cargo test -p tdi-ai --test tdi11_model_observation_accounting --locked

printf 'TDI-11.2 model-observation registry/timing/H11-A eligibility: PRESENT (non-pinning)\n'
printf 'TDI-11.2 exact_observation_registry / timing / eligibility: unresolved_blocking (0/12 pins)\n'
printf 'TDI-11.2 execution authorization: UNAUTHORIZED\n'
