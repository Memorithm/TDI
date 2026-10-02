#!/usr/bin/env bash
set -euo pipefail

required=(
  "docs/TDI-27-PROGRAMME.md"
  "docs/TDI-27-AFFECTIVE-CAUSALITY.md"
  "tdi-bench/src/affect_v27.rs"
  "tdi-bench/src/concept_geometry_v27.rs"
  "tdi-bench/src/bin/tdi27_concept_geometry.rs"
)

for path in "${required[@]}"; do
  test -s "$path" || {
    echo "missing required TDI-27 bootstrap surface: $path" >&2
    exit 1
  }
done

grep -Fq "confirmatory_execution_authorized: false" docs/TDI-27-PROGRAMME.md
grep -Fq "final_execution_authorized: false" docs/TDI-27-PROGRAMME.md
grep -Fq "synthetic_development_only" tdi-bench/src/bin/tdi27_concept_geometry.rs
grep -Fq "statistical_decision_pinned=false" tdi-bench/src/bin/tdi27_concept_geometry.rs
grep -Fq "confirmatory_result=false" tdi-bench/src/bin/tdi27_concept_geometry.rs

if git ls-files | grep -E '(^|/)(TDI-27|tdi27).*(FINAL|final).*(RESULT|result)' >/dev/null; then
  echo "TDI-27 Stage 0 must not contain a final result surface" >&2
  exit 1
fi

# Check application authorization before starting any software tests.
grep -Fxq "synthetic_development_execution_authorized: true" docs/TDI-27-AFFECTIVE-CAUSALITY.md
grep -Fxq "real_model_execution_authorized: false" docs/TDI-27-AFFECTIVE-CAUSALITY.md
grep -Fxq "confirmatory_execution_authorized: false" docs/TDI-27-AFFECTIVE-CAUSALITY.md
grep -Fxq "final_execution_authorized: false" docs/TDI-27-AFFECTIVE-CAUSALITY.md

# Library-only filters retain all TDI-27 tests without launching historical
# binary test harnesses, even when those harnesses would run zero tests.
cargo test --locked -p tdi-bench --lib concept_geometry_v27
cargo test --locked -p tdi-bench --lib affect_v27
