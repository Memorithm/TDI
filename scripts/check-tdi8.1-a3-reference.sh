#!/usr/bin/env bash
set -euo pipefail

fail() {
    echo "TDI-8.1 A3 reference ERROR: $*" >&2
    exit 1
}

SOURCE="tdi-ai/src/assr_h_reference.rs"
DOC="docs/TDI-8.1-A3-INTEGRATED-REFERENCE.md"
HARNESS="tdi-ai/tests/tdi8_assr_h_reference_compile.rs"
LIB="tdi-ai/src/lib.rs"

for file in "$SOURCE" "$DOC" "$HARNESS" "$LIB"; do
    test -s "$file" || fail "missing A3 reference surface: $file"
done

grep -Fq 'pub mod assr_h_reference;' "$LIB" \
    || fail "A3 reference is not exported by tdi-ai"
grep -Fq 'pub struct A3Reference' "$SOURCE" \
    || fail "A3 public reference missing"
grep -Fq 'let mut fused_input = self.workspace.unbind(read_key)?;' "$SOURCE" \
    || fail "VSA read-before-A2 integration missing"
grep -Fq 'self.vsa_fusion_gain * *fused' "$SOURCE" \
    || fail "explicit VSA/input fusion missing"
grep -Fq 'self.a2.step(&fused_input, read_key, write_key)' "$SOURCE" \
    || fail "integrated A3 step no longer delegates to unchanged A2 semantics"
grep -Fq 'pub fn store_vsa(' "$SOURCE" \
    || fail "explicit atomic VSA store surface missing"
grep -Fq '.with_vsa_workspace(vsa.workspace_bits())' "$SOURCE" \
    || fail "A3 persistent VSA accounting missing"
grep -Fq 'checked_add_bits(a2.temporary_working(), vsa.temporary_working_bits())' "$SOURCE" \
    || fail "integrated A3 temporary accounting missing"
grep -Fq 'VSA_FUSION_GAIN_STATIC_BITS' "$SOURCE" \
    || fail "A3 fusion-gain static accounting missing"
grep -Fq 'use tdi_ai::assr_h_reference::A3Reference;' "$HARNESS" \
    || fail "downstream smoke test does not consume the public A3 API"

for test_name in \
    constructor_requires_input_vsa_width_match_and_finite_gain \
    empty_vsa_workspace_preserves_a2_step_semantics_bit_exactly \
    vsa_readout_changes_the_integrated_a3_recurrent_input \
    rejected_integrated_step_cannot_mutate_a2_or_vsa_state \
    rejected_vsa_store_is_atomic_and_does_not_touch_a2 \
    a3_accounting_reports_integrated_vsa_and_peak_temporary_storage \
    synthetic_partitions_can_match_a1_a2_a3_dynamic_budget_exactly \
    snapshot_and_reset_cover_both_a2_and_vsa_persistent_state; do
    grep -Fq "fn ${test_name}()" "$SOURCE" \
        || fail "required A3 oracle test missing: $test_name"
done

if grep -n -E 'TDI8(_|[.]?)?2?_?CONFIRM|TDI82_CONFIRM|TDI8_CONFIRM' \
    "$SOURCE" "$HARNESS" >/tmp/tdi8-a3-token-scan.log 2>/dev/null; then
    cat /tmp/tdi8-a3-token-scan.log >&2
    fail "unexpected TDI-8 confirmation-token surface in A3 implementation"
fi
rm -f /tmp/tdi8-a3-token-scan.log

cargo test -p tdi-ai --locked 'assr_h_reference::tests'
cargo test -p tdi-ai --locked --test tdi8_assr_h_reference_compile

printf 'TDI-8.1 A3 public API: VERIFIED\n'
printf 'TDI-8.1 VSA read/fuse/A2 operation order: VERIFIED\n'
printf 'TDI-8.1 VSA store atomicity boundary: VERIFIED\n'
printf 'TDI-8.1 A3 exact memory accounting: VERIFIED\n'
printf 'TDI-8.1 exact matched-budget representability: VERIFIED\n'
printf 'TDI-8.2 confirmation surface in A3 implementation: ABSENT\n'
printf 'TDI-8.1 A3 reference gate: PASS\n'
