//! Baseline registry and paired-evaluation contract for TDI-2.2.
//!
//! Baseline algorithms receive the same observation-only `InductionBatch` as
//! candidate inductors. Expected labels and evaluator verdicts are represented
//! only by the post-induction paired-evaluation surface below, so they cannot be
//! obtained through the baseline execution trait.

use core::fmt::Write as _;

use super::tdi2_induction_input::InductionBatch;
use super::tdi2_induction_split::{InductionDomain, frozen_population};
use super::tdi2_template_induction::EpisodeId;

/// Baseline families preregistered by the TDI-2.2 campaign.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BaselineFamily {
    ExactBooleanOverlap,
    NearestBooleanOverlap,
    PrototypeKnn,
    DecisionTreeRule,
    FirstOrderAntiUnification,
    IlpRuleInduction,
    StructureMapping,
    NumericLearner,
}

impl BaselineFamily {
    /// Stable machine-readable identity used by evidence records.
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            Self::ExactBooleanOverlap => "exact-boolean-overlap",
            Self::NearestBooleanOverlap => "nearest-boolean-overlap",
            Self::PrototypeKnn => "prototype-knn",
            Self::DecisionTreeRule => "decision-tree-rule",
            Self::FirstOrderAntiUnification => "first-order-anti-unification",
            Self::IlpRuleInduction => "ilp-rule-induction",
            Self::StructureMapping => "structure-mapping",
            Self::NumericLearner => "numeric-learner",
        }
    }
}

/// Versioned identity of one concrete baseline implementation/configuration.
///
/// A family is only a comparison category. Reproducible evidence additionally
/// binds the exact implementation/configuration identity supplied by the baseline.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BaselineIdentity {
    family: BaselineFamily,
    algorithm: String,
}

impl BaselineIdentity {
    pub fn new(
        family: BaselineFamily,
        algorithm: impl Into<String>,
    ) -> Result<Self, PairedEvaluationError> {
        let algorithm = algorithm.into();
        if algorithm.is_empty() {
            return Err(PairedEvaluationError::EmptyBaselineIdentity);
        }
        Ok(Self { family, algorithm })
    }

    #[must_use]
    pub const fn family(&self) -> BaselineFamily {
        self.family
    }

    #[must_use]
    pub fn algorithm(&self) -> &str {
        &self.algorithm
    }
}

/// Observation-only execution boundary shared by all TDI-2.2 baselines.
pub trait LabelFreeBaseline<Output> {
    /// Algorithm-specific failure.
    type Error;

    /// Declared baseline family.
    fn family(&self) -> BaselineFamily;

    /// Stable versioned identity of this exact algorithm/configuration.
    fn algorithm_identity(&self) -> &str;

    /// Produce baseline output from exactly the declared observation-only batch.
    fn run(&self, batch: &InductionBatch) -> Result<Output, Self::Error>;
}

/// Post-induction verdict for one evaluation arm.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EvaluationVerdict {
    Correct,
    Incorrect,
    Abstained,
}

impl EvaluationVerdict {
    const fn is_correct(self) -> bool {
        matches!(self, Self::Correct)
    }

    const fn is_abstained(self) -> bool {
        matches!(self, Self::Abstained)
    }

    const fn key(self) -> &'static str {
        match self {
            Self::Correct => "correct",
            Self::Incorrect => "incorrect",
            Self::Abstained => "abstained",
        }
    }
}

/// One matched case: candidate and baseline verdicts are inseparable.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PairedCase {
    episode: EpisodeId,
    candidate: EvaluationVerdict,
    baseline: EvaluationVerdict,
}

impl PairedCase {
    #[must_use]
    pub const fn new(
        episode: EpisodeId,
        candidate: EvaluationVerdict,
        baseline: EvaluationVerdict,
    ) -> Self {
        Self {
            episode,
            candidate,
            baseline,
        }
    }

