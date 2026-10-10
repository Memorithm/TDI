#![cfg(feature = "experimental")]

//! TDI-24 issue #690: authoritative Stage-C provenance (envelope v2).
//!
//! Negative and positive tests proving that no case is scored without a
//! validated envelope that binds the complete evaluator configuration, the
//! exact admitted population with per-case registered seeds, the evaluator
//! source digest and the compiler that actually built the crate. Software
//! semantics only: no training, no protected/final access, no scientific claim.

use tdi_ai::experimental::tdi24_eval::{
    AdmittedCase, BUILD_RUSTC_VERSION, EvalArm, EvalError, EvaluatorConfig, EvaluatorRun,
    LEGACY_PROVENANCE_ENVELOPE_CONTRACT, PROVENANCE_ENVELOPE_CONTRACT,
    PROVENANCE_SOURCE_DIGEST_CONTRACT, PROVENANCE_TOOLCHAIN_FEATURES, ProvenanceEnvelope,
    StageCPreflightBudget, build_rustc_release, handedness_sign, non_chiral_sign,
    run_stage_c_preflight, stage_c_code_identity_bundle, stage_c_config_identity_bundle,
    stage_c_data_identity_bundle, stage_c_preflight_admitted_cases, stage_c_seed_identity,
    stage_c_source_digest, validate_provenance_binding, validate_provenance_envelope,
    validate_stage_c_preflight_report,
};
use tdi_ai::experimental::tdi24_tasks::{
    DataSplit, SeedDomain, non_chiral_control_case, reflection_discriminative_pair, register_seed,
    seal_non_chiral_control, seal_reflection_discriminative,
};

const DEV: DataSplit = DataSplit::Development;

fn invalid<T>(reason: &'static str) -> Result<T, EvalError> {
    Err(EvalError::ProvenanceEnvelopeInvalid { reason })
}

/// Re-derive every identity for a (possibly tampered) admitted list so the
/// envelope is internally consistent and only the targeted check can fire.
fn consistent_envelope(
    config: &EvaluatorConfig,
    admitted: Vec<AdmittedCase>,
) -> Result<ProvenanceEnvelope, EvalError> {
    ProvenanceEnvelope::for_pinned_stage_c_run(config, admitted)
}

#[test]
fn envelope_v2_contract_supersedes_v1() {
    assert_eq!(PROVENANCE_ENVELOPE_CONTRACT, "tdi24-provenance-envelope-v2");
    assert_eq!(
        LEGACY_PROVENANCE_ENVELOPE_CONTRACT,
        "tdi24-provenance-envelope-v1"
    );
    let config = EvaluatorConfig::v6(DEV).unwrap();
    let admitted =
        stage_c_preflight_admitted_cases(DEV, StageCPreflightBudget::bounded(1, 0)).unwrap();
    let mut envelope = consistent_envelope(&config, admitted).unwrap();
    envelope.provenance_contract = LEGACY_PROVENANCE_ENVELOPE_CONTRACT;
    assert_eq!(
        validate_provenance_envelope(&envelope),
        invalid("contract_drift")
    );
}

#[test]
fn evaluation_requires_a_bound_envelope() {
    let case = seal_reflection_discriminative(&reflection_discriminative_pair(2).unwrap().right);
    let mut unbound = EvaluatorRun::open(EvaluatorConfig::v6(DEV).unwrap()).unwrap();
    assert_eq!(
        unbound.evaluate_v6_binary(&case, handedness_sign),
        invalid("provenance_required")
    );
    let mut unbound_c6 = EvaluatorRun::open(EvaluatorConfig::c6(DEV).unwrap()).unwrap();
    assert_eq!(
        unbound_c6.evaluate_c6_binary(&case, handedness_sign),
        invalid("provenance_required")
    );
    assert!(unbound.records().is_empty());
}

