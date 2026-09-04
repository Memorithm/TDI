//! Deterministic symbolic task generators for the frozen TDI-9 P1/P2/P3 families.
//!
//! The policy-visible task surface is structurally separated from evaluator-only
//! metadata. Generator seed, hidden difficulty stratum and exact target are kept
//! in [`EvaluatorRecord`], never in [`PolicyTask`]. Concrete final parameters,
//! seed domains and population sizes remain external TDI-9.1 decisions.

use core::fmt;

const SPLITMIX_GAMMA: u64 = 0x9e37_79b9_7f4a_7c15;
const DOMAIN_P1_TARGET: u64 = 0x7031_2d74_6172_6701;
const DOMAIN_P1_PAIR_ORDER: u64 = 0x7031_2d70_6169_7201;
const DOMAIN_P2_TARGET: u64 = 0x7032_2d74_6172_6701;
const DOMAIN_P2_ROW: u64 = 0x7032_2d72_6f77_0001;
const DOMAIN_P3_TARGET: u64 = 0x7033_2d74_6172_6701;
const DOMAIN_P3_PRE_MAGNITUDE: u64 = 0x7033_2d70_7265_0001;
const DOMAIN_P3_RECOVERY_MAGNITUDE: u64 = 0x7033_2d72_6563_0001;

/// Frozen TDI-9 mechanistic task-family vocabulary.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AdaptiveTaskFamily {
    /// P1: staged evidence accumulation.
    StagedEvidenceAccumulation,
    /// P2: verification-sensitive inference.
    VerificationSensitiveInference,
    /// P3: recoverable deceptive fork.
    RecoverableDeceptiveFork,
}

/// Frozen TDI-9 primary difficulty-stratum vocabulary.
///
/// This label is evaluator metadata. It is deliberately absent from
/// [`PolicyTask`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DifficultyStratum {
    Shallow,
    Intermediate,
    Deep,
}

/// Exact binary target used by P1.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BinaryTarget {
    Negative,
    Positive,
}

impl BinaryTarget {
    #[must_use]
    const fn signed_unit(self) -> i8 {
        match self {
            Self::Negative => -1,
            Self::Positive => 1,
        }
    }

    #[must_use]
    const fn opposite_unit(self) -> i8 {
        -self.signed_unit()
    }
}

/// Branch identity used by P3 public events and evaluator targets.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ForkBranch {
    Left,
    Right,
}

impl ForkBranch {
    #[must_use]
    pub const fn opposite(self) -> Self {
        match self {
            Self::Left => Self::Right,
            Self::Right => Self::Left,
        }
    }
}

/// Caller-supplied P1 generator parameters.
///
/// `decisive_step` is the first one-based prefix at which the final majority is
/// mathematically irreversible under the remaining ±1 evidence budget. It is
/// generator/evaluator configuration and is not inserted into policy input.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct P1Config {
    evidence_count: u32,
    decisive_step: u32,
}

impl P1Config {
    pub fn new(evidence_count: u32, decisive_step: u32) -> Result<Self, AdaptiveTaskError> {
        if evidence_count < 3 {
            return Err(AdaptiveTaskError::P1EvidenceCountTooSmall { evidence_count });
        }
        if evidence_count % 2 == 0 {
            return Err(AdaptiveTaskError::P1EvidenceCountMustBeOdd { evidence_count });
        }
        let minimum = evidence_count.div_ceil(2);
        if decisive_step < minimum || decisive_step > evidence_count {
            return Err(AdaptiveTaskError::P1DecisiveStepOutOfRange {
                minimum,
                maximum: evidence_count,
                actual: decisive_step,
            });
        }
        Ok(Self {
            evidence_count,
            decisive_step,
        })
    }

    #[must_use]
    pub const fn evidence_count(self) -> u32 {
        self.evidence_count
    }

    #[must_use]
    pub const fn decisive_step(self) -> u32 {
        self.decisive_step
    }
}

/// Caller-supplied P2 generator parameters.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct P2Config {
    width: u8,
}

