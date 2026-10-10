#!/usr/bin/env bash
set -euo pipefail
cargo +1.97.1 fmt --all -- --check
rustfmt +1.97.1 --edition 2024 --check tdi-ai/src/tdi25_matched_reference.rs
cargo +1.97.1 clippy -p tdi-ai --features experimental --all-targets -- -D warnings
cargo +1.97.1 test -p tdi-ai --features experimental --lib tdi25_eval
cargo +1.97.1 test -p tdi-ai --features experimental --test tdi25_reference_cost
grep -Fq 'tdi25-reference-cost-v1' tdi-ai/src/tdi25_eval.rs
grep -Fq 'pub const T430_QUALIFIED_TIMING_ENVIRONMENT: QualifiedTimingEnvironment =' tdi-ai/src/tdi25_eval.rs
tr -d ' \n' < tdi-ai/src/tdi25_eval.rs | grep -Fq 'pubconstQUALIFIED_TIMING_ENVIRONMENT:Option<QualifiedTimingEnvironment>=Some(T430_QUALIFIED_TIMING_ENVIRONMENT);'
grep -Fq 'pub const REFERENCE_TIMING_STATUS: &str = "timing_non_qualifie";' tdi-ai/src/tdi25_eval.rs
grep -Fq 'pub const fn t6_pair_accounting(' tdi-ai/src/tdi25_eval.rs
grep -Fq 'pub const fn c6_pair_accounting(' tdi-ai/src/tdi25_eval.rs
grep -Fq 'pub fn run_qualified_reference_timing_cell(' tdi-ai/src/tdi25_eval.rs
grep -Fq 'pub fn attest_timing_environment(' tdi-ai/src/tdi25_eval.rs
grep -Fq 'pub fn summarize_timing_cell(' tdi-ai/src/tdi25_eval.rs
grep -Fq 'cost_invalid("attestation_pinning")' tdi-ai/src/tdi25_eval.rs
grep -Fq 'cost_invalid("timing_environment_not_qualified")' tdi-ai/src/tdi25_eval.rs
grep -Fq 'cost_invalid("timing_status_drift")' tdi-ai/src/tdi25_eval.rs
grep -Fq 'cost_invalid("case_evidence_drift")' tdi-ai/src/tdi25_eval.rs
grep -Fq 'fn timing_harness_refuses_on_an_unattested_host' tdi-ai/tests/tdi25_reference_cost.rs
grep -Fq 'fn qualified_environment_is_the_frozen_t430_manifest' tdi-ai/tests/tdi25_reference_cost.rs
grep -Fq 'tdi25-reference-cost-v1' docs/TDI-25-REFERENCE-COST-V1.md
grep -Fq 'Timing non qualifié' docs/TDI-25-REFERENCE-COST-V1.md
grep -Fq 'Recorded degeneracies' docs/TDI-25-REFERENCE-COST-V1.md
grep -Fq 'tdi25-qualified-timing-environment-v1' docs/TDI-25-QUALIFIED-TIMING-ENVIRONMENT-V1.md
grep -Fq 'Recorded degeneracies' docs/TDI-25-QUALIFIED-TIMING-ENVIRONMENT-V1.md
grep -Fq 'timing non qualifié' docs/TDI-25-QUALIFIED-TIMING-ENVIRONMENT-V1.md
for value in 'environment_contract: tdi25-qualified-timing-environment-v1' 'bios_version: "2.19.0"' \
  'kernel_release: 6.12.88+deb13-amd64' 'rustc: "1.97.1"' 'pinned_cpu: 28' 'numa_membind: 0' \
  'smt_sibling_cpu: 60' 'governor: performance' 'intel_pstate_no_turbo: 1' 'warmup_runs: 5' \
  'measured_runs: 31' 'iqr_over_median_max: 0.05' 'load_average_1min_max: 2.0'; do
  grep -Fq "$value" docs/tdi25-qualified-timing-environment.yaml
done
bash -n scripts/tdi25-s47-qualified-timing.sh
grep -Fq 'trap restore EXIT' scripts/tdi25-s47-qualified-timing.sh
grep -Fq 'numactl --membind=$NUMA_NODE taskset -c $PINNED_CPU' scripts/tdi25-s47-qualified-timing.sh
# Note: `! cmd` does not trip `set -e`; the timing-artifact guards use explicit exits.
timing_files="$(git ls-files | grep -E 'tdi25-s47-timing-|tdi25_s47_timing|results\.jsonl' || true)"
stray="$(grep -Ev '^results/tdi25_s47_timing/attempt-[0-9]+/(results\.jsonl|build-provenance\.txt|fingerprint\.json|SHA256SUMS)$' <<<"$timing_files" | grep -v '^$' || true)"
if [[ -n "$stray" ]]; then echo "stray timing artifacts: $stray" >&2; exit 1; fi
(cd results/tdi25_s47_timing/attempt-1 && sha256sum -c --quiet SHA256SUMS)
grep -Fq 'ab2db3cd203dcc29054616127fff7475ac73967f918deab8d60180f651cf2f40  results.jsonl' results/tdi25_s47_timing/attempt-1/SHA256SUMS
test "$(grep -c '"cell_status":"non_qualifiee"' results/tdi25_s47_timing/attempt-1/results.jsonl)" -eq 4
grep -Fq 'non qualifiée' docs/TDI-25-S47-TIMING-ATTEMPT-1.md
grep -Fq 'NON QUALIFIÉS' docs/TDI-25-S47-TIMING-ATTEMPT-1.md
grep -Fq 'Recorded degeneracies' docs/TDI-25-S47-TIMING-ATTEMPT-1.md
if grep -Fq '"cell_status":"qualifiee"' results/tdi25_s47_timing/attempt-1/results.jsonl; then exit 1; fi
! grep -Eq 'DataSplit::(Protected|Final|Holdout)' tdi-ai/src/tdi25_eval.rs tdi-ai/tests/tdi25_reference_cost.rs
grep -Eq '\| 46 \| Numerical precision study \| \*\*landed\*\* in #[0-9]+, exact head `[0-9a-f]{40}`, merge `[0-9a-f]{40}`;' docs/TDI-25-CAMPAIGN-50.md
grep -Eq '\| 47 \| Reference cost study \| \*\*(current candidate|landed)[^|]*timing non qualifié' docs/TDI-25-CAMPAIGN-50.md
merged_count="$(grep -Eo '\*\*[0-9]+/50 merged\*\*' docs/TDI-25-CAMPAIGN-50.md | head -n 1 | tr -dc '0-9/' | cut -d/ -f1)"
test "${merged_count:-0}" -ge 46
! grep -Fq '#PRNUM' docs/TDI-25-CAMPAIGN-50.md
