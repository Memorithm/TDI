#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"

required=(
  "docs/TDI-22-PROGRAMME.md"
  "docs/TDI-22.0-SCOPE.md"
  "docs/TDI-22.0-STATUS.md"
  "tdi-ai/src/tdi22_torsor.rs"
)

for path in "${required[@]}"; do
  test -f "$path" || {
    echo "missing TDI-22 bootstrap file: $path" >&2
    exit 1
  }
done

grep -Fq 'active Stage-0 bootstrap; not frozen; no confirmatory execution authorised' \
  docs/TDI-22-PROGRAMME.md
grep -Fq 'no confirmatory execution authorised' docs/TDI-22.0-SCOPE.md
grep -Fq 'no confirmatory execution authorised' docs/TDI-22.0-STATUS.md
grep -Fq 'pub mod tdi22_torsor;' tdi-ai/src/experimental.rs
grep -Fq 'tdi22-torsor-dual-pairing-v1' tdi-ai/src/tdi22_torsor.rs

echo 'TDI-22 bootstrap structure: OK'
cargo test -p tdi-ai --features experimental tdi22_torsor
