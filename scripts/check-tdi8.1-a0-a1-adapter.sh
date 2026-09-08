#!/usr/bin/env bash
set -euo pipefail

fail() {
    echo "TDI-8.1 A0/A1 adapter ERROR: $*" >&2
    exit 1
}

ADAPTER_SOURCE="tdi-ai/src/task_adapters.rs"
PREFLIGHT="tdi-ai/src/bin/tdi8-a0-a1-adapter-preflight/main.rs"
DOC="docs/TDI-8.1-A0-A1-ADAPTER-PREFLIGHT.md"

for file in "$ADAPTER_SOURCE" "$PREFLIGHT" "$DOC"; do
    test -s "$file" || fail "missing bounded A0/A1 adapter surface: $file"
done

# The concrete reviewed A0/A1 policies live in the reusable adapter module.
# Continue checking the original semantic invariants after extraction.
grep -Fq 'impl SymbolicTaskAdapter for A0Adapter' "$ADAPTER_SOURCE" \
    || fail "A0 SymbolicTaskAdapter implementation missing"
grep -Fq 'impl SymbolicTaskAdapter for A1Adapter' "$ADAPTER_SOURCE" \
    || fail "A1 SymbolicTaskAdapter implementation missing"
grep -Fq 'a0_association_query_key(key_code)' "$ADAPTER_SOURCE" \
    || fail "A0 association query no longer uses target-blind exact query key"
grep -Fq 'a0_payload_query_key(position)' "$ADAPTER_SOURCE" \
    || fail "A0 payload query no longer uses requested position only"
grep -Fq 'ExactStatePrediction::InvalidEncoding => TaskPrediction::Invalid' "$ADAPTER_SOURCE" \
    || fail "A1 finite noncanonical readout no longer maps to evaluated invalid prediction"
grep -Fq 'self.reference.step(&input)?;' "$ADAPTER_SOURCE" \
    || fail "A1 adapter no longer advances the bounded recurrent reference"

# The preflight remains the owner of fixture-only parameters and qualification
# assertions. It must consume the public library implementation rather than
# recompiling or duplicating adapter policy locally.
grep -Fq 'use tdi_ai::task_adapters::{A0Adapter, A1Adapter};' "$PREFLIGHT" \
    || fail "A0/A1 preflight does not consume the public reusable adapters"
if grep -Fq '#[path = "../../task_adapters.rs"]' "$PREFLIGHT"; then
    fail "A0/A1 preflight must not recompile the reusable adapter source"
fi
grep -Fq 'a1_invalid_readout=COUNTED_AS_FAILURE' "$PREFLIGHT" \
    || fail "A1 invalid-readout preflight assertion marker missing"
grep -Fq 'a2_a3_adapter_policy=NOT_SELECTED' "$PREFLIGHT" \
    || fail "A2/A3 policy non-selection marker missing"
grep -Fq 'Choosing what is read and written for association, payload and distractor events is part of the architecture semantics' "$DOC" \
    || fail "A2/A3 semantic deferral rationale missing"

# Later qualified adapters may coexist in the shared library. This historical
# A0/A1 qualification must itself remain scoped to A0/A1 and must not instantiate
# A2/A3 policy in its preflight binary.
if grep -Eq 'A2Adapter|A3Adapter' "$PREFLIGHT"; then
    fail "A0/A1 preflight unexpectedly instantiates A2/A3 adapter policy"
fi

cargo run --locked -p tdi-ai --bin tdi8-a0-a1-adapter-preflight

printf 'TDI-8.1 A0 full-history adapter: VERIFIED\n'
printf 'TDI-8.1 A1 encoder/recurrent/readout bridge: VERIFIED\n'
printf 'TDI-8.1 A1 invalid readout accounting: VERIFIED\n'
printf 'TDI-8.1 reusable adapter extraction: VERIFIED\n'
printf 'TDI-8.1 A0/A1 preflight scope: A0_A1_ONLY\n'
printf 'TDI-8.1 A0/A1 adapter preflight: PASS\n'
