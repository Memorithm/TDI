#![cfg(feature = "experimental")]

//! TDI-24 slice 36: structure-preserving learned basis prototype (Phase D).
//!
//! An orthogonal parameterisation `O(theta)` (product of the 15 Givens
//! rotations of the six-slot carrier) is evaluated at four declared probes
//! with the unchanged C6 reference weights. These tests qualify software
//! semantics and identities only: no training, no angle selection, no
//! protected/final access, no attribution or scientific claim.

use tdi_ai::experimental::tdi24_chiral::{Chiral6, chiral_score};
use tdi_ai::experimental::tdi24_eval::{
    C6_REFERENCE_WEIGHTS, EvalArm, EvalError, EvalOutcome, LEARNED_BASIS_PARAMETER_COUNT,
    LEARNED_BASIS_PROBE_COUNT, LEARNED_BASIS_PROTOTYPE_CONTRACT, LEARNED_BASIS_TOLERANCE,
    LearnedBasisProbeRole, LearnedBasisPrototypeReport, MAX_PREFLIGHT_PAIRS_PER_FAMILY,
    STAGE_C_PREFLIGHT_FAMILIES, StageCPreflightBudget, TrainableCapacity,
    learned_basis_commutes_with_structure, learned_basis_planes, learned_basis_probes,
    learned_basis_transform, run_learned_basis_prototype, run_learned_basis_prototype_for_label,
    run_stage_c_preflight, validate_learned_basis_probe, validate_learned_basis_prototype_report,
    validate_learned_basis_transform,
};
use tdi_ai::experimental::tdi24_tasks::DataSplit;

fn rejection(report: &LearnedBasisPrototypeReport) -> &'static str {
    match validate_learned_basis_prototype_report(report) {
        Err(EvalError::LearnedBasisPrototypeInvalid { reason }) => reason,
        other => panic!("expected a learned-basis prototype rejection, got {other:?}"),
    }
}

fn reason(result: Result<(), EvalError>) -> &'static str {
    match result {
        Err(EvalError::LearnedBasisPrototypeInvalid { reason }) => reason,
        other => panic!("expected a learned-basis prototype rejection, got {other:?}"),
    }
}

#[test]
fn contract_pin_and_parameterisation_are_declared() {
    assert_eq!(
        LEARNED_BASIS_PROTOTYPE_CONTRACT,
        "tdi24-learned-basis-prototype-v1"
    );
    // C(6,2) Givens planes: derived from the carrier width, not tuned.
    assert_eq!(LEARNED_BASIS_PARAMETER_COUNT, 15);
    assert_eq!(LEARNED_BASIS_PROBE_COUNT, 4);
    let planes = learned_basis_planes();
    assert_eq!(planes[0], (0, 1));
    assert_eq!(planes[14], (4, 5));
    assert!(planes.windows(2).all(|pair| pair[0] < pair[1]));
}

#[test]
fn declared_probes_are_orthogonal_and_preserve_the_algebra() {
    let probes = learned_basis_probes();
    let roles: Vec<_> = probes.iter().map(|probe| probe.role).collect();
    assert_eq!(
        roles,
        [
            LearnedBasisProbeRole::Identity,
            LearnedBasisProbeRole::Gauge,
            LearnedBasisProbeRole::SectorMixing,
            LearnedBasisProbeRole::Generic,
        ]
    );
    for (index, probe) in probes.iter().enumerate() {
        assert_eq!(probe.index, index);
        validate_learned_basis_probe(probe).unwrap();
        let transform = learned_basis_transform(probe).unwrap();
        validate_learned_basis_transform(&transform).unwrap();
        let commutes = learned_basis_commutes_with_structure(&transform);
        assert_eq!(
            commutes,
            matches!(
                probe.role,
                LearnedBasisProbeRole::Identity | LearnedBasisProbeRole::Gauge
            )
        );
    }
}

