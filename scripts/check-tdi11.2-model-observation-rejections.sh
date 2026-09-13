#!/usr/bin/env bash
set -euo pipefail

fail() {
    echo "TDI-11.2 model-observation rejection ERROR: $*" >&2
    exit 1
}

MODULE="tdi-ai/src/hallucination_model_observation_rejections.rs"
TEST="tdi-ai/tests/tdi11_model_observation_rejections.rs"
DOC="docs/TDI-11.2-MODEL-OBSERVATION-REJECTIONS.md"
FREEZE="docs/tdi11.2-model-observation-freeze.json"
STATUS="docs/TDI-11.2-STATUS.md"

for file in "$MODULE" "$TEST" "$DOC" "$FREEZE" "$STATUS"; do
    test -s "$file" || fail "missing surface: $file"
done

grep -Fq '#[repr(u16)]' "$MODULE" || fail "stable numeric rejection representation missing"
grep -Fq 'pub enum ModelObservationRejectionCode' "$MODULE" || fail "typed rejection code missing"
grep -Fq 'pub struct ModelObservationRejectionRecord' "$MODULE" || fail "rejection record missing"
grep -Fq 'pub struct ModelObservationTraceProvenance' "$MODULE" || fail "provenance scaffold missing"
grep -Fq 'CANDIDATE_TYPED_REJECTION_VOCABULARY' "$MODULE" || fail "non-authorizing rejection candidate missing"
grep -Fq 'CANDIDATE_PROVENANCE_SCHEMA' "$MODULE" || fail "non-authorizing provenance candidate missing"
grep -Fq 'NOT A PIN' "$DOC" || fail "docs must declare NOT A PIN"
grep -Fq 'ModelObservationRejectionCode' "$DOC" || fail "docs missing rejection candidate id"
grep -Fq 'ModelObservationTraceProvenance' "$DOC" || fail "docs missing provenance candidate id"

# Must not promote into stable tdi-ai API.
if grep -Eq 'pub mod hallucination_model_observation_rejections|mod hallucination_model_observation_rejections' tdi-ai/src/lib.rs; then
    fail "model-observation rejection layer was promoted into stable tdi-ai API"
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
print("TDI-11.2 freeze pins remain 0/12; execution flags hard-false")
PY

grep -Eq '0/12' "$STATUS" || fail "STATUS must still report 0/12 pinned"

for test_name in \
    candidate_identifiers_are_stable_and_non_pinning \
    source_class_mismatch_maps_to_stable_rejection_code \
    timing_non_monotonic_maps_through_adapter_timing_wrapper \
    undeclared_channel_is_packet_rejection_not_quality_outcome \
    rejection_numeric_codes_are_exact_unique_and_stable \
    provenance_scaffold_accepts_dev_val_and_frames_canonical_record \
    provenance_scaffold_rejects_forbidden_surface_tokens_and_empty_fields \
    adapter_identity_failures_map_to_0x01xx_range; do
    grep -Fq "fn $test_name" "$TEST" || fail "missing qualification test: $test_name"
done

rustfmt --edition 2024 --check "$MODULE" "$TEST"
cargo clippy -p tdi-ai --test tdi11_model_observation_rejections --locked -- -D warnings
cargo test -p tdi-ai --test tdi11_model_observation_rejections --locked

printf 'TDI-11.2 model-observation rejection vocabulary: PRESENT (non-pinning)\n'
printf 'TDI-11.2 model-observation provenance scaffold: PRESENT (non-pinning)\n'
printf 'TDI-11.2 execution authorization: UNAUTHORIZED\n'
