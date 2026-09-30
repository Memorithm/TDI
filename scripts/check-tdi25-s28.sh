#!/usr/bin/env bash
set -euo pipefail
cargo +1.97.1 fmt --all -- --check
rustfmt +1.97.1 --edition 2024 --check tdi-ai/src/tdi25_matched_reference.rs
cargo +1.97.1 clippy -p tdi-ai --features experimental --all-targets -- -D warnings
cargo +1.97.1 test -p tdi-ai --features experimental --lib tdi25_eval
cargo +1.97.1 test -p tdi-ai --features experimental --test tdi25_production_pairing
cargo +1.97.1 test -p tdi-ai --features experimental --test tdi25_matched_reference
grep -Fq 'tdi25-family-stratified-synthesis-v1' tdi-ai/src/tdi25_eval.rs
grep -Fq 'pub fn synthesize_family_stratified_effects' tdi-ai/src/tdi25_eval.rs
grep -Fq 'pub fn synthesize_family_stratified_from_revealed_outcomes' tdi-ai/src/tdi25_eval.rs
grep -Fq 'pub struct FamilyStratifiedSynthesisReport' tdi-ai/src/tdi25_eval.rs
grep -Fq 'pub struct FamilySignedEffect' tdi-ai/src/tdi25_eval.rs
grep -Fq 'pub enum SignedEffectClass' tdi-ai/src/tdi25_eval.rs
! grep -Fq 'pub fn classify_signed_effect' tdi-ai/src/tdi25_eval.rs
grep -Fq 'sign_reversal_present' tdi-ai/src/tdi25_eval.rs
grep -Fq 'claims_clean_pooled_win' tdi-ai/src/tdi25_eval.rs
grep -Fq 'reason: "missing_family"' tdi-ai/src/tdi25_eval.rs
grep -Fq '| 27 | Paired uncertainty engine | **landed** in #683, exact head `9bf33373630ea7b7bdc338da359c9eb1fcc845d9`, merge `ff7de14f6f3968f2fb5679e96d962a5f651d43b4`;' docs/TDI-25-CAMPAIGN-50.md
grep -Fq '| 28 | Family-stratified synthesis | **current candidate**' docs/TDI-25-CAMPAIGN-50.md
grep -Fq '**27/50 merged**' docs/TDI-25-CAMPAIGN-50.md
! grep -Fq '#PRNUM' docs/TDI-25-CAMPAIGN-50.md
