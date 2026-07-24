#!/usr/bin/env bash
#
# TDI-6.4 reproduction script — causal probe (does recovery depend on WHICH
# node is perturbed, for a fixed system?). One factor changed vs TDI-5.6: the
# perturbation protocol itself — every node 0..width is analyzed via
# analyze_branching_recovery, exhaustively, instead of the single historical
# node width-1 every prior experiment used. This stays entirely on the
# bit-exact rational track (preregistration Section 1.2): analyze_branching_
# recovery, uniform_branching_state_distribution and distribution_overlap are
# called more times, not replaced by a non-exact computation. Reproduction is
# therefore BYTE-EXACT (Section 1.2, 20), like TDI-5.2...5.8, unlike the
# tolerance-based TDI-6.1/6.2/6.3/6.5 track. This script performs the real,
# preregistered 120,000-record TDI-6.4 run (3 blocks x 40,000 accepted
# records) exactly once.
#
# The real command, reserved for a deliberate human action, is:
#
#   TDI64_CONFIRM_FULL_RUN=I_ACCEPT_THE_TDI64_FREEZE_RULE \
#     bash scripts/reproduce-tdi6.4.sh
#
# Running this script WITHOUT that exact environment variable refuses
# before any generation, per-node analysis or bootstrap (see
# `require_full_run_confirmation` below). Nothing in this repository's
# CI workflows ever sets that variable.

set -Eeuo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"
ROOT="$(cd -- "${SCRIPT_DIR}/.." && pwd -P)"

cd "$ROOT"

TDI64_CONFIRM_VAR="TDI64_CONFIRM_FULL_RUN"
TDI64_CONFIRM_VALUE="I_ACCEPT_THE_TDI64_FREEZE_RULE"

PREREG_HASH="docs/TDI-6.4-CAUSAL-PROBE-PREREGISTRATION.sha256"
EVALUATOR_HASH="docs/TDI-6.4-CAUSAL-PROBE-EVALUATOR.sha256"
SCIENTIFIC_HASH="docs/TDI-6.4-SCIENTIFIC-CODE.sha256"

FROZEN_TDI51_PREREG_HASH="docs/TDI-5.1-CONTINUOUS-DEFICIT-GEOMETRY-PREREGISTRATION.sha256"
FROZEN_TDI51_EVALUATOR_HASH="docs/TDI-5.1-CONTINUOUS-DEFICIT-GEOMETRY-EVALUATOR.sha256"
FROZEN_TDI51_SCIENTIFIC_HASH="docs/TDI-5.1-SCIENTIFIC-CODE.sha256"

FROZEN_TDI52_PREREG_HASH="docs/TDI-5.2-INDEPENDENT-OVERLAP-ABLATION-PREREGISTRATION.sha256"
FROZEN_TDI52_EVALUATOR_HASH="docs/TDI-5.2-INDEPENDENT-OVERLAP-ABLATION-EVALUATOR.sha256"
FROZEN_TDI52_SCIENTIFIC_HASH="docs/TDI-5.2-SCIENTIFIC-CODE.sha256"

FROZEN_TDI53_PREREG_HASH="docs/TDI-5.3-INDEPENDENT-OVERLAP-ACTIVATION-PREREGISTRATION.sha256"
FROZEN_TDI53_EVALUATOR_HASH="docs/TDI-5.3-INDEPENDENT-OVERLAP-ACTIVATION-EVALUATOR.sha256"
FROZEN_TDI53_SCIENTIFIC_HASH="docs/TDI-5.3-SCIENTIFIC-CODE.sha256"

FROZEN_TDI54_PREREG_HASH="docs/TDI-5.4-NONLINEAR-OVERLAP-SUFFICIENCY-PREREGISTRATION.sha256"
FROZEN_TDI54_EVALUATOR_HASH="docs/TDI-5.4-NONLINEAR-OVERLAP-SUFFICIENCY-EVALUATOR.sha256"
FROZEN_TDI54_SCIENTIFIC_HASH="docs/TDI-5.4-SCIENTIFIC-CODE.sha256"

FROZEN_TDI55_PREREG_HASH="docs/TDI-5.5-OVERLAP-BASELINE-CHALLENGE-PREREGISTRATION.sha256"
FROZEN_TDI55_EVALUATOR_HASH="docs/TDI-5.5-OVERLAP-BASELINE-CHALLENGE-EVALUATOR.sha256"
FROZEN_TDI55_SCIENTIFIC_HASH="docs/TDI-5.5-SCIENTIFIC-CODE.sha256"

