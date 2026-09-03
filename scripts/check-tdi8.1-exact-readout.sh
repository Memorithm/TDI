#!/usr/bin/env bash
set -euo pipefail

fail() {
    echo "TDI-8.1 exact readout ERROR: $*" >&2
    exit 1
}

SOURCE="tdi-ai/src/task_readout.rs"
ENCODING="tdi-ai/src/task_encoding.rs"
BIN="tdi-ai/src/bin/tdi8-task-readout-preflight/main.rs"
DOC="docs/TDI-8.1-EXACT-READOUT-PREFLIGHT.md"

for file in "$SOURCE" "$ENCODING" "$BIN" "$DOC"; do
    test -s "$file" || fail "missing required exact-readout surface: $file"
done

grep -Fq 'pub struct ExactStateReadoutLayout' "$SOURCE" \
    || fail "caller-selected readout layout missing"
grep -Fq 'pub fn decode_state(self, state: &[f64])' "$SOURCE" \
    || fail "state-only readout API drifted"
grep -Fq 'ExactU64Binary64::decode(coordinates)' "$SOURCE" \
    || fail "canonical PR #112 decoder is not reused"
grep -Fq 'StateWidthMismatch' "$SOURCE" \
    || fail "runtime state-width drift is not rejected"
grep -Fq 'NonFiniteState' "$SOURCE" \
    || fail "non-finite recurrent state is not rejected"
grep -Fq 'DuplicateReadoutIndex' "$SOURCE" \
    || fail "duplicate readout coordinates are not rejected"
grep -Fq 'no target argument' "$DOC" \
    || fail "target-blind readout boundary is not documented"
grep -Fq 'no candidate vocabulary' "$DOC" \
    || fail "candidate-vocabulary exclusion is not documented"
grep -Fq 'No default positions are provided.' "$DOC" \
    || fail "readout coordinate non-freeze boundary is missing"

# The implementation API must not accept an expected/target symbol or a symbol
# candidate set. Comments/documentation can discuss those concepts; inspect only
# the function declaration block used by the arm-facing readout.
python3 - <<'PY'
from pathlib import Path
source = Path("tdi-ai/src/task_readout.rs").read_text()
needle = "pub fn decode_state(self, state: &[f64])"
if needle not in source:
    raise SystemExit("decode_state signature not found")
start = source.index(needle)
end = source.index("{", start)
signature = source[start:end]
for forbidden in ("target", "expected", "candidates", "vocabulary"):
    if forbidden in signature:
        raise SystemExit(f"forbidden readout argument in signature: {forbidden}")
PY

cargo test --locked -p tdi-ai --bin tdi8-task-readout-preflight

printf 'TDI-8.1 exact readout target input: ABSENT\n'
printf 'TDI-8.1 exact readout candidate vocabulary: ABSENT\n'
printf 'TDI-8.1 exact readout coordinate defaults: ABSENT\n'
printf 'TDI-8.1 canonical two-limb decoder reuse: VERIFIED\n'
printf 'TDI-8.1 exact readout preflight: PASS\n'
