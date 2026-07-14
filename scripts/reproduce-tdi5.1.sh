#!/usr/bin/env bash

set -Eeuo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"
ROOT="$(cd -- "${SCRIPT_DIR}/.." && pwd -P)"

cd "$ROOT"

PREREG_HASH="docs/TDI-5.1-CONTINUOUS-DEFICIT-GEOMETRY-PREREGISTRATION.sha256"
EVALUATOR_HASH="docs/TDI-5.1-CONTINUOUS-DEFICIT-GEOMETRY-EVALUATOR.sha256"
SCIENTIFIC_HASH="docs/TDI-5.1-SCIENTIFIC-CODE.sha256"
FROZEN_TDI5_PREREG_HASH="docs/TDI-5-CONTINUOUS-DEFICIT-GEOMETRY-PREREGISTRATION.sha256"
FROZEN_TDI5_EVALUATOR_HASH="docs/TDI-5-CONTINUOUS-DEFICIT-GEOMETRY-EVALUATOR.sha256"
FROZEN_TDI5_SCIENTIFIC_HASH="docs/TDI-5-SCIENTIFIC-CODE.sha256"

RESULT_DIR="results/tdi5.1-continuous-deficit-geometry"
RESULT_FILE="${RESULT_DIR}/tdi-continuous-deficit-geometry-v51.log"
METADATA_FILE="${RESULT_DIR}/tdi-continuous-deficit-geometry-v51.metadata.txt"
RESULT_HASH_FILE="${RESULT_DIR}/tdi-continuous-deficit-geometry-v51.log.sha256"
COMPLETION_MARKER="${RESULT_DIR}/tdi-continuous-deficit-geometry-v51.complete"
LOCK_DIR="${RESULT_DIR}/.tdi5.1.lock"

BINARY_NAME="tdi-continuous-deficit-geometry-v51"
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
        log_error "repository must be clean before the full TDI-5.1 evaluation"
        git status --short
        exit 1
    fi
}

refuse_existing_output() {
    if [[ -e "$COMPLETION_MARKER" ]]; then
        log_error "a completed TDI-5.1 run already exists"
        ls -lh -- "$COMPLETION_MARKER" "$RESULT_FILE" "$RESULT_HASH_FILE" "$METADATA_FILE" >&2
        exit 1
    fi

    if [[ -e "$RESULT_FILE" || -e "$RESULT_HASH_FILE" || -e "$METADATA_FILE" ]]; then
        log_error "incomplete TDI-5.1 output exists; refusing to overwrite it"
        ls -lh -- "$RESULT_DIR" >&2
        exit 1
    fi
}

acquire_lock() {
    mkdir -p -- "$RESULT_DIR"

    if ! mkdir -- "$LOCK_DIR" 2>/dev/null; then
        log_error "another TDI-5.1 reproduction appears to be running: ${LOCK_DIR}"
        exit 1
    fi

    LOCK_HELD=true
}

write_initial_metadata() {
    local -r start_timestamp="$1"
    local -r command_line="$2"

    {
        printf 'experiment=TDI-5.1 continuous deficit geometry\n'
        printf 'start_utc=%s\n' "$start_timestamp"
        printf 'commit=%s\n' "$(git rev-parse HEAD)"
        printf 'repository=%s\n' "$ROOT"
        printf 'command_line=%s\n' "$command_line"
        printf 'rustc=%s\n' "$(rustc --version)"
        printf 'cargo=%s\n' "$(cargo --version)"
        printf 'uname=%s\n' "$(uname -a)"
        printf 'cargo_target_dir=%s\n' "$CARGO_TARGET_DIR"
        printf 'evaluator_sha256=%s\n' "$(sha256sum "tdi-bench/src/bin/${BINARY_NAME}.rs" | awk '{print $1}')"
        printf 'preregistration_sha256=%s\n' "$(sha256sum "docs/TDI-5.1-CONTINUOUS-DEFICIT-GEOMETRY-PREREGISTRATION.md" | awk '{print $1}')"
        printf 'scientific_manifest_sha256=%s\n' "$(sha256sum "$SCIENTIFIC_HASH" | awk '{print $1}')"
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

    if ! grep -q 'CRITERE TRAJECTOIRE TDI-5.1D\|CRITÈRE TRAJECTOIRE TDI-5.1D' "$RESULT_FILE"; then
        log_error "result file does not contain the final TDI-5.1D criterion"
        exit 1
    fi
}

mark_complete() {
    local -r end_timestamp="$1"
    local -r result_hash="$2"

    {
        printf 'experiment=TDI-5.1 continuous deficit geometry\n'
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

log_info "=== REPRODUCTION TDI-5.1 ==="
log_info "repository: ${ROOT}"

refuse_existing_output
require_clean_git
acquire_lock

log_info "verifying frozen TDI-5 hashes"
sha256sum -c "$FROZEN_TDI5_PREREG_HASH"
sha256sum -c "$FROZEN_TDI5_EVALUATOR_HASH"
sha256sum -c "$FROZEN_TDI5_SCIENTIFIC_HASH"

log_info "verifying TDI-5.1 hashes"
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
readonly COMMAND_LINE="$(write_command_line "$BINARY_PATH")"

write_initial_metadata "$START_TIMESTAMP" "$COMMAND_LINE"

log_info "starting preregistered full evaluation"
set +e
"$BINARY_PATH" | tee "$RESULT_FILE"
RUN_STATUS=${PIPESTATUS[0]}
set -e

if [[ "$RUN_STATUS" -ne 0 ]]; then
    log_error "TDI-5.1 evaluator failed with status ${RUN_STATUS}"
    exit "$RUN_STATUS"
fi

verify_complete_output

sha256sum "$RESULT_FILE" > "$RESULT_HASH_FILE"
RESULT_HASH="$(awk '{print $1}' "$RESULT_HASH_FILE")"
END_TIMESTAMP="$(date -u '+%Y-%m-%dT%H:%M:%SZ')"

append_final_metadata "$END_TIMESTAMP" "$RESULT_HASH"
mark_complete "$END_TIMESTAMP" "$RESULT_HASH"

log_info "TDI-5.1 reproduction completed"
log_info "result: ${RESULT_FILE}"
log_info "result sha256: ${RESULT_HASH}"
