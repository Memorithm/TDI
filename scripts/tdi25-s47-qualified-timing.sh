#!/usr/bin/env bash
# TDI-25 slice 47: qualified timing run under the frozen T430 protocol,
# V3 amendment (docs/TDI-25-QUALIFIED-TIMING-ENVIRONMENT-V3.md,
#  docs/tdi25-qualified-timing-environment-v3.yaml; V1/V2 kept for the record).
# V3 frequency verdict: per-cell APERF/MPERF effective frequency must be
# within +-1% of the median effective across the 4 cells AND >= the
# 1785000 kHz anti-throttling floor (cross_cell_frequency_gate in Rust).
#
#   bash scripts/tdi25-s47-qualified-timing.sh build                 # normal user
#   sudo bash scripts/tdi25-s47-qualified-timing.sh run <attempt> [parent-dir]
#
# Attempts are numbered (attempt 1 was run under V1 and is recorded in
# results/tdi25_s47_timing/attempt-1); every attempt is recorded, whatever
# its outcome. The run id is tdi25-s47-timing-<commit12>-attempt-<N> and the
# output goes to <parent-dir>/<run id> (default: ../tdi25-s47-timing-runs).
# Preconditions refuse WITHOUT measuring (no attempt consumed): load1 > 2 at
# start (top CPU consumers are printed; pause them per the V2 checklist),
# /dev/cpu/28/msr unreadable (run `modprobe msr`). The script never stops
# services itself.
#
# The run mode refuses on any host other than the frozen Dell PowerEdge T430,
# pins to CPU 28 + NUMA node 0, sets governor=performance on CPUs 28/60 and
# intel_pstate/no_turbo=1 (restored on exit), checks that the SMT sibling
# CPU 60 is idle and the 1-min load average at the start and end of every
# cell, runs 5 warmup + 31 measured passes per cell and records median, min
# and IQR. A rejected cell is reported "non_qualifiee"; there is no rerun.
set -euo pipefail

repo="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo"
target="${CARGO_TARGET_DIR:-$repo/target}"
bin="$target/release/examples/tdi25_s47_qualified_timing"
stamp="$bin.build-stamp"

TOOLCHAIN="1.97.1"
CPU_MODEL="E5-2683 v4"
KERNEL="6.12.88+deb13-amd64"
BIOS="2.19.0"
HOSTNAME_EXPECTED="debian"
PINNED_CPU=28
SIBLING_CPU=60
NUMA_NODE=0
GOVERNOR="performance"
MAX_LOAD="2.0"
PROTOCOL="tdi25-qualified-timing-environment-v3"
FLOOR_KHZ="1785000"
MEDIAN_TOL="0.01"
SIBLING_MAX_BUSY="0.01"
CELLS=("development t6" "development c6" "validation t6" "validation c6")

die() { echo "tdi25-s47-qualified-timing: REFUSED: $*" >&2; exit 1; }
git_() { git -c safe.directory="$repo" "$@"; }
# Clean tree, ignoring untracked run directories left inside the checkout.
dirty() { git_ status --porcelain | grep -Ev '^\?\? tdi25-s47-timing-[^/]*/?$' || true; }

mode="${1:-}"
case "$mode" in
build)
  [[ $EUID -ne 0 ]] || die "build as the normal user, not root"
  [[ -z "$(dirty)" ]] || die "git tree is not clean"
  rustc_vv="$(rustc +"$TOOLCHAIN" -Vv)" || die "rustc $TOOLCHAIN missing (rustup toolchain install $TOOLCHAIN)"
  grep -q "^release: $TOOLCHAIN\$" <<<"$rustc_vv" || die "rustc is not $TOOLCHAIN"
  cargo +"$TOOLCHAIN" build --release --locked -p tdi-ai --features experimental \
    --example tdi25_s47_qualified_timing
  {
    echo "commit=$(git_ rev-parse HEAD)"
    echo "tree=$(git_ rev-parse 'HEAD^{tree}')"
    echo "sha256=$(sha256sum "$bin" | cut -d' ' -f1)"
    sed 's/^/rustc_vv: /' <<<"$rustc_vv"
  } >"$stamp"
  echo "built $bin"
  cat "$stamp"
  ;;