#[test]
fn non_orthogonal_or_non_finite_transforms_are_rejected() {
    let identity = learned_basis_transform(&learned_basis_probes()[0]).unwrap();
    let mut scaled = identity;
    scaled[0][0] = 2.0;
    assert_eq!(
        reason(validate_learned_basis_transform(&scaled)),
        "orthogonality_failure"
    );
    let mut sheared = identity;
    sheared[0][3] = 0.5;
    assert_eq!(
        reason(validate_learned_basis_transform(&sheared)),
        "orthogonality_failure"
    );
    let mut nan = identity;
    nan[2][2] = f64::NAN;
    assert_eq!(
        reason(validate_learned_basis_transform(&nan)),
        "non_finite_parameter"
    );
    let mut probe = learned_basis_probes()[3];
    probe.angles[0] = f64::INFINITY;
    assert_eq!(
        reason(validate_learned_basis_probe(&probe)),
        "non_finite_parameter"
    );
    let mut drifted = learned_basis_probes()[2];
    drifted.angles[0] = 0.125;
    assert_eq!(
        reason(validate_learned_basis_probe(&drifted)),
        "probe_order_drift"
    );
    let mut relabelled = learned_basis_probes()[1];
    relabelled.role = LearnedBasisProbeRole::Generic;
    assert_eq!(
        reason(validate_learned_basis_probe(&relabelled)),
        "probe_order_drift"
    );
}

#[test]
fn identity_probe_reproduces_the_stage_c_c6_evaluator_exactly() {
    let budget = StageCPreflightBudget::bounded(3, 1);
    let report = run_learned_basis_prototype(DataSplit::Development, budget).unwrap();
    let preflight = run_stage_c_preflight(DataSplit::Development, budget).unwrap();
    let identity: Vec<_> = report
        .cases
        .iter()
        .filter(|case| case.probe_index == 0)
        .collect();
    assert_eq!(identity.len(), preflight.c6_records.len());
    for (case, record) in identity.iter().zip(&preflight.c6_records) {
        assert_eq!(record.arm, EvalArm::C6);
        assert_eq!((case.family, case.case_id), (record.family, record.case_id));
        match record.outcome {
            EvalOutcome::Scored { score, correct } => {
                assert_eq!(score.to_bits(), case.score.to_bits());
                assert_eq!(score.to_bits(), case.reference_score.to_bits());
                assert_eq!(correct, case.correct);
            }
            ref other => panic!("unexpected outcome {other:?}"),
        }
    }
}

#[test]
fn hand_calculated_sector_mixing_score() {
    // Probe 2 rotates the (0,4) plane by pi/4: x0' = (x0 - x4)/sqrt2,
    // x4' = (x0 + x4)/sqrt2. Slots 0 and 4 are not paired by J, so chi moves.
    let probe = learned_basis_probes()[2];
    let transform = learned_basis_transform(&probe).unwrap();
    let apply = |carrier: Chiral6| {
        let x = carrier.as_array();
        let mut y = [0.0; 6];
        for row in 0..6 {
            y[row] = (0..6)
                .map(|column| transform[row][column] * x[column])
                .sum();
        }
        Chiral6::from_array(y).unwrap()
    };
    // q = e0, k = e3: canonical s = 0, chi = 1.
    let query = Chiral6::new([1.0, 0.0, 0.0], [0.0, 0.0, 0.0]).unwrap();
    let key = Chiral6::new([0.0, 0.0, 0.0], [1.0, 0.0, 0.0]).unwrap();
    assert_eq!(chiral_score(query, key, C6_REFERENCE_WEIGHTS).unwrap(), 1.0);
    // q' = (e0 + e4)/sqrt2, k' = e3: chi = 1/sqrt2, s = 0.
    let rotated = chiral_score(apply(query), apply(key), C6_REFERENCE_WEIGHTS).unwrap();
    assert!((rotated - core::f64::consts::FRAC_1_SQRT_2).abs() <= LEARNED_BASIS_TOLERANCE);
}

