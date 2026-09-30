// Included as tdi25_eval::matched_reference so outcome minting stays inside
// the evaluator privacy boundary. See the prospectively committed protocol
// docs/TDI-25-MATCHED-REFERENCE-V1.md. Development/Validation only.

use super::{
    ComparisonArm, DataSplit, EvalError, MAX_CASES_PER_RUN, MAX_SEED_BLOCKS_PER_SYNTHESIS,
    RevealedMatchOutcome, TaskFamily, Tdi25Error,
};
use crate::experimental::tdi22_torsor::{Torsor3, Twist3, Vec3, direct_pairing};
use crate::experimental::tdi24_chiral::{Chiral6, ChiralScoreWeights};
use crate::experimental::tdi25_tasks::{SeedDomain, mix_registered_seed};
use crate::experimental::tdi25_torsor_chiral::{
    chiral_arm_score, torsor_arm_score, validate_source_contracts,
};

/// Explicitly distinct from the legacy Phase-B population.
pub const MATCHED_POPULATION_CONTRACT: &str = "tdi25-matched-reference-population-v1";
/// Version of the fixed, non-trained, paired reference evaluation.
pub const MATCHED_REFERENCE_CONTRACT: &str = "tdi25-matched-reference-evaluator-v1";
/// Versioned primary paths, distinct from the legacy arm-specific evaluators.
pub const MATCHED_T6_CONTRACT: &str = "tdi25-matched-t6-evaluator-v1";
pub const MATCHED_C6_CONTRACT: &str = "tdi25-matched-c6-evaluator-v1";
/// Shared numeric input, excluding identity, target, allocation and reports.
pub const SHARED_INPUT_SCALARS: usize = 18;
/// Fixed score weights used by C6 on every family, without training.
pub const MATCHED_CHIRAL_WEIGHTS: ChiralScoreWeights = ChiralScoreWeights {
    alpha: 1.0,
    beta: 0.0,
    gamma: 1.0,
};

/// One registered common target for each declared family.
#[must_use]
pub const fn common_target_contract(family: TaskFamily) -> &'static str {
    match family {
        TaskFamily::TorsorFavorable => "tdi25-matched-target-torsor-v1",
        TaskFamily::ChiralFavorable => "tdi25-matched-target-chiral-v1",
        TaskFamily::Mixed => "tdi25-matched-target-mixed-v1",
        TaskFamily::Neutral => "tdi25-matched-target-neutral-v1",
    }
}

/// The identical immutable numeric view supplied to both scoring functions.
/// No family, split, seed, target, correctness or alternative-arm score exists
/// in this type. There is deliberately no public constructor or mutator.
#[derive(Clone, Debug, PartialEq)]
pub struct MatchedInput {
    query: [f64; 6],
    key: [f64; 6],
    key_position: [f64; 3],
    query_position: [f64; 3],
}

impl MatchedInput {
    #[must_use]
    pub const fn query(&self) -> [f64; 6] {
        self.query
    }
    #[must_use]
    pub const fn key(&self) -> [f64; 6] {
        self.key
    }
    #[must_use]
    pub const fn key_position(&self) -> [f64; 3] {
        self.key_position
    }
    #[must_use]
    pub const fn query_position(&self) -> [f64; 3] {
        self.query_position
    }
}

/// A complete bounded run of both fixed primary scorers on one common block.
/// Only actual scoring can mint its retained outcomes. Callers may borrow
/// reporting fields but cannot construct a run or inject a target or match bit.
#[derive(Clone, Debug, PartialEq)]
pub struct MatchedPrimaryRun {
    split: DataSplit,
    family: TaskFamily,
    seed_block: u64,
    inputs: Vec<MatchedInput>,
    t6_scores: Vec<f64>,
    c6_scores: Vec<f64>,
    t6_outcomes: Vec<RevealedMatchOutcome>,
    c6_outcomes: Vec<RevealedMatchOutcome>,
}

