#!/usr/bin/env bash
set -euo pipefail

mode=${1:?check mode required}

case "$mode" in
  formatting)
    cargo fmt --all -- --check
    ;;
  tests)
    cargo test --workspace --locked
    ;;
  clippy)
    cargo clippy --workspace --all-targets --locked -- -D warnings
    ;;
  integrity)
    sha256sum -c docs/TDI-2-CONTINUOUS-PREREGISTRATION.sha256
    sha256sum -c docs/TDI-3-INTERWIDTH-PREREGISTRATION.sha256
    sha256sum -c docs/TDI-3-INTERWIDTH-EVALUATOR.sha256
    sha256sum -c docs/TDI-4-TARGET-GEOMETRY-PREREGISTRATION.sha256
    sha256sum -c docs/TDI-4-TARGET-GEOMETRY-EVALUATOR.sha256
    bash -n scripts/reproduce-tdi4.sh
    test -s docs/TDI-3-SCIENTIFIC-CODE.sha256
    test -s docs/TDI-4-SCIENTIFIC-CODE.sha256
    if test -f docs/TDI-7.0-ATTENTION-RECOVERY-PREREGISTRATION.sha256; then
      sha256sum -c docs/TDI-7.0-ATTENTION-RECOVERY-PREREGISTRATION.sha256
    fi
    if test -f docs/TDI-8.0-ASSR-PREREGISTRATION.md; then
      bash -n scripts/check-tdi8-bootstrap.sh
      bash scripts/check-tdi8-bootstrap.sh
    fi
    ;;
  public-formatting)
    shopt -s nullglob
    files=(tdi-bench/src/bin/tdi-attention-v71*.rs)
    for example in \
      tdi-ai/examples/tdi7_features.rs \
      tdi-ai/examples/tdi7_evidence_schema.rs \
      tdi-ai/examples/tdi7_arming_decision.rs \
      tdi-ai/examples/tdi7_seed_selection_decision.rs \
      tdi-ai/examples/tdi7_rejection_policy_decision.rs \
      tdi-ai/examples/tdi7_final_holdout.rs; do
      if test -f "$example"; then
        files+=("$example")
      fi
    done
    test ${#files[@]} -gt 0
    rustfmt --edition 2024 --check "${files[@]}"
    ;;
  public-tests)
    cargo test --workspace --locked
    ;;
  public-clippy)
    shopt -s nullglob
    bins=()
    for file in tdi-bench/src/bin/tdi-attention-v71*.rs; do
      bins+=(--bin "$(basename "$file" .rs)")
    done
    test ${#bins[@]} -gt 0
    cargo clippy -p tdi-bench "${bins[@]}" --locked -- -D warnings
    for example in \
      tdi7_features \
      tdi7_evidence_schema \
      tdi7_arming_decision \
      tdi7_seed_selection_decision \
      tdi7_rejection_policy_decision \
      tdi7_final_holdout; do
      if test -f "tdi-ai/examples/${example}.rs"; then
        cargo clippy -p tdi-ai --example "$example" --locked -- -D warnings
      fi
    done
    ;;
  public-integrity)
    sha256sum -c docs/TDI-2-CONTINUOUS-PREREGISTRATION.sha256
    sha256sum -c docs/TDI-3-INTERWIDTH-PREREGISTRATION.sha256
    sha256sum -c docs/TDI-3-INTERWIDTH-EVALUATOR.sha256
    sha256sum -c docs/TDI-4-TARGET-GEOMETRY-PREREGISTRATION.sha256
    sha256sum -c docs/TDI-4-TARGET-GEOMETRY-EVALUATOR.sha256
    bash -n scripts/reproduce-tdi4.sh
    test -s docs/TDI-3-SCIENTIFIC-CODE.sha256
    test -s docs/TDI-4-SCIENTIFIC-CODE.sha256
    if test -f docs/TDI-7.0-ATTENTION-RECOVERY-PREREGISTRATION.sha256; then
      sha256sum -c docs/TDI-7.0-ATTENTION-RECOVERY-PREREGISTRATION.sha256
    fi
    if test -f docs/TDI-8.0-ASSR-PREREGISTRATION.md; then
      bash -n scripts/check-tdi8-bootstrap.sh
      bash scripts/check-tdi8-bootstrap.sh
    fi
    if test -f docs/TDI-9.0-ADAPTIVE-INFERENCE-PREREGISTRATION.md; then
      bash -n scripts/check-tdi9-bootstrap.sh
      bash scripts/check-tdi9-bootstrap.sh
    fi
    ;;
  msrv)
    mapfile -t versions < <(sed -n 's/^rust-version = "\([^"]*\)"$/\1/p' Cargo.toml)
    if test "${#versions[@]}" -ne 1 || test -z "${versions[0]}"; then
      echo "Cargo.toml must declare exactly one workspace rust-version" >&2
      exit 1
    fi
    cargo +"${versions[0]}" check --workspace --all-targets --locked
    ;;
  *)
    echo "unknown priority check mode: $mode" >&2
    exit 2
    ;;
esac
