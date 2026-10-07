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
//! No training,
//! confirmatory execution, protected/final evaluation, or scientific claim is
//! authorised here.

use core::fmt;

use super::tdi24_accounting::ScoreArm;
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
pub const PROVENANCE_ENVELOPE_CONTRACT: &str = "tdi24-provenance-envelope-v1";

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

/// Declared rustc channel string for Stage-C provenance (matches CI gate `1.97.1`).
///
/// Software-surface identity only — does **not** invent a configuration-freeze pin
/// and does not authorise protected/final execution.
pub const PROVENANCE_TOOLCHAIN_CHANNEL: &str = "1.97.1";

/// Declared Cargo feature set retained in the provenance toolchain identity.
pub const PROVENANCE_TOOLCHAIN_FEATURES: &str = "experimental";

/// Combined toolchain identity token: `{channel}/{features}`.
pub const PROVENANCE_TOOLCHAIN_ID: &str = "1.97.1/experimental";

/// Maximum UTF-8 byte length admitted for code/config/data/seed identity strings.
pub const MAX_PROVENANCE_IDENTITY_BYTES: usize = 384;

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
    /// Prefer [`Self::open_with_provenance`] when a Stage-C identity envelope is available.
    pub fn open(config: EvaluatorConfig) -> Result<Self, EvalError> {
        Self::open_inner(config, None)
    }

    /// Open a bounded non-final run bound to a validated provenance envelope.
    ///
    /// Fail-closed when the envelope drifts from the run arm/split or fails
    /// [`validate_provenance_envelope`]. Does not execute protected/final data.
    pub fn open_with_provenance(
        config: EvaluatorConfig,
        envelope: ProvenanceEnvelope,
    ) -> Result<Self, EvalError> {
        validate_provenance_envelope(&envelope)?;
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
        Self::open_inner(config, Some(envelope))
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
        | EvalError::LearnedBasisPrototypeInvalid { .. } => Ok(FailureClass::Invalid),
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

/// One immutable provenance envelope for a single non-final evaluator run.
///
/// Captures code/config/data/seed/toolchain identity so later Stage-C preflight
/// (slice 30) can bind run identity without inventing freeze pins.
/// Development/Validation only — protected/final splits are rejected.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProvenanceEnvelope {
    pub provenance_contract: &'static str,
    pub arm: EvalArm,
    pub split: DataSplit,
    /// Code-surface identity (crate/module contract pins or content-digest placeholder).
    pub code_identity: String,
    /// Configuration identity (evaluator/matcher/metric registry contract pins).
    pub config_identity: String,
    /// Data identity (split + dataset/canonical digest contract refs already used).
    pub data_identity: String,
    /// Seed identity (registered seed / group_id domain tags; no new seed material).
    pub seed_identity: String,
    /// Declared rustc channel (see [`PROVENANCE_TOOLCHAIN_CHANNEL`]).
    pub toolchain_channel: &'static str,
    /// Declared Cargo features (see [`PROVENANCE_TOOLCHAIN_FEATURES`]).
    pub toolchain_features: &'static str,
}

impl ProvenanceEnvelope {
    /// Construct and validate a provenance envelope for a Development/Validation run.
    #[allow(clippy::too_many_arguments)]
    pub fn for_non_final_run(
        arm: EvalArm,
        split: DataSplit,
        code_identity: impl Into<String>,
        config_identity: impl Into<String>,
        data_identity: impl Into<String>,
        seed_identity: impl Into<String>,
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
            toolchain_channel,
            toolchain_features,
        };
        validate_provenance_envelope(&envelope)?;
        Ok(envelope)
    }

    /// Canonical Stage-C reference envelope using pinned toolchain channel/features
    /// and identity bundles derived from already-landed contracts / registered seeds.
    pub fn for_pinned_stage_c_run(
        arm: EvalArm,
        split: DataSplit,
        registered: &RegisteredSeed,
        group_id: u64,
    ) -> Result<Self, EvalError> {
        Self::for_non_final_run(
            arm,
            split,
            stage_c_code_identity_bundle(arm),
            stage_c_config_identity_bundle(),
            stage_c_data_identity_bundle(split),
            stage_c_seed_identity(registered, group_id),
            PROVENANCE_TOOLCHAIN_CHANNEL,
            PROVENANCE_TOOLCHAIN_FEATURES,
        )
    }

    /// Combined toolchain identity token `{channel}/{features}`.
    #[must_use]
    pub fn toolchain_identity(&self) -> String {
        format!("{}/{}", self.toolchain_channel, self.toolchain_features)
    }
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

