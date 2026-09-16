#!/usr/bin/env bash
set -euo pipefail

fail() {
    echo "TDI-23 Stage-0 bootstrap ERROR: $*" >&2
    exit 1
}

PROGRAMME="docs/TDI-23-PROGRAMME.md"
SCOPE="docs/TDI-23.0-SCOPE.md"
STATUS="docs/TDI-23.0-STATUS.md"
GLOBAL_REDUCTION_DOC="docs/TDI-23.0-GLOBAL-REDUCTION.md"
FREEZE_TEMPLATE="docs/tdi23/tdi23.0-stage0-freeze.template.json"
FREEZE_VALIDATOR="scripts/check-tdi23.0-freeze-template.py"
CATEGORICAL_SRC="tdi-ai/src/tdi23_categorical.rs"
REDUCTION_SRC="tdi-ai/src/tdi23_reduction.rs"
EXPERIMENTAL_FACADE="tdi-ai/src/experimental.rs"
WORKFLOW=".github/workflows/tdi23-stage0-bootstrap.yml"

for file in \
    "$PROGRAMME" \
    "$SCOPE" \
    "$STATUS" \
    "$GLOBAL_REDUCTION_DOC" \
    "$FREEZE_TEMPLATE" \
    "$FREEZE_VALIDATOR" \
    "$CATEGORICAL_SRC" \
    "$REDUCTION_SRC" \
    "$EXPERIMENTAL_FACADE" \
    "$WORKFLOW"; do
    test -s "$file" || fail "missing required Stage-0 surface: $file"
done

grep -Fq 'active Stage 0 bootstrap; not frozen; no confirmatory execution authorised' \
    "$PROGRAMME" || fail "programme Stage-0 status drifted"
grep -Fq '`softmax` is nonlinear' "$PROGRAMME" \
    || fail "programme lost explicit nonlinear softmax boundary"
grep -Fq 'ACTIVE STAGE-0 BOOTSTRAP / NON-FINAL' "$SCOPE" \
    || fail "scope status drifted"
grep -Fq 'tdi23-real-fdhilb-dagger-v1' "$CATEGORICAL_SRC" \
    || fail "versioned categorical contract missing"
grep -Fq 'tdi23-coordinate-reduction-v1' "$REDUCTION_SRC" \
    || fail "versioned global-reduction contract missing"
grep -Fq 'R(f^dagger) = R(f)^dagger' "$GLOBAL_REDUCTION_DOC" \
    || fail "global-reduction dagger preservation statement missing"
grep -Fq 'not claimed to be a functor on arbitrary morphisms' "$GLOBAL_REDUCTION_DOC" \
    || fail "global-reduction document lost explicit non-functoriality boundary"
grep -Fq 'pub mod tdi23_categorical;' "$EXPERIMENTAL_FACADE" \
    || fail "TDI-23 categorical module is not exposed through experimental facade"
grep -Fq 'pub mod tdi23_reduction;' "$EXPERIMENTAL_FACADE" \
    || fail "TDI-23 reduction module is not exposed through experimental facade"
grep -Fq 'bash scripts/check-tdi23-stage0-bootstrap.sh' "$WORKFLOW" \
    || fail "dedicated Stage-0 workflow no longer invokes the bootstrap gate"

python3 "$FREEZE_VALIDATOR" "$FREEZE_TEMPLATE" --self-test

if grep -Eiq \
    'softmax_is_fdhilb_morphism|categorical attention (is|has been) universally superior|quantum advantage established|FLAT-ATTENTION integration is authorized|confirmatory execution is authorized|global reduction is lossless|global reduction is functorial|global reduction improves performance' \
    "$PROGRAMME" "$SCOPE" "$STATUS" "$GLOBAL_REDUCTION_DOC" "$CATEGORICAL_SRC" "$REDUCTION_SRC"; then
    fail "scientific-boundary gate rejected an unsupported positive claim"
fi

# Scan the complete checked-out tree, including conventional docs/results/artifacts/
# release-manifests locations. Exclude only VCS/build internals, never scientific
# output directories, so a forbidden TDI-23 final/confirmatory payload fails closed.
mapfile -t forbidden < <(
    find . \
        -path './.git' -prune -o \
        -path './target' -prune -o \
        -type f \
        \( -iname '*tdi23-final*' -o -iname '*tdi23_final*' \
           -o -iname '*tdi23-confirm*' -o -iname '*tdi23_confirm*' \) \
        -print | sort
)
if ((${#forbidden[@]} > 0)); then
    printf 'TDI-23 Stage-0 forbids confirmatory surfaces:\n' >&2
    printf '  %s\n' "${forbidden[@]}" >&2
    exit 1
fi

cargo fmt --all -- --check
cargo clippy -p tdi-ai --features experimental --all-targets -- -D warnings
cargo test -p tdi-ai --features experimental tdi23

echo "TDI-23 Stage-0 bootstrap checks passed"
