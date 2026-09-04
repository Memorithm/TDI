#!/usr/bin/env bash
set -euo pipefail

cargo +1.97.1 fmt --all -- --check
cargo +1.97.1 clippy -p tdi-operator --all-targets -- -D warnings
cargo +1.97.1 test -p tdi-operator --test cavity_pointwise_subunit_counterexample
cargo +1.97.1 test -p tdi-operator

grep -Fq 'alpha_k = 1 - 1/(k+1)^2 = k(k+2)/(k+1)^2' \
    docs/tdi10/TDI-10.5-POINTWISE-SUBUNIT-COUNTEREXAMPLE.md
grep -Fq '= (n+2)/(2(n+1))' \
    docs/tdi10/TDI-10.5-POINTWISE-SUBUNIT-COUNTEREXAMPLE.md
grep -Fq 'COUNTEREXAMPLE/falsification boundary' \
    docs/tdi10/TDI-10.5-POINTWISE-SUBUNIT-COUNTEREXAMPLE.md

if grep -Eiq \
    'pointwise[^\n]*(subunit|alpha[^\n]*<[^\n]*1)[^\n]*(proves|implies|guarantees)[^\n]*(uniform contraction|boundary decay|convergence)|TDI-10\.5[^\n]*(proves|establishes)[^\n]*(convergence theorem|uniform contraction)' \
    docs/tdi10/TDI-10.5-POINTWISE-SUBUNIT-COUNTEREXAMPLE.md \
    tdi-operator/tests/cavity_pointwise_subunit_counterexample.rs; then
  echo 'TDI-10.5 scientific-boundary gate rejected an unsupported convergence claim' >&2
  exit 1
fi
