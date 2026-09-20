#!/usr/bin/env bash
set -euo pipefail

cargo test --locked -p tdi-ai --lib --all-features tdi2_template_learning
cargo test --locked -p tdi-ai --lib --all-features tdi2_analogical_mapping
cargo test --locked -p tdi-ai --lib --all-features tdi2_cross_domain

printf '%s\n' \
  'tdi2.2-cycle3-summary-v1;stage_c=template-induction;stage_d=automatic-mapping;stage_e=cross-domain;primary_holdout=closed'
