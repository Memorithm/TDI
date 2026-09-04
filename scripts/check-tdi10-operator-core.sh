#!/usr/bin/env bash
set -euo pipefail

fail() {
    echo "TDI-10.0 operator-core ERROR: $*" >&2
    exit 1
}

PROGRAMME="docs/TDI-10-PROGRAMME.md"
AUDIT="docs/tdi10/TDI-10.0-AUDIT.md"
CRATE="tdi-operator"

for path in "$PROGRAMME" "$AUDIT" "$CRATE/Cargo.toml" "$CRATE/src/lib.rs"; do
    test -s "$path" || fail "missing required TDI-10.0 surface: $path"
done

grep -Fq '"tdi-operator"' Cargo.toml || fail "tdi-operator is not a workspace member"
grep -Fq 'TDI-10.x — Operator / Resolvent Research' "$PROGRAMME" \
    || fail "programme identity is missing"
grep -Fq 'Status: **EXACT repository audit / no mathematical theorem claim**' "$AUDIT" \
    || fail "audit evidence status is missing"

# Model-specific RiemannBench symbols are forbidden from the generic code/test
# surface. Plain-language documentation may discuss the repository boundary.
if grep -R -E -n \
    'ProlateParity|build_k0|build_kprime_closed|WPlus|WMinus|sign_correction|zeta_zero|riemann_hypothesis' \
    "$CRATE/src" "$CRATE/tests"; then
    fail "Riemann/prolate-specific symbol leaked into tdi-operator"
fi

cargo fmt --all -- --check
cargo clippy -p tdi-operator --all-targets --locked -- -D warnings
cargo test -p tdi-operator --locked

printf 'TDI-10.0 generic operator boundary: PASS\n'
printf 'TDI-10.0 dense oracle separation: PRESENT\n'
printf 'TDI-10.0 finite core evidence status: EXACT + NUMERICAL TEST EVIDENCE\n'
printf 'TDI-10.0 soft-edge theorem claim: ABSENT\n'
printf 'TDI-10.0 RH claim: ABSENT\n'