impl P2Config {
    /// Width is limited to 2..=63 so every candidate fits in one exact `u64`
    /// while all bit-shifts remain defined.
    pub fn new(width: u8) -> Result<Self, AdaptiveTaskError> {
        if !(2..=63).contains(&width) {
            return Err(AdaptiveTaskError::P2WidthOutOfRange { width });
        }
        Ok(Self { width })
    }

    #[must_use]
    pub const fn width(self) -> u8 {
        self.width
    }
}

/// Caller-supplied P3 generator parameters.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct P3Config {
    pre_contradiction_steps: u32,
    recovery_steps: u32,
}

impl P3Config {
    pub fn new(
        pre_contradiction_steps: u32,
        recovery_steps: u32,
    ) -> Result<Self, AdaptiveTaskError> {
        if pre_contradiction_steps == 0 {
            return Err(AdaptiveTaskError::P3NeedsPreContradictionEvidence);
        }
        if recovery_steps == 0 {
            return Err(AdaptiveTaskError::P3NeedsRecoveryEvidence);
        }
        Ok(Self {
            pre_contradiction_steps,
            recovery_steps,
        })
    }

    #[must_use]
    pub const fn pre_contradiction_steps(self) -> u32 {
        self.pre_contradiction_steps
    }

    #[must_use]
    pub const fn recovery_steps(self) -> u32 {
        self.recovery_steps
    }
}

/// One exact GF(2) parity constraint in P2.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ParityConstraint {
    mask: u64,
    parity: bool,
}

impl ParityConstraint {
    #[must_use]
    pub const fn mask(self) -> u64 {
        self.mask
    }

    #[must_use]
    pub const fn parity(self) -> bool {
        self.parity
    }
}

/// One policy-visible event in the P3 deceptive-fork task.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ForkEvent {
    /// A branch choice becomes available. A checkpoint may later be created by
    /// the execution policy, but checkpoint semantics are not defined here.
    ChoicePoint,
    /// Local branch evidence. Deltas are exact small non-negative integers.
    Evidence {
        left_delta: i16,
        right_delta: i16,
    },
    /// Later ordinary task evidence proves one branch invalid.
    EliminateBranch { branch: ForkBranch },
}

/// Policy-visible P1 task. It contains evidence only.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct P1PolicyTask {
    evidence: Vec<i8>,
}

impl P1PolicyTask {
    #[must_use]
    pub fn evidence(&self) -> &[i8] {
        &self.evidence
    }
}

/// Policy-visible P2 task. The exact target vector is not stored here.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct P2PolicyTask {
    width: u8,
    constraints: Vec<ParityConstraint>,
}

impl P2PolicyTask {
    #[must_use]
    pub const fn width(&self) -> u8 {
        self.width
    }

    #[must_use]
    pub fn constraints(&self) -> &[ParityConstraint] {
        &self.constraints
    }
}

/// Policy-visible P3 task. Hidden correct/decoy labels are not stored here.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct P3PolicyTask {
    events: Vec<ForkEvent>,
}

impl P3PolicyTask {
    #[must_use]
    pub fn events(&self) -> &[ForkEvent] {
        &self.events
    }
}

/// Complete task input that may be supplied to a solver/policy path.
///
/// It intentionally has no seed, difficulty-stratum or evaluator-target field.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PolicyTask {
    P1(P1PolicyTask),
    P2(P2PolicyTask),
    P3(P3PolicyTask),
}

impl PolicyTask {
    #[must_use]
    pub const fn family(&self) -> AdaptiveTaskFamily {
        match self {
            Self::P1(_) => AdaptiveTaskFamily::StagedEvidenceAccumulation,
            Self::P2(_) => AdaptiveTaskFamily::VerificationSensitiveInference,
            Self::P3(_) => AdaptiveTaskFamily::RecoverableDeceptiveFork,
        }
    }

