#!/usr/bin/env bash
set -euo pipefail

fail() {
    echo "TDI-8.1 associative-memory ERROR: $*" >&2
    exit 1
}

SOURCE="tdi-ai/src/associative_memory.rs"
DOC="docs/TDI-8.1-ASSOCIATIVE-MEMORY.md"
LIB="tdi-ai/src/lib.rs"

for file in "$SOURCE" "$DOC" "$LIB"; do
    test -s "$file" || fail "missing associative-memory surface: $file"
done

grep -Fq 'pub mod associative_memory;' "$LIB" \
    || fail "associative-memory module is not public in tdi-ai"
grep -Fq 'pub struct DirectMappedAssociativeMemory' "$SOURCE" \
    || fail "direct-mapped reference memory missing"
grep -Fq 'pub fn address_for(&self, key: u64) -> u64' "$SOURCE" \
    || fail "deterministic address projection surface missing"
grep -Fq 'fn splitmix64(mut value: u64) -> u64' "$SOURCE" \
    || fail "fixed integer address mixer missing"
grep -Fq 'ReplacedCollision' "$SOURCE" \
    || fail "collision replacement outcome missing"
grep -Fq 'CollisionMiss' "$SOURCE" \
    || fail "collision miss read state missing"
grep -Fq 'NonFinitePayload' "$SOURCE" \
    || fail "non-finite payload rejection missing"
grep -Fq 'OCCUPANCY_BITS_PER_SLOT: u128 = 8' "$SOURCE" \
    || fail "occupancy metadata accounting drifted"
grep -Fq 'TAG_BITS_PER_SLOT: u128 = 64' "$SOURCE" \
    || fail "tag metadata accounting drifted"
grep -Fq 'LAYOUT_METADATA_BITS: u128 = 128' "$SOURCE" \
    || fail "layout metadata accounting drifted"
grep -Fq 'PROJECTION_STATIC_BITS: u128 = 256' "$SOURCE" \
    || fail "projection static accounting drifted"

for test_name in \
    address_projection_is_deterministic_and_bounded \
    empty_insert_and_hit_have_explicit_semantics \
    same_key_update_replaces_payload_in_place \
    collision_is_observable_and_replacement_is_deterministic \
    rejected_write_cannot_partially_mutate_table \
    payload_width_mismatch_fails_closed \
    declared_storage_accounting_includes_payload_metadata_and_projection_constants \
    clear_removes_residency_without_changing_projection; do
    grep -Fq "fn ${test_name}()" "$SOURCE" \
        || fail "required associative-memory oracle test missing: $test_name"
done

if grep -E -n 'HashMap|DefaultHasher|RandomState' "$SOURCE" >/tmp/tdi8-assoc-hash-scan.log; then
    cat /tmp/tdi8-assoc-hash-scan.log >&2
    fail "reference addressing must not depend on randomized hash state"
fi
rm -f /tmp/tdi8-assoc-hash-scan.log

cargo test -p tdi-ai --locked 'associative_memory::tests'

printf 'TDI-8.1 associative-memory public surface: VERIFIED\n'
printf 'TDI-8.1 address/read/write/collision/replacement oracles: VERIFIED\n'
printf 'TDI-8.1 associative-memory accounting constants: VERIFIED\n'
printf 'TDI-8.1 randomized hash dependency: ABSENT\n'
printf 'TDI-8.1 associative-memory gate: PASS\n'
