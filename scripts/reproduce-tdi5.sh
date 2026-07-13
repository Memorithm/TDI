#!/usr/bin/env bash

set -o pipefail

ROOT="$(
    cd "$(dirname "${BASH_SOURCE[0]}")/.." &&
    pwd
)"

cd "$ROOT" || exit 1

PREREG_HASH="docs/TDI-5-CONTINUOUS-DEFICIT-GEOMETRY-PREREGISTRATION.sha256"
EVALUATOR_HASH="docs/TDI-5-CONTINUOUS-DEFICIT-GEOMETRY-EVALUATOR.sha256"
SCIENTIFIC_HASH="docs/TDI-5-SCIENTIFIC-CODE.sha256"

RESULT_DIR="results"
RESULT_FILE="${RESULT_DIR}/tdi-continuous-deficit-geometry.log"

BINARY_NAME="tdi-continuous-deficit-geometry"

export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-target}"

echo "=== REPRODUCTION TDI-5 ==="
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

sha256sum -c "$SCIENTIFIC_HASH" || exit 1

echo
echo "=== ÉTAT GIT ==="

if [ -n "$(git status --porcelain)" ]; then
    echo "ERREUR : le dépôt doit être propre avant l’évaluation"
    git status --short
    exit 1
fi

if [ -e "$RESULT_FILE" ]; then
    echo "ERREUR : un résultat TDI-5 existe déjà"
    ls -lh "$RESULT_FILE"
    exit 1
fi

git status --short --branch

echo
echo "=== COMPILATION RELEASE HORS LIGNE ==="

cargo build \
    --release \
    --offline \
    --bin "$BINARY_NAME" ||
    exit 1

mkdir -p "$RESULT_DIR"

echo
echo "=== ÉVALUATION PRÉENREGISTRÉE UNIQUE ==="

"$CARGO_TARGET_DIR/release/$BINARY_NAME" |
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