FROZEN_TDI56_PREREG_HASH="docs/TDI-5.6-EXACT-SPECTRAL-CHALLENGE-PREREGISTRATION.sha256"
FROZEN_TDI56_EVALUATOR_HASH="docs/TDI-5.6-EXACT-SPECTRAL-CHALLENGE-EVALUATOR.sha256"
FROZEN_TDI56_SCIENTIFIC_HASH="docs/TDI-5.6-SCIENTIFIC-CODE.sha256"

FROZEN_TDI57_PREREG_HASH="docs/TDI-5.7-GENERATOR-ROBUSTNESS-PREREGISTRATION.sha256"
FROZEN_TDI57_EVALUATOR_HASH="docs/TDI-5.7-GENERATOR-ROBUSTNESS-EVALUATOR.sha256"
FROZEN_TDI57_SCIENTIFIC_HASH="docs/TDI-5.7-SCIENTIFIC-CODE.sha256"

FROZEN_TDI58_PREREG_HASH="docs/TDI-5.8-CROSS-WIDTH-INVARIANCE-PREREGISTRATION.sha256"
FROZEN_TDI58_EVALUATOR_HASH="docs/TDI-5.8-CROSS-WIDTH-INVARIANCE-EVALUATOR.sha256"
FROZEN_TDI58_SCIENTIFIC_HASH="docs/TDI-5.8-SCIENTIFIC-CODE.sha256"

FROZEN_TDI61_PREREG_HASH="docs/TDI-6.1-SPECTRAL-GAP-MIXING-TIME-PREREGISTRATION.sha256"
FROZEN_TDI61_EVALUATOR_HASH="docs/TDI-6.1-SPECTRAL-GAP-MIXING-TIME-EVALUATOR.sha256"
FROZEN_TDI61_SCIENTIFIC_HASH="docs/TDI-6.1-SCIENTIFIC-CODE.sha256"

FROZEN_TDI62_PREREG_HASH="docs/TDI-6.2-NONLINEAR-SUFFICIENCY-PREREGISTRATION.sha256"
FROZEN_TDI62_EVALUATOR_HASH="docs/TDI-6.2-NONLINEAR-SUFFICIENCY-EVALUATOR.sha256"
FROZEN_TDI62_SCIENTIFIC_HASH="docs/TDI-6.2-SCIENTIFIC-CODE.sha256"

FROZEN_TDI63_PREREG_HASH="docs/TDI-6.3-INFORMATION-DECOMPOSITION-PREREGISTRATION.sha256"
FROZEN_TDI63_EVALUATOR_HASH="docs/TDI-6.3-INFORMATION-DECOMPOSITION-EVALUATOR.sha256"
FROZEN_TDI63_SCIENTIFIC_HASH="docs/TDI-6.3-SCIENTIFIC-CODE.sha256"

FROZEN_TDI65_PREREG_HASH="docs/TDI-6.5-GENERATOR-FAMILY-SPECTRAL-ROBUSTNESS-PREREGISTRATION.sha256"
FROZEN_TDI65_EVALUATOR_HASH="docs/TDI-6.5-GENERATOR-FAMILY-SPECTRAL-ROBUSTNESS-EVALUATOR.sha256"
FROZEN_TDI65_SCIENTIFIC_HASH="docs/TDI-6.5-SCIENTIFIC-CODE.sha256"

RESULT_DIR="results/tdi6.4-causal-probe"
RESULT_FILE="${RESULT_DIR}/tdi-independent-overlap-ablation-v64.log"
METADATA_FILE="${RESULT_DIR}/tdi-independent-overlap-ablation-v64.metadata.txt"
RESULT_HASH_FILE="${RESULT_DIR}/tdi-independent-overlap-ablation-v64.log.sha256"
COMPLETION_MARKER="${RESULT_DIR}/tdi-independent-overlap-ablation-v64.complete"
LOCK_DIR="${RESULT_DIR}/.tdi6.4.lock"

BINARY_NAME="tdi-independent-overlap-ablation-v64"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-target}"
BINARY_PATH="${CARGO_TARGET_DIR}/release/${BINARY_NAME}"