/// Validate a provenance envelope against the closed contract and identity rules.
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
    if envelope.toolchain_channel != PROVENANCE_TOOLCHAIN_CHANNEL
        || envelope.toolchain_features != PROVENANCE_TOOLCHAIN_FEATURES
    {
        return Err(EvalError::ProvenanceEnvelopeInvalid {
            reason: "toolchain_drift",
        });
    }
    if envelope.toolchain_identity() != PROVENANCE_TOOLCHAIN_ID {
        return Err(EvalError::ProvenanceEnvelopeInvalid {
            reason: "toolchain_drift",
        });
    }
    // Seed-domain tag must agree with the declared non-final split.
    let expected_domain = SeedDomain::from_split(envelope.split);
    let domain_needle = format!("domain={}", expected_domain.as_str());
    if !envelope.seed_identity.contains(&domain_needle) {
        return Err(EvalError::ProvenanceEnvelopeInvalid {
            reason: "seed_domain_split_mismatch",
        });
    }
    if !envelope.seed_identity.contains(SEED_REGISTRY_CONTRACT) {
        return Err(EvalError::ProvenanceEnvelopeInvalid {
            reason: "seed_registry_contract_missing",
        });
    }
    Ok(())
}

/// Build a deterministic config-identity bundle from the live Stage-C contract pins.
///
/// Concatenates known evaluator/matcher/metric/uncertainty/failure/provenance
/// contract ids in a stable order. Does not invent freeze pins.
#[must_use]
pub fn stage_c_config_identity_bundle() -> String {
    [
        EVALUATOR_ENVELOPE_CONTRACT,
        READOUT_BUDGET_CONTRACT,
        PARAMETER_COUNT_MATCHER_CONTRACT,
        INITIALIZATION_MATCHER_CONTRACT,
        OPTIMIZER_UPDATE_BUDGET_CONTRACT,
        METRIC_REGISTRY_CONTRACT,
        PAIRED_UNCERTAINTY_CONTRACT,
        FAILURE_TAXONOMY_CONTRACT,
        PROVENANCE_ENVELOPE_CONTRACT,
    ]
    .join("|")
}

/// Build a deterministic code-identity bundle for the V6/C6 evaluator surface.
#[must_use]
pub fn stage_c_code_identity_bundle(arm: EvalArm) -> String {
    format!(
        "{}|{}|{}",
        EVALUATOR_ENVELOPE_CONTRACT,
        arm.evaluator_contract(),
        PROVENANCE_ENVELOPE_CONTRACT
    )
}

/// Build a data-identity bundle from already-used split + dataset canonical refs.
#[must_use]
pub fn stage_c_data_identity_bundle(split: DataSplit) -> String {
    format!(
        "{}|{}|split={}",
        SPLIT_MANIFEST_CONTRACT,
        DATASET_CANONICALIZATION_CONTRACT,
        split.as_str()
    )
}

