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
grep -Fq 'pub enum A3VsaReadRoute' "$SOURCE" \
    || fail "A3 explicit VSA read routing missing"
grep -Fq 'pub fn step_routed(' "$SOURCE" \
    || fail "A3 independent VSA/A2 routing surface missing"
grep -Fq 'self.step_routed(input, A3VsaReadRoute::Key(read_key), read_key, write_key)' "$SOURCE" \
    || fail "legacy A3 step no longer preserves the original same-key route"
grep -Fq 'let mut fused_input = self.workspace.unbind(vsa_read_key)?;' "$SOURCE" \
    || fail "explicit keyed VSA read-before-A2 integration missing"
grep -Fq 'self.vsa_fusion_gain * *fused' "$SOURCE" \
    || fail "explicit VSA/input fusion missing"
grep -Fq '.step(&fused_input, a2_read_key, a2_write_key)' "$SOURCE" \
    || fail "routed A3 step no longer delegates fused input to unchanged A2 semantics"
grep -Fq 'A3VsaReadRoute::Skip => Ok(self.a2.step(input, a2_read_key, a2_write_key)?),' "$SOURCE" \
    || fail "A3 Skip route no longer bypasses VSA fusion"
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
    legacy_step_matches_explicit_same_key_route_bit_exactly \
    routed_skip_ignores_nonempty_vsa_and_preserves_a2_semantics_bit_exactly \
    routed_vsa_key_and_a2_read_key_are_independent \
    vsa_readout_changes_the_integrated_a3_recurrent_input \
    rejected_integrated_step_cannot_mutate_a2_or_vsa_state \
    rejected_vsa_store_is_atomic_and_does_not_touch_a2 \
    a3_accounting_reports_integrated_vsa_and_peak_temporary_storage \
    synthetic_partitions_can_match_a1_a2_a3_dynamic_budget_exactly \
    snapshot_and_reset_cover_both_a2_and_vsa_persistent_state; do
    grep -Fq "fn ${test_name}()" "$SOURCE" \
        || fail "required A3 oracle test missing: $test_name"
done

# The parent TDI-8.1 foundation gate owns the repository-wide confirmation-token
# absence scan. Keeping the same forbidden literals in this child gate would
# make the parent scanner flag its own detection pattern as a confirmation
# surface, so this gate deliberately does not duplicate that scan.

if test -f scripts/check-tdi8.1-a3-routing.sh; then
    bash scripts/check-tdi8.1-a3-routing.sh
fi

cargo test -p tdi-ai --locked 'assr_h_reference::tests'
cargo test -p tdi-ai --locked --test tdi8_assr_h_reference_compile

printf 'TDI-8.1 A3 public API: VERIFIED\n'
printf 'TDI-8.1 VSA/A2 routed integration: VERIFIED\n'
printf 'TDI-8.1 legacy same-key semantics: PRESERVED\n'
printf 'TDI-8.1 VSA store atomicity boundary: VERIFIED\n'
printf 'TDI-8.1 A3 exact memory accounting: VERIFIED\n'
printf 'TDI-8.1 exact matched-budget representability: VERIFIED\n'
printf 'TDI-8.1 A3 reference gate: PASS\n'
