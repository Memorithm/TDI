#![cfg(feature = "experimental")]

//! TDI-24 slice 30: bounded Stage-C preflight smoke campaign.
//!
//! Exercises the merged Phase-C machinery (slices 21–29) end-to-end on
//! synthetic Development/Validation cases under a capped budget. These tests
//! qualify software semantics only: no training, no protected/final access,
//! no performance or scientific claim.

use tdi_ai::experimental::tdi24_eval::{
    C6_EVALUATOR_CONTRACT, EvalArm, EvalError, EvalOutcome, FAILURE_TAXONOMY_CONTRACT,
    MAX_CASES_PER_RUN, MAX_PREFLIGHT_PAIRS_PER_FAMILY, METRIC_REGISTRY_CONTRACT, MetricRegistry,
    PAIRED_UNCERTAINTY_CONTRACT, PROVENANCE_ENVELOPE_CONTRACT, PROVENANCE_TOOLCHAIN_ID,
    STAGE_C_PREFLIGHT_CONTRACT, STAGE_C_PREFLIGHT_FAMILIES, StageCPreflightBudget,
    V6_EVALUATOR_CONTRACT, run_stage_c_preflight, run_stage_c_preflight_for_label,
    validate_stage_c_preflight_report,
};
use tdi_ai::experimental::tdi24_tasks::{DataSplit, PROTECTED_LABEL_CONTRACT, TaskFamily};

#[test]
fn stage_c_preflight_runs_end_to_end_on_both_non_final_splits() {
    assert_eq!(STAGE_C_PREFLIGHT_CONTRACT, "tdi24-stage-c-preflight-v1");
    assert_eq!(MAX_PREFLIGHT_PAIRS_PER_FAMILY * 8, MAX_CASES_PER_RUN);
    for split in [DataSplit::Development, DataSplit::Validation] {
        let budget = StageCPreflightBudget::bounded(4, 3);
        let report = run_stage_c_preflight(split, budget).unwrap();
        validate_stage_c_preflight_report(&report).unwrap();

        assert_eq!(report.split, split);
        assert_eq!(report.cases_per_arm, 32);
        assert_eq!(report.v6_records.len(), 32);
        assert_eq!(report.c6_records.len(), 32);
        assert!(!report.protected_or_final_access);
        assert!(!report.training_executed);
        assert!(!report.scientific_claim);
        assert!(report.experimental_non_final);

        // Slices 23–25: matched non-trained capacity, initialization and budget.
        assert_eq!(report.matched_parameters.trainable_parameters, 0);
        assert_eq!(report.matched_initialization.left_arm, EvalArm::V6);
        assert_eq!(report.matched_initialization.right_arm, EvalArm::C6);
        assert_eq!(report.matched_optimizer_budget.examples, 32);
        assert_eq!(report.matched_optimizer_budget.updates, 0);
        // Slice 26: pinned registry.
        assert_eq!(report.metric_registry, MetricRegistry::pinned());

        // Slices 21/22: every record is a sealed, contract-pinned evaluator row.
        for (v6, c6) in report.v6_records.iter().zip(&report.c6_records) {
            assert_eq!(v6.arm, EvalArm::V6);
            assert_eq!(c6.arm, EvalArm::C6);
            assert_eq!(v6.split, split);
            assert_eq!(v6.arm_contract, V6_EVALUATOR_CONTRACT);
            assert_eq!(c6.arm_contract, C6_EVALUATOR_CONTRACT);
            assert_eq!(v6.metric_registry_contract, METRIC_REGISTRY_CONTRACT);
            assert_eq!(v6.label_contract, PROTECTED_LABEL_CONTRACT);
            assert_eq!(
                (v6.family, v6.case_id, v6.group_id, &v6.canonical_digest),
                (c6.family, c6.case_id, c6.group_id, &c6.canonical_digest)
            );
            assert!(matches!(v6.outcome, EvalOutcome::Scored { .. }));
            assert!(matches!(c6.outcome, EvalOutcome::Scored { .. }));
        }
        // Every declared Phase-B family is exercised with both members per pair.
        for family in STAGE_C_PREFLIGHT_FAMILIES {
            let count = report
                .v6_records
                .iter()
                .filter(|record| record.family == *family)
                .count();
            assert_eq!(count, 8, "family {}", family.as_str());
        }

        // Slice 28: failures retained (none on the reference fixtures).
        assert!(report.v6_failures.records().is_empty());
        assert!(report.c6_failures.records().is_empty());
        assert_eq!(
            report.v6_failures.taxonomy_contract(),
            FAILURE_TAXONOMY_CONTRACT
        );

        // Slice 27: paired summary over revealed bits only.
        let summary = report.paired_summary.as_ref().unwrap();
        assert_eq!(summary.n_pairs, 32);
        assert_eq!(summary.split, split);
        assert_eq!(summary.uncertainty_contract, PAIRED_UNCERTAINTY_CONTRACT);
        assert!(summary.paired_difference_ci.lower <= summary.paired_difference_ci.upper);

        // Slice 29: paired provenance envelopes bound to each arm.
        assert_eq!(
            report.v6_provenance.provenance_contract,
            PROVENANCE_ENVELOPE_CONTRACT
        );
        assert_eq!(report.v6_provenance.arm, EvalArm::V6);
        assert_eq!(report.c6_provenance.arm, EvalArm::C6);
        assert_eq!(
            report.v6_provenance.seed_identity,
            report.c6_provenance.seed_identity
        );
        assert_eq!(
            report.v6_provenance.toolchain_identity(),
            PROVENANCE_TOOLCHAIN_ID
        );
        assert!(
            report
                .v6_provenance
                .seed_identity
                .contains(&format!("domain={}", split.as_str()))
        );
    }
}

