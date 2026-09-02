#!/usr/bin/env bash
set -euo pipefail

fail() {
    echo "TDI-8.1 A3 integration ERROR: $*" >&2
    exit 1
}

SOURCE="tdi-ai/src/a3_reference.rs"
HARNESS="tdi-ai/tests/tdi8_a3_reference_compile.rs"

for file in "$SOURCE" "$HARNESS"; do
    test -s "$file" || fail "missing A3 integration surface: $file"
done

grep -Fq 'pub struct A3Reference' "$SOURCE" \
    || fail "A3 reference missing"
grep -Fq 'pub struct A3RecurrentParameters' "$SOURCE" \
    || fail "A3 recurrent parameter contract missing"
grep -Fq 'self.associative_memory.read(read_key)' "$SOURCE" \
    || fail "A3 associative lookup missing"
grep -Fq 'self.vsa_workspace.unbind(read_key)?' "$SOURCE" \
    || fail "A3 VSA read/unbind missing"
grep -Fq 'self.vsa_workspace.bundle(key, &next)?' "$SOURCE" \
    || fail "A3 VSA write/bundle missing"
grep -Fq '.with_vsa_workspace(vsa.workspace_bits())' "$SOURCE" \
    || fail "A3 VSA persistent accounting missing"
grep -Fq '.checked_add(vsa.temporary_working_bits().get())' "$SOURCE" \
    || fail "A3 concurrent recurrent+VSA temporary peak is not accounted"
grep -Fq 'ReferenceArm::A3' "$SOURCE" \
    || fail "A3 arm accounting/snapshot identity missing"
grep -Fq '#[path = "../src/a3_reference.rs"]' "$HARNESS" \
    || fail "A3 integration compile harness is not pinned to source"

for test_name in \
    a3_matches_a2_before_vsa_contains_information \
    vsa_write_then_read_contributes_to_later_state \
    rejected_step_does_not_mutate_any_persistent_a3_state \
    a3_requires_state_aligned_external_memory_widths \
    a3_accounting_includes_vsa_and_concurrent_peak_scratch \
    snapshot_captures_all_three_persistent_a3_components \
    reset_clears_recurrent_associative_and_vsa_state; do
    grep -Fq "fn ${test_name}()" "$SOURCE" \
        || fail "required A3 oracle test missing: $test_name"
done

# Bounded software-oracle validation only. No TDI-8.2 surface is created or
# invoked here.
cargo test -p tdi-ai --locked --test tdi8_a3_reference_compile

printf 'TDI-8.1 A3 recurrent/associative/VSA integration: VERIFIED\n'
printf 'TDI-8.1 A3 lookup-before-write ordering: VERIFIED\n'
printf 'TDI-8.1 A3 concurrent temporary-memory accounting: VERIFIED\n'
printf 'TDI-8.1 A3 recoverable-failure mutation guard: VERIFIED\n'
printf 'TDI-8.1 A3 integration gate: PASS\n'