    #[must_use]
    pub const fn episode(self) -> EpisodeId {
        self.episode
    }

    #[must_use]
    pub const fn candidate(self) -> EvaluationVerdict {
        self.candidate
    }

    #[must_use]
    pub const fn baseline(self) -> EvaluationVerdict {
        self.baseline
    }
}

/// Invalid paired-evaluation construction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PairedEvaluationError {
    EmptyEvaluation,
    EmptyBaselineIdentity,
    DuplicateEpisode {
        episode: EpisodeId,
    },
    EpisodeOutsidePopulation {
        domain: InductionDomain,
        episode: EpisodeId,
    },
}

/// Deterministic matched comparison on one frozen non-final population.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PairedEvaluation {
    domain: InductionDomain,
    baseline: BaselineIdentity,
    cases: Vec<PairedCase>,
}

impl PairedEvaluation {
    pub fn new(
        domain: InductionDomain,
        baseline: BaselineIdentity,
        mut cases: Vec<PairedCase>,
    ) -> Result<Self, PairedEvaluationError> {
        if cases.is_empty() {
            return Err(PairedEvaluationError::EmptyEvaluation);
        }
        let population = frozen_population(domain);
        for case in &cases {
            if !population.contains(case.episode()) {
                return Err(PairedEvaluationError::EpisodeOutsidePopulation {
                    domain,
                    episode: case.episode(),
                });
            }
        }
        cases.sort_unstable_by_key(|case| case.episode());
        if let Some(episode) = cases
            .windows(2)
            .find_map(|pair| (pair[0].episode() == pair[1].episode()).then_some(pair[0].episode()))
        {
            return Err(PairedEvaluationError::DuplicateEpisode { episode });
        }
        Ok(Self {
            domain,
            baseline,
            cases,
        })
    }

    #[must_use]
    pub const fn domain(&self) -> InductionDomain {
        self.domain
    }

    #[must_use]
    pub const fn baseline(&self) -> &BaselineIdentity {
        &self.baseline
    }

    #[must_use]
    pub fn cases(&self) -> &[PairedCase] {
        &self.cases
    }

    #[must_use]
    pub fn summary(&self) -> PairedSummary {
        let mut summary = PairedSummary::default();
        for case in &self.cases {
            summary.observe(case.candidate(), case.baseline());
        }
        summary
    }

    /// Stable matched evidence record. It contains no expected labels.
    ///
    /// In addition to aggregates, this binds the exact evaluated episode set and
    /// per-case paired verdicts so distinct subsets or outcome assignments cannot
    /// collapse to the same evidence identity.
    #[must_use]
    pub fn canonical_record(&self) -> String {
        let summary = self.summary();
        let domain = match self.domain {
            InductionDomain::Development => "development",
            InductionDomain::Validation => "validation",
        };
        let mut record = format!(
            "tdi2.2-paired-evaluation-v2;domain={domain};baseline_family={};baseline_algorithm_hex=",
            self.baseline.family().key(),
        );
        for byte in self.baseline.algorithm().as_bytes() {
            let _ = write!(record, "{byte:02x}");
        }
        record.push_str(";cases=");
        for case in &self.cases {
            let _ = write!(
                record,
                "{}:{}/{}|",
                case.episode().raw(),
                case.candidate().key(),
                case.baseline().key(),
            );
        }
        let _ = write!(
            record,
            ";total={};both_correct={};candidate_only={};baseline_only={};neither_correct={};candidate_abstained={};baseline_abstained={}",
            summary.total,
            summary.both_correct,
            summary.candidate_only,
            summary.baseline_only,
            summary.neither_correct,
            summary.candidate_abstained,
            summary.baseline_abstained,
        );
        record
    }
}

/// Aggregate paired counts with abstention retained explicitly.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PairedSummary {
    pub total: u64,
    pub both_correct: u64,
    pub candidate_only: u64,
    pub baseline_only: u64,
    pub neither_correct: u64,
    pub candidate_abstained: u64,
    pub baseline_abstained: u64,
}

