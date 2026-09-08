#!/usr/bin/env bash
set -euo pipefail

fail() {
    echo "TDI-8.1 A3 adapter ERROR: $*" >&2
    exit 1
}

SOURCE="tdi-ai/src/bin/tdi8-a3-adapter-preflight/main.rs"
ADAPTER="tdi-ai/src/task_adapters/a3.rs"
PARENT="tdi-ai/src/task_adapters.rs"
A3="tdi-ai/src/assr_h_reference.rs"
VSA="tdi-ai/src/vsa_workspace.rs"
DOC="docs/TDI-8.1-A3-ADAPTER-PREFLIGHT.md"

for file in "$SOURCE" "$ADAPTER" "$PARENT" "$A3" "$VSA" "$DOC"; do
    test -s "$file" || fail "missing bounded A3 adapter qualification surface: $file"
done

grep -Fq 'pub mod a3;' "$PARENT" \
    || fail "A3 reusable adapter module is not exposed through task_adapters"
grep -Fq 'use tdi_ai::task_adapters::a3::{A3Adapter, A3AdapterError, A3Diagnostics};' "$SOURCE" \
    || fail "A3 preflight no longer consumes the reusable adapter"
grep -Fq 'impl SymbolicTaskAdapter for A3Adapter' "$ADAPTER" \
    || fail "A3 SymbolicTaskAdapter implementation missing"
grep -Fq 'distractor_read_key_for_instance(&instance)?;' "$SOURCE" \
    || fail "instance-scoped neutral non-query read key is missing"
grep -Fq 'self.reference.step_skip_vsa_and_store(' "$ADAPTER" \
    || fail "A3 write events no longer use the atomic A2+VSA transaction"
grep -Fq 'Some(logical_key)' "$ADAPTER" \
    || fail "A2 write key is no longer the prepared logical store key"
grep -Fq 'logical_key,' "$ADAPTER" \
    || fail "shared logical A2/VSA store key marker missing"
grep -Fq 'A3VsaReadRoute::Skip' "$ADAPTER" \
    || fail "distractor VSA-skip routing missing"
grep -Fq 'A3VsaReadRoute::Key(read_key),' "$ADAPTER" \
    || fail "queries no longer route the logical key through VSA"
grep -Fq 'read_key,' "$ADAPTER" \
    || fail "queries no longer share the logical key with the A2 read"
grep -Fq 'let mut next_payload_keys = self.payload_keys;' "$ADAPTER" \
    || fail "payload routing is no longer prepared transactionally"
grep -Fq 'self.payload_keys = next_payload_keys;' "$ADAPTER" \
    || fail "payload cursor is no longer committed after successful atomic store"
grep -Fq 'UnexpectedNeutralReadHit' "$ADAPTER" \
    || fail "neutral non-query A2 hit is no longer fail-closed"
grep -Fq 'FIXTURE_ASSOCIATIVE_FUSION_GAIN: f64 = 1.0' "$SOURCE" \
    || fail "dual-path fixture no longer exercises non-zero A2 fusion"
grep -Fq 'FIXTURE_VSA_FUSION_GAIN: f64 = 1.0' "$SOURCE" \
    || fail "dual-path fixture no longer exercises non-zero VSA fusion"
grep -Fq 'FIXTURE_INPUT_WEIGHT: f64 = 0.5' "$SOURCE" \
    || fail "dual-path half-signal fixture changed without gate review"
grep -Fq 'payload_cursor_does_not_advance_when_atomic_a2_step_rejects' "$SOURCE" \
    || fail "transactional payload rejection regression missing"
grep -Fq 'distractor_and_query_do_not_mutate_vsa_workspace' "$SOURCE" \
    || fail "VSA no-mutation regression for distractor/query missing"
grep -Fq 'isolated_association_round_trip_uses_both_a2_and_vsa' "$SOURCE" \
    || fail "isolated association dual-path regression missing"
grep -Fq 'T1/T3 multi-item VSA superposition/cleanup behavior' "$DOC" \
    || fail "multi-item VSA non-qualification boundary missing"
grep -Fq 'TDI-8.2 executable: absent' "$DOC" \
    || fail "TDI-8.2 absence boundary missing"

grep -Fq 'pub fn step_skip_vsa_and_store(' "$A3" \
    || fail "required atomic A3 transaction missing from reference"
grep -Fq 'pub(crate) fn prepare_bundle(' "$VSA" \
    || fail "required prepared VSA substrate missing"
grep -Fq 'pub(crate) fn commit_prepared_bundle(' "$VSA" \
    || fail "required infallible prepared VSA commit missing"

cargo test --locked -p tdi-ai --bin tdi8-a3-adapter-preflight
cargo run --locked -p tdi-ai --bin tdi8-a3-adapter-preflight

printf 'TDI-8.1 A3 write routing: ATOMIC_A2_PLUS_VSA_WITH_SKIP_READ\n'
printf 'TDI-8.1 A3 query routing: SHARED_LOGICAL_A2_VSA_KEY_NO_WRITE\n'
printf 'TDI-8.1 A3 payload cursor: COMMIT_AFTER_ATOMIC_SUCCESS\n'
printf 'TDI-8.1 A3 fixture mechanisms: A2_NONZERO_AND_VSA_NONZERO\n'
printf 'TDI-8.1 evaluator metadata leakage: ABSENT\n'
printf 'TDI-8.1 multi-item VSA policy: NOT_QUALIFIED_BY_THIS_PREFLIGHT\n'
printf 'TDI-8.2 executable/token/result surface: ABSENT\n'
printf 'TDI-8.1 A3 adapter preflight: PASS\n'
