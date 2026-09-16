#!/usr/bin/env bash
set -euo pipefail

fail() {
    echo "TDI-23.1 provenance ERROR: $*" >&2
    exit 1
}

DOC="docs/TDI-23.1-PROVENANCE.md"
SRC="tdi-ai/src/tdi23_ir_provenance.rs"
FACADE="tdi-ai/src/experimental.rs"
WORKFLOW=".github/workflows/tdi23.1-provenance.yml"

for file in "$DOC" "$SRC" "$FACADE" "$WORKFLOW"; do
    test -s "$file" || fail "missing required provenance surface: $file"
done

grep -Fq 'tdi23.1-rooted-ir-manifest-v1' "$SRC" \
    || fail "versioned rooted-manifest contract missing"
grep -Fq 'pub mod tdi23_ir_provenance;' "$FACADE" \
    || fail "rooted provenance module is not exposed through experimental facade"
grep -Fq 'validate_rooted_subgraph' "$SRC" \
    || fail "independent rooted validator missing"
grep -Fq 'canonical_rooted_manifest' "$SRC" \
    || fail "canonical rooted manifest emitter missing"
grep -Fq 'to_bits()' "$SRC" \
    || fail "exact f64 bit serialization missing"
grep -Fq 'name_hex=' "$SRC" \
    || fail "delimiter-safe object-name encoding missing"
grep -Fq 'tdi23_1_manifest_ignores_runtime_owner_token' "$SRC" \
    || fail "runtime-owner independence control missing"
grep -Fq 'tdi23_1_manifest_preserves_f64_bit_patterns' "$SRC" \
    || fail "f64 bit-pattern control missing"
grep -Fq 'construction-order-sensitive' "$DOC" \
    || fail "manifest determinism boundary missing"
grep -Fq 'does not claim' "$DOC" \
    || fail "non-canonicalization boundary missing"
grep -Fq 'bash scripts/check-tdi23.1-provenance.sh' "$WORKFLOW" \
    || fail "dedicated provenance workflow no longer invokes its gate"

if grep -Eiq \
    'graph-isomorphism canonicalization (is|has been) established|cryptographic collision resistance (is|has been) established|globally unique graph identifier (is|has been) established|rewrite search is authorized|FLAT-ATTENTION integration is authorized|confirmatory execution is authorized|final execution is authorized' \
    "$DOC" "$SRC"; then
    fail "provenance boundary gate rejected an unsupported positive claim"
fi

cargo fmt --all -- --check
cargo clippy -p tdi-ai --features experimental --all-targets -- -D warnings
cargo test -p tdi-ai --features experimental tdi23_1_manifest
cargo test -p tdi-ai --features experimental tdi23_1_root_validation

echo "TDI-23.1 rooted IR provenance checks passed"
