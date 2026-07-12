#!/usr/bin/env bash

cd "$(dirname "$0")/.." || exit 1

STATUS=0
OUTPUT="/tmp/tdi-branching-continuous-reproduced.log"

echo "=== FORMAT ==="
cargo fmt --all -- --check || STATUS=1

echo
echo "=== TESTS ==="
cargo test --workspace || STATUS=1

echo
echo "=== CLIPPY ==="
cargo clippy --workspace --all-targets -- -D warnings || STATUS=1

echo
echo "=== PREREGISTRATION HASH ==="
EXPECTED_HASH="$(
    awk '{print $1}' \
        docs/TDI-2-CONTINUOUS-PREREGISTRATION.sha256
)"

ACTUAL_HASH="$(
    sha256sum docs/TDI-2-CONTINUOUS-PREREGISTRATION.md |
    awk '{print $1}'
)"

echo "expected: $EXPECTED_HASH"
echo "actual  : $ACTUAL_HASH"

if [ "$EXPECTED_HASH" != "$ACTUAL_HASH" ]; then
    echo "ERROR: preregistration hash mismatch"
    STATUS=1
fi

echo
echo "=== TDI-2 CONTINUOUS BENCHMARK ==="
cargo run --release --quiet \
    -p tdi-bench \
    --bin tdi-branching-continuous \
    2>&1 | tee "$OUTPUT"

RUN_STATUS=${PIPESTATUS[0]}

if [ "$RUN_STATUS" -ne 0 ]; then
    STATUS=1
fi

echo
echo "=== REFERENCE OUTPUT ==="

if cmp -s "$OUTPUT" results/tdi-branching-continuous.log; then
    echo "deterministic output: identical"
else
    echo "deterministic output: DIFFERENT"
    diff -u \
        results/tdi-branching-continuous.log \
        "$OUTPUT" || true
    STATUS=1
fi

echo
echo "=== RESULT ==="

if [ "$STATUS" -eq 0 ]; then
    echo "TDI-2 reproducibility validation: PASSED"
else
    echo "TDI-2 reproducibility validation: FAILED"
fi

exit "$STATUS"
