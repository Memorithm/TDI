#!/usr/bin/env bash
set -euo pipefail

fail() {
    echo "TDI-23.2 REWRITE ERROR: $*" >&2
    exit 1
}

SCOPE="docs/TDI-23.2-REWRITE-CALCULUS.md"
STATUS="docs/TDI-23.2-STATUS.md"
REWRITE_SRC="tdi-ai/src/tdi23_rewrite.rs"
EQUIVALENCE_SRC="tdi-ai/src/tdi23_ir_equivalence.rs"
PROVENANCE_SRC="tdi-ai/src/tdi23_ir_provenance.rs"
EXPERIMENTAL_FACADE="tdi-ai/src/experimental.rs"
WORKFLOW=".github/workflows/tdi23.2-rewrite.yml"

for file in "$SCOPE" "$STATUS" "$REWRITE_SRC" "$EQUIVALENCE_SRC" "$PROVENANCE_SRC" "$EXPERIMENTAL_FACADE" "$WORKFLOW"; do
    test -s "$file" || fail "missing required TDI-23.2 surface: $file"
done

grep -Fq 'tdi23.2-local-rewrite-calculus-v1' "$REWRITE_SRC" \
    || fail "versioned local rewrite contract missing"
grep -Fq 'compare_exact_linear_semantics' "$REWRITE_SRC" \
    || fail "TDI-23.1 exact equivalence oracle is not wired into rewrite acceptance"
grep -Fq 'validate_rooted_subgraph' "$REWRITE_SRC" \
    || fail "independent rooted validation is not wired into rewrite acceptance"
grep -Fq 'FiniteValueExact' "$REWRITE_SRC" \
    || fail "finite-value exactness class missing"
grep -Fq 'BitwiseExact' "$REWRITE_SRC" \
    || fail "bitwise exactness class missing"
grep -Fq 'pub mod tdi23_rewrite;' "$EXPERIMENTAL_FACADE" \
    || fail "rewrite module is not exposed through the experimental facade"
grep -Fq 'NO REWRITE SEARCH / NO CONFIRMATORY EXECUTION' "$SCOPE" \
    || fail "rewrite scope authorization boundary drifted"
grep -Fq 'DRAFT PR / NOT FROZEN / NO REWRITE SEARCH' "$STATUS" \
    || fail "TDI-23.2 status no longer records the non-frozen draft boundary"
grep -Fq 'bash scripts/check-tdi23.2-rewrite.sh' "$WORKFLOW" \
    || fail "dedicated workflow no longer invokes the TDI-23.2 gate"

if grep -Eiq \
    'rewrite search is authorized|recursive rewriting is authorized|reassociation is authorized|tensor.*rewrite is authorized|direct[- ]sum.*rewrite is authorized|approximate equivalence is authorized|FLAT-ATTENTION integration is authorized|confirmatory execution is authorized|final execution is authorized' \
    "$SCOPE" "$STATUS" "$REWRITE_SRC"; then
    fail "scientific-boundary gate rejected an unsupported positive claim"
fi

cargo fmt --all -- --check
cargo clippy -p tdi-ai --features experimental --all-targets -- -D warnings
cargo test -p tdi-ai --features experimental tdi23_2_rewrite

echo "TDI-23.2 bounded local rewrite checks passed"