#[test]
fn stage_c_preflight_is_deterministic_and_split_sensitive() {
    let budget = StageCPreflightBudget::bounded(2, 0);
    let first = run_stage_c_preflight(DataSplit::Development, budget).unwrap();
    let second = run_stage_c_preflight(DataSplit::Development, budget).unwrap();
    assert_eq!(first, second);

    let validation = run_stage_c_preflight(DataSplit::Validation, budget).unwrap();
    assert_ne!(
        first.v6_records[0].canonical_digest,
        validation.v6_records[0].canonical_digest
    );
    assert_ne!(
        first.v6_provenance.seed_identity,
        validation.v6_provenance.seed_identity
    );
    assert_ne!(
        first.v6_provenance.data_identity,
        validation.v6_provenance.data_identity
    );
}

#[test]
fn stage_c_preflight_full_budget_fits_the_case_cap() {
    let budget = StageCPreflightBudget::bounded(MAX_PREFLIGHT_PAIRS_PER_FAMILY, 0);
    let report = run_stage_c_preflight(DataSplit::Validation, budget).unwrap();
    assert_eq!(report.cases_per_arm, MAX_CASES_PER_RUN);
    assert_eq!(report.v6_records.len() as u64, MAX_CASES_PER_RUN);
    assert_eq!(report.c6_records.len() as u64, MAX_CASES_PER_RUN);
}

#[test]
fn stage_c_preflight_rejects_unbounded_or_drifted_budgets() {
    let reject = |budget: StageCPreflightBudget, reason: &'static str| {
        assert_eq!(
            run_stage_c_preflight(DataSplit::Development, budget),
            Err(EvalError::StageCPreflightInvalid { reason })
        );
    };
    reject(StageCPreflightBudget::bounded(0, 0), "empty_budget");
    reject(
        StageCPreflightBudget::bounded(MAX_PREFLIGHT_PAIRS_PER_FAMILY + 1, 0),
        "budget_exceeds_case_cap",
    );
    reject(
        StageCPreflightBudget::bounded(1, u64::MAX),
        "pair_id_overflow",
    );
    reject(
        StageCPreflightBudget::bounded(1, 1_u64 << 61),
        "case_generation_failed",
    );
    let mut drifted = StageCPreflightBudget::bounded(1, 0);
    drifted.preflight_contract = "tdi24-stage-c-preflight-v0";
    reject(drifted, "contract_drift");
}

