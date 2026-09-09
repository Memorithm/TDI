#!/usr/bin/env python3
import json
from pathlib import Path

PATH = Path("docs/tdi11.4-uncertainty-stratified-preregistration.json")
REQUIRED_TOP = {
    "schema", "status", "external_prior_art", "observation", "hypothesis",
    "design", "anti_leakage", "acceptance_rule", "stop_conditions",
}
REQUIRED_POPULATIONS = {"Development", "Validation"}
REQUIRED_STRATA = {"model_id", "task_id", "hallucination_type"}
REQUIRED_HALLUCINATION_TYPES = {"intrinsic", "extrinsic"}
REQUIRED_ESTIMATOR_FAMILIES = {"information_theoretic", "sampling_based", "reflexive"}
REQUIRED_BASELINES = {"constant_score", "random_rank", "existing_authorized_TDI11_signal_if_available"}
REQUIRED_METRICS = {"AUROC", "AUPRC", "risk_coverage_curve"}
REQUIRED_CALIBRATION_METRICS = {"Brier_score", "expected_calibration_error"}
REQUIRED_REPORTING = {
    "per_stratum_metrics",
    "macro_aggregate_only_after_per_stratum_reporting",
    "confidence_intervals_or_resampling_uncertainty",
    "negative_results_preserved",
    "estimator_compute_cost",
    "missing_or_invalid_signal_rate",
}


def fail(message: str) -> None:
    raise SystemExit(f"TDI-11.4 preregistration invalid: {message}")


def main() -> None:
    data = json.loads(PATH.read_text(encoding="utf-8"))
    missing = REQUIRED_TOP - data.keys()
    if missing:
        fail(f"missing top-level keys: {sorted(missing)}")

    if data["schema"] != "tdi11.4-uncertainty-stratified-preregistration-v1":
        fail("unexpected schema")
    if data["status"] != "preregistered_nonfinal_nonexecuting":
        fail("artifact must remain non-final and non-executing")

    prior = data["external_prior_art"]
    if prior.get("role") != "external_prior_art_only_not_memorithm_evidence":
        fail("external paper must not be represented as Memorithm evidence")
    if prior.get("arxiv_id") != "2605.27016":
        fail("prior-art identity changed")

    design = data["design"]
    if set(design.get("populations", [])) != REQUIRED_POPULATIONS:
        fail("Development and Validation populations must remain explicitly frozen")
    if design.get("final_test_allowed") is not False:
        fail("final test must remain forbidden")
    if design.get("model_execution_allowed_by_this_artifact") is not False:
        fail("this artifact must not authorize model execution")
    if set(design.get("stratification_axes", [])) != REQUIRED_STRATA:
        fail("model/task/hallucination-type stratification is mandatory")
    if set(design.get("hallucination_types", [])) != REQUIRED_HALLUCINATION_TYPES:
        fail("intrinsic/extrinsic hallucination categories must remain frozen")
    if set(design.get("candidate_estimator_families", [])) != REQUIRED_ESTIMATOR_FAMILIES:
        fail("candidate estimator families changed")
    if set(design.get("minimum_baselines", [])) != REQUIRED_BASELINES:
        fail("minimum baselines must remain frozen")
    if not REQUIRED_METRICS.issubset(design.get("primary_metrics", [])):
        fail("required discrimination and risk-coverage metrics missing")
    if set(design.get("calibration_metrics", [])) != REQUIRED_CALIBRATION_METRICS:
        fail("calibration metrics must remain frozen")
    if set(design.get("reporting_requirements", [])) != REQUIRED_REPORTING:
        fail("per-stratum reporting and negative-result requirements must remain frozen")

    leakage = data["anti_leakage"]
    if leakage.get("threshold_selection") != "Development_only":
        fail("threshold selection must be Development-only")
    if leakage.get("model_or_estimator_selection") != "Development_only":
        fail("model/estimator selection must be Development-only")
    if leakage.get("validation_use") != "single_frozen_protocol_evaluation_after_selection":
        fail("Validation must remain a single frozen-protocol evaluation after selection")
    if leakage.get("final_holdout") != "forbidden_until_separately_authorized":
        fail("final holdout protection changed")
    if leakage.get("no_benchmark_target_optimization") is not True:
        fail("benchmark-target optimization must remain prohibited")

    acceptance = data["acceptance_rule"]
    if acceptance.get("state") != "unresolved_blocking":
        fail("acceptance thresholds must remain unresolved before a separate freeze")
    required_before = set(acceptance.get("required_before_execution", []))
    for item in {
        "exact_model_freeze", "exact_observation_freeze", "exact_estimator_implementations",
        "exact_strata_membership", "exact_resampling_procedure", "numerical_materiality_thresholds",
    }:
        if item not in required_before:
            fail(f"missing pre-execution requirement: {item}")

    print("TDI-11.4 uncertainty preregistration: valid, non-final, non-executing")


if __name__ == "__main__":
    main()
