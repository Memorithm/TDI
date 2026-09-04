#!/usr/bin/env bash
set -euo pipefail

fail() {
    echo "TDI-10.3 drift-factorization ERROR: $*" >&2
    exit 1
}

MODULE="tdi-operator/src/factorization.rs"
TRANSPORT="tdi-operator/src/transport.rs"
TEST="tdi-operator/tests/cavity_factorization.rs"
DOC="docs/tdi10/TDI-10.3-DRIFT-FACTORIZATION.md"

for path in "$MODULE" "$TRANSPORT" "$TEST" "$DOC"; do
    test -s "$path" || fail "missing TDI-10.3 surface: $path"
done

grep -Fq 'delta = rho + eta_edge + eta_reference' "$DOC" \
    || fail "exact drift factorization is missing"
grep -Fq 'alpha = nu * chi' "$DOC" \
    || fail "exact transport-factor factorization is missing"
grep -Fq '**EXACT algebra**' "$DOC" \
    || fail "exact algebraic status is not explicit"
grep -Fq '**NUMERICAL EVIDENCE**' "$DOC" \
    || fail "implementation evidence status is not explicit"
grep -Fq 'does **not establish**' "$DOC" \
    || fail "non-claims boundary is missing"

if grep -R -E -n \
    'ProlateParity|build_k0|build_kprime_closed|WPlus|WMinus|zeta_zero|riemann_hypothesis' \
    "$MODULE" "$TEST"; then
    fail "Riemann/prolate-specific symbol leaked into drift factorization"
fi

for forbidden in \
    'UniformContraction' \
    'ContractionCertificate' \
    'CumulativeContractionTheorem' \
    'SoftEdgeTheorem' \
    'SlowlyVaryingJacobiTheorem' \
    'UniformResolventTheorem'; do
    if grep -Fq "$forbidden" "$MODULE" "$TEST"; then
        fail "premature theorem/certificate surface leaked into TDI-10.3: $forbidden"
    fi
done

# Terminology is part of the scientific boundary: the generic factor is not a
# contraction certificate until a separate theorem supplies a bound.
if grep -Fq 'pub fn contraction' "$MODULE"; then
    fail "factorization API prematurely names a generic factor as contraction"
fi

cargo fmt --all -- --check
cargo clippy -p tdi-operator --all-targets --locked -- -D warnings
cargo test -p tdi-operator --test cavity_factorization --locked
cargo test -p tdi-operator --locked

printf 'TDI-10.3 drift decomposition: EXACT\n'
printf 'TDI-10.3 transport-factor decomposition: EXACT\n'
printf 'TDI-10.3 local reference metadata: EXPLICIT CALLER INPUT\n'
printf 'TDI-10.3 implementation regression checks: NUMERICAL EVIDENCE\n'
printf 'TDI-10.3 uniform contraction certificate: ABSENT\n'
printf 'TDI-10.3 soft-edge theorem claim: ABSENT\n'
printf 'TDI-10.3 RH claim: ABSENT\n'
printf 'TDI-10.3 drift factorization gate: PASS\n'
