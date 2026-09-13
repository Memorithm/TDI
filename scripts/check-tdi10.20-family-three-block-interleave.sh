#!/usr/bin/env bash
set -euo pipefail

cargo +1.97.1 fmt --all -- --check
cargo +1.97.1 clippy -p tdi-operator --all-targets -- -D warnings
cargo +1.97.1 test -p tdi-operator --test cavity_family_three_block_interleave
cargo +1.97.1 test -p tdi-operator

grep -Fq 'EXACT claim 1 — Three-block multiplicative product identity' \
    docs/tdi10/TDI-10.20-FAMILY-THREE-BLOCK-INTERLEAVE.md
grep -Fq 'EXACT claim 2 — Mixed `{U,D,S}` permutation closed forms' \
    docs/tdi10/TDI-10.20-FAMILY-THREE-BLOCK-INTERLEAVE.md
grep -Fq 'EXACT claim 3 — Same-family three-block products under index restart' \
    docs/tdi10/TDI-10.20-FAMILY-THREE-BLOCK-INTERLEAVE.md
grep -Fq 'EXACT claim 4 — Interleaved pair-schedule products' \
    docs/tdi10/TDI-10.20-FAMILY-THREE-BLOCK-INTERLEAVE.md
grep -Fq 'P_{U→D→S}(ℓ,m,n;ρ) = ρ^ℓ · 1/(m+1) · (n+2)/(2(n+1))' \
    docs/tdi10/TDI-10.20-FAMILY-THREE-BLOCK-INTERLEAVE.md
grep -Fq 'P_{U→S→D}(ℓ,m,n;ρ) = ρ^ℓ · (m+2)/(2(m+1)) · 1/(n+1)' \
    docs/tdi10/TDI-10.20-FAMILY-THREE-BLOCK-INTERLEAVE.md
grep -Fq 'P_{D→U→S}(ℓ,m,n;ρ) = 1/(ℓ+1) · ρ^m · (n+2)/(2(n+1))' \
    docs/tdi10/TDI-10.20-FAMILY-THREE-BLOCK-INTERLEAVE.md
grep -Fq 'P_{D→S→U}(ℓ,m,n;ρ) = 1/(ℓ+1) · (m+2)/(2(m+1)) · ρ^n' \
    docs/tdi10/TDI-10.20-FAMILY-THREE-BLOCK-INTERLEAVE.md
grep -Fq 'P_{S→U→D}(ℓ,m,n;ρ) = (ℓ+2)/(2(ℓ+1)) · ρ^m · 1/(n+1)' \
    docs/tdi10/TDI-10.20-FAMILY-THREE-BLOCK-INTERLEAVE.md
grep -Fq 'P_{S→D→U}(ℓ,m,n;ρ) = (ℓ+2)/(2(ℓ+1)) · 1/(m+1) · ρ^n' \
    docs/tdi10/TDI-10.20-FAMILY-THREE-BLOCK-INTERLEAVE.md
grep -Fq 'P_{(UD)^k}(ρ) = (ρ/2)^k' \
    docs/tdi10/TDI-10.20-FAMILY-THREE-BLOCK-INTERLEAVE.md
grep -Fq 'P_{(US)^k}(ρ) = (3ρ/4)^k' \
    docs/tdi10/TDI-10.20-FAMILY-THREE-BLOCK-INTERLEAVE.md
grep -Fq 'P_{(DS)^k} = (3/8)^k' \
    docs/tdi10/TDI-10.20-FAMILY-THREE-BLOCK-INTERLEAVE.md
grep -Fq 'FrozenToeplitzCavity::contraction' \
    docs/tdi10/TDI-10.20-FAMILY-THREE-BLOCK-INTERLEAVE.md
grep -Fq 'REFUTED: three-block `{U,D,S}` order is immaterial' \
    docs/tdi10/TDI-10.20-FAMILY-THREE-BLOCK-INTERLEAVE.md
grep -Fq 'REFUTED: interleaving vs blocking is immaterial' \
    docs/tdi10/TDI-10.20-FAMILY-THREE-BLOCK-INTERLEAVE.md
grep -Fq 'REFUTED: index-restart bookkeeping is immaterial for Type-D / Type-S' \
    docs/tdi10/TDI-10.20-FAMILY-THREE-BLOCK-INTERLEAVE.md
grep -Fq 'REFUTED: Type-S middle block erases Type-U prefix decay uniformly in `ℓ`' \
    docs/tdi10/TDI-10.20-FAMILY-THREE-BLOCK-INTERLEAVE.md
grep -Fq 'REFUTED: finite Type-D prefix plus Type-S middle forces liminf 0 under Type-U of fixed `n`' \
    docs/tdi10/TDI-10.20-FAMILY-THREE-BLOCK-INTERLEAVE.md
grep -Fq 'indices restart at 1' \
    docs/tdi10/TDI-10.20-FAMILY-THREE-BLOCK-INTERLEAVE.md

if grep -Eiq \
    'soft-edge theorem|slowly varying Jacobi theorem|Riemann hypothesis|RH claim|proves slowly-varying' \
    docs/tdi10/TDI-10.20-FAMILY-THREE-BLOCK-INTERLEAVE.md \
    tdi-operator/tests/cavity_family_three_block_interleave.rs; then
  echo 'TDI-10.20 scientific-boundary gate rejected an unsupported claim' >&2
  exit 1
fi

printf 'TDI-10.20 three-block / interleaved family composition: PASS\n'