LOCK_HELD=false

log_info() {
    printf '[%s] INFO: %s\n' "$(date -u '+%Y-%m-%dT%H:%M:%SZ')" "$*" >&2
}

log_error() {
    printf '[%s] ERROR: %s\n' "$(date -u '+%Y-%m-%dT%H:%M:%SZ')" "$*" >&2
}

cleanup() {
    if [[ "$LOCK_HELD" == true && -d "$LOCK_DIR" ]]; then
        rmdir -- "$LOCK_DIR" 2>/dev/null || true
    fi
}

on_error() {
    local -r line="$1"
    log_error "reproduction failed at line ${line}"
}

trap 'on_error "$LINENO"' ERR
trap cleanup EXIT

require_command() {
    local -r command_name="$1"

    if ! command -v "$command_name" >/dev/null 2>&1; then
        log_error "required command not found: ${command_name}"
        exit 1
    fi
}

# The human confirmation token: without the exact value, this script must
# refuse before verifying hashes, before building, and before any
# generation, per-node analysis or bootstrap. This is checked in the script
# itself (not just inside the evaluator) so a human who forgot to set it is
# told immediately, rather than after paying for hash verification and a
# release build.
require_full_run_confirmation() {
    local -r actual="${!TDI64_CONFIRM_VAR:-}"

    if [[ "$actual" != "$TDI64_CONFIRM_VALUE" ]]; then
        log_error "refusing: ${TDI64_CONFIRM_VAR} must be set to the exact value ${TDI64_CONFIRM_VALUE}"
        log_error "the real command is:"
        log_error "  ${TDI64_CONFIRM_VAR}=${TDI64_CONFIRM_VALUE} bash ${BASH_SOURCE[0]}"
        exit 1
    fi
}

write_command_line() {
    local -a command=("$@")
    local rendered=""

    for argument in "${command[@]}"; do
        rendered+="$(printf '%q' "$argument") "
    done

    printf '%s\n' "${rendered% }"
}

require_clean_git() {
    if [[ -n "$(git status --porcelain --untracked-files=all)" ]]; then
        log_error "repository must be clean before the full TDI-6.4 evaluation"
        git status --short
        exit 1
    fi
}

refuse_existing_output() {
    if [[ -e "$COMPLETION_MARKER" ]]; then
        log_error "a completed TDI-6.4 run already exists"
        ls -lh -- "$COMPLETION_MARKER" "$RESULT_FILE" "$RESULT_HASH_FILE" "$METADATA_FILE" >&2
        exit 1
    fi

    if [[ -e "$RESULT_FILE" || -e "$RESULT_HASH_FILE" || -e "$METADATA_FILE" ]]; then
        log_error "incomplete TDI-6.4 output exists; refusing to overwrite it"
        ls -lh -- "$RESULT_DIR" >&2
        exit 1
    fi
}

acquire_lock() {
    mkdir -p -- "$RESULT_DIR"

    if ! mkdir -- "$LOCK_DIR" 2>/dev/null; then
        log_error "another TDI-6.4 reproduction appears to be running: ${LOCK_DIR}"
        exit 1
    fi

    LOCK_HELD=true
}

write_initial_metadata() {
    local -r start_timestamp="$1"
    local -r command_line="$2"

    {
        printf 'experiment=TDI-6.4 causal probe (per-node perturbation heterogeneity)\n'
        printf 'start_utc=%s\n' "$start_timestamp"
        printf 'commit=%s\n' "$(git rev-parse HEAD)"
        printf 'repository=%s\n' "$ROOT"
        printf 'command_line=%s\n' "$command_line"
        printf 'rustc=%s\n' "$(rustc --version)"
        printf 'cargo=%s\n' "$(cargo --version)"
        printf 'uname=%s\n' "$(uname -a)"
        printf 'cargo_target_dir=%s\n' "$CARGO_TARGET_DIR"
        printf 'exactness_regime=bit-exact (ExactRatio arithmetic); byte-exact reproduction\n'
        printf 'evaluator_sha256=%s\n' "$(sha256sum "tdi-bench/src/bin/${BINARY_NAME}.rs" | awk '{print $1}')"
        printf 'preregistration_sha256=%s\n' "$(sha256sum "docs/TDI-6.4-CAUSAL-PROBE-PREREGISTRATION.md" | awk '{print $1}')"
        printf 'scientific_manifest_sha256=%s\n' "$(sha256sum "$SCIENTIFIC_HASH" | awk '{print $1}')"
        printf 'frozen_tdi56_evaluator_sha256=%s\n' "$(sha256sum "tdi-bench/src/bin/tdi-independent-overlap-ablation-v56.rs" | awk '{print $1}')"
        printf 'frozen_tdi56_preregistration_sha256=%s\n' "$(sha256sum "docs/TDI-5.6-EXACT-SPECTRAL-CHALLENGE-PREREGISTRATION.md" | awk '{print $1}')"
    } > "$METADATA_FILE"
}