    #[must_use]
    pub fn event_count(&self) -> usize {
        match self {
            Self::P1(task) => task.evidence.len(),
            Self::P2(task) => task.constraints.len(),
            Self::P3(task) => task.events.len(),
        }
    }
}

/// Exact evaluator-owned target for one generated instance.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EvaluatorTarget {
    P1(BinaryTarget),
    P2 { bits: u64, width: u8 },
    P3(ForkBranch),
}

/// Generator-side oracle metadata used to validate task construction.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EvaluatorOracle {
    P1 {
        earliest_decisive_step: u32,
    },
    P2 {
        independent_constraint_count: u8,
    },
    P3 {
        contradiction_event_index: u32,
        decoy_branch: ForkBranch,
    },
}

/// Evaluator-only record separated from the policy input.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct EvaluatorRecord {
    family: AdaptiveTaskFamily,
    stratum: DifficultyStratum,
    seed: u64,
    target: EvaluatorTarget,
    oracle: EvaluatorOracle,
}

impl EvaluatorRecord {
    #[must_use]
    pub const fn family(self) -> AdaptiveTaskFamily {
        self.family
    }

    #[must_use]
    pub const fn stratum(self) -> DifficultyStratum {
        self.stratum
    }

    #[must_use]
    pub const fn seed(self) -> u64 {
        self.seed
    }

    #[must_use]
    pub const fn target(self) -> EvaluatorTarget {
        self.target
    }

    #[must_use]
    pub const fn oracle(self) -> EvaluatorOracle {
        self.oracle
    }
}

/// Generated instance with an explicit policy/evaluator type boundary.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GeneratedTask {
    policy: PolicyTask,
    evaluator: EvaluatorRecord,
}

impl GeneratedTask {
    #[must_use]
    pub fn policy(&self) -> &PolicyTask {
        &self.policy
    }

    #[must_use]
    pub const fn evaluator(&self) -> EvaluatorRecord {
        self.evaluator
    }

    #[must_use]
    pub fn into_parts(self) -> (PolicyTask, EvaluatorRecord) {
        (self.policy, self.evaluator)
    }
}

/// Generate one P1 staged-evidence instance.
pub fn generate_p1(
    config: P1Config,
    stratum: DifficultyStratum,
    seed: u64,
) -> Result<GeneratedTask, AdaptiveTaskError> {
    let target = if derive(seed, DOMAIN_P1_TARGET, 0) & 1 == 0 {
        BinaryTarget::Positive
    } else {
        BinaryTarget::Negative
    };
    let count = usize::try_from(config.evidence_count)
        .map_err(|_| AdaptiveTaskError::EventCountOverflow)?;
    let mut evidence = Vec::new();
    evidence
        .try_reserve_exact(count)
        .map_err(|_| AdaptiveTaskError::AllocationFailed)?;

    let majority_count = config.evidence_count.div_ceil(2);
    let opposite_before = config.decisive_step - majority_count;
    let mut target_before = majority_count - 1;
    let mut opposite_remaining = opposite_before;
    let mut pair_index = 0u64;

    while opposite_remaining > 0 {
        let target_first = derive(seed, DOMAIN_P1_PAIR_ORDER, pair_index) & 1 == 0;
        if target_first {
            evidence.push(target.signed_unit());
            evidence.push(target.opposite_unit());
        } else {
            evidence.push(target.opposite_unit());
            evidence.push(target.signed_unit());
        }
        target_before -= 1;
        opposite_remaining -= 1;
        pair_index = pair_index.wrapping_add(1);
    }
    for _ in 0..target_before {
        evidence.push(target.signed_unit());
    }

    evidence.push(target.signed_unit());
    for _ in config.decisive_step..config.evidence_count {
        evidence.push(target.opposite_unit());
    }

    if evidence.len() != count {
        return Err(AdaptiveTaskError::ConstructionInvariant);
    }
    let earliest = earliest_irreversible_majority(&evidence)
        .ok_or(AdaptiveTaskError::ConstructionInvariant)?;
    if earliest != config.decisive_step {
        return Err(AdaptiveTaskError::ConstructionInvariant);
    }

    Ok(GeneratedTask {
        policy: PolicyTask::P1(P1PolicyTask { evidence }),
        evaluator: EvaluatorRecord {
            family: AdaptiveTaskFamily::StagedEvidenceAccumulation,
            stratum,
            seed,
            target: EvaluatorTarget::P1(target),
            oracle: EvaluatorOracle::P1 {
                earliest_decisive_step: earliest,
            },
        },
    })
}

