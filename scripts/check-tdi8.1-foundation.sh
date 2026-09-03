#!/usr/bin/env bash
set -euo pipefail

fail() {
    echo "TDI-8.1 foundation ERROR: $*" >&2
    exit 1
}

SOURCE="tdi-ai/src/assr.rs"
LIB="tdi-ai/src/lib.rs"
FOUNDATION="docs/TDI-8.1-REFERENCE-FOUNDATION.md"
STATUS="docs/TDI-8.1-STATUS.md"

for file in "$SOURCE" "$LIB" "$FOUNDATION" "$STATUS"; do
    test -s "$file" || fail "missing required TDI-8.1 foundation surface: $file"
done

# Protect the reviewed reference vocabulary and accounting invariants merged by
# PR #89. This is a structural integrity gate, not a scientific result test.
grep -Fq 'pub enum ReferenceArm' "$SOURCE" \
    || fail "ReferenceArm ladder missing"
for arm in A0 A1 A2 A3; do
    grep -Fq "    $arm," "$SOURCE" \
        || fail "ReferenceArm::$arm missing"
done

grep -Fq 'pub struct StorageBits(u128);' "$SOURCE" \
    || fail "exact u128 storage accounting missing"
grep -Fq 'RequiredComponentMissing' "$SOURCE" \
    || fail "required defining-component rejection missing"
grep -Fq 'DynamicBudgetMismatch' "$SOURCE" \
    || fail "matched-budget rejection missing"
grep -Fq '.checked_add(self.temporary_working)' "$SOURCE" \
    || fail "temporary working storage is no longer counted in matched dynamic memory"
grep -Fq 'pub struct MatchedDynamicBudget' "$SOURCE" \
    || fail "matched dynamic-budget validator missing"
grep -Fq 'require_nonzero(arm, MemoryComponent::RecurrentState, self.recurrent_state)?;' "$SOURCE" \
    || fail "bounded recurrent arms no longer require recurrent state"
grep -Fq 'fn defining_components_must_have_nonzero_storage()' "$SOURCE" \
    || fail "defining-component regression test missing"
grep -Fq 'fn temporary_working_storage_participates_in_matched_budget()' "$SOURCE" \
    || fail "temporary-work budget regression test missing"
grep -Fq 'fn exact_accounting_fails_closed_on_u128_overflow()' "$SOURCE" \
    || fail "overflow regression test missing"

grep -Fq 'pub use assr::{' "$LIB" \
    || fail "ASSR foundation is not exported by tdi-ai"
for symbol in MatchedDynamicBudget MemoryAccounting MemoryAccountingError MemoryComponent ReferenceArm ReferenceSnapshot StorageBits; do
    grep -Fq "$symbol" "$LIB" \
        || fail "tdi-ai export missing: $symbol"
done

# TDI-8.2 must remain absent throughout bounded TDI-8.1 work. Search library
# sources as well as examples/binaries/scripts because TDI-8 now owns library
# code in tdi-ai/src.
mapfile -t forbidden_files < <(
    find tdi-ai tdi-bench scripts -type f \
        \( -iname '*tdi8.2*' -o -iname '*tdi8_2*' -o -iname '*tdi8-final*' -o -iname '*tdi8_final*' -o -iname '*tdi82*holdout*' \) \
        ! -path 'scripts/check-tdi8.1-foundation.sh' \
        ! -path 'scripts/check-tdi8-bootstrap.sh' \
        -print
)
if ((${#forbidden_files[@]} != 0)); then
    printf 'Unexpected TDI-8.2/final-holdout surfaces:\n' >&2
    printf '  %s\n' "${forbidden_files[@]}" >&2
    fail "TDI-8.2 must not exist during bounded TDI-8.1 work"
fi

if grep -R -n -E 'TDI8(_|[.]?)?2?_?CONFIRM|TDI82_CONFIRM|TDI8_CONFIRM' \
    tdi-ai/src tdi-ai/examples tdi-bench/src scripts \
    --exclude='check-tdi8.1-foundation.sh' \
    --exclude='check-tdi8-bootstrap.sh' \
    >/tmp/tdi8-foundation-token-scan.log 2>/dev/null; then
    cat /tmp/tdi8-foundation-token-scan.log >&2
    fail "unexpected TDI-8 confirmation-token surface"
fi
rm -f /tmp/tdi8-foundation-token-scan.log

# Later bounded TDI-8.1 primitives become additive integrity gates once their
# source is present. They remain non-confirmatory and must not create TDI-8.2.
if test -f tdi-ai/src/associative_memory.rs; then
    test -s scripts/check-tdi8.1-associative-memory.sh \
        || fail "associative-memory source exists without its oracle gate"
    bash scripts/check-tdi8.1-associative-memory.sh
fi

if test -f tdi-ai/src/assr_reference.rs; then
    test -s scripts/check-tdi8.1-a1-a2-reference.sh \
        || fail "A1/A2 reference source exists without its oracle gate"
    bash scripts/check-tdi8.1-a1-a2-reference.sh
fi

if test -f tdi-ai/src/vsa_workspace.rs; then
    test -s scripts/check-tdi8.1-vsa-workspace.sh \
        || fail "VSA workspace source exists without its oracle gate"
    bash scripts/check-tdi8.1-vsa-workspace.sh
fi

if test -f tdi-ai/src/assr_h_reference.rs; then
    test -s scripts/check-tdi8.1-a3-reference.sh \
        || fail "A3 reference source exists without its oracle gate"
    bash scripts/check-tdi8.1-a3-reference.sh
fi

if test -f tdi-ai/src/full_history_reference.rs; then
    test -s scripts/check-tdi8.1-a0-reference.sh \
        || fail "A0 full-history source exists without its oracle gate"
    bash scripts/check-tdi8.1-a0-reference.sh
fi

if test -f tdi-ai/src/task_generators.rs; then
    test -s scripts/check-tdi8.1-task-generators.sh \
        || fail "T1/T2/T3 task-generator source exists without its oracle gate"
    bash scripts/check-tdi8.1-task-generators.sh
fi

printf 'TDI-8.1 reference-arm vocabulary: VERIFIED\n'
printf 'TDI-8.1 exact memory-accounting invariants: VERIFIED\n'
printf 'TDI-8.1 defining-component guards: VERIFIED\n'
printf 'TDI-8.1 matched dynamic-budget guard: VERIFIED\n'
printf 'TDI-8.2 executable/token surface: ABSENT\n'
printf 'TDI-8.1 foundation gate: PASS\n'