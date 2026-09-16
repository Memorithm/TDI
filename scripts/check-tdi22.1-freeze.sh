#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"

check_blob() {
  local path="$1" expected="$2" actual
  actual="$(git hash-object "$path")"
  [[ "$actual" == "$expected" ]] || {
    echo "TDI-22.1 freeze mismatch: $path" >&2
    echo "expected: $expected" >&2
    echo "actual:   $actual" >&2
    exit 1
  }
}

check_blob "docs/TDI-22.1-PREREGISTRATION.md" "fdeceed48229221b42bd6591550f060e594909cb"
check_blob "docs/TDI-22.1-ARM-CONTRACT.md" "08dfda1eda9f2d730080e74359474537b2e03b5a"
check_blob "docs/TDI-22.1-FREEZE.md" "d74e11ea56ec2e67d3daf50fc767243a75b01954"
check_blob "docs/TDI-22.1-SEED-CONTRACT.md" "b4fde0b5358e777e33ef04616ed28dded7d16421"
check_blob "docs/TDI-22.1-GENERATOR-CONTRACT.md" "e6b76c2f2a46090b277af9095827d0f6cc2d2a28"
check_blob "docs/TDI-22.1-RECORD-CONTRACT.md" "86eaed60b7fdda2945b53cf180c18da6d8a1fc7a"
check_blob "docs/TDI-22.1-IMPLEMENTATION-GATE.md" "2cae6ad10b613365afca96d7a92bb605000ca8c1"
check_blob "docs/TDI-22.1-STATUS.md" "4637e890d21601ada5776d2964e74b259feb946e"
check_blob "scripts/check-tdi22-freeze.sh" "b9873fa89e8b13d391a15cd10d899e00b85caa84"
check_blob "scripts/check-tdi22.1-preregistration.sh" "ca254d9bc15ab24f7c7a09a9178cf5da2b5d90b8"

bash scripts/check-tdi22.1-preregistration.sh

grep -Fq 'selector = (cell_id << 56) | episode_index' docs/TDI-22.1-SEED-CONTRACT.md
grep -Fq 'AmbiguousT1Top' docs/TDI-22.1-FREEZE.md
grep -Fq 'dynamic_state_bits' docs/TDI-22.1-RECORD-CONTRACT.md
grep -Fq 'static_parameter_bits' docs/TDI-22.1-RECORD-CONTRACT.md
grep -Fq 'ASCII TAB byte' docs/TDI-22.1-RECORD-CONTRACT.md
grep -Fq 'unsigned integer bit counts' docs/TDI-22.1-RECORD-CONTRACT.md
grep -Fq 'unavailable identity or score: literal `none`' docs/TDI-22.1-RECORD-CONTRACT.md
grep -Fq 'quarter_step(w) = ((w mod 15) - 7) / 4' docs/TDI-22.1-SEED-CONTRACT.md
grep -Fq 'worst frozen F1 transport' docs/TDI-22.1-FREEZE.md
grep -Fq 'M(P)=C-P x R' docs/TDI-22.1-GENERATOR-CONTRACT.md

if find results artifacts -type f 2>/dev/null \
  \( -iname '*tdi22*' -o -iname '*tdi-22*' \) -print -quit | grep -q .; then
  echo 'TDI-22.1 freeze forbids committed TDI-22 result/artifact payloads' >&2
  exit 1
fi

if find . -type f 2>/dev/null \
  \( -iname '*tdi22*final*' -o -iname '*tdi-22*final*' -o -iname '*tdi22*confirm*' -o -iname '*tdi-22*confirm*' \) \
  -print -quit | grep -q .; then
  echo 'TDI-22 final/confirmatory material is forbidden by the Stage-1 freeze' >&2
  exit 1
fi

echo 'TDI-22.1 content-addressed freeze: OK'
