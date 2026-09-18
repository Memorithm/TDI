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
grep -Fq 'BLOCKED ON TDI-23.1 FREEZE' "$SCOPE" \
    || fail "scope no longer records the TDI-23.1 freeze dependency"
grep -Fq 'NO REWRITE SEARCH / NO CONFIRMATORY EXECUTION' "$SCOPE" \
    || fail "rewrite scope authorization boundary drifted"
grep -Fq 'BOUNDED PREPARATORY CALCULUS MERGED / BLOCKED ON TDI-23.1 FREEZE' "$STATUS" \
    || fail "TDI-23.2 status no longer records merged-preparatory/freeze boundaries"
grep -Fq 'bash scripts/check-tdi23.2-rewrite.sh' "$WORKFLOW" \
    || fail "dedicated workflow no longer invokes the TDI-23.2 gate"

# Reject affirmative authorization claims only. Required statements such as
# "No FLAT-ATTENTION integration is authorized" must remain valid evidence of
# the fail-closed boundary rather than triggering this guard themselves.
FORBIDDEN_AFFIRMATIVE='^[[:space:]]*([-*][[:space:]]*)?(rewrite search|recursive rewriting|reassociation|(tensor(-product)?|direct[- ]sum|tensor(-product)?/direct[- ]sum)([[:space:]]+algebraic)?[[:space:]]+rewrites?|approximate equivalence|FLAT-ATTENTION integration|confirmatory execution|final execution|confirmatory or final execution)[[:space:]]+(is|are)[[:space:]]+authorized([[:space:][:punct:]]|$)'

# Mutation fixtures keep the guard itself fail-closed: natural combined
# affirmative forms must be rejected, while required negative forms must not
# be misclassified as authorization.
printf '%s\n' '- Confirmatory or final execution is authorized.' \
    | grep -Eiq "$FORBIDDEN_AFFIRMATIVE" \
    || fail "authorization guard missed combined confirmatory/final execution claim"
if printf '%s\n' '- No confirmatory or final execution is authorized.' | grep -Eiq "$FORBIDDEN_AFFIRMATIVE"; then
    fail "authorization guard rejected required negative confirmatory/final statement"
fi
printf '%s\n' '- Tensor/direct-sum algebraic rewrites are authorized.' \
    | grep -Eiq "$FORBIDDEN_AFFIRMATIVE" \
    || fail "authorization guard missed combined tensor/direct-sum algebraic rewrite claim"
if printf '%s\n' '- No tensor/direct-sum algebraic rewrites are authorized.' | grep -Eiq "$FORBIDDEN_AFFIRMATIVE"; then
    fail "authorization guard rejected required negative tensor/direct-sum statement"
fi
printf '%s\n' '- Tensor-product/direct-sum algebraic rewrites are authorized.' \
    | grep -Eiq "$FORBIDDEN_AFFIRMATIVE" \
    || fail "authorization guard missed combined tensor-product/direct-sum algebraic rewrite claim"
if printf '%s\n' '- No tensor-product/direct-sum algebraic rewrites are authorized.' | grep -Eiq "$FORBIDDEN_AFFIRMATIVE"; then
    fail "authorization guard rejected required negative tensor-product/direct-sum statement"
fi
if printf '%s\n' '- No FLAT-ATTENTION integration is authorized.' | grep -Eiq "$FORBIDDEN_AFFIRMATIVE"; then
    fail "authorization guard rejected required negative FLAT statement"
fi

if grep -Eiq "$FORBIDDEN_AFFIRMATIVE" "$SCOPE" "$STATUS" "$REWRITE_SRC"; then
    fail "scientific-boundary gate rejected an unsupported affirmative authorization claim"
fi

cargo fmt --all -- --check
cargo clippy -p tdi-ai --features experimental --all-targets -- -D warnings
cargo test -p tdi-ai --features experimental tdi23_2_rewrite

echo "TDI-23.2 bounded local rewrite checks passed"
