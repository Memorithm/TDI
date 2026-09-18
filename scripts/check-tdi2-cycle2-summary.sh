#!/usr/bin/env bash
set -euo pipefail

run_example() {
  cargo run --quiet --locked -p tdi-ai --example "$1" --features experimental
}

motif="$(run_example tdi2_intuition_validation)"
context="$(run_example tdi2_intuition_context_validation)"
temporal="$(run_example tdi2_intuition_temporal_transfer_validation)"
analogy="$(run_example tdi2_intuition_analogy_validation)"
replay="$(run_example tdi2_intuition_deterministic_replay)"

grep -Fq 'domain=validation;family=motif-retrieval;start=2000;count=34' <<<"$motif"
grep -Fq 'total=34;selected=34;correct=34;incorrect=0;abstained=0' <<<"$motif"

grep -Fq 'domain=validation;family=context-template-transfer;start=4000;count=26' <<<"$context"
grep -Fq 'total=52;selected=52;correct=52;incorrect=0;abstained=0' <<<"$context"

grep -Fq 'tdi2.1-temporal-validation-v1;cases=33;correct=33' <<<"$temporal"
grep -Fq 'tdi2.1-analogy-validation-v1;cases=32;transfer_exact=32;no_transfer_exact=0' <<<"$analogy"
grep -Fq 'tdi2.1-deterministic-replay-v1;motif_equal=true;context_equal=true' <<<"$replay"

printf '%s\n' 'tdi2.1-cycle2-summary-v1;motif=34/34;context=52/52;temporal=33/33;analogy=32/32;replay=true;primary_holdout=closed'
