#!/usr/bin/env bash
set -euo pipefail

fail() {
    echo "TDI-8.1 operation accounting ERROR: $*" >&2
    exit 1
}

SOURCE="tdi-ai/src/reference_operation_accounting.rs"
PREFLIGHT="tdi-ai/src/bin/tdi8-operation-accounting-preflight/main.rs"
DOC="docs/TDI-8.1-OPERATION-ACCOUNTING.md"

for file in "$SOURCE" "$PREFLIGHT" "$DOC"; do
    test -s "$file" || fail "missing operation-accounting qualification surface: $file"
done

grep -Fq 'pub struct ReferenceOperationAccounting' "$SOURCE" \
    || fail "component-wise accounting record missing"
grep -Fq 'pub enum ReferenceOperationAccountingError' "$SOURCE" \
    || fail "typed accounting rejection missing"
grep -Fq 'left.checked_add(right)' "$SOURCE" \
    || fail "checked-add overflow guard missing"
grep -Fq 'left.checked_mul(right)' "$SOURCE" \
    || fail "checked-multiply overflow guard missing"
grep -Fq 'pub fn a0_read(' "$SOURCE" \
    || fail "A0 read accounting missing"
grep -Fq 'pub fn a1_step(' "$SOURCE" \
    || fail "A1 recurrent accounting missing"
grep -Fq 'pub fn a2_step(' "$SOURCE" \
    || fail "A2 observed-step accounting missing"
grep -Fq 'matches!(report.read(), A2ReadStatus::Hit { .. })' "$SOURCE" \
    || fail "A2 hit-conditioned fusion accounting missing"
grep -Fq 'if report.write().is_some()' "$SOURCE" \
    || fail "A2 observed-write accounting missing"
grep -Fq 'pub fn a3_routed_step(' "$SOURCE" \
    || fail "A3 routed-read accounting missing"
grep -Fq 'A3VsaReadRoute::Key(_)' "$SOURCE" \
    || fail "A3 keyed VSA-read accounting missing"
grep -Fq 'pub fn a3_skip_and_store_step(' "$SOURCE" \
    || fail "A3 atomic-store accounting missing"
grep -Fq 'accounting.vsa_bind_terms = width;' "$SOURCE" \
    || fail "A3 bind accounting missing"
grep -Fq 'accounting.vsa_bundle_terms = width;' "$SOURCE" \
    || fail "A3 bundle accounting missing"
grep -Fq 'pub fn total(self)' "$SOURCE" \
    || fail "checked semantic total missing"
grep -Fq 'not CPU instructions, FLOPs, wall-clock time, energy, bandwidth' "$DOC" \
    || fail "hardware non-claim boundary missing"
grep -Fq 'TDI-8.2 seeds, runner, result payload or authorization surface' "$DOC" \
    || fail "TDI-8.2 absence boundary missing"

cargo test --locked -p tdi-ai --bin tdi8-operation-accounting-preflight
cargo run --locked -p tdi-ai --bin tdi8-operation-accounting-preflight

printf 'TDI-8.1 operation vocabulary: EXACT_REFERENCE_SEMANTIC_UNITS\n'
printf 'TDI-8.1 overflow policy: FAIL_CLOSED_U128\n'
printf 'TDI-8.1 hardware FLOP/runtime claim: ABSENT\n'
printf 'TDI-8.2 executable/token/result surface: ABSENT\n'
printf 'TDI-8.1 operation accounting gate: PASS\n'
