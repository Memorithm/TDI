#!/usr/bin/env bash
set -euo pipefail

fail() {
    echo "TDI-9.1 foundation ERROR: $*" >&2
    exit 1
}

bash scripts/check-tdi9-bootstrap.sh

MODULE="tdi-ai/src/adaptive_inference.rs"
TEST="tdi-ai/tests/tdi9_adaptive_inference_compile.rs"
STATUS="docs/TDI-9.1-STATUS.md"

for file in "$MODULE" "$TEST" "$STATUS"; do
    test -s "$file" || fail "missing foundation surface: $file"
done

grep -Fq 'pub enum PolicyArm' "$MODULE" || fail "policy-arm identity missing"
grep -Fq 'C0FixedCompute' "$MODULE" || fail "C0 identity missing"
grep -Fq 'C1StaticPreallocation' "$MODULE" || fail "C1 identity missing"
grep -Fq 'C2AdaptiveStopping' "$MODULE" || fail "C2 identity missing"
grep -Fq 'C3VerificationRecovery' "$MODULE" || fail "C3 identity missing"
grep -Fq 'pub enum InferenceAction' "$MODULE" || fail "action vocabulary missing"
grep -Fq 'Continue' "$MODULE" || fail "CONTINUE action missing"
grep -Fq 'Verify' "$MODULE" || fail "VERIFY action missing"
grep -Fq 'Backtrack' "$MODULE" || fail "BACKTRACK action missing"
grep -Fq 'Stop' "$MODULE" || fail "STOP action missing"
grep -Fq 'pub struct PolicyObservation' "$MODULE" || fail "observation carrier missing"
grep -Fq 'VerifierSignalForbidden' "$MODULE" || fail "C3 verifier boundary missing"
grep -Fq 'CheckpointMetadataForbidden' "$MODULE" || fail "C3 checkpoint boundary missing"
grep -Fq 'pub struct ResourceEnvelope' "$MODULE" || fail "resource envelope missing"
grep -Fq 'ComputeComponent::Replay' "$TEST" || fail "replay accounting is not exercised"
grep -Fq 'total_memory_bits' "$TEST" || fail "shared memory accounting is not exercised"

# The first foundation slice must remain infrastructure only. These names are
# reserved for later, separately reviewed TDI-9.1 slices.
if find tdi-ai tdi-bench scripts .github/workflows -type f \
    \( -iname '*tdi9.2*' -o -iname '*tdi9_2*' -o -iname '*tdi9-final*' -o -iname '*tdi9_final*' \) \
    ! -path 'scripts/check-tdi9-bootstrap.sh' \
    ! -path 'scripts/check-tdi9.1-foundation.sh' \
    -print -quit | grep -q .; then
    fail "TDI-9.2/final executable surface exists during TDI-9.1 foundation"
fi

if grep -R -n -E 'struct (P1|P2|P3).*Task|trait .*Solver|struct .*Solver|trait .*PolicySearch|struct .*PolicySearch' \
    "$MODULE" "$TEST" >/tmp/tdi91-foundation-scope.log; then
    cat /tmp/tdi91-foundation-scope.log >&2
    fail "foundation slice silently expanded into solver/task/search semantics"
fi
rm -f /tmp/tdi91-foundation-scope.log

rustfmt --edition 2024 --check "$MODULE" "$TEST"
cargo test -p tdi-ai --test tdi9_adaptive_inference_compile --locked

printf 'TDI-9.1 policy ladder: PRESENT\n'
printf 'TDI-9.1 leakage-safe observation carrier: PRESENT\n'
printf 'TDI-9.1 exact bounded resource meter: PRESENT\n'
printf 'TDI-9.2/final executable surface: ABSENT\n'
printf 'TDI-9.1 policy-accounting foundation gate: PASS\n'
