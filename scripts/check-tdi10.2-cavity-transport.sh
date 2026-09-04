#!/usr/bin/env bash
set -euo pipefail

fail() {
    echo "TDI-10.2 cavity-transport ERROR: $*" >&2
    exit 1
}

MODULE="tdi-operator/src/transport.rs"
TEST="tdi-operator/tests/cavity_transport.rs"
DOC="docs/tdi10/TDI-10.2-CAVITY-TRANSPORT.md"

for path in "$MODULE" "$TEST" "$DOC"; do
    test -s "$path" || fail "missing TDI-10.2 surface: $path"
done

grep -Fq 'E_i = alpha E_j + delta' "$DOC" \
    || fail "exact transport identity is missing from documentation"
grep -Fq 'Scientific status' "$DOC" \
    || fail "scientific status section is missing"
grep -Fq '**EXACT**' "$DOC" \
    || fail "exact finite-algebra status is not explicit"
grep -Fq 'NUMERICAL EVIDENCE' "$DOC" \
    || fail "implementation evidence status is not explicit"
grep -Fq 'does **not establish**' "$DOC" \
    || fail "non-claims boundary is missing"

# TDI-10.2 is generic algebra only. Riemann/prolate semantics and premature
# slowly-varying/soft-edge theorem APIs remain forbidden in this increment.
if grep -R -E -n \
    'ProlateParity|build_k0|build_kprime_closed|WPlus|WMinus|zeta_zero|riemann_hypothesis' \
    "$MODULE" "$TEST"; then
    fail "Riemann/prolate-specific symbol leaked into cavity transport"
fi

for forbidden in \
    'UniformContraction' \
    'CumulativeContractionTheorem' \
    'SoftEdgeTheorem' \
    'SlowlyVaryingJacobiTheorem' \
    'UniformResolventTheorem'; do
    if grep -Fq "$forbidden" "$MODULE" "$TEST"; then
        fail "premature theorem surface leaked into TDI-10.2: $forbidden"
    fi
done

cargo fmt --all -- --check
cargo clippy -p tdi-operator --all-targets --locked -- -D warnings
cargo test -p tdi-operator --test cavity_transport --locked
cargo test -p tdi-operator --locked

printf 'TDI-10.2 finite cavity transport identity: EXACT\n'
printf 'TDI-10.2 reference-cavity construction rule: CALLER-SUPPLIED / NOT PROMOTED\n'
printf 'TDI-10.2 implementation regression checks: NUMERICAL EVIDENCE\n'
printf 'TDI-10.2 uniform contraction claim: ABSENT\n'
printf 'TDI-10.2 soft-edge theorem claim: ABSENT\n'
printf 'TDI-10.2 RH claim: ABSENT\n'
printf 'TDI-10.2 cavity transport gate: PASS\n'
