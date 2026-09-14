#!/usr/bin/env bash
# Non-final representation checks only; no trajectory or final runner.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"
bash scripts/check-tdi9-bootstrap.sh
bash scripts/check-tdi9.3-representation-calibration.sh