impl PairedSummary {
    fn observe(&mut self, candidate: EvaluationVerdict, baseline: EvaluationVerdict) {
        self.total += 1;
        match (candidate.is_correct(), baseline.is_correct()) {
            (true, true) => self.both_correct += 1,
            (true, false) => self.candidate_only += 1,
            (false, true) => self.baseline_only += 1,
            (false, false) => self.neither_correct += 1,
        }
        self.candidate_abstained += u64::from(candidate.is_abstained());
        self.baseline_abstained += u64::from(baseline.is_abstained());
    }

    #[must_use]
    pub fn net_correct_advantage(self) -> i128 {
        i128::from(self.candidate_only) - i128::from(self.baseline_only)
    }
}

impl core::fmt::Display for PairedEvaluationError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::EmptyEvaluation => formatter.write_str("paired evaluation must not be empty"),
            Self::EmptyBaselineIdentity => {
                formatter.write_str("baseline algorithm/configuration identity must not be empty")
            }
            Self::DuplicateEpisode { episode } => {
                write!(
                    formatter,
                    "episode {} is evaluated more than once",
                    episode.raw()
                )
            }
            Self::EpisodeOutsidePopulation { domain, episode } => write!(
                formatter,
                "episode {} is outside the frozen {domain:?} population",
                episode.raw()
            ),
        }
    }
}

impl std::error::Error for PairedEvaluationError {}

#[cfg(test)]
mod tests {
    use super::{
        BaselineFamily, BaselineIdentity, EvaluationVerdict, PairedCase, PairedEvaluation,
        PairedEvaluationError,
    };
    use crate::experimental::tdi2_induction_split::{
        DEVELOPMENT_START, InductionDomain, VALIDATION_START,
    };
    use crate::experimental::tdi2_template_induction::EpisodeId;

    fn baseline(family: BaselineFamily, algorithm: &str) -> BaselineIdentity {
        BaselineIdentity::new(family, algorithm).expect("baseline identity")
    }

    #[test]
    fn preregistered_baseline_keys_are_stable_and_unique() {
        let families = [
            BaselineFamily::ExactBooleanOverlap,
            BaselineFamily::NearestBooleanOverlap,
            BaselineFamily::PrototypeKnn,
            BaselineFamily::DecisionTreeRule,
            BaselineFamily::FirstOrderAntiUnification,
            BaselineFamily::IlpRuleInduction,
            BaselineFamily::StructureMapping,
            BaselineFamily::NumericLearner,
        ];
        let mut keys = families.map(BaselineFamily::key);
        keys.sort_unstable();
        assert!(keys.windows(2).all(|pair| pair[0] != pair[1]));
    }

    #[test]
    fn paired_contract_preserves_case_matching_and_abstention() {
        let evaluation = PairedEvaluation::new(
            InductionDomain::Development,
            baseline(
                BaselineFamily::NearestBooleanOverlap,
                "nearest-overlap/v1:k=1",
            ),
            vec![
                PairedCase::new(
                    EpisodeId::new(DEVELOPMENT_START + 1),
                    EvaluationVerdict::Correct,
                    EvaluationVerdict::Incorrect,
                ),
                PairedCase::new(
                    EpisodeId::new(DEVELOPMENT_START),
                    EvaluationVerdict::Abstained,
                    EvaluationVerdict::Correct,
                ),
            ],
        )
        .expect("matched evaluation");
        assert_eq!(evaluation.cases()[0].episode().raw(), DEVELOPMENT_START);
        let summary = evaluation.summary();
        assert_eq!(summary.total, 2);
        assert_eq!(summary.candidate_only, 1);
        assert_eq!(summary.baseline_only, 1);
        assert_eq!(summary.candidate_abstained, 1);
        assert_eq!(summary.net_correct_advantage(), 0);
        let record = evaluation.canonical_record();
        assert!(record.contains("total=2"));
        assert!(record.contains("baseline_family=nearest-boolean-overlap"));
        assert!(record.contains("100000:abstained/correct|"));
        assert!(record.contains("100001:correct/incorrect|"));
    }