#[test]
fn smoke_study_covers_every_probe_and_family_on_both_non_final_splits() {
    let budget = StageCPreflightBudget::bounded(MAX_PREFLIGHT_PAIRS_PER_FAMILY, 0);
    for split in [DataSplit::Development, DataSplit::Validation] {
        let report = run_learned_basis_prototype(split, budget).unwrap();
        let per_probe = budget.cases_per_arm().unwrap();
        assert_eq!(report.split, split);
        assert_eq!(report.prototype_contract, LEARNED_BASIS_PROTOTYPE_CONTRACT);
        assert_eq!(report.probes, learned_basis_probes().to_vec());
        assert_eq!(report.weights, C6_REFERENCE_WEIGHTS);
        assert_eq!(report.capacity, TrainableCapacity::reference_c6());
        assert_eq!(report.basis_parameter_count, LEARNED_BASIS_PARAMETER_COUNT);
        assert_eq!(
            report.cases.len() as u64,
            per_probe * LEARNED_BASIS_PROBE_COUNT as u64
        );
        assert_eq!(
            report.summaries.len(),
            LEARNED_BASIS_PROBE_COUNT * STAGE_C_PREFLIGHT_FAMILIES.len()
        );
        assert!(!report.protected_or_final_access);
        assert!(!report.training_executed);
        assert!(!report.scientific_claim);
        assert!(report.experimental_non_final);
        for group in report.cases.chunks(LEARNED_BASIS_PROBE_COUNT) {
            let gauge = &group[1];
            assert!(
                (gauge.score - gauge.reference_score).abs()
                    <= LEARNED_BASIS_TOLERANCE * gauge.reference_score.abs().max(1.0)
            );
            assert_eq!(gauge.correct, group[0].correct);
        }
        // Non-gauge probes are not score-invariant on the case stream.
        for probe_index in [2, 3] {
            assert!(
                report
                    .cases
                    .chunks(LEARNED_BASIS_PROBE_COUNT)
                    .any(|group| (group[probe_index].score - group[0].score).abs()
                        > LEARNED_BASIS_TOLERANCE * group[0].score.abs().max(1.0))
            );
        }
        validate_learned_basis_prototype_report(&report).unwrap();
    }
}

#[test]
fn tampered_reports_fail_closed() {
    let report =
        run_learned_basis_prototype(DataSplit::Development, StageCPreflightBudget::bounded(2, 0))
            .unwrap();
    let tamper = |mutate: &dyn Fn(&mut LearnedBasisPrototypeReport), reason: &str| {
        let mut tampered = report.clone();
        mutate(&mut tampered);
        assert_eq!(rejection(&tampered), reason);
    };
    tamper(
        &|r| r.prototype_contract = "tdi24-learned-basis-prototype-v0",
        "contract_drift",
    );
    tamper(
        &|r| {
            r.probes.pop();
        },
        "probe_set_incomplete",
    );
    tamper(&|r| r.probes.swap(0, 1), "probe_order_drift");
    tamper(&|r| r.probes[3].angles[4] += 0.01, "probe_order_drift");
    tamper(&|r| r.weights.gamma = 0.0, "weights_drift");
    tamper(
        &|r| r.capacity = TrainableCapacity::reference_v6(),
        "capacity_mismatch",
    );
    tamper(&|r| r.basis_parameter_count = 16, "parameter_count_drift");
    tamper(
        &|r| {
            r.cases.pop();
        },
        "case_count",
    );
    tamper(&|r| r.cases.swap(0, 1), "case_order");
    tamper(&|r| r.cases[0].score += 1.0, "canonical_reference_drift");
    tamper(&|r| r.cases[1].score += 1.0, "gauge_invariance_drift");
    tamper(&|r| r.summaries[0].correct += 1, "summary_drift");
    // A coherent forgery on a free probe is still caught by regeneration.
    tamper(&|r| r.cases[3].score += 1.0, "case_evidence_drift");
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
}

#[test]
fn protected_or_final_labels_and_bad_budgets_never_score() {
    for label in ["protected", "final", "holdout", ""] {
        assert_eq!(
            run_learned_basis_prototype_for_label(label, StageCPreflightBudget::bounded(1, 0)),
            Err(EvalError::ProtectedOrFinalSplit)
        );
    }
    assert_eq!(
        run_learned_basis_prototype(
            DataSplit::Development,
            StageCPreflightBudget::bounded(MAX_PREFLIGHT_PAIRS_PER_FAMILY + 1, 0)
        ),
        Err(EvalError::StageCPreflightInvalid {
            reason: "budget_exceeds_case_cap"
        })
    );
    assert_eq!(
        run_learned_basis_prototype(DataSplit::Development, StageCPreflightBudget::bounded(0, 0)),
        Err(EvalError::StageCPreflightInvalid {
            reason: "empty_budget"
        })
    );
}

#[test]
fn study_is_deterministic() {
    let budget = StageCPreflightBudget::bounded(2, 0);
    assert_eq!(
        run_learned_basis_prototype(DataSplit::Validation, budget).unwrap(),
        run_learned_basis_prototype(DataSplit::Validation, budget).unwrap()
    );
}
