#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

check_blob() {
  local path="$1" expected="$2" actual
  actual="$(git -C "$root" hash-object "$path")"
  [[ "$actual" == "$expected" ]] || {
    echo "TDI-22.0 freeze mismatch: $path" >&2
    echo "expected: $expected" >&2
    echo "actual:   $actual" >&2
    exit 1
  }
}

check_blob "docs/TDI-22-PROGRAMME.md" "6649b39552aa04128fcc39980ffcbb8743be7d1a"
check_blob "docs/TDI-22.0-SCOPE.md" "47143b190b70cd59b1972c6fbb594ae4456c6e2e"
check_blob "tdi-ai/src/tdi22_torsor.rs" "419052f499ef83c1909248e12263f5ca846a3b96"
check_blob "tdi-ai/tests/tdi22_torsor_properties.rs" "42f3246223b39f8fbc69492b4350914c02d257ec"
check_blob "scripts/check-tdi22-bootstrap.sh" "7e535217eef2b01732a949336a2e347c7617c932"
check_blob ".github/workflows/tdi22-torsor-bootstrap.yml" "6c53e2fee3023325b649dea29ee0cb7a9c46d918"

test -s "$root/docs/TDI-22.0-FREEZE.md"
test -s "$root/docs/TDI-22.0-IMPLEMENTATION-GATE.md"
grep -Fq '## TDI-22.x torsor-attention bootstrap and stage gate' "$root/AGENTS.md"
grep -Fq 'T3 full-torsor versus T4 matched six-component non-torsor bilinear control' "$root/AGENTS.md"

# Evidence payloads remain forbidden until a later explicit results stage.
if find "$root/results" "$root/artifacts" -type f \
  \( -iname '*tdi22*' -o -iname '*tdi-22*' \) -print -quit 2>/dev/null | grep -q .; then
  echo 'TDI-22 result/artifact payloads are not authorized by Stage 0/1' >&2
  exit 1
fi

# Final/confirmatory material is forbidden at every currently authorized stage.
if find "$root" -type f \
  \( -iname '*tdi22*final*' -o -iname '*tdi-22*final*' -o -iname '*tdi22*confirm*' -o -iname '*tdi-22*confirm*' \) \
  -print -quit 2>/dev/null | grep -q .; then
  echo 'TDI-22 final/confirmatory material is forbidden' >&2
  exit 1
fi

# Before the Stage-1 freeze, no later Rust surface is allowed. Once that freeze
# exists, Stage-0 remains pinned while separately gated non-final Stage-2 source
# may coexist.
if [[ ! -s "$root/docs/TDI-22.1-FREEZE.md" ]]; then
  mapfile -t tdi22_rust < <(
    find "$root/tdi-ai" -type f \
      \( -name '*tdi22*.rs' -o -name '*tdi-22*.rs' \) -printf '%P\n' | sort
  )
  expected_rust=("src/tdi22_torsor.rs" "tests/tdi22_torsor_properties.rs")
  [[ "${tdi22_rust[*]}" == "${expected_rust[*]}" ]] || {
    echo 'unexpected TDI-22 Rust surface before Stage-1 freeze' >&2
    printf 'observed: %s\n' "${tdi22_rust[@]}" >&2
    exit 1
  }
fi

bash "$root/scripts/check-tdi22-bootstrap.sh"
cargo test -p tdi-ai --features experimental --test tdi22_torsor_properties

echo 'TDI-22.0 freeze integrity: OK'
