//! TDI-24 Phase-C matched evaluation machinery.
//!
//! Slice 21 lands the shared evaluator envelope and the V6 arm: a deterministic
//! non-final Development/Validation path that scores through the matched vector
//! reference, never observes sealed targets inside the inference callback, and
//! refuses protected/final population labels. Slice 22 adds the matched C6 arm
//! under the same envelope and readout budget. Slice 23 adds an explicit
//! trainable-capacity / parameter-count matcher that accepts only matched V6/C6
//! configurations and rejects unmatched capacity fail-closed. Slice 24 adds a
//! paired deterministic initialization-policy matcher so V6/C6 share one seed
//! stream and kind, rejecting unpaired draws fail-closed. Slice 25 adds an
//! optimizer/update-budget matcher requiring identical examples, ordering,
//! steps and stopping rule across paired arms. Slice 26 freezes the versioned
//! metric registry: one primary quality metric plus an ordered closed set of
//! secondary diagnostics for Stage-C, experimental and non-final only. Slice 27
//! adds the paired uncertainty engine: confidence intervals and effect
//! summaries over already-revealed V6/C6 match outcomes without accepting raw
//! protected labels. Slice 28 introduces an explicit retained failure taxonomy:
//! typed Invalid/Numerical/Resource/Task failures classified from EvalError and
//! kept in a FailureRecord ledger rather than dropped. Slice 29 adds the
//! provenance envelope: code/config/data/seed/toolchain identity retained per
//! non-final run, validated fail-closed without inventing freeze pins. Slice 30
//! closes Phase C with a bounded Stage-C preflight smoke campaign: slices 21–29
//! run end-to-end on synthetic Development/Validation cases under a capped
//! budget, every outcome is retained, and no protected/final path exists. Slice 31
//! opens Phase D with the `gamma=0` ablation: the C6 reference weights with only
//! the parity-odd coefficient zeroed (alpha/beta fixed, matched capacity), run on
//! the same bounded Stage-C case stream, with exact per-case identities showing
//! the parity-odd channel contributes nothing and enantiomorphic scores coincide.
//! Slice 32 adds the `beta=0` ablation: only the mirror-even coefficient is
//! zeroed (alpha/gamma fixed, matched capacity) on the same case stream. The
//! matched C6 reference already carries `beta=0`, so the ablation is recorded
//! explicitly as the identity (bit-for-bit equal scores) while the per-case
//! mirror-even observable is retained and the parity-odd channel stays intact.
//! Slice 33 adds the direct-only collapse: both extra C6 channels are zeroed
//! (`beta=gamma=0`, alpha and capacity fixed) on the same case stream and the
//! collapsed C6 score is checked bit-for-bit against the matched V6 score.
//! Because the reference already has `beta=0`, the collapse shares its weights
//! with the slice-31 `gamma=0` ablation; that coincidence is recorded explicitly.
//! Slice 34 adds the parity-shuffle control: the carrier slots of query and key
//! are relabelled by one fixed permutation drawn from the reused Slice-18
//! registered seed, rejecting draws that keep or swap the parity sectors as
//! blocks. Weights and capacity are unchanged and the direct-product multiset is
//! preserved bit-for-bit, while `P^T M P != ±M` and `P^T J P != ±J` show the
//! `H+`/`H-` structure is destroyed reproducibly.
//! Slice 35 adds the fixed-M sensitivity study: under one frozen rule, every
//! choice of three carrier slots as the parity-even sector (`C(6,3) = 20`
//! fixed mirror bases, enumerated in lexicographic order, canonical first) is
//! scored with the unchanged C6 reference weights. Each basis is checked to
//! satisfy the exact chiral algebra (`M'^2 = I`, `J'^T = -J'`, `J'^2 = -I`,
//! `M' J' M' = -J'`); the canonical basis must reproduce the Stage-C C6
//! evaluator bit-for-bit and complementary bases must give exactly opposite
//! parity-odd observables. No basis is selected or tuned.
//! Slice 36 adds a structure-preserving learned basis prototype: an orthogonal
//! parameterisation `O(theta)` (product of the 15 Givens rotations of the
//! carrier) whose effective `O^T M O`, `O^T J O` keep the chiral algebra by
//! construction; non-orthogonal transforms are rejected. Four declared probes
//! (identity, gauge `diag(R, R)`, sector mixing, generic) are scored with the
//! unchanged C6 weights; the identity reproduces Stage-C C6 bit-for-bit and
//! the gauge probe is score-invariant within a declared tolerance. No angle is
//! trained or selected.
//! Slice 37 adds the head-sharing ablation: a two-head C6 score (mean of head
//! scores) with one chiral structure shared by both heads versus a per-head
//! structure (head `h` uses the slice-35 fixed mirror basis of rank `h`), at
//! identical weights and zero trainable capacity on both arms. The shared arm
//! reproduces the single-head Stage-C C6 score bit-for-bit.
//! Slice 38 adds width scaling: C6 and its direct-only matched arm are scored
//! at every preregistered matched carrier width `2n`, `n = 1..=3` (the
//! carrier restricted to the first `n` slots of each sector), with identical
//! weights and zero-trainable capacity; full width reproduces Stage-C C6
//! bit-for-bit and no width is selected.
//! Slice 39 adds sequence-length scaling: non-overlapping windows of
//! consecutive same-family cases at preregistered lengths `[2, 4, 8]` form
//! attention rows through the shared masked-softmax reference under both mask
//! policies, for C6 and its direct-only arm, with exact bounded cost
//! accounting (score evaluations, normalizer calls, masked entries).
//! Slice 40 adds the Stage-D attribution audit: every Phase-C/D slice (30 to
//! 39) is regenerated and validated, and the only admissible claim class is
//! software semantics on Development/Validation; scientific attribution stays
//! inadmissible. Slice 41 (Phase E) adds multi-seed replication: the
//! Stage-C preflight on four frozen, disjoint seed blocks with per-block
//! paired summaries, pooled counts and descriptive sign tallies. Slice 42
//! adds input-noise robustness: declared deterministic perturbation families
//! (isotropic, even-only, odd-only) at declared amplitudes, applied
//! identically to both paired arms, with label-free decision stability.
//! Issue #690 makes the Stage-C provenance authoritative (envelope v2): no
//! case can be scored without a validated envelope; the envelope binds the
//! complete canonical [`EvaluatorConfig`], the exact admitted population
//! (per-case family, `group_id`, canonical digest and registered seed), a
//! source-content digest of the TDI-24 evaluator surface, and the version of
//! the compiler that actually built this crate.
//! No training,
//! confirmatory execution, protected/final evaluation, or scientific claim is
//! authorised here.

use core::fmt;

use super::tdi24_accounting::ScoreArm;
use super::tdi24_attention::{
    MASKING_CONTRACT, MaskPolicy, NORMALIZER_CONTRACT, normalize_with_policy,
};
use super::tdi24_chiral::{
    CHIRAL_CONTRACT, CHIRAL_WIDTH, Chiral6, ChiralError, ChiralObservables, ChiralScoreWeights,
    chiral_score, enantiomorphic_scores, observables,
};
use super::tdi24_tasks::{
    DATASET_CANONICALIZATION_CONTRACT, DataSplit, DirectionTarget, HandednessTarget, InferenceView,
    LabeledCase, NonChiralTarget, PROTECTED_LABEL_CONTRACT, ReflectionInvariantTarget,
    RegisteredSeed, SEED_REGISTRY_CONTRACT, SPLIT_MANIFEST_CONTRACT, SeedDomain, TaskFamily,
    canonicalize_inference_view, direction_reversal_pair_in_split,
    non_chiral_control_case_in_split, reflection_discriminative_pair_in_split,
    reflection_nuisance_pair_in_split, register_seed, run_inference_callback,
    seal_direction_reversal, seal_non_chiral_control, seal_reflection_discriminative,
    seal_reflection_nuisance,
};
use super::tdi24_vector::{VECTOR6_CONTRACT, VECTOR6_WIDTH, Vector6, Vector6Error, vector6_score};

/// Shared evaluator envelope consumed by V6 now and by later matched arms.
pub const EVALUATOR_ENVELOPE_CONTRACT: &str = "tdi24-evaluator-envelope-v1";

/// Versioned V6 evaluator contract.
pub const V6_EVALUATOR_CONTRACT: &str = "tdi24-v6-evaluator-v1";

/// Versioned C6 evaluator contract.
pub const C6_EVALUATOR_CONTRACT: &str = "tdi24-c6-evaluator-v1";

/// Matched readout-budget contract shared across Phase-C evaluator arms.
pub const READOUT_BUDGET_CONTRACT: &str = "tdi24-readout-budget-v1";

/// Versioned trainable-capacity / parameter-count matcher contract.
pub const PARAMETER_COUNT_MATCHER_CONTRACT: &str = "tdi24-parameter-count-matcher-v1";

/// Versioned paired deterministic initialization-policy matcher contract.
pub const INITIALIZATION_MATCHER_CONTRACT: &str = "tdi24-initialization-matcher-v1";

/// Versioned optimizer/update-budget matcher contract.
pub const OPTIMIZER_UPDATE_BUDGET_CONTRACT: &str = "tdi24-optimizer-update-budget-v1";

/// Versioned Stage-C metric-registry contract.
pub const METRIC_REGISTRY_CONTRACT: &str = "tdi24-metric-registry-v1";

/// Primary metric id: paired task accuracy / score-match rate vs oracle.
pub const PRIMARY_METRIC_PAIRED_TASK_ACCURACY: &str = "paired_task_accuracy";

/// Secondary diagnostic: paired task outcome difference (V6 vs C6).
pub const SECONDARY_PAIRED_OUTCOME_DIFFERENCE: &str = "paired_outcome_difference";

/// Secondary diagnostic: mirror-swap identity error.
pub const SECONDARY_MIRROR_SWAP_IDENTITY_ERROR: &str = "mirror_swap_identity_error";

/// Secondary diagnostic: parity-equivariance / invariance error.
pub const SECONDARY_PARITY_EQUIVARIANCE_INVARIANCE_ERROR: &str =
    "parity_equivariance_invariance_error";

/// Secondary diagnostic: calibration / confidence error when scores are exposed.
pub const SECONDARY_CALIBRATION_CONFIDENCE_ERROR: &str = "calibration_confidence_error";

/// Secondary diagnostic: gradient / stability for trained arms.
pub const SECONDARY_GRADIENT_STABILITY: &str = "gradient_stability";

/// Secondary diagnostic: op-count / memory / latency / throughput under a qualified env only.
pub const SECONDARY_OP_COUNT_MEMORY_LATENCY_THROUGHPUT: &str = "op_count_memory_latency_throughput";

/// Maximum secondary diagnostics admitted in one frozen registry.
pub const MAX_SECONDARY_DIAGNOSTICS: usize = 6;

/// Versioned paired uncertainty engine contract.
pub const PAIRED_UNCERTAINTY_CONTRACT: &str = "tdi24-paired-uncertainty-v1";

/// Versioned retained failure-taxonomy contract.
pub const FAILURE_TAXONOMY_CONTRACT: &str = "tdi24-failure-taxonomy-v1";

/// Versioned provenance-envelope contract: code/config/data/seed/toolchain identity.
///
/// v2 (issue #690) binds the complete evaluator configuration, the exact
/// admitted population with per-case seeds, a source-content digest and the
/// actual build compiler. Envelopes under the superseded
/// [`LEGACY_PROVENANCE_ENVELOPE_CONTRACT`] are rejected as `contract_drift`.
pub const PROVENANCE_ENVELOPE_CONTRACT: &str = "tdi24-provenance-envelope-v2";

/// Superseded slice-29 envelope contract (`tdi24-provenance-envelope-v1`).
///
/// Retained for lineage only: it carried semantic contract labels instead of
/// exact code/config/data/seed/compiler identity and is never admitted.
pub const LEGACY_PROVENANCE_ENVELOPE_CONTRACT: &str = "tdi24-provenance-envelope-v1";

/// Canonical evaluator-configuration identity contract bound by envelope v2.
pub const PROVENANCE_CONFIG_IDENTITY_CONTRACT: &str = "tdi24-evaluator-config-identity-v1";

/// Source-content digest contract for the code identity bound by envelope v2.
///
/// FNV-1a 64 over the ordered `(path, byte length, bytes)` of every TDI-24
/// evaluator source file compiled into this crate. Non-cryptographic: it
/// identifies the exact source text, it does not authenticate it.
pub const PROVENANCE_SOURCE_DIGEST_CONTRACT: &str = "tdi24-source-digest-fnv1a64-v1";

/// Admitted-population identity contract bound by envelope v2.
pub const PROVENANCE_POPULATION_CONTRACT: &str = "tdi24-admitted-population-v1";

/// Per-case seed identity contract bound by envelope v2.
pub const PROVENANCE_CASE_SEED_CONTRACT: &str = "tdi24-case-seed-binding-v1";

/// `rustc --version` of the compiler that actually built this crate.
///
/// Captured by `tdi-ai/build.rs` from Cargo's `RUSTC`; `unavailable` when the
/// compiler could not be queried, which makes every envelope fail closed.
pub const BUILD_RUSTC_VERSION: &str = env!("TDI_AI_BUILD_RUSTC_VERSION");

/// Ordered TDI-24 evaluator source surface bound into the code identity.
const PROVENANCE_SOURCE_FILES: [(&str, &str); 6] = [
    (
        "tdi-ai/src/tdi24_accounting.rs",
        include_str!("tdi24_accounting.rs"),
    ),
    (
        "tdi-ai/src/tdi24_attention.rs",
        include_str!("tdi24_attention.rs"),
    ),
    (
        "tdi-ai/src/tdi24_chiral.rs",
        include_str!("tdi24_chiral.rs"),
    ),
    ("tdi-ai/src/tdi24_eval.rs", include_str!("tdi24_eval.rs")),
    ("tdi-ai/src/tdi24_tasks.rs", include_str!("tdi24_tasks.rs")),
    (
        "tdi-ai/src/tdi24_vector.rs",
        include_str!("tdi24_vector.rs"),
    ),
];

/// Versioned Stage-C bounded preflight contract (slice 30).
pub const STAGE_C_PREFLIGHT_CONTRACT: &str = "tdi24-stage-c-preflight-v1";

/// Phase-D `gamma=0` ablation contract pin (slice 31).
pub const GAMMA_ZERO_ABLATION_CONTRACT: &str = "tdi24-gamma-zero-ablation-v1";

/// Phase-D `beta=0` ablation contract pin (slice 32).
pub const BETA_ZERO_ABLATION_CONTRACT: &str = "tdi24-beta-zero-ablation-v1";

/// Phase-D direct-only collapse contract pin (slice 33).
pub const DIRECT_ONLY_COLLAPSE_CONTRACT: &str = "tdi24-direct-only-collapse-v1";

/// Declared absolute tolerance of the collapsed-C6-vs-V6 score match (slice 33).
///
/// Exactly zero: the collapsed C6 path and V6 share the identical checked dot
/// accumulation, so the match is required bit-for-bit. Not a freeze pin.
pub const DIRECT_ONLY_V6_MATCH_TOLERANCE: f64 = 0.0;

/// Phase-D parity-shuffle control contract pin (slice 34).
pub const PARITY_SHUFFLE_CONTROL_CONTRACT: &str = "tdi24-parity-shuffle-control-v1";

/// Upper bound on deterministic rejection draws for the parity shuffle.
///
/// A software termination bound only (about 90% of uniform draws already mix
/// the sectors); not a freeze pin and not a tuned parameter.
pub const MAX_PARITY_SHUFFLE_DRAWS: u32 = 64;

/// Phase-D fixed-M sensitivity contract pin (slice 35).
pub const FIXED_M_SENSITIVITY_CONTRACT: &str = "tdi24-fixed-m-sensitivity-v1";

/// Number of fixed mirror bases under the frozen rule: every choice of three
/// of the six carrier slots as the parity-even sector, `C(6,3) = 20`.
///
/// Derived from the carrier width, not a tuned parameter and not a freeze pin.
pub const FIXED_MIRROR_BASIS_COUNT: usize = 20;

/// Matched C6 reference score weights (alpha, beta, gamma) used since slice 22.
pub const C6_REFERENCE_WEIGHTS: ChiralScoreWeights = ChiralScoreWeights {
    alpha: 1.0,
    beta: 0.0,
    gamma: 1.0,
};

/// Declared CI rustc release for the TDI-24 gates (`1.97.1`).
///
/// Since issue #690 this is a declaration only: envelopes record the release
/// of the compiler that actually built the evaluator ([`build_rustc_release`]),
/// never this constant. Does **not** invent a configuration-freeze pin and does
/// not authorise protected/final execution.
pub const PROVENANCE_TOOLCHAIN_CHANNEL: &str = "1.97.1";

/// Declared Cargo feature set retained in the provenance toolchain identity.
pub const PROVENANCE_TOOLCHAIN_FEATURES: &str = "experimental";

/// Declared CI toolchain identity token: `{channel}/{features}`.
///
/// Equal to an envelope's [`ProvenanceEnvelope::toolchain_identity`] only when
/// the evaluator was actually built by the declared CI release.
pub const PROVENANCE_TOOLCHAIN_ID: &str = "1.97.1/experimental";

/// Maximum UTF-8 byte length admitted for code/config/data/seed identity strings.
pub const MAX_PROVENANCE_IDENTITY_BYTES: usize = 1024;

/// Nominal two-sided confidence level for Stage-C paired uncertainty summaries.
pub const PAIRED_UNCERTAINTY_LEVEL: f64 = 0.95;

/// Φ^{-1}(0.975) critical value for two-sided 95% Wilson intervals.
/// Deterministic constant; no RNG and no runtime table lookup.
pub const NORMAL_CRITICAL_Z_95: f64 = 1.959_963_984_540_054;

/// ln(2 / 0.05), used by deterministic two-sided 95% Hoeffding bounds.
/// Kept as a literal so the uncertainty surface does not depend on runtime logs.
pub const HOEFFDING_LOG_40: f64 = 3.688_879_454_113_936_3;

/// Maximum cases admitted to one non-final evaluator run.
pub const MAX_CASES_PER_RUN: u64 = 64;

/// Maximum typed failures retained in one non-final failure ledger.
pub const MAX_FAILURES_PER_RUN: u64 = MAX_CASES_PER_RUN;

/// Maximum scalar fields retained per case readout (score + correctness flag).
pub const MAX_READOUT_SCALARS_PER_CASE: u64 = 2;

/// Non-trained update budget for the Slice-21 reference path.
pub const NON_TRAINED_UPDATE_BUDGET: u64 = 0;

/// Evaluation arm identity for Phase C.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EvalArm {
    /// Matched six-dimensional vector control.
    V6,
    /// Matched six-dimensional chiral candidate.
    C6,
}

impl EvalArm {
    /// Stable lowercase arm label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::V6 => "v6",
            Self::C6 => "c6",
        }
    }

    /// Arm-specific evaluator contract pin.
    #[must_use]
    pub const fn evaluator_contract(self) -> &'static str {
        match self {
            Self::V6 => V6_EVALUATOR_CONTRACT,
            Self::C6 => C6_EVALUATOR_CONTRACT,
        }
    }

    /// Accounting arm used for operation/storage pairing.
    #[must_use]
    pub const fn score_arm(self) -> ScoreArm {
        match self {
            Self::V6 => ScoreArm::V6,
            Self::C6 => ScoreArm::C6,
        }
    }
}

/// Fixed matched readout budget shared by V6 and later C6.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReadoutBudget {
    /// Maximum cases admitted to one run.
    pub max_cases: u64,
    /// Maximum scalar fields retained per case.
    pub max_readout_scalars_per_case: u64,
    /// Parameter-update steps; zero on the non-trained reference path.
    pub updates: u64,
    /// Budget contract pin.
    pub contract: &'static str,
}

impl ReadoutBudget {
    /// Canonical matched non-trained budget.
    #[must_use]
    pub const fn matched_non_trained() -> Self {
        Self {
            max_cases: MAX_CASES_PER_RUN,
            max_readout_scalars_per_case: MAX_READOUT_SCALARS_PER_CASE,
            updates: NON_TRAINED_UPDATE_BUDGET,
            contract: READOUT_BUDGET_CONTRACT,
        }
    }
}

/// Immutable configuration for one non-final evaluator run.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EvaluatorConfig {
    /// Development or Validation only.
    pub split: DataSplit,
    /// Arm under evaluation.
    pub arm: EvalArm,
    /// Matched readout budget.
    pub budget: ReadoutBudget,
    /// Shared envelope contract pin.
    pub envelope_contract: &'static str,
    /// Validated, exact Stage-C metric registry.
    pub metric_registry: MetricRegistry,
}

impl EvaluatorConfig {
    /// Construct a fail-closed V6 Development/Validation configuration.
    pub fn v6(split: DataSplit) -> Result<Self, EvalError> {
        validate_non_final_split(split)?;
        Ok(Self {
            split,
            arm: EvalArm::V6,
            budget: ReadoutBudget::matched_non_trained(),
            envelope_contract: EVALUATOR_ENVELOPE_CONTRACT,
            metric_registry: MetricRegistry::pinned(),
        })
    }

    /// Construct a fail-closed C6 Development/Validation configuration.
    pub fn c6(split: DataSplit) -> Result<Self, EvalError> {
        validate_non_final_split(split)?;
        Ok(Self {
            split,
            arm: EvalArm::C6,
            budget: ReadoutBudget::matched_non_trained(),
            envelope_contract: EVALUATOR_ENVELOPE_CONTRACT,
            metric_registry: MetricRegistry::pinned(),
        })
    }
}

/// Retained failure category for one evaluated case.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EvalFailure {
    /// Arm arithmetic rejected a non-finite intermediate.
    Numerical,
    /// Task/oracle mapping could not be applied.
    Task,
    /// Readout or case budget exhausted.
    Resource,
    /// Contract or split identity rejected.
    Contract,
}

impl EvalFailure {
    /// Stable lowercase failure token.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Numerical => "numerical",
            Self::Task => "task",
            Self::Resource => "resource",
            Self::Contract => "contract",
        }
    }
}

/// Outcome retained for one evaluated case.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EvalOutcome {
    /// Finite arm score plus sealed-label correctness on the evaluation path.
    Scored { score: f64, correct: bool },
    /// Typed failure retained instead of dropping the case.
    Failure(EvalFailure),
}

/// One immutable non-final evaluation record.
#[derive(Clone, Debug, PartialEq)]
pub struct EvalRecord {
    pub arm: EvalArm,
    pub split: DataSplit,
    pub family: TaskFamily,
    pub case_id: u64,
    pub group_id: u64,
    pub outcome: EvalOutcome,
    pub canonical_digest: String,
    pub envelope_contract: &'static str,
    pub arm_contract: &'static str,
    pub budget_contract: &'static str,
    pub metric_registry_contract: &'static str,
    pub vector_contract: &'static str,
    pub label_contract: &'static str,
}

/// Accumulator that enforces the matched readout budget.
#[derive(Clone, Debug, PartialEq)]
pub struct EvaluatorRun {
    config: EvaluatorConfig,
    records: Vec<EvalRecord>,
    /// Optional validated provenance envelope bound at open (Slice 29).
    provenance: Option<ProvenanceEnvelope>,
}

impl EvaluatorRun {
    /// Open a bounded non-final run without a bound provenance envelope.
    ///
    /// Such a run validates its configuration but **cannot evaluate**: every
    /// `evaluate_*` call fails closed with `provenance_required` (issue #690).
    /// Use [`Self::open_with_provenance`] to score cases.
    pub fn open(config: EvaluatorConfig) -> Result<Self, EvalError> {
        Self::open_inner(config, None)
    }

    /// Open a bounded non-final run bound to a validated provenance envelope.
    ///
    /// Fail-closed when the envelope drifts from the run arm/split, from the
    /// complete canonical configuration, or fails
    /// [`validate_provenance_envelope`]. Does not execute protected/final data.
    pub fn open_with_provenance(
        config: EvaluatorConfig,
        envelope: ProvenanceEnvelope,
    ) -> Result<Self, EvalError> {
        validate_provenance_binding(&envelope, &config)?;
        Self::open_inner(config, Some(envelope))
    }

    /// Admit one case before scoring: a bound envelope must exist, list this
    /// exact case (family, `case_id`, `group_id`, canonical digest) and bind it
    /// to its registered per-case seed; each admitted case is scored once.
    fn admit_case(&self, view: &InferenceView) -> Result<(), EvalError> {
        let Some(envelope) = self.provenance.as_ref() else {
            return Err(EvalError::ProvenanceEnvelopeInvalid {
                reason: "provenance_required",
            });
        };
        admit_view_against_population(&envelope.admitted_cases, view)?;
        if self
            .records
            .iter()
            .any(|record| record.family == view.family && record.case_id == view.case_id)
        {
            return Err(EvalError::ProvenanceEnvelopeInvalid {
                reason: "case_already_evaluated",
            });
        }
        Ok(())
    }

    fn open_inner(
        config: EvaluatorConfig,
        provenance: Option<ProvenanceEnvelope>,
    ) -> Result<Self, EvalError> {
        config.metric_registry.admit_split(config.split)?;
        if config.envelope_contract != EVALUATOR_ENVELOPE_CONTRACT {
            return Err(EvalError::ContractMismatch {
                field: "envelope_contract",
            });
        }
        if config.budget.contract != READOUT_BUDGET_CONTRACT {
            return Err(EvalError::ContractMismatch {
                field: "budget_contract",
            });
        }
        if config.budget.max_cases == 0
            || config.budget.max_cases > MAX_CASES_PER_RUN
            || config.budget.max_readout_scalars_per_case == 0
            || config.budget.max_readout_scalars_per_case > MAX_READOUT_SCALARS_PER_CASE
        {
            return Err(EvalError::InvalidBudget);
        }
        Ok(Self {
            config,
            records: Vec::new(),
            provenance,
        })
    }

    /// Borrow the immutable run configuration.
    #[must_use]
    pub const fn config(&self) -> &EvaluatorConfig {
        &self.config
    }

    /// Borrow the bound provenance envelope when present.
    #[must_use]
    pub const fn provenance(&self) -> Option<&ProvenanceEnvelope> {
        self.provenance.as_ref()
    }

    /// Borrow emitted records in admission order.
    #[must_use]
    pub fn records(&self) -> &[EvalRecord] {
        &self.records
    }

    /// Evaluate one sealed labeled case through the V6 path.
    pub fn evaluate_v6_binary<T, S>(
        &mut self,
        case: &LabeledCase<T>,
        oracle_sign: S,
    ) -> Result<&EvalRecord, EvalError>
    where
        S: FnOnce(&T) -> Result<i8, EvalError>,
    {
        if self.records.len() as u64 >= self.config.budget.max_cases {
            return Err(EvalError::CaseBudgetExceeded);
        }

        let view = case.inference_view();
        if view.split != self.config.split {
            return Err(EvalError::SplitMismatch {
                expected: self.config.split,
                actual: view.split,
            });
        }
        self.admit_case(view)?;

        // Inference callback sees only the view; sealed target stays outside.
        let scored = run_inference_callback(case, score_v6_from_view);

        let outcome = match scored {
            Ok(score) => {
                if self.config.budget.max_readout_scalars_per_case < 2 {
                    EvalOutcome::Failure(EvalFailure::Resource)
                } else {
                    match oracle_sign(case.protected_label().reveal_for_evaluation()) {
                        Ok(sign) if sign == 1 || sign == -1 => {
                            let correct = score * f64::from(sign) > 0.0;
                            EvalOutcome::Scored { score, correct }
                        }
                        Ok(_) => EvalOutcome::Failure(EvalFailure::Task),
                        Err(_) => EvalOutcome::Failure(EvalFailure::Task),
                    }
                }
            }
            Err(EvalError::Numerical(_)) => EvalOutcome::Failure(EvalFailure::Numerical),
            Err(EvalError::ContractMismatch { .. }) => EvalOutcome::Failure(EvalFailure::Contract),
            Err(_) => EvalOutcome::Failure(EvalFailure::Task),
        };

        let digest = canonicalize_inference_view(view).digest;
        self.records.push(EvalRecord {
            arm: EvalArm::V6,
            split: view.split,
            family: view.family,
            case_id: view.case_id,
            group_id: view.group_id,
            outcome,
            canonical_digest: digest,
            envelope_contract: EVALUATOR_ENVELOPE_CONTRACT,
            arm_contract: V6_EVALUATOR_CONTRACT,
            budget_contract: READOUT_BUDGET_CONTRACT,
            metric_registry_contract: self.config.metric_registry.registry_contract,
            vector_contract: VECTOR6_CONTRACT,
            label_contract: view.label_contract,
        });
        Ok(self.records.last().expect("just pushed"))
    }

    /// Evaluate one sealed labeled case through the C6 path.
    pub fn evaluate_c6_binary<T, S>(
        &mut self,
        case: &LabeledCase<T>,
        oracle_sign: S,
    ) -> Result<&EvalRecord, EvalError>
    where
        S: FnOnce(&T) -> Result<i8, EvalError>,
    {
        if self.config.arm != EvalArm::C6 {
            return Err(EvalError::UnsupportedArm);
        }
        if self.records.len() as u64 >= self.config.budget.max_cases {
            return Err(EvalError::CaseBudgetExceeded);
        }
        let view = case.inference_view();
        if view.split != self.config.split {
            return Err(EvalError::SplitMismatch {
                expected: self.config.split,
                actual: view.split,
            });
        }
        self.admit_case(view)?;
        let scored = run_inference_callback(case, score_c6_from_view);
        let outcome = match scored {
            Ok(score) if self.config.budget.max_readout_scalars_per_case >= 2 => {
                match oracle_sign(case.protected_label().reveal_for_evaluation()) {
                    Ok(sign) if sign == 1 || sign == -1 => EvalOutcome::Scored {
                        score,
                        correct: score * f64::from(sign) > 0.0,
                    },
                    _ => EvalOutcome::Failure(EvalFailure::Task),
                }
            }
            Ok(_) => EvalOutcome::Failure(EvalFailure::Resource),
            Err(EvalError::ChiralNumerical(_)) => EvalOutcome::Failure(EvalFailure::Numerical),
            Err(EvalError::ContractMismatch { .. }) => EvalOutcome::Failure(EvalFailure::Contract),
            Err(_) => EvalOutcome::Failure(EvalFailure::Task),
        };
        let digest = canonicalize_inference_view(view).digest;
        self.records.push(EvalRecord {
            arm: EvalArm::C6,
            split: view.split,
            family: view.family,
            case_id: view.case_id,
            group_id: view.group_id,
            outcome,
            canonical_digest: digest,
            envelope_contract: EVALUATOR_ENVELOPE_CONTRACT,
            arm_contract: C6_EVALUATOR_CONTRACT,
            budget_contract: READOUT_BUDGET_CONTRACT,
            metric_registry_contract: self.config.metric_registry.registry_contract,
            vector_contract: CHIRAL_CONTRACT,
            label_contract: view.label_contract,
        });
        Ok(self.records.last().expect("just pushed"))
    }
}

/// Score an inference view with the matched C6 chiral reference.
pub fn score_c6_from_view(view: &InferenceView) -> Result<f64, EvalError> {
    let q = view.query.as_array();
    let k = view.key.as_array();
    let query = Chiral6::from_array(q).map_err(EvalError::ChiralNumerical)?;
    let key = Chiral6::from_array(k).map_err(EvalError::ChiralNumerical)?;
    let weights = ChiralScoreWeights::new(
        C6_REFERENCE_WEIGHTS.alpha,
        C6_REFERENCE_WEIGHTS.beta,
        C6_REFERENCE_WEIGHTS.gamma,
    )
    .map_err(EvalError::ChiralNumerical)?;
    chiral_score(query, key, weights).map_err(EvalError::ChiralNumerical)
}

/// `gamma=0` ablation of `reference`: only the parity-odd coefficient changes.
#[must_use]
pub const fn gamma_zero_weights(reference: ChiralScoreWeights) -> ChiralScoreWeights {
    ChiralScoreWeights {
        alpha: reference.alpha,
        beta: reference.beta,
        gamma: 0.0,
    }
}

/// Score an inference view with the `gamma=0` ablation of the C6 reference.
pub fn score_c6_gamma_zero_from_view(view: &InferenceView) -> Result<f64, EvalError> {
    let query = Chiral6::from_array(view.query.as_array()).map_err(EvalError::ChiralNumerical)?;
    let key = Chiral6::from_array(view.key.as_array()).map_err(EvalError::ChiralNumerical)?;
    chiral_score(query, key, gamma_zero_weights(C6_REFERENCE_WEIGHTS))
        .map_err(EvalError::ChiralNumerical)
}

/// `beta=0` ablation of `reference`: only the mirror-even coefficient changes.
#[must_use]
pub const fn beta_zero_weights(reference: ChiralScoreWeights) -> ChiralScoreWeights {
    ChiralScoreWeights {
        alpha: reference.alpha,
        beta: 0.0,
        gamma: reference.gamma,
    }
}

/// Score an inference view with the `beta=0` ablation of the C6 reference.
pub fn score_c6_beta_zero_from_view(view: &InferenceView) -> Result<f64, EvalError> {
    let query = Chiral6::from_array(view.query.as_array()).map_err(EvalError::ChiralNumerical)?;
    let key = Chiral6::from_array(view.key.as_array()).map_err(EvalError::ChiralNumerical)?;
    chiral_score(query, key, beta_zero_weights(C6_REFERENCE_WEIGHTS))
        .map_err(EvalError::ChiralNumerical)
}

/// Direct-only collapse of `reference`: both extra channels (mirror-even
/// `beta` and parity-odd `gamma`) are zeroed; only `alpha` is kept.
#[must_use]
pub const fn direct_only_weights(reference: ChiralScoreWeights) -> ChiralScoreWeights {
    ChiralScoreWeights {
        alpha: reference.alpha,
        beta: 0.0,
        gamma: 0.0,
    }
}

/// Score an inference view with the direct-only collapse of the C6 reference.
pub fn score_c6_direct_only_from_view(view: &InferenceView) -> Result<f64, EvalError> {
    let query = Chiral6::from_array(view.query.as_array()).map_err(EvalError::ChiralNumerical)?;
    let key = Chiral6::from_array(view.key.as_array()).map_err(EvalError::ChiralNumerical)?;
    chiral_score(query, key, direct_only_weights(C6_REFERENCE_WEIGHTS))
        .map_err(EvalError::ChiralNumerical)
}

/// Score an inference view with the matched V6 vector reference.
pub fn score_v6_from_view(view: &InferenceView) -> Result<f64, EvalError> {
    let query = Vector6::new(view.query.as_array()).map_err(EvalError::Numerical)?;
    let key = Vector6::new(view.key.as_array()).map_err(EvalError::Numerical)?;
    vector6_score(query, key).map_err(EvalError::Numerical)
}

/// Explicit trainable-capacity descriptor for one Phase-C arm.
///
/// The non-trained Slice-21/22 reference path declares zero trainable
/// parameters at matched carrier width. Later trained arms must publish an
/// exact count through this descriptor rather than silently compensating.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TrainableCapacity {
    /// Arm whose capacity is declared.
    pub arm: EvalArm,
    /// Exact trainable parameter count for this configuration.
    pub trainable_parameters: u64,
    /// Carrier width consumed by the arm.
    pub carrier_width: u64,
    /// Matcher contract pin.
    pub matcher_contract: &'static str,
}

impl TrainableCapacity {
    /// Canonical non-trained V6 reference capacity.
    #[must_use]
    pub const fn reference_v6() -> Self {
        Self {
            arm: EvalArm::V6,
            trainable_parameters: 0,
            carrier_width: VECTOR6_WIDTH as u64,
            matcher_contract: PARAMETER_COUNT_MATCHER_CONTRACT,
        }
    }

    /// Canonical non-trained C6 reference capacity.
    #[must_use]
    pub const fn reference_c6() -> Self {
        Self {
            arm: EvalArm::C6,
            trainable_parameters: 0,
            carrier_width: CHIRAL_WIDTH as u64,
            matcher_contract: PARAMETER_COUNT_MATCHER_CONTRACT,
        }
    }
}

/// Witness that two arm capacities are matched under the Slice-23 contract.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MatchedParameterCount {
    pub trainable_parameters: u64,
    pub carrier_width: u64,
    pub left_arm: EvalArm,
    pub right_arm: EvalArm,
    pub matcher_contract: &'static str,
}

/// Accept only matched trainable-capacity configurations; reject mismatches
/// fail-closed without silently compensating.
pub fn match_parameter_counts(
    left: TrainableCapacity,
    right: TrainableCapacity,
) -> Result<MatchedParameterCount, EvalError> {
    if left.matcher_contract != PARAMETER_COUNT_MATCHER_CONTRACT
        || right.matcher_contract != PARAMETER_COUNT_MATCHER_CONTRACT
    {
        return Err(EvalError::ContractMismatch {
            field: "parameter_count_matcher_contract",
        });
    }
    if left.trainable_parameters != right.trainable_parameters {
        return Err(EvalError::ParameterCountMismatch {
            left_arm: left.arm,
            right_arm: right.arm,
            left_parameters: left.trainable_parameters,
            right_parameters: right.trainable_parameters,
        });
    }
    if left.carrier_width != right.carrier_width {
        return Err(EvalError::ParameterCountMismatch {
            left_arm: left.arm,
            right_arm: right.arm,
            left_parameters: left.trainable_parameters,
            right_parameters: right.trainable_parameters,
        });
    }
    Ok(MatchedParameterCount {
        trainable_parameters: left.trainable_parameters,
        carrier_width: left.carrier_width,
        left_arm: left.arm,
        right_arm: right.arm,
        matcher_contract: PARAMETER_COUNT_MATCHER_CONTRACT,
    })
}

/// Deterministic initialization kind shared by paired Phase-C arms.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InitializationKind {
    /// Non-trained reference path: no parameter draw is performed.
    ReferenceNone,
    /// Deterministic paired draw from a shared seed stream (trained arms).
    DeterministicPaired,
}

impl InitializationKind {
    /// Stable lowercase label for manifests and audits.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ReferenceNone => "reference_none",
            Self::DeterministicPaired => "deterministic_paired",
        }
    }
}

/// Explicit initialization policy for one Phase-C arm.
///
/// Paired V6/C6 configurations must publish identical seed and kind through
/// this descriptor rather than silently using divergent draws.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InitializationPolicy {
    /// Arm whose initialization is declared.
    pub arm: EvalArm,
    /// Initialization kind for this configuration.
    pub kind: InitializationKind,
    /// Shared deterministic seed for the paired draw stream.
    pub seed: u64,
    /// Stream discriminator kept identical across paired arms.
    pub stream_id: u64,
    /// Matcher contract pin.
    pub matcher_contract: &'static str,
}

impl InitializationPolicy {
    /// Canonical non-trained V6 reference initialization (no parameter draw).
    #[must_use]
    pub const fn reference_v6(seed: u64) -> Self {
        Self {
            arm: EvalArm::V6,
            kind: InitializationKind::ReferenceNone,
            seed,
            stream_id: 0,
            matcher_contract: INITIALIZATION_MATCHER_CONTRACT,
        }
    }

    /// Canonical non-trained C6 reference initialization (no parameter draw).
    #[must_use]
    pub const fn reference_c6(seed: u64) -> Self {
        Self {
            arm: EvalArm::C6,
            kind: InitializationKind::ReferenceNone,
            seed,
            stream_id: 0,
            matcher_contract: INITIALIZATION_MATCHER_CONTRACT,
        }
    }
}

/// Witness that two arm initialization policies are matched under Slice 24.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MatchedInitialization {
    pub kind: InitializationKind,
    pub seed: u64,
    pub stream_id: u64,
    pub left_arm: EvalArm,
    pub right_arm: EvalArm,
    pub matcher_contract: &'static str,
}

/// Accept only paired deterministic initialization policies; reject unpaired
/// seeds/kinds fail-closed without silently compensating.
pub fn match_initialization(
    left: InitializationPolicy,
    right: InitializationPolicy,
) -> Result<MatchedInitialization, EvalError> {
    if left.matcher_contract != INITIALIZATION_MATCHER_CONTRACT
        || right.matcher_contract != INITIALIZATION_MATCHER_CONTRACT
    {
        return Err(EvalError::ContractMismatch {
            field: "initialization_matcher_contract",
        });
    }
    if left.kind != right.kind || left.seed != right.seed || left.stream_id != right.stream_id {
        return Err(EvalError::InitializationMismatch {
            left_arm: left.arm,
            right_arm: right.arm,
            left_seed: left.seed,
            right_seed: right.seed,
        });
    }
    Ok(MatchedInitialization {
        kind: left.kind,
        seed: left.seed,
        stream_id: left.stream_id,
        left_arm: left.arm,
        right_arm: right.arm,
        matcher_contract: INITIALIZATION_MATCHER_CONTRACT,
    })
}

/// Declared stopping rule for a Phase-C optimizer/update budget.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StoppingRule {
    /// Non-trained path: stop after the declared example budget (zero updates).
    ExhaustExamples,
    /// Trained path: stop after the declared fixed update count.
    FixedUpdates,
}

impl StoppingRule {
    /// Stable lowercase label for manifests and audits.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ExhaustExamples => "exhaust_examples",
            Self::FixedUpdates => "fixed_updates",
        }
    }
}

/// Explicit optimizer/update budget for one Phase-C arm.
///
/// Paired V6/C6 configurations must publish identical examples, ordering seed,
/// update steps and stopping rule rather than silently diverging.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OptimizerUpdateBudget {
    /// Arm whose budget is declared.
    pub arm: EvalArm,
    /// Number of examples admitted under this budget.
    pub examples: u64,
    /// Parameter-update steps (zero on the non-trained reference path).
    pub updates: u64,
    /// Shared deterministic seed fixing example ordering across paired arms.
    pub ordering_seed: u64,
    /// Stopping rule applied identically to both arms.
    pub stopping: StoppingRule,
    /// Matcher contract pin.
    pub matcher_contract: &'static str,
}

impl OptimizerUpdateBudget {
    /// Canonical non-trained V6 reference budget.
    #[must_use]
    pub const fn reference_v6(examples: u64, ordering_seed: u64) -> Self {
        Self {
            arm: EvalArm::V6,
            examples,
            updates: NON_TRAINED_UPDATE_BUDGET,
            ordering_seed,
            stopping: StoppingRule::ExhaustExamples,
            matcher_contract: OPTIMIZER_UPDATE_BUDGET_CONTRACT,
        }
    }

    /// Canonical non-trained C6 reference budget.
    #[must_use]
    pub const fn reference_c6(examples: u64, ordering_seed: u64) -> Self {
        Self {
            arm: EvalArm::C6,
            examples,
            updates: NON_TRAINED_UPDATE_BUDGET,
            ordering_seed,
            stopping: StoppingRule::ExhaustExamples,
            matcher_contract: OPTIMIZER_UPDATE_BUDGET_CONTRACT,
        }
    }
}

/// Witness that two optimizer/update budgets are matched under Slice 25.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MatchedOptimizerUpdateBudget {
    pub examples: u64,
    pub updates: u64,
    pub ordering_seed: u64,
    pub stopping: StoppingRule,
    pub left_arm: EvalArm,
    pub right_arm: EvalArm,
    pub matcher_contract: &'static str,
}

/// Accept only matched optimizer/update budgets; reject unpaired examples,
/// ordering, steps or stopping rules fail-closed.
pub fn match_optimizer_update_budgets(
    left: OptimizerUpdateBudget,
    right: OptimizerUpdateBudget,
) -> Result<MatchedOptimizerUpdateBudget, EvalError> {
    if left.matcher_contract != OPTIMIZER_UPDATE_BUDGET_CONTRACT
        || right.matcher_contract != OPTIMIZER_UPDATE_BUDGET_CONTRACT
    {
        return Err(EvalError::ContractMismatch {
            field: "optimizer_update_budget_contract",
        });
    }
    if left.examples == 0 || right.examples == 0 {
        return Err(EvalError::InvalidBudget);
    }
    if left.examples != right.examples
        || left.updates != right.updates
        || left.ordering_seed != right.ordering_seed
        || left.stopping != right.stopping
    {
        return Err(EvalError::OptimizerUpdateBudgetMismatch {
            left_arm: left.arm,
            right_arm: right.arm,
            left_examples: left.examples,
            right_examples: right.examples,
            left_updates: left.updates,
            right_updates: right.updates,
        });
    }
    Ok(MatchedOptimizerUpdateBudget {
        examples: left.examples,
        updates: left.updates,
        ordering_seed: left.ordering_seed,
        stopping: left.stopping,
        left_arm: left.arm,
        right_arm: right.arm,
        matcher_contract: OPTIMIZER_UPDATE_BUDGET_CONTRACT,
    })
}

/// Closed-set primary quality metric for Stage-C evaluation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PrimaryMetricId {
    /// Paired task accuracy / score-match rate versus the sealed oracle on
    /// Development/Validation only.
    PairedTaskAccuracy,
}

impl PrimaryMetricId {
    /// Stable lowercase metric id.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::PairedTaskAccuracy => PRIMARY_METRIC_PAIRED_TASK_ACCURACY,
        }
    }
}

/// Parse a primary metric id from the closed Stage-C set.
pub fn parse_primary_metric_id(label: &str) -> Result<PrimaryMetricId, EvalError> {
    match label {
        PRIMARY_METRIC_PAIRED_TASK_ACCURACY => Ok(PrimaryMetricId::PairedTaskAccuracy),
        "" => Err(EvalError::MetricRegistryInvalid {
            reason: "empty_primary",
        }),
        _ => Err(EvalError::MetricRegistryInvalid {
            reason: "unknown_primary",
        }),
    }
}

/// Closed-set secondary diagnostics admitted for Stage-C under the frozen registry.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SecondaryDiagnosticId {
    /// Paired task outcome difference across matched arms.
    PairedOutcomeDifference,
    /// Mirror-swap identity error.
    MirrorSwapIdentityError,
    /// Parity-equivariance / invariance error where applicable.
    ParityEquivarianceInvarianceError,
    /// Calibration / confidence error when predictions expose scores.
    CalibrationConfidenceError,
    /// Gradient / stability diagnostics for trained arms.
    GradientStability,
    /// Operation count, memory, latency, throughput under an explicitly qualified environment.
    OpCountMemoryLatencyThroughput,
}

impl SecondaryDiagnosticId {
    /// Stable lowercase diagnostic id.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::PairedOutcomeDifference => SECONDARY_PAIRED_OUTCOME_DIFFERENCE,
            Self::MirrorSwapIdentityError => SECONDARY_MIRROR_SWAP_IDENTITY_ERROR,
            Self::ParityEquivarianceInvarianceError => {
                SECONDARY_PARITY_EQUIVARIANCE_INVARIANCE_ERROR
            }
            Self::CalibrationConfidenceError => SECONDARY_CALIBRATION_CONFIDENCE_ERROR,
            Self::GradientStability => SECONDARY_GRADIENT_STABILITY,
            Self::OpCountMemoryLatencyThroughput => SECONDARY_OP_COUNT_MEMORY_LATENCY_THROUGHPUT,
        }
    }

    /// Canonical ordered Stage-C secondary diagnostic set.
    #[must_use]
    pub const fn admitted_set() -> &'static [Self] {
        PINNED_SECONDARY_DIAGNOSTICS
    }
}

/// Parse a secondary diagnostic id from the closed Stage-C set.
pub fn parse_secondary_diagnostic_id(label: &str) -> Result<SecondaryDiagnosticId, EvalError> {
    match label {
        SECONDARY_PAIRED_OUTCOME_DIFFERENCE => Ok(SecondaryDiagnosticId::PairedOutcomeDifference),
        SECONDARY_MIRROR_SWAP_IDENTITY_ERROR => Ok(SecondaryDiagnosticId::MirrorSwapIdentityError),
        SECONDARY_PARITY_EQUIVARIANCE_INVARIANCE_ERROR => {
            Ok(SecondaryDiagnosticId::ParityEquivarianceInvarianceError)
        }
        SECONDARY_CALIBRATION_CONFIDENCE_ERROR => {
            Ok(SecondaryDiagnosticId::CalibrationConfidenceError)
        }
        SECONDARY_GRADIENT_STABILITY => Ok(SecondaryDiagnosticId::GradientStability),
        SECONDARY_OP_COUNT_MEMORY_LATENCY_THROUGHPUT => {
            Ok(SecondaryDiagnosticId::OpCountMemoryLatencyThroughput)
        }
        _ => Err(EvalError::MetricRegistryInvalid {
            reason: "unknown_secondary",
        }),
    }
}

/// Canonical ordered secondary diagnostics frozen for Stage-C.
pub const PINNED_SECONDARY_DIAGNOSTICS: &[SecondaryDiagnosticId] = &[
    SecondaryDiagnosticId::PairedOutcomeDifference,
    SecondaryDiagnosticId::MirrorSwapIdentityError,
    SecondaryDiagnosticId::ParityEquivarianceInvarianceError,
    SecondaryDiagnosticId::CalibrationConfidenceError,
    SecondaryDiagnosticId::GradientStability,
    SecondaryDiagnosticId::OpCountMemoryLatencyThroughput,
];

/// Frozen Stage-C metric registry: one primary metric plus ordered secondaries.
///
/// Experimental and non-final only; never authorises protected/final evaluation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MetricRegistry {
    /// Primary quality metric frozen before evaluation.
    pub primary: PrimaryMetricId,
    /// Ordered closed-set secondary diagnostics admitted for Stage-C.
    pub secondaries: &'static [SecondaryDiagnosticId],
    /// Registry contract pin.
    pub registry_contract: &'static str,
    /// Must remain true: registry is experimental and non-final only.
    pub experimental_non_final: bool,
}

impl MetricRegistry {
    /// Canonical frozen Stage-C metric registry under the versioned contract.
    #[must_use]
    pub const fn pinned() -> Self {
        Self {
            primary: PrimaryMetricId::PairedTaskAccuracy,
            secondaries: PINNED_SECONDARY_DIAGNOSTICS,
            registry_contract: METRIC_REGISTRY_CONTRACT,
            experimental_non_final: true,
        }
    }

    /// Reject protected/final splits; Development/Validation only.
    pub fn admit_split(self, split: DataSplit) -> Result<(), EvalError> {
        validate_metric_registry(&self)?;
        validate_non_final_split(split)
    }
}

/// Freeze a metric registry under the versioned contract; fail-closed on invalid sets.
pub fn freeze_metric_registry(
    primary: Option<PrimaryMetricId>,
    secondaries: &'static [SecondaryDiagnosticId],
) -> Result<MetricRegistry, EvalError> {
    let Some(primary) = primary else {
        return Err(EvalError::MetricRegistryInvalid {
            reason: "empty_primary",
        });
    };
    let registry = MetricRegistry {
        primary,
        secondaries,
        registry_contract: METRIC_REGISTRY_CONTRACT,
        experimental_non_final: true,
    };
    validate_metric_registry(&registry)?;
    Ok(registry)
}

/// Validate a metric registry: reject contract drift, empty primary, duplicate
/// secondaries, oversized sets, invented ids, non-experimental finals, and any
/// secondary sequence that is not the complete canonical ordered registry.
pub fn validate_metric_registry(registry: &MetricRegistry) -> Result<(), EvalError> {
    if registry.registry_contract != METRIC_REGISTRY_CONTRACT {
        return Err(EvalError::ContractMismatch {
            field: "metric_registry_contract",
        });
    }
    if !registry.experimental_non_final {
        return Err(EvalError::MetricRegistryInvalid {
            reason: "experimental_non_final_required",
        });
    }
    // Re-parse primary through the closed-set gate (guards future variants).
    parse_primary_metric_id(registry.primary.as_str())?;
    if registry.secondaries.len() > MAX_SECONDARY_DIAGNOSTICS {
        return Err(EvalError::MetricRegistryInvalid {
            reason: "too_many_secondaries",
        });
    }
    for (index, secondary) in registry.secondaries.iter().enumerate() {
        parse_secondary_diagnostic_id(secondary.as_str())?;
        for prior in &registry.secondaries[..index] {
            if prior == secondary {
                return Err(EvalError::MetricRegistryInvalid {
                    reason: "duplicate_secondary",
                });
            }
        }
        if !PINNED_SECONDARY_DIAGNOSTICS.contains(secondary) {
            return Err(EvalError::MetricRegistryInvalid {
                reason: "unknown_secondary",
            });
        }
    }
    if registry.secondaries != PINNED_SECONDARY_DIAGNOSTICS {
        return Err(EvalError::MetricRegistryInvalid {
            reason: "secondary_sequence_mismatch",
        });
    }
    Ok(())
}

/// Interval construction method retained on every Stage-C uncertainty summary.
///
/// Choice (documented, deterministic, no RNG):
/// - [`UncertaintyMethod::WilsonScore`] for single-arm accuracy rates when
///   callers provide independent revealed match bits.
/// - [`UncertaintyMethod::BoundedHoeffdingPairedDifference`] for direct paired
///   differences in [-1, 1]. The finite-sample bound remains non-degenerate at
///   all-tie and all-win boundaries.
/// - The cluster-aware variants treat each record `group_id` as one independent
///   unit and use the observed cluster sizes in the Hoeffding range term. This
///   preserves related mirrored/reflected/reversal/nuisance cases as clusters.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum UncertaintyMethod {
    /// Wilson score interval on an independent single-arm match-oracle rate.
    WilsonScore,
    /// Bounded Hoeffding interval on independent paired differences.
    BoundedHoeffdingPairedDifference,
    /// Cluster-aware Hoeffding interval on a Bernoulli mean.
    ClusterHoeffdingBernoulliMean,
    /// Cluster-aware Hoeffding interval on paired differences.
    ClusterHoeffdingPairedDifference,
}

impl UncertaintyMethod {
    /// Stable lowercase method id.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::WilsonScore => "wilson_score",
            Self::BoundedHoeffdingPairedDifference => "bounded_hoeffding_paired_difference",
            Self::ClusterHoeffdingBernoulliMean => "cluster_hoeffding_bernoulli_mean",
            Self::ClusterHoeffdingPairedDifference => "cluster_hoeffding_paired_difference",
        }
    }
}

/// Closed confidence interval with retained method and nominal level.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ConfidenceInterval {
    pub lower: f64,
    pub upper: f64,
    pub confidence: f64,
    pub method: UncertaintyMethod,
}

impl ConfidenceInterval {
    fn checked(
        lower: f64,
        upper: f64,
        confidence: f64,
        method: UncertaintyMethod,
    ) -> Result<Self, EvalError> {
        if !lower.is_finite() || !upper.is_finite() || !confidence.is_finite() {
            return Err(EvalError::PairedUncertaintyInvalid {
                reason: "non_finite",
            });
        }
        if lower > upper {
            return Err(EvalError::PairedUncertaintyInvalid {
                reason: "non_finite",
            });
        }
        Ok(Self {
            lower,
            upper,
            confidence,
            method,
        })
    }
}

/// Already-revealed match bit admitted into the uncertainty surface.
///
/// Construct only from evaluator-retained `matches_oracle` / `correct` flags
/// (or synthetic test vectors of the same shape). There is intentionally no
/// constructor from [`super::tdi24_tasks::ProtectedLabel`], raw sealed targets,
/// or leak oracles beyond what [`EvalRecord`] already retained.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RevealedMatchOutcome {
    /// Whether the arm score matched the sealed oracle on the evaluation path.
    pub matches_oracle: bool,
}

impl RevealedMatchOutcome {
    /// Build from an evaluator-retained correctness / match-oracle bit.
    #[must_use]
    pub const fn from_matches_oracle(matches_oracle: bool) -> Self {
        Self { matches_oracle }
    }
}

/// Paired V6/C6 effect summary bound to the frozen metric registry.
///
/// Experimental and non-final only; never authorises protected/final evaluation
/// or scientific claims.
#[derive(Clone, Debug, PartialEq)]
pub struct PairedEffectSummary {
    pub split: DataSplit,
    pub n_pairs: u64,
    pub v6_accuracy: f64,
    pub c6_accuracy: f64,
    /// Mean of per-pair differences `I(C6)-I(V6)`; equals `c6_accuracy - v6_accuracy`.
    pub paired_difference_mean: f64,
    pub paired_difference_ci: ConfidenceInterval,
    pub v6_accuracy_ci: ConfidenceInterval,
    pub c6_accuracy_ci: ConfidenceInterval,
    pub primary_metric: PrimaryMetricId,
    pub secondary_paired_outcome_difference: SecondaryDiagnosticId,
    pub uncertainty_contract: &'static str,
    pub metric_registry_contract: &'static str,
    pub experimental_non_final: bool,
}

/// Wilson score interval for `successes` in `n` Bernoulli trials at 95%.
fn wilson_score_interval(successes: u64, n: u64) -> Result<ConfidenceInterval, EvalError> {
    if n == 0 {
        return Err(EvalError::PairedUncertaintyInvalid {
            reason: "empty_pairs",
        });
    }
    let n_f = n as f64;
    let p = successes as f64 / n_f;
    let z = NORMAL_CRITICAL_Z_95;
    let z2 = z * z;
    let denom = 1.0 + z2 / n_f;
    if !denom.is_finite() || denom == 0.0 {
        return Err(EvalError::PairedUncertaintyInvalid {
            reason: "non_finite",
        });
    }
    let center = (p + z2 / (2.0 * n_f)) / denom;
    let inner = p * (1.0 - p) / n_f + z2 / (4.0 * n_f * n_f);
    if !inner.is_finite() || inner < 0.0 {
        return Err(EvalError::PairedUncertaintyInvalid {
            reason: "non_finite",
        });
    }
    let margin = z * inner.sqrt() / denom;
    ConfidenceInterval::checked(
        center - margin,
        center + margin,
        PAIRED_UNCERTAINTY_LEVEL,
        UncertaintyMethod::WilsonScore,
    )
}

/// Distribution-free Hoeffding interval on a bounded mean.
///
/// `sum_squared_cluster_sizes` is `n` for independent rows and
/// `sum_g n_g^2` for record groups. The paired-difference range is [-1, 1]
/// (width two); Bernoulli means use [0, 1] (width one). Bounds are clipped to
/// the natural range and remain non-degenerate at finite-sample boundaries.
fn bounded_hoeffding_mean_interval(
    mean: f64,
    n: usize,
    sum_squared_cluster_sizes: u64,
    lower_bound: f64,
    upper_bound: f64,
    method: UncertaintyMethod,
) -> Result<ConfidenceInterval, EvalError> {
    if n < 2 {
        return Err(EvalError::PairedUncertaintyInvalid {
            reason: "insufficient_pairs",
        });
    }
    if !mean.is_finite() || lower_bound >= upper_bound {
        return Err(EvalError::PairedUncertaintyInvalid {
            reason: "non_finite",
        });
    }
    let n_u64 = n as u64;
    if sum_squared_cluster_sizes < n_u64 || sum_squared_cluster_sizes > n_u64.saturating_mul(n_u64)
    {
        return Err(EvalError::PairedUncertaintyInvalid {
            reason: "invalid_cluster_sizes",
        });
    }
    let n_f = n as f64;
    let range_width = upper_bound - lower_bound;
    let margin = range_width
        * ((sum_squared_cluster_sizes as f64 * HOEFFDING_LOG_40) / (2.0 * n_f * n_f)).sqrt();
    if !margin.is_finite() {
        return Err(EvalError::PairedUncertaintyInvalid {
            reason: "non_finite",
        });
    }
    ConfidenceInterval::checked(
        (mean - margin).max(lower_bound),
        (mean + margin).min(upper_bound),
        PAIRED_UNCERTAINTY_LEVEL,
        method,
    )
}

/// Bounded finite-sample CI on the mean of paired outcome differences.
fn bounded_hoeffding_paired_difference_ci(
    differences: &[f64],
    sum_squared_cluster_sizes: u64,
    cluster_aware: bool,
) -> Result<(f64, ConfidenceInterval), EvalError> {
    if differences.is_empty() {
        return Err(EvalError::PairedUncertaintyInvalid {
            reason: "empty_pairs",
        });
    }
    if differences.len() < 2 {
        return Err(EvalError::PairedUncertaintyInvalid {
            reason: "insufficient_pairs",
        });
    }
    let n_f = differences.len() as f64;
    let mean = differences.iter().sum::<f64>() / n_f;
    let method = if cluster_aware {
        UncertaintyMethod::ClusterHoeffdingPairedDifference
    } else {
        UncertaintyMethod::BoundedHoeffdingPairedDifference
    };
    let ci = bounded_hoeffding_mean_interval(
        mean,
        differences.len(),
        sum_squared_cluster_sizes,
        -1.0,
        1.0,
        method,
    )?;
    Ok((mean, ci))
}

/// Bind a registry to the frozen Stage-C pin required by the uncertainty engine.
fn require_pinned_metric_registry(registry: &MetricRegistry) -> Result<(), EvalError> {
    validate_metric_registry(registry)?;
    if *registry != MetricRegistry::pinned() {
        return Err(EvalError::PairedUncertaintyInvalid {
            reason: "registry_not_pinned",
        });
    }
    if registry.primary != PrimaryMetricId::PairedTaskAccuracy {
        return Err(EvalError::PairedUncertaintyInvalid {
            reason: "primary_metric_mismatch",
        });
    }
    if !registry
        .secondaries
        .contains(&SecondaryDiagnosticId::PairedOutcomeDifference)
    {
        return Err(EvalError::PairedUncertaintyInvalid {
            reason: "secondary_paired_outcome_missing",
        });
    }
    Ok(())
}

/// Summarise paired V6/C6 match outcomes into accuracy rates, paired effect, and CIs.
///
/// Consumes only already-revealed [`RevealedMatchOutcome`] bits (evaluator-retained
/// match-oracle / correctness flags). Rejects empty pairs, V6/C6 length mismatch,
/// non-finite statistics, contract / registry pin drift, and protected/final splits.
pub fn summarize_paired_uncertainty(
    split: DataSplit,
    v6_matches: &[RevealedMatchOutcome],
    c6_matches: &[RevealedMatchOutcome],
    registry: &MetricRegistry,
) -> Result<PairedEffectSummary, EvalError> {
    summarize_paired_uncertainty_inner(split, v6_matches, c6_matches, registry, None)
}

fn summarize_paired_uncertainty_inner(
    split: DataSplit,
    v6_matches: &[RevealedMatchOutcome],
    c6_matches: &[RevealedMatchOutcome],
    registry: &MetricRegistry,
    cluster_sum_squares: Option<u64>,
) -> Result<PairedEffectSummary, EvalError> {
    validate_non_final_split(split)?;
    require_pinned_metric_registry(registry)?;

    if v6_matches.is_empty() || c6_matches.is_empty() {
        return Err(EvalError::PairedUncertaintyInvalid {
            reason: "empty_pairs",
        });
    }
    if v6_matches.len() != c6_matches.len() {
        return Err(EvalError::PairedUncertaintyInvalid {
            reason: "length_mismatch",
        });
    }
    if v6_matches.len() < 2 {
        return Err(EvalError::PairedUncertaintyInvalid {
            reason: "insufficient_pairs",
        });
    }
    if v6_matches.len() > MAX_CASES_PER_RUN as usize {
        return Err(EvalError::PairedUncertaintyInvalid {
            reason: "too_many_pairs",
        });
    }

    let n = v6_matches.len();
    let mut v6_successes = 0_u64;
    let mut c6_successes = 0_u64;
    let mut differences = Vec::with_capacity(n);
    for (v6, c6) in v6_matches.iter().zip(c6_matches.iter()) {
        let v = u64::from(v6.matches_oracle);
        let c = u64::from(c6.matches_oracle);
        v6_successes += v;
        c6_successes += c;
        differences.push(c as f64 - v as f64);
    }

    let n_f = n as f64;
    let v6_accuracy = v6_successes as f64 / n_f;
    let c6_accuracy = c6_successes as f64 / n_f;
    if !v6_accuracy.is_finite() || !c6_accuracy.is_finite() {
        return Err(EvalError::PairedUncertaintyInvalid {
            reason: "non_finite",
        });
    }

    let cluster_aware = cluster_sum_squares.is_some();
    let sum_squared_cluster_sizes = cluster_sum_squares.unwrap_or(n as u64);
    let (v6_accuracy_ci, c6_accuracy_ci) = if cluster_aware {
        (
            bounded_hoeffding_mean_interval(
                v6_accuracy,
                n,
                sum_squared_cluster_sizes,
                0.0,
                1.0,
                UncertaintyMethod::ClusterHoeffdingBernoulliMean,
            )?,
            bounded_hoeffding_mean_interval(
                c6_accuracy,
                n,
                sum_squared_cluster_sizes,
                0.0,
                1.0,
                UncertaintyMethod::ClusterHoeffdingBernoulliMean,
            )?,
        )
    } else {
        (
            wilson_score_interval(v6_successes, n as u64)?,
            wilson_score_interval(c6_successes, n as u64)?,
        )
    };
    let (paired_difference_mean, paired_difference_ci) = bounded_hoeffding_paired_difference_ci(
        &differences,
        sum_squared_cluster_sizes,
        cluster_aware,
    )?;

    // Consistency: mean of paired differences must equal accuracy gap.
    let expected_gap = c6_accuracy - v6_accuracy;
    if !paired_difference_mean.is_finite() || (paired_difference_mean - expected_gap).abs() > 1e-12
    {
        return Err(EvalError::PairedUncertaintyInvalid {
            reason: "non_finite",
        });
    }

    Ok(PairedEffectSummary {
        split,
        n_pairs: n as u64,
        v6_accuracy,
        c6_accuracy,
        paired_difference_mean,
        paired_difference_ci,
        v6_accuracy_ci,
        c6_accuracy_ci,
        primary_metric: registry.primary,
        secondary_paired_outcome_difference: SecondaryDiagnosticId::PairedOutcomeDifference,
        uncertainty_contract: PAIRED_UNCERTAINTY_CONTRACT,
        metric_registry_contract: registry.registry_contract,
        experimental_non_final: true,
    })
}

/// Extract revealed match bits from evaluator records for one arm/split.
///
/// Accepts only [`EvalOutcome::Scored`] rows whose arm, split, and metric-registry
/// contract match the expected pins. Failures and contract drift fail closed.
/// Does not accept [`super::tdi24_tasks::ProtectedLabel`] or raw sealed targets.
pub fn revealed_matches_from_records(
    records: &[EvalRecord],
    expected_arm: EvalArm,
    expected_split: DataSplit,
) -> Result<Vec<RevealedMatchOutcome>, EvalError> {
    validate_non_final_split(expected_split)?;
    if records.is_empty() {
        return Err(EvalError::PairedUncertaintyInvalid {
            reason: "empty_pairs",
        });
    }
    if records.len() > MAX_CASES_PER_RUN as usize {
        return Err(EvalError::PairedUncertaintyInvalid {
            reason: "too_many_pairs",
        });
    }
    let mut out = Vec::with_capacity(records.len());
    for record in records {
        if record.arm != expected_arm {
            return Err(EvalError::PairedUncertaintyInvalid {
                reason: "arm_mismatch",
            });
        }
        if record.split != expected_split {
            return Err(EvalError::SplitMismatch {
                expected: expected_split,
                actual: record.split,
            });
        }
        if record.metric_registry_contract != METRIC_REGISTRY_CONTRACT {
            return Err(EvalError::ContractMismatch {
                field: "metric_registry_contract",
            });
        }
        if record.envelope_contract != EVALUATOR_ENVELOPE_CONTRACT {
            return Err(EvalError::ContractMismatch {
                field: "envelope_contract",
            });
        }
        if record.arm_contract != expected_arm.evaluator_contract() {
            return Err(EvalError::ContractMismatch {
                field: "arm_contract",
            });
        }
        if record.budget_contract != READOUT_BUDGET_CONTRACT {
            return Err(EvalError::ContractMismatch {
                field: "budget_contract",
            });
        }
        let expected_vector_contract = match expected_arm {
            EvalArm::V6 => VECTOR6_CONTRACT,
            EvalArm::C6 => CHIRAL_CONTRACT,
        };
        if record.vector_contract != expected_vector_contract {
            return Err(EvalError::ContractMismatch {
                field: "vector_contract",
            });
        }
        if record.label_contract != PROTECTED_LABEL_CONTRACT {
            return Err(EvalError::ContractMismatch {
                field: "label_contract",
            });
        }
        match record.outcome {
            EvalOutcome::Scored { correct, .. } => {
                out.push(RevealedMatchOutcome::from_matches_oracle(correct));
            }
            EvalOutcome::Failure(_) => {
                return Err(EvalError::PairedUncertaintyInvalid {
                    reason: "unscored_outcome",
                });
            }
        }
    }
    Ok(out)
}

/// Return sum_g n_g^2 for the record grouping, bounded by the evaluator cap.
fn sum_squared_group_sizes(records: &[EvalRecord]) -> Result<u64, EvalError> {
    let mut groups: Vec<(TaskFamily, u64, u64)> = Vec::with_capacity(records.len());
    for record in records {
        if let Some((_, _, size)) = groups
            .iter_mut()
            .find(|(family, group_id, _)| *family == record.family && *group_id == record.group_id)
        {
            *size += 1;
        } else {
            groups.push((record.family, record.group_id, 1));
        }
    }
    groups.into_iter().try_fold(0_u64, |sum, (_, _, size)| {
        size.checked_mul(size)
            .and_then(|square| sum.checked_add(square))
            .ok_or(EvalError::PairedUncertaintyInvalid {
                reason: "invalid_cluster_sizes",
            })
    })
}

/// Summarise paired V6/C6 [`EvalRecord`] slices without re-entering label oracles.
///
/// Records must be equal-length and identity-aligned in order: matching
/// `family`, `case_id`, `group_id`, and `canonical_digest` at each index.
/// Positional zip without identity checks is rejected fail-closed.
pub fn summarize_paired_uncertainty_from_records(
    split: DataSplit,
    v6_records: &[EvalRecord],
    c6_records: &[EvalRecord],
    registry: &MetricRegistry,
) -> Result<PairedEffectSummary, EvalError> {
    if v6_records.len() != c6_records.len() {
        return Err(EvalError::PairedUncertaintyInvalid {
            reason: "length_mismatch",
        });
    }
    if v6_records.len() > MAX_CASES_PER_RUN as usize {
        return Err(EvalError::PairedUncertaintyInvalid {
            reason: "too_many_pairs",
        });
    }
    for (index, (left, right)) in v6_records.iter().zip(c6_records.iter()).enumerate() {
        if left.family != right.family
            || left.case_id != right.case_id
            || left.group_id != right.group_id
            || left.canonical_digest != right.canonical_digest
        {
            return Err(EvalError::PairedUncertaintyInvalid {
                reason: "pair_identity_mismatch",
            });
        }
        if v6_records[..index].iter().any(|prior| {
            prior.family == left.family
                && prior.case_id == left.case_id
                && prior.group_id == left.group_id
                && prior.canonical_digest == left.canonical_digest
        }) {
            return Err(EvalError::PairedUncertaintyInvalid {
                reason: "duplicate_pair_identity",
            });
        }
    }
    let cluster_sum_squares = sum_squared_group_sizes(v6_records)?;
    let v6 = revealed_matches_from_records(v6_records, EvalArm::V6, split)?;
    let c6 = revealed_matches_from_records(c6_records, EvalArm::C6, split)?;
    summarize_paired_uncertainty_inner(split, &v6, &c6, registry, Some(cluster_sum_squares))
}

/// Closed retained failure class covering the campaign gate wording
/// (invalid / numerical / resource / task).
///
/// Experimental and non-final only. Unknown class labels fail closed.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FailureClass {
    /// Contract, budget, registry, matcher, or other invalid configuration/input.
    Invalid,
    /// Arm arithmetic rejected a non-finite intermediate.
    Numerical,
    /// Readout or case / failure-ledger budget exhausted.
    Resource,
    /// Task, oracle, or case-split mapping could not be applied.
    Task,
}

impl FailureClass {
    /// Stable lowercase failure-class token.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Invalid => "invalid",
            Self::Numerical => "numerical",
            Self::Resource => "resource",
            Self::Task => "task",
        }
    }
}

/// Parse a failure-class label; unknown tokens fail closed.
pub fn parse_failure_class(label: &str) -> Result<FailureClass, EvalError> {
    match label {
        "invalid" => Ok(FailureClass::Invalid),
        "numerical" => Ok(FailureClass::Numerical),
        "resource" => Ok(FailureClass::Resource),
        "task" => Ok(FailureClass::Task),
        "" => Err(EvalError::FailureTaxonomyInvalid {
            reason: "empty_class",
        }),
        _ => Err(EvalError::FailureTaxonomyInvalid {
            reason: "unknown_class",
        }),
    }
}

/// One retained typed failure; never silently discarded once classified.
///
/// Bound to [`FAILURE_TAXONOMY_CONTRACT`]. Development/Validation only —
/// constructing with a protected/final split is rejected by
/// [`retain_failure`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FailureRecord {
    pub class: FailureClass,
    pub arm: EvalArm,
    pub split: DataSplit,
    pub case_id: Option<u64>,
    pub message_code: &'static str,
    pub taxonomy_contract: &'static str,
}

/// Accumulator that retains every classified failure for one non-final run.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FailureLedger {
    split: DataSplit,
    taxonomy_contract: &'static str,
    records: Vec<FailureRecord>,
}

impl FailureLedger {
    /// Open a bounded Development/Validation failure ledger.
    pub fn open(split: DataSplit) -> Result<Self, EvalError> {
        validate_non_final_split(split)?;
        Ok(Self {
            split,
            taxonomy_contract: FAILURE_TAXONOMY_CONTRACT,
            records: Vec::new(),
        })
    }

    /// Borrow the split this ledger was opened under.
    #[must_use]
    pub const fn split(&self) -> DataSplit {
        self.split
    }

    /// Borrow the taxonomy contract pin.
    #[must_use]
    pub const fn taxonomy_contract(&self) -> &'static str {
        self.taxonomy_contract
    }

    /// Borrow retained failure records in admission order.
    #[must_use]
    pub fn records(&self) -> &[FailureRecord] {
        &self.records
    }

    /// Classify and retain an [`EvalError`]; never silently discards.
    pub fn retain_eval_error(
        &mut self,
        error: &EvalError,
        arm: EvalArm,
        case_id: Option<u64>,
    ) -> Result<&FailureRecord, EvalError> {
        let record = retain_eval_error(error, arm, self.split, case_id)?;
        self.push_validated(record)
    }

    /// Map an evaluator-retained [`EvalFailure`] and keep it; never silently discards.
    pub fn retain_eval_failure(
        &mut self,
        failure: EvalFailure,
        arm: EvalArm,
        case_id: Option<u64>,
    ) -> Result<&FailureRecord, EvalError> {
        let record = retain_eval_failure(failure, arm, self.split, case_id)?;
        self.push_validated(record)
    }

    /// Retain an already-built record after validating pins and capacity.
    pub fn retain(&mut self, record: FailureRecord) -> Result<&FailureRecord, EvalError> {
        validate_failure_record(&record)?;
        if record.split != self.split {
            return Err(EvalError::SplitMismatch {
                expected: self.split,
                actual: record.split,
            });
        }
        self.push_validated(record)
    }

    fn push_validated(&mut self, record: FailureRecord) -> Result<&FailureRecord, EvalError> {
        if self.taxonomy_contract != FAILURE_TAXONOMY_CONTRACT {
            return Err(EvalError::FailureTaxonomyInvalid {
                reason: "contract_drift",
            });
        }
        if record.taxonomy_contract != FAILURE_TAXONOMY_CONTRACT {
            return Err(EvalError::FailureTaxonomyInvalid {
                reason: "contract_drift",
            });
        }
        if self.records.len() as u64 >= MAX_FAILURES_PER_RUN {
            // Ledger capacity is a resource bound (same class as case budget).
            return Err(EvalError::CaseBudgetExceeded);
        }
        self.records.push(record);
        Ok(self.records.last().expect("just pushed"))
    }
}

/// Classify an [`EvalError`] into the closed [`FailureClass`] set.
///
/// Never silently discards: every admitted error maps to exactly one class.
/// [`EvalError::ProtectedOrFinalSplit`] is not retained as a taxonomy class —
/// it stays a hard rejection so this surface cannot authorise protected/final
/// evaluation.
pub fn classify_eval_error(error: &EvalError) -> Result<FailureClass, EvalError> {
    match error {
        EvalError::ProtectedOrFinalSplit => Err(EvalError::ProtectedOrFinalSplit),
        EvalError::Numerical(_) | EvalError::ChiralNumerical(_) => Ok(FailureClass::Numerical),
        EvalError::CaseBudgetExceeded => Ok(FailureClass::Resource),
        EvalError::SplitMismatch { .. } => Ok(FailureClass::Task),
        EvalError::ContractMismatch { .. }
        | EvalError::InvalidBudget
        | EvalError::UnsupportedArm
        | EvalError::ParameterCountMismatch { .. }
        | EvalError::InitializationMismatch { .. }
        | EvalError::OptimizerUpdateBudgetMismatch { .. }
        | EvalError::MetricRegistryInvalid { .. }
        | EvalError::PairedUncertaintyInvalid { .. }
        | EvalError::FailureTaxonomyInvalid { .. }
        | EvalError::ProvenanceEnvelopeInvalid { .. }
        | EvalError::StageCPreflightInvalid { .. }
        | EvalError::GammaZeroAblationInvalid { .. }
        | EvalError::BetaZeroAblationInvalid { .. }
        | EvalError::DirectOnlyCollapseInvalid { .. }
        | EvalError::ParityShuffleControlInvalid { .. }
        | EvalError::FixedMSensitivityInvalid { .. }
        | EvalError::LearnedBasisPrototypeInvalid { .. }
        | EvalError::HeadSharingAblationInvalid { .. }
        | EvalError::WidthScalingInvalid { .. }
        | EvalError::SequenceLengthScalingInvalid { .. }
        | EvalError::StageDAttributionAuditInvalid { .. }
        | EvalError::MultiSeedReplicationInvalid { .. }
        | EvalError::InputNoiseRobustnessInvalid { .. }
        | EvalError::ReflectionAdversarialInvalid { .. }
        | EvalError::NumericalPrecisionInvalid { .. } => Ok(FailureClass::Invalid),
    }
}

/// Map an evaluator-retained [`EvalFailure`] onto the closed taxonomy.
#[must_use]
pub const fn classify_eval_failure(failure: EvalFailure) -> FailureClass {
    match failure {
        EvalFailure::Numerical => FailureClass::Numerical,
        EvalFailure::Task => FailureClass::Task,
        EvalFailure::Resource => FailureClass::Resource,
        // Contract/config failures land in the campaign "invalid" class.
        EvalFailure::Contract => FailureClass::Invalid,
    }
}

/// Stable message code for one [`EvalError`] variant.
#[must_use]
pub const fn eval_error_message_code(error: &EvalError) -> &'static str {
    match error {
        EvalError::ProtectedOrFinalSplit => "protected_or_final_split",
        EvalError::SplitMismatch { .. } => "split_mismatch",
        EvalError::ContractMismatch { .. } => "contract_mismatch",
        EvalError::InvalidBudget => "invalid_budget",
        EvalError::UnsupportedArm => "unsupported_arm",
        EvalError::CaseBudgetExceeded => "case_budget_exceeded",
        EvalError::Numerical(_) => "numerical",
        EvalError::ChiralNumerical(_) => "chiral_numerical",
        EvalError::ParameterCountMismatch { .. } => "parameter_count_mismatch",
        EvalError::InitializationMismatch { .. } => "initialization_mismatch",
        EvalError::OptimizerUpdateBudgetMismatch { .. } => "optimizer_update_budget_mismatch",
        EvalError::MetricRegistryInvalid { .. } => "metric_registry_invalid",
        EvalError::PairedUncertaintyInvalid { .. } => "paired_uncertainty_invalid",
        EvalError::FailureTaxonomyInvalid { .. } => "failure_taxonomy_invalid",
        EvalError::ProvenanceEnvelopeInvalid { .. } => "provenance_envelope_invalid",
        EvalError::StageCPreflightInvalid { .. } => "stage_c_preflight_invalid",
        EvalError::GammaZeroAblationInvalid { .. } => "gamma_zero_ablation_invalid",
        EvalError::BetaZeroAblationInvalid { .. } => "beta_zero_ablation_invalid",
        EvalError::DirectOnlyCollapseInvalid { .. } => "direct_only_collapse_invalid",
        EvalError::ParityShuffleControlInvalid { .. } => "parity_shuffle_control_invalid",
        EvalError::FixedMSensitivityInvalid { .. } => "fixed_m_sensitivity_invalid",
        EvalError::LearnedBasisPrototypeInvalid { .. } => "learned_basis_prototype_invalid",
        EvalError::HeadSharingAblationInvalid { .. } => "head_sharing_ablation_invalid",
        EvalError::WidthScalingInvalid { .. } => "width_scaling_invalid",
        EvalError::SequenceLengthScalingInvalid { .. } => "sequence_length_scaling_invalid",
        EvalError::StageDAttributionAuditInvalid { .. } => "stage_d_attribution_audit_invalid",
        EvalError::MultiSeedReplicationInvalid { .. } => "multi_seed_replication_invalid",
        EvalError::InputNoiseRobustnessInvalid { .. } => "input_noise_robustness_invalid",
        EvalError::ReflectionAdversarialInvalid { .. } => "reflection_adversarial_invalid",
        EvalError::NumericalPrecisionInvalid { .. } => "numerical_precision_invalid",
    }
}

/// Stable message code for one evaluator-retained [`EvalFailure`].
#[must_use]
pub const fn eval_failure_message_code(failure: EvalFailure) -> &'static str {
    match failure {
        EvalFailure::Numerical => "eval_failure_numerical",
        EvalFailure::Task => "eval_failure_task",
        EvalFailure::Resource => "eval_failure_resource",
        EvalFailure::Contract => "eval_failure_contract",
    }
}

/// Validate a retained failure record against the closed taxonomy pin.
pub fn validate_failure_record(record: &FailureRecord) -> Result<(), EvalError> {
    validate_non_final_split(record.split)?;
    if record.taxonomy_contract != FAILURE_TAXONOMY_CONTRACT {
        return Err(EvalError::FailureTaxonomyInvalid {
            reason: "contract_drift",
        });
    }
    // Round-trip through the closed parser so invented class labels fail closed.
    let parsed = parse_failure_class(record.class.as_str())?;
    if parsed != record.class {
        return Err(EvalError::FailureTaxonomyInvalid {
            reason: "unknown_class",
        });
    }
    if record.message_code.is_empty() {
        return Err(EvalError::FailureTaxonomyInvalid {
            reason: "empty_message_code",
        });
    }
    Ok(())
}

/// Retain a classified failure; never silently discards.
///
/// Rejects protected/final splits, empty message codes, and class/contract
/// drift. Successful returns always carry [`FAILURE_TAXONOMY_CONTRACT`].
pub fn retain_failure(
    class: FailureClass,
    arm: EvalArm,
    split: DataSplit,
    case_id: Option<u64>,
    message_code: &'static str,
) -> Result<FailureRecord, EvalError> {
    let record = FailureRecord {
        class,
        arm,
        split,
        case_id,
        message_code,
        taxonomy_contract: FAILURE_TAXONOMY_CONTRACT,
    };
    validate_failure_record(&record)?;
    Ok(record)
}

/// Classify an [`EvalError`] and retain it as a [`FailureRecord`].
///
/// Never silently discards. Protected/final errors remain hard rejections.
pub fn retain_eval_error(
    error: &EvalError,
    arm: EvalArm,
    split: DataSplit,
    case_id: Option<u64>,
) -> Result<FailureRecord, EvalError> {
    let class = classify_eval_error(error)?;
    retain_failure(class, arm, split, case_id, eval_error_message_code(error))
}

/// Map an evaluator-retained [`EvalFailure`] into a [`FailureRecord`].
pub fn retain_eval_failure(
    failure: EvalFailure,
    arm: EvalArm,
    split: DataSplit,
    case_id: Option<u64>,
) -> Result<FailureRecord, EvalError> {
    retain_failure(
        classify_eval_failure(failure),
        arm,
        split,
        case_id,
        eval_failure_message_code(failure),
    )
}

/// Validate evaluator provenance pins on a source [`EvalRecord`].
///
/// Mirrors the contract gate used by [`revealed_matches_from_records`] so stale
/// or drifted records cannot be repinned under the current taxonomy.
pub fn validate_eval_record_contracts(record: &EvalRecord) -> Result<(), EvalError> {
    validate_non_final_split(record.split)?;
    if record.envelope_contract != EVALUATOR_ENVELOPE_CONTRACT {
        return Err(EvalError::ContractMismatch {
            field: "envelope_contract",
        });
    }
    if record.arm_contract != record.arm.evaluator_contract() {
        return Err(EvalError::ContractMismatch {
            field: "arm_contract",
        });
    }
    if record.budget_contract != READOUT_BUDGET_CONTRACT {
        return Err(EvalError::ContractMismatch {
            field: "budget_contract",
        });
    }
    if record.metric_registry_contract != METRIC_REGISTRY_CONTRACT {
        return Err(EvalError::ContractMismatch {
            field: "metric_registry_contract",
        });
    }
    let expected_vector_contract = match record.arm {
        EvalArm::V6 => VECTOR6_CONTRACT,
        EvalArm::C6 => CHIRAL_CONTRACT,
    };
    if record.vector_contract != expected_vector_contract {
        return Err(EvalError::ContractMismatch {
            field: "vector_contract",
        });
    }
    if record.label_contract != PROTECTED_LABEL_CONTRACT {
        return Err(EvalError::ContractMismatch {
            field: "label_contract",
        });
    }
    Ok(())
}

/// Extract a retained failure from an [`EvalRecord`] outcome when present.
///
/// Scored outcomes yield `Ok(None)`. Failure outcomes are classified and
/// retained; they are never dropped. Stale envelope/arm/budget/registry/vector/
/// label contract pins fail closed before repinning under the taxonomy.
pub fn retain_from_eval_record(record: &EvalRecord) -> Result<Option<FailureRecord>, EvalError> {
    validate_eval_record_contracts(record)?;
    match record.outcome {
        EvalOutcome::Scored { .. } => Ok(None),
        EvalOutcome::Failure(failure) => Ok(Some(retain_eval_failure(
            failure,
            record.arm,
            record.split,
            Some(record.case_id),
        )?)),
    }
}

/// One admitted case bound into a provenance envelope (issue #690).
///
/// Identifies the exact inference-visible case and the registered per-case
/// seed `register_seed(domain(split), family, group_id)` that the Phase-B
/// generators derive it from. Never carries a target or oracle label.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AdmittedCase {
    pub family: TaskFamily,
    pub case_id: u64,
    pub group_id: u64,
    /// Canonical inference-view digest (slice 19).
    pub canonical_digest: String,
    /// Registered seed bound to this case's family and `group_id` (slice 18).
    pub seed: RegisteredSeed,
}

impl AdmittedCase {
    /// Bind one inference view: canonical digest plus its registered seed.
    #[must_use]
    pub fn from_view(view: &InferenceView) -> Self {
        Self {
            family: view.family,
            case_id: view.case_id,
            group_id: view.group_id,
            canonical_digest: canonicalize_inference_view(view).digest,
            seed: register_seed(
                SeedDomain::from_split(view.split),
                view.family,
                view.group_id,
            ),
        }
    }

    fn canonical_line(&self) -> String {
        format!(
            "family={};case={:016x};group={:016x};digest={};seed={:016x}\n",
            self.family.as_str(),
            self.case_id,
            self.group_id,
            self.canonical_digest,
            self.seed.mixed_seed
        )
    }
}

/// One immutable provenance envelope for a single non-final evaluator run.
///
/// Envelope v2 (issue #690) binds the exact source surface, the complete
/// canonical evaluator configuration, the admitted population and its per-case
/// seeds, and the compiler that actually built the evaluator. No freeze pin is
/// invented. Development/Validation only — protected/final splits are rejected.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProvenanceEnvelope {
    pub provenance_contract: &'static str,
    pub arm: EvalArm,
    pub split: DataSplit,
    /// Source-content digest of the TDI-24 evaluator surface plus arm contracts.
    pub code_identity: String,
    /// Canonical rendering of the complete [`EvaluatorConfig`].
    pub config_identity: String,
    /// Admitted-population digest (count, ordered case digests and seeds).
    pub data_identity: String,
    /// Per-case seed binding digest under the slice-18 registry.
    pub seed_identity: String,
    /// Ordered admitted population; every scored case must appear here.
    pub admitted_cases: Vec<AdmittedCase>,
    /// Release of the compiler that built the evaluator (see [`build_rustc_release`]).
    pub toolchain_channel: &'static str,
    /// Declared Cargo features (see [`PROVENANCE_TOOLCHAIN_FEATURES`]).
    pub toolchain_features: &'static str,
    /// Full `rustc --version` of the build compiler ([`BUILD_RUSTC_VERSION`]).
    pub compiler_identity: &'static str,
}

impl ProvenanceEnvelope {
    /// Construct and validate a provenance envelope for a Development/Validation run.
    ///
    /// Every identity is checked against the values recomputed from the
    /// admitted population and the compiled source; use
    /// [`Self::for_pinned_stage_c_run`] to derive them.
    #[allow(clippy::too_many_arguments)]
    pub fn for_non_final_run(
        arm: EvalArm,
        split: DataSplit,
        code_identity: impl Into<String>,
        config_identity: impl Into<String>,
        data_identity: impl Into<String>,
        seed_identity: impl Into<String>,
        admitted_cases: Vec<AdmittedCase>,
        toolchain_channel: &'static str,
        toolchain_features: &'static str,
    ) -> Result<Self, EvalError> {
        let envelope = Self {
            provenance_contract: PROVENANCE_ENVELOPE_CONTRACT,
            arm,
            split,
            code_identity: code_identity.into(),
            config_identity: config_identity.into(),
            data_identity: data_identity.into(),
            seed_identity: seed_identity.into(),
            admitted_cases,
            toolchain_channel,
            toolchain_features,
            compiler_identity: BUILD_RUSTC_VERSION,
        };
        validate_provenance_envelope(&envelope)?;
        Ok(envelope)
    }

    /// Authoritative Stage-C envelope derived from the actual run configuration
    /// and the exact admitted population (issue #690).
    pub fn for_pinned_stage_c_run(
        config: &EvaluatorConfig,
        admitted_cases: Vec<AdmittedCase>,
    ) -> Result<Self, EvalError> {
        let envelope = Self::for_non_final_run(
            config.arm,
            config.split,
            stage_c_code_identity_bundle(config.arm),
            stage_c_config_identity_bundle(config),
            stage_c_data_identity_bundle(config.split, &admitted_cases),
            stage_c_seed_identity(config.split, &admitted_cases),
            admitted_cases,
            build_rustc_release(),
            PROVENANCE_TOOLCHAIN_FEATURES,
        )?;
        validate_provenance_binding(&envelope, config)?;
        Ok(envelope)
    }

    /// Combined toolchain identity token `{channel}/{features}`.
    #[must_use]
    pub fn toolchain_identity(&self) -> String {
        format!("{}/{}", self.toolchain_channel, self.toolchain_features)
    }
}

/// Release token of the build compiler, e.g. `1.97.1` from
/// `rustc 1.97.1 (hash date)`; `unavailable` when it cannot be parsed.
#[must_use]
pub fn build_rustc_release() -> &'static str {
    parse_rustc_release(BUILD_RUSTC_VERSION).unwrap_or("unavailable")
}

/// Parse the release token of a `rustc --version` line.
#[must_use]
pub fn parse_rustc_release(version: &str) -> Option<&str> {
    let mut parts = version.split_whitespace();
    if parts.next() != Some("rustc") {
        return None;
    }
    let release = parts.next()?;
    let numeric = release.split('-').next()?;
    let mut fields = 0_usize;
    for field in numeric.split('.') {
        if field.is_empty() || !field.bytes().all(|byte| byte.is_ascii_digit()) {
            return None;
        }
        fields += 1;
    }
    (fields == 3).then_some(release)
}

fn fnv1a64_update(mut hash: u64, bytes: &[u8]) -> u64 {
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

const FNV1A64_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;

/// FNV-1a 64 digest of the ordered TDI-24 evaluator source surface.
#[must_use]
pub fn stage_c_source_digest() -> String {
    let mut hash = FNV1A64_OFFSET;
    for (path, text) in PROVENANCE_SOURCE_FILES {
        hash = fnv1a64_update(hash, path.as_bytes());
        hash = fnv1a64_update(hash, &[0]);
        hash = fnv1a64_update(hash, &(text.len() as u64).to_le_bytes());
        hash = fnv1a64_update(hash, text.as_bytes());
    }
    format!("{hash:016x}")
}

fn validate_identity_string(field: &'static str, value: &str) -> Result<(), EvalError> {
    if value.is_empty() {
        return Err(EvalError::ProvenanceEnvelopeInvalid {
            reason: match field {
                "code_identity" => "empty_code_identity",
                "config_identity" => "empty_config_identity",
                "data_identity" => "empty_data_identity",
                "seed_identity" => "empty_seed_identity",
                _ => "empty_identity",
            },
        });
    }
    if value.len() > MAX_PROVENANCE_IDENTITY_BYTES {
        return Err(EvalError::ProvenanceEnvelopeInvalid {
            reason: "identity_too_long",
        });
    }
    if value.chars().any(|ch| ch.is_control()) {
        return Err(EvalError::ProvenanceEnvelopeInvalid {
            reason: "identity_control_char",
        });
    }
    Ok(())
}

fn validate_admitted_case(split: DataSplit, case: &AdmittedCase) -> Result<(), EvalError> {
    if case.seed.domain != SeedDomain::from_split(split) {
        return Err(EvalError::ProvenanceEnvelopeInvalid {
            reason: "seed_domain_split_mismatch",
        });
    }
    if case.seed.registry_contract != SEED_REGISTRY_CONTRACT {
        return Err(EvalError::ProvenanceEnvelopeInvalid {
            reason: "seed_registry_contract_missing",
        });
    }
    if case.seed != register_seed(SeedDomain::from_split(split), case.family, case.group_id) {
        return Err(EvalError::ProvenanceEnvelopeInvalid {
            reason: "case_seed_mismatch",
        });
    }
    if case.canonical_digest.len() != 16
        || !case
            .canonical_digest
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(EvalError::ProvenanceEnvelopeInvalid {
            reason: "malformed_case_digest",
        });
    }
    Ok(())
}

/// Validate a provenance envelope against the closed contract and identity rules.
///
/// Recomputes the code, data and seed identities from the compiled source and
/// the admitted population, and requires the actual build compiler. The
/// configuration identity is checked against a concrete configuration by
/// [`validate_provenance_binding`].
pub fn validate_provenance_envelope(envelope: &ProvenanceEnvelope) -> Result<(), EvalError> {
    validate_non_final_split(envelope.split)?;
    if envelope.provenance_contract != PROVENANCE_ENVELOPE_CONTRACT {
        return Err(EvalError::ProvenanceEnvelopeInvalid {
            reason: "contract_drift",
        });
    }
    validate_identity_string("code_identity", &envelope.code_identity)?;
    validate_identity_string("config_identity", &envelope.config_identity)?;
    validate_identity_string("data_identity", &envelope.data_identity)?;
    validate_identity_string("seed_identity", &envelope.seed_identity)?;
    if envelope.toolchain_channel.is_empty() {
        return Err(EvalError::ProvenanceEnvelopeInvalid {
            reason: "empty_toolchain_channel",
        });
    }
    if envelope.toolchain_features.is_empty() {
        return Err(EvalError::ProvenanceEnvelopeInvalid {
            reason: "empty_toolchain_features",
        });
    }
    if parse_rustc_release(envelope.compiler_identity).is_none() {
        return Err(EvalError::ProvenanceEnvelopeInvalid {
            reason: "compiler_identity_unavailable",
        });
    }
    if envelope.compiler_identity != BUILD_RUSTC_VERSION
        || envelope.toolchain_channel != build_rustc_release()
        || envelope.toolchain_features != PROVENANCE_TOOLCHAIN_FEATURES
    {
        return Err(EvalError::ProvenanceEnvelopeInvalid {
            reason: "toolchain_drift",
        });
    }
    if envelope.code_identity != stage_c_code_identity_bundle(envelope.arm) {
        return Err(EvalError::ProvenanceEnvelopeInvalid {
            reason: "code_identity_drift",
        });
    }
    if !envelope
        .config_identity
        .starts_with(PROVENANCE_CONFIG_IDENTITY_CONTRACT)
    {
        return Err(EvalError::ProvenanceEnvelopeInvalid {
            reason: "config_identity_contract_missing",
        });
    }
    if envelope.admitted_cases.is_empty() {
        return Err(EvalError::ProvenanceEnvelopeInvalid {
            reason: "empty_population",
        });
    }
    if envelope.admitted_cases.len() as u64 > MAX_CASES_PER_RUN {
        return Err(EvalError::ProvenanceEnvelopeInvalid {
            reason: "population_exceeds_case_cap",
        });
    }
    for (index, case) in envelope.admitted_cases.iter().enumerate() {
        validate_admitted_case(envelope.split, case)?;
        if envelope.admitted_cases[..index]
            .iter()
            .any(|earlier| earlier.family == case.family && earlier.case_id == case.case_id)
        {
            return Err(EvalError::ProvenanceEnvelopeInvalid {
                reason: "duplicate_admitted_case",
            });
        }
    }
    if envelope.data_identity
        != stage_c_data_identity_bundle(envelope.split, &envelope.admitted_cases)
    {
        return Err(EvalError::ProvenanceEnvelopeInvalid {
            reason: "data_identity_drift",
        });
    }
    if envelope.seed_identity != stage_c_seed_identity(envelope.split, &envelope.admitted_cases) {
        return Err(EvalError::ProvenanceEnvelopeInvalid {
            reason: "seed_identity_drift",
        });
    }
    Ok(())
}

/// Validate an envelope and bind it to one concrete evaluator configuration:
/// arm, split, the complete canonical configuration and the case budget.
pub fn validate_provenance_binding(
    envelope: &ProvenanceEnvelope,
    config: &EvaluatorConfig,
) -> Result<(), EvalError> {
    validate_provenance_envelope(envelope)?;
    if envelope.arm != config.arm {
        return Err(EvalError::ProvenanceEnvelopeInvalid {
            reason: "arm_mismatch",
        });
    }
    if envelope.split != config.split {
        return Err(EvalError::ProvenanceEnvelopeInvalid {
            reason: "split_mismatch",
        });
    }
    if envelope.config_identity != stage_c_config_identity_bundle(config) {
        return Err(EvalError::ProvenanceEnvelopeInvalid {
            reason: "config_identity_mismatch",
        });
    }
    if envelope.admitted_cases.len() as u64 > config.budget.max_cases {
        return Err(EvalError::ProvenanceEnvelopeInvalid {
            reason: "population_exceeds_budget",
        });
    }
    Ok(())
}

/// Check one inference view against an admitted population before scoring.
fn admit_view_against_population(
    admitted: &[AdmittedCase],
    view: &InferenceView,
) -> Result<(), EvalError> {
    let Some(entry) = admitted
        .iter()
        .find(|case| case.family == view.family && case.case_id == view.case_id)
    else {
        return Err(EvalError::ProvenanceEnvelopeInvalid {
            reason: "case_not_admitted",
        });
    };
    if entry.group_id != view.group_id {
        return Err(EvalError::ProvenanceEnvelopeInvalid {
            reason: "case_seed_mismatch",
        });
    }
    if entry.seed
        != register_seed(
            SeedDomain::from_split(view.split),
            view.family,
            view.group_id,
        )
    {
        return Err(EvalError::ProvenanceEnvelopeInvalid {
            reason: "case_seed_mismatch",
        });
    }
    if entry.canonical_digest != canonicalize_inference_view(view).digest {
        return Err(EvalError::ProvenanceEnvelopeInvalid {
            reason: "case_population_mismatch",
        });
    }
    Ok(())
}

/// Canonical identity of the complete evaluator configuration.
///
/// Renders arm, split, every budget field, the envelope/budget/arm contracts,
/// the metric registry with its ordered diagnostics, and the Stage-C matcher,
/// uncertainty, failure and provenance contracts in a stable order. Does not
/// invent freeze pins.
#[must_use]
pub fn stage_c_config_identity_bundle(config: &EvaluatorConfig) -> String {
    let secondaries = config
        .metric_registry
        .secondaries
        .iter()
        .map(|diagnostic| diagnostic.as_str())
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "{PROVENANCE_CONFIG_IDENTITY_CONTRACT};arm={};arm_contract={};split={};envelope={};\
         budget_contract={};max_cases={};max_readout_scalars_per_case={};updates={};\
         registry={};primary={};secondaries=[{}];registry_non_final={};matchers={}|{}|{};\
         uncertainty={};failure_taxonomy={};provenance={}",
        config.arm.as_str(),
        config.arm.evaluator_contract(),
        config.split.as_str(),
        config.envelope_contract,
        config.budget.contract,
        config.budget.max_cases,
        config.budget.max_readout_scalars_per_case,
        config.budget.updates,
        config.metric_registry.registry_contract,
        config.metric_registry.primary.as_str(),
        secondaries,
        config.metric_registry.experimental_non_final,
        PARAMETER_COUNT_MATCHER_CONTRACT,
        INITIALIZATION_MATCHER_CONTRACT,
        OPTIMIZER_UPDATE_BUDGET_CONTRACT,
        PAIRED_UNCERTAINTY_CONTRACT,
        FAILURE_TAXONOMY_CONTRACT,
        PROVENANCE_ENVELOPE_CONTRACT,
    )
}

/// Code identity: exact source-content digest of the TDI-24 evaluator surface
/// compiled into this crate, plus the envelope and arm contracts.
#[must_use]
pub fn stage_c_code_identity_bundle(arm: EvalArm) -> String {
    format!(
        "{}:{}|files={}|{}|{}|{}",
        PROVENANCE_SOURCE_DIGEST_CONTRACT,
        stage_c_source_digest(),
        PROVENANCE_SOURCE_FILES.len(),
        EVALUATOR_ENVELOPE_CONTRACT,
        arm.evaluator_contract(),
        PROVENANCE_ENVELOPE_CONTRACT
    )
}

/// Data identity: split manifest + canonicalization contracts and a digest of
/// the exact ordered admitted population.
#[must_use]
pub fn stage_c_data_identity_bundle(split: DataSplit, admitted: &[AdmittedCase]) -> String {
    let mut hash = FNV1A64_OFFSET;
    for case in admitted {
        hash = fnv1a64_update(hash, case.canonical_line().as_bytes());
    }
    format!(
        "{}|{}|{}|split={}|cases={}|population={:016x}",
        PROVENANCE_POPULATION_CONTRACT,
        SPLIT_MANIFEST_CONTRACT,
        DATASET_CANONICALIZATION_CONTRACT,
        split.as_str(),
        admitted.len(),
        hash
    )
}

/// Seed identity: per-case binding of every admitted case's family and
/// `group_id` to its slice-18 registered seed. No new seed material.
#[must_use]
pub fn stage_c_seed_identity(split: DataSplit, admitted: &[AdmittedCase]) -> String {
    let mut hash = FNV1A64_OFFSET;
    for case in admitted {
        let line = format!(
            "{}:{:016x}:{:016x}:{:016x}\n",
            case.family.as_str(),
            case.group_id,
            case.seed.local_seed,
            case.seed.mixed_seed
        );
        hash = fnv1a64_update(hash, line.as_bytes());
    }
    format!(
        "{}|{}|domain={}|cases={}|case_seeds={:016x}",
        SEED_REGISTRY_CONTRACT,
        PROVENANCE_CASE_SEED_CONTRACT,
        SeedDomain::from_split(split).as_str(),
        admitted.len(),
        hash
    )
}

/// Canonical ordered Phase-B families exercised by the Stage-C preflight.
pub const STAGE_C_PREFLIGHT_FAMILIES: &[TaskFamily] = &[
    TaskFamily::ReflectionDiscriminative,
    TaskFamily::ReflectionNuisance,
    TaskFamily::DirectionReversal,
    TaskFamily::NonChiralControl,
];

/// Members emitted per declared pair (mirrored / reflected / reversed / nuisance twin).
pub const STAGE_C_PREFLIGHT_MEMBERS_PER_PAIR: u64 = 2;

/// Maximum pairs per family so one preflight never exceeds [`MAX_CASES_PER_RUN`].
///
/// `4 families × 2 members × 8 pairs = 64` cases per arm.
pub const MAX_PREFLIGHT_PAIRS_PER_FAMILY: u64 = MAX_CASES_PER_RUN / 8;

/// Bounded Stage-C preflight budget: pairs per family plus the first pair id.
///
/// Software smoke budget only — not a freeze pin and not a confirmatory
/// population. Every case is derived from already-landed Phase-B generators.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StageCPreflightBudget {
    /// Pairs generated per Phase-B family (`1..=MAX_PREFLIGHT_PAIRS_PER_FAMILY`).
    pub pairs_per_family: u64,
    /// First deterministic pair id (registered local seed for provenance).
    pub first_pair_id: u64,
    /// Preflight contract pin.
    pub preflight_contract: &'static str,
}

impl StageCPreflightBudget {
    /// Construct a bounded preflight budget under the versioned contract.
    #[must_use]
    pub const fn bounded(pairs_per_family: u64, first_pair_id: u64) -> Self {
        Self {
            pairs_per_family,
            first_pair_id,
            preflight_contract: STAGE_C_PREFLIGHT_CONTRACT,
        }
    }

    /// Cases evaluated per arm under this budget; fail-closed when unbounded.
    pub fn cases_per_arm(self) -> Result<u64, EvalError> {
        validate_stage_c_preflight_budget(self)?;
        Ok(self.pairs_per_family
            * STAGE_C_PREFLIGHT_MEMBERS_PER_PAIR
            * STAGE_C_PREFLIGHT_FAMILIES.len() as u64)
    }
}

/// Validate a preflight budget: contract pin, non-empty, capped, no id overflow.
pub fn validate_stage_c_preflight_budget(budget: StageCPreflightBudget) -> Result<(), EvalError> {
    if budget.preflight_contract != STAGE_C_PREFLIGHT_CONTRACT {
        return Err(EvalError::StageCPreflightInvalid {
            reason: "contract_drift",
        });
    }
    if budget.pairs_per_family == 0 {
        return Err(EvalError::StageCPreflightInvalid {
            reason: "empty_budget",
        });
    }
    if budget.pairs_per_family > MAX_PREFLIGHT_PAIRS_PER_FAMILY {
        return Err(EvalError::StageCPreflightInvalid {
            reason: "budget_exceeds_case_cap",
        });
    }
    if budget
        .first_pair_id
        .checked_add(budget.pairs_per_family)
        .is_none()
    {
        return Err(EvalError::StageCPreflightInvalid {
            reason: "pair_id_overflow",
        });
    }
    Ok(())
}

/// Immutable report of one bounded Stage-C preflight on one non-final split.
///
/// Exercises slices 21–29 end-to-end (V6/C6 evaluators, parameter-count,
/// initialization and optimizer/update-budget matchers, pinned metric
/// registry, paired uncertainty, failure taxonomy and provenance envelopes) on
/// synthetic Development/Validation cases. Never trains, never touches a
/// protected/final population and never authorises a scientific claim.
#[derive(Clone, Debug, PartialEq)]
pub struct StageCPreflightReport {
    pub preflight_contract: &'static str,
    pub split: DataSplit,
    pub budget: StageCPreflightBudget,
    pub cases_per_arm: u64,
    pub matched_parameters: MatchedParameterCount,
    pub matched_initialization: MatchedInitialization,
    pub matched_optimizer_budget: MatchedOptimizerUpdateBudget,
    pub metric_registry: MetricRegistry,
    pub v6_provenance: ProvenanceEnvelope,
    pub c6_provenance: ProvenanceEnvelope,
    pub v6_records: Vec<EvalRecord>,
    pub c6_records: Vec<EvalRecord>,
    pub v6_failures: FailureLedger,
    pub c6_failures: FailureLedger,
    /// Present only when every case scored on both arms (no retained failure).
    pub paired_summary: Option<PairedEffectSummary>,
    /// Must remain false: the preflight has no protected/final access path.
    pub protected_or_final_access: bool,
    /// Must remain false: reference arms are non-trained.
    pub training_executed: bool,
    /// Must remain false: smoke qualification only.
    pub scientific_claim: bool,
    /// Must remain true: experimental and non-final only.
    pub experimental_non_final: bool,
}

/// Retain an evaluator outcome (scored or failed) without silently dropping it.
fn retain_preflight_outcome(
    outcome: Result<&EvalRecord, EvalError>,
    arm: EvalArm,
    case_id: u64,
    ledger: &mut FailureLedger,
) -> Result<(), EvalError> {
    match outcome {
        Ok(record) => {
            if let Some(failure) = retain_from_eval_record(record)? {
                ledger.retain(failure)?;
            }
            Ok(())
        }
        Err(EvalError::ProtectedOrFinalSplit) => Err(EvalError::ProtectedOrFinalSplit),
        // Provenance/admission violations abort the run (issue #690): an
        // unadmitted case is never converted into a retained failure.
        Err(error @ EvalError::ProvenanceEnvelopeInvalid { .. }) => Err(error),
        Err(error) => {
            ledger.retain_eval_error(&error, arm, Some(case_id))?;
            Ok(())
        }
    }
}

/// Evaluate one sealed case on both matched arms in identical order.
fn evaluate_preflight_pair<T, S>(
    case: &LabeledCase<T>,
    oracle_sign: S,
    v6_run: &mut EvaluatorRun,
    c6_run: &mut EvaluatorRun,
    v6_failures: &mut FailureLedger,
    c6_failures: &mut FailureLedger,
) -> Result<(), EvalError>
where
    S: Fn(&T) -> Result<i8, EvalError>,
{
    let case_id = case.inference_view().case_id;
    retain_preflight_outcome(
        v6_run.evaluate_v6_binary(case, &oracle_sign),
        EvalArm::V6,
        case_id,
        v6_failures,
    )?;
    retain_preflight_outcome(
        c6_run.evaluate_c6_binary(case, &oracle_sign),
        EvalArm::C6,
        case_id,
        c6_failures,
    )
}

const fn preflight_generation_failed() -> EvalError {
    EvalError::StageCPreflightInvalid {
        reason: "case_generation_failed",
    }
}

/// One sealed Stage-C preflight case, tagged by family-specific oracle.
enum PreflightCase {
    Discriminative(LabeledCase<HandednessTarget>),
    Nuisance(LabeledCase<ReflectionInvariantTarget>),
    Direction(LabeledCase<DirectionTarget>),
    NonChiral(LabeledCase<NonChiralTarget>),
}

impl PreflightCase {
    fn inference_view(&self) -> &InferenceView {
        match self {
            Self::Discriminative(case) => case.inference_view(),
            Self::Nuisance(case) => case.inference_view(),
            Self::Direction(case) => case.inference_view(),
            Self::NonChiral(case) => case.inference_view(),
        }
    }
}

/// Generate the exact ordered preflight population from landed Phase-B generators.
fn stage_c_preflight_population(
    split: DataSplit,
    budget: StageCPreflightBudget,
) -> Result<Vec<PreflightCase>, EvalError> {
    validate_stage_c_preflight_budget(budget)?;
    let mut cases = Vec::new();
    let last_pair_id = budget.first_pair_id + budget.pairs_per_family;
    for pair_id in budget.first_pair_id..last_pair_id {
        let discriminative = reflection_discriminative_pair_in_split(pair_id, split)
            .map_err(|_| preflight_generation_failed())?;
        for member in [discriminative.right, discriminative.left] {
            cases.push(PreflightCase::Discriminative(
                seal_reflection_discriminative(&member),
            ));
        }
        let nuisance = reflection_nuisance_pair_in_split(pair_id, split)
            .map_err(|_| preflight_generation_failed())?;
        for member in [nuisance.canonical, nuisance.reflected] {
            cases.push(PreflightCase::Nuisance(seal_reflection_nuisance(&member)));
        }
        let direction = direction_reversal_pair_in_split(pair_id, split)
            .map_err(|_| preflight_generation_failed())?;
        for member in [direction.forward, direction.reverse] {
            cases.push(PreflightCase::Direction(seal_direction_reversal(&member)));
        }
        let base_case_id = pair_id
            .checked_mul(STAGE_C_PREFLIGHT_MEMBERS_PER_PAIR)
            .ok_or_else(preflight_generation_failed)?;
        for case_id in [base_case_id, base_case_id + 1] {
            let control = non_chiral_control_case_in_split(case_id, split)
                .map_err(|_| preflight_generation_failed())?;
            cases.push(PreflightCase::NonChiral(seal_non_chiral_control(&control)));
        }
    }
    Ok(cases)
}

/// Admitted population (canonical digests and per-case seeds) of one
/// bounded Stage-C preflight, in evaluation order (issue #690).
pub fn stage_c_preflight_admitted_cases(
    split: DataSplit,
    budget: StageCPreflightBudget,
) -> Result<Vec<AdmittedCase>, EvalError> {
    validate_non_final_split(split)?;
    Ok(stage_c_preflight_population(split, budget)?
        .iter()
        .map(|case| AdmittedCase::from_view(case.inference_view()))
        .collect())
}

/// Expected Stage-C preflight configuration for one arm (`max_cases = cases_per_arm`).
pub fn stage_c_preflight_config(
    split: DataSplit,
    arm: EvalArm,
    cases_per_arm: u64,
) -> Result<EvaluatorConfig, EvalError> {
    let mut config = match arm {
        EvalArm::V6 => EvaluatorConfig::v6(split)?,
        EvalArm::C6 => EvaluatorConfig::c6(split)?,
    };
    config.budget.max_cases = cases_per_arm;
    Ok(config)
}

/// Run one bounded Stage-C preflight smoke campaign on a non-final split.
///
/// Deterministic; no RNG, no training, no protected/final population. The
/// initialization / ordering seed reuses the Slice-18 registered seed for
/// `(split domain, ReflectionDiscriminative, first_pair_id)`; each admitted
/// case carries its own registered seed in the provenance envelope (issue
/// #690). No new seed material and no freeze pin is invented.
pub fn run_stage_c_preflight(
    split: DataSplit,
    budget: StageCPreflightBudget,
) -> Result<StageCPreflightReport, EvalError> {
    validate_non_final_split(split)?;
    let cases_per_arm = budget.cases_per_arm()?;

    let registered = register_seed(
        SeedDomain::from_split(split),
        STAGE_C_PREFLIGHT_FAMILIES[0],
        budget.first_pair_id,
    );
    let matched_parameters = match_parameter_counts(
        TrainableCapacity::reference_v6(),
        TrainableCapacity::reference_c6(),
    )?;
    let matched_initialization = match_initialization(
        InitializationPolicy::reference_v6(registered.mixed_seed),
        InitializationPolicy::reference_c6(registered.mixed_seed),
    )?;
    let matched_optimizer_budget = match_optimizer_update_budgets(
        OptimizerUpdateBudget::reference_v6(cases_per_arm, registered.mixed_seed),
        OptimizerUpdateBudget::reference_c6(cases_per_arm, registered.mixed_seed),
    )?;
    let metric_registry = MetricRegistry::pinned();
    validate_metric_registry(&metric_registry)?;

    // Admit the exact population before any case is scored.
    let population = stage_c_preflight_population(split, budget)?;
    let admitted: Vec<AdmittedCase> = population
        .iter()
        .map(|case| AdmittedCase::from_view(case.inference_view()))
        .collect();
    if admitted.len() as u64 != cases_per_arm {
        return Err(EvalError::StageCPreflightInvalid {
            reason: "population_size_mismatch",
        });
    }
    let v6_config = stage_c_preflight_config(split, EvalArm::V6, cases_per_arm)?;
    let c6_config = stage_c_preflight_config(split, EvalArm::C6, cases_per_arm)?;
    let v6_provenance = ProvenanceEnvelope::for_pinned_stage_c_run(&v6_config, admitted.clone())?;
    let c6_provenance = ProvenanceEnvelope::for_pinned_stage_c_run(&c6_config, admitted)?;
    let mut v6_run = EvaluatorRun::open_with_provenance(v6_config, v6_provenance.clone())?;
    let mut c6_run = EvaluatorRun::open_with_provenance(c6_config, c6_provenance.clone())?;
    let mut v6_failures = FailureLedger::open(split)?;
    let mut c6_failures = FailureLedger::open(split)?;

    for case in &population {
        match case {
            PreflightCase::Discriminative(sealed) => evaluate_preflight_pair(
                sealed,
                handedness_sign,
                &mut v6_run,
                &mut c6_run,
                &mut v6_failures,
                &mut c6_failures,
            )?,
            PreflightCase::Nuisance(sealed) => evaluate_preflight_pair(
                sealed,
                reflection_invariant_sign,
                &mut v6_run,
                &mut c6_run,
                &mut v6_failures,
                &mut c6_failures,
            )?,
            PreflightCase::Direction(sealed) => evaluate_preflight_pair(
                sealed,
                direction_sign,
                &mut v6_run,
                &mut c6_run,
                &mut v6_failures,
                &mut c6_failures,
            )?,
            PreflightCase::NonChiral(sealed) => evaluate_preflight_pair(
                sealed,
                non_chiral_sign,
                &mut v6_run,
                &mut c6_run,
                &mut v6_failures,
                &mut c6_failures,
            )?,
        }
    }

    let v6_records = v6_run.records().to_vec();
    let c6_records = c6_run.records().to_vec();
    let paired_summary = if v6_failures.records().is_empty() && c6_failures.records().is_empty() {
        Some(summarize_paired_uncertainty_from_records(
            split,
            &v6_records,
            &c6_records,
            &metric_registry,
        )?)
    } else {
        None
    };

    let report = StageCPreflightReport {
        preflight_contract: STAGE_C_PREFLIGHT_CONTRACT,
        split,
        budget,
        cases_per_arm,
        matched_parameters,
        matched_initialization,
        matched_optimizer_budget,
        metric_registry,
        v6_provenance,
        c6_provenance,
        v6_records,
        c6_records,
        v6_failures,
        c6_failures,
        paired_summary,
        protected_or_final_access: false,
        training_executed: false,
        scientific_claim: false,
        experimental_non_final: true,
    };
    validate_stage_c_preflight_report(&report)?;
    Ok(report)
}

/// Parse a split label and run the preflight; protected/final labels fail closed
/// before any case is generated.
pub fn run_stage_c_preflight_for_label(
    split_label: &str,
    budget: StageCPreflightBudget,
) -> Result<StageCPreflightReport, EvalError> {
    let split = parse_non_final_split(split_label)?;
    run_stage_c_preflight(split, budget)
}

fn validate_preflight_arm(
    split: DataSplit,
    arm: EvalArm,
    cases_per_arm: u64,
    records: &[EvalRecord],
    failures: &FailureLedger,
    provenance: &ProvenanceEnvelope,
) -> Result<(), EvalError> {
    validate_provenance_envelope(provenance)?;
    if provenance.arm != arm || provenance.split != split {
        return Err(EvalError::StageCPreflightInvalid {
            reason: "provenance_binding_mismatch",
        });
    }
    // Issue #690: the envelope must bind the exact preflight configuration
    // and an admitted population of exactly `cases_per_arm` cases.
    let expected_config = stage_c_preflight_config(split, arm, cases_per_arm)?;
    validate_provenance_binding(provenance, &expected_config).map_err(|_| {
        EvalError::StageCPreflightInvalid {
            reason: "provenance_config_mismatch",
        }
    })?;
    if provenance.admitted_cases.len() as u64 != cases_per_arm {
        return Err(EvalError::StageCPreflightInvalid {
            reason: "population_size_mismatch",
        });
    }
    if failures.split() != split || failures.taxonomy_contract() != FAILURE_TAXONOMY_CONTRACT {
        return Err(EvalError::StageCPreflightInvalid {
            reason: "failure_ledger_binding_mismatch",
        });
    }
    let records_len = records.len() as u64;
    if records_len > cases_per_arm {
        return Err(EvalError::StageCPreflightInvalid {
            reason: "case_budget_exceeded",
        });
    }
    // Every attempted case is either an emitted record or a retained failure.
    let emitted_failures = records
        .iter()
        .filter(|record| matches!(record.outcome, EvalOutcome::Failure(_)))
        .count() as u64;
    let hard_failures = (failures.records().len() as u64)
        .checked_sub(emitted_failures)
        .ok_or(EvalError::StageCPreflightInvalid {
            reason: "failure_not_retained",
        })?;
    if records_len + hard_failures != cases_per_arm {
        return Err(EvalError::StageCPreflightInvalid {
            reason: "unaccounted_cases",
        });
    }
    // Records bind injectively and in admission order to the full admitted
    // identity (family, case, group, canonical digest); a hard failure may
    // only skip an admitted case, never duplicate or reorder one.
    let mut next_admitted = 0usize;
    for record in records {
        validate_eval_record_contracts(record)?;
        if record.arm != arm || record.split != split {
            return Err(EvalError::StageCPreflightInvalid {
                reason: "record_binding_mismatch",
            });
        }
        let Some(offset) = provenance.admitted_cases[next_admitted..]
            .iter()
            .position(|case| case.family == record.family && case.case_id == record.case_id)
        else {
            return Err(EvalError::StageCPreflightInvalid {
                reason: "record_not_admitted",
            });
        };
        let admitted = &provenance.admitted_cases[next_admitted + offset];
        if admitted.group_id != record.group_id
            || admitted.canonical_digest != record.canonical_digest
        {
            return Err(EvalError::StageCPreflightInvalid {
                reason: "record_identity_mismatch",
            });
        }
        next_admitted += offset + 1;
    }
    for failure in failures.records() {
        validate_failure_record(failure)?;
        if failure.arm != arm {
            return Err(EvalError::StageCPreflightInvalid {
                reason: "failure_arm_mismatch",
            });
        }
        let unadmitted = failure.case_id.is_some_and(|case_id| {
            !provenance
                .admitted_cases
                .iter()
                .any(|case| case.case_id == case_id)
        });
        if unadmitted {
            return Err(EvalError::StageCPreflightInvalid {
                reason: "failure_not_admitted",
            });
        }
    }
    Ok(())
}

/// Validate a preflight report: contract pin, non-authorising flags, matched
/// budgets, per-arm provenance/accounting and paired-summary consistency.
pub fn validate_stage_c_preflight_report(report: &StageCPreflightReport) -> Result<(), EvalError> {
    validate_non_final_split(report.split)?;
    if report.preflight_contract != STAGE_C_PREFLIGHT_CONTRACT {
        return Err(EvalError::StageCPreflightInvalid {
            reason: "contract_drift",
        });
    }
    if report.protected_or_final_access {
        return Err(EvalError::StageCPreflightInvalid {
            reason: "protected_or_final_access_forbidden",
        });
    }
    if report.training_executed {
        return Err(EvalError::StageCPreflightInvalid {
            reason: "training_forbidden",
        });
    }
    if report.scientific_claim {
        return Err(EvalError::StageCPreflightInvalid {
            reason: "scientific_claim_forbidden",
        });
    }
    if !report.experimental_non_final {
        return Err(EvalError::StageCPreflightInvalid {
            reason: "experimental_non_final_required",
        });
    }
    if report.budget.cases_per_arm()? != report.cases_per_arm {
        return Err(EvalError::StageCPreflightInvalid {
            reason: "case_count_mismatch",
        });
    }
    if report.cases_per_arm > MAX_CASES_PER_RUN {
        return Err(EvalError::StageCPreflightInvalid {
            reason: "budget_exceeds_case_cap",
        });
    }
    if report.matched_parameters.matcher_contract != PARAMETER_COUNT_MATCHER_CONTRACT
        || report.matched_initialization.matcher_contract != INITIALIZATION_MATCHER_CONTRACT
        || report.matched_optimizer_budget.matcher_contract != OPTIMIZER_UPDATE_BUDGET_CONTRACT
    {
        return Err(EvalError::StageCPreflightInvalid {
            reason: "matcher_contract_drift",
        });
    }
    if report.matched_optimizer_budget.examples != report.cases_per_arm
        || report.matched_optimizer_budget.updates != NON_TRAINED_UPDATE_BUDGET
        || report.matched_parameters.trainable_parameters != 0
    {
        return Err(EvalError::StageCPreflightInvalid {
            reason: "non_trained_budget_mismatch",
        });
    }
    if report.metric_registry != MetricRegistry::pinned() {
        return Err(EvalError::StageCPreflightInvalid {
            reason: "registry_not_pinned",
        });
    }
    validate_preflight_arm(
        report.split,
        EvalArm::V6,
        report.cases_per_arm,
        &report.v6_records,
        &report.v6_failures,
        &report.v6_provenance,
    )?;
    validate_preflight_arm(
        report.split,
        EvalArm::C6,
        report.cases_per_arm,
        &report.c6_records,
        &report.c6_failures,
        &report.c6_provenance,
    )?;
    // Config identities differ by arm only; each was bound to its expected
    // arm configuration above. Population and per-case seeds must be paired.
    if report.v6_provenance.seed_identity != report.c6_provenance.seed_identity
        || report.v6_provenance.data_identity != report.c6_provenance.data_identity
        || report.v6_provenance.admitted_cases != report.c6_provenance.admitted_cases
        || report.v6_provenance.code_identity.split('|').next()
            != report.c6_provenance.code_identity.split('|').next()
    {
        return Err(EvalError::StageCPreflightInvalid {
            reason: "unpaired_provenance",
        });
    }
    let failure_free =
        report.v6_failures.records().is_empty() && report.c6_failures.records().is_empty();
    match (&report.paired_summary, failure_free) {
        (Some(summary), true) => {
            if summary.split != report.split
                || summary.n_pairs != report.cases_per_arm
                || summary.uncertainty_contract != PAIRED_UNCERTAINTY_CONTRACT
                || !summary.experimental_non_final
            {
                return Err(EvalError::StageCPreflightInvalid {
                    reason: "paired_summary_mismatch",
                });
            }
        }
        (None, false) => {}
        (Some(_), false) => {
            return Err(EvalError::StageCPreflightInvalid {
                reason: "summary_hides_failures",
            });
        }
        (None, true) => {
            return Err(EvalError::StageCPreflightInvalid {
                reason: "missing_paired_summary",
            });
        }
    }
    Ok(())
}

/// One case scored by the C6 reference and its `gamma=0` ablation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GammaZeroAblationCase {
    pub family: TaskFamily,
    pub case_id: u64,
    pub reference_score: f64,
    pub ablated_score: f64,
    /// Unweighted parity-odd observable `q^T J k`.
    pub parity_odd_channel: f64,
    pub reference_correct: bool,
    pub ablated_correct: bool,
}

/// Per-family correct counts for the reference and the ablation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GammaZeroFamilySummary {
    pub family: TaskFamily,
    pub n_cases: u64,
    pub reference_correct: u64,
    pub ablated_correct: u64,
}

/// Immutable `gamma=0` ablation report on one non-final split.
#[derive(Clone, Debug, PartialEq)]
pub struct GammaZeroAblationReport {
    pub ablation_contract: &'static str,
    pub split: DataSplit,
    pub budget: StageCPreflightBudget,
    pub reference_weights: ChiralScoreWeights,
    pub ablated_weights: ChiralScoreWeights,
    /// Identical capacity on both sides: the ablation removes no parameter.
    pub reference_capacity: TrainableCapacity,
    pub ablated_capacity: TrainableCapacity,
    pub cases: Vec<GammaZeroAblationCase>,
    pub families: Vec<GammaZeroFamilySummary>,
    /// Must remain false.
    pub protected_or_final_access: bool,
    /// Must remain false: both sides are the non-trained reference.
    pub training_executed: bool,
    /// Must remain false: attribution is decided only by the Stage-D audit.
    pub scientific_claim: bool,
    /// Must remain true.
    pub experimental_non_final: bool,
}

const fn gamma_zero_invalid(reason: &'static str) -> EvalError {
    EvalError::GammaZeroAblationInvalid { reason }
}

/// Check the exact per-case identities of the ablation.
fn check_gamma_zero_identities(
    query: Chiral6,
    key: Chiral6,
    reference: ChiralScoreWeights,
    reference_score: f64,
    ablated_score: f64,
) -> Result<f64, EvalError> {
    let channels = observables(query, key).map_err(EvalError::ChiralNumerical)?;
    let ablated = gamma_zero_weights(reference);
    // The parity-odd channel is the only difference between the two scores.
    if reference_score != ablated_score + reference.gamma * channels.chiral {
        return Err(gamma_zero_invalid("parity_odd_residual"));
    }
    // With gamma=0 the right/left enantiomorphic scores coincide exactly.
    let (right, left) =
        enantiomorphic_scores(query, key, ablated).map_err(EvalError::ChiralNumerical)?;
    if right != left || right != ablated_score {
        return Err(gamma_zero_invalid("enantiomorphic_split"));
    }
    Ok(channels.chiral)
}

fn ablate_gamma_zero_case<T, S>(
    case: &LabeledCase<T>,
    split: DataSplit,
    oracle_sign: S,
    cases: &mut Vec<GammaZeroAblationCase>,
) -> Result<(), EvalError>
where
    S: Fn(&T) -> Result<i8, EvalError>,
{
    let view = case.inference_view();
    if view.split != split {
        return Err(EvalError::SplitMismatch {
            expected: split,
            actual: view.split,
        });
    }
    let reference_score = run_inference_callback(case, score_c6_from_view)?;
    let ablated_score = run_inference_callback(case, score_c6_gamma_zero_from_view)?;
    let query = Chiral6::from_array(view.query.as_array()).map_err(EvalError::ChiralNumerical)?;
    let key = Chiral6::from_array(view.key.as_array()).map_err(EvalError::ChiralNumerical)?;
    let parity_odd_channel = check_gamma_zero_identities(
        query,
        key,
        C6_REFERENCE_WEIGHTS,
        reference_score,
        ablated_score,
    )?;
    let sign = oracle_sign(case.protected_label().reveal_for_evaluation())?;
    if sign != 1 && sign != -1 {
        return Err(gamma_zero_invalid("oracle_sign"));
    }
    let sign = f64::from(sign);
    cases.push(GammaZeroAblationCase {
        family: view.family,
        case_id: view.case_id,
        reference_score,
        ablated_score,
        parity_odd_channel,
        reference_correct: reference_score * sign > 0.0,
        ablated_correct: ablated_score * sign > 0.0,
    });
    Ok(())
}

fn summarize_gamma_zero_families(cases: &[GammaZeroAblationCase]) -> Vec<GammaZeroFamilySummary> {
    STAGE_C_PREFLIGHT_FAMILIES
        .iter()
        .map(|family| {
            let members = cases.iter().filter(|case| case.family == *family);
            GammaZeroFamilySummary {
                family: *family,
                n_cases: members.clone().count() as u64,
                reference_correct: members.clone().filter(|c| c.reference_correct).count() as u64,
                ablated_correct: members.filter(|c| c.ablated_correct).count() as u64,
            }
        })
        .collect()
}

/// Run the `gamma=0` ablation on the bounded Stage-C case stream.
///
/// Same generators, pair ids, split and capacity as the Stage-C preflight;
/// only the parity-odd coefficient changes. Any scoring or identity failure
/// aborts fail-closed; nothing is silently dropped.
pub fn run_gamma_zero_ablation(
    split: DataSplit,
    budget: StageCPreflightBudget,
) -> Result<GammaZeroAblationReport, EvalError> {
    validate_non_final_split(split)?;
    let cases_per_arm = budget.cases_per_arm()?;
    let mut cases = Vec::new();
    let last_pair_id = budget.first_pair_id + budget.pairs_per_family;
    for pair_id in budget.first_pair_id..last_pair_id {
        let discriminative = reflection_discriminative_pair_in_split(pair_id, split)
            .map_err(|_| preflight_generation_failed())?;
        for member in [discriminative.right, discriminative.left] {
            ablate_gamma_zero_case(
                &seal_reflection_discriminative(&member),
                split,
                handedness_sign,
                &mut cases,
            )?;
        }
        let nuisance = reflection_nuisance_pair_in_split(pair_id, split)
            .map_err(|_| preflight_generation_failed())?;
        for member in [nuisance.canonical, nuisance.reflected] {
            ablate_gamma_zero_case(
                &seal_reflection_nuisance(&member),
                split,
                reflection_invariant_sign,
                &mut cases,
            )?;
        }
        let direction = direction_reversal_pair_in_split(pair_id, split)
            .map_err(|_| preflight_generation_failed())?;
        for member in [direction.forward, direction.reverse] {
            ablate_gamma_zero_case(
                &seal_direction_reversal(&member),
                split,
                direction_sign,
                &mut cases,
            )?;
        }
        let base_case_id = pair_id
            .checked_mul(STAGE_C_PREFLIGHT_MEMBERS_PER_PAIR)
            .ok_or_else(preflight_generation_failed)?;
        for case_id in [base_case_id, base_case_id + 1] {
            let control = non_chiral_control_case_in_split(case_id, split)
                .map_err(|_| preflight_generation_failed())?;
            ablate_gamma_zero_case(
                &seal_non_chiral_control(&control),
                split,
                non_chiral_sign,
                &mut cases,
            )?;
        }
    }
    if cases.len() as u64 != cases_per_arm {
        return Err(gamma_zero_invalid("case_count"));
    }
    let families = summarize_gamma_zero_families(&cases);
    let report = GammaZeroAblationReport {
        ablation_contract: GAMMA_ZERO_ABLATION_CONTRACT,
        split,
        budget,
        reference_weights: C6_REFERENCE_WEIGHTS,
        ablated_weights: gamma_zero_weights(C6_REFERENCE_WEIGHTS),
        reference_capacity: TrainableCapacity::reference_c6(),
        ablated_capacity: TrainableCapacity::reference_c6(),
        cases,
        families,
        protected_or_final_access: false,
        training_executed: false,
        scientific_claim: false,
        experimental_non_final: true,
    };
    validate_gamma_zero_ablation_report(&report)?;
    Ok(report)
}

/// Parse a split label first; protected/final labels never generate a case.
pub fn run_gamma_zero_ablation_for_label(
    split_label: &str,
    budget: StageCPreflightBudget,
) -> Result<GammaZeroAblationReport, EvalError> {
    run_gamma_zero_ablation(parse_non_final_split(split_label)?, budget)
}

/// Validate a `gamma=0` ablation report: pins, all-else-fixed weights, matched
/// capacity, bounded coverage, exact per-case parity-odd residual, recomputed
/// family counts, and the no-access / no-training / no-claim flags.
pub fn validate_gamma_zero_ablation_report(
    report: &GammaZeroAblationReport,
) -> Result<(), EvalError> {
    validate_non_final_split(report.split)?;
    if report.ablation_contract != GAMMA_ZERO_ABLATION_CONTRACT {
        return Err(gamma_zero_invalid("contract_drift"));
    }
    if report.reference_weights != C6_REFERENCE_WEIGHTS {
        return Err(gamma_zero_invalid("reference_weights_drift"));
    }
    if report.ablated_weights != gamma_zero_weights(report.reference_weights) {
        return Err(gamma_zero_invalid("ablated_weights_drift"));
    }
    if report.reference_capacity != report.ablated_capacity
        || report.reference_capacity != TrainableCapacity::reference_c6()
    {
        return Err(gamma_zero_invalid("capacity_mismatch"));
    }
    let expected = report.budget.cases_per_arm()?;
    if report.cases.len() as u64 != expected {
        return Err(gamma_zero_invalid("case_count"));
    }
    for case in &report.cases {
        if case.reference_score
            != case.ablated_score + report.reference_weights.gamma * case.parity_odd_channel
        {
            return Err(gamma_zero_invalid("parity_odd_residual"));
        }
    }
    if report.families != summarize_gamma_zero_families(&report.cases)
        || report.families.iter().map(|f| f.n_cases).sum::<u64>() != expected
    {
        return Err(gamma_zero_invalid("family_summary_drift"));
    }
    if report.protected_or_final_access {
        return Err(gamma_zero_invalid("protected_or_final_access"));
    }
    if report.training_executed {
        return Err(gamma_zero_invalid("training_executed"));
    }
    if report.scientific_claim {
        return Err(gamma_zero_invalid("scientific_claim"));
    }
    if !report.experimental_non_final {
        return Err(gamma_zero_invalid("experimental_non_final"));
    }
    Ok(())
}

/// One case scored by the C6 reference and its `beta=0` ablation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BetaZeroAblationCase {
    pub family: TaskFamily,
    pub case_id: u64,
    pub reference_score: f64,
    pub ablated_score: f64,
    /// Unweighted mirror-even observable `q^T M k`.
    pub mirror_even_channel: f64,
    pub reference_correct: bool,
    pub ablated_correct: bool,
}

/// Per-family correct counts for the reference and the ablation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BetaZeroFamilySummary {
    pub family: TaskFamily,
    pub n_cases: u64,
    pub reference_correct: u64,
    pub ablated_correct: u64,
}

/// Immutable `beta=0` ablation report on one non-final split.
#[derive(Clone, Debug, PartialEq)]
pub struct BetaZeroAblationReport {
    pub ablation_contract: &'static str,
    pub split: DataSplit,
    pub budget: StageCPreflightBudget,
    pub reference_weights: ChiralScoreWeights,
    pub ablated_weights: ChiralScoreWeights,
    /// True when the matched C6 reference already carries `beta=0`: the
    /// ablation is then the identity, recorded as such rather than hidden.
    pub reference_beta_already_zero: bool,
    /// Identical capacity on both sides: the ablation removes no parameter.
    pub reference_capacity: TrainableCapacity,
    pub ablated_capacity: TrainableCapacity,
    pub cases: Vec<BetaZeroAblationCase>,
    pub families: Vec<BetaZeroFamilySummary>,
    /// Must remain false.
    pub protected_or_final_access: bool,
    /// Must remain false: both sides are the non-trained reference.
    pub training_executed: bool,
    /// Must remain false: attribution is decided only by the Stage-D audit.
    pub scientific_claim: bool,
    /// Must remain true.
    pub experimental_non_final: bool,
}

const fn beta_zero_invalid(reason: &'static str) -> EvalError {
    EvalError::BetaZeroAblationInvalid { reason }
}

/// Check the exact per-case identities of the ablation.
fn check_beta_zero_identities(
    query: Chiral6,
    key: Chiral6,
    reference: ChiralScoreWeights,
    reference_score: f64,
    ablated_score: f64,
) -> Result<f64, EvalError> {
    let channels = observables(query, key).map_err(EvalError::ChiralNumerical)?;
    let ablated = beta_zero_weights(reference);
    // The mirror-even channel is the only difference between the two scores.
    if reference_score != ablated_score + reference.beta * channels.mirrored {
        return Err(beta_zero_invalid("mirror_even_residual"));
    }
    // A reference that already has beta=0 makes the ablation bit-for-bit inert.
    if reference.beta == 0.0 && reference_score.to_bits() != ablated_score.to_bits() {
        return Err(beta_zero_invalid("degenerate_identity"));
    }
    // The parity-odd channel is untouched: the right enantiomorphic branch is
    // the ablated score itself, and the left branch flips only `gamma * chi`.
    let (right, left) =
        enantiomorphic_scores(query, key, ablated).map_err(EvalError::ChiralNumerical)?;
    if right != ablated_score || (ablated.gamma * channels.chiral != 0.0 && right == left) {
        return Err(beta_zero_invalid("parity_odd_channel_lost"));
    }
    Ok(channels.mirrored)
}

fn ablate_beta_zero_case<T, S>(
    case: &LabeledCase<T>,
    split: DataSplit,
    oracle_sign: S,
    cases: &mut Vec<BetaZeroAblationCase>,
) -> Result<(), EvalError>
where
    S: Fn(&T) -> Result<i8, EvalError>,
{
    let view = case.inference_view();
    if view.split != split {
        return Err(EvalError::SplitMismatch {
            expected: split,
            actual: view.split,
        });
    }
    let reference_score = run_inference_callback(case, score_c6_from_view)?;
    let ablated_score = run_inference_callback(case, score_c6_beta_zero_from_view)?;
    let query = Chiral6::from_array(view.query.as_array()).map_err(EvalError::ChiralNumerical)?;
    let key = Chiral6::from_array(view.key.as_array()).map_err(EvalError::ChiralNumerical)?;
    let mirror_even_channel = check_beta_zero_identities(
        query,
        key,
        C6_REFERENCE_WEIGHTS,
        reference_score,
        ablated_score,
    )?;
    let sign = oracle_sign(case.protected_label().reveal_for_evaluation())?;
    if sign != 1 && sign != -1 {
        return Err(beta_zero_invalid("oracle_sign"));
    }
    let sign = f64::from(sign);
    cases.push(BetaZeroAblationCase {
        family: view.family,
        case_id: view.case_id,
        reference_score,
        ablated_score,
        mirror_even_channel,
        reference_correct: reference_score * sign > 0.0,
        ablated_correct: ablated_score * sign > 0.0,
    });
    Ok(())
}

fn summarize_beta_zero_families(cases: &[BetaZeroAblationCase]) -> Vec<BetaZeroFamilySummary> {
    STAGE_C_PREFLIGHT_FAMILIES
        .iter()
        .map(|family| {
            let members = cases.iter().filter(|case| case.family == *family);
            BetaZeroFamilySummary {
                family: *family,
                n_cases: members.clone().count() as u64,
                reference_correct: members.clone().filter(|c| c.reference_correct).count() as u64,
                ablated_correct: members.filter(|c| c.ablated_correct).count() as u64,
            }
        })
        .collect()
}

/// Run the `beta=0` ablation on the bounded Stage-C case stream.
///
/// Same generators, pair ids, split and capacity as the Stage-C preflight;
/// only the mirror-even coefficient is set to zero. The matched C6 reference
/// already uses `beta=0`, so on this stream the ablation is the identity and
/// the report says so explicitly (`reference_beta_already_zero`). Any scoring or identity failure
/// aborts fail-closed; nothing is silently dropped.
pub fn run_beta_zero_ablation(
    split: DataSplit,
    budget: StageCPreflightBudget,
) -> Result<BetaZeroAblationReport, EvalError> {
    validate_non_final_split(split)?;
    let cases_per_arm = budget.cases_per_arm()?;
    let mut cases = Vec::new();
    let last_pair_id = budget.first_pair_id + budget.pairs_per_family;
    for pair_id in budget.first_pair_id..last_pair_id {
        let discriminative = reflection_discriminative_pair_in_split(pair_id, split)
            .map_err(|_| preflight_generation_failed())?;
        for member in [discriminative.right, discriminative.left] {
            ablate_beta_zero_case(
                &seal_reflection_discriminative(&member),
                split,
                handedness_sign,
                &mut cases,
            )?;
        }
        let nuisance = reflection_nuisance_pair_in_split(pair_id, split)
            .map_err(|_| preflight_generation_failed())?;
        for member in [nuisance.canonical, nuisance.reflected] {
            ablate_beta_zero_case(
                &seal_reflection_nuisance(&member),
                split,
                reflection_invariant_sign,
                &mut cases,
            )?;
        }
        let direction = direction_reversal_pair_in_split(pair_id, split)
            .map_err(|_| preflight_generation_failed())?;
        for member in [direction.forward, direction.reverse] {
            ablate_beta_zero_case(
                &seal_direction_reversal(&member),
                split,
                direction_sign,
                &mut cases,
            )?;
        }
        let base_case_id = pair_id
            .checked_mul(STAGE_C_PREFLIGHT_MEMBERS_PER_PAIR)
            .ok_or_else(preflight_generation_failed)?;
        for case_id in [base_case_id, base_case_id + 1] {
            let control = non_chiral_control_case_in_split(case_id, split)
                .map_err(|_| preflight_generation_failed())?;
            ablate_beta_zero_case(
                &seal_non_chiral_control(&control),
                split,
                non_chiral_sign,
                &mut cases,
            )?;
        }
    }
    if cases.len() as u64 != cases_per_arm {
        return Err(beta_zero_invalid("case_count"));
    }
    let families = summarize_beta_zero_families(&cases);
    let report = BetaZeroAblationReport {
        ablation_contract: BETA_ZERO_ABLATION_CONTRACT,
        split,
        budget,
        reference_weights: C6_REFERENCE_WEIGHTS,
        ablated_weights: beta_zero_weights(C6_REFERENCE_WEIGHTS),
        reference_beta_already_zero: C6_REFERENCE_WEIGHTS.beta == 0.0,
        reference_capacity: TrainableCapacity::reference_c6(),
        ablated_capacity: TrainableCapacity::reference_c6(),
        cases,
        families,
        protected_or_final_access: false,
        training_executed: false,
        scientific_claim: false,
        experimental_non_final: true,
    };
    validate_beta_zero_ablation_report(&report)?;
    Ok(report)
}

/// Parse a split label first; protected/final labels never generate a case.
pub fn run_beta_zero_ablation_for_label(
    split_label: &str,
    budget: StageCPreflightBudget,
) -> Result<BetaZeroAblationReport, EvalError> {
    run_beta_zero_ablation(parse_non_final_split(split_label)?, budget)
}

/// Validate a `beta=0` ablation report: pins, all-else-fixed weights, the
/// explicit degeneracy flag, matched capacity, bounded coverage, the exact
/// per-case mirror-even residual (bit-for-bit identity when degenerate),
/// recomputed family counts, and the no-access / no-training / no-claim flags.
pub fn validate_beta_zero_ablation_report(
    report: &BetaZeroAblationReport,
) -> Result<(), EvalError> {
    validate_non_final_split(report.split)?;
    if report.ablation_contract != BETA_ZERO_ABLATION_CONTRACT {
        return Err(beta_zero_invalid("contract_drift"));
    }
    if report.reference_weights != C6_REFERENCE_WEIGHTS {
        return Err(beta_zero_invalid("reference_weights_drift"));
    }
    if report.ablated_weights != beta_zero_weights(report.reference_weights) {
        return Err(beta_zero_invalid("ablated_weights_drift"));
    }
    if report.reference_beta_already_zero != (report.reference_weights.beta == 0.0) {
        return Err(beta_zero_invalid("degeneracy_flag_drift"));
    }
    if report.reference_capacity != report.ablated_capacity
        || report.reference_capacity != TrainableCapacity::reference_c6()
    {
        return Err(beta_zero_invalid("capacity_mismatch"));
    }
    let expected = report.budget.cases_per_arm()?;
    if report.cases.len() as u64 != expected {
        return Err(beta_zero_invalid("case_count"));
    }
    for case in &report.cases {
        if case.reference_score
            != case.ablated_score + report.reference_weights.beta * case.mirror_even_channel
        {
            return Err(beta_zero_invalid("mirror_even_residual"));
        }
        if report.reference_beta_already_zero
            && (case.reference_score.to_bits() != case.ablated_score.to_bits()
                || case.reference_correct != case.ablated_correct)
        {
            return Err(beta_zero_invalid("degenerate_identity"));
        }
    }
    if report.families != summarize_beta_zero_families(&report.cases)
        || report.families.iter().map(|f| f.n_cases).sum::<u64>() != expected
    {
        return Err(beta_zero_invalid("family_summary_drift"));
    }
    if report.protected_or_final_access {
        return Err(beta_zero_invalid("protected_or_final_access"));
    }
    if report.training_executed {
        return Err(beta_zero_invalid("training_executed"));
    }
    if report.scientific_claim {
        return Err(beta_zero_invalid("scientific_claim"));
    }
    if !report.experimental_non_final {
        return Err(beta_zero_invalid("experimental_non_final"));
    }
    Ok(())
}

/// One case scored by the C6 reference, its direct-only collapse and V6.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DirectOnlyCollapseCase {
    pub family: TaskFamily,
    pub case_id: u64,
    pub reference_score: f64,
    pub collapsed_score: f64,
    /// Matched V6 score on the same inference view.
    pub v6_score: f64,
    /// Unweighted mirror-even observable `q^T M k` removed by the collapse.
    pub mirror_even_channel: f64,
    /// Unweighted parity-odd observable `q^T J k` removed by the collapse.
    pub parity_odd_channel: f64,
    pub reference_correct: bool,
    pub collapsed_correct: bool,
    pub v6_correct: bool,
}

/// Per-family correct counts for the reference, the collapse and V6.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DirectOnlyFamilySummary {
    pub family: TaskFamily,
    pub n_cases: u64,
    pub reference_correct: u64,
    pub collapsed_correct: u64,
    pub v6_correct: u64,
}

/// Immutable direct-only collapse report on one non-final split.
#[derive(Clone, Debug, PartialEq)]
pub struct DirectOnlyCollapseReport {
    pub collapse_contract: &'static str,
    pub split: DataSplit,
    pub budget: StageCPreflightBudget,
    pub reference_weights: ChiralScoreWeights,
    pub collapsed_weights: ChiralScoreWeights,
    /// Declared absolute tolerance of the collapsed-vs-V6 match; must be the
    /// exact pin [`DIRECT_ONLY_V6_MATCH_TOLERANCE`] (`0.0`, bit-for-bit).
    pub v6_match_tolerance: f64,
    /// True when the matched C6 reference already carries `beta=0`: the
    /// collapse then has the same weights as the slice-31 `gamma=0` ablation,
    /// recorded as such rather than hidden.
    pub collapse_coincides_with_gamma_zero: bool,
    /// Identical capacity on both C6 sides: the collapse removes no parameter.
    pub reference_capacity: TrainableCapacity,
    pub collapsed_capacity: TrainableCapacity,
    /// Matched V6 capacity the collapsed path is compared against.
    pub v6_capacity: TrainableCapacity,
    pub cases: Vec<DirectOnlyCollapseCase>,
    pub families: Vec<DirectOnlyFamilySummary>,
    /// Must remain false.
    pub protected_or_final_access: bool,
    /// Must remain false: every side is the non-trained reference.
    pub training_executed: bool,
    /// Must remain false: attribution is decided only by the Stage-D audit.
    pub scientific_claim: bool,
    /// Must remain true.
    pub experimental_non_final: bool,
}

const fn direct_only_invalid(reason: &'static str) -> EvalError {
    EvalError::DirectOnlyCollapseInvalid { reason }
}

/// Exact collapsed-vs-V6 comparison under the declared tolerance pin.
fn direct_only_matches_v6(collapsed: f64, v6: f64, tolerance: f64) -> bool {
    if tolerance == 0.0 {
        collapsed.to_bits() == v6.to_bits()
    } else {
        (collapsed - v6).abs() <= tolerance
    }
}

/// Check the exact per-case identities of the collapse.
fn check_direct_only_identities(
    query: Chiral6,
    key: Chiral6,
    reference: ChiralScoreWeights,
    reference_score: f64,
    collapsed_score: f64,
    v6_score: f64,
) -> Result<ChiralObservables, EvalError> {
    let channels = observables(query, key).map_err(EvalError::ChiralNumerical)?;
    let collapsed = direct_only_weights(reference);
    // The two extra channels are the only difference between the C6 scores.
    if reference_score
        != collapsed_score + reference.beta * channels.mirrored + reference.gamma * channels.chiral
    {
        return Err(direct_only_invalid("removed_channel_residual"));
    }
    // The collapsed path is the V6 direct score on the identical carrier.
    if !direct_only_matches_v6(collapsed_score, v6_score, DIRECT_ONLY_V6_MATCH_TOLERANCE)
        || !direct_only_matches_v6(channels.direct, v6_score, DIRECT_ONLY_V6_MATCH_TOLERANCE)
    {
        return Err(direct_only_invalid("v6_score_mismatch"));
    }
    // With no parity-odd channel the right/left enantiomorphic scores coincide.
    let (right, left) =
        enantiomorphic_scores(query, key, collapsed).map_err(EvalError::ChiralNumerical)?;
    if right != left || right != collapsed_score {
        return Err(direct_only_invalid("enantiomorphic_split"));
    }
    Ok(channels)
}

fn collapse_direct_only_case<T, S>(
    case: &LabeledCase<T>,
    split: DataSplit,
    oracle_sign: S,
    cases: &mut Vec<DirectOnlyCollapseCase>,
) -> Result<(), EvalError>
where
    S: Fn(&T) -> Result<i8, EvalError>,
{
    let view = case.inference_view();
    if view.split != split {
        return Err(EvalError::SplitMismatch {
            expected: split,
            actual: view.split,
        });
    }
    let reference_score = run_inference_callback(case, score_c6_from_view)?;
    let collapsed_score = run_inference_callback(case, score_c6_direct_only_from_view)?;
    let v6_score = run_inference_callback(case, score_v6_from_view)?;
    let query = Chiral6::from_array(view.query.as_array()).map_err(EvalError::ChiralNumerical)?;
    let key = Chiral6::from_array(view.key.as_array()).map_err(EvalError::ChiralNumerical)?;
    let channels = check_direct_only_identities(
        query,
        key,
        C6_REFERENCE_WEIGHTS,
        reference_score,
        collapsed_score,
        v6_score,
    )?;
    let sign = oracle_sign(case.protected_label().reveal_for_evaluation())?;
    if sign != 1 && sign != -1 {
        return Err(direct_only_invalid("oracle_sign"));
    }
    let sign = f64::from(sign);
    cases.push(DirectOnlyCollapseCase {
        family: view.family,
        case_id: view.case_id,
        reference_score,
        collapsed_score,
        v6_score,
        mirror_even_channel: channels.mirrored,
        parity_odd_channel: channels.chiral,
        reference_correct: reference_score * sign > 0.0,
        collapsed_correct: collapsed_score * sign > 0.0,
        v6_correct: v6_score * sign > 0.0,
    });
    Ok(())
}

fn summarize_direct_only_families(
    cases: &[DirectOnlyCollapseCase],
) -> Vec<DirectOnlyFamilySummary> {
    STAGE_C_PREFLIGHT_FAMILIES
        .iter()
        .map(|family| {
            let members = cases.iter().filter(|case| case.family == *family);
            DirectOnlyFamilySummary {
                family: *family,
                n_cases: members.clone().count() as u64,
                reference_correct: members.clone().filter(|c| c.reference_correct).count() as u64,
                collapsed_correct: members.clone().filter(|c| c.collapsed_correct).count() as u64,
                v6_correct: members.filter(|c| c.v6_correct).count() as u64,
            }
        })
        .collect()
}

/// Run the direct-only collapse on the bounded Stage-C case stream.
///
/// Same generators, pair ids, split and capacity as the Stage-C preflight;
/// both extra C6 channels (`beta`, `gamma`) are zeroed and the collapsed path
/// is checked bit-for-bit against the matched V6 score on the same view. The
/// matched C6 reference already uses `beta=0`, so the collapse has the same
/// weights as the slice-31 `gamma=0` ablation and the report says so
/// explicitly (`collapse_coincides_with_gamma_zero`). Any scoring or identity
/// failure aborts fail-closed; nothing is silently dropped.
pub fn run_direct_only_collapse(
    split: DataSplit,
    budget: StageCPreflightBudget,
) -> Result<DirectOnlyCollapseReport, EvalError> {
    validate_non_final_split(split)?;
    let cases_per_arm = budget.cases_per_arm()?;
    let mut cases = Vec::new();
    let last_pair_id = budget.first_pair_id + budget.pairs_per_family;
    for pair_id in budget.first_pair_id..last_pair_id {
        let discriminative = reflection_discriminative_pair_in_split(pair_id, split)
            .map_err(|_| preflight_generation_failed())?;
        for member in [discriminative.right, discriminative.left] {
            collapse_direct_only_case(
                &seal_reflection_discriminative(&member),
                split,
                handedness_sign,
                &mut cases,
            )?;
        }
        let nuisance = reflection_nuisance_pair_in_split(pair_id, split)
            .map_err(|_| preflight_generation_failed())?;
        for member in [nuisance.canonical, nuisance.reflected] {
            collapse_direct_only_case(
                &seal_reflection_nuisance(&member),
                split,
                reflection_invariant_sign,
                &mut cases,
            )?;
        }
        let direction = direction_reversal_pair_in_split(pair_id, split)
            .map_err(|_| preflight_generation_failed())?;
        for member in [direction.forward, direction.reverse] {
            collapse_direct_only_case(
                &seal_direction_reversal(&member),
                split,
                direction_sign,
                &mut cases,
            )?;
        }
        let base_case_id = pair_id
            .checked_mul(STAGE_C_PREFLIGHT_MEMBERS_PER_PAIR)
            .ok_or_else(preflight_generation_failed)?;
        for case_id in [base_case_id, base_case_id + 1] {
            let control = non_chiral_control_case_in_split(case_id, split)
                .map_err(|_| preflight_generation_failed())?;
            collapse_direct_only_case(
                &seal_non_chiral_control(&control),
                split,
                non_chiral_sign,
                &mut cases,
            )?;
        }
    }
    if cases.len() as u64 != cases_per_arm {
        return Err(direct_only_invalid("case_count"));
    }
    let families = summarize_direct_only_families(&cases);
    let collapsed_weights = direct_only_weights(C6_REFERENCE_WEIGHTS);
    let report = DirectOnlyCollapseReport {
        collapse_contract: DIRECT_ONLY_COLLAPSE_CONTRACT,
        split,
        budget,
        reference_weights: C6_REFERENCE_WEIGHTS,
        collapsed_weights,
        v6_match_tolerance: DIRECT_ONLY_V6_MATCH_TOLERANCE,
        collapse_coincides_with_gamma_zero: collapsed_weights
            == gamma_zero_weights(C6_REFERENCE_WEIGHTS),
        reference_capacity: TrainableCapacity::reference_c6(),
        collapsed_capacity: TrainableCapacity::reference_c6(),
        v6_capacity: TrainableCapacity::reference_v6(),
        cases,
        families,
        protected_or_final_access: false,
        training_executed: false,
        scientific_claim: false,
        experimental_non_final: true,
    };
    validate_direct_only_collapse_report(&report)?;
    Ok(report)
}

/// Parse a split label first; protected/final labels never generate a case.
pub fn run_direct_only_collapse_for_label(
    split_label: &str,
    budget: StageCPreflightBudget,
) -> Result<DirectOnlyCollapseReport, EvalError> {
    run_direct_only_collapse(parse_non_final_split(split_label)?, budget)
}

/// Validate a direct-only collapse report: pins, collapsed weights, the
/// declared exact V6 tolerance, the explicit `gamma=0` coincidence flag,
/// matched C6/V6 capacity, bounded coverage, the exact per-case removed-channel
/// residual, the bit-for-bit V6 score/correctness match, recomputed family
/// counts, and the no-access / no-training / no-claim flags.
pub fn validate_direct_only_collapse_report(
    report: &DirectOnlyCollapseReport,
) -> Result<(), EvalError> {
    validate_non_final_split(report.split)?;
    if report.collapse_contract != DIRECT_ONLY_COLLAPSE_CONTRACT {
        return Err(direct_only_invalid("contract_drift"));
    }
    if report.reference_weights != C6_REFERENCE_WEIGHTS {
        return Err(direct_only_invalid("reference_weights_drift"));
    }
    if report.collapsed_weights != direct_only_weights(report.reference_weights) {
        return Err(direct_only_invalid("collapsed_weights_drift"));
    }
    // V6 score semantics are the unweighted direct channel `s = q^T k`.
    if report.collapsed_weights.alpha != 1.0 {
        return Err(direct_only_invalid("v6_semantics_alpha"));
    }
    if report.v6_match_tolerance.to_bits() != DIRECT_ONLY_V6_MATCH_TOLERANCE.to_bits() {
        return Err(direct_only_invalid("tolerance_drift"));
    }
    if report.collapse_coincides_with_gamma_zero
        != (report.collapsed_weights == gamma_zero_weights(report.reference_weights))
    {
        return Err(direct_only_invalid("degeneracy_flag_drift"));
    }
    if report.reference_capacity != report.collapsed_capacity
        || report.reference_capacity != TrainableCapacity::reference_c6()
        || report.v6_capacity != TrainableCapacity::reference_v6()
        || match_parameter_counts(report.v6_capacity, report.collapsed_capacity).is_err()
    {
        return Err(direct_only_invalid("capacity_mismatch"));
    }
    let expected = report.budget.cases_per_arm()?;
    if report.cases.len() as u64 != expected {
        return Err(direct_only_invalid("case_count"));
    }
    for case in &report.cases {
        if case.reference_score
            != case.collapsed_score
                + report.reference_weights.beta * case.mirror_even_channel
                + report.reference_weights.gamma * case.parity_odd_channel
        {
            return Err(direct_only_invalid("removed_channel_residual"));
        }
        if !direct_only_matches_v6(
            case.collapsed_score,
            case.v6_score,
            report.v6_match_tolerance,
        ) || case.collapsed_correct != case.v6_correct
        {
            return Err(direct_only_invalid("v6_score_mismatch"));
        }
    }
    if report.families != summarize_direct_only_families(&report.cases)
        || report.families.iter().map(|f| f.n_cases).sum::<u64>() != expected
    {
        return Err(direct_only_invalid("family_summary_drift"));
    }
    if report.protected_or_final_access {
        return Err(direct_only_invalid("protected_or_final_access"));
    }
    if report.training_executed {
        return Err(direct_only_invalid("training_executed"));
    }
    if report.scientific_claim {
        return Err(direct_only_invalid("scientific_claim"));
    }
    if !report.experimental_non_final {
        return Err(direct_only_invalid("experimental_non_final"));
    }
    Ok(())
}

/// One case scored by the C6 reference and by the same reference on the
/// parity-shuffled carrier (slice 34).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ParityShuffleCase {
    pub family: TaskFamily,
    pub case_id: u64,
    pub reference_score: f64,
    pub shuffled_score: f64,
    /// Primitive `s/m/chi` observables of the unshuffled pair.
    pub reference_channels: ChiralObservables,
    /// Primitive `s/m/chi` observables of the parity-shuffled pair.
    pub shuffled_channels: ChiralObservables,
    /// True when the six coordinate products `q_i k_i` are the same multiset
    /// (bit-for-bit) before and after the shuffle: the shuffle only relabels
    /// carrier slots, it adds or removes no information.
    pub direct_products_preserved: bool,
    pub reference_correct: bool,
    pub shuffled_correct: bool,
}

/// Per-family correct counts for the reference and the parity-shuffle control.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ParityShuffleFamilySummary {
    pub family: TaskFamily,
    pub n_cases: u64,
    pub reference_correct: u64,
    pub shuffled_correct: u64,
    /// Cases whose parity-odd observable changed under the shuffle.
    pub parity_odd_changed: u64,
}

/// Deterministic carrier-slot permutation used by the parity-shuffle control.
///
/// `permutation[i]` is the source slot of shuffled slot `i`:
/// `shuffled[i] = x[permutation[i]]`. The permutation is drawn from an
/// already-registered seed (no new seed material) and must mix the parity
/// sectors, so the fixed `M`/`J` act on relabelled coordinates that no longer
/// carry the `H+`/`H-` split.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ParityShuffle {
    pub permutation: [usize; CHIRAL_WIDTH],
    /// Registered mixed seed the permutation was drawn from.
    pub source_seed: u64,
    /// One-based index of the accepted draw (bounded by
    /// [`MAX_PARITY_SHUFFLE_DRAWS`]).
    pub accepted_draw: u32,
}

impl ParityShuffle {
    /// Apply the slot relabelling to one carrier.
    #[must_use]
    pub fn apply(&self, carrier: Chiral6) -> Chiral6 {
        let source = carrier.as_array();
        let mut shuffled = [0.0; CHIRAL_WIDTH];
        for (slot, value) in shuffled.iter_mut().enumerate() {
            *value = source[self.permutation[slot]];
        }
        Chiral6::from_array(shuffled).expect("permutation of a finite carrier is finite")
    }

    /// Number of parity-even source slots moved into the parity-odd sector.
    #[must_use]
    pub fn cross_sector_moves(&self) -> usize {
        self.permutation[CHIRAL_WIDTH / 2..]
            .iter()
            .filter(|source| **source < CHIRAL_WIDTH / 2)
            .count()
    }
}

/// Immutable parity-shuffle control report on one non-final split.
#[derive(Clone, Debug, PartialEq)]
pub struct ParityShuffleControlReport {
    pub control_contract: &'static str,
    pub split: DataSplit,
    pub budget: StageCPreflightBudget,
    /// Reused Slice-18 registered seed `(split domain, ReflectionDiscriminative,
    /// first_pair_id)`; no new seed material.
    pub registered_seed: RegisteredSeed,
    pub shuffle: ParityShuffle,
    /// Identical weights on both sides: the control changes no coefficient.
    pub reference_weights: ChiralScoreWeights,
    pub shuffled_weights: ChiralScoreWeights,
    /// Identical capacity on both sides: the shuffle is a fixed relabelling
    /// with zero trainable parameters.
    pub reference_capacity: TrainableCapacity,
    pub shuffled_capacity: TrainableCapacity,
    pub cases: Vec<ParityShuffleCase>,
    pub families: Vec<ParityShuffleFamilySummary>,
    /// Must remain false.
    pub protected_or_final_access: bool,
    /// Must remain false: both sides are the non-trained reference.
    pub training_executed: bool,
    /// Must remain false: attribution is decided only by the Stage-D audit.
    pub scientific_claim: bool,
    /// Must remain true.
    pub experimental_non_final: bool,
}

const fn parity_shuffle_invalid(reason: &'static str) -> EvalError {
    EvalError::ParityShuffleControlInvalid { reason }
}

fn parity_shuffle_splitmix(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut mixed = *state;
    mixed = (mixed ^ (mixed >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    mixed = (mixed ^ (mixed >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    mixed ^ (mixed >> 31)
}

const PARITY_EVEN_SIGNS: [i8; CHIRAL_WIDTH] = [1, 1, 1, -1, -1, -1];

/// Mirror involution `M` as an exact signed 6x6 matrix.
fn mirror_matrix() -> [[i8; CHIRAL_WIDTH]; CHIRAL_WIDTH] {
    let mut matrix = [[0; CHIRAL_WIDTH]; CHIRAL_WIDTH];
    for (index, row) in matrix.iter_mut().enumerate() {
        row[index] = PARITY_EVEN_SIGNS[index];
    }
    matrix
}

/// Complex structure `J(x+,x-)=(x-,-x+)` as an exact signed 6x6 matrix.
fn complex_structure_matrix() -> [[i8; CHIRAL_WIDTH]; CHIRAL_WIDTH] {
    let half = CHIRAL_WIDTH / 2;
    let mut matrix = [[0; CHIRAL_WIDTH]; CHIRAL_WIDTH];
    for index in 0..half {
        matrix[index][index + half] = 1;
        matrix[index + half][index] = -1;
    }
    matrix
}

/// Effective operator `P^T A P` seen by the original coordinates when `A`
/// acts on the shuffled carrier `P x`.
fn conjugate_by_shuffle(
    matrix: [[i8; CHIRAL_WIDTH]; CHIRAL_WIDTH],
    permutation: [usize; CHIRAL_WIDTH],
) -> [[i8; CHIRAL_WIDTH]; CHIRAL_WIDTH] {
    let mut inverse = [0usize; CHIRAL_WIDTH];
    for (slot, source) in permutation.iter().enumerate() {
        inverse[*source] = slot;
    }
    let mut conjugated = [[0; CHIRAL_WIDTH]; CHIRAL_WIDTH];
    for (row, out) in conjugated.iter_mut().enumerate() {
        for (column, value) in out.iter_mut().enumerate() {
            *value = matrix[inverse[row]][inverse[column]];
        }
    }
    conjugated
}

fn negate_matrix(matrix: [[i8; CHIRAL_WIDTH]; CHIRAL_WIDTH]) -> [[i8; CHIRAL_WIDTH]; CHIRAL_WIDTH] {
    let mut negated = matrix;
    for row in &mut negated {
        for value in row.iter_mut() {
            *value = -*value;
        }
    }
    negated
}

/// True when the slot relabelling keeps or swaps the two parity sectors as
/// blocks (it then preserves the `H+`/`H-` structure up to sign).
fn shuffle_preserves_sector_blocks(permutation: [usize; CHIRAL_WIDTH]) -> bool {
    let half = CHIRAL_WIDTH / 2;
    let cross = permutation[half..]
        .iter()
        .filter(|source| **source < half)
        .count();
    cross == 0 || cross == half
}

/// Draw the parity-shuffle permutation from a registered mixed seed.
///
/// Fisher–Yates over a SplitMix64 stream seeded by `source_seed`; draws that
/// keep or swap the parity sectors as blocks are rejected deterministically
/// and the next draw is taken, at most [`MAX_PARITY_SHUFFLE_DRAWS`] times.
/// Same seed, same permutation.
pub fn parity_shuffle_from_seed(source_seed: u64) -> Result<ParityShuffle, EvalError> {
    let mut state = source_seed;
    for draw in 1..=MAX_PARITY_SHUFFLE_DRAWS {
        let mut permutation = [0, 1, 2, 3, 4, 5];
        for index in (1..CHIRAL_WIDTH).rev() {
            let bound = index as u64 + 1;
            let pick = (parity_shuffle_splitmix(&mut state) % bound) as usize;
            permutation.swap(index, pick);
        }
        if !shuffle_preserves_sector_blocks(permutation) {
            let shuffle = ParityShuffle {
                permutation,
                source_seed,
                accepted_draw: draw,
            };
            validate_parity_shuffle(&shuffle)?;
            return Ok(shuffle);
        }
    }
    Err(parity_shuffle_invalid("draw_budget_exhausted"))
}

/// Validate a parity shuffle: bijection, sector mixing, and exact proof that
/// neither the mirror involution nor the complex structure survives the
/// relabelling (`P^T M P != ±M`, `P^T J P != ±J`).
pub fn validate_parity_shuffle(shuffle: &ParityShuffle) -> Result<(), EvalError> {
    let mut seen = [false; CHIRAL_WIDTH];
    for source in shuffle.permutation {
        if source >= CHIRAL_WIDTH || seen[source] {
            return Err(parity_shuffle_invalid("not_a_permutation"));
        }
        seen[source] = true;
    }
    if shuffle.accepted_draw == 0 || shuffle.accepted_draw > MAX_PARITY_SHUFFLE_DRAWS {
        return Err(parity_shuffle_invalid("draw_budget_exhausted"));
    }
    if shuffle_preserves_sector_blocks(shuffle.permutation) {
        return Err(parity_shuffle_invalid("sector_blocks_preserved"));
    }
    let mirror = mirror_matrix();
    let shuffled_mirror = conjugate_by_shuffle(mirror, shuffle.permutation);
    if shuffled_mirror == mirror || shuffled_mirror == negate_matrix(mirror) {
        return Err(parity_shuffle_invalid("mirror_structure_preserved"));
    }
    let complex = complex_structure_matrix();
    let shuffled_complex = conjugate_by_shuffle(complex, shuffle.permutation);
    if shuffled_complex == complex || shuffled_complex == negate_matrix(complex) {
        return Err(parity_shuffle_invalid("complex_structure_preserved"));
    }
    Ok(())
}

fn sorted_product_bits(query: Chiral6, key: Chiral6) -> [u64; CHIRAL_WIDTH] {
    let q = query.as_array();
    let k = key.as_array();
    let mut bits = [0u64; CHIRAL_WIDTH];
    for (index, value) in bits.iter_mut().enumerate() {
        *value = (q[index] * k[index]).to_bits();
    }
    bits.sort_unstable();
    bits
}

fn control_parity_shuffle_case<T, S>(
    case: &LabeledCase<T>,
    split: DataSplit,
    shuffle: &ParityShuffle,
    oracle_sign: S,
    cases: &mut Vec<ParityShuffleCase>,
) -> Result<(), EvalError>
where
    S: Fn(&T) -> Result<i8, EvalError>,
{
    let view = case.inference_view();
    if view.split != split {
        return Err(EvalError::SplitMismatch {
            expected: split,
            actual: view.split,
        });
    }
    let reference_score = run_inference_callback(case, score_c6_from_view)?;
    let shuffled_score = run_inference_callback(case, |view: &InferenceView| {
        score_c6_parity_shuffled_from_view(view, shuffle)
    })?;
    let query = Chiral6::from_array(view.query.as_array()).map_err(EvalError::ChiralNumerical)?;
    let key = Chiral6::from_array(view.key.as_array()).map_err(EvalError::ChiralNumerical)?;
    let shuffled_query = shuffle.apply(query);
    let shuffled_key = shuffle.apply(key);
    let reference_channels = observables(query, key).map_err(EvalError::ChiralNumerical)?;
    let shuffled_channels =
        observables(shuffled_query, shuffled_key).map_err(EvalError::ChiralNumerical)?;
    let direct_products_preserved =
        sorted_product_bits(query, key) == sorted_product_bits(shuffled_query, shuffled_key);
    if !direct_products_preserved {
        return Err(parity_shuffle_invalid("direct_product_multiset_drift"));
    }
    let sign = oracle_sign(case.protected_label().reveal_for_evaluation())?;
    if sign != 1 && sign != -1 {
        return Err(parity_shuffle_invalid("oracle_sign"));
    }
    let sign = f64::from(sign);
    cases.push(ParityShuffleCase {
        family: view.family,
        case_id: view.case_id,
        reference_score,
        shuffled_score,
        reference_channels,
        shuffled_channels,
        direct_products_preserved,
        reference_correct: reference_score * sign > 0.0,
        shuffled_correct: shuffled_score * sign > 0.0,
    });
    Ok(())
}

/// Score an inference view with the C6 reference on the parity-shuffled carrier.
pub fn score_c6_parity_shuffled_from_view(
    view: &InferenceView,
    shuffle: &ParityShuffle,
) -> Result<f64, EvalError> {
    validate_parity_shuffle(shuffle)?;
    let query = Chiral6::from_array(view.query.as_array()).map_err(EvalError::ChiralNumerical)?;
    let key = Chiral6::from_array(view.key.as_array()).map_err(EvalError::ChiralNumerical)?;
    chiral_score(
        shuffle.apply(query),
        shuffle.apply(key),
        C6_REFERENCE_WEIGHTS,
    )
    .map_err(EvalError::ChiralNumerical)
}

fn summarize_parity_shuffle_families(
    cases: &[ParityShuffleCase],
) -> Vec<ParityShuffleFamilySummary> {
    STAGE_C_PREFLIGHT_FAMILIES
        .iter()
        .map(|family| {
            let members = cases.iter().filter(|case| case.family == *family);
            ParityShuffleFamilySummary {
                family: *family,
                n_cases: members.clone().count() as u64,
                reference_correct: members.clone().filter(|c| c.reference_correct).count() as u64,
                shuffled_correct: members.clone().filter(|c| c.shuffled_correct).count() as u64,
                parity_odd_changed: members
                    .filter(|c| {
                        c.reference_channels.chiral.to_bits()
                            != c.shuffled_channels.chiral.to_bits()
                    })
                    .count() as u64,
            }
        })
        .collect()
}

fn parity_shuffle_registered_seed(
    split: DataSplit,
    budget: StageCPreflightBudget,
) -> RegisteredSeed {
    register_seed(
        SeedDomain::from_split(split),
        STAGE_C_PREFLIGHT_FAMILIES[0],
        budget.first_pair_id,
    )
}

/// Run the parity-shuffle control on the bounded Stage-C case stream.
///
/// Same generators, pair ids, split, weights and capacity as the Stage-C
/// preflight; only the carrier slots are relabelled by a fixed, seed-derived
/// permutation that mixes `H+` and `H-`. Any scoring or identity failure
/// aborts fail-closed; nothing is silently dropped.
pub fn run_parity_shuffle_control(
    split: DataSplit,
    budget: StageCPreflightBudget,
) -> Result<ParityShuffleControlReport, EvalError> {
    let report = collect_parity_shuffle_control(split, budget)?;
    validate_parity_shuffle_control_report(&report)?;
    Ok(report)
}

/// Generate the parity-shuffle control report without validating it.
///
/// Shared by [`run_parity_shuffle_control`] and by the validator, which
/// regenerates every case from the report's split and budget instead of
/// trusting stored per-case evidence.
fn collect_parity_shuffle_control(
    split: DataSplit,
    budget: StageCPreflightBudget,
) -> Result<ParityShuffleControlReport, EvalError> {
    validate_non_final_split(split)?;
    let cases_per_arm = budget.cases_per_arm()?;
    let registered_seed = parity_shuffle_registered_seed(split, budget);
    let shuffle = parity_shuffle_from_seed(registered_seed.mixed_seed)?;
    let mut cases = Vec::new();
    let last_pair_id = budget.first_pair_id + budget.pairs_per_family;
    for pair_id in budget.first_pair_id..last_pair_id {
        let discriminative = reflection_discriminative_pair_in_split(pair_id, split)
            .map_err(|_| preflight_generation_failed())?;
        for member in [discriminative.right, discriminative.left] {
            control_parity_shuffle_case(
                &seal_reflection_discriminative(&member),
                split,
                &shuffle,
                handedness_sign,
                &mut cases,
            )?;
        }
        let nuisance = reflection_nuisance_pair_in_split(pair_id, split)
            .map_err(|_| preflight_generation_failed())?;
        for member in [nuisance.canonical, nuisance.reflected] {
            control_parity_shuffle_case(
                &seal_reflection_nuisance(&member),
                split,
                &shuffle,
                reflection_invariant_sign,
                &mut cases,
            )?;
        }
        let direction = direction_reversal_pair_in_split(pair_id, split)
            .map_err(|_| preflight_generation_failed())?;
        for member in [direction.forward, direction.reverse] {
            control_parity_shuffle_case(
                &seal_direction_reversal(&member),
                split,
                &shuffle,
                direction_sign,
                &mut cases,
            )?;
        }
        let base_case_id = pair_id
            .checked_mul(STAGE_C_PREFLIGHT_MEMBERS_PER_PAIR)
            .ok_or_else(preflight_generation_failed)?;
        for case_id in [base_case_id, base_case_id + 1] {
            let control = non_chiral_control_case_in_split(case_id, split)
                .map_err(|_| preflight_generation_failed())?;
            control_parity_shuffle_case(
                &seal_non_chiral_control(&control),
                split,
                &shuffle,
                non_chiral_sign,
                &mut cases,
            )?;
        }
    }
    if cases.len() as u64 != cases_per_arm {
        return Err(parity_shuffle_invalid("case_count"));
    }
    let families = summarize_parity_shuffle_families(&cases);
    Ok(ParityShuffleControlReport {
        control_contract: PARITY_SHUFFLE_CONTROL_CONTRACT,
        split,
        budget,
        registered_seed,
        shuffle,
        reference_weights: C6_REFERENCE_WEIGHTS,
        shuffled_weights: C6_REFERENCE_WEIGHTS,
        reference_capacity: TrainableCapacity::reference_c6(),
        shuffled_capacity: TrainableCapacity::reference_c6(),
        cases,
        families,
        protected_or_final_access: false,
        training_executed: false,
        scientific_claim: false,
        experimental_non_final: true,
    })
}

/// Score implied by retained primitive channels, accumulated in exactly the
/// order of the upstream `chiral_score` (so the comparison is bit-exact).
fn weighted_channel_score(channels: ChiralObservables, weights: ChiralScoreWeights) -> f64 {
    (weights.alpha * channels.direct + weights.beta * channels.mirrored)
        + weights.gamma * channels.chiral
}

/// Parse a split label first; protected/final labels never generate a case.
pub fn run_parity_shuffle_control_for_label(
    split_label: &str,
    budget: StageCPreflightBudget,
) -> Result<ParityShuffleControlReport, EvalError> {
    run_parity_shuffle_control(parse_non_final_split(split_label)?, budget)
}

/// Validate a parity-shuffle control report: pins, reproducible seed and
/// permutation, structure-destroying shuffle, identical weights and capacity,
/// bounded coverage, preserved direct-product multiset, recomputed family
/// counts, and the no-access / no-training / no-claim flags.
pub fn validate_parity_shuffle_control_report(
    report: &ParityShuffleControlReport,
) -> Result<(), EvalError> {
    validate_non_final_split(report.split)?;
    if report.control_contract != PARITY_SHUFFLE_CONTROL_CONTRACT {
        return Err(parity_shuffle_invalid("contract_drift"));
    }
    if report.registered_seed != parity_shuffle_registered_seed(report.split, report.budget) {
        return Err(parity_shuffle_invalid("seed_drift"));
    }
    validate_parity_shuffle(&report.shuffle)?;
    if report.shuffle != parity_shuffle_from_seed(report.registered_seed.mixed_seed)? {
        return Err(parity_shuffle_invalid("shuffle_not_reproducible"));
    }
    if report.reference_weights != C6_REFERENCE_WEIGHTS {
        return Err(parity_shuffle_invalid("reference_weights_drift"));
    }
    if report.shuffled_weights != report.reference_weights {
        return Err(parity_shuffle_invalid("shuffled_weights_drift"));
    }
    if report.reference_capacity != report.shuffled_capacity
        || report.reference_capacity != TrainableCapacity::reference_c6()
    {
        return Err(parity_shuffle_invalid("capacity_mismatch"));
    }
    let expected = report.budget.cases_per_arm()?;
    if report.cases.len() as u64 != expected {
        return Err(parity_shuffle_invalid("case_count"));
    }
    if report
        .cases
        .iter()
        .any(|case| !case.direct_products_preserved)
    {
        return Err(parity_shuffle_invalid("direct_product_multiset_drift"));
    }
    for case in &report.cases {
        if case.reference_score.to_bits()
            != weighted_channel_score(case.reference_channels, report.reference_weights).to_bits()
            || case.shuffled_score.to_bits()
                != weighted_channel_score(case.shuffled_channels, report.shuffled_weights).to_bits()
        {
            return Err(parity_shuffle_invalid("score_channel_drift"));
        }
    }
    if report.families != summarize_parity_shuffle_families(&report.cases)
        || report.families.iter().map(|f| f.n_cases).sum::<u64>() != expected
    {
        return Err(parity_shuffle_invalid("family_summary_drift"));
    }
    if report.protected_or_final_access {
        return Err(parity_shuffle_invalid("protected_or_final_access"));
    }
    if report.training_executed {
        return Err(parity_shuffle_invalid("training_executed"));
    }
    if report.scientific_claim {
        return Err(parity_shuffle_invalid("scientific_claim"));
    }
    if !report.experimental_non_final {
        return Err(parity_shuffle_invalid("experimental_non_final"));
    }
    // Stored evidence is never trusted: every case (scores, channels,
    // preserved-product flag and correctness) is regenerated from the
    // report's split, budget and reproducible shuffle and compared exactly.
    let regenerated = collect_parity_shuffle_control(report.split, report.budget)?;
    if regenerated.shuffle != report.shuffle || regenerated.cases != report.cases {
        return Err(parity_shuffle_invalid("case_evidence_drift"));
    }
    Ok(())
}

/// One fixed mirror basis of the slice-35 family.
///
/// `even_slots` are the carrier slots declared parity-even (`H+`) and
/// `odd_slots` the complementary parity-odd slots (`H-`), both ascending; the
/// `i`-th even slot is paired with the `i`-th odd slot by the complex
/// structure. Scoring a carrier under this basis is scoring the relabelled
/// carrier `(x[even_slots], x[odd_slots])` with the canonical `M`/`J`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FixedMirrorBasis {
    /// Lexicographic rank of `even_slots` in the frozen enumeration.
    pub index: usize,
    pub even_slots: [usize; 3],
    pub odd_slots: [usize; 3],
}

impl FixedMirrorBasis {
    /// Slot relabelling `shuffled[i] = x[permutation[i]]`.
    #[must_use]
    pub const fn permutation(&self) -> [usize; CHIRAL_WIDTH] {
        [
            self.even_slots[0],
            self.even_slots[1],
            self.even_slots[2],
            self.odd_slots[0],
            self.odd_slots[1],
            self.odd_slots[2],
        ]
    }

    /// Express a carrier in this basis.
    #[must_use]
    pub fn apply(&self, carrier: Chiral6) -> Chiral6 {
        let source = carrier.as_array();
        let permutation = self.permutation();
        let mut relabelled = [0.0; CHIRAL_WIDTH];
        for (slot, value) in relabelled.iter_mut().enumerate() {
            *value = source[permutation[slot]];
        }
        Chiral6::from_array(relabelled).expect("permutation of a finite carrier is finite")
    }

    /// True for the declared canonical basis `H+ = {0,1,2}`.
    #[must_use]
    pub const fn is_canonical(&self) -> bool {
        self.even_slots[0] == 0 && self.even_slots[1] == 1 && self.even_slots[2] == 2
    }

    /// Index of the complementary basis (sectors exchanged).
    #[must_use]
    pub const fn complement_index(&self) -> usize {
        FIXED_MIRROR_BASIS_COUNT - 1 - self.index
    }
}

/// The frozen slice-35 basis family: every 3-subset of the six slots as the
/// parity-even sector, in lexicographic order (canonical basis first). There
/// is no selection, filtering or tuning; the family is complete.
#[must_use]
pub fn fixed_mirror_bases() -> [FixedMirrorBasis; FIXED_MIRROR_BASIS_COUNT] {
    let mut bases = [FixedMirrorBasis {
        index: 0,
        even_slots: [0, 1, 2],
        odd_slots: [3, 4, 5],
    }; FIXED_MIRROR_BASIS_COUNT];
    let mut index = 0;
    for a in 0..CHIRAL_WIDTH {
        for b in a + 1..CHIRAL_WIDTH {
            for c in b + 1..CHIRAL_WIDTH {
                let mut odd = [0usize; 3];
                let mut cursor = 0;
                for slot in 0..CHIRAL_WIDTH {
                    if slot != a && slot != b && slot != c {
                        odd[cursor] = slot;
                        cursor += 1;
                    }
                }
                bases[index] = FixedMirrorBasis {
                    index,
                    even_slots: [a, b, c],
                    odd_slots: odd,
                };
                index += 1;
            }
        }
    }
    bases
}

type SignedMatrix = [[i8; CHIRAL_WIDTH]; CHIRAL_WIDTH];

fn signed_matmul(left: SignedMatrix, right: SignedMatrix) -> SignedMatrix {
    let mut product = [[0i8; CHIRAL_WIDTH]; CHIRAL_WIDTH];
    for (row, out) in product.iter_mut().enumerate() {
        for (column, value) in out.iter_mut().enumerate() {
            *value = (0..CHIRAL_WIDTH)
                .map(|inner| left[row][inner] * right[inner][column])
                .sum();
        }
    }
    product
}

fn signed_transpose(matrix: SignedMatrix) -> SignedMatrix {
    let mut transposed = [[0i8; CHIRAL_WIDTH]; CHIRAL_WIDTH];
    for (row, values) in matrix.iter().enumerate() {
        for (column, value) in values.iter().enumerate() {
            transposed[column][row] = *value;
        }
    }
    transposed
}

fn signed_identity() -> SignedMatrix {
    let mut identity = [[0i8; CHIRAL_WIDTH]; CHIRAL_WIDTH];
    for (index, row) in identity.iter_mut().enumerate() {
        row[index] = 1;
    }
    identity
}

const fn fixed_m_invalid(reason: &'static str) -> EvalError {
    EvalError::FixedMSensitivityInvalid { reason }
}

/// Validate one basis: a true ascending sector partition at its frozen rank,
/// and the exact chiral algebra for the effective `M' = P^T M P` and
/// `J' = P^T J P` (`M'^2 = I`, `J'^T = -J'`, `J'^2 = -I`, `M' J' M' = -J'`).
pub fn validate_fixed_mirror_basis(basis: &FixedMirrorBasis) -> Result<(), EvalError> {
    if basis.index >= FIXED_MIRROR_BASIS_COUNT {
        return Err(fixed_m_invalid("basis_order_drift"));
    }
    let mut seen = [false; CHIRAL_WIDTH];
    for slot in basis.permutation() {
        if slot >= CHIRAL_WIDTH || seen[slot] {
            return Err(fixed_m_invalid("basis_not_a_sector_partition"));
        }
        seen[slot] = true;
    }
    if !(basis.even_slots[0] < basis.even_slots[1] && basis.even_slots[1] < basis.even_slots[2])
        || !(basis.odd_slots[0] < basis.odd_slots[1] && basis.odd_slots[1] < basis.odd_slots[2])
    {
        return Err(fixed_m_invalid("basis_not_a_sector_partition"));
    }
    if fixed_mirror_bases()[basis.index] != *basis {
        return Err(fixed_m_invalid("basis_order_drift"));
    }
    let permutation = basis.permutation();
    let mirror = conjugate_by_shuffle(mirror_matrix(), permutation);
    let complex = conjugate_by_shuffle(complex_structure_matrix(), permutation);
    let identity = signed_identity();
    if signed_matmul(mirror, mirror) != identity
        || signed_transpose(complex) != negate_matrix(complex)
        || signed_matmul(complex, complex) != negate_matrix(identity)
        || signed_matmul(signed_matmul(mirror, complex), mirror) != negate_matrix(complex)
    {
        return Err(fixed_m_invalid("basis_algebra_failure"));
    }
    Ok(())
}

/// One case scored by the C6 reference weights under one fixed mirror basis.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FixedMSensitivityCase {
    pub family: TaskFamily,
    pub case_id: u64,
    pub basis_index: usize,
    pub score: f64,
    /// Parity-odd observable `chi` under this basis.
    pub chiral: f64,
    pub correct: bool,
}

/// Per-basis, per-family correct counts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FixedMBasisFamilySummary {
    pub basis_index: usize,
    pub family: TaskFamily,
    pub n_cases: u64,
    pub correct: u64,
}

/// Immutable fixed-M sensitivity report on one non-final split.
#[derive(Clone, Debug, PartialEq)]
pub struct FixedMSensitivityReport {
    pub sensitivity_contract: &'static str,
    pub split: DataSplit,
    pub budget: StageCPreflightBudget,
    /// The complete frozen basis family, canonical first.
    pub bases: Vec<FixedMirrorBasis>,
    /// Identical C6 reference weights under every basis.
    pub weights: ChiralScoreWeights,
    /// Identical capacity under every basis: a basis is a fixed relabelling
    /// with zero trainable parameters.
    pub capacity: TrainableCapacity,
    /// Case-major, basis-minor order.
    pub cases: Vec<FixedMSensitivityCase>,
    pub summaries: Vec<FixedMBasisFamilySummary>,
    /// Must remain false.
    pub protected_or_final_access: bool,
    /// Must remain false: every basis is scored with the non-trained reference.
    pub training_executed: bool,
    /// Must remain false: no basis is selected; attribution is decided only by
    /// the Stage-D audit.
    pub scientific_claim: bool,
    /// Must remain true.
    pub experimental_non_final: bool,
}

/// Score an inference view with the C6 reference weights under one basis.
pub fn score_c6_in_basis_from_view(
    view: &InferenceView,
    basis: &FixedMirrorBasis,
) -> Result<f64, EvalError> {
    validate_fixed_mirror_basis(basis)?;
    let query = Chiral6::from_array(view.query.as_array()).map_err(EvalError::ChiralNumerical)?;
    let key = Chiral6::from_array(view.key.as_array()).map_err(EvalError::ChiralNumerical)?;
    chiral_score(basis.apply(query), basis.apply(key), C6_REFERENCE_WEIGHTS)
        .map_err(EvalError::ChiralNumerical)
}

fn fixed_m_case<T, S>(
    case: &LabeledCase<T>,
    split: DataSplit,
    bases: &[FixedMirrorBasis; FIXED_MIRROR_BASIS_COUNT],
    oracle_sign: S,
    cases: &mut Vec<FixedMSensitivityCase>,
) -> Result<(), EvalError>
where
    S: Fn(&T) -> Result<i8, EvalError>,
{
    let view = case.inference_view();
    if view.split != split {
        return Err(EvalError::SplitMismatch {
            expected: split,
            actual: view.split,
        });
    }
    let sign = oracle_sign(case.protected_label().reveal_for_evaluation())?;
    if sign != 1 && sign != -1 {
        return Err(fixed_m_invalid("oracle_sign"));
    }
    let sign = f64::from(sign);
    let reference = run_inference_callback(case, score_c6_from_view)?;
    let query = Chiral6::from_array(view.query.as_array()).map_err(EvalError::ChiralNumerical)?;
    let key = Chiral6::from_array(view.key.as_array()).map_err(EvalError::ChiralNumerical)?;
    for basis in bases {
        let score = run_inference_callback(case, |view: &InferenceView| {
            score_c6_in_basis_from_view(view, basis)
        })?;
        if basis.is_canonical() && score.to_bits() != reference.to_bits() {
            return Err(fixed_m_invalid("canonical_reference_drift"));
        }
        let chiral = basis
            .apply(query)
            .chiral_pairing(basis.apply(key))
            .map_err(EvalError::ChiralNumerical)?;
        cases.push(FixedMSensitivityCase {
            family: view.family,
            case_id: view.case_id,
            basis_index: basis.index,
            score,
            chiral,
            correct: score * sign > 0.0,
        });
    }
    Ok(())
}

fn summarize_fixed_m(cases: &[FixedMSensitivityCase]) -> Vec<FixedMBasisFamilySummary> {
    let mut summaries =
        Vec::with_capacity(FIXED_MIRROR_BASIS_COUNT * STAGE_C_PREFLIGHT_FAMILIES.len());
    for basis_index in 0..FIXED_MIRROR_BASIS_COUNT {
        for family in STAGE_C_PREFLIGHT_FAMILIES {
            let members = cases
                .iter()
                .filter(|case| case.basis_index == basis_index && case.family == *family);
            summaries.push(FixedMBasisFamilySummary {
                basis_index,
                family: *family,
                n_cases: members.clone().count() as u64,
                correct: members.filter(|case| case.correct).count() as u64,
            });
        }
    }
    summaries
}

fn collect_fixed_m_cases(
    split: DataSplit,
    budget: StageCPreflightBudget,
) -> Result<Vec<FixedMSensitivityCase>, EvalError> {
    validate_non_final_split(split)?;
    budget.cases_per_arm()?;
    let bases = fixed_mirror_bases();
    for basis in &bases {
        validate_fixed_mirror_basis(basis)?;
    }
    let mut cases = Vec::new();
    let last_pair_id = budget.first_pair_id + budget.pairs_per_family;
    for pair_id in budget.first_pair_id..last_pair_id {
        let discriminative = reflection_discriminative_pair_in_split(pair_id, split)
            .map_err(|_| preflight_generation_failed())?;
        for member in [discriminative.right, discriminative.left] {
            fixed_m_case(
                &seal_reflection_discriminative(&member),
                split,
                &bases,
                handedness_sign,
                &mut cases,
            )?;
        }
        let nuisance = reflection_nuisance_pair_in_split(pair_id, split)
            .map_err(|_| preflight_generation_failed())?;
        for member in [nuisance.canonical, nuisance.reflected] {
            fixed_m_case(
                &seal_reflection_nuisance(&member),
                split,
                &bases,
                reflection_invariant_sign,
                &mut cases,
            )?;
        }
        let direction = direction_reversal_pair_in_split(pair_id, split)
            .map_err(|_| preflight_generation_failed())?;
        for member in [direction.forward, direction.reverse] {
            fixed_m_case(
                &seal_direction_reversal(&member),
                split,
                &bases,
                direction_sign,
                &mut cases,
            )?;
        }
        let base_case_id = pair_id
            .checked_mul(STAGE_C_PREFLIGHT_MEMBERS_PER_PAIR)
            .ok_or_else(preflight_generation_failed)?;
        for case_id in [base_case_id, base_case_id + 1] {
            let control = non_chiral_control_case_in_split(case_id, split)
                .map_err(|_| preflight_generation_failed())?;
            fixed_m_case(
                &seal_non_chiral_control(&control),
                split,
                &bases,
                non_chiral_sign,
                &mut cases,
            )?;
        }
    }
    Ok(cases)
}

/// Run the fixed-M sensitivity study on the bounded Stage-C case stream.
///
/// Same generators, pair ids, split, weights and capacity as the Stage-C
/// preflight; every case is scored under all twenty frozen mirror bases. Any
/// scoring or identity failure aborts fail-closed; nothing is dropped.
pub fn run_fixed_m_sensitivity(
    split: DataSplit,
    budget: StageCPreflightBudget,
) -> Result<FixedMSensitivityReport, EvalError> {
    let cases = collect_fixed_m_cases(split, budget)?;
    let summaries = summarize_fixed_m(&cases);
    let report = FixedMSensitivityReport {
        sensitivity_contract: FIXED_M_SENSITIVITY_CONTRACT,
        split,
        budget,
        bases: fixed_mirror_bases().to_vec(),
        weights: C6_REFERENCE_WEIGHTS,
        capacity: TrainableCapacity::reference_c6(),
        cases,
        summaries,
        protected_or_final_access: false,
        training_executed: false,
        scientific_claim: false,
        experimental_non_final: true,
    };
    validate_fixed_m_sensitivity_report(&report)?;
    Ok(report)
}

/// Parse a split label first; protected/final labels never generate a case.
pub fn run_fixed_m_sensitivity_for_label(
    split_label: &str,
    budget: StageCPreflightBudget,
) -> Result<FixedMSensitivityReport, EvalError> {
    run_fixed_m_sensitivity(parse_non_final_split(split_label)?, budget)
}

/// Validate a fixed-M sensitivity report: contract pin, the complete frozen
/// basis family in order with exact algebra, identical weights and capacity,
/// bounded coverage in case-major/basis-minor order, exact complementary-basis
/// antisymmetry of `chi`, recomputed summaries, the no-access / no-training /
/// no-claim flags, and regenerated per-case evidence.
pub fn validate_fixed_m_sensitivity_report(
    report: &FixedMSensitivityReport,
) -> Result<(), EvalError> {
    validate_non_final_split(report.split)?;
    if report.sensitivity_contract != FIXED_M_SENSITIVITY_CONTRACT {
        return Err(fixed_m_invalid("contract_drift"));
    }
    if report.bases.len() != FIXED_MIRROR_BASIS_COUNT {
        return Err(fixed_m_invalid("basis_family_incomplete"));
    }
    for (position, basis) in report.bases.iter().enumerate() {
        if basis.index != position {
            return Err(fixed_m_invalid("basis_order_drift"));
        }
        validate_fixed_mirror_basis(basis)?;
    }
    if !report.bases[0].is_canonical() {
        return Err(fixed_m_invalid("basis_order_drift"));
    }
    if report.weights != C6_REFERENCE_WEIGHTS {
        return Err(fixed_m_invalid("weights_drift"));
    }
    if report.capacity != TrainableCapacity::reference_c6() {
        return Err(fixed_m_invalid("capacity_mismatch"));
    }
    let per_basis = report.budget.cases_per_arm()?;
    if report.cases.len() as u64 != per_basis * FIXED_MIRROR_BASIS_COUNT as u64 {
        return Err(fixed_m_invalid("case_count"));
    }
    for group in report.cases.chunks(FIXED_MIRROR_BASIS_COUNT) {
        for (basis_index, case) in group.iter().enumerate() {
            if case.basis_index != basis_index
                || case.family != group[0].family
                || case.case_id != group[0].case_id
            {
                return Err(fixed_m_invalid("case_order"));
            }
        }
        for case in group {
            let complement = &group[report.bases[case.basis_index].complement_index()];
            if case.chiral != -complement.chiral {
                return Err(fixed_m_invalid("complement_antisymmetry"));
            }
        }
    }
    if report.summaries != summarize_fixed_m(&report.cases)
        || report
            .summaries
            .iter()
            .filter(|summary| summary.basis_index == 0)
            .map(|summary| summary.n_cases)
            .sum::<u64>()
            != per_basis
    {
        return Err(fixed_m_invalid("summary_drift"));
    }
    if report.protected_or_final_access {
        return Err(fixed_m_invalid("protected_or_final_access"));
    }
    if report.training_executed {
        return Err(fixed_m_invalid("training_executed"));
    }
    if report.scientific_claim {
        return Err(fixed_m_invalid("scientific_claim"));
    }
    if !report.experimental_non_final {
        return Err(fixed_m_invalid("experimental_non_final"));
    }
    // Stored evidence is never trusted: every case is regenerated from the
    // report's split and budget under the frozen basis family and compared
    // exactly.
    if collect_fixed_m_cases(report.split, report.budget)? != report.cases {
        return Err(fixed_m_invalid("case_evidence_drift"));
    }
    Ok(())
}

/// Phase-D structure-preserving learned basis prototype contract pin (slice 36).
pub const LEARNED_BASIS_PROTOTYPE_CONTRACT: &str = "tdi24-learned-basis-prototype-v1";

/// Number of Givens planes `(i, j)`, `i < j`, of the six-slot carrier:
/// `C(6,2) = 15`. Derived from the carrier width; not a freeze pin.
pub const LEARNED_BASIS_PARAMETER_COUNT: usize = 15;

/// Number of declared, non-tuned prototype probes (identity, gauge,
/// sector-mixing, generic). Not a freeze pin and not selected from outcomes.
pub const LEARNED_BASIS_PROBE_COUNT: usize = 4;

/// Declared floating-point tolerance for the orthogonality, algebra and gauge
/// invariance checks of the prototype. A numerical guard, not a freeze pin.
pub const LEARNED_BASIS_TOLERANCE: f64 = 1e-12;

type RealMatrix = [[f64; CHIRAL_WIDTH]; CHIRAL_WIDTH];

const fn learned_basis_invalid(reason: &'static str) -> EvalError {
    EvalError::LearnedBasisPrototypeInvalid { reason }
}

/// Role of a prototype probe in the declared probe set.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LearnedBasisProbeRole {
    /// All angles zero: must reproduce the Stage-C C6 evaluator bit for bit.
    Identity,
    /// `O = diag(R, R)`: commutes with `M` and `J`, scores must be invariant.
    Gauge,
    /// One rotation in the `(0, 4)` plane mixing the two parity sectors across
    /// a pair not coupled by `J` (the `(0, 3)` plane commutes with `J`).
    SectorMixing,
    /// Every plane rotated by a distinct declared angle.
    Generic,
}

/// One point of the orthogonal parameterisation
/// `O(theta) = G(0,1) G(0,2) ... G(4,5)`, a product of Givens rotations in
/// lexicographic plane order. Every point is orthogonal by construction, so
/// the effective operators `M' = O^T M O`, `J' = O^T J O` satisfy the chiral
/// algebra; nothing is trained, tuned or selected.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LearnedBasisProbe {
    pub index: usize,
    pub role: LearnedBasisProbeRole,
    /// Angles in lexicographic plane order `(0,1), (0,2), ..., (4,5)`.
    pub angles: [f64; LEARNED_BASIS_PARAMETER_COUNT],
}

/// Givens planes in lexicographic order.
#[must_use]
pub fn learned_basis_planes() -> [(usize, usize); LEARNED_BASIS_PARAMETER_COUNT] {
    let mut planes = [(0, 0); LEARNED_BASIS_PARAMETER_COUNT];
    let mut index = 0;
    for i in 0..CHIRAL_WIDTH {
        for j in i + 1..CHIRAL_WIDTH {
            planes[index] = (i, j);
            index += 1;
        }
    }
    planes
}

fn plane_index(i: usize, j: usize) -> usize {
    learned_basis_planes()
        .iter()
        .position(|plane| *plane == (i, j))
        .expect("declared plane")
}

/// The declared prototype probe set, in fixed order.
#[must_use]
pub fn learned_basis_probes() -> [LearnedBasisProbe; LEARNED_BASIS_PROBE_COUNT] {
    use core::f64::consts::PI;
    let identity = [0.0; LEARNED_BASIS_PARAMETER_COUNT];
    let mut gauge = identity;
    // The same SO(3) rotation on H+ = {0,1,2} and on H- = {3,4,5}.
    gauge[plane_index(0, 1)] = PI / 5.0;
    gauge[plane_index(1, 2)] = PI / 7.0;
    gauge[plane_index(3, 4)] = PI / 5.0;
    gauge[plane_index(4, 5)] = PI / 7.0;
    let mut mixing = identity;
    mixing[plane_index(0, 4)] = PI / 4.0;
    let mut generic = identity;
    for (plane, angle) in generic.iter_mut().enumerate() {
        *angle = (plane as f64 + 1.0) * PI / 32.0;
    }
    [
        LearnedBasisProbe {
            index: 0,
            role: LearnedBasisProbeRole::Identity,
            angles: identity,
        },
        LearnedBasisProbe {
            index: 1,
            role: LearnedBasisProbeRole::Gauge,
            angles: gauge,
        },
        LearnedBasisProbe {
            index: 2,
            role: LearnedBasisProbeRole::SectorMixing,
            angles: mixing,
        },
        LearnedBasisProbe {
            index: 3,
            role: LearnedBasisProbeRole::Generic,
            angles: generic,
        },
    ]
}

fn real_identity() -> RealMatrix {
    let mut identity = [[0.0; CHIRAL_WIDTH]; CHIRAL_WIDTH];
    for (index, row) in identity.iter_mut().enumerate() {
        row[index] = 1.0;
    }
    identity
}

fn real_matmul(left: &RealMatrix, right: &RealMatrix) -> RealMatrix {
    let mut product = [[0.0; CHIRAL_WIDTH]; CHIRAL_WIDTH];
    for (row, out) in product.iter_mut().enumerate() {
        for (column, value) in out.iter_mut().enumerate() {
            let mut sum = left[row][0] * right[0][column];
            for inner in 1..CHIRAL_WIDTH {
                sum += left[row][inner] * right[inner][column];
            }
            *value = sum;
        }
    }
    product
}

fn real_transpose(matrix: &RealMatrix) -> RealMatrix {
    let mut transposed = [[0.0; CHIRAL_WIDTH]; CHIRAL_WIDTH];
    for (row, values) in matrix.iter().enumerate() {
        for (column, value) in values.iter().enumerate() {
            transposed[column][row] = *value;
        }
    }
    transposed
}

fn real_from_signed(matrix: [[i8; CHIRAL_WIDTH]; CHIRAL_WIDTH]) -> RealMatrix {
    let mut real = [[0.0; CHIRAL_WIDTH]; CHIRAL_WIDTH];
    for (row, values) in matrix.iter().enumerate() {
        for (column, value) in values.iter().enumerate() {
            real[row][column] = f64::from(*value);
        }
    }
    real
}

/// Max-abs distance `|| left - sign * right ||_inf` over entries.
fn real_distance(left: &RealMatrix, right: &RealMatrix, sign: f64) -> f64 {
    let mut distance: f64 = 0.0;
    for row in 0..CHIRAL_WIDTH {
        for column in 0..CHIRAL_WIDTH {
            distance = distance.max((left[row][column] - sign * right[row][column]).abs());
        }
    }
    distance
}

/// Orthogonal transform of a probe: the Givens product in lexicographic order.
pub fn learned_basis_transform(probe: &LearnedBasisProbe) -> Result<RealMatrix, EvalError> {
    if probe.angles.iter().any(|angle| !angle.is_finite()) {
        return Err(learned_basis_invalid("non_finite_parameter"));
    }
    let mut transform = real_identity();
    for (plane, (i, j)) in learned_basis_planes().iter().enumerate() {
        let angle = probe.angles[plane];
        if angle == 0.0 {
            continue;
        }
        let mut givens = real_identity();
        let (sin, cos) = angle.sin_cos();
        givens[*i][*i] = cos;
        givens[*j][*j] = cos;
        givens[*i][*j] = -sin;
        givens[*j][*i] = sin;
        transform = real_matmul(&transform, &givens);
    }
    Ok(transform)
}

/// Validate a candidate basis transform: finite, orthogonal within the
/// declared tolerance, and the effective `M' = O^T M O`, `J' = O^T J O`
/// satisfy `M'^2 = I`, `J'^T = -J'`, `J'^2 = -I`, `M' J' M' = -J'`.
/// Non-orthogonal (e.g. scaling or shearing) transforms are rejected.
pub fn validate_learned_basis_transform(transform: &RealMatrix) -> Result<(), EvalError> {
    if transform.iter().flatten().any(|value| !value.is_finite()) {
        return Err(learned_basis_invalid("non_finite_parameter"));
    }
    let identity = real_identity();
    let transposed = real_transpose(transform);
    if real_distance(&real_matmul(&transposed, transform), &identity, 1.0) > LEARNED_BASIS_TOLERANCE
    {
        return Err(learned_basis_invalid("orthogonality_failure"));
    }
    let mirror = real_matmul(
        &real_matmul(&transposed, &real_from_signed(mirror_matrix())),
        transform,
    );
    let complex = real_matmul(
        &real_matmul(&transposed, &real_from_signed(complex_structure_matrix())),
        transform,
    );
    if real_distance(&real_matmul(&mirror, &mirror), &identity, 1.0) > LEARNED_BASIS_TOLERANCE
        || real_distance(&real_transpose(&complex), &complex, -1.0) > LEARNED_BASIS_TOLERANCE
        || real_distance(&real_matmul(&complex, &complex), &identity, -1.0)
            > LEARNED_BASIS_TOLERANCE
        || real_distance(
            &real_matmul(&real_matmul(&mirror, &complex), &mirror),
            &complex,
            -1.0,
        ) > LEARNED_BASIS_TOLERANCE
    {
        return Err(learned_basis_invalid("basis_algebra_failure"));
    }
    Ok(())
}

/// True when the transform commutes with both `M` and `J` within tolerance
/// (the gauge subgroup `diag(R, R)`, `R` in `O(3)`).
pub fn learned_basis_commutes_with_structure(transform: &RealMatrix) -> bool {
    let transposed = real_transpose(transform);
    let mirror = real_from_signed(mirror_matrix());
    let complex = real_from_signed(complex_structure_matrix());
    real_distance(
        &real_matmul(&real_matmul(&transposed, &mirror), transform),
        &mirror,
        1.0,
    ) <= LEARNED_BASIS_TOLERANCE
        && real_distance(
            &real_matmul(&real_matmul(&transposed, &complex), transform),
            &complex,
            1.0,
        ) <= LEARNED_BASIS_TOLERANCE
}

/// Validate one probe against the declared set and its transform.
pub fn validate_learned_basis_probe(probe: &LearnedBasisProbe) -> Result<(), EvalError> {
    if probe.index >= LEARNED_BASIS_PROBE_COUNT {
        return Err(learned_basis_invalid("probe_order_drift"));
    }
    let transform = learned_basis_transform(probe)?;
    if learned_basis_probes()[probe.index] != *probe {
        return Err(learned_basis_invalid("probe_order_drift"));
    }
    validate_learned_basis_transform(&transform)?;
    let commutes = learned_basis_commutes_with_structure(&transform);
    match probe.role {
        LearnedBasisProbeRole::Identity | LearnedBasisProbeRole::Gauge if !commutes => {
            Err(learned_basis_invalid("gauge_structure_drift"))
        }
        LearnedBasisProbeRole::SectorMixing | LearnedBasisProbeRole::Generic if commutes => {
            Err(learned_basis_invalid("probe_role_drift"))
        }
        _ => Ok(()),
    }
}

fn apply_learned_basis(transform: &RealMatrix, carrier: Chiral6) -> Result<Chiral6, EvalError> {
    let source = carrier.as_array();
    let mut rotated = [0.0; CHIRAL_WIDTH];
    for (row, value) in rotated.iter_mut().enumerate() {
        let mut sum = transform[row][0] * source[0];
        for column in 1..CHIRAL_WIDTH {
            sum += transform[row][column] * source[column];
        }
        *value = sum;
    }
    Chiral6::from_array(rotated).map_err(EvalError::ChiralNumerical)
}

/// Score an inference view with the C6 reference weights in the basis of one
/// prototype probe: `chiral_score(O q, O k)`, i.e. using `O^T M O`, `O^T J O`.
pub fn score_c6_in_learned_basis_from_view(
    view: &InferenceView,
    probe: &LearnedBasisProbe,
) -> Result<f64, EvalError> {
    validate_learned_basis_probe(probe)?;
    let transform = learned_basis_transform(probe)?;
    let query = Chiral6::from_array(view.query.as_array()).map_err(EvalError::ChiralNumerical)?;
    let key = Chiral6::from_array(view.key.as_array()).map_err(EvalError::ChiralNumerical)?;
    if probe.role == LearnedBasisProbeRole::Identity {
        // The identity transform is exact; score the untouched carrier.
        return chiral_score(query, key, C6_REFERENCE_WEIGHTS).map_err(EvalError::ChiralNumerical);
    }
    chiral_score(
        apply_learned_basis(&transform, query)?,
        apply_learned_basis(&transform, key)?,
        C6_REFERENCE_WEIGHTS,
    )
    .map_err(EvalError::ChiralNumerical)
}

/// One case scored under one prototype probe.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LearnedBasisCase {
    pub family: TaskFamily,
    pub case_id: u64,
    pub probe_index: usize,
    pub score: f64,
    /// Stage-C C6 reference score of the same case (canonical basis).
    pub reference_score: f64,
    pub correct: bool,
}

/// Per-probe, per-family correct counts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LearnedBasisFamilySummary {
    pub probe_index: usize,
    pub family: TaskFamily,
    pub n_cases: u64,
    pub correct: u64,
}

/// Immutable learned-basis prototype report on one non-final split.
#[derive(Clone, Debug, PartialEq)]
pub struct LearnedBasisPrototypeReport {
    pub prototype_contract: &'static str,
    pub split: DataSplit,
    pub budget: StageCPreflightBudget,
    pub probes: Vec<LearnedBasisProbe>,
    pub weights: ChiralScoreWeights,
    /// Scored capacity: the non-trained C6 reference under every probe.
    pub capacity: TrainableCapacity,
    /// Declared basis parameters of the prototype (`C(6,2) = 15` angles);
    /// none is trained here.
    pub basis_parameter_count: usize,
    /// Case-major, probe-minor order.
    pub cases: Vec<LearnedBasisCase>,
    pub summaries: Vec<LearnedBasisFamilySummary>,
    /// Must remain false.
    pub protected_or_final_access: bool,
    /// Must remain false: no angle is fitted.
    pub training_executed: bool,
    /// Must remain false.
    pub scientific_claim: bool,
    /// Must remain true.
    pub experimental_non_final: bool,
}

fn learned_basis_case<T, S>(
    case: &LabeledCase<T>,
    split: DataSplit,
    probes: &[LearnedBasisProbe; LEARNED_BASIS_PROBE_COUNT],
    oracle_sign: S,
    cases: &mut Vec<LearnedBasisCase>,
) -> Result<(), EvalError>
where
    S: Fn(&T) -> Result<i8, EvalError>,
{
    let view = case.inference_view();
    if view.split != split {
        return Err(EvalError::SplitMismatch {
            expected: split,
            actual: view.split,
        });
    }
    let sign = oracle_sign(case.protected_label().reveal_for_evaluation())?;
    if sign != 1 && sign != -1 {
        return Err(learned_basis_invalid("oracle_sign"));
    }
    let sign = f64::from(sign);
    let reference = run_inference_callback(case, score_c6_from_view)?;
    for probe in probes {
        let score = run_inference_callback(case, |view: &InferenceView| {
            score_c6_in_learned_basis_from_view(view, probe)
        })?;
        check_learned_basis_score(probe, score, reference)?;
        cases.push(LearnedBasisCase {
            family: view.family,
            case_id: view.case_id,
            probe_index: probe.index,
            score,
            reference_score: reference,
            correct: score * sign > 0.0,
        });
    }
    Ok(())
}

fn check_learned_basis_score(
    probe: &LearnedBasisProbe,
    score: f64,
    reference: f64,
) -> Result<(), EvalError> {
    match probe.role {
        LearnedBasisProbeRole::Identity if score.to_bits() != reference.to_bits() => {
            Err(learned_basis_invalid("canonical_reference_drift"))
        }
        LearnedBasisProbeRole::Gauge
            if (score - reference).abs() > LEARNED_BASIS_TOLERANCE * reference.abs().max(1.0) =>
        {
            Err(learned_basis_invalid("gauge_invariance_drift"))
        }
        _ => Ok(()),
    }
}

fn summarize_learned_basis(cases: &[LearnedBasisCase]) -> Vec<LearnedBasisFamilySummary> {
    let mut summaries =
        Vec::with_capacity(LEARNED_BASIS_PROBE_COUNT * STAGE_C_PREFLIGHT_FAMILIES.len());
    for probe_index in 0..LEARNED_BASIS_PROBE_COUNT {
        for family in STAGE_C_PREFLIGHT_FAMILIES {
            let members = cases
                .iter()
                .filter(|case| case.probe_index == probe_index && case.family == *family);
            summaries.push(LearnedBasisFamilySummary {
                probe_index,
                family: *family,
                n_cases: members.clone().count() as u64,
                correct: members.filter(|case| case.correct).count() as u64,
            });
        }
    }
    summaries
}

fn collect_learned_basis_cases(
    split: DataSplit,
    budget: StageCPreflightBudget,
) -> Result<Vec<LearnedBasisCase>, EvalError> {
    validate_non_final_split(split)?;
    budget.cases_per_arm()?;
    let probes = learned_basis_probes();
    for probe in &probes {
        validate_learned_basis_probe(probe)?;
    }
    let mut cases = Vec::new();
    let last_pair_id = budget.first_pair_id + budget.pairs_per_family;
    for pair_id in budget.first_pair_id..last_pair_id {
        let discriminative = reflection_discriminative_pair_in_split(pair_id, split)
            .map_err(|_| preflight_generation_failed())?;
        for member in [discriminative.right, discriminative.left] {
            learned_basis_case(
                &seal_reflection_discriminative(&member),
                split,
                &probes,
                handedness_sign,
                &mut cases,
            )?;
        }
        let nuisance = reflection_nuisance_pair_in_split(pair_id, split)
            .map_err(|_| preflight_generation_failed())?;
        for member in [nuisance.canonical, nuisance.reflected] {
            learned_basis_case(
                &seal_reflection_nuisance(&member),
                split,
                &probes,
                reflection_invariant_sign,
                &mut cases,
            )?;
        }
        let direction = direction_reversal_pair_in_split(pair_id, split)
            .map_err(|_| preflight_generation_failed())?;
        for member in [direction.forward, direction.reverse] {
            learned_basis_case(
                &seal_direction_reversal(&member),
                split,
                &probes,
                direction_sign,
                &mut cases,
            )?;
        }
        let base_case_id = pair_id
            .checked_mul(STAGE_C_PREFLIGHT_MEMBERS_PER_PAIR)
            .ok_or_else(preflight_generation_failed)?;
        for case_id in [base_case_id, base_case_id + 1] {
            let control = non_chiral_control_case_in_split(case_id, split)
                .map_err(|_| preflight_generation_failed())?;
            learned_basis_case(
                &seal_non_chiral_control(&control),
                split,
                &probes,
                non_chiral_sign,
                &mut cases,
            )?;
        }
    }
    Ok(cases)
}

/// Run the structure-preserving learned basis prototype on the bounded
/// Stage-C case stream. Every case is scored under every declared probe;
/// any identity failure aborts fail-closed and nothing is dropped.
pub fn run_learned_basis_prototype(
    split: DataSplit,
    budget: StageCPreflightBudget,
) -> Result<LearnedBasisPrototypeReport, EvalError> {
    let cases = collect_learned_basis_cases(split, budget)?;
    let summaries = summarize_learned_basis(&cases);
    let report = LearnedBasisPrototypeReport {
        prototype_contract: LEARNED_BASIS_PROTOTYPE_CONTRACT,
        split,
        budget,
        probes: learned_basis_probes().to_vec(),
        weights: C6_REFERENCE_WEIGHTS,
        capacity: TrainableCapacity::reference_c6(),
        basis_parameter_count: LEARNED_BASIS_PARAMETER_COUNT,
        cases,
        summaries,
        protected_or_final_access: false,
        training_executed: false,
        scientific_claim: false,
        experimental_non_final: true,
    };
    validate_learned_basis_prototype_report(&report)?;
    Ok(report)
}

/// Parse a split label first; protected/final labels never generate a case.
pub fn run_learned_basis_prototype_for_label(
    split_label: &str,
    budget: StageCPreflightBudget,
) -> Result<LearnedBasisPrototypeReport, EvalError> {
    run_learned_basis_prototype(parse_non_final_split(split_label)?, budget)
}

/// Validate a learned-basis prototype report: contract pin, the declared probe
/// set in order with orthogonality and algebra, weights/capacity/parameter
/// count, case-major/probe-minor coverage, identity and gauge identities,
/// recomputed summaries, flags, and regenerated per-case evidence.
pub fn validate_learned_basis_prototype_report(
    report: &LearnedBasisPrototypeReport,
) -> Result<(), EvalError> {
    validate_non_final_split(report.split)?;
    if report.prototype_contract != LEARNED_BASIS_PROTOTYPE_CONTRACT {
        return Err(learned_basis_invalid("contract_drift"));
    }
    if report.probes.len() != LEARNED_BASIS_PROBE_COUNT {
        return Err(learned_basis_invalid("probe_set_incomplete"));
    }
    for (position, probe) in report.probes.iter().enumerate() {
        if probe.index != position {
            return Err(learned_basis_invalid("probe_order_drift"));
        }
        validate_learned_basis_probe(probe)?;
    }
    if report.weights != C6_REFERENCE_WEIGHTS {
        return Err(learned_basis_invalid("weights_drift"));
    }
    if report.capacity != TrainableCapacity::reference_c6() {
        return Err(learned_basis_invalid("capacity_mismatch"));
    }
    if report.basis_parameter_count != LEARNED_BASIS_PARAMETER_COUNT {
        return Err(learned_basis_invalid("parameter_count_drift"));
    }
    let per_probe = report.budget.cases_per_arm()?;
    if report.cases.len() as u64 != per_probe * LEARNED_BASIS_PROBE_COUNT as u64 {
        return Err(learned_basis_invalid("case_count"));
    }
    for group in report.cases.chunks(LEARNED_BASIS_PROBE_COUNT) {
        for (probe_index, case) in group.iter().enumerate() {
            if case.probe_index != probe_index
                || case.family != group[0].family
                || case.case_id != group[0].case_id
                || case.reference_score.to_bits() != group[0].reference_score.to_bits()
            {
                return Err(learned_basis_invalid("case_order"));
            }
            check_learned_basis_score(
                &report.probes[probe_index],
                case.score,
                case.reference_score,
            )?;
        }
    }
    if report.summaries != summarize_learned_basis(&report.cases) {
        return Err(learned_basis_invalid("summary_drift"));
    }
    if report.protected_or_final_access {
        return Err(learned_basis_invalid("protected_or_final_access"));
    }
    if report.training_executed {
        return Err(learned_basis_invalid("training_executed"));
    }
    if report.scientific_claim {
        return Err(learned_basis_invalid("scientific_claim"));
    }
    if !report.experimental_non_final {
        return Err(learned_basis_invalid("experimental_non_final"));
    }
    if collect_learned_basis_cases(report.split, report.budget)? != report.cases {
        return Err(learned_basis_invalid("case_evidence_drift"));
    }
    Ok(())
}

/// Phase-D head-sharing ablation contract pin (slice 37).
pub const HEAD_SHARING_ABLATION_CONTRACT: &str = "tdi24-head-sharing-ablation-v1";

/// Declared head count of both arms of the head-sharing ablation. A software
/// configuration of this ablation, identical on both arms; not a freeze pin.
pub const HEAD_SHARING_HEAD_COUNT: usize = 2;

/// Per-head arm rule: head `h` uses the slice-35 fixed mirror basis of rank
/// `h` (canonical first, then the next basis in the frozen lexicographic
/// order). The shared arm uses rank 0 on every head. Not tuned, not selected.
pub const PER_HEAD_BASIS_RANKS: [usize; HEAD_SHARING_HEAD_COUNT] = [0, 1];

const fn head_sharing_invalid(reason: &'static str) -> EvalError {
    EvalError::HeadSharingAblationInvalid { reason }
}

/// Mean of per-head scores, summed in head order.
fn mean_head_score(scores: &[f64; HEAD_SHARING_HEAD_COUNT]) -> Result<f64, EvalError> {
    let mut sum = scores[0];
    for score in &scores[1..] {
        sum += score;
    }
    let mean = sum / HEAD_SHARING_HEAD_COUNT as f64;
    if mean.is_finite() {
        Ok(mean)
    } else {
        Err(head_sharing_invalid("non_finite_head_score"))
    }
}

/// Multi-head C6 score with one chiral structure shared by every head.
pub fn shared_head_c6_score_from_view(view: &InferenceView) -> Result<f64, EvalError> {
    let canonical = fixed_mirror_bases()[0];
    let head = score_c6_in_basis_from_view(view, &canonical)?;
    mean_head_score(&[head; HEAD_SHARING_HEAD_COUNT])
}

/// Per-head C6 head scores: head `h` uses basis `PER_HEAD_BASIS_RANKS[h]`.
pub fn per_head_c6_head_scores_from_view(
    view: &InferenceView,
) -> Result<[f64; HEAD_SHARING_HEAD_COUNT], EvalError> {
    let bases = fixed_mirror_bases();
    let mut scores = [0.0; HEAD_SHARING_HEAD_COUNT];
    for (head, rank) in PER_HEAD_BASIS_RANKS.iter().enumerate() {
        scores[head] = score_c6_in_basis_from_view(view, &bases[*rank])?;
    }
    Ok(scores)
}

/// Multi-head C6 score with a per-head chiral structure.
pub fn per_head_c6_score_from_view(view: &InferenceView) -> Result<f64, EvalError> {
    mean_head_score(&per_head_c6_head_scores_from_view(view)?)
}

/// One case scored by the shared and per-head multi-head arms.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HeadSharingCase {
    pub family: TaskFamily,
    pub case_id: u64,
    /// Single-head Stage-C C6 reference score.
    pub reference_score: f64,
    pub shared_score: f64,
    pub per_head_score: f64,
    /// Per-head arm head scores in head order.
    pub per_head_head_scores: [f64; HEAD_SHARING_HEAD_COUNT],
    pub shared_correct: bool,
    pub per_head_correct: bool,
}

/// Per-family correct counts of both arms.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HeadSharingFamilySummary {
    pub family: TaskFamily,
    pub n_cases: u64,
    pub shared_correct: u64,
    pub per_head_correct: u64,
    /// Cases where the two arms disagree on correctness.
    pub disagreements: u64,
}

/// Immutable head-sharing ablation report on one non-final split.
#[derive(Clone, Debug, PartialEq)]
pub struct HeadSharingAblationReport {
    pub ablation_contract: &'static str,
    pub basis_contract: &'static str,
    pub split: DataSplit,
    pub budget: StageCPreflightBudget,
    pub head_count: usize,
    pub shared_basis_ranks: [usize; HEAD_SHARING_HEAD_COUNT],
    pub per_head_basis_ranks: [usize; HEAD_SHARING_HEAD_COUNT],
    /// Identical weights on both arms.
    pub shared_weights: ChiralScoreWeights,
    pub per_head_weights: ChiralScoreWeights,
    /// Identical capacity on both arms: bases are fixed relabellings.
    pub shared_capacity: TrainableCapacity,
    pub per_head_capacity: TrainableCapacity,
    pub cases: Vec<HeadSharingCase>,
    pub families: Vec<HeadSharingFamilySummary>,
    /// Must remain false.
    pub protected_or_final_access: bool,
    /// Must remain false.
    pub training_executed: bool,
    /// Must remain false.
    pub scientific_claim: bool,
    /// Must remain true.
    pub experimental_non_final: bool,
}

fn head_sharing_case<T, S>(
    case: &LabeledCase<T>,
    split: DataSplit,
    oracle_sign: S,
    cases: &mut Vec<HeadSharingCase>,
) -> Result<(), EvalError>
where
    S: Fn(&T) -> Result<i8, EvalError>,
{
    let view = case.inference_view();
    if view.split != split {
        return Err(EvalError::SplitMismatch {
            expected: split,
            actual: view.split,
        });
    }
    let sign = oracle_sign(case.protected_label().reveal_for_evaluation())?;
    if sign != 1 && sign != -1 {
        return Err(head_sharing_invalid("oracle_sign"));
    }
    let sign = f64::from(sign);
    let reference_score = run_inference_callback(case, score_c6_from_view)?;
    let shared_score = run_inference_callback(case, shared_head_c6_score_from_view)?;
    let per_head_head_scores = run_inference_callback(case, per_head_c6_head_scores_from_view)?;
    let per_head_score = run_inference_callback(case, per_head_c6_score_from_view)?;
    let record = HeadSharingCase {
        family: view.family,
        case_id: view.case_id,
        reference_score,
        shared_score,
        per_head_score,
        per_head_head_scores,
        shared_correct: shared_score * sign > 0.0,
        per_head_correct: per_head_score * sign > 0.0,
    };
    check_head_sharing_case(&record)?;
    cases.push(record);
    Ok(())
}

fn check_head_sharing_case(case: &HeadSharingCase) -> Result<(), EvalError> {
    // Sharing one structure over identical heads is exactly the single head.
    if case.shared_score.to_bits() != case.reference_score.to_bits() {
        return Err(head_sharing_invalid("shared_reference_drift"));
    }
    // Head 0 of the per-head arm is the canonical basis.
    if case.per_head_head_scores[0].to_bits() != case.reference_score.to_bits() {
        return Err(head_sharing_invalid("canonical_head_drift"));
    }
    if mean_head_score(&case.per_head_head_scores)?.to_bits() != case.per_head_score.to_bits() {
        return Err(head_sharing_invalid("head_mean_drift"));
    }
    Ok(())
}

fn summarize_head_sharing(cases: &[HeadSharingCase]) -> Vec<HeadSharingFamilySummary> {
    STAGE_C_PREFLIGHT_FAMILIES
        .iter()
        .map(|family| {
            let members = cases.iter().filter(|case| case.family == *family);
            HeadSharingFamilySummary {
                family: *family,
                n_cases: members.clone().count() as u64,
                shared_correct: members.clone().filter(|c| c.shared_correct).count() as u64,
                per_head_correct: members.clone().filter(|c| c.per_head_correct).count() as u64,
                disagreements: members
                    .filter(|c| c.shared_correct != c.per_head_correct)
                    .count() as u64,
            }
        })
        .collect()
}

fn collect_head_sharing_cases(
    split: DataSplit,
    budget: StageCPreflightBudget,
) -> Result<Vec<HeadSharingCase>, EvalError> {
    validate_non_final_split(split)?;
    budget.cases_per_arm()?;
    let mut cases = Vec::new();
    let last_pair_id = budget.first_pair_id + budget.pairs_per_family;
    for pair_id in budget.first_pair_id..last_pair_id {
        let discriminative = reflection_discriminative_pair_in_split(pair_id, split)
            .map_err(|_| preflight_generation_failed())?;
        for member in [discriminative.right, discriminative.left] {
            head_sharing_case(
                &seal_reflection_discriminative(&member),
                split,
                handedness_sign,
                &mut cases,
            )?;
        }
        let nuisance = reflection_nuisance_pair_in_split(pair_id, split)
            .map_err(|_| preflight_generation_failed())?;
        for member in [nuisance.canonical, nuisance.reflected] {
            head_sharing_case(
                &seal_reflection_nuisance(&member),
                split,
                reflection_invariant_sign,
                &mut cases,
            )?;
        }
        let direction = direction_reversal_pair_in_split(pair_id, split)
            .map_err(|_| preflight_generation_failed())?;
        for member in [direction.forward, direction.reverse] {
            head_sharing_case(
                &seal_direction_reversal(&member),
                split,
                direction_sign,
                &mut cases,
            )?;
        }
        let base_case_id = pair_id
            .checked_mul(STAGE_C_PREFLIGHT_MEMBERS_PER_PAIR)
            .ok_or_else(preflight_generation_failed)?;
        for case_id in [base_case_id, base_case_id + 1] {
            let control = non_chiral_control_case_in_split(case_id, split)
                .map_err(|_| preflight_generation_failed())?;
            head_sharing_case(
                &seal_non_chiral_control(&control),
                split,
                non_chiral_sign,
                &mut cases,
            )?;
        }
    }
    Ok(cases)
}

/// Run the head-sharing ablation on the bounded Stage-C case stream.
pub fn run_head_sharing_ablation(
    split: DataSplit,
    budget: StageCPreflightBudget,
) -> Result<HeadSharingAblationReport, EvalError> {
    let cases = collect_head_sharing_cases(split, budget)?;
    let families = summarize_head_sharing(&cases);
    let report = HeadSharingAblationReport {
        ablation_contract: HEAD_SHARING_ABLATION_CONTRACT,
        basis_contract: FIXED_M_SENSITIVITY_CONTRACT,
        split,
        budget,
        head_count: HEAD_SHARING_HEAD_COUNT,
        shared_basis_ranks: [0; HEAD_SHARING_HEAD_COUNT],
        per_head_basis_ranks: PER_HEAD_BASIS_RANKS,
        shared_weights: C6_REFERENCE_WEIGHTS,
        per_head_weights: C6_REFERENCE_WEIGHTS,
        shared_capacity: TrainableCapacity::reference_c6(),
        per_head_capacity: TrainableCapacity::reference_c6(),
        cases,
        families,
        protected_or_final_access: false,
        training_executed: false,
        scientific_claim: false,
        experimental_non_final: true,
    };
    validate_head_sharing_ablation_report(&report)?;
    Ok(report)
}

/// Parse a split label first; protected/final labels never generate a case.
pub fn run_head_sharing_ablation_for_label(
    split_label: &str,
    budget: StageCPreflightBudget,
) -> Result<HeadSharingAblationReport, EvalError> {
    run_head_sharing_ablation(parse_non_final_split(split_label)?, budget)
}

/// Validate a head-sharing ablation report: pins, head count and basis
/// ranks, identical weights and capacity, case identities, recomputed
/// summaries, flags and regenerated evidence.
pub fn validate_head_sharing_ablation_report(
    report: &HeadSharingAblationReport,
) -> Result<(), EvalError> {
    validate_non_final_split(report.split)?;
    if report.ablation_contract != HEAD_SHARING_ABLATION_CONTRACT {
        return Err(head_sharing_invalid("contract_drift"));
    }
    if report.basis_contract != FIXED_M_SENSITIVITY_CONTRACT {
        return Err(head_sharing_invalid("basis_contract_drift"));
    }
    if report.head_count != HEAD_SHARING_HEAD_COUNT {
        return Err(head_sharing_invalid("head_count_drift"));
    }
    if report.shared_basis_ranks != [0; HEAD_SHARING_HEAD_COUNT]
        || report.per_head_basis_ranks != PER_HEAD_BASIS_RANKS
    {
        return Err(head_sharing_invalid("basis_rank_drift"));
    }
    if report.shared_weights != C6_REFERENCE_WEIGHTS
        || report.per_head_weights != report.shared_weights
    {
        return Err(head_sharing_invalid("weights_drift"));
    }
    if report.shared_capacity != TrainableCapacity::reference_c6()
        || report.per_head_capacity != report.shared_capacity
    {
        return Err(head_sharing_invalid("capacity_mismatch"));
    }
    let expected = report.budget.cases_per_arm()?;
    if report.cases.len() as u64 != expected {
        return Err(head_sharing_invalid("case_count"));
    }
    for case in &report.cases {
        check_head_sharing_case(case)?;
    }
    if report.families != summarize_head_sharing(&report.cases) {
        return Err(head_sharing_invalid("summary_drift"));
    }
    if report.protected_or_final_access {
        return Err(head_sharing_invalid("protected_or_final_access"));
    }
    if report.training_executed {
        return Err(head_sharing_invalid("training_executed"));
    }
    if report.scientific_claim {
        return Err(head_sharing_invalid("scientific_claim"));
    }
    if !report.experimental_non_final {
        return Err(head_sharing_invalid("experimental_non_final"));
    }
    if collect_head_sharing_cases(report.split, report.budget)? != report.cases {
        return Err(head_sharing_invalid("case_evidence_drift"));
    }
    Ok(())
}

/// Phase-D width-scaling contract pin (slice 38).
pub const WIDTH_SCALING_CONTRACT: &str = "tdi24-width-scaling-v1";

/// Number of matched widths: every sector width `n = 1..=3` of the carrier.
pub const WIDTH_SCALING_WIDTH_COUNT: usize = 3;

/// Preregistered matched carrier widths `2n`, `n = 1..=3`: all even widths up
/// to the carrier width, derived from it, evaluated together, never selected.
pub const WIDTH_SCALING_WIDTHS: [usize; WIDTH_SCALING_WIDTH_COUNT] = [2, 4, 6];

const fn width_scaling_invalid(reason: &'static str) -> EvalError {
    EvalError::WidthScalingInvalid { reason }
}

/// Restrict a carrier to the first `width / 2` slots of each sector (the
/// remaining slots are zero), so `M` and `J` act on the matched sub-carrier.
pub fn restrict_carrier_to_width(carrier: Chiral6, width: usize) -> Result<Chiral6, EvalError> {
    if !WIDTH_SCALING_WIDTHS.contains(&width) {
        return Err(width_scaling_invalid("width_not_registered"));
    }
    if width == CHIRAL_WIDTH {
        return Ok(carrier);
    }
    let half = width / 2;
    let mut even = carrier.even();
    let mut odd = carrier.odd();
    for slot in half..3 {
        even[slot] = 0.0;
        odd[slot] = 0.0;
    }
    Chiral6::new(even, odd).map_err(EvalError::ChiralNumerical)
}

/// Score a view at one matched width with the given weights.
pub fn score_at_width_from_view(
    view: &InferenceView,
    width: usize,
    weights: ChiralScoreWeights,
) -> Result<f64, EvalError> {
    let query = Chiral6::from_array(view.query.as_array()).map_err(EvalError::ChiralNumerical)?;
    let key = Chiral6::from_array(view.key.as_array()).map_err(EvalError::ChiralNumerical)?;
    chiral_score(
        restrict_carrier_to_width(query, width)?,
        restrict_carrier_to_width(key, width)?,
        weights,
    )
    .map_err(EvalError::ChiralNumerical)
}

/// One case scored by C6 and its direct-only matched arm at every width.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WidthScalingCase {
    pub family: TaskFamily,
    pub case_id: u64,
    /// Stage-C C6 reference score at full width.
    pub reference_score: f64,
    pub c6_scores: [f64; WIDTH_SCALING_WIDTH_COUNT],
    pub direct_scores: [f64; WIDTH_SCALING_WIDTH_COUNT],
    pub c6_correct: [bool; WIDTH_SCALING_WIDTH_COUNT],
    pub direct_correct: [bool; WIDTH_SCALING_WIDTH_COUNT],
}

/// Per-width, per-family correct counts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WidthScalingFamilySummary {
    pub width: usize,
    pub family: TaskFamily,
    pub n_cases: u64,
    pub c6_correct: u64,
    pub direct_correct: u64,
}

/// Immutable width-scaling report on one non-final split.
#[derive(Clone, Debug, PartialEq)]
pub struct WidthScalingReport {
    pub scaling_contract: &'static str,
    pub split: DataSplit,
    pub budget: StageCPreflightBudget,
    pub widths: [usize; WIDTH_SCALING_WIDTH_COUNT],
    pub c6_weights: ChiralScoreWeights,
    pub direct_weights: ChiralScoreWeights,
    /// Zero trainable parameters for both arms at every width.
    pub c6_capacity: TrainableCapacity,
    pub direct_capacity: TrainableCapacity,
    pub cases: Vec<WidthScalingCase>,
    pub summaries: Vec<WidthScalingFamilySummary>,
    /// Must remain false.
    pub protected_or_final_access: bool,
    /// Must remain false.
    pub training_executed: bool,
    /// Must remain false: no width is selected.
    pub scientific_claim: bool,
    /// Must remain true.
    pub experimental_non_final: bool,
}

fn width_scaling_case<T, S>(
    case: &LabeledCase<T>,
    split: DataSplit,
    oracle_sign: S,
    cases: &mut Vec<WidthScalingCase>,
) -> Result<(), EvalError>
where
    S: Fn(&T) -> Result<i8, EvalError>,
{
    let view = case.inference_view();
    if view.split != split {
        return Err(EvalError::SplitMismatch {
            expected: split,
            actual: view.split,
        });
    }
    let sign = oracle_sign(case.protected_label().reveal_for_evaluation())?;
    if sign != 1 && sign != -1 {
        return Err(width_scaling_invalid("oracle_sign"));
    }
    let sign = f64::from(sign);
    let reference_score = run_inference_callback(case, score_c6_from_view)?;
    let direct_weights = direct_only_weights(C6_REFERENCE_WEIGHTS);
    let mut record = WidthScalingCase {
        family: view.family,
        case_id: view.case_id,
        reference_score,
        c6_scores: [0.0; WIDTH_SCALING_WIDTH_COUNT],
        direct_scores: [0.0; WIDTH_SCALING_WIDTH_COUNT],
        c6_correct: [false; WIDTH_SCALING_WIDTH_COUNT],
        direct_correct: [false; WIDTH_SCALING_WIDTH_COUNT],
    };
    for (index, width) in WIDTH_SCALING_WIDTHS.iter().enumerate() {
        let c6 = run_inference_callback(case, |view: &InferenceView| {
            score_at_width_from_view(view, *width, C6_REFERENCE_WEIGHTS)
        })?;
        let direct = run_inference_callback(case, |view: &InferenceView| {
            score_at_width_from_view(view, *width, direct_weights)
        })?;
        record.c6_scores[index] = c6;
        record.direct_scores[index] = direct;
        record.c6_correct[index] = c6 * sign > 0.0;
        record.direct_correct[index] = direct * sign > 0.0;
    }
    if record.c6_scores[WIDTH_SCALING_WIDTH_COUNT - 1].to_bits() != reference_score.to_bits() {
        return Err(width_scaling_invalid("full_width_reference_drift"));
    }
    cases.push(record);
    Ok(())
}

fn summarize_width_scaling(cases: &[WidthScalingCase]) -> Vec<WidthScalingFamilySummary> {
    let mut summaries = Vec::new();
    for (index, width) in WIDTH_SCALING_WIDTHS.iter().enumerate() {
        for family in STAGE_C_PREFLIGHT_FAMILIES {
            let members = cases.iter().filter(|case| case.family == *family);
            summaries.push(WidthScalingFamilySummary {
                width: *width,
                family: *family,
                n_cases: members.clone().count() as u64,
                c6_correct: members.clone().filter(|c| c.c6_correct[index]).count() as u64,
                direct_correct: members.filter(|c| c.direct_correct[index]).count() as u64,
            });
        }
    }
    summaries
}

fn collect_width_scaling_cases(
    split: DataSplit,
    budget: StageCPreflightBudget,
) -> Result<Vec<WidthScalingCase>, EvalError> {
    validate_non_final_split(split)?;
    budget.cases_per_arm()?;
    let mut cases = Vec::new();
    let last_pair_id = budget.first_pair_id + budget.pairs_per_family;
    for pair_id in budget.first_pair_id..last_pair_id {
        let discriminative = reflection_discriminative_pair_in_split(pair_id, split)
            .map_err(|_| preflight_generation_failed())?;
        for member in [discriminative.right, discriminative.left] {
            width_scaling_case(
                &seal_reflection_discriminative(&member),
                split,
                handedness_sign,
                &mut cases,
            )?;
        }
        let nuisance = reflection_nuisance_pair_in_split(pair_id, split)
            .map_err(|_| preflight_generation_failed())?;
        for member in [nuisance.canonical, nuisance.reflected] {
            width_scaling_case(
                &seal_reflection_nuisance(&member),
                split,
                reflection_invariant_sign,
                &mut cases,
            )?;
        }
        let direction = direction_reversal_pair_in_split(pair_id, split)
            .map_err(|_| preflight_generation_failed())?;
        for member in [direction.forward, direction.reverse] {
            width_scaling_case(
                &seal_direction_reversal(&member),
                split,
                direction_sign,
                &mut cases,
            )?;
        }
        let base_case_id = pair_id
            .checked_mul(STAGE_C_PREFLIGHT_MEMBERS_PER_PAIR)
            .ok_or_else(preflight_generation_failed)?;
        for case_id in [base_case_id, base_case_id + 1] {
            let control = non_chiral_control_case_in_split(case_id, split)
                .map_err(|_| preflight_generation_failed())?;
            width_scaling_case(
                &seal_non_chiral_control(&control),
                split,
                non_chiral_sign,
                &mut cases,
            )?;
        }
    }
    Ok(cases)
}

/// Run the width-scaling study on the bounded Stage-C case stream.
pub fn run_width_scaling(
    split: DataSplit,
    budget: StageCPreflightBudget,
) -> Result<WidthScalingReport, EvalError> {
    let cases = collect_width_scaling_cases(split, budget)?;
    let summaries = summarize_width_scaling(&cases);
    let report = WidthScalingReport {
        scaling_contract: WIDTH_SCALING_CONTRACT,
        split,
        budget,
        widths: WIDTH_SCALING_WIDTHS,
        c6_weights: C6_REFERENCE_WEIGHTS,
        direct_weights: direct_only_weights(C6_REFERENCE_WEIGHTS),
        c6_capacity: TrainableCapacity::reference_c6(),
        direct_capacity: TrainableCapacity::reference_c6(),
        cases,
        summaries,
        protected_or_final_access: false,
        training_executed: false,
        scientific_claim: false,
        experimental_non_final: true,
    };
    validate_width_scaling_report(&report)?;
    Ok(report)
}

/// Parse a split label first; protected/final labels never generate a case.
pub fn run_width_scaling_for_label(
    split_label: &str,
    budget: StageCPreflightBudget,
) -> Result<WidthScalingReport, EvalError> {
    run_width_scaling(parse_non_final_split(split_label)?, budget)
}

/// Validate a width-scaling report: pin, the complete preregistered width
/// set, weights and capacity, full-width reproduction, summaries, flags and
/// regenerated evidence.
pub fn validate_width_scaling_report(report: &WidthScalingReport) -> Result<(), EvalError> {
    validate_non_final_split(report.split)?;
    if report.scaling_contract != WIDTH_SCALING_CONTRACT {
        return Err(width_scaling_invalid("contract_drift"));
    }
    if report.widths != WIDTH_SCALING_WIDTHS {
        return Err(width_scaling_invalid("width_set_drift"));
    }
    if report.c6_weights != C6_REFERENCE_WEIGHTS
        || report.direct_weights != direct_only_weights(C6_REFERENCE_WEIGHTS)
    {
        return Err(width_scaling_invalid("weights_drift"));
    }
    if report.c6_capacity != TrainableCapacity::reference_c6()
        || report.direct_capacity != report.c6_capacity
    {
        return Err(width_scaling_invalid("capacity_mismatch"));
    }
    if report.cases.len() as u64 != report.budget.cases_per_arm()? {
        return Err(width_scaling_invalid("case_count"));
    }
    for case in &report.cases {
        if case.c6_scores[WIDTH_SCALING_WIDTH_COUNT - 1].to_bits() != case.reference_score.to_bits()
        {
            return Err(width_scaling_invalid("full_width_reference_drift"));
        }
    }
    if report.summaries != summarize_width_scaling(&report.cases) {
        return Err(width_scaling_invalid("summary_drift"));
    }
    if report.protected_or_final_access {
        return Err(width_scaling_invalid("protected_or_final_access"));
    }
    if report.training_executed {
        return Err(width_scaling_invalid("training_executed"));
    }
    if report.scientific_claim {
        return Err(width_scaling_invalid("scientific_claim"));
    }
    if !report.experimental_non_final {
        return Err(width_scaling_invalid("experimental_non_final"));
    }
    if collect_width_scaling_cases(report.split, report.budget)? != report.cases {
        return Err(width_scaling_invalid("case_evidence_drift"));
    }
    Ok(())
}

/// Phase-D sequence-length scaling contract pin (slice 39).
pub const SEQUENCE_LENGTH_SCALING_CONTRACT: &str = "tdi24-sequence-length-scaling-v1";

/// Number of preregistered sequence lengths.
pub const SEQUENCE_LENGTH_COUNT: usize = 3;

/// Preregistered matched sequence lengths, identical for every arm and mask
/// policy, all reported, none selected. A declared software configuration of
/// this study, not a freeze pin.
pub const SEQUENCE_LENGTHS: [usize; SEQUENCE_LENGTH_COUNT] = [2, 4, 8];

/// Shared mask policies, both evaluated for every arm and length.
pub const SEQUENCE_MASK_POLICIES: [MaskPolicy; 2] = [MaskPolicy::Full, MaskPolicy::Causal];

/// Declared tolerance for the row-sum check of the shared normalizer.
pub const SEQUENCE_ROW_SUM_TOLERANCE: f64 = 1e-12;

const fn sequence_invalid(reason: &'static str) -> EvalError {
    EvalError::SequenceLengthScalingInvalid { reason }
}

/// Scoring arm of the sequence-length study.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SequenceArm {
    /// C6 reference weights `(1, 0, 1)`.
    C6,
    /// Direct-only matched arm `(1, 0, 0)`.
    DirectOnly,
}

/// Both arms in fixed order.
pub const SEQUENCE_ARMS: [SequenceArm; 2] = [SequenceArm::C6, SequenceArm::DirectOnly];

impl SequenceArm {
    #[must_use]
    pub const fn weights(self) -> ChiralScoreWeights {
        match self {
            Self::C6 => C6_REFERENCE_WEIGHTS,
            Self::DirectOnly => direct_only_weights(C6_REFERENCE_WEIGHTS),
        }
    }
}

/// Bounded cost and outcome accounting for one (length, mask, arm, family).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SequenceCell {
    pub length: usize,
    pub policy: MaskPolicy,
    pub arm: SequenceArm,
    pub family: TaskFamily,
    /// Non-overlapping windows of `length` consecutive cases of the family.
    pub windows: u64,
    /// Family cases left outside a full window (recorded, never scored).
    pub unwindowed_cases: u64,
    /// Pairwise score evaluations: `windows * length^2`.
    pub score_evaluations: u64,
    /// Normalizer calls: `windows * length`.
    pub normalizer_calls: u64,
    /// Attention entries forced to zero by the mask.
    pub masked_entries: u64,
    /// Rows whose maximal attention weight is on the row's own key.
    pub self_retrieval_rows: u64,
    /// Max `|sum(row) - 1|` over all rows.
    pub max_row_sum_error: f64,
}

/// Immutable sequence-length scaling report on one non-final split.
#[derive(Clone, Debug, PartialEq)]
pub struct SequenceLengthScalingReport {
    pub scaling_contract: &'static str,
    pub normalizer_contract: &'static str,
    pub masking_contract: &'static str,
    pub split: DataSplit,
    pub budget: StageCPreflightBudget,
    pub lengths: [usize; SEQUENCE_LENGTH_COUNT],
    /// Zero trainable parameters for both arms at every length.
    pub capacity: TrainableCapacity,
    pub cells: Vec<SequenceCell>,
    /// Must remain false.
    pub protected_or_final_access: bool,
    /// Must remain false.
    pub training_executed: bool,
    /// Must remain false: no length is selected.
    pub scientific_claim: bool,
    /// Must remain true.
    pub experimental_non_final: bool,
}

type SequenceItem = (TaskFamily, Chiral6, Chiral6, f64);

fn sequence_item<T>(case: &LabeledCase<T>, split: DataSplit) -> Result<SequenceItem, EvalError> {
    let view = case.inference_view();
    if view.split != split {
        return Err(EvalError::SplitMismatch {
            expected: split,
            actual: view.split,
        });
    }
    let query = Chiral6::from_array(view.query.as_array()).map_err(EvalError::ChiralNumerical)?;
    let key = Chiral6::from_array(view.key.as_array()).map_err(EvalError::ChiralNumerical)?;
    let reference = run_inference_callback(case, score_c6_from_view)?;
    Ok((view.family, query, key, reference))
}

fn collect_sequence_items(
    split: DataSplit,
    budget: StageCPreflightBudget,
) -> Result<Vec<SequenceItem>, EvalError> {
    validate_non_final_split(split)?;
    budget.cases_per_arm()?;
    let mut items = Vec::new();
    let last_pair_id = budget.first_pair_id + budget.pairs_per_family;
    for pair_id in budget.first_pair_id..last_pair_id {
        let discriminative = reflection_discriminative_pair_in_split(pair_id, split)
            .map_err(|_| preflight_generation_failed())?;
        for member in [discriminative.right, discriminative.left] {
            items.push(sequence_item(
                &seal_reflection_discriminative(&member),
                split,
            )?);
        }
        let nuisance = reflection_nuisance_pair_in_split(pair_id, split)
            .map_err(|_| preflight_generation_failed())?;
        for member in [nuisance.canonical, nuisance.reflected] {
            items.push(sequence_item(&seal_reflection_nuisance(&member), split)?);
        }
        let direction = direction_reversal_pair_in_split(pair_id, split)
            .map_err(|_| preflight_generation_failed())?;
        for member in [direction.forward, direction.reverse] {
            items.push(sequence_item(&seal_direction_reversal(&member), split)?);
        }
        let base_case_id = pair_id
            .checked_mul(STAGE_C_PREFLIGHT_MEMBERS_PER_PAIR)
            .ok_or_else(preflight_generation_failed)?;
        for case_id in [base_case_id, base_case_id + 1] {
            let control = non_chiral_control_case_in_split(case_id, split)
                .map_err(|_| preflight_generation_failed())?;
            items.push(sequence_item(&seal_non_chiral_control(&control), split)?);
        }
    }
    Ok(items)
}

/// Attention rows of one window: row `i` is the shared masked softmax of the
/// logits `score(q_i, k_j)` under the arm's weights.
pub fn sequence_window_rows(
    queries: &[Chiral6],
    keys: &[Chiral6],
    policy: MaskPolicy,
    arm: SequenceArm,
) -> Result<Vec<Vec<f64>>, EvalError> {
    normalize_sequence_logits(&sequence_window_logits(queries, keys, arm)?, policy)
}

/// Logit matrix of one window: entry `(i, j)` is `score(q_i, k_j)`. Exactly
/// `L^2` score evaluations; nothing else in the slice calls the scorer.
pub fn sequence_window_logits(
    queries: &[Chiral6],
    keys: &[Chiral6],
    arm: SequenceArm,
) -> Result<Vec<Vec<f64>>, EvalError> {
    if queries.len() != keys.len() || !SEQUENCE_LENGTHS.contains(&queries.len()) {
        return Err(sequence_invalid("length_not_registered"));
    }
    let mut logits = Vec::with_capacity(queries.len());
    for query in queries {
        let mut row = Vec::with_capacity(keys.len());
        for key in keys {
            row.push(
                chiral_score(*query, *key, arm.weights()).map_err(EvalError::ChiralNumerical)?,
            );
        }
        logits.push(row);
    }
    Ok(logits)
}

fn normalize_sequence_logits(
    logits: &[Vec<f64>],
    policy: MaskPolicy,
) -> Result<Vec<Vec<f64>>, EvalError> {
    logits
        .iter()
        .enumerate()
        .map(|(row, values)| {
            normalize_with_policy(values, policy, row)
                .map_err(|_| sequence_invalid("normalizer_failure"))
        })
        .collect()
}

fn sequence_cell(
    items: &[SequenceItem],
    length: usize,
    policy: MaskPolicy,
    arm: SequenceArm,
    family: TaskFamily,
) -> Result<SequenceCell, EvalError> {
    let members: Vec<&SequenceItem> = items.iter().filter(|item| item.0 == family).collect();
    let windows = members.len() / length;
    let mut cell = SequenceCell {
        length,
        policy,
        arm,
        family,
        windows: windows as u64,
        unwindowed_cases: (members.len() - windows * length) as u64,
        score_evaluations: 0,
        normalizer_calls: 0,
        masked_entries: 0,
        self_retrieval_rows: 0,
        max_row_sum_error: 0.0,
    };
    for window in members.chunks_exact(length) {
        let queries: Vec<Chiral6> = window.iter().map(|item| item.1).collect();
        let keys: Vec<Chiral6> = window.iter().map(|item| item.2).collect();
        let logits = sequence_window_logits(&queries, &keys, arm)?;
        for (index, item) in window.iter().enumerate() {
            // The diagonal logit (already counted among the L^2 evaluations)
            // is the single-case Stage-C C6 score.
            if arm == SequenceArm::C6 && logits[index][index].to_bits() != item.3.to_bits() {
                return Err(sequence_invalid("diagonal_reference_drift"));
            }
        }
        let rows = normalize_sequence_logits(&logits, policy)?;
        cell.score_evaluations += (length * length) as u64;
        cell.normalizer_calls += length as u64;
        for (row_index, row) in rows.iter().enumerate() {
            let sum: f64 = row.iter().sum();
            cell.max_row_sum_error = cell.max_row_sum_error.max((sum - 1.0).abs());
            let zeros = row.iter().filter(|value| **value == 0.0).count();
            let expected_masked = match policy {
                MaskPolicy::Full => 0,
                MaskPolicy::Causal => length - 1 - row_index,
            };
            if zeros < expected_masked {
                return Err(sequence_invalid("mask_drift"));
            }
            cell.masked_entries += expected_masked as u64;
            let best =
                row.iter().enumerate().fold(
                    0,
                    |best, (index, value)| if *value > row[best] { index } else { best },
                );
            if best == row_index {
                cell.self_retrieval_rows += 1;
            }
        }
    }
    if cell.max_row_sum_error > SEQUENCE_ROW_SUM_TOLERANCE {
        return Err(sequence_invalid("row_sum_drift"));
    }
    Ok(cell)
}

fn collect_sequence_cells(
    split: DataSplit,
    budget: StageCPreflightBudget,
) -> Result<Vec<SequenceCell>, EvalError> {
    let items = collect_sequence_items(split, budget)?;
    let mut cells = Vec::new();
    for length in SEQUENCE_LENGTHS {
        for policy in SEQUENCE_MASK_POLICIES {
            for arm in SEQUENCE_ARMS {
                for family in STAGE_C_PREFLIGHT_FAMILIES {
                    cells.push(sequence_cell(&items, length, policy, arm, *family)?);
                }
            }
        }
    }
    Ok(cells)
}

/// Run the sequence-length scaling study on the bounded Stage-C case stream.
pub fn run_sequence_length_scaling(
    split: DataSplit,
    budget: StageCPreflightBudget,
) -> Result<SequenceLengthScalingReport, EvalError> {
    let cells = collect_sequence_cells(split, budget)?;
    let report = SequenceLengthScalingReport {
        scaling_contract: SEQUENCE_LENGTH_SCALING_CONTRACT,
        normalizer_contract: NORMALIZER_CONTRACT,
        masking_contract: MASKING_CONTRACT,
        split,
        budget,
        lengths: SEQUENCE_LENGTHS,
        capacity: TrainableCapacity::reference_c6(),
        cells,
        protected_or_final_access: false,
        training_executed: false,
        scientific_claim: false,
        experimental_non_final: true,
    };
    validate_sequence_length_scaling_report(&report)?;
    Ok(report)
}

/// Parse a split label first; protected/final labels never generate a case.
pub fn run_sequence_length_scaling_for_label(
    split_label: &str,
    budget: StageCPreflightBudget,
) -> Result<SequenceLengthScalingReport, EvalError> {
    run_sequence_length_scaling(parse_non_final_split(split_label)?, budget)
}

/// Validate a sequence-length scaling report: pins, length set, capacity,
/// exact cost accounting per cell, row-sum bound, flags and regenerated
/// evidence.
pub fn validate_sequence_length_scaling_report(
    report: &SequenceLengthScalingReport,
) -> Result<(), EvalError> {
    validate_non_final_split(report.split)?;
    if report.scaling_contract != SEQUENCE_LENGTH_SCALING_CONTRACT {
        return Err(sequence_invalid("contract_drift"));
    }
    if report.normalizer_contract != NORMALIZER_CONTRACT
        || report.masking_contract != MASKING_CONTRACT
    {
        return Err(sequence_invalid("normalizer_contract_drift"));
    }
    if report.lengths != SEQUENCE_LENGTHS {
        return Err(sequence_invalid("length_set_drift"));
    }
    if report.capacity != TrainableCapacity::reference_c6() {
        return Err(sequence_invalid("capacity_mismatch"));
    }
    let expected_cells = SEQUENCE_LENGTH_COUNT
        * SEQUENCE_MASK_POLICIES.len()
        * SEQUENCE_ARMS.len()
        * STAGE_C_PREFLIGHT_FAMILIES.len();
    if report.cells.len() != expected_cells {
        return Err(sequence_invalid("cell_count"));
    }
    let per_family = report.budget.cases_per_arm()? / STAGE_C_PREFLIGHT_FAMILIES.len() as u64;
    for cell in &report.cells {
        // Reject unregistered lengths before any arithmetic on them.
        if !SEQUENCE_LENGTHS.contains(&cell.length) {
            return Err(sequence_invalid("length_not_registered"));
        }
        let length = cell.length as u64;
        let masked_per_window = match cell.policy {
            MaskPolicy::Full => 0,
            MaskPolicy::Causal => length * (length - 1) / 2,
        };
        if cell.windows * length + cell.unwindowed_cases != per_family
            || cell.unwindowed_cases >= length
            || cell.score_evaluations != cell.windows * length * length
            || cell.normalizer_calls != cell.windows * length
            || cell.masked_entries != cell.windows * masked_per_window
            || cell.self_retrieval_rows > cell.normalizer_calls
        {
            return Err(sequence_invalid("cost_accounting_drift"));
        }
        if cell.max_row_sum_error.is_nan() || cell.max_row_sum_error > SEQUENCE_ROW_SUM_TOLERANCE {
            return Err(sequence_invalid("row_sum_drift"));
        }
    }
    if report.protected_or_final_access {
        return Err(sequence_invalid("protected_or_final_access"));
    }
    if report.training_executed {
        return Err(sequence_invalid("training_executed"));
    }
    if report.scientific_claim {
        return Err(sequence_invalid("scientific_claim"));
    }
    if !report.experimental_non_final {
        return Err(sequence_invalid("experimental_non_final"));
    }
    if collect_sequence_cells(report.split, report.budget)? != report.cells {
        return Err(sequence_invalid("case_evidence_drift"));
    }
    Ok(())
}

/// Phase-D Stage-D attribution audit contract pin (slice 40).
pub const STAGE_D_ATTRIBUTION_AUDIT_CONTRACT: &str = "tdi24-stage-d-attribution-audit-v1";

/// Audited Phase-C/D slices in campaign order (slices 30 through 39) with
/// their frozen contract pins. Nothing outside this registry is audited.
pub const ATTRIBUTION_AUDITED_SLICES: [(u8, &str); 10] = [
    (30, STAGE_C_PREFLIGHT_CONTRACT),
    (31, GAMMA_ZERO_ABLATION_CONTRACT),
    (32, BETA_ZERO_ABLATION_CONTRACT),
    (33, DIRECT_ONLY_COLLAPSE_CONTRACT),
    (34, PARITY_SHUFFLE_CONTROL_CONTRACT),
    (35, FIXED_M_SENSITIVITY_CONTRACT),
    (36, LEARNED_BASIS_PROTOTYPE_CONTRACT),
    (37, HEAD_SHARING_ABLATION_CONTRACT),
    (38, WIDTH_SCALING_CONTRACT),
    (39, SEQUENCE_LENGTH_SCALING_CONTRACT),
];

/// Claim class an audited slice may support after the ablations.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AttributionClaim {
    /// The validated report supports only software-semantics statements
    /// (contracts, accounting, fail-closed behavior) on Development/Validation.
    SoftwareSemanticsOnly,
    /// Withheld: the report failed validation or set a forbidden flag.
    Withheld,
}

/// One audited slice.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AttributionEntry {
    pub slice: u8,
    pub contract: &'static str,
    /// The upstream validator accepted the regenerated report.
    pub validated: bool,
    /// FNV-1a 64 digest of the regenerated report's `Debug` rendering; binds
    /// the entry to the exact split, budget and evidence.
    pub evidence_digest: u64,
    /// Flags read from the report itself.
    pub protected_or_final_access: bool,
    pub training_executed: bool,
    pub scientific_claim: bool,
    pub experimental_non_final: bool,
    pub admissible: AttributionClaim,
}

/// Immutable Stage-D attribution audit on one non-final split.
#[derive(Clone, Debug, PartialEq)]
pub struct StageDAttributionAuditReport {
    pub audit_contract: &'static str,
    pub split: DataSplit,
    pub budget: StageCPreflightBudget,
    pub entries: Vec<AttributionEntry>,
    /// Must remain false: no scientific attribution is admissible without a
    /// trained, preregistered Stage-D run, which this campaign never executes.
    pub scientific_attribution_admissible: bool,
    /// Must remain false.
    pub protected_or_final_access: bool,
    /// Must remain false.
    pub training_executed: bool,
    /// Must remain false.
    pub scientific_claim: bool,
    /// Must remain true.
    pub experimental_non_final: bool,
}

const fn attribution_invalid(reason: &'static str) -> EvalError {
    EvalError::StageDAttributionAuditInvalid { reason }
}

fn attribution_digest(rendering: &str) -> u64 {
    rendering.bytes().fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
        (hash ^ u64::from(byte)).wrapping_mul(0x0100_0000_01b3)
    })
}

const fn admissible_claim(entry: &AttributionEntry) -> AttributionClaim {
    if entry.validated
        && !entry.protected_or_final_access
        && !entry.training_executed
        && !entry.scientific_claim
        && entry.experimental_non_final
    {
        AttributionClaim::SoftwareSemanticsOnly
    } else {
        AttributionClaim::Withheld
    }
}

fn collect_attribution_entries(
    split: DataSplit,
    budget: StageCPreflightBudget,
) -> Result<Vec<AttributionEntry>, EvalError> {
    macro_rules! audit {
        ($slice:expr, $contract:expr, $report:expr) => {{
            let report = $report;
            let mut entry = AttributionEntry {
                slice: $slice,
                contract: $contract,
                validated: true,
                evidence_digest: attribution_digest(&format!("{report:?}")),
                protected_or_final_access: report.protected_or_final_access,
                training_executed: report.training_executed,
                scientific_claim: report.scientific_claim,
                experimental_non_final: report.experimental_non_final,
                admissible: AttributionClaim::Withheld,
            };
            entry.admissible = admissible_claim(&entry);
            entry
        }};
    }
    // Every runner validates its own report before returning it; any
    // rejection aborts the audit fail-closed.
    Ok(vec![
        audit!(
            30,
            STAGE_C_PREFLIGHT_CONTRACT,
            run_stage_c_preflight(split, budget)?
        ),
        audit!(
            31,
            GAMMA_ZERO_ABLATION_CONTRACT,
            run_gamma_zero_ablation(split, budget)?
        ),
        audit!(
            32,
            BETA_ZERO_ABLATION_CONTRACT,
            run_beta_zero_ablation(split, budget)?
        ),
        audit!(
            33,
            DIRECT_ONLY_COLLAPSE_CONTRACT,
            run_direct_only_collapse(split, budget)?
        ),
        audit!(
            34,
            PARITY_SHUFFLE_CONTROL_CONTRACT,
            run_parity_shuffle_control(split, budget)?
        ),
        audit!(
            35,
            FIXED_M_SENSITIVITY_CONTRACT,
            run_fixed_m_sensitivity(split, budget)?
        ),
        audit!(
            36,
            LEARNED_BASIS_PROTOTYPE_CONTRACT,
            run_learned_basis_prototype(split, budget)?
        ),
        audit!(
            37,
            HEAD_SHARING_ABLATION_CONTRACT,
            run_head_sharing_ablation(split, budget)?
        ),
        audit!(
            38,
            WIDTH_SCALING_CONTRACT,
            run_width_scaling(split, budget)?
        ),
        audit!(
            39,
            SEQUENCE_LENGTH_SCALING_CONTRACT,
            run_sequence_length_scaling(split, budget)?
        ),
    ])
}

/// Run the Stage-D attribution audit over every audited slice.
pub fn run_stage_d_attribution_audit(
    split: DataSplit,
    budget: StageCPreflightBudget,
) -> Result<StageDAttributionAuditReport, EvalError> {
    validate_non_final_split(split)?;
    budget.cases_per_arm()?;
    let report = StageDAttributionAuditReport {
        audit_contract: STAGE_D_ATTRIBUTION_AUDIT_CONTRACT,
        split,
        budget,
        entries: collect_attribution_entries(split, budget)?,
        scientific_attribution_admissible: false,
        protected_or_final_access: false,
        training_executed: false,
        scientific_claim: false,
        experimental_non_final: true,
    };
    validate_stage_d_attribution_audit_report(&report)?;
    Ok(report)
}

/// Parse a split label first; protected/final labels never generate a case.
pub fn run_stage_d_attribution_audit_for_label(
    split_label: &str,
    budget: StageCPreflightBudget,
) -> Result<StageDAttributionAuditReport, EvalError> {
    run_stage_d_attribution_audit(parse_non_final_split(split_label)?, budget)
}

/// Validate a Stage-D attribution audit: pin, the complete audited registry
/// in order, per-entry admissibility derived from flags, no withheld entry,
/// no scientific attribution, flags and regenerated evidence.
pub fn validate_stage_d_attribution_audit_report(
    report: &StageDAttributionAuditReport,
) -> Result<(), EvalError> {
    validate_non_final_split(report.split)?;
    if report.audit_contract != STAGE_D_ATTRIBUTION_AUDIT_CONTRACT {
        return Err(attribution_invalid("contract_drift"));
    }
    report.budget.cases_per_arm()?;
    if report.entries.len() != ATTRIBUTION_AUDITED_SLICES.len() {
        return Err(attribution_invalid("registry_drift"));
    }
    for (entry, (slice, contract)) in report.entries.iter().zip(ATTRIBUTION_AUDITED_SLICES) {
        if entry.slice != slice || entry.contract != contract {
            return Err(attribution_invalid("registry_drift"));
        }
        if entry.admissible != admissible_claim(entry) {
            return Err(attribution_invalid("admissibility_drift"));
        }
        if entry.admissible != AttributionClaim::SoftwareSemanticsOnly {
            return Err(attribution_invalid("withheld_entry"));
        }
    }
    if report.scientific_attribution_admissible {
        return Err(attribution_invalid("scientific_attribution_admissible"));
    }
    if report.protected_or_final_access {
        return Err(attribution_invalid("protected_or_final_access"));
    }
    if report.training_executed {
        return Err(attribution_invalid("training_executed"));
    }
    if report.scientific_claim {
        return Err(attribution_invalid("scientific_claim"));
    }
    if !report.experimental_non_final {
        return Err(attribution_invalid("experimental_non_final"));
    }
    if collect_attribution_entries(report.split, report.budget)? != report.entries {
        return Err(attribution_invalid("case_evidence_drift"));
    }
    Ok(())
}

/// Phase-E multi-seed replication contract pin (slice 41).
pub const MULTI_SEED_REPLICATION_CONTRACT: &str = "tdi24-multi-seed-replication-v1";

/// Frozen seed blocks of the replication. Block `b` starts at pair id
/// `b * MULTI_SEED_BLOCK_STRIDE`, so the blocks are disjoint by construction.
pub const MULTI_SEED_BLOCKS: [u64; 4] = [0, 1, 2, 3];

/// Pair-id stride between frozen blocks; equal to the per-family pair cap so
/// no admissible budget can make two blocks overlap.
pub const MULTI_SEED_BLOCK_STRIDE: u64 = MAX_PREFLIGHT_PAIRS_PER_FAMILY;

/// One frozen seed block: a full Stage-C preflight on a disjoint pair range.
#[derive(Clone, Debug, PartialEq)]
pub struct SeedBlockReplication {
    pub block: u64,
    pub first_pair_id: u64,
    pub cases_per_arm: u64,
    pub v6_matches: u64,
    pub c6_matches: u64,
    /// Paired V6/C6 summary of this block (slice-27 engine, unchanged).
    pub paired_summary: Option<PairedEffectSummary>,
}

/// Immutable multi-seed replication report on one non-final split.
#[derive(Clone, Debug, PartialEq)]
pub struct MultiSeedReplicationReport {
    pub replication_contract: &'static str,
    pub preflight_contract: &'static str,
    pub split: DataSplit,
    /// Base budget; `first_pair_id` must be 0 (offsets are frozen).
    pub budget: StageCPreflightBudget,
    pub blocks: Vec<SeedBlockReplication>,
    pub pooled_cases_per_arm: u64,
    pub pooled_v6_matches: u64,
    pub pooled_c6_matches: u64,
    /// Blocks with C6 > V6, C6 < V6 and C6 == V6 matches. Descriptive only.
    pub blocks_c6_ahead: u64,
    pub blocks_v6_ahead: u64,
    pub blocks_tied: u64,
    /// Must remain false.
    pub protected_or_final_access: bool,
    /// Must remain false.
    pub training_executed: bool,
    /// Must remain false: replication counts are not a claim.
    pub scientific_claim: bool,
    /// Must remain true.
    pub experimental_non_final: bool,
}

const fn replication_invalid(reason: &'static str) -> EvalError {
    EvalError::MultiSeedReplicationInvalid { reason }
}

fn count_matches(records: &[EvalRecord]) -> u64 {
    records
        .iter()
        .filter(|record| matches!(record.outcome, EvalOutcome::Scored { correct: true, .. }))
        .count() as u64
}

fn collect_seed_blocks(
    split: DataSplit,
    budget: StageCPreflightBudget,
) -> Result<Vec<SeedBlockReplication>, EvalError> {
    let mut blocks = Vec::with_capacity(MULTI_SEED_BLOCKS.len());
    let mut seen = Vec::new();
    for block in MULTI_SEED_BLOCKS {
        let first_pair_id = block * MULTI_SEED_BLOCK_STRIDE;
        let block_budget = StageCPreflightBudget::bounded(budget.pairs_per_family, first_pair_id);
        let report = run_stage_c_preflight(split, block_budget)?;
        // V6 and C6 score the same cases, so case digests are taken once.
        for record in &report.v6_records {
            seen.push(record.canonical_digest.clone());
        }
        blocks.push(SeedBlockReplication {
            block,
            first_pair_id,
            cases_per_arm: report.cases_per_arm,
            v6_matches: count_matches(&report.v6_records),
            c6_matches: count_matches(&report.c6_records),
            paired_summary: report.paired_summary,
        });
    }
    let total = seen.len();
    seen.sort_unstable();
    seen.dedup();
    if seen.len() != total {
        return Err(replication_invalid("block_overlap"));
    }
    Ok(blocks)
}

/// Run the multi-seed replication over every frozen seed block.
pub fn run_multi_seed_replication(
    split: DataSplit,
    budget: StageCPreflightBudget,
) -> Result<MultiSeedReplicationReport, EvalError> {
    validate_non_final_split(split)?;
    budget.cases_per_arm()?;
    if budget.first_pair_id != 0 {
        return Err(replication_invalid("frozen_offset_drift"));
    }
    let blocks = collect_seed_blocks(split, budget)?;
    let report = MultiSeedReplicationReport {
        replication_contract: MULTI_SEED_REPLICATION_CONTRACT,
        preflight_contract: STAGE_C_PREFLIGHT_CONTRACT,
        split,
        budget,
        pooled_cases_per_arm: blocks.iter().map(|b| b.cases_per_arm).sum(),
        pooled_v6_matches: blocks.iter().map(|b| b.v6_matches).sum(),
        pooled_c6_matches: blocks.iter().map(|b| b.c6_matches).sum(),
        blocks_c6_ahead: blocks
            .iter()
            .filter(|b| b.c6_matches > b.v6_matches)
            .count() as u64,
        blocks_v6_ahead: blocks
            .iter()
            .filter(|b| b.c6_matches < b.v6_matches)
            .count() as u64,
        blocks_tied: blocks
            .iter()
            .filter(|b| b.c6_matches == b.v6_matches)
            .count() as u64,
        blocks,
        protected_or_final_access: false,
        training_executed: false,
        scientific_claim: false,
        experimental_non_final: true,
    };
    validate_multi_seed_replication_report(&report)?;
    Ok(report)
}

/// Parse a split label first; protected/final labels never generate a case.
pub fn run_multi_seed_replication_for_label(
    split_label: &str,
    budget: StageCPreflightBudget,
) -> Result<MultiSeedReplicationReport, EvalError> {
    run_multi_seed_replication(parse_non_final_split(split_label)?, budget)
}

/// Validate a multi-seed replication report: pins, frozen blocks and
/// offsets, per-block consistency, pooled sums, sign tallies, flags and
/// regenerated evidence.
pub fn validate_multi_seed_replication_report(
    report: &MultiSeedReplicationReport,
) -> Result<(), EvalError> {
    validate_non_final_split(report.split)?;
    if report.replication_contract != MULTI_SEED_REPLICATION_CONTRACT {
        return Err(replication_invalid("contract_drift"));
    }
    if report.preflight_contract != STAGE_C_PREFLIGHT_CONTRACT {
        return Err(replication_invalid("preflight_contract_drift"));
    }
    let per_block = report.budget.cases_per_arm()?;
    if report.budget.first_pair_id != 0 {
        return Err(replication_invalid("frozen_offset_drift"));
    }
    if report.blocks.len() != MULTI_SEED_BLOCKS.len() {
        return Err(replication_invalid("block_set_drift"));
    }
    for (entry, block) in report.blocks.iter().zip(MULTI_SEED_BLOCKS) {
        if entry.block != block || entry.first_pair_id != block * MULTI_SEED_BLOCK_STRIDE {
            return Err(replication_invalid("block_set_drift"));
        }
        if entry.cases_per_arm != per_block
            || entry.v6_matches > per_block
            || entry.c6_matches > per_block
        {
            return Err(replication_invalid("block_count_drift"));
        }
        if entry
            .paired_summary
            .as_ref()
            .is_some_and(|summary| summary.split != report.split)
        {
            return Err(replication_invalid("block_count_drift"));
        }
    }
    let blocks = &report.blocks;
    if report.pooled_cases_per_arm != blocks.iter().map(|b| b.cases_per_arm).sum::<u64>()
        || report.pooled_v6_matches != blocks.iter().map(|b| b.v6_matches).sum::<u64>()
        || report.pooled_c6_matches != blocks.iter().map(|b| b.c6_matches).sum::<u64>()
    {
        return Err(replication_invalid("pooled_sum_drift"));
    }
    let ahead = blocks
        .iter()
        .filter(|b| b.c6_matches > b.v6_matches)
        .count() as u64;
    let behind = blocks
        .iter()
        .filter(|b| b.c6_matches < b.v6_matches)
        .count() as u64;
    let tied = blocks
        .iter()
        .filter(|b| b.c6_matches == b.v6_matches)
        .count() as u64;
    if (
        report.blocks_c6_ahead,
        report.blocks_v6_ahead,
        report.blocks_tied,
    ) != (ahead, behind, tied)
    {
        return Err(replication_invalid("sign_tally_drift"));
    }
    if report.protected_or_final_access {
        return Err(replication_invalid("protected_or_final_access"));
    }
    if report.training_executed {
        return Err(replication_invalid("training_executed"));
    }
    if report.scientific_claim {
        return Err(replication_invalid("scientific_claim"));
    }
    if !report.experimental_non_final {
        return Err(replication_invalid("experimental_non_final"));
    }
    if collect_seed_blocks(report.split, report.budget)? != report.blocks {
        return Err(replication_invalid("case_evidence_drift"));
    }
    Ok(())
}

/// Phase-E input-noise robustness contract pin (slice 42).
pub const INPUT_NOISE_ROBUSTNESS_CONTRACT: &str = "tdi24-input-noise-robustness-v1";

/// Declared perturbation families on the six chiral components.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NoiseFamily {
    /// All six components.
    Isotropic,
    /// Mirror-even components only.
    EvenOnly,
    /// Mirror-odd components only.
    OddOnly,
}

impl NoiseFamily {
    const fn mask(self) -> [bool; 6] {
        match self {
            Self::Isotropic => [true; 6],
            Self::EvenOnly => [true, true, true, false, false, false],
            Self::OddOnly => [false, false, false, true, true, true],
        }
    }
}

/// Declared perturbation families, all reported.
pub const NOISE_FAMILIES: [NoiseFamily; 3] = [
    NoiseFamily::Isotropic,
    NoiseFamily::EvenOnly,
    NoiseFamily::OddOnly,
];

/// Declared absolute amplitudes, all reported.
pub const NOISE_AMPLITUDES: [f64; 3] = [1e-3, 1e-2, 1e-1];

/// Paired arms: both receive the identical perturbed query and key.
pub const NOISE_ARMS: [SequenceArm; 2] = [SequenceArm::C6, SequenceArm::DirectOnly];

/// Per (noise family, amplitude, arm, task family) decision stability.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NoiseRobustnessCell {
    pub noise: NoiseFamily,
    pub amplitude: f64,
    pub arm: SequenceArm,
    pub family: TaskFamily,
    pub n_cases: u64,
    /// Cases whose perturbed score keeps the clean score's sign (zero counts
    /// as its own sign).
    pub sign_stable: u64,
    pub max_abs_score_change: f64,
}

/// Immutable input-noise robustness report on one non-final split.
#[derive(Clone, Debug, PartialEq)]
pub struct InputNoiseRobustnessReport {
    pub robustness_contract: &'static str,
    pub split: DataSplit,
    pub budget: StageCPreflightBudget,
    /// FNV-1a 64 of the contract string; no free seed constant.
    pub noise_seed: u64,
    pub capacity: TrainableCapacity,
    /// Noise-major, then amplitude, arm, task family.
    pub cells: Vec<NoiseRobustnessCell>,
    /// Must remain false.
    pub protected_or_final_access: bool,
    /// Must remain false.
    pub training_executed: bool,
    /// Must remain false.
    pub scientific_claim: bool,
    /// Must remain true.
    pub experimental_non_final: bool,
}

const fn noise_invalid(reason: &'static str) -> EvalError {
    EvalError::InputNoiseRobustnessInvalid { reason }
}

/// Seed of the declared perturbation stream, derived from the contract pin.
#[must_use]
pub fn input_noise_seed() -> u64 {
    INPUT_NOISE_ROBUSTNESS_CONTRACT
        .bytes()
        .fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
            (hash ^ u64::from(byte)).wrapping_mul(0x0100_0000_01b3)
        })
}

fn splitmix64(mut state: u64) -> u64 {
    state = state.wrapping_add(0x9e37_79b9_7f4a_7c15);
    state = (state ^ (state >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    state = (state ^ (state >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    state ^ (state >> 31)
}

/// Deterministic perturbation in `[-amplitude, amplitude]` for one component.
/// Depends on the case index, the operand (0 query, 1 key) and the component
/// only; never on the arm, so both arms see identical inputs.
fn noise_component(case_index: u64, operand: u64, component: u64, amplitude: f64) -> f64 {
    let mixed = splitmix64(
        input_noise_seed() ^ splitmix64(case_index ^ splitmix64(operand * 8 + component)),
    );
    let unit = (mixed >> 11) as f64 / (1u64 << 53) as f64;
    amplitude * (2.0 * unit - 1.0)
}

/// Perturb one operand under a declared family and amplitude.
pub fn perturb_operand(
    value: Chiral6,
    noise: NoiseFamily,
    amplitude: f64,
    case_index: u64,
    operand: u64,
) -> Result<Chiral6, EvalError> {
    if !NOISE_AMPLITUDES.contains(&amplitude) {
        return Err(noise_invalid("amplitude_not_registered"));
    }
    let mut data = value.as_array();
    for (component, (entry, active)) in data.iter_mut().zip(noise.mask()).enumerate() {
        if active {
            *entry += noise_component(case_index, operand, component as u64, amplitude);
        }
    }
    Chiral6::from_array(data).map_err(EvalError::ChiralNumerical)
}

fn collect_noise_cells(
    split: DataSplit,
    budget: StageCPreflightBudget,
) -> Result<Vec<NoiseRobustnessCell>, EvalError> {
    let items = collect_sequence_items(split, budget)?;
    let mut cells = Vec::new();
    for noise in NOISE_FAMILIES {
        for amplitude in NOISE_AMPLITUDES {
            for arm in NOISE_ARMS {
                for family in STAGE_C_PREFLIGHT_FAMILIES {
                    let mut cell = NoiseRobustnessCell {
                        noise,
                        amplitude,
                        arm,
                        family: *family,
                        n_cases: 0,
                        sign_stable: 0,
                        max_abs_score_change: 0.0,
                    };
                    for (index, item) in items.iter().enumerate() {
                        if item.0 != *family {
                            continue;
                        }
                        let clean = chiral_score(item.1, item.2, arm.weights())
                            .map_err(EvalError::ChiralNumerical)?;
                        if arm == SequenceArm::C6 && clean.to_bits() != item.3.to_bits() {
                            return Err(noise_invalid("clean_reference_drift"));
                        }
                        let query = perturb_operand(item.1, noise, amplitude, index as u64, 0)?;
                        let key = perturb_operand(item.2, noise, amplitude, index as u64, 1)?;
                        let noisy = chiral_score(query, key, arm.weights())
                            .map_err(EvalError::ChiralNumerical)?;
                        cell.n_cases += 1;
                        if noisy.signum() == clean.signum() && (noisy == 0.0) == (clean == 0.0) {
                            cell.sign_stable += 1;
                        }
                        cell.max_abs_score_change =
                            cell.max_abs_score_change.max((noisy - clean).abs());
                    }
                    cells.push(cell);
                }
            }
        }
    }
    Ok(cells)
}

/// Run the input-noise robustness study on the bounded Stage-C stream.
pub fn run_input_noise_robustness(
    split: DataSplit,
    budget: StageCPreflightBudget,
) -> Result<InputNoiseRobustnessReport, EvalError> {
    validate_non_final_split(split)?;
    budget.cases_per_arm()?;
    let report = InputNoiseRobustnessReport {
        robustness_contract: INPUT_NOISE_ROBUSTNESS_CONTRACT,
        split,
        budget,
        noise_seed: input_noise_seed(),
        capacity: TrainableCapacity::reference_c6(),
        cells: collect_noise_cells(split, budget)?,
        protected_or_final_access: false,
        training_executed: false,
        scientific_claim: false,
        experimental_non_final: true,
    };
    validate_input_noise_robustness_report(&report)?;
    Ok(report)
}

/// Parse a split label first; protected/final labels never generate a case.
pub fn run_input_noise_robustness_for_label(
    split_label: &str,
    budget: StageCPreflightBudget,
) -> Result<InputNoiseRobustnessReport, EvalError> {
    run_input_noise_robustness(parse_non_final_split(split_label)?, budget)
}

/// Validate an input-noise robustness report: pins, seed, capacity, the
/// complete declared grid in order, paired case counts, finite changes,
/// flags and regenerated evidence.
pub fn validate_input_noise_robustness_report(
    report: &InputNoiseRobustnessReport,
) -> Result<(), EvalError> {
    validate_non_final_split(report.split)?;
    if report.robustness_contract != INPUT_NOISE_ROBUSTNESS_CONTRACT {
        return Err(noise_invalid("contract_drift"));
    }
    if report.noise_seed != input_noise_seed() {
        return Err(noise_invalid("seed_drift"));
    }
    if report.capacity != TrainableCapacity::reference_c6() {
        return Err(noise_invalid("capacity_mismatch"));
    }
    let per_family = report.budget.cases_per_arm()? / STAGE_C_PREFLIGHT_FAMILIES.len() as u64;
    let expected = NOISE_FAMILIES.len()
        * NOISE_AMPLITUDES.len()
        * NOISE_ARMS.len()
        * STAGE_C_PREFLIGHT_FAMILIES.len();
    if report.cells.len() != expected {
        return Err(noise_invalid("cell_count"));
    }
    let mut position = 0usize;
    for noise in NOISE_FAMILIES {
        for amplitude in NOISE_AMPLITUDES {
            for arm in NOISE_ARMS {
                for family in STAGE_C_PREFLIGHT_FAMILIES {
                    let cell = &report.cells[position];
                    position += 1;
                    if cell.noise != noise
                        || cell.amplitude.to_bits() != amplitude.to_bits()
                        || cell.arm != arm
                        || cell.family != *family
                    {
                        return Err(noise_invalid("grid_drift"));
                    }
                    if cell.n_cases != per_family || cell.sign_stable > cell.n_cases {
                        return Err(noise_invalid("paired_count_drift"));
                    }
                    if !cell.max_abs_score_change.is_finite() || cell.max_abs_score_change < 0.0 {
                        return Err(noise_invalid("score_change_drift"));
                    }
                }
            }
        }
    }
    if report.protected_or_final_access {
        return Err(noise_invalid("protected_or_final_access"));
    }
    if report.training_executed {
        return Err(noise_invalid("training_executed"));
    }
    if report.scientific_claim {
        return Err(noise_invalid("scientific_claim"));
    }
    if !report.experimental_non_final {
        return Err(noise_invalid("experimental_non_final"));
    }
    if collect_noise_cells(report.split, report.budget)? != report.cells {
        return Err(noise_invalid("case_evidence_drift"));
    }
    Ok(())
}

/// Phase-E reflection adversarial set contract pin (slice 43).
pub const REFLECTION_ADVERSARIAL_CONTRACT: &str = "tdi24-reflection-adversarial-set-v1";

/// Declared direct-to-chiral dominance ratios `|s| / chi` of the hard
/// mirrored pairs, straddling the C6 decision boundary `1`; all reported.
pub const ADVERSARIAL_DOMINANCE_RATIOS: [f64; 5] = [0.5, 0.9, 0.99, 1.01, 2.0];

/// Declared chirality scales `chi / |q|^2` (ordinary and near-achiral); all
/// reported.
pub const ADVERSARIAL_CHIRALITY_SCALES: [f64; 2] = [1.0, 1e-3];

/// Declared norm of the orthogonal nuisance component relative to `|q|`.
pub const ADVERSARIAL_NUISANCE_RATIO: f64 = 2.0;

/// Paired arms scored on the identical adversarial pairs.
pub const ADVERSARIAL_ARMS: [SequenceArm; 2] = [SequenceArm::C6, SequenceArm::DirectOnly];

/// Relative tolerance of the construction check on `s` and `chi`.
const ADVERSARIAL_CONSTRUCTION_TOLERANCE: f64 = 1e-9;

/// One hard mirrored pair: the left member is the exact mirror `(Mq, Mk)`
/// of the right member. Generated from the declared geometry and the
/// registered seed only; no arm score is consulted.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AdversarialMirrorPair {
    pub pair_id: u64,
    pub scale: f64,
    pub ratio: f64,
    /// Declared sign of the direct channel `s` (alternates by pair).
    pub direct_sign: i8,
    pub right_query: Chiral6,
    pub right_key: Chiral6,
    pub left_query: Chiral6,
    pub left_key: Chiral6,
}

/// Per (scale, ratio, arm) handedness accounting on the adversarial pairs.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ReflectionAdversarialCell {
    pub scale: f64,
    pub ratio: f64,
    pub arm: SequenceArm,
    pub n_pairs: u64,
    /// Members (two per pair) whose score sign matches the declared
    /// handedness (right `+1`, left `-1`).
    pub members_correct: u64,
    /// Pairs whose two members are both correct.
    pub pairs_discriminated: u64,
}

/// Immutable reflection adversarial set report on one non-final split.
#[derive(Clone, Debug, PartialEq)]
pub struct ReflectionAdversarialReport {
    pub adversarial_contract: &'static str,
    pub split: DataSplit,
    pub budget: StageCPreflightBudget,
    /// FNV-1a 64 of the contract string; no free seed constant.
    pub generator_seed: u64,
    pub capacity: TrainableCapacity,
    /// FNV-1a 64 over the bit patterns of every generated pair.
    pub pair_digest: u64,
    /// Scale-major, then ratio, then arm.
    pub cells: Vec<ReflectionAdversarialCell>,
    /// Must remain false.
    pub protected_or_final_access: bool,
    /// Must remain false.
    pub training_executed: bool,
    /// Must remain false.
    pub scientific_claim: bool,
    /// Must remain true.
    pub experimental_non_final: bool,
}

const fn adversarial_invalid(reason: &'static str) -> EvalError {
    EvalError::ReflectionAdversarialInvalid { reason }
}

/// Seed of the adversarial generator, derived from the contract pin.
#[must_use]
pub fn reflection_adversarial_seed() -> u64 {
    REFLECTION_ADVERSARIAL_CONTRACT
        .bytes()
        .fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
            (hash ^ u64::from(byte)).wrapping_mul(0x0100_0000_01b3)
        })
}

fn adversarial_unit(state: u64, index: u64) -> f64 {
    let mixed = splitmix64(state ^ splitmix64(index));
    let unit = (mixed >> 11) as f64 / (1u64 << 53) as f64;
    2.0 * unit - 1.0
}

fn dot6(lhs: &[f64; 6], rhs: &[f64; 6]) -> f64 {
    lhs.iter().zip(rhs).map(|(a, b)| a * b).sum()
}

/// Base geometry of one pair: query `q` and nuisance `r` orthogonal to
/// `q` and `Jq` with `|r| = ADVERSARIAL_NUISANCE_RATIO * |q|`.
fn adversarial_base(
    split: DataSplit,
    pair_id: u64,
) -> Result<([f64; 6], [f64; 6], f64), EvalError> {
    let registered = register_seed(
        SeedDomain::from_split(split),
        TaskFamily::ReflectionDiscriminative,
        pair_id,
    );
    let state = reflection_adversarial_seed() ^ splitmix64(registered.mixed_seed);
    let mut query = [0.0; 6];
    for (index, entry) in query.iter_mut().enumerate() {
        let unit = adversarial_unit(state, index as u64);
        // Magnitudes in [0.5, 1] keep |q|^2 >= 1.5: no degenerate query.
        *entry = (0.5 + 0.5 * unit.abs()).copysign(unit);
    }
    let q = Chiral6::from_array(query).map_err(EvalError::ChiralNumerical)?;
    let jq = q.complex_structure().as_array();
    let norm_sq = dot6(&query, &query);
    let mut nuisance = [0.0; 6];
    for (index, entry) in nuisance.iter_mut().enumerate() {
        *entry = adversarial_unit(state, 6 + index as u64);
    }
    // Gram-Schmidt against the orthogonal pair (q, Jq), applied twice.
    for _ in 0..2 {
        let along_q = dot6(&nuisance, &query) / norm_sq;
        let along_jq = dot6(&nuisance, &jq) / norm_sq;
        for index in 0..6 {
            nuisance[index] -= along_q * query[index] + along_jq * jq[index];
        }
    }
    let residual = dot6(&nuisance, &nuisance).sqrt();
    if !residual.is_finite() || residual < 1e-6 {
        return Err(adversarial_invalid("degenerate_nuisance"));
    }
    let target = ADVERSARIAL_NUISANCE_RATIO * norm_sq.sqrt();
    for entry in &mut nuisance {
        *entry *= target / residual;
    }
    Ok((query, nuisance, norm_sq))
}

/// Generate one hard mirrored pair: `k = sigma*rho*scale*q - scale*Jq + r`
/// gives `chi(q, k) = scale*|q|^2 > 0` and `s(q, k) = sigma*rho*chi`.
fn adversarial_pair(
    split: DataSplit,
    pair_id: u64,
    pair_index: u64,
    scale: f64,
    ratio: f64,
) -> Result<AdversarialMirrorPair, EvalError> {
    let (query, nuisance, norm_sq) = adversarial_base(split, pair_id)?;
    let direct_sign: i8 = if pair_index % 2 == 0 { 1 } else { -1 };
    let q = Chiral6::from_array(query).map_err(EvalError::ChiralNumerical)?;
    let jq = q.complex_structure().as_array();
    let along = f64::from(direct_sign) * ratio * scale;
    let mut key = [0.0; 6];
    for index in 0..6 {
        key[index] = along * query[index] - scale * jq[index] + nuisance[index];
    }
    let k = Chiral6::from_array(key).map_err(EvalError::ChiralNumerical)?;
    let channels = observables(q, k).map_err(EvalError::ChiralNumerical)?;
    let chi = scale * norm_sq;
    let tolerance = ADVERSARIAL_CONSTRUCTION_TOLERANCE * (1.0 + norm_sq);
    if (channels.chiral - chi).abs() > tolerance * scale.max(1e-3)
        || (channels.direct - along * norm_sq).abs() > tolerance * scale.max(1e-3)
        || channels.chiral <= 0.0
    {
        return Err(adversarial_invalid("construction_drift"));
    }
    Ok(AdversarialMirrorPair {
        pair_id,
        scale,
        ratio,
        direct_sign,
        right_query: q,
        right_key: k,
        left_query: q.mirror(),
        left_key: k.mirror(),
    })
}

/// Generate the full adversarial set (scale-major, then ratio, then pair).
///
/// Depends only on the split, the budget, the declared grid and the
/// registered seeds; it takes no arm and never evaluates a score.
pub fn reflection_adversarial_pairs(
    split: DataSplit,
    budget: StageCPreflightBudget,
) -> Result<Vec<AdversarialMirrorPair>, EvalError> {
    validate_non_final_split(split)?;
    budget.cases_per_arm()?;
    let mut pairs = Vec::new();
    for scale in ADVERSARIAL_CHIRALITY_SCALES {
        for ratio in ADVERSARIAL_DOMINANCE_RATIOS {
            for pair_index in 0..budget.pairs_per_family {
                let pair_id = budget.first_pair_id + pair_index;
                pairs.push(adversarial_pair(split, pair_id, pair_index, scale, ratio)?);
            }
        }
    }
    Ok(pairs)
}

fn adversarial_pair_digest(pairs: &[AdversarialMirrorPair]) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for pair in pairs {
        for operand in [
            pair.right_query,
            pair.right_key,
            pair.left_query,
            pair.left_key,
        ] {
            for value in operand.as_array() {
                for byte in value.to_bits().to_le_bytes() {
                    hash = (hash ^ u64::from(byte)).wrapping_mul(0x0100_0000_01b3);
                }
            }
        }
    }
    hash
}

fn collect_adversarial_cells(
    pairs: &[AdversarialMirrorPair],
) -> Result<Vec<ReflectionAdversarialCell>, EvalError> {
    let mut cells = Vec::new();
    for scale in ADVERSARIAL_CHIRALITY_SCALES {
        for ratio in ADVERSARIAL_DOMINANCE_RATIOS {
            for arm in ADVERSARIAL_ARMS {
                let mut cell = ReflectionAdversarialCell {
                    scale,
                    ratio,
                    arm,
                    n_pairs: 0,
                    members_correct: 0,
                    pairs_discriminated: 0,
                };
                for pair in pairs.iter().filter(|pair| {
                    pair.scale.to_bits() == scale.to_bits()
                        && pair.ratio.to_bits() == ratio.to_bits()
                }) {
                    if pair.left_query != pair.right_query.mirror()
                        || pair.left_key != pair.right_key.mirror()
                    {
                        return Err(adversarial_invalid("mirror_drift"));
                    }
                    let right = chiral_score(pair.right_query, pair.right_key, arm.weights())
                        .map_err(EvalError::ChiralNumerical)?;
                    let left = chiral_score(pair.left_query, pair.left_key, arm.weights())
                        .map_err(EvalError::ChiralNumerical)?;
                    let right_ok = right > 0.0;
                    let left_ok = left < 0.0;
                    cell.n_pairs += 1;
                    cell.members_correct += u64::from(right_ok) + u64::from(left_ok);
                    cell.pairs_discriminated += u64::from(right_ok && left_ok);
                }
                cells.push(cell);
            }
        }
    }
    Ok(cells)
}

/// Run the reflection adversarial set on one non-final split.
pub fn run_reflection_adversarial_set(
    split: DataSplit,
    budget: StageCPreflightBudget,
) -> Result<ReflectionAdversarialReport, EvalError> {
    let pairs = reflection_adversarial_pairs(split, budget)?;
    let report = ReflectionAdversarialReport {
        adversarial_contract: REFLECTION_ADVERSARIAL_CONTRACT,
        split,
        budget,
        generator_seed: reflection_adversarial_seed(),
        capacity: TrainableCapacity::reference_c6(),
        pair_digest: adversarial_pair_digest(&pairs),
        cells: collect_adversarial_cells(&pairs)?,
        protected_or_final_access: false,
        training_executed: false,
        scientific_claim: false,
        experimental_non_final: true,
    };
    validate_reflection_adversarial_report(&report)?;
    Ok(report)
}

/// Parse a split label first; protected/final labels never generate a pair.
pub fn run_reflection_adversarial_set_for_label(
    split_label: &str,
    budget: StageCPreflightBudget,
) -> Result<ReflectionAdversarialReport, EvalError> {
    run_reflection_adversarial_set(parse_non_final_split(split_label)?, budget)
}

/// Validate a reflection adversarial report: pins, seed, capacity, the
/// complete declared grid in order, bounded counts, the structural
/// direct-only invariant, flags and regenerated evidence.
pub fn validate_reflection_adversarial_report(
    report: &ReflectionAdversarialReport,
) -> Result<(), EvalError> {
    validate_non_final_split(report.split)?;
    if report.adversarial_contract != REFLECTION_ADVERSARIAL_CONTRACT {
        return Err(adversarial_invalid("contract_drift"));
    }
    if report.generator_seed != reflection_adversarial_seed() {
        return Err(adversarial_invalid("seed_drift"));
    }
    if report.capacity != TrainableCapacity::reference_c6() {
        return Err(adversarial_invalid("capacity_mismatch"));
    }
    report.budget.cases_per_arm()?;
    let expected = ADVERSARIAL_CHIRALITY_SCALES.len()
        * ADVERSARIAL_DOMINANCE_RATIOS.len()
        * ADVERSARIAL_ARMS.len();
    if report.cells.len() != expected {
        return Err(adversarial_invalid("cell_count"));
    }
    let mut position = 0usize;
    for scale in ADVERSARIAL_CHIRALITY_SCALES {
        for ratio in ADVERSARIAL_DOMINANCE_RATIOS {
            for arm in ADVERSARIAL_ARMS {
                let cell = &report.cells[position];
                position += 1;
                if cell.scale.to_bits() != scale.to_bits()
                    || cell.ratio.to_bits() != ratio.to_bits()
                    || cell.arm != arm
                {
                    return Err(adversarial_invalid("grid_drift"));
                }
                if cell.n_pairs != report.budget.pairs_per_family
                    || cell.members_correct > 2 * cell.n_pairs
                    || cell.pairs_discriminated > cell.n_pairs
                    || 2 * cell.pairs_discriminated > cell.members_correct
                {
                    return Err(adversarial_invalid("paired_count_drift"));
                }
                // Mirrored members share every even channel, so an arm
                // without the parity-odd channel scores them identically.
                if arm == SequenceArm::DirectOnly && cell.pairs_discriminated != 0 {
                    return Err(adversarial_invalid("direct_only_discrimination"));
                }
            }
        }
    }
    if report.protected_or_final_access {
        return Err(adversarial_invalid("protected_or_final_access"));
    }
    if report.training_executed {
        return Err(adversarial_invalid("training_executed"));
    }
    if report.scientific_claim {
        return Err(adversarial_invalid("scientific_claim"));
    }
    if !report.experimental_non_final {
        return Err(adversarial_invalid("experimental_non_final"));
    }
    let pairs = reflection_adversarial_pairs(report.split, report.budget)?;
    if adversarial_pair_digest(&pairs) != report.pair_digest {
        return Err(adversarial_invalid("pair_digest_drift"));
    }
    if collect_adversarial_cells(&pairs)? != report.cells {
        return Err(adversarial_invalid("case_evidence_drift"));
    }
    Ok(())
}

/// Phase-E numerical precision study contract pin (slice 44).
pub const NUMERICAL_PRECISION_CONTRACT: &str = "tdi24-numerical-precision-v1";

/// Declared error-bound factor: an f32 score is within tolerance iff
/// `|score_f32 - score_f64| <= PRECISION_BOUND_FACTOR * f32::EPSILON * condition`,
/// where `condition` is the sum of the absolute weighted products entering the
/// score (a forward-error bound for 12-term f32 dot products with rounded
/// inputs). Declared before any run; not tuned.
pub const PRECISION_BOUND_FACTOR: f64 = 32.0;

/// Paired arms evaluated in f64 (reference) and f32.
pub const PRECISION_ARMS: [SequenceArm; 2] = [SequenceArm::C6, SequenceArm::DirectOnly];

/// Case groups of the precision study: the four Stage-C task families and
/// the slice-43 reflection adversarial set.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PrecisionGroup {
    StageC(TaskFamily),
    ReflectionAdversarial,
}

/// Per (group, arm) f64-vs-f32 accounting.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NumericalPrecisionCell {
    pub group: PrecisionGroup,
    pub arm: SequenceArm,
    pub n_cases: u64,
    /// f32 score finite and within the declared bound.
    pub within_tolerance: u64,
    /// f32 score finite but outside the declared bound.
    pub tolerance_failures: u64,
    /// f32 score non-finite (overflow); counted, never dropped.
    pub non_finite_f32: u64,
    /// Cases whose f32 decision sign differs from the f64 sign.
    pub sign_flips: u64,
    pub max_abs_error: f64,
    /// Largest `|error| / bound`; `<= 1` iff every finite case is within.
    pub max_error_to_bound: f64,
}

/// Immutable numerical precision report on one non-final split.
#[derive(Clone, Debug, PartialEq)]
pub struct NumericalPrecisionReport {
    pub precision_contract: &'static str,
    pub split: DataSplit,
    pub budget: StageCPreflightBudget,
    pub bound_factor: f64,
    pub capacity: TrainableCapacity,
    /// Stage-C families in canonical order, then the adversarial set; arm-minor.
    pub cells: Vec<NumericalPrecisionCell>,
    /// Must remain false.
    pub protected_or_final_access: bool,
    /// Must remain false.
    pub training_executed: bool,
    /// Must remain false.
    pub scientific_claim: bool,
    /// Must remain true.
    pub experimental_non_final: bool,
}

const fn precision_invalid(reason: &'static str) -> EvalError {
    EvalError::NumericalPrecisionInvalid { reason }
}

/// f32 evaluation of `alpha*s + beta*m + gamma*chi` with the same term
/// order as the f64 reference; inputs and weights are rounded to f32.
/// Returns the f32 score (widened) and the condition `sum |weighted terms|`.
#[must_use]
pub fn chiral_score_f32(query: Chiral6, key: Chiral6, weights: ChiralScoreWeights) -> (f64, f64) {
    let q = query.as_array().map(|value| value as f32);
    let k = key.as_array().map(|value| value as f32);
    let (alpha, beta, gamma) = (
        weights.alpha as f32,
        weights.beta as f32,
        weights.gamma as f32,
    );
    let mirrored_key = [k[0], k[1], k[2], -k[3], -k[4], -k[5]];
    let mut direct = 0.0f32;
    let mut mirrored = 0.0f32;
    let mut condition = 0.0f64;
    for index in 0..6 {
        direct += q[index] * k[index];
        mirrored += q[index] * mirrored_key[index];
        let product = f64::from(q[index]) * f64::from(k[index]);
        condition += (weights.alpha.abs() + weights.beta.abs()) * product.abs();
    }
    let mut chiral = 0.0f32;
    for index in 0..3 {
        chiral += q[index] * k[index + 3] - q[index + 3] * k[index];
        condition += weights.gamma.abs() * (f64::from(q[index]) * f64::from(k[index + 3])).abs()
            + weights.gamma.abs() * (f64::from(q[index + 3]) * f64::from(k[index])).abs();
    }
    let score = alpha * direct + beta * mirrored + gamma * chiral;
    (f64::from(score), condition)
}

fn precision_case(
    cell: &mut NumericalPrecisionCell,
    query: Chiral6,
    key: Chiral6,
) -> Result<(), EvalError> {
    let weights = cell.arm.weights();
    let reference = chiral_score(query, key, weights).map_err(EvalError::ChiralNumerical)?;
    let (single, condition) = chiral_score_f32(query, key, weights);
    cell.n_cases += 1;
    if !single.is_finite() {
        cell.non_finite_f32 += 1;
        return Ok(());
    }
    let error = (single - reference).abs();
    let bound = PRECISION_BOUND_FACTOR * f64::from(f32::EPSILON) * condition;
    let ratio = if bound > 0.0 {
        error / bound
    } else if error == 0.0 {
        0.0
    } else {
        f64::INFINITY
    };
    if ratio <= 1.0 {
        cell.within_tolerance += 1;
    } else {
        cell.tolerance_failures += 1;
    }
    if single.signum() != reference.signum() || (single == 0.0) != (reference == 0.0) {
        cell.sign_flips += 1;
    }
    cell.max_abs_error = cell.max_abs_error.max(error);
    cell.max_error_to_bound = cell.max_error_to_bound.max(ratio);
    Ok(())
}

fn precision_groups() -> Vec<PrecisionGroup> {
    let mut groups: Vec<PrecisionGroup> = STAGE_C_PREFLIGHT_FAMILIES
        .iter()
        .map(|family| PrecisionGroup::StageC(*family))
        .collect();
    groups.push(PrecisionGroup::ReflectionAdversarial);
    groups
}

fn collect_precision_cells(
    split: DataSplit,
    budget: StageCPreflightBudget,
) -> Result<Vec<NumericalPrecisionCell>, EvalError> {
    let items = collect_sequence_items(split, budget)?;
    let pairs = reflection_adversarial_pairs(split, budget)?;
    let mut cells = Vec::new();
    for group in precision_groups() {
        for arm in PRECISION_ARMS {
            let mut cell = NumericalPrecisionCell {
                group,
                arm,
                n_cases: 0,
                within_tolerance: 0,
                tolerance_failures: 0,
                non_finite_f32: 0,
                sign_flips: 0,
                max_abs_error: 0.0,
                max_error_to_bound: 0.0,
            };
            match group {
                PrecisionGroup::StageC(family) => {
                    for item in items.iter().filter(|item| item.0 == family) {
                        precision_case(&mut cell, item.1, item.2)?;
                    }
                }
                PrecisionGroup::ReflectionAdversarial => {
                    for pair in &pairs {
                        precision_case(&mut cell, pair.right_query, pair.right_key)?;
                        precision_case(&mut cell, pair.left_query, pair.left_key)?;
                    }
                }
            }
            cells.push(cell);
        }
    }
    Ok(cells)
}

/// Run the f64/f32 numerical precision study on one non-final split.
pub fn run_numerical_precision(
    split: DataSplit,
    budget: StageCPreflightBudget,
) -> Result<NumericalPrecisionReport, EvalError> {
    validate_non_final_split(split)?;
    budget.cases_per_arm()?;
    let report = NumericalPrecisionReport {
        precision_contract: NUMERICAL_PRECISION_CONTRACT,
        split,
        budget,
        bound_factor: PRECISION_BOUND_FACTOR,
        capacity: TrainableCapacity::reference_c6(),
        cells: collect_precision_cells(split, budget)?,
        protected_or_final_access: false,
        training_executed: false,
        scientific_claim: false,
        experimental_non_final: true,
    };
    validate_numerical_precision_report(&report)?;
    Ok(report)
}

/// Parse a split label first; protected/final labels never generate a case.
pub fn run_numerical_precision_for_label(
    split_label: &str,
    budget: StageCPreflightBudget,
) -> Result<NumericalPrecisionReport, EvalError> {
    run_numerical_precision(parse_non_final_split(split_label)?, budget)
}

/// Validate a numerical precision report: pins, declared bound, capacity,
/// the complete group/arm grid in order, complete failure accounting,
/// finite error statistics, flags and regenerated evidence.
pub fn validate_numerical_precision_report(
    report: &NumericalPrecisionReport,
) -> Result<(), EvalError> {
    validate_non_final_split(report.split)?;
    if report.precision_contract != NUMERICAL_PRECISION_CONTRACT {
        return Err(precision_invalid("contract_drift"));
    }
    if report.bound_factor.to_bits() != PRECISION_BOUND_FACTOR.to_bits() {
        return Err(precision_invalid("tolerance_drift"));
    }
    if report.capacity != TrainableCapacity::reference_c6() {
        return Err(precision_invalid("capacity_mismatch"));
    }
    let per_family = report.budget.cases_per_arm()? / STAGE_C_PREFLIGHT_FAMILIES.len() as u64;
    let adversarial = 2
        * report.budget.pairs_per_family
        * (ADVERSARIAL_CHIRALITY_SCALES.len() * ADVERSARIAL_DOMINANCE_RATIOS.len()) as u64;
    let groups = precision_groups();
    if report.cells.len() != groups.len() * PRECISION_ARMS.len() {
        return Err(precision_invalid("cell_count"));
    }
    let mut position = 0usize;
    for group in groups {
        for arm in PRECISION_ARMS {
            let cell = &report.cells[position];
            position += 1;
            if cell.group != group || cell.arm != arm {
                return Err(precision_invalid("grid_drift"));
            }
            let expected = match group {
                PrecisionGroup::StageC(_) => per_family,
                PrecisionGroup::ReflectionAdversarial => adversarial,
            };
            // Every case is accounted exactly once; none is dropped.
            if cell.n_cases != expected
                || cell.within_tolerance + cell.tolerance_failures + cell.non_finite_f32
                    != cell.n_cases
                || cell.sign_flips > cell.n_cases
            {
                return Err(precision_invalid("failure_accounting_drift"));
            }
            if !cell.max_abs_error.is_finite()
                || cell.max_abs_error < 0.0
                || cell.max_error_to_bound.is_nan()
                || cell.max_error_to_bound < 0.0
                || ((cell.tolerance_failures == 0) != (cell.max_error_to_bound <= 1.0))
            {
                return Err(precision_invalid("error_statistic_drift"));
            }
        }
    }
    if report.protected_or_final_access {
        return Err(precision_invalid("protected_or_final_access"));
    }
    if report.training_executed {
        return Err(precision_invalid("training_executed"));
    }
    if report.scientific_claim {
        return Err(precision_invalid("scientific_claim"));
    }
    if !report.experimental_non_final {
        return Err(precision_invalid("experimental_non_final"));
    }
    if collect_precision_cells(report.split, report.budget)? != report.cells {
        return Err(precision_invalid("case_evidence_drift"));
    }
    Ok(())
}

/// Reject any split identity outside Development/Validation.
pub fn validate_non_final_split(split: DataSplit) -> Result<(), EvalError> {
    match split {
        DataSplit::Development | DataSplit::Validation => Ok(()),
    }
}

/// Parse a split label, rejecting protected/final identities fail-closed.
pub fn parse_non_final_split(label: &str) -> Result<DataSplit, EvalError> {
    match label {
        "development" => Ok(DataSplit::Development),
        "validation" => Ok(DataSplit::Validation),
        "protected" | "final" => Err(EvalError::ProtectedOrFinalSplit),
        _ => Err(EvalError::ProtectedOrFinalSplit),
    }
}

/// Fail-closed evaluator errors.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EvalError {
    /// Protected/final or otherwise unknown split identity.
    ProtectedOrFinalSplit,
    /// Case split does not match the opened run.
    SplitMismatch {
        expected: DataSplit,
        actual: DataSplit,
    },
    /// Envelope or budget contract pin drifted.
    ContractMismatch { field: &'static str },
    /// Budget fields are zero or otherwise inadmissible.
    InvalidBudget,
    /// Arm is not admitted by this slice.
    UnsupportedArm,
    /// Matched case budget exhausted.
    CaseBudgetExceeded,
    /// V6 arithmetic rejected the carriers.
    Numerical(Vector6Error),
    /// C6 arithmetic rejected the carriers.
    ChiralNumerical(ChiralError),
    /// Trainable parameter counts or carrier widths are unmatched.
    ParameterCountMismatch {
        left_arm: EvalArm,
        right_arm: EvalArm,
        left_parameters: u64,
        right_parameters: u64,
    },
    /// Initialization seed/kind/stream are unmatched across paired arms.
    InitializationMismatch {
        left_arm: EvalArm,
        right_arm: EvalArm,
        left_seed: u64,
        right_seed: u64,
    },
    /// Optimizer/update budgets are unmatched across paired arms.
    OptimizerUpdateBudgetMismatch {
        left_arm: EvalArm,
        right_arm: EvalArm,
        left_examples: u64,
        right_examples: u64,
        left_updates: u64,
        right_updates: u64,
    },
    /// Metric registry failed closed-set validation.
    MetricRegistryInvalid { reason: &'static str },
    /// Paired uncertainty engine rejected inputs or produced non-finite stats.
    PairedUncertaintyInvalid { reason: &'static str },
    /// Failure taxonomy rejected an unknown class, empty code, or contract drift.
    FailureTaxonomyInvalid { reason: &'static str },
    /// Provenance envelope rejected empty/drifted identity fields or protected split.
    ProvenanceEnvelopeInvalid { reason: &'static str },
    /// Stage-C preflight rejected an unbounded budget, drifted report or forbidden flag.
    StageCPreflightInvalid { reason: &'static str },
    /// `gamma=0` ablation rejected drifted weights, identities, budget or flags.
    GammaZeroAblationInvalid { reason: &'static str },
    /// `beta=0` ablation rejected drifted weights, identities, budget or flags.
    BetaZeroAblationInvalid { reason: &'static str },
    /// Direct-only collapse rejected drifted weights, V6 mismatch, budget or flags.
    DirectOnlyCollapseInvalid { reason: &'static str },
    /// Parity-shuffle control rejected a structure-preserving or irreproducible
    /// shuffle, drifted weights/capacity, budget or flags.
    ParityShuffleControlInvalid { reason: &'static str },
    /// Fixed-M sensitivity rejected a non-algebraic or reordered basis family,
    /// drifted weights/capacity, evidence, budget or flags.
    FixedMSensitivityInvalid { reason: &'static str },
    /// Learned-basis prototype rejected a non-orthogonal or non-algebraic
    /// transform, a gauge or identity drift, or drifted evidence/flags.
    LearnedBasisPrototypeInvalid { reason: &'static str },
    /// Head-sharing ablation rejected drifted heads, bases, weights,
    /// capacity, evidence or flags.
    HeadSharingAblationInvalid { reason: &'static str },
    /// Width scaling rejected a drifted width set, weights, capacity,
    /// evidence or flags.
    WidthScalingInvalid { reason: &'static str },
    /// Sequence-length scaling rejected drifted lengths, masks, cost
    /// accounting, evidence or flags.
    SequenceLengthScalingInvalid { reason: &'static str },
    /// Stage-D attribution audit rejected a drifted registry, admissibility,
    /// evidence or claims.
    StageDAttributionAuditInvalid { reason: &'static str },
    /// Multi-seed replication rejected drifted blocks, sums, evidence or
    /// claims.
    MultiSeedReplicationInvalid { reason: &'static str },
    /// Input-noise robustness rejected a drifted grid, seed, counts, evidence
    /// or claims.
    InputNoiseRobustnessInvalid { reason: &'static str },
    /// Reflection adversarial set rejected a drifted grid, seed, pair
    /// construction, counts, evidence or claims.
    ReflectionAdversarialInvalid { reason: &'static str },
    /// Numerical precision study rejected a drifted bound, grid, failure
    /// accounting, evidence or claims.
    NumericalPrecisionInvalid { reason: &'static str },
}

impl fmt::Display for EvalError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ProtectedOrFinalSplit => {
                formatter.write_str("tdi-24 evaluator rejects protected/final splits")
            }
            Self::SplitMismatch { expected, actual } => write!(
                formatter,
                "evaluator split mismatch: expected {}, got {}",
                expected.as_str(),
                actual.as_str()
            ),
            Self::ContractMismatch { field } => {
                write!(formatter, "evaluator contract mismatch: {field}")
            }
            Self::InvalidBudget => formatter.write_str("evaluator readout budget is invalid"),
            Self::UnsupportedArm => formatter.write_str("evaluator arm is not supported"),
            Self::CaseBudgetExceeded => formatter.write_str("evaluator case budget exceeded"),
            Self::Numerical(error) => write!(formatter, "v6 numerical failure: {error}"),
            Self::ChiralNumerical(error) => write!(formatter, "c6 numerical failure: {error}"),
            Self::ParameterCountMismatch {
                left_arm,
                right_arm,
                left_parameters,
                right_parameters,
            } => write!(
                formatter,
                "parameter-count mismatch: {}={} vs {}={}",
                left_arm.as_str(),
                left_parameters,
                right_arm.as_str(),
                right_parameters
            ),
            Self::InitializationMismatch {
                left_arm,
                right_arm,
                left_seed,
                right_seed,
            } => write!(
                formatter,
                "initialization mismatch: {} seed={} vs {} seed={}",
                left_arm.as_str(),
                left_seed,
                right_arm.as_str(),
                right_seed
            ),
            Self::OptimizerUpdateBudgetMismatch {
                left_arm,
                right_arm,
                left_examples,
                right_examples,
                left_updates,
                right_updates,
            } => write!(
                formatter,
                "optimizer/update-budget mismatch: {} examples={} updates={} vs {} examples={} updates={}",
                left_arm.as_str(),
                left_examples,
                left_updates,
                right_arm.as_str(),
                right_examples,
                right_updates
            ),
            Self::MetricRegistryInvalid { reason } => {
                write!(formatter, "metric registry invalid: {reason}")
            }
            Self::PairedUncertaintyInvalid { reason } => {
                write!(formatter, "paired uncertainty invalid: {reason}")
            }
            Self::FailureTaxonomyInvalid { reason } => {
                write!(formatter, "failure taxonomy invalid: {reason}")
            }
            Self::ProvenanceEnvelopeInvalid { reason } => {
                write!(formatter, "provenance envelope invalid: {reason}")
            }
            Self::StageCPreflightInvalid { reason } => {
                write!(formatter, "stage-c preflight invalid: {reason}")
            }
            Self::GammaZeroAblationInvalid { reason } => {
                write!(formatter, "gamma=0 ablation invalid: {reason}")
            }
            Self::BetaZeroAblationInvalid { reason } => {
                write!(formatter, "beta=0 ablation invalid: {reason}")
            }
            Self::DirectOnlyCollapseInvalid { reason } => {
                write!(formatter, "direct-only collapse invalid: {reason}")
            }
            Self::ParityShuffleControlInvalid { reason } => {
                write!(formatter, "parity-shuffle control invalid: {reason}")
            }
            Self::FixedMSensitivityInvalid { reason } => {
                write!(formatter, "fixed-M sensitivity invalid: {reason}")
            }
            Self::LearnedBasisPrototypeInvalid { reason } => {
                write!(formatter, "learned-basis prototype invalid: {reason}")
            }
            Self::HeadSharingAblationInvalid { reason } => {
                write!(formatter, "head-sharing ablation invalid: {reason}")
            }
            Self::WidthScalingInvalid { reason } => {
                write!(formatter, "width scaling invalid: {reason}")
            }
            Self::SequenceLengthScalingInvalid { reason } => {
                write!(formatter, "sequence-length scaling invalid: {reason}")
            }
            Self::StageDAttributionAuditInvalid { reason } => {
                write!(formatter, "Stage-D attribution audit invalid: {reason}")
            }
            Self::MultiSeedReplicationInvalid { reason } => {
                write!(formatter, "multi-seed replication invalid: {reason}")
            }
            Self::InputNoiseRobustnessInvalid { reason } => {
                write!(formatter, "input-noise robustness invalid: {reason}")
            }
            Self::ReflectionAdversarialInvalid { reason } => {
                write!(formatter, "reflection adversarial set invalid: {reason}")
            }
            Self::NumericalPrecisionInvalid { reason } => {
                write!(formatter, "numerical precision study invalid: {reason}")
            }
        }
    }
}

impl std::error::Error for EvalError {}

/// Map handedness oracle to a signed binary label.
pub fn handedness_sign(target: &HandednessTarget) -> Result<i8, EvalError> {
    match target {
        HandednessTarget::Right => Ok(1),
        HandednessTarget::Left => Ok(-1),
    }
}

/// Map reflection-invariant oracle to a signed binary label.
pub fn reflection_invariant_sign(target: &ReflectionInvariantTarget) -> Result<i8, EvalError> {
    match target {
        ReflectionInvariantTarget::ClassA => Ok(1),
        ReflectionInvariantTarget::ClassB => Ok(-1),
    }
}

/// Map direction oracle to a signed binary label.
pub fn direction_sign(target: &DirectionTarget) -> Result<i8, EvalError> {
    match target {
        DirectionTarget::Forward => Ok(1),
        DirectionTarget::Reverse => Ok(-1),
    }
}

/// Map non-chiral control oracle to a signed binary label.
pub fn non_chiral_sign(target: &NonChiralTarget) -> Result<i8, EvalError> {
    match target {
        NonChiralTarget::ClassA => Ok(1),
        NonChiralTarget::ClassB => Ok(-1),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::experimental::tdi24_tasks::{
        PROTECTED_LABEL_CONTRACT, SeedDomain, TaskFamily, non_chiral_control_case,
        reflection_discriminative_pair, reflection_discriminative_pair_in_split,
        reflection_nuisance_pair, register_seed, seal_non_chiral_control,
        seal_reflection_discriminative, seal_reflection_nuisance,
    };

    /// Open a run whose provenance envelope admits exactly `views` (issue #690).
    fn admitted_run(config: EvaluatorConfig, views: &[&InferenceView]) -> EvaluatorRun {
        let admitted = views
            .iter()
            .map(|view| AdmittedCase::from_view(view))
            .collect();
        let envelope = ProvenanceEnvelope::for_pinned_stage_c_run(&config, admitted).unwrap();
        EvaluatorRun::open_with_provenance(config, envelope).unwrap()
    }

    #[test]
    fn c6_evaluator_contract_and_budget_match_v6() {
        assert_eq!(C6_EVALUATOR_CONTRACT, "tdi24-c6-evaluator-v1");
        let v6 = EvaluatorConfig::v6(DataSplit::Development).unwrap();
        let c6 = EvaluatorConfig::c6(DataSplit::Development).unwrap();
        assert_eq!(v6.budget, c6.budget);
        assert_eq!(EvalArm::C6.score_arm(), ScoreArm::C6);
    }

    #[test]
    fn c6_development_and_validation_paths_are_deterministic_and_sealed() {
        for split in [DataSplit::Development, DataSplit::Validation] {
            let case = reflection_discriminative_pair_in_split(7, split)
                .unwrap()
                .right;
            let labeled = seal_reflection_discriminative(&case);
            let mut first = admitted_run(
                EvaluatorConfig::c6(split).unwrap(),
                &[labeled.inference_view()],
            );
            let a = first
                .evaluate_c6_binary(&labeled, handedness_sign)
                .unwrap()
                .clone();
            let mut second = admitted_run(
                EvaluatorConfig::c6(split).unwrap(),
                &[labeled.inference_view()],
            );
            let b = second
                .evaluate_c6_binary(&labeled, handedness_sign)
                .unwrap()
                .clone();
            assert_eq!(a, b);
            assert_eq!(a.arm, EvalArm::C6);
            assert_eq!(a.arm_contract, C6_EVALUATOR_CONTRACT);
            assert_eq!(a.vector_contract, CHIRAL_CONTRACT);
            assert!(matches!(a.outcome, EvalOutcome::Scored { .. }));
        }
    }

    #[test]
    fn v6_evaluator_contracts_and_budget_are_pinned() {
        assert_eq!(EVALUATOR_ENVELOPE_CONTRACT, "tdi24-evaluator-envelope-v1");
        assert_eq!(V6_EVALUATOR_CONTRACT, "tdi24-v6-evaluator-v1");
        assert_eq!(READOUT_BUDGET_CONTRACT, "tdi24-readout-budget-v1");
        let budget = ReadoutBudget::matched_non_trained();
        assert_eq!(budget.max_cases, 64);
        assert_eq!(budget.max_readout_scalars_per_case, 2);
        assert_eq!(budget.updates, 0);
        assert_eq!(EvalArm::V6.score_arm(), ScoreArm::V6);
        assert!(parse_non_final_split("protected").is_err());
        assert!(parse_non_final_split("final").is_err());
        assert_eq!(
            parse_non_final_split("development").unwrap(),
            DataSplit::Development
        );
    }

    #[test]
    fn v6_development_path_is_deterministic_and_label_sealed() {
        let labeled =
            seal_reflection_discriminative(&reflection_discriminative_pair(3).unwrap().right);
        let mut run = admitted_run(
            EvaluatorConfig::v6(DataSplit::Development).unwrap(),
            &[labeled.inference_view()],
        );
        let rendered = run_inference_callback(&labeled, |view| format!("{view:?}"));
        assert!(!rendered.contains("Right"));
        assert!(!rendered.contains("Left"));
        assert!(!rendered.contains("target"));

        let first = run
            .evaluate_v6_binary(&labeled, handedness_sign)
            .unwrap()
            .clone();
        let second = {
            let mut again = admitted_run(
                EvaluatorConfig::v6(DataSplit::Development).unwrap(),
                &[labeled.inference_view()],
            );
            again
                .evaluate_v6_binary(&labeled, handedness_sign)
                .unwrap()
                .clone()
        };
        assert_eq!(first, second);
        assert_eq!(first.arm, EvalArm::V6);
        assert_eq!(first.split, DataSplit::Development);
        assert_eq!(first.arm_contract, V6_EVALUATOR_CONTRACT);
        assert_eq!(first.vector_contract, VECTOR6_CONTRACT);
        assert_eq!(first.label_contract, PROTECTED_LABEL_CONTRACT);
        assert!(matches!(first.outcome, EvalOutcome::Scored { .. }));
        assert_eq!(handedness_sign(&HandednessTarget::Right).unwrap(), 1);
    }

    #[test]
    fn v6_validation_path_accepts_split_manifest_cases() {
        let labeled = seal_reflection_discriminative(
            &reflection_discriminative_pair_in_split(5, DataSplit::Validation)
                .unwrap()
                .left,
        );
        let mut run = admitted_run(
            EvaluatorConfig::v6(DataSplit::Validation).unwrap(),
            &[labeled.inference_view()],
        );
        let record = run.evaluate_v6_binary(&labeled, handedness_sign).unwrap();
        assert_eq!(record.split, DataSplit::Validation);
        assert_eq!(record.family.as_str(), "reflection_discriminative");
    }

    #[test]
    fn v6_rejects_split_mismatch_and_budget_overflow() {
        let development =
            seal_reflection_discriminative(&reflection_discriminative_pair(1).unwrap().right);
        let mut run = admitted_run(
            EvaluatorConfig::v6(DataSplit::Development).unwrap(),
            &[development.inference_view()],
        );
        let validation = seal_reflection_discriminative(
            &reflection_discriminative_pair_in_split(1, DataSplit::Validation)
                .unwrap()
                .right,
        );
        assert!(matches!(
            run.evaluate_v6_binary(&validation, handedness_sign),
            Err(EvalError::SplitMismatch { .. })
        ));

        let mut tight = EvaluatorConfig::v6(DataSplit::Development).unwrap();
        tight.budget.max_cases = 1;
        let a = seal_non_chiral_control(&non_chiral_control_case(0).unwrap());
        let b = seal_non_chiral_control(&non_chiral_control_case(1).unwrap());
        let mut limited = admitted_run(tight, &[a.inference_view()]);
        assert!(limited.evaluate_v6_binary(&a, non_chiral_sign).is_ok());
        assert_eq!(
            limited.evaluate_v6_binary(&b, non_chiral_sign),
            Err(EvalError::CaseBudgetExceeded)
        );
    }

    #[test]
    fn v6_scores_nuisance_and_non_chiral_families_without_oracle_leakage() {
        let nuisance = seal_reflection_nuisance(&reflection_nuisance_pair(2).unwrap().canonical);
        let control = seal_non_chiral_control(&non_chiral_control_case(4).unwrap());
        let mut run = admitted_run(
            EvaluatorConfig::v6(DataSplit::Development).unwrap(),
            &[nuisance.inference_view(), control.inference_view()],
        );
        let n = run
            .evaluate_v6_binary(&nuisance, reflection_invariant_sign)
            .unwrap();
        assert!(matches!(n.outcome, EvalOutcome::Scored { .. }));
        let c = run.evaluate_v6_binary(&control, non_chiral_sign).unwrap();
        assert!(matches!(c.outcome, EvalOutcome::Scored { .. }));
        // Non-chiral even-sector targets are linearly readable by V6 on several seeds.
        let mut hits = 0u32;
        for case_id in 0..8u64 {
            let sealed = seal_non_chiral_control(&non_chiral_control_case(case_id).unwrap());
            let mut probe = admitted_run(
                EvaluatorConfig::v6(DataSplit::Development).unwrap(),
                &[sealed.inference_view()],
            );
            let record = probe.evaluate_v6_binary(&sealed, non_chiral_sign).unwrap();
            if matches!(record.outcome, EvalOutcome::Scored { correct: true, .. }) {
                hits += 1;
            }
        }
        assert!(
            hits >= 1,
            "V6 should recover at least one non-chiral even-sector label"
        );
        let leaked = run_inference_callback(&nuisance, |view| format!("{view:?}"));
        assert!(!leaked.contains("ClassA"));
        assert!(!leaked.contains("ClassB"));
    }

    #[test]
    fn parameter_count_matcher_accepts_matched_reference_capacities() {
        assert_eq!(
            PARAMETER_COUNT_MATCHER_CONTRACT,
            "tdi24-parameter-count-matcher-v1"
        );
        let v6 = TrainableCapacity::reference_v6();
        let c6 = TrainableCapacity::reference_c6();
        assert_eq!(v6.trainable_parameters, 0);
        assert_eq!(c6.trainable_parameters, 0);
        assert_eq!(v6.carrier_width, VECTOR6_WIDTH as u64);
        assert_eq!(c6.carrier_width, CHIRAL_WIDTH as u64);
        let matched = match_parameter_counts(v6, c6).unwrap();
        assert_eq!(matched.trainable_parameters, 0);
        assert_eq!(matched.carrier_width, VECTOR6_WIDTH as u64);
        assert_eq!(matched.left_arm, EvalArm::V6);
        assert_eq!(matched.right_arm, EvalArm::C6);
        assert_eq!(matched.matcher_contract, PARAMETER_COUNT_MATCHER_CONTRACT);
        // Matched capacity is admissible on Development and Validation only.
        for split in [DataSplit::Development, DataSplit::Validation] {
            assert!(validate_non_final_split(split).is_ok());
            let _ = EvaluatorConfig::v6(split).unwrap();
            let _ = EvaluatorConfig::c6(split).unwrap();
        }
        assert!(parse_non_final_split("protected").is_err());
        assert!(parse_non_final_split("final").is_err());
    }

    #[test]
    fn parameter_count_matcher_rejects_unmatched_trainable_capacity() {
        let v6 = TrainableCapacity::reference_v6();
        let mut inflated = TrainableCapacity::reference_c6();
        inflated.trainable_parameters = 16;
        assert_eq!(
            match_parameter_counts(v6, inflated),
            Err(EvalError::ParameterCountMismatch {
                left_arm: EvalArm::V6,
                right_arm: EvalArm::C6,
                left_parameters: 0,
                right_parameters: 16,
            })
        );

        let mut wide = TrainableCapacity::reference_c6();
        wide.carrier_width = 8;
        assert!(matches!(
            match_parameter_counts(v6, wide),
            Err(EvalError::ParameterCountMismatch { .. })
        ));

        let mut drifted = TrainableCapacity::reference_v6();
        drifted.matcher_contract = "not-a-matcher";
        assert_eq!(
            match_parameter_counts(drifted, TrainableCapacity::reference_c6()),
            Err(EvalError::ContractMismatch {
                field: "parameter_count_matcher_contract",
            })
        );
    }

    #[test]
    fn initialization_matcher_accepts_paired_reference_policies() {
        assert_eq!(
            INITIALIZATION_MATCHER_CONTRACT,
            "tdi24-initialization-matcher-v1"
        );
        let seed = 42_u64;
        let v6 = InitializationPolicy::reference_v6(seed);
        let c6 = InitializationPolicy::reference_c6(seed);
        assert_eq!(v6.kind, InitializationKind::ReferenceNone);
        assert_eq!(c6.kind, InitializationKind::ReferenceNone);
        assert_eq!(v6.seed, seed);
        assert_eq!(c6.seed, seed);
        assert_eq!(v6.stream_id, 0);
        assert_eq!(InitializationKind::ReferenceNone.as_str(), "reference_none");
        assert_eq!(
            InitializationKind::DeterministicPaired.as_str(),
            "deterministic_paired"
        );
        let matched = match_initialization(v6, c6).unwrap();
        assert_eq!(matched.kind, InitializationKind::ReferenceNone);
        assert_eq!(matched.seed, seed);
        assert_eq!(matched.stream_id, 0);
        assert_eq!(matched.left_arm, EvalArm::V6);
        assert_eq!(matched.right_arm, EvalArm::C6);
        assert_eq!(matched.matcher_contract, INITIALIZATION_MATCHER_CONTRACT);
        // Paired init remains admissible only on Development/Validation.
        for split in [DataSplit::Development, DataSplit::Validation] {
            assert!(validate_non_final_split(split).is_ok());
        }
        assert!(parse_non_final_split("protected").is_err());
        assert!(parse_non_final_split("final").is_err());
    }

    #[test]
    fn initialization_matcher_rejects_unpaired_seed_or_kind() {
        let v6 = InitializationPolicy::reference_v6(7);
        let other_seed = InitializationPolicy::reference_c6(8);
        assert_eq!(
            match_initialization(v6, other_seed),
            Err(EvalError::InitializationMismatch {
                left_arm: EvalArm::V6,
                right_arm: EvalArm::C6,
                left_seed: 7,
                right_seed: 8,
            })
        );

        let mut other_kind = InitializationPolicy::reference_c6(7);
        other_kind.kind = InitializationKind::DeterministicPaired;
        assert!(matches!(
            match_initialization(v6, other_kind),
            Err(EvalError::InitializationMismatch { .. })
        ));

        let mut other_stream = InitializationPolicy::reference_c6(7);
        other_stream.stream_id = 1;
        assert!(matches!(
            match_initialization(v6, other_stream),
            Err(EvalError::InitializationMismatch { .. })
        ));

        let mut drifted = InitializationPolicy::reference_v6(7);
        drifted.matcher_contract = "not-an-init-matcher";
        assert_eq!(
            match_initialization(drifted, InitializationPolicy::reference_c6(7)),
            Err(EvalError::ContractMismatch {
                field: "initialization_matcher_contract",
            })
        );
    }

    #[test]
    fn optimizer_update_budget_matcher_accepts_paired_reference_budgets() {
        assert_eq!(
            OPTIMIZER_UPDATE_BUDGET_CONTRACT,
            "tdi24-optimizer-update-budget-v1"
        );
        let examples = 16_u64;
        let ordering_seed = 99_u64;
        let v6 = OptimizerUpdateBudget::reference_v6(examples, ordering_seed);
        let c6 = OptimizerUpdateBudget::reference_c6(examples, ordering_seed);
        assert_eq!(v6.updates, NON_TRAINED_UPDATE_BUDGET);
        assert_eq!(c6.updates, NON_TRAINED_UPDATE_BUDGET);
        assert_eq!(v6.stopping, StoppingRule::ExhaustExamples);
        assert_eq!(StoppingRule::ExhaustExamples.as_str(), "exhaust_examples");
        assert_eq!(StoppingRule::FixedUpdates.as_str(), "fixed_updates");
        let matched = match_optimizer_update_budgets(v6, c6).unwrap();
        assert_eq!(matched.examples, examples);
        assert_eq!(matched.updates, 0);
        assert_eq!(matched.ordering_seed, ordering_seed);
        assert_eq!(matched.stopping, StoppingRule::ExhaustExamples);
        assert_eq!(matched.left_arm, EvalArm::V6);
        assert_eq!(matched.right_arm, EvalArm::C6);
        assert_eq!(matched.matcher_contract, OPTIMIZER_UPDATE_BUDGET_CONTRACT);
        for split in [DataSplit::Development, DataSplit::Validation] {
            assert!(validate_non_final_split(split).is_ok());
        }
        assert!(parse_non_final_split("protected").is_err());
        assert!(parse_non_final_split("final").is_err());
    }

    #[test]
    fn optimizer_update_budget_matcher_rejects_unpaired_budgets() {
        let v6 = OptimizerUpdateBudget::reference_v6(8, 1);
        let other_examples = OptimizerUpdateBudget::reference_c6(9, 1);
        assert_eq!(
            match_optimizer_update_budgets(v6, other_examples),
            Err(EvalError::OptimizerUpdateBudgetMismatch {
                left_arm: EvalArm::V6,
                right_arm: EvalArm::C6,
                left_examples: 8,
                right_examples: 9,
                left_updates: 0,
                right_updates: 0,
            })
        );

        let mut other_updates = OptimizerUpdateBudget::reference_c6(8, 1);
        other_updates.updates = 4;
        other_updates.stopping = StoppingRule::FixedUpdates;
        assert!(matches!(
            match_optimizer_update_budgets(v6, other_updates),
            Err(EvalError::OptimizerUpdateBudgetMismatch { .. })
        ));

        let other_order = OptimizerUpdateBudget::reference_c6(8, 2);
        assert!(matches!(
            match_optimizer_update_budgets(v6, other_order),
            Err(EvalError::OptimizerUpdateBudgetMismatch { .. })
        ));

        assert_eq!(
            match_optimizer_update_budgets(
                OptimizerUpdateBudget::reference_v6(0, 1),
                OptimizerUpdateBudget::reference_c6(0, 1)
            ),
            Err(EvalError::InvalidBudget)
        );

        let mut drifted = OptimizerUpdateBudget::reference_v6(8, 1);
        drifted.matcher_contract = "not-an-opt-budget";
        assert_eq!(
            match_optimizer_update_budgets(drifted, OptimizerUpdateBudget::reference_c6(8, 1)),
            Err(EvalError::ContractMismatch {
                field: "optimizer_update_budget_contract",
            })
        );
    }

    #[test]
    fn metric_registry_pins_primary_and_ordered_secondaries() {
        assert_eq!(METRIC_REGISTRY_CONTRACT, "tdi24-metric-registry-v1");
        assert_eq!(PRIMARY_METRIC_PAIRED_TASK_ACCURACY, "paired_task_accuracy");
        assert_eq!(
            PINNED_SECONDARY_DIAGNOSTICS.len(),
            MAX_SECONDARY_DIAGNOSTICS
        );
        assert_eq!(
            SecondaryDiagnosticId::admitted_set(),
            PINNED_SECONDARY_DIAGNOSTICS
        );

        let pinned = MetricRegistry::pinned();
        assert_eq!(pinned.primary, PrimaryMetricId::PairedTaskAccuracy);
        assert_eq!(pinned.primary.as_str(), PRIMARY_METRIC_PAIRED_TASK_ACCURACY);
        assert_eq!(pinned.secondaries, PINNED_SECONDARY_DIAGNOSTICS);
        assert_eq!(pinned.registry_contract, METRIC_REGISTRY_CONTRACT);
        assert!(pinned.experimental_non_final);
        validate_metric_registry(&pinned).unwrap();

        let frozen = freeze_metric_registry(
            Some(PrimaryMetricId::PairedTaskAccuracy),
            PINNED_SECONDARY_DIAGNOSTICS,
        )
        .unwrap();
        assert_eq!(frozen, pinned);

        assert_eq!(
            pinned.secondaries[0].as_str(),
            SECONDARY_PAIRED_OUTCOME_DIFFERENCE
        );
        assert_eq!(
            pinned.secondaries[1].as_str(),
            SECONDARY_MIRROR_SWAP_IDENTITY_ERROR
        );
        assert_eq!(
            pinned.secondaries[2].as_str(),
            SECONDARY_PARITY_EQUIVARIANCE_INVARIANCE_ERROR
        );
        assert_eq!(
            pinned.secondaries[3].as_str(),
            SECONDARY_CALIBRATION_CONFIDENCE_ERROR
        );
        assert_eq!(pinned.secondaries[4].as_str(), SECONDARY_GRADIENT_STABILITY);
        assert_eq!(
            pinned.secondaries[5].as_str(),
            SECONDARY_OP_COUNT_MEMORY_LATENCY_THROUGHPUT
        );

        for split in [DataSplit::Development, DataSplit::Validation] {
            assert!(pinned.admit_split(split).is_ok());
        }
        // Protected/final labels are rejected by the non-final split gate.
        assert!(parse_non_final_split("protected").is_err());
        assert!(parse_non_final_split("final").is_err());
        assert_eq!(
            parse_primary_metric_id(PRIMARY_METRIC_PAIRED_TASK_ACCURACY).unwrap(),
            PrimaryMetricId::PairedTaskAccuracy
        );
        assert_eq!(
            parse_secondary_diagnostic_id(SECONDARY_PAIRED_OUTCOME_DIFFERENCE).unwrap(),
            SecondaryDiagnosticId::PairedOutcomeDifference
        );
    }

    #[test]
    fn metric_registry_rejects_drift_duplicates_and_invention() {
        assert_eq!(
            freeze_metric_registry(None, PINNED_SECONDARY_DIAGNOSTICS),
            Err(EvalError::MetricRegistryInvalid {
                reason: "empty_primary",
            })
        );
        assert_eq!(
            parse_primary_metric_id(""),
            Err(EvalError::MetricRegistryInvalid {
                reason: "empty_primary",
            })
        );
        assert_eq!(
            parse_primary_metric_id("invented_primary"),
            Err(EvalError::MetricRegistryInvalid {
                reason: "unknown_primary",
            })
        );
        assert_eq!(
            parse_secondary_diagnostic_id("invented_secondary"),
            Err(EvalError::MetricRegistryInvalid {
                reason: "unknown_secondary",
            })
        );

        const DUPLICATE_SECONDARIES: &[SecondaryDiagnosticId] = &[
            SecondaryDiagnosticId::PairedOutcomeDifference,
            SecondaryDiagnosticId::MirrorSwapIdentityError,
            SecondaryDiagnosticId::PairedOutcomeDifference,
        ];
        assert_eq!(
            freeze_metric_registry(
                Some(PrimaryMetricId::PairedTaskAccuracy),
                DUPLICATE_SECONDARIES
            ),
            Err(EvalError::MetricRegistryInvalid {
                reason: "duplicate_secondary",
            })
        );

        const OVERSIZED_SECONDARIES: &[SecondaryDiagnosticId] = &[
            SecondaryDiagnosticId::PairedOutcomeDifference,
            SecondaryDiagnosticId::MirrorSwapIdentityError,
            SecondaryDiagnosticId::ParityEquivarianceInvarianceError,
            SecondaryDiagnosticId::CalibrationConfidenceError,
            SecondaryDiagnosticId::GradientStability,
            SecondaryDiagnosticId::OpCountMemoryLatencyThroughput,
            SecondaryDiagnosticId::PairedOutcomeDifference,
        ];
        assert_eq!(
            freeze_metric_registry(
                Some(PrimaryMetricId::PairedTaskAccuracy),
                OVERSIZED_SECONDARIES
            ),
            Err(EvalError::MetricRegistryInvalid {
                reason: "too_many_secondaries",
            })
        );

        const TRUNCATED_SECONDARIES: &[SecondaryDiagnosticId] = &[
            SecondaryDiagnosticId::PairedOutcomeDifference,
            SecondaryDiagnosticId::MirrorSwapIdentityError,
            SecondaryDiagnosticId::ParityEquivarianceInvarianceError,
            SecondaryDiagnosticId::CalibrationConfidenceError,
            SecondaryDiagnosticId::GradientStability,
        ];
        assert_eq!(
            freeze_metric_registry(
                Some(PrimaryMetricId::PairedTaskAccuracy),
                TRUNCATED_SECONDARIES
            ),
            Err(EvalError::MetricRegistryInvalid {
                reason: "secondary_sequence_mismatch",
            })
        );
        assert_eq!(
            freeze_metric_registry(Some(PrimaryMetricId::PairedTaskAccuracy), &[]),
            Err(EvalError::MetricRegistryInvalid {
                reason: "secondary_sequence_mismatch",
            })
        );

        const REORDERED_SECONDARIES: &[SecondaryDiagnosticId] = &[
            SecondaryDiagnosticId::MirrorSwapIdentityError,
            SecondaryDiagnosticId::PairedOutcomeDifference,
            SecondaryDiagnosticId::ParityEquivarianceInvarianceError,
            SecondaryDiagnosticId::CalibrationConfidenceError,
            SecondaryDiagnosticId::GradientStability,
            SecondaryDiagnosticId::OpCountMemoryLatencyThroughput,
        ];
        assert_eq!(
            freeze_metric_registry(
                Some(PrimaryMetricId::PairedTaskAccuracy),
                REORDERED_SECONDARIES
            ),
            Err(EvalError::MetricRegistryInvalid {
                reason: "secondary_sequence_mismatch",
            })
        );

        let malformed = MetricRegistry {
            secondaries: &[],
            ..MetricRegistry::pinned()
        };
        assert_eq!(
            malformed.admit_split(DataSplit::Development),
            Err(EvalError::MetricRegistryInvalid {
                reason: "secondary_sequence_mismatch",
            })
        );
        let mut config = EvaluatorConfig::v6(DataSplit::Development).unwrap();
        config.metric_registry = malformed;
        assert_eq!(
            EvaluatorRun::open(config),
            Err(EvalError::MetricRegistryInvalid {
                reason: "secondary_sequence_mismatch",
            })
        );

        let mut drifted = MetricRegistry::pinned();
        drifted.registry_contract = "not-a-metric-registry";
        assert_eq!(
            validate_metric_registry(&drifted),
            Err(EvalError::ContractMismatch {
                field: "metric_registry_contract",
            })
        );

        let mut finalized = MetricRegistry::pinned();
        finalized.experimental_non_final = false;
        assert_eq!(
            validate_metric_registry(&finalized),
            Err(EvalError::MetricRegistryInvalid {
                reason: "experimental_non_final_required",
            })
        );
        assert_eq!(
            finalized.admit_split(DataSplit::Development),
            Err(EvalError::MetricRegistryInvalid {
                reason: "experimental_non_final_required",
            })
        );
    }

    fn matches(bits: &[bool]) -> Vec<RevealedMatchOutcome> {
        bits.iter()
            .copied()
            .map(RevealedMatchOutcome::from_matches_oracle)
            .collect()
    }

    #[test]
    fn paired_uncertainty_happy_path_on_synthetic_match_vectors() {
        assert_eq!(PAIRED_UNCERTAINTY_CONTRACT, "tdi24-paired-uncertainty-v1");
        assert_eq!(UncertaintyMethod::WilsonScore.as_str(), "wilson_score");
        assert_eq!(
            UncertaintyMethod::BoundedHoeffdingPairedDifference.as_str(),
            "bounded_hoeffding_paired_difference"
        );

        // V6: T T F F  -> accuracy 0.5
        // C6: T T T F  -> accuracy 0.75
        // diffs: 0,0,1,0 -> mean 0.25
        let v6 = matches(&[true, true, false, false]);
        let c6 = matches(&[true, true, true, false]);
        let registry = MetricRegistry::pinned();
        let summary =
            summarize_paired_uncertainty(DataSplit::Development, &v6, &c6, &registry).unwrap();

        assert_eq!(summary.n_pairs, 4);
        assert!((summary.v6_accuracy - 0.5).abs() < 1e-12);
        assert!((summary.c6_accuracy - 0.75).abs() < 1e-12);
        assert!((summary.paired_difference_mean - 0.25).abs() < 1e-12);
        assert_eq!(summary.primary_metric, PrimaryMetricId::PairedTaskAccuracy);
        assert_eq!(
            summary.secondary_paired_outcome_difference,
            SecondaryDiagnosticId::PairedOutcomeDifference
        );
        assert_eq!(summary.uncertainty_contract, PAIRED_UNCERTAINTY_CONTRACT);
        assert_eq!(summary.metric_registry_contract, METRIC_REGISTRY_CONTRACT);
        assert!(summary.experimental_non_final);
        assert_eq!(
            summary.paired_difference_ci.method,
            UncertaintyMethod::BoundedHoeffdingPairedDifference
        );
        assert_eq!(
            summary.v6_accuracy_ci.method,
            UncertaintyMethod::WilsonScore
        );
        assert_eq!(
            summary.c6_accuracy_ci.method,
            UncertaintyMethod::WilsonScore
        );
        assert!(summary.paired_difference_ci.lower <= summary.paired_difference_mean);
        assert!(summary.paired_difference_ci.upper >= summary.paired_difference_mean);
        assert!(summary.paired_difference_ci.lower < summary.paired_difference_ci.upper);
        assert!(summary.v6_accuracy_ci.lower <= summary.v6_accuracy);
        assert!(summary.v6_accuracy_ci.upper >= summary.v6_accuracy);

        let validation =
            summarize_paired_uncertainty(DataSplit::Validation, &v6, &c6, &registry).unwrap();
        assert_eq!(validation.split, DataSplit::Validation);
    }

    #[test]
    fn paired_uncertainty_boundary_intervals_remain_non_degenerate() {
        let registry = MetricRegistry::pinned();
        let ties = matches(&[true; MAX_CASES_PER_RUN as usize]);
        let all_ties =
            summarize_paired_uncertainty(DataSplit::Development, &ties, &ties, &registry).unwrap();
        assert_eq!(all_ties.paired_difference_mean, 0.0);
        assert!(all_ties.paired_difference_ci.lower < 0.0);
        assert!(all_ties.paired_difference_ci.upper > 0.0);

        let v6 = matches(&[false; MAX_CASES_PER_RUN as usize]);
        let c6 = matches(&[true; MAX_CASES_PER_RUN as usize]);
        let all_c6_wins =
            summarize_paired_uncertainty(DataSplit::Development, &v6, &c6, &registry).unwrap();
        assert_eq!(all_c6_wins.paired_difference_mean, 1.0);
        assert!(all_c6_wins.paired_difference_ci.lower < 1.0);
        assert_eq!(all_c6_wins.paired_difference_ci.upper, 1.0);
    }

    #[test]
    fn paired_uncertainty_rejects_empty_mismatch_nan_and_registry_drift() {
        let registry = MetricRegistry::pinned();
        let v6 = matches(&[true, false]);
        let c6 = matches(&[true, true]);

        assert_eq!(
            summarize_paired_uncertainty(DataSplit::Development, &[], &c6, &registry),
            Err(EvalError::PairedUncertaintyInvalid {
                reason: "empty_pairs",
            })
        );
        assert_eq!(
            summarize_paired_uncertainty(DataSplit::Development, &v6, &[], &registry),
            Err(EvalError::PairedUncertaintyInvalid {
                reason: "empty_pairs",
            })
        );
        assert_eq!(
            summarize_paired_uncertainty(DataSplit::Development, &v6, &matches(&[true]), &registry),
            Err(EvalError::PairedUncertaintyInvalid {
                reason: "length_mismatch",
            })
        );
        assert_eq!(
            summarize_paired_uncertainty(DataSplit::Development, &v6, &c6, &registry).and_then(
                |_| {
                    // Direct non-finite gate on ConfidenceInterval::checked.
                    ConfidenceInterval::checked(
                        f64::NAN,
                        1.0,
                        PAIRED_UNCERTAINTY_LEVEL,
                        UncertaintyMethod::WilsonScore,
                    )
                }
            ),
            Err(EvalError::PairedUncertaintyInvalid {
                reason: "non_finite",
            })
        );
        assert_eq!(
            ConfidenceInterval::checked(
                0.0,
                f64::INFINITY,
                PAIRED_UNCERTAINTY_LEVEL,
                UncertaintyMethod::BoundedHoeffdingPairedDifference,
            ),
            Err(EvalError::PairedUncertaintyInvalid {
                reason: "non_finite",
            })
        );

        let mut drifted = MetricRegistry::pinned();
        drifted.registry_contract = "not-paired-uncertainty-registry";
        assert_eq!(
            summarize_paired_uncertainty(DataSplit::Development, &v6, &c6, &drifted),
            Err(EvalError::ContractMismatch {
                field: "metric_registry_contract",
            })
        );

        // Same contract pin but empty secondaries drifts away from MetricRegistry::pinned().
        let malformed = MetricRegistry {
            secondaries: &[],
            ..MetricRegistry::pinned()
        };
        assert_eq!(
            summarize_paired_uncertainty(DataSplit::Development, &v6, &c6, &malformed),
            Err(EvalError::MetricRegistryInvalid {
                reason: "secondary_sequence_mismatch",
            })
        );

        assert_eq!(
            summarize_paired_uncertainty(
                DataSplit::Development,
                &matches(&[true]),
                &matches(&[false]),
                &registry,
            ),
            Err(EvalError::PairedUncertaintyInvalid {
                reason: "insufficient_pairs",
            })
        );

        let oversized =
            vec![RevealedMatchOutcome::from_matches_oracle(true); MAX_CASES_PER_RUN as usize + 1];
        assert_eq!(
            summarize_paired_uncertainty(DataSplit::Development, &oversized, &oversized, &registry,),
            Err(EvalError::PairedUncertaintyInvalid {
                reason: "too_many_pairs",
            })
        );

        assert_eq!(
            parse_non_final_split("protected"),
            Err(EvalError::ProtectedOrFinalSplit)
        );
        assert_eq!(
            parse_non_final_split("final"),
            Err(EvalError::ProtectedOrFinalSplit)
        );
    }

    #[test]
    fn paired_uncertainty_from_records_accepts_scored_rejects_failures() {
        let registry = MetricRegistry::pinned();
        let v6_records = [
            EvalRecord {
                arm: EvalArm::V6,
                split: DataSplit::Development,
                family: TaskFamily::ReflectionDiscriminative,
                case_id: 1,
                group_id: 1,
                outcome: EvalOutcome::Scored {
                    score: 1.0,
                    correct: true,
                },
                canonical_digest: "pair-a".into(),
                envelope_contract: EVALUATOR_ENVELOPE_CONTRACT,
                arm_contract: V6_EVALUATOR_CONTRACT,
                budget_contract: READOUT_BUDGET_CONTRACT,
                metric_registry_contract: METRIC_REGISTRY_CONTRACT,
                vector_contract: VECTOR6_CONTRACT,
                label_contract: PROTECTED_LABEL_CONTRACT,
            },
            EvalRecord {
                arm: EvalArm::V6,
                split: DataSplit::Development,
                family: TaskFamily::ReflectionDiscriminative,
                case_id: 2,
                group_id: 1,
                outcome: EvalOutcome::Scored {
                    score: -1.0,
                    correct: false,
                },
                canonical_digest: "pair-b".into(),
                envelope_contract: EVALUATOR_ENVELOPE_CONTRACT,
                arm_contract: V6_EVALUATOR_CONTRACT,
                budget_contract: READOUT_BUDGET_CONTRACT,
                metric_registry_contract: METRIC_REGISTRY_CONTRACT,
                vector_contract: VECTOR6_CONTRACT,
                label_contract: PROTECTED_LABEL_CONTRACT,
            },
        ];
        let c6_records = [
            EvalRecord {
                arm: EvalArm::C6,
                split: DataSplit::Development,
                family: TaskFamily::ReflectionDiscriminative,
                case_id: 1,
                group_id: 1,
                outcome: EvalOutcome::Scored {
                    score: 1.0,
                    correct: true,
                },
                canonical_digest: "pair-a".into(),
                envelope_contract: EVALUATOR_ENVELOPE_CONTRACT,
                arm_contract: C6_EVALUATOR_CONTRACT,
                budget_contract: READOUT_BUDGET_CONTRACT,
                metric_registry_contract: METRIC_REGISTRY_CONTRACT,
                vector_contract: CHIRAL_CONTRACT,
                label_contract: PROTECTED_LABEL_CONTRACT,
            },
            EvalRecord {
                arm: EvalArm::C6,
                split: DataSplit::Development,
                family: TaskFamily::ReflectionDiscriminative,
                case_id: 2,
                group_id: 1,
                outcome: EvalOutcome::Scored {
                    score: 1.0,
                    correct: true,
                },
                canonical_digest: "pair-b".into(),
                envelope_contract: EVALUATOR_ENVELOPE_CONTRACT,
                arm_contract: C6_EVALUATOR_CONTRACT,
                budget_contract: READOUT_BUDGET_CONTRACT,
                metric_registry_contract: METRIC_REGISTRY_CONTRACT,
                vector_contract: CHIRAL_CONTRACT,
                label_contract: PROTECTED_LABEL_CONTRACT,
            },
        ];

        let summary = summarize_paired_uncertainty_from_records(
            DataSplit::Development,
            &v6_records,
            &c6_records,
            &registry,
        )
        .unwrap();
        assert_eq!(summary.n_pairs, 2);
        assert!((summary.v6_accuracy - 0.5).abs() < 1e-12);
        assert!((summary.c6_accuracy - 1.0).abs() < 1e-12);
        assert!((summary.paired_difference_mean - 0.5).abs() < 1e-12);
        assert_eq!(sum_squared_group_sizes(&v6_records).unwrap(), 4);
        let mut same_numeric_group_different_family = v6_records.clone();
        same_numeric_group_different_family[1].family = TaskFamily::ReflectionNuisance;
        assert_eq!(
            sum_squared_group_sizes(&same_numeric_group_different_family).unwrap(),
            2
        );
        assert_eq!(
            summary.v6_accuracy_ci.method,
            UncertaintyMethod::ClusterHoeffdingBernoulliMean
        );
        assert_eq!(
            summary.c6_accuracy_ci.method,
            UncertaintyMethod::ClusterHoeffdingBernoulliMean
        );
        assert_eq!(
            summary.paired_difference_ci.method,
            UncertaintyMethod::ClusterHoeffdingPairedDifference
        );
        assert!(summary.paired_difference_ci.lower < summary.paired_difference_ci.upper);

        // Typed surface admits only RevealedMatchOutcome / EvalRecord scores —
        // there is no ProtectedLabel constructor on the uncertainty API.
        let _revealed = RevealedMatchOutcome::from_matches_oracle(true);
        assert!(_revealed.matches_oracle);

        let mut failed = c6_records.clone();
        failed[1].outcome = EvalOutcome::Failure(EvalFailure::Task);
        assert_eq!(
            summarize_paired_uncertainty_from_records(
                DataSplit::Development,
                &v6_records,
                &failed,
                &registry,
            ),
            Err(EvalError::PairedUncertaintyInvalid {
                reason: "unscored_outcome",
            })
        );

        let mut wrong_arm = v6_records.clone();
        wrong_arm[0].arm = EvalArm::C6;
        assert_eq!(
            revealed_matches_from_records(&wrong_arm, EvalArm::V6, DataSplit::Development),
            Err(EvalError::PairedUncertaintyInvalid {
                reason: "arm_mismatch",
            })
        );

        let oversized_records = vec![v6_records[0].clone(); MAX_CASES_PER_RUN as usize + 1];
        assert_eq!(
            revealed_matches_from_records(&oversized_records, EvalArm::V6, DataSplit::Development,),
            Err(EvalError::PairedUncertaintyInvalid {
                reason: "too_many_pairs",
            })
        );
        let oversized_c6_records = vec![c6_records[0].clone(); MAX_CASES_PER_RUN as usize + 1];
        assert_eq!(
            summarize_paired_uncertainty_from_records(
                DataSplit::Development,
                &oversized_records,
                &oversized_c6_records,
                &registry,
            ),
            Err(EvalError::PairedUncertaintyInvalid {
                reason: "too_many_pairs",
            })
        );

        let mut stale_arm_contract = v6_records.clone();
        stale_arm_contract[0].arm_contract = "stale-v6-evaluator";
        assert_eq!(
            revealed_matches_from_records(&stale_arm_contract, EvalArm::V6, DataSplit::Development,),
            Err(EvalError::ContractMismatch {
                field: "arm_contract",
            })
        );

        let mut stale_budget_contract = v6_records.clone();
        stale_budget_contract[0].budget_contract = "stale-readout-budget";
        assert_eq!(
            revealed_matches_from_records(
                &stale_budget_contract,
                EvalArm::V6,
                DataSplit::Development,
            ),
            Err(EvalError::ContractMismatch {
                field: "budget_contract",
            })
        );

        let mut stale_vector_contract = c6_records.clone();
        stale_vector_contract[0].vector_contract = VECTOR6_CONTRACT;
        assert_eq!(
            revealed_matches_from_records(
                &stale_vector_contract,
                EvalArm::C6,
                DataSplit::Development,
            ),
            Err(EvalError::ContractMismatch {
                field: "vector_contract",
            })
        );

        let mut stale_label_contract = v6_records.clone();
        stale_label_contract[0].label_contract = "stale-label-contract";
        assert_eq!(
            revealed_matches_from_records(
                &stale_label_contract,
                EvalArm::V6,
                DataSplit::Development,
            ),
            Err(EvalError::ContractMismatch {
                field: "label_contract",
            })
        );

        let mut mismatched = c6_records.clone();
        mismatched[0].case_id = 99;
        assert_eq!(
            summarize_paired_uncertainty_from_records(
                DataSplit::Development,
                &v6_records,
                &mismatched,
                &registry,
            ),
            Err(EvalError::PairedUncertaintyInvalid {
                reason: "pair_identity_mismatch",
            })
        );

        let mut duplicate_v6 = v6_records.clone();
        duplicate_v6[1].family = duplicate_v6[0].family;
        duplicate_v6[1].case_id = duplicate_v6[0].case_id;
        duplicate_v6[1].group_id = duplicate_v6[0].group_id;
        duplicate_v6[1].canonical_digest = duplicate_v6[0].canonical_digest.clone();
        let mut duplicate_c6 = c6_records.clone();
        duplicate_c6[1].family = duplicate_c6[0].family;
        duplicate_c6[1].case_id = duplicate_c6[0].case_id;
        duplicate_c6[1].group_id = duplicate_c6[0].group_id;
        duplicate_c6[1].canonical_digest = duplicate_c6[0].canonical_digest.clone();
        assert_eq!(
            summarize_paired_uncertainty_from_records(
                DataSplit::Development,
                &duplicate_v6,
                &duplicate_c6,
                &registry,
            ),
            Err(EvalError::PairedUncertaintyInvalid {
                reason: "duplicate_pair_identity",
            })
        );
    }

    #[test]
    fn failure_taxonomy_retains_each_closed_class() {
        assert_eq!(FAILURE_TAXONOMY_CONTRACT, "tdi24-failure-taxonomy-v1");
        assert_eq!(MAX_FAILURES_PER_RUN, MAX_CASES_PER_RUN);

        let cases = [
            (
                FailureClass::Invalid,
                EvalError::ContractMismatch {
                    field: "envelope_contract",
                },
                "contract_mismatch",
            ),
            (
                FailureClass::Numerical,
                EvalError::Numerical(Vector6Error::NonFiniteVector),
                "numerical",
            ),
            (
                FailureClass::Resource,
                EvalError::CaseBudgetExceeded,
                "case_budget_exceeded",
            ),
            (
                FailureClass::Task,
                EvalError::SplitMismatch {
                    expected: DataSplit::Development,
                    actual: DataSplit::Validation,
                },
                "split_mismatch",
            ),
        ];

        let mut ledger = FailureLedger::open(DataSplit::Development).unwrap();
        for (idx, (class, error, code)) in cases.into_iter().enumerate() {
            assert_eq!(classify_eval_error(&error).unwrap(), class);
            assert_eq!(eval_error_message_code(&error), code);
            let record = ledger
                .retain_eval_error(&error, EvalArm::V6, Some(idx as u64))
                .unwrap()
                .clone();
            assert_eq!(record.class, class);
            assert_eq!(record.arm, EvalArm::V6);
            assert_eq!(record.split, DataSplit::Development);
            assert_eq!(record.case_id, Some(idx as u64));
            assert_eq!(record.message_code, code);
            assert_eq!(record.taxonomy_contract, FAILURE_TAXONOMY_CONTRACT);
            validate_failure_record(&record).unwrap();
        }
        assert_eq!(ledger.records().len(), 4);

        // EvalFailure::Contract maps into the campaign Invalid class and is retained.
        let contract = retain_eval_failure(
            EvalFailure::Contract,
            EvalArm::C6,
            DataSplit::Validation,
            Some(99),
        )
        .unwrap();
        assert_eq!(contract.class, FailureClass::Invalid);
        assert_eq!(contract.message_code, "eval_failure_contract");
        assert_eq!(
            classify_eval_failure(EvalFailure::Numerical),
            FailureClass::Numerical
        );
        assert_eq!(classify_eval_failure(EvalFailure::Task), FailureClass::Task);
        assert_eq!(
            classify_eval_failure(EvalFailure::Resource),
            FailureClass::Resource
        );
        assert_eq!(
            classify_eval_failure(EvalFailure::Contract),
            FailureClass::Invalid
        );
    }

    #[test]
    fn failure_taxonomy_rejects_unknown_class_contract_drift_and_protected() {
        assert_eq!(
            parse_failure_class("invented"),
            Err(EvalError::FailureTaxonomyInvalid {
                reason: "unknown_class",
            })
        );
        assert_eq!(
            parse_failure_class(""),
            Err(EvalError::FailureTaxonomyInvalid {
                reason: "empty_class",
            })
        );
        for label in ["invalid", "numerical", "resource", "task"] {
            assert_eq!(parse_failure_class(label).unwrap().as_str(), label);
        }

        assert_eq!(
            retain_failure(
                FailureClass::Invalid,
                EvalArm::V6,
                DataSplit::Development,
                None,
                "",
            ),
            Err(EvalError::FailureTaxonomyInvalid {
                reason: "empty_message_code",
            })
        );

        let drifted = FailureRecord {
            class: FailureClass::Task,
            arm: EvalArm::V6,
            split: DataSplit::Development,
            case_id: None,
            message_code: "split_mismatch",
            taxonomy_contract: "tdi24-failure-taxonomy-v0-drift",
        };
        assert_eq!(
            validate_failure_record(&drifted),
            Err(EvalError::FailureTaxonomyInvalid {
                reason: "contract_drift",
            })
        );

        assert_eq!(
            classify_eval_error(&EvalError::ProtectedOrFinalSplit),
            Err(EvalError::ProtectedOrFinalSplit)
        );
        assert_eq!(
            retain_eval_error(
                &EvalError::ProtectedOrFinalSplit,
                EvalArm::V6,
                DataSplit::Development,
                None,
            ),
            Err(EvalError::ProtectedOrFinalSplit)
        );
        assert_eq!(
            parse_non_final_split("protected"),
            Err(EvalError::ProtectedOrFinalSplit)
        );
        assert_eq!(
            parse_non_final_split("final"),
            Err(EvalError::ProtectedOrFinalSplit)
        );

        // Chiral numerical errors classify as Numerical and retain.
        let chiral = retain_eval_error(
            &EvalError::ChiralNumerical(ChiralError::NonFiniteVector),
            EvalArm::C6,
            DataSplit::Validation,
            Some(7),
        )
        .unwrap();
        assert_eq!(chiral.class, FailureClass::Numerical);
        assert_eq!(chiral.message_code, "chiral_numerical");
        assert_eq!(chiral.arm, EvalArm::C6);

        // Matcher / registry / uncertainty invalids land in Invalid and retain.
        for error in [
            EvalError::InvalidBudget,
            EvalError::UnsupportedArm,
            EvalError::ParameterCountMismatch {
                left_arm: EvalArm::V6,
                right_arm: EvalArm::C6,
                left_parameters: 1,
                right_parameters: 2,
            },
            EvalError::InitializationMismatch {
                left_arm: EvalArm::V6,
                right_arm: EvalArm::C6,
                left_seed: 1,
                right_seed: 2,
            },
            EvalError::OptimizerUpdateBudgetMismatch {
                left_arm: EvalArm::V6,
                right_arm: EvalArm::C6,
                left_examples: 1,
                right_examples: 2,
                left_updates: 3,
                right_updates: 4,
            },
            EvalError::MetricRegistryInvalid {
                reason: "invented_primary",
            },
            EvalError::PairedUncertaintyInvalid {
                reason: "empty_pairs",
            },
            EvalError::FailureTaxonomyInvalid {
                reason: "unknown_class",
            },
        ] {
            let record =
                retain_eval_error(&error, EvalArm::V6, DataSplit::Development, None).unwrap();
            assert_eq!(record.class, FailureClass::Invalid);
            assert_eq!(record.taxonomy_contract, FAILURE_TAXONOMY_CONTRACT);
        }
    }

    #[test]
    fn failure_ledger_retains_eval_failures_and_rejects_capacity_drift() {
        let mut ledger = FailureLedger::open(DataSplit::Validation).unwrap();
        assert_eq!(ledger.split(), DataSplit::Validation);
        assert_eq!(ledger.taxonomy_contract(), FAILURE_TAXONOMY_CONTRACT);

        for failure in [
            EvalFailure::Numerical,
            EvalFailure::Task,
            EvalFailure::Resource,
            EvalFailure::Contract,
        ] {
            let record = ledger
                .retain_eval_failure(failure, EvalArm::C6, Some(1))
                .unwrap()
                .clone();
            assert_eq!(record.class, classify_eval_failure(failure));
            assert_eq!(record.split, DataSplit::Validation);
        }
        assert_eq!(ledger.records().len(), 4);

        let scored = EvalRecord {
            arm: EvalArm::V6,
            split: DataSplit::Development,
            family: TaskFamily::ReflectionDiscriminative,
            case_id: 1,
            group_id: 1,
            outcome: EvalOutcome::Scored {
                score: 1.0,
                correct: true,
            },
            canonical_digest: "unused".to_string(),
            envelope_contract: EVALUATOR_ENVELOPE_CONTRACT,
            arm_contract: V6_EVALUATOR_CONTRACT,
            budget_contract: READOUT_BUDGET_CONTRACT,
            metric_registry_contract: METRIC_REGISTRY_CONTRACT,
            vector_contract: VECTOR6_CONTRACT,
            label_contract: PROTECTED_LABEL_CONTRACT,
        };
        assert_eq!(retain_from_eval_record(&scored).unwrap(), None);

        let failed = EvalRecord {
            outcome: EvalOutcome::Failure(EvalFailure::Resource),
            ..scored.clone()
        };
        let retained = retain_from_eval_record(&failed).unwrap().unwrap();
        assert_eq!(retained.class, FailureClass::Resource);
        assert_eq!(retained.case_id, Some(1));

        // Split mismatch against the open ledger fails closed.
        assert_eq!(
            ledger.retain(retained),
            Err(EvalError::SplitMismatch {
                expected: DataSplit::Validation,
                actual: DataSplit::Development,
            })
        );

        // Fill a Development ledger to capacity and reject the next retain.
        let mut full = FailureLedger::open(DataSplit::Development).unwrap();
        for i in 0..MAX_FAILURES_PER_RUN {
            full.retain_eval_error(&EvalError::InvalidBudget, EvalArm::V6, Some(i))
                .unwrap();
        }
        assert_eq!(full.records().len() as u64, MAX_FAILURES_PER_RUN);
        assert_eq!(
            full.retain_eval_error(&EvalError::InvalidBudget, EvalArm::V6, None),
            Err(EvalError::CaseBudgetExceeded)
        );
        assert_eq!(
            classify_eval_error(&EvalError::CaseBudgetExceeded).unwrap(),
            FailureClass::Resource
        );

        // Stale arm_contract on a source EvalRecord fails closed before retain.
        let stale = EvalRecord {
            arm: EvalArm::V6,
            split: DataSplit::Development,
            family: TaskFamily::ReflectionDiscriminative,
            case_id: 3,
            group_id: 3,
            outcome: EvalOutcome::Failure(EvalFailure::Task),
            canonical_digest: "stale".to_string(),
            envelope_contract: EVALUATOR_ENVELOPE_CONTRACT,
            arm_contract: "tdi24-v6-evaluator-v0-drift",
            budget_contract: READOUT_BUDGET_CONTRACT,
            metric_registry_contract: METRIC_REGISTRY_CONTRACT,
            vector_contract: VECTOR6_CONTRACT,
            label_contract: PROTECTED_LABEL_CONTRACT,
        };
        assert_eq!(
            retain_from_eval_record(&stale),
            Err(EvalError::ContractMismatch {
                field: "arm_contract",
            })
        );
    }
    #[test]
    fn evaluator_open_rejects_budgets_beyond_failure_ledger_capacity() {
        let mut too_many_cases = EvaluatorConfig::v6(DataSplit::Development).unwrap();
        too_many_cases.budget.max_cases = MAX_FAILURES_PER_RUN + 1;
        assert_eq!(
            EvaluatorRun::open(too_many_cases),
            Err(EvalError::InvalidBudget)
        );

        let mut too_many_scalars = EvaluatorConfig::c6(DataSplit::Validation).unwrap();
        too_many_scalars.budget.max_readout_scalars_per_case = MAX_READOUT_SCALARS_PER_CASE + 1;
        assert_eq!(
            EvaluatorRun::open(too_many_scalars),
            Err(EvalError::InvalidBudget)
        );
    }

    fn invalid<T>(reason: &'static str) -> Result<T, EvalError> {
        Err(EvalError::ProvenanceEnvelopeInvalid { reason })
    }

    fn sample_admitted(split: DataSplit, pairs: u64) -> Vec<AdmittedCase> {
        stage_c_preflight_admitted_cases(split, StageCPreflightBudget::bounded(pairs, 0)).unwrap()
    }

    #[test]
    fn provenance_envelope_accepts_pinned_non_final_run() {
        assert_eq!(PROVENANCE_ENVELOPE_CONTRACT, "tdi24-provenance-envelope-v2");
        assert_eq!(
            LEGACY_PROVENANCE_ENVELOPE_CONTRACT,
            "tdi24-provenance-envelope-v1"
        );
        assert_eq!(PROVENANCE_TOOLCHAIN_CHANNEL, "1.97.1");
        assert_eq!(PROVENANCE_TOOLCHAIN_FEATURES, "experimental");
        assert_eq!(PROVENANCE_TOOLCHAIN_ID, "1.97.1/experimental");

        let config = EvaluatorConfig::v6(DataSplit::Development).unwrap();
        let admitted = sample_admitted(DataSplit::Development, 1);
        let envelope =
            ProvenanceEnvelope::for_pinned_stage_c_run(&config, admitted.clone()).unwrap();

        assert_eq!(envelope.provenance_contract, PROVENANCE_ENVELOPE_CONTRACT);
        assert_eq!(envelope.arm, EvalArm::V6);
        assert_eq!(envelope.split, DataSplit::Development);
        assert_eq!(
            envelope.code_identity,
            stage_c_code_identity_bundle(EvalArm::V6)
        );
        assert!(
            envelope
                .code_identity
                .contains(PROVENANCE_SOURCE_DIGEST_CONTRACT)
        );
        assert!(envelope.code_identity.contains(&stage_c_source_digest()));
        assert_eq!(
            envelope.config_identity,
            stage_c_config_identity_bundle(&config)
        );
        assert!(envelope.config_identity.contains("max_cases=64"));
        assert!(envelope.config_identity.contains("updates=0"));
        assert!(envelope.config_identity.contains("primary="));
        assert_eq!(
            envelope.data_identity,
            stage_c_data_identity_bundle(DataSplit::Development, &admitted)
        );
        assert!(envelope.data_identity.contains(SPLIT_MANIFEST_CONTRACT));
        assert!(
            envelope
                .data_identity
                .contains(DATASET_CANONICALIZATION_CONTRACT)
        );
        assert!(envelope.data_identity.contains("cases=8"));
        assert_eq!(
            envelope.seed_identity,
            stage_c_seed_identity(DataSplit::Development, &admitted)
        );
        assert!(envelope.seed_identity.contains(SEED_REGISTRY_CONTRACT));
        assert!(envelope.seed_identity.contains("domain=development"));
        for case in &envelope.admitted_cases {
            assert_eq!(
                case.seed,
                register_seed(SeedDomain::Development, case.family, case.group_id)
            );
        }
        // Toolchain is the compiler that actually built this crate.
        assert!(BUILD_RUSTC_VERSION.starts_with("rustc "));
        assert_eq!(envelope.compiler_identity, BUILD_RUSTC_VERSION);
        assert_eq!(envelope.toolchain_channel, build_rustc_release());
        assert_eq!(envelope.toolchain_features, PROVENANCE_TOOLCHAIN_FEATURES);
        assert_eq!(
            envelope.toolchain_identity(),
            format!(
                "{}/{}",
                build_rustc_release(),
                PROVENANCE_TOOLCHAIN_FEATURES
            )
        );
        validate_provenance_envelope(&envelope).unwrap();
        validate_provenance_binding(&envelope, &config).unwrap();

        let run = EvaluatorRun::open_with_provenance(config, envelope.clone()).unwrap();
        assert_eq!(run.provenance(), Some(&envelope));
        assert!(run.records().is_empty());

        let c6_config = EvaluatorConfig::c6(DataSplit::Validation).unwrap();
        let c6 = ProvenanceEnvelope::for_pinned_stage_c_run(
            &c6_config,
            sample_admitted(DataSplit::Validation, 1),
        )
        .unwrap();
        assert_eq!(c6.arm, EvalArm::C6);
        assert_eq!(c6.split, DataSplit::Validation);
        assert!(c6.code_identity.contains(C6_EVALUATOR_CONTRACT));
        assert!(c6.seed_identity.contains("domain=validation"));
        assert_ne!(c6.data_identity, envelope.data_identity);
    }

    #[test]
    fn rustc_release_parser_fails_closed() {
        assert_eq!(
            parse_rustc_release("rustc 1.97.1 (abcdef012 2026-08-01)"),
            Some("1.97.1")
        );
        assert_eq!(
            parse_rustc_release("rustc 1.99.0-nightly (abc 2026-09-01)"),
            Some("1.99.0-nightly")
        );
        assert_eq!(parse_rustc_release("unavailable"), None);
        assert_eq!(parse_rustc_release("rustc"), None);
        assert_eq!(parse_rustc_release("rustc 1.97"), None);
        assert_eq!(parse_rustc_release("cargo 1.97.1"), None);
        assert!(parse_rustc_release(BUILD_RUSTC_VERSION).is_some());
    }

    #[test]
    fn provenance_envelope_rejects_empty_protected_and_toolchain_drift() {
        let split = DataSplit::Development;
        let admitted = sample_admitted(split, 1);
        let code = stage_c_code_identity_bundle(EvalArm::V6);
        let config_identity = stage_c_config_identity_bundle(&EvaluatorConfig::v6(split).unwrap());
        let data = stage_c_data_identity_bundle(split, &admitted);
        let seed = stage_c_seed_identity(split, &admitted);
        let release = build_rustc_release();
        let build = |code: &str,
                     config: &str,
                     data: &str,
                     seed: &str,
                     cases: Vec<AdmittedCase>,
                     channel: &'static str,
                     features: &'static str| {
            ProvenanceEnvelope::for_non_final_run(
                EvalArm::V6,
                split,
                code,
                config,
                data,
                seed,
                cases,
                channel,
                features,
            )
        };

        assert!(
            build(
                &code,
                &config_identity,
                &data,
                &seed,
                admitted.clone(),
                release,
                PROVENANCE_TOOLCHAIN_FEATURES
            )
            .is_ok()
        );
        let ok = |code: &str, config: &str, data: &str, seed: &str| {
            build(
                code,
                config,
                data,
                seed,
                admitted.clone(),
                release,
                PROVENANCE_TOOLCHAIN_FEATURES,
            )
        };
        assert_eq!(
            ok("", &config_identity, &data, &seed),
            invalid("empty_code_identity")
        );
        assert_eq!(
            ok(&code, "", &data, &seed),
            invalid("empty_config_identity")
        );
        assert_eq!(
            ok(&code, &config_identity, "", &seed),
            invalid("empty_data_identity")
        );
        assert_eq!(
            ok(&code, &config_identity, &data, ""),
            invalid("empty_seed_identity")
        );
        // Semantic labels are no longer accepted as identities.
        assert_eq!(
            ok("code", &config_identity, &data, &seed),
            invalid("code_identity_drift")
        );
        assert_eq!(
            ok(&code, "config", &data, &seed),
            invalid("config_identity_contract_missing")
        );
        assert_eq!(
            ok(&code, &config_identity, "data", &seed),
            invalid("data_identity_drift")
        );
        assert_eq!(
            ok(&code, &config_identity, &data, "seed"),
            invalid("seed_identity_drift")
        );
        assert_eq!(
            build(
                &code,
                &config_identity,
                &data,
                &seed,
                admitted.clone(),
                "",
                PROVENANCE_TOOLCHAIN_FEATURES
            ),
            invalid("empty_toolchain_channel")
        );
        assert_eq!(
            build(
                &code,
                &config_identity,
                &data,
                &seed,
                admitted.clone(),
                release,
                ""
            ),
            invalid("empty_toolchain_features")
        );
        assert_eq!(
            build(
                &code,
                &config_identity,
                &data,
                &seed,
                admitted.clone(),
                "1.96.0-not-the-build-compiler",
                PROVENANCE_TOOLCHAIN_FEATURES
            ),
            invalid("toolchain_drift")
        );
        assert_eq!(
            build(
                &code,
                &config_identity,
                &data,
                &seed,
                admitted.clone(),
                release,
                "default"
            ),
            invalid("toolchain_drift")
        );
        let mut unknown_compiler = ProvenanceEnvelope::for_pinned_stage_c_run(
            &EvaluatorConfig::v6(split).unwrap(),
            admitted.clone(),
        )
        .unwrap();
        unknown_compiler.compiler_identity = "unavailable";
        assert_eq!(
            validate_provenance_envelope(&unknown_compiler),
            invalid("compiler_identity_unavailable")
        );
        unknown_compiler.compiler_identity = "rustc 1.0.0 (000000000 2015-05-15)";
        assert_eq!(
            validate_provenance_envelope(&unknown_compiler),
            invalid("toolchain_drift")
        );

        let overlong = "x".repeat(MAX_PROVENANCE_IDENTITY_BYTES + 1);
        assert_eq!(
            ok(&overlong, &config_identity, &data, &seed),
            invalid("identity_too_long")
        );
        assert_eq!(
            build(
                &code,
                &config_identity,
                &stage_c_data_identity_bundle(split, &[]),
                &stage_c_seed_identity(split, &[]),
                Vec::new(),
                release,
                PROVENANCE_TOOLCHAIN_FEATURES
            ),
            invalid("empty_population")
        );

        assert_eq!(
            parse_non_final_split("protected"),
            Err(EvalError::ProtectedOrFinalSplit)
        );
        assert_eq!(
            parse_non_final_split("final"),
            Err(EvalError::ProtectedOrFinalSplit)
        );

        // Per-case seed domain must match the split.
        let validation_cases = sample_admitted(DataSplit::Validation, 1);
        assert_eq!(
            build(
                &code,
                &config_identity,
                &stage_c_data_identity_bundle(split, &validation_cases),
                &stage_c_seed_identity(split, &validation_cases),
                validation_cases,
                release,
                PROVENANCE_TOOLCHAIN_FEATURES
            ),
            invalid("seed_domain_split_mismatch")
        );

        let mut drifted = ProvenanceEnvelope::for_pinned_stage_c_run(
            &EvaluatorConfig::v6(split).unwrap(),
            admitted.clone(),
        )
        .unwrap();
        drifted.provenance_contract = "tdi24-provenance-envelope-v0-drift";
        assert_eq!(
            validate_provenance_envelope(&drifted),
            invalid("contract_drift")
        );
        drifted.provenance_contract = LEGACY_PROVENANCE_ENVELOPE_CONTRACT;
        assert_eq!(
            validate_provenance_envelope(&drifted),
            invalid("contract_drift")
        );

        // Arm/split/config binding on open_with_provenance.
        let envelope = ProvenanceEnvelope::for_pinned_stage_c_run(
            &EvaluatorConfig::v6(split).unwrap(),
            admitted,
        )
        .unwrap();
        assert_eq!(
            EvaluatorRun::open_with_provenance(
                EvaluatorConfig::c6(split).unwrap(),
                envelope.clone(),
            ),
            invalid("arm_mismatch")
        );
        assert_eq!(
            EvaluatorRun::open_with_provenance(
                EvaluatorConfig::v6(DataSplit::Validation).unwrap(),
                envelope.clone(),
            ),
            invalid("split_mismatch")
        );
        let mut other_budget = EvaluatorConfig::v6(split).unwrap();
        other_budget.budget.max_cases = 8;
        assert_eq!(
            EvaluatorRun::open_with_provenance(other_budget, envelope),
            invalid("config_identity_mismatch")
        );

        assert_eq!(
            classify_eval_error(&EvalError::ProvenanceEnvelopeInvalid {
                reason: "contract_drift",
            })
            .unwrap(),
            FailureClass::Invalid
        );
        assert_eq!(
            eval_error_message_code(&EvalError::ProvenanceEnvelopeInvalid {
                reason: "contract_drift",
            }),
            "provenance_envelope_invalid"
        );

        let bundle = stage_c_config_identity_bundle(&EvaluatorConfig::v6(split).unwrap());
        assert!(bundle.contains(FAILURE_TAXONOMY_CONTRACT));
        assert!(bundle.contains(PROVENANCE_ENVELOPE_CONTRACT));
        assert!(!bundle.contains("freeze"));
    }

    #[test]
    fn stage_c_preflight_retains_failures_and_never_summarises_over_them() {
        let mut ledger = FailureLedger::open(DataSplit::Development).unwrap();
        retain_preflight_outcome(
            Err(EvalError::CaseBudgetExceeded),
            EvalArm::C6,
            9,
            &mut ledger,
        )
        .unwrap();
        assert_eq!(ledger.records().len(), 1);
        assert_eq!(ledger.records()[0].class, FailureClass::Resource);
        assert_eq!(ledger.records()[0].arm, EvalArm::C6);
        assert_eq!(ledger.records()[0].case_id, Some(9));
        assert_eq!(
            retain_preflight_outcome(
                Err(EvalError::ProtectedOrFinalSplit),
                EvalArm::V6,
                0,
                &mut ledger,
            ),
            Err(EvalError::ProtectedOrFinalSplit)
        );
        assert_eq!(ledger.records().len(), 1);

        let report =
            run_stage_c_preflight(DataSplit::Development, StageCPreflightBudget::bounded(1, 0))
                .unwrap();
        // A retained hard failure replacing a dropped record keeps accounting
        // closed, but then the paired summary must be absent.
        let mut failed = report.clone();
        let dropped = failed.c6_records.pop().unwrap();
        failed
            .c6_failures
            .retain_eval_error(
                &EvalError::CaseBudgetExceeded,
                EvalArm::C6,
                Some(dropped.case_id),
            )
            .unwrap();
        assert_eq!(
            validate_stage_c_preflight_report(&failed),
            Err(EvalError::StageCPreflightInvalid {
                reason: "summary_hides_failures",
            })
        );
        failed.paired_summary = None;
        validate_stage_c_preflight_report(&failed).unwrap();

        let mut wrong_arm = report.clone();
        wrong_arm
            .v6_failures
            .retain_eval_error(&EvalError::CaseBudgetExceeded, EvalArm::C6, None)
            .unwrap();
        wrong_arm.v6_records.pop();
        wrong_arm.paired_summary = None;
        assert_eq!(
            validate_stage_c_preflight_report(&wrong_arm),
            Err(EvalError::StageCPreflightInvalid {
                reason: "failure_arm_mismatch",
            })
        );

        assert_eq!(
            classify_eval_error(&EvalError::StageCPreflightInvalid {
                reason: "empty_budget",
            })
            .unwrap(),
            FailureClass::Invalid
        );
        assert_eq!(
            eval_error_message_code(&EvalError::StageCPreflightInvalid {
                reason: "empty_budget",
            }),
            "stage_c_preflight_invalid"
        );
        assert!(
            !stage_c_config_identity_bundle(&EvaluatorConfig::v6(DataSplit::Development).unwrap())
                .contains(STAGE_C_PREFLIGHT_CONTRACT)
        );
    }
}