/// Generate one P2 verification-sensitive parity-system instance.
///
/// The first `width` rows form a unit-lower-triangular GF(2) system and are
/// therefore linearly independent. The unique satisfying bit-vector remains in
/// the evaluator record rather than the policy task.
pub fn generate_p2(
    config: P2Config,
    stratum: DifficultyStratum,
    seed: u64,
) -> Result<GeneratedTask, AdaptiveTaskError> {
    let width = config.width;
    let value_mask = (1u64 << width) - 1;
    let target = derive(seed, DOMAIN_P2_TARGET, 0) & value_mask;
    let mut constraints = Vec::new();
    constraints
        .try_reserve_exact(usize::from(width))
        .map_err(|_| AdaptiveTaskError::AllocationFailed)?;

    for bit in 0..width {
        let lower_mask = if bit == 0 { 0 } else { (1u64 << bit) - 1 };
        let random_lower = derive(seed, DOMAIN_P2_ROW, u64::from(bit)) & lower_mask;
        let row_mask = (1u64 << bit) | random_lower;
        let parity = (target & row_mask).count_ones() % 2 == 1;
        constraints.push(ParityConstraint {
            mask: row_mask,
            parity,
        });
    }

    Ok(GeneratedTask {
        policy: PolicyTask::P2(P2PolicyTask { width, constraints }),
        evaluator: EvaluatorRecord {
            family: AdaptiveTaskFamily::VerificationSensitiveInference,
            stratum,
            seed,
            target: EvaluatorTarget::P2 {
                bits: target,
                width,
            },
            oracle: EvaluatorOracle::P2 {
                independent_constraint_count: width,
            },
        },
    })
}

/// Generate one P3 recoverable deceptive-fork instance.
pub fn generate_p3(
    config: P3Config,
    stratum: DifficultyStratum,
    seed: u64,
) -> Result<GeneratedTask, AdaptiveTaskError> {
    let target = if derive(seed, DOMAIN_P3_TARGET, 0) & 1 == 0 {
        ForkBranch::Left
    } else {
        ForkBranch::Right
    };
    let decoy = target.opposite();
    let total_events_u32 = config
        .pre_contradiction_steps
        .checked_add(config.recovery_steps)
        .and_then(|value| value.checked_add(2))
        .ok_or(AdaptiveTaskError::EventCountOverflow)?;
    let total_events = usize::try_from(total_events_u32)
        .map_err(|_| AdaptiveTaskError::EventCountOverflow)?;
    let mut events = Vec::new();
    events
        .try_reserve_exact(total_events)
        .map_err(|_| AdaptiveTaskError::AllocationFailed)?;
    events.push(ForkEvent::ChoicePoint);

    for index in 0..config.pre_contradiction_steps {
        let magnitude = 1 + (derive(seed, DOMAIN_P3_PRE_MAGNITUDE, u64::from(index)) % 3) as i16;
        events.push(branch_evidence(decoy, magnitude));
    }

    let contradiction_event_index = u32::try_from(events.len())
        .map_err(|_| AdaptiveTaskError::EventCountOverflow)?;
    events.push(ForkEvent::EliminateBranch { branch: decoy });

    for index in 0..config.recovery_steps {
        let magnitude =
            1 + (derive(seed, DOMAIN_P3_RECOVERY_MAGNITUDE, u64::from(index)) % 3) as i16;
        events.push(branch_evidence(target, magnitude));
    }

    if events.len() != total_events {
        return Err(AdaptiveTaskError::ConstructionInvariant);
    }

    Ok(GeneratedTask {
        policy: PolicyTask::P3(P3PolicyTask { events }),
        evaluator: EvaluatorRecord {
            family: AdaptiveTaskFamily::RecoverableDeceptiveFork,
            stratum,
            seed,
            target: EvaluatorTarget::P3(target),
            oracle: EvaluatorOracle::P3 {
                contradiction_event_index,
                decoy_branch: decoy,
            },
        },
    })
}

