#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"

expected="c28fcd4fb5aacff62409d5ade87b5a05479e178e"
actual="$(git hash-object docs/TDI-22.1A-T1-VALUE-METRIC-AMENDMENT.md)"
[[ "$actual" == "$expected" ]] || {
  echo 'TDI-22.1A T1 value amendment hash mismatch' >&2
  echo "expected: $expected" >&2
  echo "actual:   $actual" >&2
  exit 1
}

bash scripts/check-tdi22.2-generator.sh

grep -Fq 'tdi22-t1-value-record-v1' docs/TDI-22.1A-T1-VALUE-METRIC-AMENDMENT.md
grep -Fq 'Exactly one companion line must exist for every T1 base record' docs/TDI-22.1A-T1-VALUE-METRIC-AMENDMENT.md
grep -Fq 'independent of identity equality' docs/TDI-22.1A-T1-VALUE-METRIC-AMENDMENT.md
grep -Fq 'before any TDI-22.2 Development/Validation campaign evidence is persisted' docs/TDI-22.1A-T1-VALUE-METRIC-AMENDMENT.md

if find results artifacts -type f 2>/dev/null \
  \( -iname '*tdi22*' -o -iname '*tdi-22*' \) -print -quit | grep -q .; then
  echo 'TDI-22.1A must be frozen before TDI-22 campaign evidence is persisted' >&2
  exit 1
fi

echo 'TDI-22.1A T1 value-reconstruction amendment: OK'
