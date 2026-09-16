#!/usr/bin/env bash
set -euo pipefail

fail() {
    echo "TDI-23.1 IR ERROR: $*" >&2
    exit 1
}

PROGRAMME="docs/TDI-23-PROGRAMME.md"
SCOPE="docs/TDI-23.1-SCOPE.md"
STATUS="docs/TDI-23.1-STATUS.md"
IR_SRC="tdi-ai/src/tdi23_ir.rs"
EXPERIMENTAL_FACADE="tdi-ai/src/experimental.rs"
WORKFLOW=".github/workflows/tdi23.1-ir.yml"

for file in "$PROGRAMME" "$SCOPE" "$STATUS" "$IR_SRC" "$EXPERIMENTAL_FACADE" "$WORKFLOW"; do
    test -s "$file" || fail "missing required TDI-23.1 surface: $file"
done

grep -Fq 'tdi23.1-categorical-attention-ir-v1' "$IR_SRC" \
    || fail "versioned IR contract missing"
grep -Fq 'pub mod tdi23_ir;' "$EXPERIMENTAL_FACADE" \
    || fail "TDI-23.1 IR is not exposed through the experimental facade"
grep -Fq 'DirectSum' "$IR_SRC" \
    || fail "direct-sum construction missing"
grep -Fq 'TensorProduct' "$IR_SRC" \
    || fail "tensor-product construction missing"
grep -Fq 'CompositionObjectMismatch' "$IR_SRC" \
    || fail "exact middle-object legality check missing"
grep -Fq 'ForeignObjectHandle' "$IR_SRC" \
    || fail "foreign object-handle ownership guard missing"
grep -Fq 'ForeignNodeHandle' "$IR_SRC" \
    || fail "foreign node-handle ownership guard missing"
grep -Fq 'tdi23_1_handles_are_bound_to_their_owning_ir' "$IR_SRC" \
    || fail "cross-IR handle negative control missing"
grep -Fq 'DaggerCrossesNonlinearBoundary' "$IR_SRC" \
    || fail "dagger nonlinear-boundary guard missing"
grep -Fq 'MissingReductionAnnotation' "$IR_SRC" \
    || fail "explicit reduction-annotation guard missing"
grep -Fq 'composition_reduction_defect_max_abs' "$IR_SRC" \
    || fail "composition reduction-defect audit missing"
grep -Fq 'ACTIVE DEVELOPMENT / NOT FROZEN / NO CONFIRMATORY EXECUTION AUTHORIZED' "$SCOPE" \
    || fail "TDI-23.1 scope authorization boundary drifted"
grep -Fq 'bash scripts/check-tdi23.1-ir.sh' "$WORKFLOW" \
    || fail "dedicated TDI-23.1 workflow no longer invokes the IR gate"

if grep -Eiq \
    'softmax(_| )is(_| )fdhilb(_| )morphism|global reduction is lossless|global reduction is functorial|categorical attention is universally superior|FLAT-ATTENTION integration is authorized|confirmatory execution is authorized|final execution is authorized' \
    "$PROGRAMME" "$SCOPE" "$STATUS" "$IR_SRC"; then
    fail "scientific-boundary gate rejected an unsupported positive claim"
fi

cargo fmt --all -- --check
cargo clippy -p tdi-ai --features experimental --all-targets -- -D warnings
cargo test -p tdi-ai --features experimental tdi23_1

echo "TDI-23.1 categorical attention IR checks passed"