fn branch_evidence(branch: ForkBranch, magnitude: i16) -> ForkEvent {
    match branch {
        ForkBranch::Left => ForkEvent::Evidence {
            left_delta: magnitude,
            right_delta: 0,
        },
        ForkBranch::Right => ForkEvent::Evidence {
            left_delta: 0,
            right_delta: magnitude,
        },
    }
}

fn earliest_irreversible_majority(evidence: &[i8]) -> Option<u32> {
    let mut sum = 0i64;
    let total = i64::try_from(evidence.len()).ok()?;
    for (index, value) in evidence.iter().copied().enumerate() {
        sum += i64::from(value);
        let observed = i64::try_from(index + 1).ok()?;
        let remaining = total - observed;
        if sum.abs() > remaining {
            return u32::try_from(index + 1).ok();
        }
    }
    None
}

fn derive(seed: u64, domain: u64, index: u64) -> u64 {
    splitmix64(seed ^ domain ^ index.wrapping_mul(SPLITMIX_GAMMA))
}

fn splitmix64(mut value: u64) -> u64 {
    value = value.wrapping_add(SPLITMIX_GAMMA);
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}

/// Typed generator failure.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AdaptiveTaskError {
    P1EvidenceCountTooSmall {
        evidence_count: u32,
    },
    P1EvidenceCountMustBeOdd {
        evidence_count: u32,
    },
    P1DecisiveStepOutOfRange {
        minimum: u32,
        maximum: u32,
        actual: u32,
    },
    P2WidthOutOfRange {
        width: u8,
    },
    P3NeedsPreContradictionEvidence,
    P3NeedsRecoveryEvidence,
    EventCountOverflow,
    AllocationFailed,
    ConstructionInvariant,
}

impl fmt::Display for AdaptiveTaskError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl std::error::Error for AdaptiveTaskError {}

#[cfg(test)]
mod tests {
    use super::{
        AdaptiveTaskFamily, AdaptiveTaskError, DifficultyStratum, EvaluatorOracle,
        EvaluatorTarget, ForkBranch, ForkEvent, P1Config, P2Config, P3Config, PolicyTask,
        generate_p1, generate_p2, generate_p3,
    };

    #[test]
    fn p1_constructs_the_requested_first_irreversible_prefix() {
        for decisive_step in 5..=9 {
            let generated = generate_p1(
                P1Config::new(9, decisive_step).expect("valid P1 config"),
                DifficultyStratum::Intermediate,
                100 + u64::from(decisive_step),
            )
            .expect("P1 generation");
            assert_eq!(
                generated.policy().family(),
                AdaptiveTaskFamily::StagedEvidenceAccumulation
            );
            assert_eq!(
                generated.evaluator().oracle(),
                EvaluatorOracle::P1 {
                    earliest_decisive_step: decisive_step,
                }
            );
            assert_eq!(generated.policy().event_count(), 9);
        }
    }

    #[test]
    fn p1_is_deterministic_for_seed_and_config() {
        let config = P1Config::new(11, 8).expect("valid P1 config");
        let first = generate_p1(config, DifficultyStratum::Deep, 0x1234).expect("P1 generation");
        let second = generate_p1(config, DifficultyStratum::Deep, 0x1234).expect("P1 generation");
        assert_eq!(first, second);
    }