impl MatchedPrimaryRun {
    /// Execute a prefix-stable block under the fixed v1 protocol.
    /// Budgets are validated before allocation or scoring. There are no
    /// callbacks, learned parameters, retries or protected/final split paths.
    pub fn evaluate(
        split: DataSplit,
        family: TaskFamily,
        seed_block: u64,
        n_cases: u64,
    ) -> Result<Self, EvalError> {
        if !(2..=MAX_CASES_PER_RUN).contains(&n_cases)
            || seed_block >= MAX_SEED_BLOCKS_PER_SYNTHESIS as u64
        {
            return Err(EvalError::InvalidBudget);
        }
        validate_source_contracts().map_err(EvalError::Bridge)?;
        let mut run = Self {
            split,
            family,
            seed_block,
            inputs: Vec::with_capacity(n_cases as usize),
            t6_scores: Vec::with_capacity(n_cases as usize),
            c6_scores: Vec::with_capacity(n_cases as usize),
            t6_outcomes: Vec::with_capacity(n_cases as usize),
            c6_outcomes: Vec::with_capacity(n_cases as usize),
        };
        for case_id in 0..n_cases {
            let input = generate_input(split, family, seed_block, case_id);
            // Target is derived independently before either scorer is invoked.
            let target = common_target(&input, family)?;
            let t6_score = score_t6(&input)?;
            let c6_score = score_c6(&input)?;
            let identity = canonical_identity(split, family, seed_block, case_id, &input);
            let seal = |arm, score| {
                let mut outcome = RevealedMatchOutcome::from_evaluator_record(
                    arm,
                    split,
                    family,
                    seed_block,
                    case_id,
                    identity.clone(),
                    shared_match(score, target),
                );
                outcome.shared_target_contract = Some(common_target_contract(family));
                outcome.integrity.shared_target_contract = outcome.shared_target_contract;
                outcome
            };
            run.t6_outcomes.push(seal(ComparisonArm::T6, t6_score));
            run.c6_outcomes.push(seal(ComparisonArm::C6, c6_score));
            run.t6_scores.push(t6_score);
            run.c6_scores.push(c6_score);
            run.inputs.push(input);
        }
        Ok(run)
    }

    #[must_use]
    pub const fn split(&self) -> DataSplit {
        self.split
    }
    #[must_use]
    pub const fn family(&self) -> TaskFamily {
        self.family
    }
    #[must_use]
    pub const fn seed_block(&self) -> u64 {
        self.seed_block
    }
    #[must_use]
    pub fn inputs(&self) -> &[MatchedInput] {
        &self.inputs
    }
    #[must_use]
    pub fn t6_scores(&self) -> &[f64] {
        &self.t6_scores
    }
    #[must_use]
    pub fn c6_scores(&self) -> &[f64] {
        &self.c6_scores
    }
    #[must_use]
    pub fn t6_outcomes(&self) -> &[RevealedMatchOutcome] {
        &self.t6_outcomes
    }
    #[must_use]
    pub fn c6_outcomes(&self) -> &[RevealedMatchOutcome] {
        &self.c6_outcomes
    }
    /// Numeric input payload only; not resident/peak process memory.
    #[must_use]
    pub const fn shared_input_bytes_per_case(&self) -> usize {
        SHARED_INPUT_SCALARS * core::mem::size_of::<f64>()
    }
}

fn generate_input(split: DataSplit, family: TaskFamily, block: u64, case_id: u64) -> MatchedInput {
    // Caller validates block < 64 and case_id < 64, so the index cannot overflow.
    let mut state =
        mix_registered_seed(SeedDomain::from_split(split), family, block * 64 + case_id);
    if state == 0 {
        state = 1;
    }
    let mut draw = || {
        state ^= state >> 12;
        state ^= state << 25;
        state ^= state >> 27;
        let word = state.wrapping_mul(2_685_821_657_736_338_717);
        ((word >> 61) as i64 - 4) as f64 / 2.0
    };
    let query = core::array::from_fn(|_| draw());
    let key = core::array::from_fn(|_| draw());
    let mut key_position = core::array::from_fn(|_| draw());
    let mut query_position = core::array::from_fn(|_| draw());
    if matches!(family, TaskFamily::ChiralFavorable | TaskFamily::Neutral) {
        key_position = [0.0; 3];
        query_position = [0.0; 3];
    }
    MatchedInput {
        query,
        key,
        key_position,
        query_position,
    }
}

fn vec3(data: [f64; 3]) -> Result<Vec3, EvalError> {
    Vec3::new(data[0], data[1], data[2])
        .map_err(Tdi25Error::Torsor)
        .map_err(EvalError::Bridge)
}

fn torsor_carriers(input: &MatchedInput) -> Result<(Twist3, Torsor3, Vec3), EvalError> {
    let q = input.query;
    let k = input.key;
    let query = Twist3::new(vec3([q[0], q[1], q[2]])?, vec3([q[3], q[4], q[5]])?)
        .map_err(Tdi25Error::Torsor)
        .map_err(EvalError::Bridge)?;
    let key = Torsor3::new(
        vec3([k[0], k[1], k[2]])?,
        vec3([k[3], k[4], k[5]])?,
        vec3(input.key_position)?,
    )
    .map_err(Tdi25Error::Torsor)
    .map_err(EvalError::Bridge)?;
    Ok((query, key, vec3(input.query_position)?))
}