append_final_metadata() {
    local -r end_timestamp="$1"
    local -r result_hash="$2"

    {
        printf 'end_utc=%s\n' "$end_timestamp"
        printf 'result_sha256=%s\n' "$result_hash"
        printf 'result_file=%s\n' "$RESULT_FILE"
        printf 'result_hash_file=%s\n' "$RESULT_HASH_FILE"
        printf 'completion_marker=%s\n' "$COMPLETION_MARKER"
    } >> "$METADATA_FILE"
}

verify_complete_output() {
    if [[ ! -s "$RESULT_FILE" ]]; then
        log_error "result file is missing or empty: ${RESULT_FILE}"
        exit 1
    fi

    # TDI-6.4 preregistration Sections 13-17 require the TDI-6.4A/B/C
    # descriptive summary lines; reproduction is byte-exact (Section 1.2/20),
    # so the result log itself is verified byte-for-byte via its SHA-256
    # below. None of TDI-6.4A/B/C is a pass/fail classification, so unlike
    # the exact TDI-5.x scripts there is no lettered verdict beyond C.
    local -a required_phrases=(
        "VERDICTS FINAUX"
        "TDI-6.4A"
        "TDI-6.4B"
        "TDI-6.4C"
    )

    for phrase in "${required_phrases[@]}"; do
        if ! grep -q -- "$phrase" "$RESULT_FILE"; then
            log_error "result file is missing the required phrase: ${phrase}"
            exit 1
        fi
    done
}

mark_complete() {
    local -r end_timestamp="$1"
    local -r result_hash="$2"

    {
        printf 'experiment=TDI-6.4 causal probe (per-node perturbation heterogeneity)\n'
        printf 'completed_utc=%s\n' "$end_timestamp"
        printf 'commit=%s\n' "$(git rev-parse HEAD)"
        printf 'result_sha256=%s\n' "$result_hash"
    } > "$COMPLETION_MARKER"

    chmod 0444 "$RESULT_FILE" "$RESULT_HASH_FILE" "$METADATA_FILE" "$COMPLETION_MARKER"
}

require_command git
require_command cargo
require_command rustc
require_command sha256sum
require_command awk
require_command grep
require_command tee
require_command uname

log_info "=== REPRODUCTION TDI-6.4 ==="
log_info "repository: ${ROOT}"

require_full_run_confirmation
refuse_existing_output
require_clean_git
acquire_lock

log_info "verifying frozen TDI-5.1 hashes"
sha256sum -c "$FROZEN_TDI51_PREREG_HASH"
sha256sum -c "$FROZEN_TDI51_EVALUATOR_HASH"
sha256sum -c "$FROZEN_TDI51_SCIENTIFIC_HASH"

log_info "verifying frozen TDI-5.2 hashes"
sha256sum -c "$FROZEN_TDI52_PREREG_HASH"
sha256sum -c "$FROZEN_TDI52_EVALUATOR_HASH"
sha256sum -c "$FROZEN_TDI52_SCIENTIFIC_HASH"

log_info "verifying frozen TDI-5.3 hashes"
sha256sum -c "$FROZEN_TDI53_PREREG_HASH"
sha256sum -c "$FROZEN_TDI53_EVALUATOR_HASH"
sha256sum -c "$FROZEN_TDI53_SCIENTIFIC_HASH"

log_info "verifying frozen TDI-5.4 hashes"
sha256sum -c "$FROZEN_TDI54_PREREG_HASH"
sha256sum -c "$FROZEN_TDI54_EVALUATOR_HASH"
sha256sum -c "$FROZEN_TDI54_SCIENTIFIC_HASH"

