#!/usr/bin/env bash
set -euo pipefail

fail() {
    echo "TDI-8.1 A3 atomic-store ERROR: $*" >&2
    exit 1
}

A3="tdi-ai/src/assr_h_reference.rs"
VSA="tdi-ai/src/vsa_workspace.rs"
DOC="docs/TDI-8.1-A3-ATOMIC-STORE.md"

for file in "$A3" "$VSA" "$DOC"; do
    test -s "$file" || fail "missing A3 atomic-store qualification surface: $file"
done

grep -Fq 'pub(crate) struct PreparedVsaBundle' "$VSA" \
    || fail "crate-private prepared VSA carrier missing"
grep -Fq 'pub(crate) fn prepare_bundle(' "$VSA" \
    || fail "fallible VSA preparation primitive missing"
grep -Fq 'pub(crate) fn commit_prepared_bundle(' "$VSA" \
    || fail "infallible prepared VSA commit primitive missing"
grep -Fq 'let prepared = self.prepare_bundle(key, payload)?;' "$VSA" \
    || fail "standalone VSA bundle no longer shares prepared path"
grep -Fq 'pub fn step_skip_vsa_and_store(' "$A3" \
    || fail "atomic A3 skip-and-store surface missing"
grep -Fq 'let prepared = self.workspace.prepare_bundle(vsa_store_key, vsa_payload)?;' "$A3" \
    || fail "A3 no longer prepares VSA before A2 mutation"
grep -Fq 'let report = self.a2.step(input, a2_read_key, a2_write_key)?;' "$A3" \
    || fail "atomic A3 path no longer executes unchanged A2 step on external input"
grep -Fq 'self.workspace.commit_prepared_bundle(prepared);' "$A3" \
    || fail "prepared VSA no longer commits after A2 success"
grep -Fq 'atomic_skip_and_store_matches_sequential_success_path_bit_exactly' "$A3" \
    || fail "successful sequential/atomic equivalence oracle missing"
grep -Fq 'rejected_atomic_store_preparation_cannot_mutate_a2_or_vsa_state' "$A3" \
    || fail "VSA-preparation rejection oracle missing"
grep -Fq 'rejected_atomic_a2_step_cannot_commit_prepared_vsa_state' "$A3" \
    || fail "post-preparation A2 rejection oracle missing"
grep -Fq 'prepared_bundle_matches_direct_bundle_bit_exactly' "$VSA" \
    || fail "prepared/direct VSA bundle equivalence oracle missing"
grep -Fq 'does **not** select' "$DOC" \
    || fail "event/payload policy non-selection boundary missing"
grep -Fq 'does not increase declared A3 dynamic-memory bits' "$DOC" \
    || fail "temporary-memory accounting rationale missing"

if grep -Eq 'struct A3Adapter|impl SymbolicTaskAdapter for A3Adapter' "$A3"; then
    fail "concrete A3 task adapter unexpectedly introduced by transaction tranche"
fi

# Do not call bootstrap here: bootstrap -> foundation -> A3 reference -> this
# child gate. The dedicated workflow checks bootstrap independently.
cargo test --locked -p tdi-ai vsa_workspace::tests
cargo test --locked -p tdi-ai assr_h_reference::tests

printf 'TDI-8.1 prepared VSA bundle: BIT_EXACT_WITH_DIRECT_BUNDLE\n'
printf 'TDI-8.1 A3 skip-and-store: CROSS_MECHANISM_ATOMIC\n'
printf 'TDI-8.1 A3 atomic-store temporary budget: UNCHANGED\n'
printf 'TDI-8.1 A3 task-adapter policy: NOT_SELECTED\n'
printf 'TDI-8.2 executable/token surface: ABSENT\n'
printf 'TDI-8.1 A3 atomic-store gate: PASS\n'
