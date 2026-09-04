#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

fail() {
    echo "ERROR: $*" >&2
    exit 1
}

manifest_hash_for() {
    local path="$1"
    awk -v wanted="$path" '$2 == wanted { print $1 }' docs/TDI-6.8-SCIENTIFIC-CODE.sha256
}

verify_tdi68_historical_integrity() {
    local manifest="docs/TDI-6.8-SCIENTIFIC-CODE.sha256"
    local expected path actual

    while read -r expected path; do
        case "$path" in
            Cargo.toml|Cargo.lock|.github/workflows/ci.yml)
                # These repository-level integration surfaces legitimately
                # evolve after TDI-6.8. The frozen manifest remains historical
                # evidence; current scientific source paths stay hash-checked.
                continue
                ;;
        esac
        [[ -f "$path" ]] || fail "historical TDI-6.8 path missing: $path"
        actual="$(sha256sum "$path" | awk '{print $1}')"
        [[ "$actual" == "$expected" ]] \
            || fail "historical TDI-6.8 path changed: $path"
    done <"$manifest"

    test -s .github/workflows/ci.yml \
        || fail "current Rust validation workflow is missing"

    local tmpdir
    tmpdir="$(mktemp -d)"
    trap 'rm -rf "$tmpdir"' RETURN

    python3 - "$tmpdir" <<'PY'
from pathlib import Path
import sys

out = Path(sys.argv[1])

cargo_toml = Path("Cargo.toml").read_text()
allowed_members = (
    '    "tdi-ai",\n',
    '    "tdi-operator",\n',
)
for member in allowed_members:
    if cargo_toml.count(member) != 1:
        raise SystemExit(
            f"Cargo.toml does not contain exactly one reviewed additive workspace member: {member.strip()}"
        )
    cargo_toml = cargo_toml.replace(member, "", 1)
(out / "Cargo.toml").write_text(cargo_toml)

cargo_lock = Path("Cargo.lock").read_text()
allowed_packages = (
    '''\n[[package]]\nname = "tdi-ai"\nversion = "0.1.0"\ndependencies = [\n "tdi-core",\n]\n''',
    '''\n[[package]]\nname = "tdi-operator"\nversion = "0.1.0"\n''',
)
for package in allowed_packages:
    if cargo_lock.count(package) != 1:
        package_name = next(
            line for line in package.splitlines() if line.startswith("name = ")
        )
        raise SystemExit(
            f"Cargo.lock does not contain exactly one reviewed additive package block: {package_name}"
        )
    cargo_lock = cargo_lock.replace(package, "", 1)
(out / "Cargo.lock").write_text(cargo_lock)
PY

    for path in Cargo.toml Cargo.lock; do
        expected="$(manifest_hash_for "$path")"
        [[ -n "$expected" ]] || fail "missing TDI-6.8 manifest entry for $path"
        actual="$(sha256sum "$tmpdir/$path" | awk '{print $1}')"
        [[ "$actual" == "$expected" ]] \
            || fail "workspace metadata drift exceeds the reviewed additive tdi-ai/tdi-operator changes: $path"
    done

    rm -rf "$tmpdir"
    trap - RETURN
}

printf '\n===== TDI-7.0 PREREGISTRATION INTEGRITY =====\n'
sha256sum -c docs/TDI-7.0-ATTENTION-RECOVERY-PREREGISTRATION.sha256

printf '\n===== HISTORICAL TDI-6.8 INTEGRITY =====\n'
verify_tdi68_historical_integrity
printf 'TDI-6.8 frozen scientific paths: OK\n'
printf 'TDI-6.8 workspace metadata projection: OK (reviewed additive tdi-ai + tdi-operator only)\n'
printf 'Current CI workflow: PRESENT (mutable repository infrastructure)\n'

printf '\n===== TDI-7.1 SPECIFICATION SURFACES =====\n'
test -f docs/TDI-7.1-EVALUATOR-SPEC.md || fail "missing TDI-7.1 evaluator specification"
test -f docs/TDI-7.1-COMPLETION-CHECKLIST.md || fail "missing TDI-7.1 completion checklist"
grep -Fq 'deterministic_local_row_stochastic_v1' docs/TDI-7.1-EVALUATOR-SPEC.md \
    || fail "semantic identifier missing from evaluator specification"
grep -Fq 'deficit = d / (1 + d)' docs/TDI-7.1-EVALUATOR-SPEC.md \
    || fail "frozen deficit formula missing from evaluator specification"
grep -Fq 'Frozen early observation depths for this evaluator: `1` and `2`.' docs/TDI-7.1-EVALUATOR-SPEC.md \
    || fail "frozen early depths missing from evaluator specification"
grep -Fq 'Target depth: `5`' docs/TDI-7.1-EVALUATOR-SPEC.md \
    || fail "target depth missing from evaluator specification"

printf '\n===== TDI-7.2 SURFACE EXCLUSION =====\n'
BOUNDED_FILES=(
    tdi-ai/examples/tdi7_features.rs
    tdi-ai/examples/tdi7_end_to_end.rs
    tdi-bench/src/bin/tdi-attention-v71-tasks.rs
    tdi-bench/src/bin/tdi-attention-v71-interventions.rs
    tdi-bench/src/bin/tdi-attention-v71-model.rs
    tdi-bench/src/bin/tdi-attention-v71-bootstrap.rs
)
for path in "${BOUNDED_FILES[@]}"; do
    test -f "$path" || fail "missing bounded evaluator file: $path"
    if grep -Fq 'I_ACCEPT_THE_TDI7_HOLDOUT_FREEZE' "$path"; then
        fail "final-holdout token leaked into bounded file: $path"
    fi
    if grep -Fq 'TDI7_CONFIRM_FINAL_HOLDOUT' "$path"; then
        fail "final-holdout authorization variable leaked into bounded file: $path"
    fi
    if grep -Fq '7_100_030_000' "$path"; then
        fail "final-holdout seed start leaked into bounded file: $path"
    fi
done

printf '\n===== TDI-7.1 PREFLIGHT HOLDOUT REFUSAL =====\n'
if TDI7_CONFIRM_FINAL_HOLDOUT=sentinel bash scripts/reproduce-tdi7.1-preflight.sh >/tmp/tdi71-refusal.out 2>&1; then
    cat /tmp/tdi71-refusal.out >&2
    fail "preflight accepted a final-holdout authorization environment"
fi
grep -Fq 'refuses any final-holdout authorization variable' /tmp/tdi71-refusal.out \
    || fail "preflight refusal reason was not explicit"
rm -f /tmp/tdi71-refusal.out

printf '\n===== TDI-7.1 COMPLETE BOUNDED PREFLIGHT =====\n'
bash scripts/reproduce-tdi7.1-preflight.sh

printf '\nTDI-7.1 readiness gate: PASS\n'
printf 'TDI-7.2 current state: HISTORICAL RESULT FROZEN / EXECUTABLE RERUN RETIRED\n'