#[test]
fn unadmitted_tampered_and_repeated_cases_are_rejected_before_scoring() {
    let admitted_view = seal_non_chiral_control(&non_chiral_control_case(0).unwrap());
    let outsider = seal_non_chiral_control(&non_chiral_control_case(1).unwrap());
    let config = EvaluatorConfig::v6(DEV).unwrap();
    let entry = AdmittedCase::from_view(admitted_view.inference_view());

    let envelope = consistent_envelope(&config, vec![entry.clone()]).unwrap();
    let mut run = EvaluatorRun::open_with_provenance(config, envelope).unwrap();
    assert_eq!(
        run.evaluate_v6_binary(&outsider, non_chiral_sign),
        invalid("case_not_admitted")
    );
    assert!(run.records().is_empty());
    assert!(
        run.evaluate_v6_binary(&admitted_view, non_chiral_sign)
            .is_ok()
    );
    assert_eq!(
        run.evaluate_v6_binary(&admitted_view, non_chiral_sign),
        invalid("case_already_evaluated")
    );
    assert_eq!(run.records().len(), 1);

    // Same (family, case_id) but a different canonical population entry.
    let mut tampered = entry.clone();
    tampered.canonical_digest = "0000000000000000".to_string();
    let envelope = consistent_envelope(&config, vec![tampered]).unwrap();
    let mut run = EvaluatorRun::open_with_provenance(config, envelope).unwrap();
    assert_eq!(
        run.evaluate_v6_binary(&admitted_view, non_chiral_sign),
        invalid("case_population_mismatch")
    );

    let mut malformed = entry.clone();
    malformed.canonical_digest = "not-a-digest".to_string();
    assert_eq!(
        consistent_envelope(&config, vec![malformed]),
        invalid("malformed_case_digest")
    );

    assert_eq!(
        consistent_envelope(&config, vec![entry.clone(), entry]),
        invalid("duplicate_admitted_case")
    );
}

#[test]
fn per_case_seeds_are_bound_and_checked() {
    let config = EvaluatorConfig::c6(DEV).unwrap();
    let admitted =
        stage_c_preflight_admitted_cases(DEV, StageCPreflightBudget::bounded(2, 5)).unwrap();
    for case in &admitted {
        assert_eq!(
            case.seed,
            register_seed(SeedDomain::Development, case.family, case.group_id)
        );
    }
    // Seeds differ across pairs of one family: not a single run-level seed.
    let first = &admitted[0];
    assert!(
        admitted
            .iter()
            .any(|case| case.family == first.family && case.seed != first.seed)
    );

    let mut wrong_seed = admitted.clone();
    wrong_seed[0].seed = register_seed(SeedDomain::Development, first.family, first.group_id + 1);
    assert_eq!(
        consistent_envelope(&config, wrong_seed),
        invalid("case_seed_mismatch")
    );

    let mut wrong_group = admitted.clone();
    wrong_group[0].group_id += 1;
    assert_eq!(
        consistent_envelope(&config, wrong_group),
        invalid("case_seed_mismatch")
    );

    let mut wrong_domain = admitted.clone();
    wrong_domain[0].seed = register_seed(SeedDomain::Validation, first.family, first.group_id);
    assert_eq!(
        consistent_envelope(&config, wrong_domain),
        invalid("seed_domain_split_mismatch")
    );

    // Identity strings are recomputed: swapping in another population's
    // identity is detected even when the admitted list is untouched.
    let other =
        stage_c_preflight_admitted_cases(DEV, StageCPreflightBudget::bounded(2, 6)).unwrap();
    let mut envelope = consistent_envelope(&config, admitted).unwrap();
    let original = envelope.clone();
    envelope.seed_identity = stage_c_seed_identity(DEV, &other);
    assert_eq!(
        validate_provenance_envelope(&envelope),
        invalid("seed_identity_drift")
    );
    envelope = original.clone();
    envelope.data_identity = stage_c_data_identity_bundle(DEV, &other);
    assert_eq!(
        validate_provenance_envelope(&envelope),
        invalid("data_identity_drift")
    );
    envelope = original;
    envelope.admitted_cases = other;
    assert_eq!(
        validate_provenance_envelope(&envelope),
        invalid("data_identity_drift")
    );
}

#[test]
fn complete_config_is_bound_not_only_contract_labels() {
    let config = EvaluatorConfig::v6(DEV).unwrap();
    let identity = stage_c_config_identity_bundle(&config);
    for field in [
        "arm=",
        "split=development",
        "max_cases=64",
        "max_readout_scalars_per_case=",
        "updates=0",
        "primary=",
        "secondaries=[",
        "registry_non_final=true",
    ] {
        assert!(identity.contains(field), "missing {field} in {identity}");
    }
    let admitted =
        stage_c_preflight_admitted_cases(DEV, StageCPreflightBudget::bounded(1, 0)).unwrap();
    let envelope = consistent_envelope(&config, admitted.clone()).unwrap();

    let mut fewer_scalars = config;
    fewer_scalars.budget.max_readout_scalars_per_case -= 1;
    assert_ne!(stage_c_config_identity_bundle(&fewer_scalars), identity);
    assert_eq!(
        validate_provenance_binding(&envelope, &fewer_scalars),
        invalid("config_identity_mismatch")
    );

    let mut smaller = config;
    smaller.budget.max_cases = 32;
    assert_eq!(
        EvaluatorRun::open_with_provenance(smaller, envelope.clone()),
        invalid("config_identity_mismatch")
    );

    let mut reordered = config;
    let secondaries = reordered.metric_registry.secondaries;
    if secondaries.len() > 1 {
        let reversed: Vec<_> = secondaries.iter().rev().copied().collect();
        reordered.metric_registry.secondaries = Box::leak(reversed.into_boxed_slice());
        assert_ne!(stage_c_config_identity_bundle(&reordered), identity);
    }

    // A population larger than the run budget cannot be admitted.
    let mut tight = config;
    tight.budget.max_cases = (admitted.len() - 1) as u64;
    assert_eq!(
        consistent_envelope(&tight, admitted),
        invalid("population_exceeds_budget")
    );
}