    #[test]
    fn canonical_record_binds_exact_case_set_and_verdict_assignment() {
        let id = baseline(BaselineFamily::PrototypeKnn, "prototype-knn/v1:k=3");
        let one = PairedEvaluation::new(
            InductionDomain::Development,
            id.clone(),
            vec![PairedCase::new(
                EpisodeId::new(DEVELOPMENT_START),
                EvaluationVerdict::Correct,
                EvaluationVerdict::Incorrect,
            )],
        )
        .expect("one");
        let other_case_same_counts = PairedEvaluation::new(
            InductionDomain::Development,
            id.clone(),
            vec![PairedCase::new(
                EpisodeId::new(DEVELOPMENT_START + 1),
                EvaluationVerdict::Correct,
                EvaluationVerdict::Incorrect,
            )],
        )
        .expect("other case");
        assert_eq!(one.summary(), other_case_same_counts.summary());
        assert_ne!(
            one.canonical_record(),
            other_case_same_counts.canonical_record()
        );

        let swapped_verdicts = PairedEvaluation::new(
            InductionDomain::Development,
            id,
            vec![PairedCase::new(
                EpisodeId::new(DEVELOPMENT_START),
                EvaluationVerdict::Incorrect,
                EvaluationVerdict::Correct,
            )],
        )
        .expect("swapped");
        assert_ne!(one.canonical_record(), swapped_verdicts.canonical_record());
    }

    #[test]
    fn same_family_different_algorithm_configurations_have_distinct_evidence() {
        let case = PairedCase::new(
            EpisodeId::new(DEVELOPMENT_START),
            EvaluationVerdict::Incorrect,
            EvaluationVerdict::Correct,
        );
        let k3 = PairedEvaluation::new(
            InductionDomain::Development,
            baseline(BaselineFamily::PrototypeKnn, "prototype-knn/v1:k=3"),
            vec![case],
        )
        .expect("k3");
        let k5 = PairedEvaluation::new(
            InductionDomain::Development,
            baseline(BaselineFamily::PrototypeKnn, "prototype-knn/v1:k=5"),
            vec![case],
        )
        .expect("k5");
        assert_ne!(k3.canonical_record(), k5.canonical_record());
        assert_eq!(
            BaselineIdentity::new(BaselineFamily::PrototypeKnn, ""),
            Err(PairedEvaluationError::EmptyBaselineIdentity)
        );
    }

    #[test]
    fn duplicate_or_cross_population_cases_fail_closed() {
        let duplicate = PairedCase::new(
            EpisodeId::new(DEVELOPMENT_START),
            EvaluationVerdict::Correct,
            EvaluationVerdict::Correct,
        );
        assert_eq!(
            PairedEvaluation::new(
                InductionDomain::Development,
                baseline(BaselineFamily::PrototypeKnn, "prototype-knn/v1:k=3"),
                vec![duplicate, duplicate],
            ),
            Err(PairedEvaluationError::DuplicateEpisode {
                episode: EpisodeId::new(DEVELOPMENT_START)
            })
        );
        assert_eq!(
            PairedEvaluation::new(
                InductionDomain::Development,
                baseline(BaselineFamily::PrototypeKnn, "prototype-knn/v1:k=3"),
                vec![PairedCase::new(
                    EpisodeId::new(VALIDATION_START),
                    EvaluationVerdict::Incorrect,
                    EvaluationVerdict::Incorrect,
                )],
            ),
            Err(PairedEvaluationError::EpisodeOutsidePopulation {
                domain: InductionDomain::Development,
                episode: EpisodeId::new(VALIDATION_START)
            })
        );
    }
}
