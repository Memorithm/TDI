#!/usr/bin/env bash
set -euo pipefail

fail() {
    echo "TDI-23.1 EQUIVALENCE ERROR: $*" >&2
    exit 1
}

PROGRAMME="docs/TDI-23-PROGRAMME.md"
STATUS="docs/TDI-23.1-STATUS.md"
SCOPE="docs/TDI-23.1-EQUIVALENCE.md"
IR_SRC="tdi-ai/src/tdi23_ir.rs"
PROVENANCE_SRC="tdi-ai/src/tdi23_ir_provenance.rs"
EQUIVALENCE_SRC="tdi-ai/src/tdi23_ir_equivalence.rs"
EXPERIMENTAL_FACADE="tdi-ai/src/experimental.rs"
WORKFLOW=".github/workflows/tdi23.1-equivalence.yml"

for file in "$PROGRAMME" "$STATUS" "$SCOPE" "$IR_SRC" "$PROVENANCE_SRC" "$EQUIVALENCE_SRC" "$EXPERIMENTAL_FACADE" "$WORKFLOW"; do
    test -s "$file" || fail "missing required TDI-23.1 equivalence surface: $file"
done

grep -Fq 'tdi23.1-boundary-aware-exact-equivalence-v1' "$EQUIVALENCE_SRC" \
    || fail "versioned exact equivalence contract missing"
grep -Fq 'tdi23.1-reduction-contract-summary-v1' "$EQUIVALENCE_SRC" \
    || fail "versioned reduction-summary contract missing"
grep -Fq 'validate_rooted_subgraph' "$EQUIVALENCE_SRC" \
    || fail "independent rooted validation is not wired into equivalence"
grep -Fq 'LinearEvaluationCrossesBoundary' "$EQUIVALENCE_SRC" \
    || fail "nonlinear-boundary fail-closed lowering guard missing"
grep -Fq 'EndpointMismatch' "$EQUIVALENCE_SRC" \
    || fail "exact endpoint-identity guard missing"
grep -Fq 'bitwise_identical' "$EQUIVALENCE_SRC" \
    || fail "IEEE-754 bit-identity diagnostic missing"
grep -Fq 'pub mod tdi23_ir_equivalence;' "$EXPERIMENTAL_FACADE" \
    || fail "equivalence module is not exposed through the experimental facade"
grep -Fq 'NO REWRITE SEARCH / NO CONFIRMATORY EXECUTION' "$SCOPE" \
    || fail "equivalence scope authorization boundary drifted"
grep -Fq 'typed IR and rooted provenance/validation merged' "$STATUS" \
    || fail "status does not record qualified TDI-23.1 foundations"
grep -Fq 'bash scripts/check-tdi23.1-equivalence.sh' "$WORKFLOW" \
    || fail "dedicated workflow no longer invokes the equivalence gate"

if grep -Eiq \
    'rewrite search is authorized|approximate equivalence is proven|global reduction is lossless|global reduction is functorial|softmax is an fdhilb morphism|FLAT-ATTENTION integration is authorized|confirmatory execution is authorized|final execution is authorized' \
    "$PROGRAMME" "$STATUS" "$SCOPE" "$EQUIVALENCE_SRC"; then
    fail "scientific-boundary gate rejected an unsupported positive claim"
fi

cargo fmt --all -- --check
cargo clippy -p tdi-ai --features experimental --all-targets -- -D warnings
cargo test -p tdi-ai --features experimental tdi23_1_equivalence

echo "TDI-23.1 exact equivalence/reduction-contract checks passed"