fn chiral_carriers(input: &MatchedInput) -> Result<(Chiral6, Chiral6), EvalError> {
    let query = Chiral6::from_array(input.query)
        .map_err(Tdi25Error::Chiral)
        .map_err(EvalError::Bridge)?;
    let key = Chiral6::from_array(input.key)
        .map_err(Tdi25Error::Chiral)
        .map_err(EvalError::Bridge)?;
    Ok((query, key))
}

fn score_t6(input: &MatchedInput) -> Result<f64, EvalError> {
    let (query, key, position) = torsor_carriers(input)?;
    torsor_arm_score(query, key, position).map_err(EvalError::Bridge)
}

fn score_c6(input: &MatchedInput) -> Result<f64, EvalError> {
    let (query, key) = chiral_carriers(input)?;
    chiral_arm_score(query, key, MATCHED_CHIRAL_WEIGHTS).map_err(EvalError::Bridge)
}

fn common_target(input: &MatchedInput, family: TaskFamily) -> Result<f64, EvalError> {
    let torsor_target = || {
        let (q, k, position) = torsor_carriers(input)?;
        // Independent upstream direct oracle, not the factorized candidate.
        direct_pairing(q, k, position)
            .map_err(Tdi25Error::Torsor)
            .map_err(EvalError::Bridge)
    };
    let chiral_target = || {
        let (q, k) = chiral_carriers(input)?;
        let direct = q
            .dot(k)
            .map_err(Tdi25Error::Chiral)
            .map_err(EvalError::Bridge)?;
        let chi = q
            .chiral_pairing(k)
            .map_err(Tdi25Error::Chiral)
            .map_err(EvalError::Bridge)?;
        Ok::<f64, EvalError>(direct + chi)
    };
    let target = match family {
        TaskFamily::TorsorFavorable => torsor_target()?,
        TaskFamily::ChiralFavorable => chiral_target()?,
        TaskFamily::Mixed => (torsor_target()? + chiral_target()?) / 2.0,
        TaskFamily::Neutral => input
            .query
            .iter()
            .zip(input.key)
            .map(|(q, k)| (q - k) * (q - k))
            .sum(),
    };
    if !target.is_finite() {
        return Err(EvalError::PairedUncertaintyInvalid {
            reason: "non_finite",
        });
    }
    Ok(target)
}

fn shared_match(score: f64, target: f64) -> bool {
    score.is_finite()
        && target.is_finite()
        && (score - target).abs() <= 1e-12 * (1.0 + score.abs().max(target.abs()))
}

