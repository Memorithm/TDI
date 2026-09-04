#!/usr/bin/env bash
set -euo pipefail

fail() {
    echo "TDI-8.1 A2 adapter ERROR: $*" >&2
    exit 1
}

SOURCE="tdi-ai/src/bin/tdi8-a2-adapter-preflight/main.rs"
DOC="docs/TDI-8.1-A2-ADAPTER-PREFLIGHT.md"

for file in "$SOURCE" "$DOC"; do
    test -s "$file" || fail "missing bounded A2 adapter surface: $file"
done

grep -Fq 'impl SymbolicTaskAdapter for A2Adapter' "$SOURCE" \
    || fail "A2 SymbolicTaskAdapter implementation missing"
grep -Fq 'distractor_read_key_for_instance(&instance)?;' "$SOURCE" \
    || fail "instance-scoped neutral read key is missing"
grep -Fq '.step(&input, self.neutral_read_key, write_key)?;' "$SOURCE" \
    || fail "non-query A2 events no longer use the neutral read key"
grep -Fq 'self.query_step(input, association_memory_key(key_code))' "$SOURCE" \
    || fail "association query no longer reads its logical key"
grep -Fq 'self.query_step(input, payload_memory_key(position))' "$SOURCE" \
    || fail "payload query no longer reads its logical key"
grep -Fq 'UnexpectedNeutralReadHit' "$SOURCE" \
    || fail "neutral-read hit is no longer fail-closed"
grep -Fq 'audit_associative_projection(&instance, &audit_memory)?;' "$SOURCE" \
    || fail "runner-side physical projection audit missing"
grep -Fq 'generator_collision_class_used_as_input=NO' "$SOURCE" \
    || fail "generator collision-class leakage assertion marker missing"
grep -Fq 'a3_vsa_policy=NOT_SELECTED' "$SOURCE" \
    || fail "A3 non-selection marker missing"
grep -Fq 'Passing the same key for both read and write would create implicit read-modify-write behavior' "$DOC" \
    || fail "neutral-read rationale missing from documentation"

if grep -Eq 'impl SymbolicTaskAdapter for A3Adapter|struct A3Adapter' "$SOURCE"; then
    fail "A3 policy unexpectedly introduced by A2 adapter tranche"
fi

cargo run --locked -p tdi-ai --bin tdi8-a2-adapter-preflight

printf 'TDI-8.1 A2 non-query read key: NEUTRAL_AND_UNWRITTEN\n'
printf 'TDI-8.1 A2 query read key: LOGICAL_QUERY_KEY\n'
printf 'TDI-8.1 A2 physical projection/runtime diagnostics: VERIFIED\n'
printf 'TDI-8.1 generator collision metadata leakage: ABSENT\n'
printf 'TDI-8.1 A3 policy: NOT_SELECTED\n'
printf 'TDI-8.1 A2 adapter preflight: PASS\n'
