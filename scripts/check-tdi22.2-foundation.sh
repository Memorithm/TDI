#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"

bash scripts/check-tdi22.1-freeze.sh

grep -Fq '#[path = "tdi22_eval.rs"]' tdi-ai/src/experimental.rs
grep -Fq 'pub mod tdi22_eval;' tdi-ai/src/experimental.rs
test -s tdi-ai/src/tdi22_eval.rs

grep -Fq 'pub const DEVELOPMENT_DOMAIN: u64 = 0x5444_4932_3244_4556;' tdi-ai/src/tdi22_eval.rs
grep -Fq 'pub const VALIDATION_DOMAIN: u64 = 0x5444_4932_3256_414c;' tdi-ai/src/tdi22_eval.rs
grep -Fq 'pub const MAX_GEOMETRY_INDEX: u64 = 255;' tdi-ai/src/tdi22_eval.rs
grep -Fq 'pub const MAX_SUPPLIED_POSITION_ABS: f64 = 1.75;' tdi-ai/src/tdi22_eval.rs
grep -Fq 'self.next_u64() % 15' tdi-ai/src/tdi22_eval.rs
grep -Fq 'factorized_resultant_dual' tdi-ai/src/tdi22_eval.rs
grep -Fq 'AmbiguousT1Top' tdi-ai/src/tdi22_eval.rs
grep -Fq 'dynamic_state_bits' tdi-ai/src/tdi22_eval.rs
grep -Fq 'static_parameter_bits' tdi-ai/src/tdi22_eval.rs
grep -Fq 'pub fn encode_line(&self) -> String' tdi-ai/src/tdi22_eval.rs

cargo test -p tdi-ai --features experimental tdi22_eval

if find results artifacts -type f 2>/dev/null \
  \( -iname '*tdi22*' -o -iname '*tdi-22*' \) -print -quit | grep -q .; then
  echo 'TDI-22.2 foundation must not commit result/artifact payloads' >&2
  exit 1
fi

echo 'TDI-22.2 evaluator foundation: OK'
