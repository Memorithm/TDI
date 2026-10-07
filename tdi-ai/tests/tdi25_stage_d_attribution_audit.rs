//! TDI-25 slice 40: Stage-D attribution audit.
//!
//! Regenerates and validates every Phase-C/D slice (30 to 39) and records
//! which claims remain admissible: software semantics only. No training, no
//! protected/final access, no scientific attribution or claim.
#![cfg(feature = "experimental")]

use tdi_ai::experimental::tdi25_eval::{
    ATTRIBUTION_AUDITED_SLICES, AttributionClaim, DATA_VOLUME_SCALING_CONTRACT, EvalError,
    STAGE_C_PREFLIGHT_CONTRACT, STAGE_D_ATTRIBUTION_AUDIT_CONTRACT, StageCPreflightBudget,
    StageDAttributionAuditReport, run_stage_d_attribution_audit,
    run_stage_d_attribution_audit_for_label, validate_stage_d_attribution_audit_report,
};
use tdi_ai::experimental::tdi25_tasks::DataSplit;

fn audit(split: DataSplit) -> StageDAttributionAuditReport {
    run_stage_d_attribution_audit(split, StageCPreflightBudget::smoke()).unwrap()
}

fn rejection(report: &StageDAttributionAuditReport) -> &'static str {
    match validate_stage_d_attribution_audit_report(report) {
        Err(EvalError::StageDAttributionAuditInvalid { reason }) => reason,
        other => panic!("expected an attribution-audit rejection, got {other:?}"),
    }
}

#[test]
fn registry_covers_slices_30_to_39_in_order() {
    assert_eq!(
        STAGE_D_ATTRIBUTION_AUDIT_CONTRACT,
        "tdi25-stage-d-attribution-audit-v1"
    );
    let slices: Vec<u8> = ATTRIBUTION_AUDITED_SLICES.iter().map(|e| e.0).collect();
    assert_eq!(slices, (30..=39).collect::<Vec<u8>>());
    assert_eq!(ATTRIBUTION_AUDITED_SLICES[0].1, STAGE_C_PREFLIGHT_CONTRACT);
    assert_eq!(
        ATTRIBUTION_AUDITED_SLICES[9].1,
        DATA_VOLUME_SCALING_CONTRACT
    );
}

#[test]
fn every_slice_supports_software_semantics_only() {
    for split in [DataSplit::Development, DataSplit::Validation] {
        let report = audit(split);
        assert_eq!(report.entries.len(), 10);
        let mut digests: Vec<u64> = report.entries.iter().map(|e| e.evidence_digest).collect();
        digests.sort_unstable();
        digests.dedup();
        assert_eq!(digests.len(), 10);
        for entry in &report.entries {
            assert!(entry.validated);
            assert!(!entry.protected_or_final_access);
            assert!(!entry.training_executed);
            assert!(!entry.scientific_claim);
            assert!(entry.experimental_non_final);
            assert_eq!(entry.admissible, AttributionClaim::SoftwareSemanticsOnly);
        }
        assert!(!report.scientific_attribution_admissible);
        assert!(!report.protected_or_final_access);
        assert!(!report.training_executed);
        assert!(!report.scientific_claim);
        assert!(report.experimental_non_final);
    }
}

#[test]
fn tampered_audits_fail_closed() {
    let report = audit(DataSplit::Development);
    let tamper = |mutate: &dyn Fn(&mut StageDAttributionAuditReport), reason: &str| {
        let mut tampered = report.clone();
        mutate(&mut tampered);
        assert_eq!(rejection(&tampered), reason);
    };
    tamper(&|r| r.audit_contract = "legacy", "contract_drift");
    tamper(
        &|r| {
            r.entries.pop();
        },
        "registry_drift",
    );
    tamper(&|r| r.entries.swap(0, 1), "registry_drift");
    tamper(&|r| r.entries[3].contract = "legacy", "registry_drift");
    tamper(
        &|r| r.entries[0].admissible = AttributionClaim::Withheld,
        "admissibility_drift",
    );
    // An upgraded flag cannot be laundered into an admissible entry.
    tamper(
        &|r| r.entries[2].scientific_claim = true,
        "admissibility_drift",
    );
    tamper(
        &|r| {
            r.entries[2].scientific_claim = true;
            r.entries[2].admissible = AttributionClaim::Withheld;
        },
        "withheld_entry",
    );
    tamper(
        &|r| {
            r.entries[5].validated = false;
            r.entries[5].admissible = AttributionClaim::Withheld;
        },
        "withheld_entry",
    );
    tamper(
        &|r| r.scientific_attribution_admissible = true,
        "scientific_attribution_admissible",
    );
    tamper(
        &|r| r.protected_or_final_access = true,
        "protected_or_final_access",
    );
    tamper(&|r| r.training_executed = true, "training_executed");
    tamper(&|r| r.scientific_claim = true, "scientific_claim");
    tamper(
        &|r| r.experimental_non_final = false,
        "experimental_non_final",
    );
    tamper(&|r| r.split = DataSplit::Validation, "case_evidence_drift");
    tamper(
        &|r| r.entries[9].evidence_digest ^= 1,
        "case_evidence_drift",
    );
    tamper(
        &|r| {
            r.budget = StageCPreflightBudget {
                seed_blocks: 2,
                cases_per_block: 4,
            }
        },
        "case_evidence_drift",
    );
}

#[test]
fn protected_or_final_labels_and_bad_budgets_never_score() {
    for label in ["protected", "final", "holdout", ""] {
        assert_eq!(
            run_stage_d_attribution_audit_for_label(label, StageCPreflightBudget::smoke()),
            Err(EvalError::ProtectedOrFinalSplit)
        );
    }
    assert!(
        run_stage_d_attribution_audit(
            DataSplit::Development,
            StageCPreflightBudget {
                seed_blocks: 0,
                cases_per_block: 8
            }
        )
        .is_err()
    );
    assert!(
        run_stage_d_attribution_audit(
            DataSplit::Development,
            StageCPreflightBudget {
                seed_blocks: 2,
                cases_per_block: 1
            }
        )
        .is_err()
    );
}

#[test]
fn audit_is_deterministic() {
    assert_eq!(audit(DataSplit::Validation), audit(DataSplit::Validation));
}