/// Build a seed-identity string from a registered seed and group_id domain tags.
///
/// Reuses the Slice-18 registry contract and mixed seed — does not invent new
/// seed material.
#[must_use]
pub fn stage_c_seed_identity(registered: &RegisteredSeed, group_id: u64) -> String {
    format!(
        "{}|domain={}|family={}|local={:016x}|mixed={:016x}|group={:016x}",
        SEED_REGISTRY_CONTRACT,
        registered.domain.as_str(),
        registered.family.as_str(),
        registered.local_seed,
        registered.mixed_seed,
        group_id
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

/// Run one bounded Stage-C preflight smoke campaign on a non-final split.
///
/// Deterministic; no RNG, no training, no protected/final population. The
/// initialization / ordering seed and provenance seed identity reuse the
/// Slice-18 registered seed for `(split domain, ReflectionDiscriminative,
/// first_pair_id)` — no new seed material and no freeze pin is invented.
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

    let v6_provenance = ProvenanceEnvelope::for_pinned_stage_c_run(
        EvalArm::V6,
        split,
        &registered,
        budget.first_pair_id,
    )?;
    let c6_provenance = ProvenanceEnvelope::for_pinned_stage_c_run(
        EvalArm::C6,
        split,
        &registered,
        budget.first_pair_id,
    )?;
    let mut v6_config = EvaluatorConfig::v6(split)?;
    v6_config.budget.max_cases = cases_per_arm;
    let mut c6_config = EvaluatorConfig::c6(split)?;
    c6_config.budget.max_cases = cases_per_arm;
    let mut v6_run = EvaluatorRun::open_with_provenance(v6_config, v6_provenance.clone())?;
    let mut c6_run = EvaluatorRun::open_with_provenance(c6_config, c6_provenance.clone())?;
    let mut v6_failures = FailureLedger::open(split)?;
    let mut c6_failures = FailureLedger::open(split)?;

    let last_pair_id = budget.first_pair_id + budget.pairs_per_family;
    for pair_id in budget.first_pair_id..last_pair_id {
        let discriminative = reflection_discriminative_pair_in_split(pair_id, split)
            .map_err(|_| preflight_generation_failed())?;
        for member in [discriminative.right, discriminative.left] {
            evaluate_preflight_pair(
                &seal_reflection_discriminative(&member),
                handedness_sign,
                &mut v6_run,
                &mut c6_run,
                &mut v6_failures,
                &mut c6_failures,
            )?;
        }
        let nuisance = reflection_nuisance_pair_in_split(pair_id, split)
            .map_err(|_| preflight_generation_failed())?;
        for member in [nuisance.canonical, nuisance.reflected] {
            evaluate_preflight_pair(
                &seal_reflection_nuisance(&member),
                reflection_invariant_sign,
                &mut v6_run,
                &mut c6_run,
                &mut v6_failures,
                &mut c6_failures,
            )?;
        }
        let direction = direction_reversal_pair_in_split(pair_id, split)
            .map_err(|_| preflight_generation_failed())?;
        for member in [direction.forward, direction.reverse] {
            evaluate_preflight_pair(
                &seal_direction_reversal(&member),
                direction_sign,
                &mut v6_run,
                &mut c6_run,
                &mut v6_failures,
                &mut c6_failures,
            )?;
        }
        let base_case_id = pair_id
            .checked_mul(STAGE_C_PREFLIGHT_MEMBERS_PER_PAIR)
            .ok_or_else(preflight_generation_failed)?;
        for case_id in [base_case_id, base_case_id + 1] {
            let control = non_chiral_control_case_in_split(case_id, split)
                .map_err(|_| preflight_generation_failed())?;
            evaluate_preflight_pair(
                &seal_non_chiral_control(&control),
                non_chiral_sign,
                &mut v6_run,
                &mut c6_run,
                &mut v6_failures,
                &mut c6_failures,
            )?;
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
    for record in records {
        validate_eval_record_contracts(record)?;
        if record.arm != arm || record.split != split {
            return Err(EvalError::StageCPreflightInvalid {
                reason: "record_binding_mismatch",
            });
        }
    }
    for failure in failures.records() {
        validate_failure_record(failure)?;
        if failure.arm != arm {
            return Err(EvalError::StageCPreflightInvalid {
                reason: "failure_arm_mismatch",
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
    if report.v6_provenance.seed_identity != report.c6_provenance.seed_identity
        || report.v6_provenance.config_identity != report.c6_provenance.config_identity
        || report.v6_provenance.data_identity != report.c6_provenance.data_identity
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
    /// One rotation in the `(0, 3)` plane mixing the two parity sectors.
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
    mixing[plane_index(0, 3)] = PI / 4.0;
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
            let mut first = EvaluatorRun::open(EvaluatorConfig::c6(split).unwrap()).unwrap();
            let a = first
                .evaluate_c6_binary(&labeled, handedness_sign)
                .unwrap()
                .clone();
            let mut second = EvaluatorRun::open(EvaluatorConfig::c6(split).unwrap()).unwrap();
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
        let mut run =
            EvaluatorRun::open(EvaluatorConfig::v6(DataSplit::Development).unwrap()).unwrap();
        let labeled =
            seal_reflection_discriminative(&reflection_discriminative_pair(3).unwrap().right);
        let rendered = run_inference_callback(&labeled, |view| format!("{view:?}"));
        assert!(!rendered.contains("Right"));
        assert!(!rendered.contains("Left"));
        assert!(!rendered.contains("target"));

        let first = run
            .evaluate_v6_binary(&labeled, handedness_sign)
            .unwrap()
            .clone();
        let second = {
            let mut again =
                EvaluatorRun::open(EvaluatorConfig::v6(DataSplit::Development).unwrap()).unwrap();
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
        let mut run =
            EvaluatorRun::open(EvaluatorConfig::v6(DataSplit::Validation).unwrap()).unwrap();
        let labeled = seal_reflection_discriminative(
            &reflection_discriminative_pair_in_split(5, DataSplit::Validation)
                .unwrap()
                .left,
        );
        let record = run.evaluate_v6_binary(&labeled, handedness_sign).unwrap();
        assert_eq!(record.split, DataSplit::Validation);
        assert_eq!(record.family.as_str(), "reflection_discriminative");
    }

    #[test]
    fn v6_rejects_split_mismatch_and_budget_overflow() {
        let mut run =
            EvaluatorRun::open(EvaluatorConfig::v6(DataSplit::Development).unwrap()).unwrap();
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
        let mut limited = EvaluatorRun::open(tight).unwrap();
        let a = seal_non_chiral_control(&non_chiral_control_case(0).unwrap());
        let b = seal_non_chiral_control(&non_chiral_control_case(1).unwrap());
        assert!(limited.evaluate_v6_binary(&a, non_chiral_sign).is_ok());
        assert_eq!(
            limited.evaluate_v6_binary(&b, non_chiral_sign),
            Err(EvalError::CaseBudgetExceeded)
        );
    }

    #[test]
    fn v6_scores_nuisance_and_non_chiral_families_without_oracle_leakage() {
        let mut run =
            EvaluatorRun::open(EvaluatorConfig::v6(DataSplit::Development).unwrap()).unwrap();
        let nuisance = seal_reflection_nuisance(&reflection_nuisance_pair(2).unwrap().canonical);
        let control = seal_non_chiral_control(&non_chiral_control_case(4).unwrap());
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
            let mut probe =
                EvaluatorRun::open(EvaluatorConfig::v6(DataSplit::Development).unwrap()).unwrap();
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

    #[test]
    fn provenance_envelope_accepts_pinned_non_final_run() {
        assert_eq!(PROVENANCE_ENVELOPE_CONTRACT, "tdi24-provenance-envelope-v1");
        assert_eq!(PROVENANCE_TOOLCHAIN_CHANNEL, "1.97.1");
        assert_eq!(PROVENANCE_TOOLCHAIN_FEATURES, "experimental");
        assert_eq!(PROVENANCE_TOOLCHAIN_ID, "1.97.1/experimental");

        let registered = register_seed(
            SeedDomain::Development,
            TaskFamily::ReflectionDiscriminative,
            42,
        );
        let envelope = ProvenanceEnvelope::for_pinned_stage_c_run(
            EvalArm::V6,
            DataSplit::Development,
            &registered,
            0x11,
        )
        .unwrap();

        assert_eq!(envelope.provenance_contract, PROVENANCE_ENVELOPE_CONTRACT);
        assert_eq!(envelope.arm, EvalArm::V6);
        assert_eq!(envelope.split, DataSplit::Development);
        assert_eq!(
            envelope.code_identity,
            stage_c_code_identity_bundle(EvalArm::V6)
        );
        assert_eq!(envelope.config_identity, stage_c_config_identity_bundle());
        assert_eq!(
            envelope.data_identity,
            stage_c_data_identity_bundle(DataSplit::Development)
        );
        assert!(envelope.data_identity.contains(SPLIT_MANIFEST_CONTRACT));
        assert!(
            envelope
                .data_identity
                .contains(DATASET_CANONICALIZATION_CONTRACT)
        );
        assert_eq!(
            envelope.seed_identity,
            stage_c_seed_identity(&registered, 0x11)
        );
        assert!(envelope.seed_identity.contains(SEED_REGISTRY_CONTRACT));
        assert!(envelope.seed_identity.contains("domain=development"));
        assert_eq!(envelope.toolchain_channel, PROVENANCE_TOOLCHAIN_CHANNEL);
        assert_eq!(envelope.toolchain_features, PROVENANCE_TOOLCHAIN_FEATURES);
        assert_eq!(envelope.toolchain_identity(), PROVENANCE_TOOLCHAIN_ID);
        validate_provenance_envelope(&envelope).unwrap();

        let config = EvaluatorConfig::v6(DataSplit::Development).unwrap();
        let run = EvaluatorRun::open_with_provenance(config, envelope.clone()).unwrap();
        assert_eq!(run.provenance(), Some(&envelope));
        assert!(run.records().is_empty());

        let registered_v = register_seed(
            SeedDomain::Validation,
            TaskFamily::ReflectionDiscriminative,
            7,
        );
        let c6 = ProvenanceEnvelope::for_pinned_stage_c_run(
            EvalArm::C6,
            DataSplit::Validation,
            &registered_v,
            0x22,
        )
        .unwrap();
        assert_eq!(c6.arm, EvalArm::C6);
        assert_eq!(c6.split, DataSplit::Validation);
        assert!(c6.code_identity.contains(C6_EVALUATOR_CONTRACT));
        assert!(c6.seed_identity.contains("domain=validation"));
    }

    #[test]
    fn provenance_envelope_rejects_empty_protected_and_toolchain_drift() {
        let registered = register_seed(
            SeedDomain::Development,
            TaskFamily::ReflectionDiscriminative,
            1,
        );
        let seed = stage_c_seed_identity(&registered, 0);
        let data = stage_c_data_identity_bundle(DataSplit::Development);

        assert_eq!(
            ProvenanceEnvelope::for_non_final_run(
                EvalArm::V6,
                DataSplit::Development,
                "",
                "config",
                data.clone(),
                seed.clone(),
                PROVENANCE_TOOLCHAIN_CHANNEL,
                PROVENANCE_TOOLCHAIN_FEATURES,
            ),
            Err(EvalError::ProvenanceEnvelopeInvalid {
                reason: "empty_code_identity",
            })
        );
        assert_eq!(
            ProvenanceEnvelope::for_non_final_run(
                EvalArm::V6,
                DataSplit::Development,
                "code",
                "",
                data.clone(),
                seed.clone(),
                PROVENANCE_TOOLCHAIN_CHANNEL,
                PROVENANCE_TOOLCHAIN_FEATURES,
            ),
            Err(EvalError::ProvenanceEnvelopeInvalid {
                reason: "empty_config_identity",
            })
        );
        assert_eq!(
            ProvenanceEnvelope::for_non_final_run(
                EvalArm::V6,
                DataSplit::Development,
                "code",
                "config",
                "",
                seed.clone(),
                PROVENANCE_TOOLCHAIN_CHANNEL,
                PROVENANCE_TOOLCHAIN_FEATURES,
            ),
            Err(EvalError::ProvenanceEnvelopeInvalid {
                reason: "empty_data_identity",
            })
        );
        assert_eq!(
            ProvenanceEnvelope::for_non_final_run(
                EvalArm::V6,
                DataSplit::Development,
                "code",
                "config",
                data.clone(),
                "",
                PROVENANCE_TOOLCHAIN_CHANNEL,
                PROVENANCE_TOOLCHAIN_FEATURES,
            ),
            Err(EvalError::ProvenanceEnvelopeInvalid {
                reason: "empty_seed_identity",
            })
        );
        assert_eq!(
            ProvenanceEnvelope::for_non_final_run(
                EvalArm::V6,
                DataSplit::Development,
                "code",
                "config",
                data.clone(),
                seed.clone(),
                "",
                PROVENANCE_TOOLCHAIN_FEATURES,
            ),
            Err(EvalError::ProvenanceEnvelopeInvalid {
                reason: "empty_toolchain_channel",
            })
        );
        assert_eq!(
            ProvenanceEnvelope::for_non_final_run(
                EvalArm::V6,
                DataSplit::Development,
                "code",
                "config",
                data.clone(),
                seed.clone(),
                PROVENANCE_TOOLCHAIN_CHANNEL,
                "",
            ),
            Err(EvalError::ProvenanceEnvelopeInvalid {
                reason: "empty_toolchain_features",
            })
        );
        assert_eq!(
            ProvenanceEnvelope::for_non_final_run(
                EvalArm::V6,
                DataSplit::Development,
                "code",
                "config",
                data.clone(),
                seed.clone(),
                "1.96.0",
                PROVENANCE_TOOLCHAIN_FEATURES,
            ),
            Err(EvalError::ProvenanceEnvelopeInvalid {
                reason: "toolchain_drift",
            })
        );
        assert_eq!(
            ProvenanceEnvelope::for_non_final_run(
                EvalArm::V6,
                DataSplit::Development,
                "code",
                "config",
                data.clone(),
                seed.clone(),
                PROVENANCE_TOOLCHAIN_CHANNEL,
                "default",
            ),
            Err(EvalError::ProvenanceEnvelopeInvalid {
                reason: "toolchain_drift",
            })
        );

        let overlong = "x".repeat(MAX_PROVENANCE_IDENTITY_BYTES + 1);
        assert_eq!(
            ProvenanceEnvelope::for_non_final_run(
                EvalArm::C6,
                DataSplit::Validation,
                overlong,
                "config",
                stage_c_data_identity_bundle(DataSplit::Validation),
                stage_c_seed_identity(
                    &register_seed(
                        SeedDomain::Validation,
                        TaskFamily::ReflectionDiscriminative,
                        1,
                    ),
                    0,
                ),
                PROVENANCE_TOOLCHAIN_CHANNEL,
                PROVENANCE_TOOLCHAIN_FEATURES,
            ),
            Err(EvalError::ProvenanceEnvelopeInvalid {
                reason: "identity_too_long",
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

        // Seed domain must match split.
        let mismatched_seed = stage_c_seed_identity(
            &register_seed(
                SeedDomain::Validation,
                TaskFamily::ReflectionDiscriminative,
                1,
            ),
            0,
        );
        assert_eq!(
            ProvenanceEnvelope::for_non_final_run(
                EvalArm::V6,
                DataSplit::Development,
                "code",
                "config",
                data.clone(),
                mismatched_seed,
                PROVENANCE_TOOLCHAIN_CHANNEL,
                PROVENANCE_TOOLCHAIN_FEATURES,
            ),
            Err(EvalError::ProvenanceEnvelopeInvalid {
                reason: "seed_domain_split_mismatch",
            })
        );

        let mut drifted = ProvenanceEnvelope::for_pinned_stage_c_run(
            EvalArm::V6,
            DataSplit::Development,
            &registered,
            0,
        )
        .unwrap();
        drifted.provenance_contract = "tdi24-provenance-envelope-v0-drift";
        assert_eq!(
            validate_provenance_envelope(&drifted),
            Err(EvalError::ProvenanceEnvelopeInvalid {
                reason: "contract_drift",
            })
        );

        // Arm/split binding on open_with_provenance.
        let envelope = ProvenanceEnvelope::for_pinned_stage_c_run(
            EvalArm::V6,
            DataSplit::Development,
            &registered,
            0,
        )
        .unwrap();
        assert_eq!(
            EvaluatorRun::open_with_provenance(
                EvaluatorConfig::c6(DataSplit::Development).unwrap(),
                envelope.clone(),
            ),
            Err(EvalError::ProvenanceEnvelopeInvalid {
                reason: "arm_mismatch",
            })
        );
        assert_eq!(
            EvaluatorRun::open_with_provenance(
                EvaluatorConfig::v6(DataSplit::Validation).unwrap(),
                envelope,
            ),
            Err(EvalError::ProvenanceEnvelopeInvalid {
                reason: "split_mismatch",
            })
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

        let bundle = stage_c_config_identity_bundle();
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
        assert!(!stage_c_config_identity_bundle().contains(STAGE_C_PREFLIGHT_CONTRACT));
    }
}
