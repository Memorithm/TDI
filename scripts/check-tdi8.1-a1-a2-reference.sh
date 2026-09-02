#!/usr/bin/env bash
set -euo pipefail

fail() {
    echo "TDI-8.1 A1/A2 reference ERROR: $*" >&2
    exit 1
}

SOURCE="tdi-ai/src/assr_reference.rs"
DOC="docs/TDI-8.1-A1-A2-RECURRENT-REFERENCE.md"
LIB="tdi-ai/src/lib.rs"

for file in "$SOURCE" "$DOC" "$LIB"; do
    test -s "$file" || fail "missing A1/A2 reference surface: $file"
done

grep -Fq 'pub mod assr_reference;' "$LIB" \
    || fail "A1/A2 reference module is not public in tdi-ai"
grep -Fq 'pub struct BoundedRecurrentCore' "$SOURCE" \
    || fail "bounded recurrent core missing"
grep -Fq 'pub struct A1Reference' "$SOURCE" \
    || fail "A1 reference missing"
grep -Fq 'pub struct A2Reference' "$SOURCE" \
    || fail "A2 reference missing"
grep -Fq 'fn hard_tanh(value: f64) -> f64' "$SOURCE" \
    || fail "deterministic hard-tanh activation missing"
grep -Fq 'AssociativePayloadWidthMismatch' "$SOURCE" \
    || fail "A2 state/payload width guard missing"
grep -Fq 'NonFiniteIntermediate' "$SOURCE" \
    || fail "non-finite intermediate rejection missing"
grep -Fq 'let read_status = match self.memory.read(read_key)' "$SOURCE" \
    || fail "A2 lookup-before-write surface missing"
grep -Fq 'Some(self.memory.write(key, &next)?)' "$SOURCE" \
    || fail "A2 optional write surface missing"
grep -Fq '.with_temporary_working(self.core.temporary_working_bits()?)' "$SOURCE" \
    || fail "A1/A2 temporary working accounting missing"

for test_name in \
    recurrent_core_is_exactly_deterministic_for_identical_inputs \
    a1_rejected_input_cannot_mutate_persistent_state \
    a1_accounting_is_valid_for_recurrent_only_arm \
    a2_requires_associative_payload_width_equal_to_state_width \
    a2_write_then_read_hit_fuses_resident_state \
    a2_lookup_happens_before_optional_write \
    a2_accounting_includes_associative_payload_metadata_and_static_constants \
    snapshots_capture_complete_persistent_reference_state; do
    grep -Fq "fn ${test_name}()" "$SOURCE" \
        || fail "required A1/A2 oracle test missing: $test_name"
done

cargo test -p tdi-ai --locked 'assr_reference::tests'

printf 'TDI-8.1 A1 recurrent reference: VERIFIED\n'
printf 'TDI-8.1 A2 recurrent+associative reference: VERIFIED\n'
printf 'TDI-8.1 lookup-before-write and fusion ordering: VERIFIED\n'
printf 'TDI-8.1 A1/A2 accounting surfaces: VERIFIED\n'
printf 'TDI-8.1 A1/A2 reference gate: PASS\n'
