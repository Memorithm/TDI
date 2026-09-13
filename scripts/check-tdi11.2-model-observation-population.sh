#!/usr/bin/env bash
set -euo pipefail

fail() {
    echo "TDI-11.2 model-observation population ERROR: $*" >&2
    exit 1
}

MODULE="tdi-ai/src/hallucination_model_observation_population.rs"
REJECTIONS="tdi-ai/src/hallucination_model_observation_rejections.rs"
TEST="tdi-ai/tests/tdi11_model_observation_population.rs"
DOC="docs/TDI-11.2-MODEL-OBSERVATION-POPULATION.md"
FREEZE="docs/tdi11.2-model-observation-freeze.json"
STATUS="docs/TDI-11.2-STATUS.md"

for file in "$MODULE" "$REJECTIONS" "$TEST" "$DOC" "$FREEZE" "$STATUS"; do
    test -s "$file" || fail "missing surface: $file"
done

grep -Fq 'pub struct ModelObservationPopulationDerivationContract' "$MODULE" || fail "population contract missing"
grep -Fq 'pub struct ModelObservationPopulationStratum' "$MODULE" || fail "population stratum missing"
grep -Fq 'CANDIDATE_POPULATION_DERIVATION_CONTRACT' "$MODULE" || fail "population candidate missing"
grep -Fq 'AUTHORIZED_POPULATION_DOMAIN_KEYS' "$MODULE" || fail "authorized domain taxonomy missing"
grep -Fq 'PopulationEmpty = 0x0801' "$REJECTIONS" || fail "0x0801 population code missing"
grep -Fq 'PopulationFinalMaterialLeak = 0x080B' "$REJECTIONS" || fail "0x080B population code missing"
grep -Fq 'NOT A PIN' "$DOC" || fail "docs must declare NOT A PIN"
grep -Fq 'ModelObservationPopulationDerivationContract' "$DOC" || fail "docs missing population candidate"

# Must not promote into stable tdi-ai API.
if grep -Eq 'pub mod hallucination_model_observation_population|mod hallucination_model_observation_population' tdi-ai/src/lib.rs; then
    fail "model-observation population layer was promoted into stable tdi-ai API"
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
entry = fields["development_validation_population_derivation"]
if entry.get("status") != "unresolved_blocking" or entry.get("value") is not None:
    raise SystemExit("development_validation_population_derivation must remain unresolved_blocking with null value")
print("TDI-11.2 freeze pins remain 0/12; population derivation unresolved; execution flags hard-false")
PY

grep -Eq '0/12' "$STATUS" || fail "STATUS must still report 0/12 pinned"

for test_name in \
    candidate_identifier_is_stable_distinct_and_non_pinning \
    authorized_domain_taxonomy_is_exact_two_keys_matching_prearm \
    contract_construction_and_membership_are_exact_and_fail_closed \
    forbidden_tokens_and_final_material_leak_fail_closed \
    seed_commitment_is_deterministic_domain_separated_and_fail_closed \
    canonical_record_frames_contract_and_display_names_candidate \
    population_codes_are_present_unique_and_stable; do
    grep -Fq "fn $test_name" "$TEST" || fail "missing qualification test: $test_name"
done

rustfmt --edition 2024 --check "$MODULE" "$REJECTIONS" "$TEST"
cargo clippy -p tdi-ai --test tdi11_model_observation_population --locked -- -D warnings
cargo test -p tdi-ai --test tdi11_model_observation_population --locked
# Existing rejection / accounting / registry vocabulary must remain green after 0x08xx extension.
cargo test -p tdi-ai --test tdi11_model_observation_rejections --locked
cargo test -p tdi-ai --test tdi11_model_observation_accounting --locked
cargo test -p tdi-ai --test tdi11_model_observation_registry --locked

printf 'TDI-11.2 model-observation population derivation: PRESENT (non-pinning)\n'
printf 'TDI-11.2 development_validation_population_derivation: unresolved_blocking (0/12 pins)\n'
printf 'TDI-11.2 execution authorization: UNAUTHORIZED\n'
