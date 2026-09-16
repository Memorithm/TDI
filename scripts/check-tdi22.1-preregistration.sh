#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"

bash scripts/check-tdi22-freeze.sh

required=(
  docs/TDI-22.1-PREREGISTRATION.md
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
grep -Fq 'TDI-22.2 execution blocked' docs/TDI-22.1-STATUS.md

# Preregistration must precede any TDI-22.2 evaluator/result surface.
if find tdi-ai examples results artifacts -type f 2>/dev/null \
  \( -iname '*tdi22.2*' -o -iname '*tdi-22.2*' -o -iname '*tdi22_2*' \) \
  -print -quit | grep -q .; then
  echo 'TDI-22.1 preregistration cannot coexist with a TDI-22.2 evaluator/result surface' >&2
  exit 1
fi

# No final/confirmatory TDI-22 material at Stage 1.
if find . -type f 2>/dev/null \
  \( -iname '*tdi22*final*' -o -iname '*tdi-22*final*' -o -iname '*tdi22*confirm*' -o -iname '*tdi-22*confirm*' \) \
  -print -quit | grep -q .; then
  echo 'TDI-22 final/confirmatory material is forbidden during TDI-22.1' >&2
  exit 1
fi

echo 'TDI-22.1 preregistration integrity: OK'
