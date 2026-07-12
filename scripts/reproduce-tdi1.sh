#!/usr/bin/env bash

cd "$(dirname "$0")/.." || exit 1

mkdir -p results

echo "=== FORMAT ==="
cargo fmt --all -- --check
FMT_STATUS=$?

echo
echo "=== TESTS ==="
cargo test --workspace
TEST_STATUS=$?

echo
echo "=== CLIPPY ==="
cargo clippy --workspace --all-targets -- -D warnings
CLIPPY_STATUS=$?

echo
echo "=== CONTRE-EXEMPLE ADVERSARIAL ==="
cargo run --release --quiet -p tdi-bench --bin tdi-bench \
  2>&1 | tee results/tdi-adversarial.log
ADVERSARIAL_STATUS=${PIPESTATUS[0]}

echo
echo "=== SCAN EXHAUSTIF ==="
cargo run --release --quiet -p tdi-bench --bin tdi-scan \
  2>&1 | tee results/tdi-scan.log
SCAN_STATUS=${PIPESTATUS[0]}

echo
echo "=== ÉVALUATION LEAVE-ONE-OUT ==="
cargo run --release --quiet -p tdi-bench --bin tdi-eval \
  2>&1 | tee results/tdi-eval.log
EVAL_STATUS=${PIPESTATUS[0]}

echo
echo "=== HOLDOUT INDÉPENDANT ==="
cargo run --release --quiet -p tdi-bench --bin tdi-holdout \
  2>&1 | tee results/tdi-holdout.log
HOLDOUT_STATUS=${PIPESTATUS[0]}

echo
echo "=== STATUTS ==="
echo "fmt         : $FMT_STATUS"
echo "tests       : $TEST_STATUS"
echo "clippy      : $CLIPPY_STATUS"
echo "adversarial : $ADVERSARIAL_STATUS"
echo "scan        : $SCAN_STATUS"
echo "eval        : $EVAL_STATUS"
echo "holdout     : $HOLDOUT_STATUS"

if [ "$FMT_STATUS" -eq 0 ] &&
   [ "$TEST_STATUS" -eq 0 ] &&
   [ "$CLIPPY_STATUS" -eq 0 ] &&
   [ "$ADVERSARIAL_STATUS" -eq 0 ] &&
   [ "$SCAN_STATUS" -eq 0 ] &&
   [ "$EVAL_STATUS" -eq 0 ] &&
   [ "$HOLDOUT_STATUS" -eq 0 ]; then
    echo
    echo "TDI-1 REPRODUCTION: SUCCESS"
    exit 0
fi

echo
echo "TDI-1 REPRODUCTION: FAILURE"
exit 1
