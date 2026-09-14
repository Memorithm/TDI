#!/usr/bin/env bash
# Development-only software qualification. Does not access any final holdout.
set -euo pipefail
root="$(git rev-parse --show-toplevel)"
cd "$root"
if [[ -n "$(git status --porcelain --untracked-files=normal)" ]]; then
  echo 'TDI-21 qualification requires a clean checkout.' >&2
  exit 1
fi
export TDI21_SOURCE_SHA="$(git rev-parse HEAD)"
printf 'source_sha=%s\n' "$TDI21_SOURCE_SHA"
rustc --version --verbose
sha256sum Cargo.lock Cargo.toml tdi-ai/Cargo.toml \
  tdi-ai/src/experimental.rs tdi-ai/src/tdi21_boolean_relational.rs \
  tdi-ai/src/tdi21_stream.rs tdi-ai/src/tdi21_provenance.rs \
  tdi-ai/src/tdi21_evaluation.rs tdi-ai/examples/tdi21_development.rs \
  tdi-ai/tests/tdi21_evaluation_contract.rs scripts/check-tdi21-development.sh
cargo test --locked -p tdi-ai --features experimental --test tdi21_audit_regressions
cargo test --locked -p tdi-ai --features experimental --test tdi21_stream_contract
cargo test --locked -p tdi-ai --features experimental --test tdi21_memory_tradeoffs -- --nocapture
cargo test --locked -p tdi-ai --features experimental --test tdi21_evaluation_contract
cargo test --locked --release -p tdi-ai --features experimental --test tdi21_audit_regressions
cargo test --locked --release -p tdi-ai --features experimental --test tdi21_stream_contract
cargo test --locked --release -p tdi-ai --features experimental --test tdi21_memory_tradeoffs -- --nocapture
cargo test --locked --release -p tdi-ai --features experimental --test tdi21_evaluation_contract
scratch="$(mktemp -d)"
trap 'rm -rf -- "$scratch"' EXIT
for trial in 1 2; do
  cargo run --locked --release -p tdi-ai --features experimental \
    --example tdi21_development > "$scratch/$trial.txt"
done
cmp "$scratch/1.txt" "$scratch/2.txt"
cat "$scratch/1.txt"
if [[ "$(git rev-parse HEAD)" != "$TDI21_SOURCE_SHA" ]] \
  || [[ -n "$(git status --porcelain --untracked-files=normal)" ]]; then
  echo 'TDI-21 qualification changed the source checkout; refusing evidence.' >&2
  exit 1
fi
echo 'TDI-21 development replay: identical output; no confirmatory claim.'