log_info "verifying frozen TDI-5.5 hashes"
sha256sum -c "$FROZEN_TDI55_PREREG_HASH"
sha256sum -c "$FROZEN_TDI55_EVALUATOR_HASH"
sha256sum -c "$FROZEN_TDI55_SCIENTIFIC_HASH"

log_info "verifying frozen TDI-5.6 hashes"
sha256sum -c "$FROZEN_TDI56_PREREG_HASH"
sha256sum -c "$FROZEN_TDI56_EVALUATOR_HASH"
sha256sum -c "$FROZEN_TDI56_SCIENTIFIC_HASH"

log_info "verifying frozen TDI-5.7 hashes"
sha256sum -c "$FROZEN_TDI57_PREREG_HASH"
sha256sum -c "$FROZEN_TDI57_EVALUATOR_HASH"
sha256sum -c "$FROZEN_TDI57_SCIENTIFIC_HASH"

log_info "verifying frozen TDI-5.8 hashes"
sha256sum -c "$FROZEN_TDI58_PREREG_HASH"
sha256sum -c "$FROZEN_TDI58_EVALUATOR_HASH"
sha256sum -c "$FROZEN_TDI58_SCIENTIFIC_HASH"

log_info "verifying frozen TDI-6.1 hashes"
sha256sum -c "$FROZEN_TDI61_PREREG_HASH"
sha256sum -c "$FROZEN_TDI61_EVALUATOR_HASH"
sha256sum -c "$FROZEN_TDI61_SCIENTIFIC_HASH"

log_info "verifying frozen TDI-6.2 hashes"
sha256sum -c "$FROZEN_TDI62_PREREG_HASH"
sha256sum -c "$FROZEN_TDI62_EVALUATOR_HASH"
sha256sum -c "$FROZEN_TDI62_SCIENTIFIC_HASH"

log_info "verifying frozen TDI-6.3 hashes"
sha256sum -c "$FROZEN_TDI63_PREREG_HASH"
sha256sum -c "$FROZEN_TDI63_EVALUATOR_HASH"
sha256sum -c "$FROZEN_TDI63_SCIENTIFIC_HASH"

log_info "verifying frozen TDI-6.5 hashes"
sha256sum -c "$FROZEN_TDI65_PREREG_HASH"
sha256sum -c "$FROZEN_TDI65_EVALUATOR_HASH"
sha256sum -c "$FROZEN_TDI65_SCIENTIFIC_HASH"

log_info "verifying TDI-6.4 hashes"
sha256sum -c "$PREREG_HASH"
sha256sum -c "$EVALUATOR_HASH"
sha256sum -c "$SCIENTIFIC_HASH"

log_info "building release evaluator offline"
cargo build --release --offline --bin "$BINARY_NAME"

if [[ ! -x "$BINARY_PATH" ]]; then
    log_error "release evaluator is missing or not executable: ${BINARY_PATH}"
    exit 1
fi

readonly START_TIMESTAMP="$(date -u '+%Y-%m-%dT%H:%M:%SZ')"
readonly COMMAND_LINE="$(write_command_line "$BINARY_PATH" "--full")"

write_initial_metadata "$START_TIMESTAMP" "$COMMAND_LINE"

log_info "starting preregistered full evaluation (3 blocks V/W/X, 40,000 accepted records each)"
set +e
"$BINARY_PATH" --full 2>&1 | tee "$RESULT_FILE"
RUN_STATUS=${PIPESTATUS[0]}
set -e

if [[ "$RUN_STATUS" -ne 0 ]]; then
    log_error "TDI-6.4 evaluator failed with status ${RUN_STATUS}"
    exit "$RUN_STATUS"
fi

verify_complete_output

sha256sum "$RESULT_FILE" > "$RESULT_HASH_FILE"
RESULT_HASH="$(awk '{print $1}' "$RESULT_HASH_FILE")"
END_TIMESTAMP="$(date -u '+%Y-%m-%dT%H:%M:%SZ')"

append_final_metadata "$END_TIMESTAMP" "$RESULT_HASH"
mark_complete "$END_TIMESTAMP" "$RESULT_HASH"

log_info "TDI-6.4 reproduction completed"
log_info "result: ${RESULT_FILE}"
log_info "result sha256: ${RESULT_HASH}"
