#!/usr/bin/env bash
set -euo pipefail

sha256sum -c docs/TDI-24-STAGE-A-SCIENTIFIC.sha256
cargo +1.97.1 fmt --all -- --check
cargo +1.97.1 clippy -p tdi-ai --features experimental --all-targets -- -D warnings
cargo +1.97.1 test -p tdi-ai --features experimental tdi24_chiral
cargo +1.97.1 test -p tdi-ai --features experimental tdi24_vector
cargo +1.97.1 test -p tdi-ai --features experimental tdi24_attention
cargo +1.97.1 test -p tdi-ai --features experimental tdi24_accounting
cargo +1.97.1 test -p tdi-ai --features experimental tdi24_stage_a

grep -Fq 'tdi24-stage-a-v1' tdi-ai/src/tdi24_stage_a.rs
grep -Fq 'tdi24-mirror-coupled-chiral-v1' tdi-ai/src/tdi24_stage_a.rs
grep -Fq 'tdi24-matched-vector6-v1' tdi-ai/src/tdi24_stage_a.rs
grep -Fq 'tdi24-channel-decomposition-v1' tdi-ai/src/tdi24_stage_a.rs
grep -Fq 'tdi24-enantiomorphic-score-pair-v1' tdi-ai/src/tdi24_stage_a.rs
grep -Fq 'tdi24-parity-recombination-v1' tdi-ai/src/tdi24_stage_a.rs
grep -Fq 'tdi24-masked-softmax-reference-v1' tdi-ai/src/tdi24_stage_a.rs
grep -Fq 'tdi24-attention-mask-reference-v1' tdi-ai/src/tdi24_stage_a.rs
grep -Fq 'tdi24-reference-accounting-v1' tdi-ai/src/tdi24_stage_a.rs

if grep -En 'todo!|unimplemented!' \
  tdi-ai/src/tdi24_chiral.rs \
  tdi-ai/src/tdi24_vector.rs \
  tdi-ai/src/tdi24_attention.rs \
  tdi-ai/src/tdi24_accounting.rs \
  tdi-ai/src/tdi24_stage_a.rs; then
  echo 'Stage-A Rust surface contains an unfinished implementation marker' >&2
  exit 1
fi

git diff --check
