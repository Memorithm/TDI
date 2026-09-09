#!/usr/bin/env python3
from pathlib import Path

PATH = Path("docs/TDI-11.4-DECOMPOSED-ENTAILMENT-PREREGISTRATION.md")

REQUIRED_SNIPPETS = (
    "Status: NON-FINAL PREREGISTRATION ONLY",
    "Execution remains blocked by the TDI-11.2 model/observation freeze gate.",
    "Threshold selection, aggregation rules and combination weights are Development-only.",
    "Evaluate once on Validation under the frozen configuration.",
    "No final dataset, final seed list, final result payload or confirmatory evaluation may be created by this study.",
    "Do not rescue a failed hypothesis by changing the detector, threshold or aggregation rule under the same hypothesis id.",
    "NLI entailment must never be treated as formal proof.",
)

FORBIDDEN_SNIPPETS = (
    "Threshold selection, aggregation rules and combination weights are Development/Validation-only.",
    "model execution is authorized",
    "final test is authorized",
)


def fail(message: str) -> None:
    raise SystemExit(f"TDI-11.4 decomposed-entailment preregistration invalid: {message}")


def main() -> None:
    text = PATH.read_text(encoding="utf-8")

    for snippet in REQUIRED_SNIPPETS:
        if snippet not in text:
            fail(f"required contract missing: {snippet!r}")

    for snippet in FORBIDDEN_SNIPPETS:
        if snippet in text:
            fail(f"forbidden contract present: {snippet!r}")

    required_headings = {
        "## Primary hypothesis H11-DE",
        "## Baseline fairness",
        "## Analysis plan",
        "## Falsification / stop conditions",
        "## Transferability gate",
        "## Non-claims",
    }
    missing_headings = sorted(heading for heading in required_headings if heading not in text)
    if missing_headings:
        fail(f"required sections missing: {missing_headings}")

    if "B1 and B2 must receive identical admissible source material" not in text:
        fail("matched source-access baseline contract is missing")

    if "Development-frozen operating points" not in text:
        fail("operating points must remain Development-frozen")

    print("TDI-11.4 decomposed-entailment preregistration: valid, non-final, non-executing")


if __name__ == "__main__":
    main()