run)
  attempt="${2:-}"
  [[ "$attempt" =~ ^[0-9]+$ ]] && ((attempt >= 3)) \
    || die "usage: sudo bash $0 run <attempt number, >= 3 under V3> [parent-dir]"
  [[ $EUID -eq 0 ]] || die "run mode needs root (governor/turbo/MSR)"
  [[ ! -e "results/tdi25_s47_timing/attempt-$attempt" ]] || die "attempt $attempt is already recorded"
  [[ ! -e "docs/TDI-25-S47-TIMING-ATTEMPT-$attempt.md" ]] || die "attempt $attempt is already recorded (attempt record exists)"
  for tool in taskset numactl awk sha256sum ps; do
    command -v "$tool" >/dev/null || die "$tool missing"
  done
  [[ -z "$(dirty)" ]] || die "git tree is not clean"
  commit="$(git_ rev-parse HEAD)"
  run_id="tdi25-s47-timing-${commit:0:12}-attempt-$attempt"
  parent="${3:-$(dirname "$repo")/tdi25-s47-timing-runs}"
  out="$parent/$run_id"
  [[ ! -e "$out" ]] || die "$out exists: no rerun of attempt $attempt"
  tree="$(git_ rev-parse 'HEAD^{tree}')"
  [[ -x "$bin" && -f "$stamp" ]] || die "binary missing: run 'bash $0 build' first"
  grep -qx "commit=$commit" "$stamp" || die "binary was built from another commit"
  grep -qx "sha256=$(sha256sum "$bin" | cut -d' ' -f1)" "$stamp" || die "binary hash drift"
  grep -qx "rustc_vv: release: $TOOLCHAIN" "$stamp" || die "binary not built with rustc $TOOLCHAIN"

  # Preconditions (refuse without measuring).
  load_now="$(awk '{print $1}' /proc/loadavg)"
  if ! awk -v x="$load_now" -v y="$MAX_LOAD" 'BEGIN{exit !(x<=y)}'; then
    echo "load1=$load_now > $MAX_LOAD. Top CPU consumers:" >&2
    ps -eo pid,user,pcpu,etime,comm --sort=-pcpu | head -n 16 >&2
    die "load1 $load_now > $MAX_LOAD at start: pause the heavy services (see docs/TDI-25-QUALIFIED-TIMING-ENVIRONMENT-V3.md checklist), wait for load1 <= $MAX_LOAD, retry; no measurement was taken"
  fi
  msr="/dev/cpu/$PINNED_CPU/msr"
  [[ -r "$msr" ]] || die "$msr unreadable: run 'modprobe msr' as root, then retry; no measurement was taken"

  # Fingerprint.
  model="$(awk -F': ' '/^model name/{print $2; exit}' /proc/cpuinfo)"
  [[ "$model" == *"$CPU_MODEL"* ]] || die "CPU model '$model'"
  [[ "$(uname -r)" == "$KERNEL" ]] || die "kernel $(uname -r)"
  bios="$(cat /sys/class/dmi/id/bios_version 2>/dev/null || true)"
  [[ "$bios" == "$BIOS" ]] || die "BIOS '$bios'"
  [[ "$(hostname)" == "$HOSTNAME_EXPECTED" ]] || die "hostname $(hostname)"
  [[ "$(cat /sys/devices/system/cpu/cpu$SIBLING_CPU/topology/thread_siblings_list)" =~ (^|[,-])$PINNED_CPU([,-]|$) ]] \
    || die "CPU $SIBLING_CPU is not the SMT sibling of CPU $PINNED_CPU"
  [[ -d /sys/devices/system/node/node$NUMA_NODE/cpu$PINNED_CPU ]] || die "CPU $PINNED_CPU not on node $NUMA_NODE"
  gov_path() { echo "/sys/devices/system/cpu/cpu$1/cpufreq/scaling_governor"; }
  turbo=/sys/devices/system/cpu/intel_pstate/no_turbo
  [[ -w "$turbo" && -w "$(gov_path $PINNED_CPU)" && -w "$(gov_path $SIBLING_CPU)" ]] \
    || die "cannot set governor/turbo"

  mkdir -p "$out"
  echo "run_id=$run_id attempt=$attempt protocol=$PROTOCOL" | tee "$out/run-id.txt"
  orig_gov_p="$(cat "$(gov_path $PINNED_CPU)")"
  orig_gov_s="$(cat "$(gov_path $SIBLING_CPU)")"
  orig_turbo="$(cat "$turbo")"
  restore() {
    echo "$orig_gov_p" >"$(gov_path $PINNED_CPU)" || true
    echo "$orig_gov_s" >"$(gov_path $SIBLING_CPU)" || true
    echo "$orig_turbo" >"$turbo" || true
    if [[ "$(cat "$(gov_path $PINNED_CPU)")" == "$orig_gov_p" && \
          "$(cat "$(gov_path $SIBLING_CPU)")" == "$orig_gov_s" && \
          "$(cat "$turbo")" == "$orig_turbo" ]]; then
      echo "restored governor=$orig_gov_p/$orig_gov_s no_turbo=$orig_turbo" | tee -a "$out/restore.log"
    else
      echo "WARNING: governor/turbo restore could not be verified" | tee -a "$out/restore.log" >&2
    fi
  }
  trap restore EXIT
  echo "$GOVERNOR" >"$(gov_path $PINNED_CPU)" || die "cannot set governor on CPU $PINNED_CPU"
  echo "$GOVERNOR" >"$(gov_path $SIBLING_CPU)" || die "cannot set governor on CPU $SIBLING_CPU"
  echo 1 >"$turbo" || die "cannot disable turbo"
  [[ "$(cat "$(gov_path $PINNED_CPU)")" == "$GOVERNOR" && "$(cat "$(gov_path $SIBLING_CPU)")" == "$GOVERNOR" && "$(cat "$turbo")" == 1 ]] \
    || die "governor/turbo values did not stick"

  load1() { awk '{print $1}' /proc/loadavg; }
  vm() { awk -v k="$1" '$1==k{print $2}' /proc/vmstat; }
  sibling_busy() {
    local a b
    a=($(awk -v c="cpu$SIBLING_CPU" '$1==c' /proc/stat)); sleep 1
    b=($(awk -v c="cpu$SIBLING_CPU" '$1==c' /proc/stat))
    awk -v u="$(( (b[1]+b[2]+b[3]+b[6]+b[7]+b[8]) - (a[1]+a[2]+a[3]+a[6]+a[7]+a[8]) ))" \
        -v i="$(( (b[4]+b[5]) - (a[4]+a[5]) ))" 'BEGIN{t=u+i; printf "%.4f", (t>0?u/t:1)}'
  }
  le() { awk -v x="$1" -v y="$2" 'BEGIN{exit !(x<=y)}'; }
  {
    echo "{\"environment_contract\":\"$PROTOCOL\",\"run_id\":\"$run_id\",\"attempt\":$attempt,\"date_utc\":\"$(date -u +%FT%TZ)\","
    echo "\"commit\":\"$commit\",\"tree\":\"$tree\",\"hostname\":\"$(hostname)\",\"cpu_model\":\"$model\","
    echo "\"kernel\":\"$(uname -r)\",\"bios\":\"$bios\",\"binary_sha256\":\"$(sha256sum "$bin" | cut -d' ' -f1)\","
    echo "\"rustc_vv\":\"$(grep '^rustc_vv: ' "$stamp" | sed 's/^rustc_vv: //' | paste -sd'|')\","
    echo "\"governor_original\":[\"$orig_gov_p\",\"$orig_gov_s\"],\"no_turbo_original\":\"$orig_turbo\","
    echo "\"governor_set\":\"$GOVERNOR\",\"no_turbo_set\":\"1\",\"loadavg\":\"$(cat /proc/loadavg)\","
    echo "\"pswpin\":$(vm pswpin),\"pswpout\":$(vm pswpout),\"mitigations\":\"$(cat /proc/cmdline | tr '"' "'")\"}"
  } | tr -d '\n' >"$out/fingerprint.json"
  echo >>"$out/fingerprint.json"

  # Collect all 4 cells first: the V3 frequency verdict needs the median
  # effective frequency across the cells of the run.
  cell_json=(); cell_busy=(); cell_l0=(); cell_l1=(); cell_eff=()
  for cell in "${CELLS[@]}"; do
    read -r split arm <<<"$cell"
    busy="$(sibling_busy)"
    l0="$(load1)"
    json="$(numactl --membind=$NUMA_NODE taskset -c $PINNED_CPU "$bin" "$split" "$arm")" \
      || die "harness refused cell $split/$arm (see stderr)"
    l1="$(load1)"
    cell_json+=("$json"); cell_busy+=("$busy"); cell_l0+=("$l0"); cell_l1+=("$l1")
    eff="$(sed -n 's/.*"effective_frequency_khz":\([0-9]*\|null\).*/\1/p' <<<"$json")"
    cell_eff+=("${eff:-null}")
  done
  # Median of the present effective frequencies (mean of the two middle
  # values for 4 cells), mirroring cross_cell_frequency_gate in Rust.
  median_eff="$(printf '%s\n' "${cell_eff[@]}" | awk '$1!="null"{print $1}' | sort -n | awk '{v[NR]=$1} END{if(NR==0)print "null"; else if(NR%2==1)print v[(NR+1)/2]; else print (v[NR/2]+v[NR/2+1])/2}')"
  for idx in "${!CELLS[@]}"; do
    read -r split arm <<<"${CELLS[$idx]}"
    json="${cell_json[$idx]}"; busy="${cell_busy[$idx]}"; l0="${cell_l0[$idx]}"; l1="${cell_l1[$idx]}"; eff="${cell_eff[$idx]}"
    status=qualifiee
    reasons=()
    le "$busy" "$SIBLING_MAX_BUSY" || reasons+=("smt_sibling_busy")
    le "$l0" "$MAX_LOAD" || reasons+=("load_start")
    le "$l1" "$MAX_LOAD" || reasons+=("load_end")
    grep -q '"iqr_ok":true' <<<"$json" || reasons+=("iqr")
    grep -q '"frequency_ok":true' <<<"$json" || reasons+=("frequency_floor")
    grep -q '"swap_ok":true' <<<"$json" || reasons+=("swap")
    # V3 cross-cell consistency: |eff - median| <= 1% of median and eff >= floor.
    ratio="null"
    if [[ "$eff" == "null" || "$median_eff" == "null" ]]; then
      reasons+=("frequency")
    else
      ratio="$(awk -v e="$eff" -v m="$median_eff" 'BEGIN{printf "%.6f", (m>0? e/m : 0)}')"
      awk -v e="$eff" -v m="$median_eff" -v tol="$MEDIAN_TOL" -v floor="$FLOOR_KHZ" 'BEGIN{d=e-m; if(d<0)d=-d; exit !(e>=floor && d<=tol*m)}' \
        || reasons+=("frequency")
    fi
    joined=""
    if ((${#reasons[@]} > 0)); then
      status=non_qualifiee
      joined="$(printf '"%s",' "${reasons[@]}")"
      joined="${joined%,}"
    fi
    printf '{"run_id":"%s","attempt":%s,"split":"%s","arm":"%s","cell_status":"%s","rejection_reasons":[%s],"smt_sibling_busy_fraction":%s,"load1_start":%s,"load1_end":%s,"frequency_median_effective_khz":%s,"frequency_ratio_to_median":%s,"harness":%s}\n' \
      "$run_id" "$attempt" "$split" "$arm" "$status" "$joined" "$busy" "$l0" "$l1" "$median_eff" "$ratio" "$json" \
      | tee -a "$out/results.jsonl"
  done
  echo "done: $out/fingerprint.json $out/results.jsonl (timing results valid for this machine only)"
  ;;
*)
  die "usage: bash $0 build | sudo bash $0 run <attempt> [parent-dir]"
  ;;
esac
