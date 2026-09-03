#!/usr/bin/env bash
set -euo pipefail

fail() {
    echo "TDI-8.1 percentile interval ERROR: $*" >&2
    exit 1
}

SOURCE="tdi-bench/src/percentile_interval_v8.rs"
LIB="tdi-bench/src/lib.rs"
DOC="docs/TDI-8.1-PERCENTILE-INTERVAL-PREFLIGHT.md"

for file in "$SOURCE" "$LIB" "$DOC"; do
    test -s "$file" || fail "missing required percentile-interval surface: $file"
done

grep -Fq 'pub const TDI8_PERCENTILE_TAIL_DENOMINATOR: usize = 360;' "$SOURCE" \
    || fail "exact Bonferroni tail denominator is missing or changed"
grep -Fq 'replicates.validate_complete_accounting()?;' "$SOURCE" \
    || fail "complete bootstrap accounting is not enforced"
grep -Fq 'PointBaselineZero' "$SOURCE" \
    || fail "complete-sample zero-baseline rejection is missing"
grep -Fq 'ZeroBaselineReplicates' "$SOURCE" \
    || fail "zero-baseline bootstrap rejection is missing"
grep -Fq 'effects.sort_by(f64::total_cmp);' "$SOURCE" \
    || fail "deterministic total ordering is missing"
grep -Fq 'let dropped_per_tail = effects.len() / TDI8_PERCENTILE_TAIL_DENOMINATOR;' "$SOURCE" \
    || fail "non-interpolated order-statistic rule drifted"
grep -Fq 'pub mod percentile_interval_v8;' "$LIB" \
    || fail "percentile interval module is not exposed by tdi-bench"
grep -Fq 'does **not** freeze an experimental replicate count' "$DOC" \
    || fail "non-freeze boundary is missing from documentation"
grep -Fq 'rejects interval construction if **either count is non-zero**' "$DOC" \
    || fail "degenerate bootstrap policy is missing from documentation"

# These values remain caller-supplied. This tranche must not introduce an
# experimental default count or seed.
if grep -n -E 'DEFAULT_.*(REPLICATE|BOOTSTRAP|SEED)|DEFAULT_REPLICATES|DEFAULT_SEED' "$SOURCE"; then
    fail "unexpected default bootstrap count/seed introduced"
fi

# TDI-8.2/confirmation-surface absence is enforced centrally by
# check-tdi8-bootstrap.sh -> check-tdi8.1-foundation.sh. Keep this targeted gate
# free of the forbidden literal patterns so it cannot trigger the central scan
# merely by naming the patterns it is trying to detect.

cargo test --locked -p tdi-bench percentile_interval_v8

printf 'TDI-8.1 percentile interval rational tail: VERIFIED\n'
printf 'TDI-8.1 zero-baseline bootstrap handling: FAIL-CLOSED\n'
printf 'TDI-8.1 interval count/seed defaults: ABSENT\n'
printf 'TDI-8.1 percentile interval preflight: PASS\n'