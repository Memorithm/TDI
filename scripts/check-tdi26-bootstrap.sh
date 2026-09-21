#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"

required=(
  "docs/TDI-26-PROGRAMME.md"
  "docs/TDI-26.0-SCOPE.md"
  "docs/TDI-26.0-STATUS.md"
  "docs/TDI-26.0-IMPLEMENTATION-GATE.md"
  "tdi-ai/src/tdi26_v888.rs"
)

for path in "${required[@]}"; do
  test -f "$path" || {
    echo "missing TDI-26 bootstrap file: $path" >&2
    exit 1
  }
done

grep -Fq 'active Stage-0 bootstrap; not frozen; no final or confirmatory execution authorised'   docs/TDI-26.0-SCOPE.md
grep -Fq 'TDI-26.1 evaluator implementation remains blocked' docs/TDI-26.0-SCOPE.md
grep -Fq 'pub mod tdi26_v888;' tdi-ai/src/experimental.rs
grep -Fq 'tdi26-v888-stage0-v1' tdi-ai/src/tdi26_v888.rs
grep -Fq 'pub const TDI26_CONFIRMATORY_AUTHORIZED: bool = false;' tdi-ai/src/tdi26_v888.rs

if find "$root/results" "$root/artifacts" -type f   \( -iname '*tdi26*' -o -iname '*tdi-26*' \) -print -quit 2>/dev/null | grep -q .; then
  echo 'TDI-26 result/artifact payloads are not authorized by Stage 0' >&2
  exit 1
fi

cargo test -p tdi-ai --features experimental tdi26

echo 'TDI-26 Stage-0 bootstrap: OK'
