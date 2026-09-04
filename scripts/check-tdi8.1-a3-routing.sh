#!/usr/bin/env bash
set -euo pipefail

fail() {
    echo "TDI-8.1 A3 routing ERROR: $*" >&2
    exit 1
}

SOURCE="tdi-ai/src/assr_h_reference.rs"
DOC="docs/TDI-8.1-A3-ROUTING-SEPARATION.md"

for file in "$SOURCE" "$DOC"; do
    test -s "$file" || fail "missing A3 routing qualification surface: $file"
done

grep -Fq 'pub enum A3VsaReadRoute' "$SOURCE" \
    || fail "explicit A3 VSA read-route enum missing"
grep -Fq 'Skip,' "$SOURCE" \
    || fail "A3 VSA Skip route missing"
grep -Fq 'Key(u64),' "$SOURCE" \
    || fail "A3 VSA keyed route missing"
grep -Fq 'pub fn step_routed(' "$SOURCE" \
    || fail "independently routed A3 step missing"
grep -Fq 'self.step_routed(input, A3VsaReadRoute::Key(read_key), read_key, write_key)' "$SOURCE" \
    || fail "legacy A3 step no longer delegates to the exact same-key route"
grep -Fq 'A3VsaReadRoute::Skip => Ok(self.a2.step(input, a2_read_key, a2_write_key)?),' "$SOURCE" \
    || fail "A3 Skip route no longer bypasses VSA fusion"
grep -Fq 'A3VsaReadRoute::Key(vsa_read_key) =>' "$SOURCE" \
    || fail "A3 keyed VSA route missing"
grep -Fq 'self.workspace.unbind(vsa_read_key)?' "$SOURCE" \
    || fail "A3 keyed route no longer unbinds the explicit VSA key"
grep -Fq 'legacy_step_matches_explicit_same_key_route_bit_exactly' "$SOURCE" \
    || fail "legacy compatibility oracle missing"
grep -Fq 'routed_skip_ignores_nonempty_vsa_and_preserves_a2_semantics_bit_exactly' "$SOURCE" \
    || fail "non-empty VSA Skip oracle missing"
grep -Fq 'routed_vsa_key_and_a2_read_key_are_independent' "$SOURCE" \
    || fail "independent VSA/A2 key oracle missing"
grep -Fq 'an A2-neutral read key is **not** a neutral VSA read key' "$DOC" \
    || fail "routing-separation rationale missing from documentation"
grep -Fq 'does **not** select' "$DOC" \
    || fail "A3 policy non-selection boundary missing"

if grep -Eq 'struct A3Adapter|impl SymbolicTaskAdapter for A3Adapter' "$SOURCE"; then
    fail "concrete A3 task adapter unexpectedly introduced into routing substrate"
fi

bash scripts/check-tdi8-bootstrap.sh
cargo test --locked -p tdi-ai assr_h_reference::tests

printf 'TDI-8.1 A3 legacy same-key route: BIT_EXACT_COMPATIBLE\n'
printf 'TDI-8.1 A3 VSA/A2 read routing: INDEPENDENT\n'
printf 'TDI-8.1 A3 VSA Skip cross-talk: ABSENT_BY_CONSTRUCTION\n'
printf 'TDI-8.1 A3 task-adapter policy: NOT_SELECTED\n'
printf 'TDI-8.2 executable/token surface: ABSENT\n'
printf 'TDI-8.1 A3 routing separation: PASS\n'
