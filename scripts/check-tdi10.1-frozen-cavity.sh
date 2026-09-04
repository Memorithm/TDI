#!/usr/bin/env bash
set -euo pipefail

fail() {
    echo "TDI-10.1 frozen-cavity ERROR: $*" >&2
    exit 1
}

MODULE="tdi-operator/src/frozen.rs"
TEST="tdi-operator/tests/frozen_toeplitz.rs"
DOC="docs/tdi10/TDI-10.1-FROZEN-CAVITY.md"

for path in "$MODULE" "$TEST" "$DOC"; do
    test -s "$path" || fail "missing TDI-10.1 surface: $path"
done

grep -Fq 'diagonal <= twice_edge_abs' "$MODULE" \
    || fail "strict positive-symbol guard a > 2|b| is missing"
grep -Fq 'let ratio = twice_edge_abs / diagonal;' "$MODULE" \
    || fail "scaled discriminant evaluation is missing"
grep -Fq 'Scientific status: **EXACT**' "$DOC" \
    || fail "frozen-model evidence status is not explicit"
grep -Fq 'NUMERICAL EVIDENCE' "$DOC" \
    || fail "quadrature evidence status is not explicit"
grep -Fq 'does not establish' "$DOC" \
    || fail "non-claims section is missing"

# TDI-10.1 is a constant generic reference model only. Model-specific and
# premature slowly-varying/soft-edge theorem surfaces remain forbidden here.
if grep -R -E -n \
    'ProlateParity|build_k0|build_kprime_closed|WPlus|WMinus|zeta_zero|riemann_hypothesis' \
    "$MODULE" "$TEST"; then
    fail "Riemann/prolate-specific symbol leaked into frozen reference model"
fi

for forbidden in 'SlowlyVarying' 'SoftEdgeTheorem' 'UniformResolventTheorem'; do
    if grep -Fq "$forbidden" "$MODULE" "$TEST"; then
        fail "premature theorem surface leaked into TDI-10.1: $forbidden"
    fi
done

cargo fmt --all -- --check
cargo clippy -p tdi-operator --all-targets --locked -- -D warnings
cargo test -p tdi-operator --test frozen_toeplitz --locked
cargo test -p tdi-operator --locked

printf 'TDI-10.1 constant positive-symbol hypothesis: PRESENT\n'
printf 'TDI-10.1 fixed-point and Green identities: EXACT MODEL FORMULAS\n'
printf 'TDI-10.1 independent Fourier implementation check: NUMERICAL EVIDENCE\n'
printf 'TDI-10.1 finite-section convergence claim: ABSENT\n'
printf 'TDI-10.1 soft-edge uniform theorem claim: ABSENT\n'
printf 'TDI-10.1 RH claim: ABSENT\n'
printf 'TDI-10.1 frozen cavity gate: PASS\n'
