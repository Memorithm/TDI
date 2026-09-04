#!/usr/bin/env bash
set -euo pipefail

fail() {
    echo "TDI-8.1 A3 atomic-store gate ERROR: $*" >&2
    exit 1
}

A3_SOURCE="tdi-ai/src/assr_h_reference.rs"
VSA_SOURCE="tdi-ai/src/vsa_workspace.rs"
DOC="docs/TDI-8.1-A3-ATOMIC-STORE-GATE.md"

for file in "$A3_SOURCE" "$VSA_SOURCE" "$DOC"; do
    test -s "$file" || fail "missing required surface: $file"
done

grep -Fq 'cross-mechanism atomicity requirement' "$DOC" \
    || fail "atomicity requirement is not pinned in documentation"
grep -Fq 'does **not** choose' "$DOC" \
    || fail "A3 policy non-selection boundary is missing"
grep -Fq 'TDI-8.2 executable: absent' "$DOC" \
    || fail "TDI-8.2 absence boundary is missing"

adapter_present=0
if grep -REq 'struct[[:space:]]+A3Adapter|impl[[:space:]]+SymbolicTaskAdapter[[:space:]]+for[[:space:]]+A3Adapter' tdi-ai/src; then
    adapter_present=1
fi
if test -e tdi-ai/src/bin/tdi8-a3-adapter-preflight; then
    adapter_present=1
fi

if [[ "$adapter_present" -eq 1 ]]; then
    grep -Eq 'prepare_(vsa_)?bundle|PreparedVsaBundle|prepare_bundle' "$VSA_SOURCE" \
        || fail "concrete A3 adapter exists before a VSA prepare/commit substrate"
    grep -Eq 'step_.*store_vsa|store_vsa_.*step|atomic.*vsa.*store|vsa.*store.*atomic' "$A3_SOURCE" \
        || fail "concrete A3 adapter exists before a reviewed atomic A2+VSA event primitive"
    grep -Eq 'commit_(prepared_)?(vsa_)?bundle|commit_prepared' "$VSA_SOURCE" \
        || fail "prepared VSA state has no explicit commit path"
fi

# The existing reference/routing tests are the compatibility floor. This gate
# intentionally does not call the bootstrap checker itself; the dedicated
# workflow invokes bootstrap first and avoids integrity-gate recursion.
cargo test --locked -p tdi-ai assr_h_reference::tests
cargo test --locked -p tdi-ai vsa_workspace::tests

printf 'TDI-8.1 A3 concrete adapter present: %s\n' "$adapter_present"
printf 'TDI-8.1 A3 cross-mechanism atomicity requirement: ENFORCED\n'
printf 'TDI-8.1 A3 task policy: NOT_SELECTED_BY_THIS_GATE\n'
printf 'TDI-8.2 executable/token/result surface: ABSENT\n'
printf 'TDI-8.1 A3 atomic-store gate: PASS\n'