#[test]
fn code_and_toolchain_identity_are_actual_not_declared() {
    let digest = stage_c_source_digest();
    assert_eq!(digest.len(), 16);
    assert!(digest.bytes().all(|byte| byte.is_ascii_hexdigit()));
    assert_eq!(digest, stage_c_source_digest());
    let code = stage_c_code_identity_bundle(EvalArm::V6);
    assert!(code.starts_with(PROVENANCE_SOURCE_DIGEST_CONTRACT));
    assert!(code.contains(&digest));

    assert!(BUILD_RUSTC_VERSION.starts_with("rustc "));
    assert!(BUILD_RUSTC_VERSION.contains(build_rustc_release()));
    let config = EvaluatorConfig::v6(DEV).unwrap();
    let admitted =
        stage_c_preflight_admitted_cases(DEV, StageCPreflightBudget::bounded(1, 0)).unwrap();
    let mut envelope = consistent_envelope(&config, admitted).unwrap();
    assert_eq!(envelope.compiler_identity, BUILD_RUSTC_VERSION);
    assert_eq!(
        envelope.toolchain_identity(),
        format!(
            "{}/{}",
            build_rustc_release(),
            PROVENANCE_TOOLCHAIN_FEATURES
        )
    );

    let honest = envelope.clone();
    envelope.code_identity = code.replace(&digest, "0123456789abcdef");
    assert_eq!(
        validate_provenance_envelope(&envelope),
        invalid("code_identity_drift")
    );
    envelope = honest;
    envelope.toolchain_channel = "0.0.1";
    assert_eq!(
        validate_provenance_envelope(&envelope),
        invalid("toolchain_drift")
    );
    envelope.toolchain_channel = build_rustc_release();
    envelope.compiler_identity = "unavailable";
    assert_eq!(
        validate_provenance_envelope(&envelope),
        invalid("compiler_identity_unavailable")
    );
}

#[test]
fn preflight_report_binds_population_config_and_seeds() {
    let budget = StageCPreflightBudget::bounded(3, 1);
    let report = run_stage_c_preflight(DEV, budget).unwrap();
    validate_stage_c_preflight_report(&report).unwrap();
    let admitted = stage_c_preflight_admitted_cases(DEV, budget).unwrap();
    assert_eq!(report.v6_provenance.admitted_cases, admitted);
    assert_eq!(report.c6_provenance.admitted_cases, admitted);
    assert_eq!(admitted.len() as u64, report.cases_per_arm);
    for (record, case) in report.v6_records.iter().zip(&admitted) {
        assert_eq!((record.family, record.case_id), (case.family, case.case_id));
        assert_eq!(record.canonical_digest, case.canonical_digest);
    }
    assert!(
        report
            .v6_provenance
            .config_identity
            .contains(&format!("max_cases={}", report.cases_per_arm))
    );

    // Swap one arm's population for another budget's population.
    let mut swapped = report.clone();
    let other_budget = StageCPreflightBudget::bounded(3, 2);
    let other = stage_c_preflight_admitted_cases(DEV, other_budget).unwrap();
    swapped.c6_provenance =
        consistent_envelope(&swapped_config(report.cases_per_arm), other).unwrap();
    assert!(validate_stage_c_preflight_report(&swapped).is_err());

    // Envelope bound to a different configuration is rejected.
    let mut drifted = report.clone();
    let wide = EvaluatorConfig::v6(DEV).unwrap();
    drifted.v6_provenance = consistent_envelope(&wide, admitted).unwrap();
    assert_eq!(
        validate_stage_c_preflight_report(&drifted),
        Err(EvalError::StageCPreflightInvalid {
            reason: "provenance_config_mismatch",
        })
    );
}

fn swapped_config(cases: u64) -> EvaluatorConfig {
    let mut config = EvaluatorConfig::c6(DEV).unwrap();
    config.budget.max_cases = cases;
    config
}