#[test]
fn stage_c_preflight_has_zero_protected_or_final_access() {
    let budget = StageCPreflightBudget::bounded(1, 0);
    for label in ["protected", "final", "holdout", "", "Development"] {
        assert_eq!(
            run_stage_c_preflight_for_label(label, budget),
            Err(EvalError::ProtectedOrFinalSplit),
            "label {label:?}"
        );
    }
    let report = run_stage_c_preflight_for_label("validation", budget).unwrap();
    assert_eq!(report.split, DataSplit::Validation);

    // Tampered reports cannot claim protected/final access, training or a result.
    let mut tampered = report.clone();
    tampered.protected_or_final_access = true;
    assert_eq!(
        validate_stage_c_preflight_report(&tampered),
        Err(EvalError::StageCPreflightInvalid {
            reason: "protected_or_final_access_forbidden",
        })
    );
    let mut tampered = report.clone();
    tampered.training_executed = true;
    assert_eq!(
        validate_stage_c_preflight_report(&tampered),
        Err(EvalError::StageCPreflightInvalid {
            reason: "training_forbidden",
        })
    );
    let mut tampered = report.clone();
    tampered.scientific_claim = true;
    assert_eq!(
        validate_stage_c_preflight_report(&tampered),
        Err(EvalError::StageCPreflightInvalid {
            reason: "scientific_claim_forbidden",
        })
    );
    let mut tampered = report;
    tampered.experimental_non_final = false;
    assert_eq!(
        validate_stage_c_preflight_report(&tampered),
        Err(EvalError::StageCPreflightInvalid {
            reason: "experimental_non_final_required",
        })
    );
}

#[test]
fn stage_c_preflight_report_rejects_dropped_or_unpaired_cases() {
    let report =
        run_stage_c_preflight(DataSplit::Development, StageCPreflightBudget::bounded(1, 0))
            .unwrap();

    let mut dropped = report.clone();
    dropped.c6_records.pop();
    assert_eq!(
        validate_stage_c_preflight_report(&dropped),
        Err(EvalError::StageCPreflightInvalid {
            reason: "unaccounted_cases",
        })
    );

    let mut swapped = report.clone();
    core::mem::swap(&mut swapped.v6_provenance, &mut swapped.c6_provenance);
    assert_eq!(
        validate_stage_c_preflight_report(&swapped),
        Err(EvalError::StageCPreflightInvalid {
            reason: "provenance_binding_mismatch",
        })
    );

    let mut relabeled = report.clone();
    relabeled.v6_records[0].family = TaskFamily::NonChiralControl;
    relabeled.v6_records[0].arm = EvalArm::C6;
    assert!(validate_stage_c_preflight_report(&relabeled).is_err());

    let mut hidden = report.clone();
    hidden.paired_summary = None;
    assert_eq!(
        validate_stage_c_preflight_report(&hidden),
        Err(EvalError::StageCPreflightInvalid {
            reason: "missing_paired_summary",
        })
    );

    let mut miscounted = report;
    miscounted.cases_per_arm = 16;
    assert_eq!(
        validate_stage_c_preflight_report(&miscounted),
        Err(EvalError::StageCPreflightInvalid {
            reason: "case_count_mismatch",
        })
    );
}

#[test]
fn stage_c_preflight_manifest_remains_non_authorizing() {
    let manifest = include_str!("../../docs/tdi24-stage-c-preflight.yaml");
    assert!(manifest.contains("stage: C"));
    assert!(manifest.contains("status: candidate_pending_exact_head_qualification_and_merge"));
    assert!(manifest.contains("preflight: tdi24-stage-c-preflight-v1"));
    assert!(manifest.contains("protected_or_final_access: false"));
    assert!(manifest.contains("confirmatory_execution: false"));
    assert!(manifest.contains("training: false"));
    assert!(manifest.contains("scientific_claim: false"));
    assert!(manifest.contains("performance_claim: false"));
    assert!(manifest.contains("max_cases_per_arm: 64"));
    assert!(!manifest.contains("freeze_pin"));
}
