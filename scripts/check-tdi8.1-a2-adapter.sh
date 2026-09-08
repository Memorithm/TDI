#!/usr/bin/env bash
set -euo pipefail

fail() {
    echo "TDI-8.1 A2 adapter ERROR: $*" >&2
    exit 1
}

ADAPTER_SOURCE="tdi-ai/src/task_adapters.rs"
PREFLIGHT="tdi-ai/src/bin/tdi8-a2-adapter-preflight/main.rs"
DOC="docs/TDI-8.1-A2-ADAPTER-PREFLIGHT.md"

for file in "$ADAPTER_SOURCE" "$PREFLIGHT" "$DOC"; do
    test -s "$file" || fail "missing bounded A2 adapter surface: $file"
done

# Preserve the reviewed A2 semantics in the reusable library module.
grep -Fq 'impl SymbolicTaskAdapter for A2Adapter' "$ADAPTER_SOURCE" \
    || fail "A2 SymbolicTaskAdapter implementation missing"
grep -Fq '.step(&input, self.neutral_read_key, write_key)?;' "$ADAPTER_SOURCE" \
    || fail "non-query A2 events no longer use the neutral read key"
grep -Fq 'self.query_step(input, association_memory_key(key_code))' "$ADAPTER_SOURCE" \
    || fail "association query no longer reads its logical key"
grep -Fq 'self.query_step(input, payload_memory_key(position))' "$ADAPTER_SOURCE" \
    || fail "payload query no longer reads its logical key"
grep -Fq 'let mut next_payload_keys = self.payload_keys;' "$ADAPTER_SOURCE" \
    || fail "payload routing is no longer prepared transactionally"
grep -Fq 'self.payload_keys = next_payload_keys;' "$ADAPTER_SOURCE" \
    || fail "payload routing is no longer committed after successful A2 step"
grep -Fq 'UnexpectedNeutralReadHit' "$ADAPTER_SOURCE" \
    || fail "neutral-read hit is no longer fail-closed"

# The preflight retains fixture-owned inputs, projection audit and qualification
# markers, and must consume the public adapter implementation.
grep -Fq 'use tdi_ai::task_adapters::A2Adapter;' "$PREFLIGHT" \
    || fail "A2 preflight does not consume the public reusable adapter"
if grep -Fq '#[path = "../../task_adapters.rs"]' "$PREFLIGHT"; then
    fail "A2 preflight must not recompile the reusable adapter source"
fi
grep -Fq 'distractor_read_key_for_instance(&instance)?;' "$PREFLIGHT" \
    || fail "instance-scoped neutral read key is missing"
grep -Fq 'audit_associative_projection(&instance, &audit_memory)?;' "$PREFLIGHT" \
    || fail "runner-side physical projection audit missing"
grep -Fq 'generator_collision_class_used_as_input=NO' "$PREFLIGHT" \
    || fail "generator collision-class leakage assertion marker missing"
grep -Fq 'a3_vsa_policy=NOT_SELECTED' "$PREFLIGHT" \
    || fail "A3 non-selection marker missing"
grep -Fq 'Passing the same key for both read and write would create implicit read-modify-write behavior' "$DOC" \
    || fail "neutral-read rationale missing from documentation"
grep -Fq 'the chronological key cursor is transactional' "$DOC" \
    || fail "transactional payload-routing semantics missing from documentation"

if grep -Eq 'impl SymbolicTaskAdapter for A3Adapter|struct A3Adapter' "$ADAPTER_SOURCE" "$PREFLIGHT"; then
    fail "A3 policy unexpectedly introduced by A2 adapter tranche"
fi

cargo run --locked -p tdi-ai --bin tdi8-a2-adapter-preflight

printf 'TDI-8.1 A2 non-query read key: NEUTRAL_AND_UNWRITTEN\n'
printf 'TDI-8.1 A2 query read key: LOGICAL_QUERY_KEY\n'
printf 'TDI-8.1 A2 payload routing: COMMIT_AFTER_SUCCESS\n'
printf 'TDI-8.1 A2 physical projection/runtime diagnostics: VERIFIED\n'
printf 'TDI-8.1 generator collision metadata leakage: ABSENT\n'
printf 'TDI-8.1 A3 policy: NOT_SELECTED\n'
printf 'TDI-8.1 A2 adapter preflight: PASS\n'
