#!/usr/bin/env bash

ROOT="$(
    cd "$(dirname "${BASH_SOURCE[0]}")/.." &&
    pwd
)"

cd "$ROOT" || exit 1

PREREG_HASH="docs/TDI-3-INTERWIDTH-PREREGISTRATION.sha256"
EVALUATOR_HASH="docs/TDI-3-INTERWIDTH-EVALUATOR.sha256"
SCIENTIFIC_CODE_HASH="docs/TDI-3-SCIENTIFIC-CODE.sha256"
RESULT_DIR="results"
RESULT_FILE="${RESULT_DIR}/tdi-interwidth-continuous.log"

export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-target}"

echo "=== REPRODUCTION TDI-3 ==="
echo "répertoire : $ROOT"
echo "rustc      : $(rustc --version)"
echo "cargo      : $(cargo --version)"
echo "commit     : $(git rev-parse HEAD)"

echo
echo "=== INTÉGRITÉ DU PRÉENREGISTREMENT ==="

sha256sum -c "$PREREG_HASH" || exit 1

echo
echo "=== INTÉGRITÉ DE L’ÉVALUATEUR GELÉ ==="

sha256sum -c "$EVALUATOR_HASH" || exit 1

echo
echo "=== INTÉGRITÉ DU CODE SCIENTIFIQUE ==="

sha256sum -c "$SCIENTIFIC_CODE_HASH" || exit 1

echo
echo "=== ÉTAT GIT ==="

if [ -n "$(git status --porcelain)" ]; then
    echo "ERREUR : le dépôt doit être propre avant l’évaluation"
    git status --short
    exit 1
fi

git status --short --branch

echo
echo "=== COMPILATION RELEASE ==="

cargo build \
    --release \
    --bin tdi-interwidth-continuous ||
    exit 1

mkdir -p "$RESULT_DIR"

echo
echo "=== ÉVALUATION PRÉENREGISTRÉE ==="

"$CARGO_TARGET_DIR/release/tdi-interwidth-continuous" |
    tee "$RESULT_FILE"

RUN_STATUS=${PIPESTATUS[0]}

if [ "$RUN_STATUS" -ne 0 ]; then
    echo "ERREUR : l’évaluation a échoué"
    exit "$RUN_STATUS"
fi

echo
echo "=== HASH DU RÉSULTAT ==="

sha256sum "$RESULT_FILE"

echo
echo "=== FIN ==="
echo "résultat : $RESULT_FILE"
