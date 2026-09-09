#!/usr/bin/env python3
import json
from pathlib import Path

PATH = Path("docs/tdi11.4-uncertainty-stratified-preregistration.json")
REQUIRED_TOP = {
    "schema", "status", "external_prior_art", "external_claim_level_prior_art",
    "observation", "hypothesis", "claim_level_secondary_hypothesis", "design",
    "anti_leakage", "acceptance_rule", "stop_conditions",
}
REQUIRED_POPULATIONS = {"Development", "Validation"}
REQUIRED_STRATA = {"model_id", "task_id", "hallucination_type"}
REQUIRED_HALLUCINATION_TYPES = {"intrinsic", "extrinsic"}
REQUIRED_ESTIMATOR_FAMILIES = {"information_theoretic", "sampling_based", "reflexive"}
REQUIRED_BASELINES = {"constant_score", "random_rank", "existing_authorized_TDI11_signal_if_available"}
REQUIRED_RESOLUTION_ABLATIONS = {"response_level", "claim_level"}
REQUIRED_ABSTENTION_ABLATIONS = {"no_abstention", "claim_level_selective_abstention"}
REQUIRED_METRICS = {"AUROC", "AUPRC", "risk_coverage_curve"}
REQUIRED_CALIBRATION_METRICS = {"Brier_score", "expected_calibration_error"}
REQUIRED_COVERAGE_UTILITY = {
    "answer_coverage", "claim_coverage", "false_abstention_rate",
    "originally_correct_answer_regression", "useful_task_performance",
}
REQUIRED_REPORTING = {
    "per_stratum_metrics",
    "macro_aggregate_only_after_per_stratum_reporting",
    "confidence_intervals_or_resampling_uncertainty",
    "negative_results_preserved",
    "estimator_compute_cost",
    "missing_or_invalid_signal_rate",
    "claim_segmentation_failure_rate",
    "abstention_rate_by_stratum",
}
REQUIRED_BEFORE_EXECUTION = {
    "exact_model_freeze", "exact_observation_freeze", "exact_estimator_implementations",
    "exact_strata_membership", "exact_resampling_procedure", "numerical_materiality_thresholds",
    "exact_claim_segmentation_contract", "exact_claim_level_score_aggregation",
    "exact_abstention_policy", "exact_coverage_and_utility_accounting",
}
REQUIRED_STOP_CONDITIONS = {
    "stop_and_report_if_signal_is_nonfinite_or_missing_above_the_frozen_tolerance",
    "stop_and_report_if_any_stratum_has_insufficient_authorized_samples",
    "stop_and_report_if_claim_segmentation_failure_exceeds_the_frozen_tolerance",
    "do_not_pool_away_a_failed_stratum",
    "do_not_retune_after_validation",
    "do_not_access_final_holdout",
}


def fail(message: str) -> None:
    raise SystemExit(f"TDI-11.4 preregistration invalid: {message}")


def require_external_prior(prior: dict, arxiv_id: str, label: str) -> None:
    if prior.get("role") != "external_prior_art_only_not_memorithm_evidence":
        fail(f"{label} must remain external prior art, not Memorithm evidence")
    if prior.get("arxiv_id") != arxiv_id:
        fail(f"{label} identity changed")


def main() -> None:
    data = json.loads(PATH.read_text(encoding="utf-8"))
    missing = REQUIRED_TOP - data.keys()
    if missing:
        fail(f"missing top-level keys: {sorted(missing)}")

    if data["schema"] != "tdi11.4-uncertainty-stratified-preregistration-v2":
        fail("unexpected schema")
    if data["status"] != "preregistered_nonfinal_nonexecuting":
        fail("artifact must remain non-final and non-executing")

    require_external_prior(data["external_prior_art"], "2605.27016", "uncertainty prior art")
    require_external_prior(
        data["external_claim_level_prior_art"], "2604.12046", "claim-level prior art"
    )

    secondary = data["claim_level_secondary_hypothesis"]
    if secondary.get("id") != "TDI11.4-H-CLAIM-CAL-02":
        fail("claim-level secondary hypothesis identity changed")
    if not secondary.get("statement") or not secondary.get("falsification"):
        fail("claim-level secondary hypothesis must remain explicitly falsifiable")

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
    if set(design.get("uncertainty_resolution_ablations", [])) != REQUIRED_RESOLUTION_ABLATIONS:
        fail("response-level/claim-level ablation must remain frozen")
    if set(design.get("abstention_ablations", [])) != REQUIRED_ABSTENTION_ABLATIONS:
        fail("no-abstention/selective-abstention ablation must remain frozen")
    if not REQUIRED_METRICS.issubset(design.get("primary_metrics", [])):
        fail("required discrimination and risk-coverage metrics missing")
    if set(design.get("calibration_metrics", [])) != REQUIRED_CALIBRATION_METRICS:
        fail("calibration metrics must remain frozen")
    if set(design.get("coverage_and_utility_metrics", [])) != REQUIRED_COVERAGE_UTILITY:
        fail("coverage/utility safeguards must remain frozen")
    if set(design.get("reporting_requirements", [])) != REQUIRED_REPORTING:
        fail("per-stratum, negative-result, claim and abstention reporting must remain frozen")

    leakage = data["anti_leakage"]
    if leakage.get("threshold_selection") != "Development_only":
        fail("threshold selection must be Development-only")
    if leakage.get("model_or_estimator_selection") != "Development_only":
        fail("model/estimator selection must be Development-only")
    if leakage.get("abstention_threshold_selection") != "Development_only":
        fail("abstention threshold selection must be Development-only")
    if leakage.get("claim_segmentation_contract_changes_after_validation") != "forbidden":
        fail("claim segmentation contract must not change after Validation begins")
    if leakage.get("validation_use") != "single_frozen_protocol_evaluation_after_selection":
        fail("Validation must remain a single frozen-protocol evaluation after selection")
    if leakage.get("final_holdout") != "forbidden_until_separately_authorized":
        fail("final holdout protection changed")
    if leakage.get("no_benchmark_target_optimization") is not True:
        fail("benchmark-target optimization must remain prohibited")

    acceptance = data["acceptance_rule"]
    if acceptance.get("state") != "unresolved_blocking":
        fail("acceptance thresholds must remain unresolved before a separate freeze")
    if set(acceptance.get("required_before_execution", [])) != REQUIRED_BEFORE_EXECUTION:
        fail("pre-execution freeze requirements changed")

    if set(data.get("stop_conditions", [])) != REQUIRED_STOP_CONDITIONS:
        fail("stop conditions changed")

    print("TDI-11.4 uncertainty preregistration v2: valid, non-final, non-executing")


if __name__ == "__main__":
    main()