fn canonical_identity(
    split: DataSplit,
    family: TaskFamily,
    block: u64,
    case_id: u64,
    input: &MatchedInput,
) -> String {
    // Lossless canonical encoding avoids treating a noncryptographic hash as
    // collision-free. The target value itself never enters this input identity.
    let mut identity = format!(
        "{MATCHED_POPULATION_CONTRACT}|{MATCHED_REFERENCE_CONTRACT}|{}|{:?}|{block}|{case_id}|{}|{MATCHED_T6_CONTRACT}|{MATCHED_C6_CONTRACT}",
        split.as_str(),
        family,
        common_target_contract(family)
    );
    for value in input
        .query
        .iter()
        .chain(&input.key)
        .chain(&input.key_position)
        .chain(&input.query_position)
    {
        use core::fmt::Write as _;
        write!(&mut identity, "|{:016x}", value.to_bits()).expect("String formatting cannot fail");
    }
    identity
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::experimental::tdi25_eval::{
        MetricRegistry, summarize_paired_uncertainty_by_seed_block,
    };

    #[test]
    fn independent_targets_have_hand_calculated_values() {
        let input = MatchedInput {
            query: [1.0, 2.0, 3.0, 4.0, 5.0, 6.0],
            key: [2.0, 0.0, -1.0, 1.0, 3.0, 2.0],
            key_position: [1.0, 0.0, 0.0],
            query_position: [0.0, 1.0, 0.0],
        };
        assert_eq!(
            common_target(&input, TaskFamily::TorsorFavorable).unwrap(),
            51.0
        );
        assert_eq!(
            common_target(&input, TaskFamily::ChiralFavorable).unwrap(),
            41.0
        );
        assert_eq!(common_target(&input, TaskFamily::Mixed).unwrap(), 46.0);
        assert_eq!(common_target(&input, TaskFamily::Neutral).unwrap(), 50.0);
        assert_eq!(score_t6(&input).unwrap(), 51.0);
        assert_eq!(score_c6(&input).unwrap(), 41.0);
    }

    #[test]
    fn generation_is_deterministic_prefix_stable_and_domain_separated() {
        for family in super::super::REQUIRED_SYNTHESIS_FAMILIES {
            let a = MatchedPrimaryRun::evaluate(DataSplit::Development, *family, 0, 2).unwrap();
            let b = MatchedPrimaryRun::evaluate(DataSplit::Development, *family, 0, 64).unwrap();
            assert_eq!(a.inputs(), &b.inputs()[..2]);
            assert_eq!(a.t6_outcomes(), &b.t6_outcomes()[..2]);
            assert_eq!(a.c6_outcomes(), &b.c6_outcomes()[..2]);
            assert_eq!(
                a,
                MatchedPrimaryRun::evaluate(DataSplit::Development, *family, 0, 2).unwrap()
            );
            let validation =
                MatchedPrimaryRun::evaluate(DataSplit::Validation, *family, 0, 2).unwrap();
            assert_ne!(a.inputs(), validation.inputs());
            assert_ne!(
                a.t6_outcomes()[0].canonical_digest,
                validation.t6_outcomes()[0].canonical_digest
            );
            let render = format!("{:?}", a.inputs());
            for forbidden in ["target", "family", "split", "seed", "oracle", "matches"] {
                assert!(!render.contains(forbidden));
            }
        }
    }

    #[test]
    fn generation_matches_the_prospective_known_answer() {
        assert_eq!(MAX_CASES_PER_RUN, 64);
        assert_eq!(MAX_SEED_BLOCKS_PER_SYNTHESIS, 64);
        let input = generate_input(DataSplit::Development, TaskFamily::TorsorFavorable, 0, 0);
        assert_eq!(input.query, [-0.5, 0.0, -1.5, 0.5, 0.5, -0.5]);
        assert_eq!(input.key, [-0.5, 1.5, 1.0, 0.0, 1.0, 0.5]);
        assert_eq!(input.key_position, [1.0, -2.0, 0.5]);
        assert_eq!(input.query_position, [-2.0, 0.5, 0.0]);
    }

    #[test]
    fn invalid_budgets_fail_before_scoring() {
        for count in [0, 1, 65, u64::MAX] {
            assert_eq!(
                MatchedPrimaryRun::evaluate(DataSplit::Development, TaskFamily::Mixed, 0, count),
                Err(EvalError::InvalidBudget)
            );
        }
        for block in [64, u64::MAX] {
            assert_eq!(
                MatchedPrimaryRun::evaluate(DataSplit::Development, TaskFamily::Mixed, block, 2),
                Err(EvalError::InvalidBudget)
            );
        }
    }

    #[test]
    fn transport_and_mirror_use_unchanged_upstream_semantics() {
        let original = generate_input(DataSplit::Development, TaskFamily::Mixed, 1, 0);
        let (q, key, position) = torsor_carriers(&original).unwrap();
        let transported = key.transport(Vec3::zero()).unwrap();
        assert_eq!(
            torsor_arm_score(q, key, position).unwrap(),
            torsor_arm_score(q, transported, position).unwrap()
        );
        let (q, k) = chiral_carriers(&original).unwrap();
        assert_eq!(q.dot(k).unwrap(), q.mirror().dot(k.mirror()).unwrap());
        assert_eq!(
            q.chiral_pairing(k).unwrap(),
            -q.mirror().chiral_pairing(k.mirror()).unwrap()
        );
    }

    #[test]
    fn stale_common_targets_and_arm_provenance_fail_closed() {
        let run =
            MatchedPrimaryRun::evaluate(DataSplit::Development, TaskFamily::Mixed, 0, 2).unwrap();
        let registry = MetricRegistry::pinned();
        let call = |left: &[RevealedMatchOutcome], right: &[RevealedMatchOutcome]| {
            summarize_paired_uncertainty_by_seed_block(
                DataSplit::Development,
                TaskFamily::Mixed,
                0,
                left,
                right,
                &registry,
            )
        };
        assert_eq!(
            call(run.c6_outcomes(), run.t6_outcomes()),
            Err(EvalError::PairedUncertaintyInvalid {
                reason: "arm_mismatch"
            })
        );
        let mut left = run.t6_outcomes().to_vec();
        let mut right = run.c6_outcomes().to_vec();
        left[0].shared_target_contract = Some("stale");
        assert_eq!(
            call(&left, &right),
            Err(EvalError::PairedUncertaintyInvalid {
                reason: "outcome_integrity_mismatch"
            })
        );
        // Only a module test can alter this private snapshot. Even with an
        // internally consistent seal, unregistered targets must be refused.
        for outcome in left.iter_mut().chain(&mut right) {
            outcome.shared_target_contract = Some("stale");
            outcome.integrity.shared_target_contract = Some("stale");
        }
        assert_eq!(
            call(&left, &right),
            Err(EvalError::PairedUncertaintyInvalid {
                reason: "unregistered_common_target_contract"
            })
        );
    }
}
