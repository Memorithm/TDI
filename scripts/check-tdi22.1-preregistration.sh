#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"

bash scripts/check-tdi22-freeze.sh

required=(
  docs/TDI-22.1-PREREGISTRATION.md
  docs/TDI-22.1-ARM-CONTRACT.md
  docs/TDI-22.1-STATUS.md
)
for path in "${required[@]}"; do
  test -s "$path" || { echo "missing TDI-22.1 preregistration file: $path" >&2; exit 1; }
done

grep -Fq 'T3 versus T4' docs/TDI-22.1-PREREGISTRATION.md
grep -Fq 'F1 — transport-consistent geometric retrieval' docs/TDI-22.1-PREREGISTRATION.md
grep -Fq 'F2 — generic six-dimensional bilinear retrieval' docs/TDI-22.1-PREREGISTRATION.md
grep -Fq 'F3 — position-nuisance associative retrieval' docs/TDI-22.1-PREREGISTRATION.md
grep -Fq 'tdi22/dev/v1' docs/TDI-22.1-PREREGISTRATION.md
grep -Fq 'tdi22/validation/v1' docs/TDI-22.1-PREREGISTRATION.md
grep -Fq 'score_T0 = v.R + omega.M(P)' docs/TDI-22.1-ARM-CONTRACT.md
grep -Fq 'score_T4 = v.R + omega.C' docs/TDI-22.1-ARM-CONTRACT.md
grep -Fq 'score_T3 - score_T4 = (Q x omega).R' docs/TDI-22.1-ARM-CONTRACT.md

if [[ -s docs/TDI-22.1-FREEZE.md ]]; then
  grep -Fq 'bounded non-final TDI-22.2' docs/TDI-22.1-STATUS.md
else
  grep -Fq 'TDI-22.2 execution blocked' docs/TDI-22.1-STATUS.md
  if find tdi-ai examples -type f 2>/dev/null \
    \( -iname '*tdi22.2*' -o -iname '*tdi-22.2*' -o -iname '*tdi22_2*' \) \
    -print -quit | grep -q .; then
    echo 'TDI-22.2 source is forbidden before the Stage-1 freeze' >&2
    exit 1
  fi
fi

# Stage 1 never authorizes committed evidence payloads.
if find results artifacts -type f 2>/dev/null \
  \( -iname '*tdi22*' -o -iname '*tdi-22*' \) -print -quit | grep -q .; then
  echo 'TDI-22 committed result/artifact payloads are forbidden during Stage 1' >&2
  exit 1
fi

if find . -type f 2>/dev/null \
  \( -iname '*tdi22*final*' -o -iname '*tdi-22*final*' -o -iname '*tdi22*confirm*' -o -iname '*tdi-22*confirm*' \) \
  -print -quit | grep -q .; then
  echo 'TDI-22 final/confirmatory material is forbidden during Stage 1' >&2
  exit 1
fi

echo 'TDI-22.1 preregistration integrity: OK'