    #[test]
    fn p2_triangular_constraints_have_one_exact_solution() {
        let generated = generate_p2(
            P2Config::new(6).expect("valid P2 config"),
            DifficultyStratum::Shallow,
            0x9876,
        )
        .expect("P2 generation");
        let (target, width) = match generated.evaluator().target() {
            EvaluatorTarget::P2 { bits, width } => (bits, width),
            _ => panic!("unexpected evaluator target"),
        };
        let constraints = match generated.policy() {
            PolicyTask::P2(task) => task.constraints(),
            _ => panic!("unexpected policy task"),
        };
        assert_eq!(constraints.len(), usize::from(width));

        let solutions = (0u64..(1u64 << width))
            .filter(|candidate| {
                constraints.iter().all(|constraint| {
                    let parity = (candidate & constraint.mask()).count_ones() % 2 == 1;
                    parity == constraint.parity()
                })
            })
            .collect::<Vec<_>>();
        assert_eq!(solutions, vec![target]);
    }

    #[test]
    fn p3_makes_the_decoy_locally_preferred_then_explicitly_invalid() {
        let generated = generate_p3(
            P3Config::new(4, 3).expect("valid P3 config"),
            DifficultyStratum::Deep,
            0x55aa,
        )
        .expect("P3 generation");
        let (target, decoy, contradiction_index) = match (
            generated.evaluator().target(),
            generated.evaluator().oracle(),
        ) {
            (
                EvaluatorTarget::P3(target),
                EvaluatorOracle::P3 {
                    contradiction_event_index,
                    decoy_branch,
                },
            ) => (target, decoy_branch, contradiction_event_index),
            _ => panic!("unexpected P3 evaluator record"),
        };
        assert_eq!(decoy, target.opposite());

        let events = match generated.policy() {
            PolicyTask::P3(task) => task.events(),
            _ => panic!("unexpected policy task"),
        };
        assert_eq!(events.first(), Some(&ForkEvent::ChoicePoint));
        assert_eq!(
            events[usize::try_from(contradiction_index).expect("index")],
            ForkEvent::EliminateBranch { branch: decoy }
        );

        let (left_pre, right_pre) = events[1..usize::try_from(contradiction_index).expect("index")]
            .iter()
            .fold((0i64, 0i64), |(left, right), event| match event {
                ForkEvent::Evidence {
                    left_delta,
                    right_delta,
                } => (
                    left + i64::from(*left_delta),
                    right + i64::from(*right_delta),
                ),
                _ => (left, right),
            });
        match decoy {
            ForkBranch::Left => assert!(left_pre > right_pre),
            ForkBranch::Right => assert!(right_pre > left_pre),
        }
    }

    #[test]
    fn evaluator_only_metadata_is_separate_from_policy_task() {
        let generated = generate_p1(
            P1Config::new(7, 6).expect("valid P1 config"),
            DifficultyStratum::Deep,
            44,
        )
        .expect("P1 generation");
        let (policy, evaluator) = generated.into_parts();
        assert_eq!(policy.family(), AdaptiveTaskFamily::StagedEvidenceAccumulation);
        assert_eq!(evaluator.stratum(), DifficultyStratum::Deep);
        assert_eq!(evaluator.seed(), 44);
        assert!(matches!(evaluator.target(), EvaluatorTarget::P1(_)));
    }

    #[test]
    fn invalid_generator_parameters_fail_closed() {
        assert!(matches!(
            P1Config::new(8, 5),
            Err(AdaptiveTaskError::P1EvidenceCountMustBeOdd { .. })
        ));
        assert!(matches!(
            P1Config::new(9, 4),
            Err(AdaptiveTaskError::P1DecisiveStepOutOfRange { .. })
        ));
        assert!(matches!(
            P2Config::new(64),
            Err(AdaptiveTaskError::P2WidthOutOfRange { .. })
        ));
        assert_eq!(
            P3Config::new(0, 1),
            Err(AdaptiveTaskError::P3NeedsPreContradictionEvidence)
        );
        assert_eq!(
            P3Config::new(1, 0),
            Err(AdaptiveTaskError::P3NeedsRecoveryEvidence)
        );
    }
}
