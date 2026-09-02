#!/usr/bin/env bash
set -euo pipefail

fail() {
    echo "TDI-8.1 VSA workspace ERROR: $*" >&2
    exit 1
}

SOURCE="tdi-ai/src/vsa_workspace.rs"
LIB="tdi-ai/src/lib.rs"
HARNESS="tdi-ai/tests/tdi8_vsa_workspace_compile.rs"

for file in "$SOURCE" "$LIB" "$HARNESS"; do
    test -s "$file" || fail "missing VSA workspace surface: $file"
done

grep -Fq 'pub struct BoundedVsaWorkspace' "$SOURCE" \
    || fail "bounded VSA workspace missing"
grep -Fq 'pub fn bind(&self, key: u64, payload: &[f64])' "$SOURCE" \
    || fail "deterministic binding surface missing"
grep -Fq 'pub fn bundle(&mut self, key: u64, payload: &[f64])' "$SOURCE" \
    || fail "bundling/superposition surface missing"
grep -Fq 'pub fn unbind(&self, key: u64)' "$SOURCE" \
    || fail "unbinding/retrieval surface missing"
grep -Fq 'pub fn similarity(&self, key: u64, candidate: &[f64])' "$SOURCE" \
    || fail "similarity/readout surface missing"
grep -Fq 'NonFiniteWorkspaceIntermediate' "$SOURCE" \
    || fail "atomic non-finite workspace guard missing"
grep -Fq 'pub mod vsa_workspace;' "$LIB" \
    || fail "VSA workspace is not exported by tdi-ai"
grep -Fq 'use tdi_ai::vsa_workspace::' "$HARNESS" \
    || fail "public API integration harness missing"

for test_name in \
    single_bundle_unbinds_exactly_for_bipolar_role \
    binding_and_superposition_are_deterministic \
    rejected_bundle_cannot_mutate_workspace \
    payload_width_mismatch_fails_closed \
    single_item_similarity_matches_exact_squared_norm \
    declared_storage_accounts_workspace_working_vector_and_static_projection; do
    grep -Fq "fn ${test_name}()" "$SOURCE" \
        || fail "required VSA oracle test missing: $test_name"
done

# This is bounded TDI-8.1 software validation only. No final/confirmatory surface
# is created or accessed by this gate.
cargo test -p tdi-ai --locked vsa_workspace
cargo test -p tdi-ai --locked --test tdi8_vsa_workspace_compile

printf 'TDI-8.1 deterministic VSA role projection: VERIFIED\n'
printf 'TDI-8.1 binding/bundling/unbinding/similarity: VERIFIED\n'
printf 'TDI-8.1 VSA storage accounting: VERIFIED\n'
printf 'TDI-8.1 VSA public API: VERIFIED\n'
printf 'TDI-8.1 VSA workspace gate: PASS\n'
