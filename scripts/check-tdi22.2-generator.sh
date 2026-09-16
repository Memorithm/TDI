#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"

bash scripts/check-tdi22.2-foundation.sh

grep -Fq '#[path = "tdi22_generator.rs"]' tdi-ai/src/experimental.rs
grep -Fq 'pub mod tdi22_generator;' tdi-ai/src/experimental.rs
test -s tdi-ai/src/tdi22_generator.rs

grep -Fq 'pub fn generate_episode(' tdi-ai/src/tdi22_generator.rs
grep -Fq 'target selection occurs entirely in content' tdi-ai/src/tdi22_generator.rs
grep -Fq 'RetryBudgetExhausted' tdi-ai/src/tdi22_generator.rs
grep -Fq 'reconstructed_local_moment' tdi-ai/src/tdi22_generator.rs

cargo test -p tdi-ai --features experimental tdi22_generator

if find results artifacts -type f 2>/dev/null \
  \( -iname '*tdi22*' -o -iname '*tdi-22*' \) -print -quit | grep -q .; then
  echo 'TDI-22.2 generator must not commit result/artifact payloads' >&2
  exit 1
fi

if find . -type f 2>/dev/null \
  \( -iname '*tdi22*final*' -o -iname '*tdi-22*final*' -o -iname '*tdi22*confirm*' -o -iname '*tdi-22*confirm*' \) \
  -print -quit | grep -q .; then
  echo 'TDI-22 final/confirmatory material remains forbidden' >&2
  exit 1
fi

echo 'TDI-22.2 deterministic P1-P5 generator: OK'
