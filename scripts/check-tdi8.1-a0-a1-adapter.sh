#!/usr/bin/env bash
set -euo pipefail

fail() {
    echo "TDI-8.1 A0/A1 adapter ERROR: $*" >&2
    exit 1
}

SOURCE="tdi-ai/src/bin/tdi8-a0-a1-adapter-preflight/main.rs"
DOC="docs/TDI-8.1-A0-A1-ADAPTER-PREFLIGHT.md"

for file in "$SOURCE" "$DOC"; do
    test -s "$file" || fail "missing bounded A0/A1 adapter surface: $file"
done

grep -Fq 'impl SymbolicTaskAdapter for A0Adapter' "$SOURCE" \
    || fail "A0 SymbolicTaskAdapter implementation missing"
grep -Fq 'impl SymbolicTaskAdapter for A1Adapter' "$SOURCE" \
    || fail "A1 SymbolicTaskAdapter implementation missing"
grep -Fq 'a0_association_query_key(key_code)' "$SOURCE" \
    || fail "A0 association query no longer uses target-blind exact query key"
grep -Fq 'a0_payload_query_key(position)' "$SOURCE" \
    || fail "A0 payload query no longer uses requested position only"
grep -Fq 'ExactStatePrediction::InvalidEncoding => TaskPrediction::Invalid' "$SOURCE" \
    || fail "A1 finite noncanonical readout no longer maps to evaluated invalid prediction"
grep -Fq 'self.reference.step(&input)?;' "$SOURCE" \
    || fail "A1 adapter no longer advances the bounded recurrent reference"
grep -Fq 'a1_invalid_readout=COUNTED_AS_FAILURE' "$SOURCE" \
    || fail "A1 invalid-readout preflight assertion marker missing"
grep -Fq 'a2_a3_adapter_policy=NOT_SELECTED' "$SOURCE" \
    || fail "A2/A3 policy non-selection marker missing"
grep -Fq 'Choosing what is read and written for association, payload and distractor events is part of the architecture semantics' "$DOC" \
    || fail "A2/A3 semantic deferral rationale missing"

# This bounded tranche must not accidentally instantiate A2/A3 concrete adapters.
if grep -Eq 'impl SymbolicTaskAdapter for A[23]Adapter|struct A[23]Adapter' "$SOURCE"; then
    fail "A2/A3 concrete adapter policy unexpectedly introduced"
fi

cargo run --locked -p tdi-ai --bin tdi8-a0-a1-adapter-preflight

printf 'TDI-8.1 A0 full-history adapter: VERIFIED\n'
printf 'TDI-8.1 A1 encoder/recurrent/readout bridge: VERIFIED\n'
printf 'TDI-8.1 A1 invalid readout accounting: VERIFIED\n'
printf 'TDI-8.1 A2/A3 adapter policy: NOT_SELECTED\n'
printf 'TDI-8.1 A0/A1 adapter preflight: PASS\n'
