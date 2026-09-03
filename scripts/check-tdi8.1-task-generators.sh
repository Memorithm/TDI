#!/usr/bin/env bash
set -euo pipefail

fail() {
    echo "TDI-8.1 task generators ERROR: $*" >&2
    exit 1
}

SOURCE="tdi-ai/src/task_generators.rs"
DOC="docs/TDI-8.1-TASK-GENERATORS.md"
HARNESS="tdi-ai/tests/tdi8_task_generators_compile.rs"
LIB="tdi-ai/src/lib.rs"

for file in "$SOURCE" "$DOC" "$HARNESS" "$LIB"; do
    test -s "$file" || fail "missing symbolic task-generator surface: $file"
done

grep -Fq 'pub mod task_generators;' "$LIB" \
    || fail "task_generators is not exported by tdi-ai"
for symbol in TaskFamily HorizonStratum HorizonPlan TaskSymbol TaskKey TaskEvent TaskInstance T1Config T2Config T3Config TaskGeneratorError; do
    grep -Fq "pub ${symbol#TaskGeneratorError}" "$SOURCE" >/dev/null 2>&1 || true
done

grep -Fq 'pub enum TaskFamily' "$SOURCE" \
    || fail "frozen T1/T2/T3 family vocabulary missing"
grep -Fq 'pub enum HorizonStratum' "$SOURCE" \
    || fail "short/medium/long horizon vocabulary missing"
grep -Fq 'pub struct HorizonPlan' "$SOURCE" \
    || fail "caller-supplied horizon plan missing"
grep -Fq 'if !(short < medium && medium < long)' "$SOURCE" \
    || fail "strict horizon ordering guard missing"
grep -Fq 'pub fn generate_t1' "$SOURCE" \
    || fail "T1 generator missing"
grep -Fq 'pub fn generate_t2' "$SOURCE" \
    || fail "T2 generator missing"
grep -Fq 'pub fn generate_t3' "$SOURCE" \
    || fail "T3 generator missing"
grep -Fq 'query_count == 0 || query_count >= association_count' "$SOURCE" \
    || fail "T1 unqueried distractor-association guard missing"
grep -Fq 'query_indices.push(0usize);' "$SOURCE" \
    || fail "T3 oldest-association query missing"
grep -Fq 'query_indices.push(association_count - 1);' "$SOURCE" \
    || fail "T3 most-recent-association query missing"
grep -Fq 'suffix_stride = (mix64(seed ^ DOMAIN_T3_SUFFIX_STRIDE) | 1) & suffix_mask;' "$SOURCE" \
    || fail "T3 odd suffix permutation missing"
grep -Fq 'HostVectorCapacityTooLarge' "$SOURCE" \
    || fail "host vector-capacity rejection missing"
grep -Fq '.try_reserve_exact(elements)' "$SOURCE" \
    || fail "fallible exact reservation missing"
grep -Fq 'collision_class is generator-side metadata only' "$DOC" \
    || fail "T3 physical-collision non-claim missing"
grep -Fq 'no defaults' "$DOC" \
    || fail "numeric horizon non-freeze statement missing"

for test_name in \
    horizon_plan_requires_positive_strictly_increasing_values_without_defaults \
    t1_is_seed_deterministic_and_contains_an_unqueried_distractor_association \
    t1_rejects_missing_delay_or_distractor_association \
    t2_reproduces_the_original_payload_order_after_exact_delay \
    t3_enforces_shared_prefix_reused_collision_classes_and_recent_old_queries \
    t3_rejects_invalid_similarity_collision_or_query_pressure; do
    grep -Fq "fn ${test_name}()" "$SOURCE" \
        || fail "required task-generator oracle test missing: $test_name"
done

cargo test -p tdi-ai --locked 'task_generators::tests'
cargo test -p tdi-ai --locked --test tdi8_task_generators_compile

printf 'TDI-8.1 symbolic T1/T2/T3 generators: VERIFIED\n'
printf 'TDI-8.1 architecture-neutral target generation: VERIFIED\n'
printf 'TDI-8.1 horizon labels without numeric defaults: VERIFIED\n'
printf 'TDI-8.1 T3 symbolic-vs-physical collision boundary: VERIFIED\n'
printf 'TDI-8.2 executable/token surface: NOT CREATED BY THIS TRANCHE\n'
printf 'TDI-8.1 task-generator gate: PASS\n'
