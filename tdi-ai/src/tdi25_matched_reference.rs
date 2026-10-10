// Included as tdi25_eval::matched_reference so outcome minting stays inside
// the evaluator privacy boundary. See the prospectively committed protocol
// docs/TDI-25-MATCHED-REFERENCE-V1.md. Development/Validation only.

use super::{
    ComparisonArm, DataSplit, EvalError, MAX_CASES_PER_RUN, MAX_SEED_BLOCKS_PER_SYNTHESIS,
    RevealedMatchOutcome, TaskFamily, Tdi25Error,
};
use crate::experimental::tdi22_torsor::{
    Torsor3, TorsorError, Twist3, Vec3, direct_pairing, factorized_pairing,
};
use crate::experimental::tdi24_attention::MaskPolicy;
use crate::experimental::tdi24_chiral::{Chiral6, ChiralScoreWeights, enantiomorphic_scores};
use crate::experimental::tdi24_eval::{
    EvalError as Tdi24EvalError, LEARNED_BASIS_PROBE_COUNT, LEARNED_BASIS_TOLERANCE,
    LearnedBasisProbe, LearnedBasisProbeRole, PRECISION_BOUND_FACTOR, ParityShuffle,
    chiral_score_f32, learned_basis_probes, learned_basis_transform, validate_learned_basis_probe,
    validate_parity_shuffle,
};
use crate::experimental::tdi25_tasks::{
    PositionGeometryArm, SeedDomain, mix_registered_seed, position_geometry_point,
};
use crate::experimental::tdi25_torsor_chiral::{
    Generic6, chiral_arm_score, generic_arm_score, normalize_arm_row, torsor_arm_score,
    validate_source_contracts,
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

/// One matched-population case scored by the unchanged T6 reference and by
/// its torsor reduction-point ablation (TDI-25 slice 31).
///
/// The ablation keeps the identical six query, six key and six geometry
/// scalars, but drops the Varignon transport: the stored moment `M(P)` is
/// paired as if it were already reduced at the query point `Q`. The target and
/// both match bits are minted here, inside the evaluator privacy boundary.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ReductionPointAblationCase {
    pub family: TaskFamily,
    pub seed_block: u64,
    pub case_id: u64,
    /// Unchanged T6 factorized score, reused from the matched primary run.
    pub reference_score: f64,
    /// Untransported score `v.R + omega.M(P)`.
    pub ablated_score: f64,
    /// Removed transport term `omega.((P - Q) x R)`.
    pub transport_term: f64,
    /// Whether the stored reduction point equals the query point exactly.
    pub reduction_points_coincide: bool,
    pub reference_matches_target: bool,
    pub ablated_matches_target: bool,
}

/// Ablated T6 score `v.R + omega.M(P)`: the reduction point is ignored.
pub fn untransported_torsor_score(query: Twist3, key: Torsor3) -> Result<f64, EvalError> {
    let score = query.linear().dot(key.resultant()) + query.angular().dot(key.moment());
    if score.is_finite() {
        Ok(score)
    } else {
        Err(EvalError::Bridge(Tdi25Error::Torsor(
            TorsorError::NonFiniteScalar {
                field: "untransported_pairing",
            },
        )))
    }
}

/// Transport term removed by the ablation: `omega.((P - Q) x R)`, using the
/// TDI-22 Varignon convention `M(Q) = M(P) + (P - Q) x R`.
pub fn reduction_point_transport_term(
    query: Twist3,
    key: Torsor3,
    query_position: Vec3,
) -> Result<f64, EvalError> {
    let arm = key.reference().minus(query_position);
    let term = query.angular().dot(arm.cross(key.resultant()));
    if term.is_finite() {
        Ok(term)
    } else {
        Err(EvalError::Bridge(Tdi25Error::Torsor(
            TorsorError::NonFiniteScalar {
                field: "transport_term",
            },
        )))
    }
}

/// Score one matched block with T6 and its reduction-point ablation.
///
/// The block is generated by the unchanged v1 protocol; the reference side is
/// the matched primary T6 score and outcome, re-checked rather than trusted.
/// Any scoring, drift or numerical failure aborts the block fail-closed.
pub fn evaluate_reduction_point_ablation(
    split: DataSplit,
    family: TaskFamily,
    seed_block: u64,
    n_cases: u64,
) -> Result<Vec<ReductionPointAblationCase>, EvalError> {
    let run = MatchedPrimaryRun::evaluate(split, family, seed_block, n_cases)?;
    let drift = |reason| EvalError::ReductionPointAblationInvalid { reason };
    let mut cases = Vec::with_capacity(run.inputs().len());
    for (index, input) in run.inputs().iter().enumerate() {
        let target = common_target(input, family)?;
        let (query, key, position) = torsor_carriers(input)?;
        let reference_score = run.t6_scores()[index];
        if reference_score.to_bits() != score_t6(input)?.to_bits() {
            return Err(drift("reference_drift"));
        }
        let reference_matches_target = shared_match(reference_score, target);
        if reference_matches_target != run.t6_outcomes()[index].matches_oracle {
            return Err(drift("reference_outcome_drift"));
        }
        let ablated_score = untransported_torsor_score(query, key)?;
        cases.push(ReductionPointAblationCase {
            family,
            seed_block,
            case_id: index as u64,
            reference_score,
            ablated_score,
            transport_term: reduction_point_transport_term(query, key, position)?,
            reduction_points_coincide: input.key_position == input.query_position,
            reference_matches_target,
            ablated_matches_target: shared_match(ablated_score, target),
        });
    }
    Ok(cases)
}

/// One matched-population case whose T6 torsor bridge is computed both through
/// the factorized and through the direct TDI-22 pairing (TDI-25 slice 32).
///
/// Both forms consume the identical six query, six key and six geometry
/// scalars. The factorized score is the unchanged T6 reference
/// `(v + Q x omega).R + omega.C` with `C = M(P) + P x R`; the direct score is
/// `v.R + omega.M(Q)` with `M(Q) = M(P) + (P - Q) x R`. The target and both
/// match bits are minted here, inside the evaluator privacy boundary.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TorsorBridgeCase {
    pub family: TaskFamily,
    pub seed_block: u64,
    pub case_id: u64,
    /// Unchanged T6 factorized score, reused from the matched primary run.
    pub factorized_score: f64,
    /// Direct upstream pairing `v.R + omega.M(Q)` on the same carriers.
    pub direct_score: f64,
    /// Monitored residual `factorized_score - direct_score`.
    pub residual: f64,
    /// Whether both forms agree bit for bit.
    pub bit_identical: bool,
    pub factorized_matches_target: bool,
    pub direct_matches_target: bool,
}

/// Direct torsor bridge score `v.R + omega.M(Q)` through the upstream TDI-22
/// `direct_pairing`; no transport or pairing logic is copied.
pub fn direct_torsor_bridge_score(
    query: Twist3,
    key: Torsor3,
    query_position: Vec3,
) -> Result<f64, EvalError> {
    direct_pairing(query, key, query_position)
        .map_err(Tdi25Error::Torsor)
        .map_err(EvalError::Bridge)
}

/// Score one matched block through both torsor bridge forms.
///
/// The block is generated by the unchanged v1 protocol; the factorized side is
/// the matched primary T6 score and outcome, re-checked against the upstream
/// factorized pairing rather than trusted. Any scoring, drift or numerical
/// failure aborts the block fail-closed.
pub fn evaluate_torsor_bridge_equivalence(
    split: DataSplit,
    family: TaskFamily,
    seed_block: u64,
    n_cases: u64,
) -> Result<Vec<TorsorBridgeCase>, EvalError> {
    let run = MatchedPrimaryRun::evaluate(split, family, seed_block, n_cases)?;
    let drift = |reason| EvalError::TorsorBridgeEquivalenceInvalid { reason };
    let mut cases = Vec::with_capacity(run.inputs().len());
    for (index, input) in run.inputs().iter().enumerate() {
        let target = common_target(input, family)?;
        let (query, key, position) = torsor_carriers(input)?;
        let factorized_score = run.t6_scores()[index];
        let upstream_factorized = factorized_pairing(query, key, position)
            .map_err(Tdi25Error::Torsor)
            .map_err(EvalError::Bridge)?;
        if factorized_score.to_bits() != upstream_factorized.to_bits()
            || factorized_score.to_bits() != score_t6(input)?.to_bits()
        {
            return Err(drift("reference_drift"));
        }
        let factorized_matches_target = shared_match(factorized_score, target);
        if factorized_matches_target != run.t6_outcomes()[index].matches_oracle {
            return Err(drift("reference_outcome_drift"));
        }
        let direct_score = direct_torsor_bridge_score(query, key, position)?;
        cases.push(TorsorBridgeCase {
            family,
            seed_block,
            case_id: index as u64,
            factorized_score,
            direct_score,
            residual: factorized_score - direct_score,
            bit_identical: factorized_score.to_bits() == direct_score.to_bits(),
            factorized_matches_target,
            direct_matches_target: shared_match(direct_score, target),
        });
    }
    Ok(cases)
}

/// One matched-population case scored by the unchanged C6 reference and by
/// its chiral `gamma=0` ablation (TDI-25 slice 33).
///
/// The ablation keeps the identical six query and six key scalars and the
/// `alpha`/`beta` coefficients; only the parity-odd coefficient is zeroed.
/// The target and both match bits are minted here, inside the evaluator
/// privacy boundary.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ChiralGammaZeroAblationCase {
    pub family: TaskFamily,
    pub seed_block: u64,
    pub case_id: u64,
    /// Unchanged C6 score, reused from the matched primary run.
    pub reference_score: f64,
    /// Ablated score `alpha*s + beta*m` (parity-odd channel removed).
    pub ablated_score: f64,
    /// Unweighted parity-odd observable `chi = q^T J k`.
    pub parity_odd_channel: f64,
    /// Right/left enantiomorphic scores of the ablation coincide exactly.
    pub enantiomorphic_split_closed: bool,
    pub reference_matches_target: bool,
    pub ablated_matches_target: bool,
}

/// `gamma=0` ablation of `reference`: only the parity-odd coefficient changes.
#[must_use]
pub const fn chiral_gamma_zero_weights(reference: ChiralScoreWeights) -> ChiralScoreWeights {
    ChiralScoreWeights {
        alpha: reference.alpha,
        beta: reference.beta,
        gamma: 0.0,
    }
}

/// Ablated C6 score: the matched C6 weights with `gamma=0`, through the
/// unchanged upstream `chiral_arm_score`.
pub fn gamma_zero_chiral_score(query: Chiral6, key: Chiral6) -> Result<f64, EvalError> {
    chiral_arm_score(
        query,
        key,
        chiral_gamma_zero_weights(MATCHED_CHIRAL_WEIGHTS),
    )
    .map_err(EvalError::Bridge)
}

/// Removed parity-odd observable `chi = q^T J k` (unweighted).
pub fn chiral_parity_odd_channel(query: Chiral6, key: Chiral6) -> Result<f64, EvalError> {
    query
        .chiral_pairing(key)
        .map_err(Tdi25Error::Chiral)
        .map_err(EvalError::Bridge)
}

/// Score one matched block with C6 and its `gamma=0` ablation.
///
/// The block is generated by the unchanged v1 protocol; the reference side is
/// the matched primary C6 score and outcome, re-checked rather than trusted.
/// Any scoring, drift or numerical failure aborts the block fail-closed.
pub fn evaluate_chiral_gamma_zero_ablation(
    split: DataSplit,
    family: TaskFamily,
    seed_block: u64,
    n_cases: u64,
) -> Result<Vec<ChiralGammaZeroAblationCase>, EvalError> {
    let run = MatchedPrimaryRun::evaluate(split, family, seed_block, n_cases)?;
    let drift = |reason| EvalError::ChiralGammaZeroAblationInvalid { reason };
    let mut cases = Vec::with_capacity(run.inputs().len());
    for (index, input) in run.inputs().iter().enumerate() {
        let target = common_target(input, family)?;
        let (query, key) = chiral_carriers(input)?;
        let reference_score = run.c6_scores()[index];
        if reference_score.to_bits() != score_c6(input)?.to_bits() {
            return Err(drift("reference_drift"));
        }
        let reference_matches_target = shared_match(reference_score, target);
        if reference_matches_target != run.c6_outcomes()[index].matches_oracle {
            return Err(drift("reference_outcome_drift"));
        }
        let ablated_score = gamma_zero_chiral_score(query, key)?;
        let (right, left) = enantiomorphic_scores(
            query,
            key,
            chiral_gamma_zero_weights(MATCHED_CHIRAL_WEIGHTS),
        )
        .map_err(Tdi25Error::Chiral)
        .map_err(EvalError::Bridge)?;
        cases.push(ChiralGammaZeroAblationCase {
            family,
            seed_block,
            case_id: index as u64,
            reference_score,
            ablated_score,
            parity_odd_channel: chiral_parity_odd_channel(query, key)?,
            enantiomorphic_split_closed: right.to_bits() == left.to_bits()
                && right == ablated_score,
            reference_matches_target,
            ablated_matches_target: shared_match(ablated_score, target),
        });
    }
    Ok(cases)
}

/// One matched-population case scored by the unchanged C6 reference and by
/// the same reference on the parity-shuffled carrier (TDI-25 slice 34).
///
/// The control reuses the TDI-24 slice-34 carrier-slot permutation unchanged:
/// query and key are relabelled by one fixed permutation that mixes the
/// `H+`/`H-` sectors. Weights, capacity and the six query and six key values
/// are unchanged; only the parity semantics of the fixed `M`/`J` are lost. The
/// target and both match bits are minted here, inside the evaluator privacy
/// boundary, from the unshuffled case.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ChiralParityShuffleCase {
    pub family: TaskFamily,
    pub seed_block: u64,
    pub case_id: u64,
    /// Unchanged C6 score, reused from the matched primary run.
    pub reference_score: f64,
    /// C6 score with the matched weights on the parity-shuffled carrier.
    pub shuffled_score: f64,
    /// Parity-odd observable `chi = q^T J k` of the unshuffled pair.
    pub reference_parity_odd: f64,
    /// Parity-odd observable of the parity-shuffled pair.
    pub shuffled_parity_odd: f64,
    /// The six query and the six key values are the same multisets
    /// (bit-for-bit) before and after the shuffle.
    pub six_values_preserved: bool,
    /// The six coordinate products `q_i k_i` are the same multiset
    /// (bit-for-bit) before and after the shuffle.
    pub direct_products_preserved: bool,
    pub reference_matches_target: bool,
    pub shuffled_matches_target: bool,
}

/// Map an upstream TDI-24 parity-shuffle rejection onto the TDI-25 control.
fn upstream_parity_shuffle_error(error: Tdi24EvalError) -> EvalError {
    match error {
        Tdi24EvalError::ParityShuffleControlInvalid { reason } => {
            EvalError::ChiralParityShuffleControlInvalid { reason }
        }
        _ => EvalError::ChiralParityShuffleControlInvalid {
            reason: "upstream_shuffle_invalid",
        },
    }
}

fn sorted_value_bits(values: [f64; 6]) -> [u64; 6] {
    let mut bits = values.map(f64::to_bits);
    bits.sort_unstable();
    bits
}

fn sorted_coordinate_product_bits(query: [f64; 6], key: [f64; 6]) -> [u64; 6] {
    let mut bits = [0u64; 6];
    for (index, value) in bits.iter_mut().enumerate() {
        *value = (query[index] * key[index]).to_bits();
    }
    bits.sort_unstable();
    bits
}

/// Matched C6 score on the parity-shuffled carrier: the identical matched
/// weights through the unchanged upstream `chiral_arm_score`.
pub fn parity_shuffled_chiral_score(
    query: Chiral6,
    key: Chiral6,
    shuffle: &ParityShuffle,
) -> Result<f64, EvalError> {
    validate_parity_shuffle(shuffle).map_err(upstream_parity_shuffle_error)?;
    chiral_arm_score(
        shuffle.apply(query),
        shuffle.apply(key),
        MATCHED_CHIRAL_WEIGHTS,
    )
    .map_err(EvalError::Bridge)
}

/// Score one matched block with C6 and its parity-shuffle control.
///
/// The block is generated by the unchanged v1 protocol; the reference side is
/// the matched primary C6 score and outcome, re-checked rather than trusted.
/// Any scoring, drift or numerical failure aborts the block fail-closed.
pub fn evaluate_chiral_parity_shuffle_control(
    split: DataSplit,
    family: TaskFamily,
    seed_block: u64,
    n_cases: u64,
    shuffle: &ParityShuffle,
) -> Result<Vec<ChiralParityShuffleCase>, EvalError> {
    validate_parity_shuffle(shuffle).map_err(upstream_parity_shuffle_error)?;
    let run = MatchedPrimaryRun::evaluate(split, family, seed_block, n_cases)?;
    let drift = |reason| EvalError::ChiralParityShuffleControlInvalid { reason };
    let mut cases = Vec::with_capacity(run.inputs().len());
    for (index, input) in run.inputs().iter().enumerate() {
        let target = common_target(input, family)?;
        let (query, key) = chiral_carriers(input)?;
        let reference_score = run.c6_scores()[index];
        if reference_score.to_bits() != score_c6(input)?.to_bits() {
            return Err(drift("reference_drift"));
        }
        let reference_matches_target = shared_match(reference_score, target);
        if reference_matches_target != run.c6_outcomes()[index].matches_oracle {
            return Err(drift("reference_outcome_drift"));
        }
        let shuffled_query = shuffle.apply(query);
        let shuffled_key = shuffle.apply(key);
        let shuffled_score = parity_shuffled_chiral_score(query, key, shuffle)?;
        let six_values_preserved = sorted_value_bits(query.as_array())
            == sorted_value_bits(shuffled_query.as_array())
            && sorted_value_bits(key.as_array()) == sorted_value_bits(shuffled_key.as_array());
        let direct_products_preserved =
            sorted_coordinate_product_bits(query.as_array(), key.as_array())
                == sorted_coordinate_product_bits(
                    shuffled_query.as_array(),
                    shuffled_key.as_array(),
                );
        cases.push(ChiralParityShuffleCase {
            family,
            seed_block,
            case_id: index as u64,
            reference_score,
            shuffled_score,
            reference_parity_odd: chiral_parity_odd_channel(query, key)?,
            shuffled_parity_odd: chiral_parity_odd_channel(shuffled_query, shuffled_key)?,
            six_values_preserved,
            direct_products_preserved,
            reference_matches_target,
            shuffled_matches_target: shared_match(shuffled_score, target),
        });
    }
    Ok(cases)
}

/// One matched-population case scored by the unchanged T6 reference and by
/// the same reference on the structure-shuffled carrier (TDI-25 slice 35).
///
/// The control relabels the six query (twist `v | omega`) and six key
/// (torsor `R | M(P)`) slots by one fixed permutation that mixes the
/// linear/angular blocks, reusing the TDI-24 slice-34 shuffle unchanged. The
/// six values, both geometry points and the capacity are unchanged; only the
/// Varignon pairing semantics (which slot is a resultant, which is a moment)
/// are lost. The target and both match bits are minted here, inside the
/// evaluator privacy boundary, from the unshuffled case.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TorsorStructureShuffleCase {
    pub family: TaskFamily,
    pub seed_block: u64,
    pub case_id: u64,
    /// Unchanged T6 factorized score, reused from the matched primary run.
    pub reference_score: f64,
    /// T6 factorized score on the structure-shuffled carrier.
    pub shuffled_score: f64,
    /// Varignon transport term `omega.((P - Q) x R)` of the unshuffled pair.
    pub reference_transport_term: f64,
    /// Transport term computed on the structure-shuffled pair.
    pub shuffled_transport_term: f64,
    /// The six query and the six key values are the same multisets
    /// (bit-for-bit) before and after the shuffle.
    pub six_values_preserved: bool,
    /// The six coordinate products `q_i k_i` of the untransported pairing are
    /// the same multiset (bit-for-bit) before and after the shuffle.
    pub untransported_products_preserved: bool,
    pub reference_matches_target: bool,
    pub shuffled_matches_target: bool,
}

fn torsor_structure_shuffle_error(error: Tdi24EvalError) -> EvalError {
    match error {
        Tdi24EvalError::ParityShuffleControlInvalid { reason } => {
            EvalError::TorsorStructureShuffleControlInvalid { reason }
        }
        _ => EvalError::TorsorStructureShuffleControlInvalid {
            reason: "upstream_shuffle_invalid",
        },
    }
}

fn permute_six(values: [f64; 6], shuffle: &ParityShuffle) -> [f64; 6] {
    let mut permuted = [0.0; 6];
    for (slot, value) in permuted.iter_mut().enumerate() {
        *value = values[shuffle.permutation[slot]];
    }
    permuted
}

fn structure_shuffled_input(input: &MatchedInput, shuffle: &ParityShuffle) -> MatchedInput {
    MatchedInput {
        query: permute_six(input.query, shuffle),
        key: permute_six(input.key, shuffle),
        key_position: input.key_position,
        query_position: input.query_position,
    }
}

/// Matched T6 score on the structure-shuffled carrier: the unchanged upstream
/// `torsor_arm_score` on the relabelled twist and torsor with unchanged
/// reduction and query points.
pub fn structure_shuffled_torsor_score(
    input: &MatchedInput,
    shuffle: &ParityShuffle,
) -> Result<f64, EvalError> {
    validate_parity_shuffle(shuffle).map_err(torsor_structure_shuffle_error)?;
    score_t6(&structure_shuffled_input(input, shuffle))
}

/// Score one matched block with T6 and its torsor structure-shuffle control.
///
/// The block is generated by the unchanged v1 protocol; the reference side is
/// the matched primary T6 score and outcome, re-checked rather than trusted.
/// Any scoring, drift or numerical failure aborts the block fail-closed.
pub fn evaluate_torsor_structure_shuffle_control(
    split: DataSplit,
    family: TaskFamily,
    seed_block: u64,
    n_cases: u64,
    shuffle: &ParityShuffle,
) -> Result<Vec<TorsorStructureShuffleCase>, EvalError> {
    validate_parity_shuffle(shuffle).map_err(torsor_structure_shuffle_error)?;
    let run = MatchedPrimaryRun::evaluate(split, family, seed_block, n_cases)?;
    let drift = |reason| EvalError::TorsorStructureShuffleControlInvalid { reason };
    let mut cases = Vec::with_capacity(run.inputs().len());
    for (index, input) in run.inputs().iter().enumerate() {
        let target = common_target(input, family)?;
        let reference_score = run.t6_scores()[index];
        if reference_score.to_bits() != score_t6(input)?.to_bits() {
            return Err(drift("reference_drift"));
        }
        let reference_matches_target = shared_match(reference_score, target);
        if reference_matches_target != run.t6_outcomes()[index].matches_oracle {
            return Err(drift("reference_outcome_drift"));
        }
        let shuffled = structure_shuffled_input(input, shuffle);
        let shuffled_score = structure_shuffled_torsor_score(input, shuffle)?;
        let (query, key, position) = torsor_carriers(input)?;
        let (shuffled_query, shuffled_key, shuffled_position) = torsor_carriers(&shuffled)?;
        let six_values_preserved = sorted_value_bits(input.query)
            == sorted_value_bits(shuffled.query)
            && sorted_value_bits(input.key) == sorted_value_bits(shuffled.key);
        let untransported_products_preserved =
            sorted_coordinate_product_bits(input.query, input.key)
                == sorted_coordinate_product_bits(shuffled.query, shuffled.key);
        cases.push(TorsorStructureShuffleCase {
            family,
            seed_block,
            case_id: index as u64,
            reference_score,
            shuffled_score,
            reference_transport_term: reduction_point_transport_term(query, key, position)?,
            shuffled_transport_term: reduction_point_transport_term(
                shuffled_query,
                shuffled_key,
                shuffled_position,
            )?,
            six_values_preserved,
            untransported_products_preserved,
            reference_matches_target,
            shuffled_matches_target: shared_match(shuffled_score, target),
        });
    }
    Ok(cases)
}

/// One matched-population case under one deterministic orthogonal basis
/// rotation (TDI-25 slice 36).
///
/// The rotation is one of the four declared TDI-24 slice-36 probes
/// (`tdi24-learned-basis-prototype-v1`, consumed unchanged) applied to the six
/// query and six key scalars. G6 is scored on the rotated pair; C6 is scored
/// on the same rotated pair for contrast. The target and match bits are minted
/// here, inside the evaluator privacy boundary, from the unrotated case.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct G6OrthogonalBasisCase {
    pub family: TaskFamily,
    pub seed_block: u64,
    pub case_id: u64,
    pub probe_index: usize,
    /// G6 score `q.k` of the unrotated pair.
    pub g6_reference_score: f64,
    /// G6 score of the rotated pair.
    pub g6_rotated_score: f64,
    /// Unchanged matched C6 score, reused from the matched primary run.
    pub c6_reference_score: f64,
    /// Matched C6 score of the rotated pair.
    pub c6_rotated_score: f64,
    pub g6_reference_matches_target: bool,
    pub g6_rotated_matches_target: bool,
}

fn g6_basis_invalid(reason: &'static str) -> EvalError {
    EvalError::G6OrthogonalBasisControlInvalid { reason }
}

fn upstream_probe_error(error: Tdi24EvalError) -> EvalError {
    match error {
        Tdi24EvalError::LearnedBasisPrototypeInvalid { reason } => g6_basis_invalid(reason),
        _ => g6_basis_invalid("upstream_probe_invalid"),
    }
}

fn rotate_six(transform: &[[f64; 6]; 6], values: [f64; 6]) -> Result<[f64; 6], EvalError> {
    let mut rotated = [0.0; 6];
    for (row, value) in rotated.iter_mut().enumerate() {
        let mut sum = transform[row][0] * values[0];
        for column in 1..6 {
            sum += transform[row][column] * values[column];
        }
        if !sum.is_finite() {
            return Err(g6_basis_invalid("non_finite_rotation"));
        }
        *value = sum;
    }
    Ok(rotated)
}

fn g6_score(query: [f64; 6], key: [f64; 6]) -> Result<f64, EvalError> {
    let query = Generic6::new(query).map_err(EvalError::Bridge)?;
    let key = Generic6::new(key).map_err(EvalError::Bridge)?;
    generic_arm_score(query, key).map_err(EvalError::Bridge)
}

/// G6 and C6 scores of one matched input in the basis of one probe.
/// The identity probe scores the untouched carrier.
pub fn rotated_generic_and_chiral_scores(
    input: &MatchedInput,
    probe: &LearnedBasisProbe,
) -> Result<(f64, f64), EvalError> {
    validate_learned_basis_probe(probe).map_err(upstream_probe_error)?;
    let (query, key) = if probe.role == LearnedBasisProbeRole::Identity {
        (input.query, input.key)
    } else {
        let transform = learned_basis_transform(probe).map_err(upstream_probe_error)?;
        (
            rotate_six(&transform, input.query)?,
            rotate_six(&transform, input.key)?,
        )
    };
    let rotated = MatchedInput {
        query,
        key,
        key_position: input.key_position,
        query_position: input.query_position,
    };
    Ok((g6_score(query, key)?, score_c6(&rotated)?))
}

/// True when two G6 scores agree within the upstream probe tolerance.
#[must_use]
pub fn g6_rotation_invariant(reference: f64, rotated: f64) -> bool {
    (reference - rotated).abs() <= LEARNED_BASIS_TOLERANCE * reference.abs().max(1.0)
}

/// Score one matched block under every declared orthogonal probe.
///
/// Case-major, probe-minor order. Any scoring, drift or invariance failure
/// aborts the block fail-closed.
pub fn evaluate_g6_orthogonal_basis_control(
    split: DataSplit,
    family: TaskFamily,
    seed_block: u64,
    n_cases: u64,
) -> Result<Vec<G6OrthogonalBasisCase>, EvalError> {
    let probes = learned_basis_probes();
    let run = MatchedPrimaryRun::evaluate(split, family, seed_block, n_cases)?;
    let mut cases = Vec::with_capacity(run.inputs().len() * LEARNED_BASIS_PROBE_COUNT);
    for (index, input) in run.inputs().iter().enumerate() {
        let target = common_target(input, family)?;
        let c6_reference_score = run.c6_scores()[index];
        if c6_reference_score.to_bits() != score_c6(input)?.to_bits() {
            return Err(g6_basis_invalid("reference_drift"));
        }
        let g6_reference_score = g6_score(input.query, input.key)?;
        for probe in &probes {
            let (g6_rotated_score, c6_rotated_score) =
                rotated_generic_and_chiral_scores(input, probe)?;
            if probe.role == LearnedBasisProbeRole::Identity
                && (g6_rotated_score.to_bits() != g6_reference_score.to_bits()
                    || c6_rotated_score.to_bits() != c6_reference_score.to_bits())
            {
                return Err(g6_basis_invalid("identity_probe_drift"));
            }
            if !g6_rotation_invariant(g6_reference_score, g6_rotated_score) {
                return Err(g6_basis_invalid("g6_rotation_invariance_drift"));
            }
            cases.push(G6OrthogonalBasisCase {
                family,
                seed_block,
                case_id: index as u64,
                probe_index: probe.index,
                g6_reference_score,
                g6_rotated_score,
                c6_reference_score,
                c6_rotated_score,
                g6_reference_matches_target: shared_match(g6_reference_score, target),
                g6_rotated_matches_target: shared_match(g6_rotated_score, target),
            });
        }
    }
    Ok(cases)
}

/// Geometry arms of the position-geometry ablation (TDI-25 slice 37), in
/// fixed order: the generated matched geometry first, then every internal arm
/// of the frozen `tdi25-position-geometry-arm-v1` registry. `External` needs
/// caller-supplied coordinates and is excluded, recorded as a degeneracy.
pub const POSITION_GEOMETRY_ABLATION_ARMS: [Option<PositionGeometryArm>; 4] = [
    None,
    Some(PositionGeometryArm::Linear),
    Some(PositionGeometryArm::Helical),
    Some(PositionGeometryArm::Learned),
];

/// One matched-population case scored by T6 under one geometry arm.
///
/// The six query and six key scalars are unchanged; only the reduction point
/// `P` and the query point `Q` come from the arm (`P` at registry index
/// `2 * case_id`, `Q` at `2 * case_id + 1`). The target and match bits are
/// minted here, inside the evaluator privacy boundary, from the generated
/// case.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PositionGeometryCase {
    pub family: TaskFamily,
    pub seed_block: u64,
    pub case_id: u64,
    /// Index into [`POSITION_GEOMETRY_ABLATION_ARMS`].
    pub arm_index: usize,
    pub key_position: [f64; 3],
    pub query_position: [f64; 3],
    pub score: f64,
    /// Varignon transport term `omega.((P - Q) x R)` under this geometry.
    pub transport_term: f64,
    pub matches_target: bool,
}

fn geometry_error(_: crate::experimental::tdi25_tasks::Tdi25TaskError) -> EvalError {
    EvalError::PositionGeometryAblationInvalid {
        reason: "geometry_generation_failed",
    }
}

/// Positions `(P, Q)` of one case under one ablation arm.
pub fn position_geometry_ablation_points(
    input: &MatchedInput,
    arm_index: usize,
    case_id: u64,
) -> Result<([f64; 3], [f64; 3]), EvalError> {
    match POSITION_GEOMETRY_ABLATION_ARMS.get(arm_index) {
        None => Err(EvalError::PositionGeometryAblationInvalid {
            reason: "arm_not_registered",
        }),
        Some(None) => Ok((input.key_position, input.query_position)),
        Some(Some(arm)) => {
            let key = position_geometry_point(*arm, 2 * case_id, None)
                .map_err(geometry_error)?
                .point;
            let query = position_geometry_point(*arm, 2 * case_id + 1, None)
                .map_err(geometry_error)?
                .point;
            Ok(([key.x, key.y, key.z], [query.x, query.y, query.z]))
        }
    }
}

/// Score one matched block with T6 under every geometry arm.
///
/// Case-major, arm-minor order. The matched arm must reproduce the matched
/// primary T6 score bit for bit. Any failure aborts fail-closed.
pub fn evaluate_position_geometry_ablation(
    split: DataSplit,
    family: TaskFamily,
    seed_block: u64,
    n_cases: u64,
) -> Result<Vec<PositionGeometryCase>, EvalError> {
    let run = MatchedPrimaryRun::evaluate(split, family, seed_block, n_cases)?;
    let mut cases = Vec::with_capacity(run.inputs().len() * POSITION_GEOMETRY_ABLATION_ARMS.len());
    for (index, input) in run.inputs().iter().enumerate() {
        let target = common_target(input, family)?;
        let reference = run.t6_scores()[index];
        for arm_index in 0..POSITION_GEOMETRY_ABLATION_ARMS.len() {
            let (key_position, query_position) =
                position_geometry_ablation_points(input, arm_index, index as u64)?;
            let placed = MatchedInput {
                query: input.query,
                key: input.key,
                key_position,
                query_position,
            };
            let score = score_t6(&placed)?;
            if arm_index == 0
                && (score.to_bits() != reference.to_bits()
                    || shared_match(score, target) != run.t6_outcomes()[index].matches_oracle)
            {
                return Err(EvalError::PositionGeometryAblationInvalid {
                    reason: "matched_reference_drift",
                });
            }
            let (query, key, position) = torsor_carriers(&placed)?;
            cases.push(PositionGeometryCase {
                family,
                seed_block,
                case_id: index as u64,
                arm_index,
                key_position,
                query_position,
                score,
                transport_term: reduction_point_transport_term(query, key, position)?,
                matches_target: shared_match(score, target),
            });
        }
    }
    Ok(cases)
}

/// Preregistered sequence lengths of the sequence-length scaling study (TDI-25
/// slice 38). Every length is reported; none is selected.
pub const SEQUENCE_SCALING_LENGTHS: [usize; 3] = [2, 4, 8];
/// Shared TDI-24 mask policies, both consumed through `normalize_arm_row`.
pub const SEQUENCE_SCALING_POLICIES: [MaskPolicy; 2] = [MaskPolicy::Full, MaskPolicy::Causal];
/// Matched primary arms, scored on identical windows.
pub const SEQUENCE_SCALING_ARMS: [ComparisonArm; 2] = [ComparisonArm::T6, ComparisonArm::C6];
/// Row-sum tolerance of every normalized row.
pub const SEQUENCE_SCALING_ROW_SUM_TOLERANCE: f64 = 1e-12;

/// Exact per-block resource accounting for one (length, policy, arm) cell.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SequenceScalingCell {
    pub family: TaskFamily,
    pub seed_block: u64,
    pub length: usize,
    pub policy: MaskPolicy,
    pub arm: ComparisonArm,
    /// Non-overlapping windows of `length` consecutive matched cases.
    pub windows: u64,
    /// Trailing cases that do not fill a window (counted, never scored).
    pub unwindowed_cases: u64,
    /// `windows * length^2` pairwise score evaluations.
    pub score_evaluations: u64,
    /// `windows * length` shared normalizer calls.
    pub normalizer_calls: u64,
    /// Entries forced to exactly zero by the mask policy.
    pub masked_entries: u64,
    /// Rows whose first maximal probability sits on the diagonal.
    pub self_retrieval_rows: u64,
    pub max_row_sum_error: f64,
}

const fn sequence_scaling_invalid(reason: &'static str) -> EvalError {
    EvalError::SequenceLengthScalingInvalid { reason }
}

fn cross_input(query: &MatchedInput, key: &MatchedInput) -> MatchedInput {
    MatchedInput {
        query: query.query,
        key: key.key,
        key_position: key.key_position,
        query_position: query.query_position,
    }
}

fn arm_score(arm: ComparisonArm, input: &MatchedInput) -> Result<f64, EvalError> {
    match arm {
        ComparisonArm::T6 => score_t6(input),
        ComparisonArm::C6 => score_c6(input),
        _ => Err(sequence_scaling_invalid("arm_not_registered")),
    }
}

/// Score one matched block on non-overlapping windows of every registered
/// length, under every mask policy, for both matched arms.
///
/// Row `i` of a window holds the arm score of query `i` against every key of
/// the window; the diagonal must reproduce the matched primary score bit for
/// bit. Order: length-major, then policy, then arm.
pub fn evaluate_sequence_length_scaling(
    split: DataSplit,
    family: TaskFamily,
    seed_block: u64,
    n_cases: u64,
) -> Result<Vec<SequenceScalingCell>, EvalError> {
    let run = MatchedPrimaryRun::evaluate(split, family, seed_block, n_cases)?;
    let inputs = run.inputs();
    let mut cells = Vec::new();
    for length in SEQUENCE_SCALING_LENGTHS {
        let windows = inputs.len() / length;
        for policy in SEQUENCE_SCALING_POLICIES {
            for arm in SEQUENCE_SCALING_ARMS {
                let reference = match arm {
                    ComparisonArm::T6 => run.t6_scores(),
                    _ => run.c6_scores(),
                };
                let mut cell = SequenceScalingCell {
                    family,
                    seed_block,
                    length,
                    policy,
                    arm,
                    windows: windows as u64,
                    unwindowed_cases: (inputs.len() % length) as u64,
                    score_evaluations: 0,
                    normalizer_calls: 0,
                    masked_entries: 0,
                    self_retrieval_rows: 0,
                    max_row_sum_error: 0.0,
                };
                for window in 0..windows {
                    let base = window * length;
                    for i in 0..length {
                        let mut logits = Vec::with_capacity(length);
                        for j in 0..length {
                            let score =
                                arm_score(arm, &cross_input(&inputs[base + i], &inputs[base + j]))?;
                            if i == j && score.to_bits() != reference[base + i].to_bits() {
                                return Err(sequence_scaling_invalid("diagonal_reference_drift"));
                            }
                            logits.push(score);
                        }
                        cell.score_evaluations += length as u64;
                        let row = normalize_arm_row(arm, &logits, policy, i)
                            .map_err(|_| sequence_scaling_invalid("normalizer_failure"))?;
                        cell.normalizer_calls += 1;
                        let p = row.probabilities;
                        for (j, value) in p.iter().enumerate() {
                            let masked = matches!(policy, MaskPolicy::Causal) && j > i;
                            if masked {
                                if *value != 0.0 {
                                    return Err(sequence_scaling_invalid("mask_drift"));
                                }
                                cell.masked_entries += 1;
                            }
                        }
                        let sum: f64 = p.iter().sum();
                        cell.max_row_sum_error = cell.max_row_sum_error.max((sum - 1.0).abs());
                        let mut best = 0;
                        for j in 1..length {
                            if p[j] > p[best] {
                                best = j;
                            }
                        }
                        if best == i {
                            cell.self_retrieval_rows += 1;
                        }
                    }
                }
                cells.push(cell);
            }
        }
    }
    Ok(cells)
}

/// Declared perturbation families of the input/noise robustness study (TDI-25
/// slice 42) on the 18 shared input scalars.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputNoiseFamily {
    /// All 18 scalars (query, key, key position, query position).
    Isotropic,
    /// The 12 query/key carrier scalars only.
    CarrierOnly,
    /// The 6 position scalars only.
    PositionOnly,
}

/// Declared perturbation families, all reported.
pub const INPUT_NOISE_FAMILIES: [InputNoiseFamily; 3] = [
    InputNoiseFamily::Isotropic,
    InputNoiseFamily::CarrierOnly,
    InputNoiseFamily::PositionOnly,
];
/// Declared absolute amplitudes, all reported.
pub const INPUT_NOISE_AMPLITUDES: [f64; 3] = [1e-3, 1e-2, 1e-1];

/// Per (block, noise family, amplitude, arm) match accounting.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InputNoiseCell {
    pub family: TaskFamily,
    pub seed_block: u64,
    pub noise: InputNoiseFamily,
    pub amplitude: f64,
    pub arm: ComparisonArm,
    pub n_cases: u64,
    /// Matches of the matched primary on clean inputs.
    pub clean_matches: u64,
    /// Matches on the perturbed input against the target recomputed from
    /// that same perturbed input inside the evaluator.
    pub noisy_matches: u64,
    /// Cases whose match bit differs between clean and perturbed inputs.
    pub flips: u64,
    /// Label-free: maximum absolute change of the arm score.
    pub max_abs_score_change: f64,
}

fn noise_mix(mut state: u64) -> u64 {
    state = state.wrapping_add(0x9e37_79b9_7f4a_7c15);
    state = (state ^ (state >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    state = (state ^ (state >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    state ^ (state >> 31)
}

/// Perturb one matched input. The draw depends on the seed, the case and the
/// scalar index only, never on the arm, so both arms see identical inputs.
pub fn perturb_matched_input(
    input: &MatchedInput,
    noise: InputNoiseFamily,
    amplitude: f64,
    seed: u64,
    case_key: u64,
) -> Result<MatchedInput, EvalError> {
    if !INPUT_NOISE_AMPLITUDES.contains(&amplitude) {
        return Err(EvalError::InputNoiseRobustnessInvalid {
            reason: "amplitude_not_registered",
        });
    }
    let mut out = input.clone();
    let draw = |slot: u64| {
        let mixed = noise_mix(seed ^ noise_mix(case_key ^ noise_mix(slot)));
        let unit = (mixed >> 11) as f64 / (1u64 << 53) as f64;
        amplitude * (2.0 * unit - 1.0)
    };
    let carriers = matches!(
        noise,
        InputNoiseFamily::Isotropic | InputNoiseFamily::CarrierOnly
    );
    let positions = matches!(
        noise,
        InputNoiseFamily::Isotropic | InputNoiseFamily::PositionOnly
    );
    for index in 0..6 {
        if carriers {
            out.query[index] += draw(index as u64);
            out.key[index] += draw(6 + index as u64);
        }
    }
    for index in 0..3 {
        if positions {
            out.key_position[index] += draw(12 + index as u64);
            out.query_position[index] += draw(15 + index as u64);
        }
    }
    Ok(out)
}

/// Score one matched block under every declared perturbation family and
/// amplitude for T6 and C6. Order: noise family, amplitude, arm.
pub fn evaluate_input_noise_robustness(
    split: DataSplit,
    family: TaskFamily,
    seed_block: u64,
    n_cases: u64,
    seed: u64,
) -> Result<Vec<InputNoiseCell>, EvalError> {
    let run = MatchedPrimaryRun::evaluate(split, family, seed_block, n_cases)?;
    let mut cells = Vec::new();
    for noise in INPUT_NOISE_FAMILIES {
        for amplitude in INPUT_NOISE_AMPLITUDES {
            for arm in SEQUENCE_SCALING_ARMS {
                let (clean, clean_scores) = match arm {
                    ComparisonArm::T6 => (run.t6_outcomes(), run.t6_scores()),
                    _ => (run.c6_outcomes(), run.c6_scores()),
                };
                let mut cell = InputNoiseCell {
                    family,
                    seed_block,
                    noise,
                    amplitude,
                    arm,
                    n_cases: 0,
                    clean_matches: 0,
                    noisy_matches: 0,
                    flips: 0,
                    max_abs_score_change: 0.0,
                };
                for (index, input) in run.inputs().iter().enumerate() {
                    // The clean arm score must reproduce the matched primary.
                    let clean_score = arm_score(arm, input)?;
                    if clean_score.to_bits() != clean_scores[index].to_bits() {
                        return Err(EvalError::InputNoiseRobustnessInvalid {
                            reason: "clean_reference_drift",
                        });
                    }
                    let case_key = (seed_block << 32) | index as u64;
                    let noisy = perturb_matched_input(input, noise, amplitude, seed, case_key)?;
                    let target = common_target(&noisy, family)?;
                    let noisy_score = arm_score(arm, &noisy)?;
                    let matched = shared_match(noisy_score, target);
                    let clean_match = clean[index].matches_oracle;
                    cell.n_cases += 1;
                    cell.clean_matches += u64::from(clean_match);
                    cell.noisy_matches += u64::from(matched);
                    cell.flips += u64::from(matched != clean_match);
                    cell.max_abs_score_change = cell
                        .max_abs_score_change
                        .max((noisy_score - clean_score).abs());
                }
                cells.push(cell);
            }
        }
    }
    Ok(cells)
}

/// Declared torsor-relevant translation/origin transformations of the
/// translation/origin stress suite (TDI-25 slice 43). Each leaves the physical
/// twist/torsor pairing exactly invariant in real arithmetic.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OriginStressTransform {
    /// Rigid frame translation: both reduction points move by `d`; the twist
    /// `(v, omega)` at `Q` and the torsor `(R, M(P))` at `P` are unchanged.
    OriginShift,
    /// The key torsor is re-reduced at `P + d` through the upstream
    /// `Torsor3::transport` (`M(P + d) = M(P) - d x R`); `Q` is unchanged.
    KeyReduction,
    /// The query twist is re-reduced at `Q + d` (`v' = v + omega x d`); the
    /// key is unchanged.
    QueryReduction,
}

/// Declared transformations, all reported.
pub const ORIGIN_STRESS_TRANSFORMS: [OriginStressTransform; 3] = [
    OriginStressTransform::OriginShift,
    OriginStressTransform::KeyReduction,
    OriginStressTransform::QueryReduction,
];
/// Declared offset magnitudes `|d|`, all reported (none selected).
pub const ORIGIN_STRESS_OFFSETS: [f64; 3] = [1.0, 1e3, 1e6];

/// Per (block, transformation, offset, arm) match and invariance accounting.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OriginStressCell {
    pub family: TaskFamily,
    pub seed_block: u64,
    pub transform: OriginStressTransform,
    pub offset: f64,
    pub arm: ComparisonArm,
    pub n_cases: u64,
    /// Matches of the matched primary on clean inputs.
    pub clean_matches: u64,
    /// Matches on the transformed input against the common target
    /// recomputed from that same transformed input inside the evaluator.
    pub stressed_matches: u64,
    /// Cases whose match bit differs between clean and transformed inputs.
    pub flips: u64,
    /// Label-free: maximum absolute change of the arm score.
    pub max_abs_score_change: f64,
    /// Maximum absolute change of the recomputed common target.
    pub max_abs_target_change: f64,
}

/// Deterministic direction of the offset for one case: a unit vector drawn
/// from the contract seed and the case key only, never from an arm or score.
pub fn origin_stress_direction(seed: u64, case_key: u64) -> Result<[f64; 3], EvalError> {
    let mut direction = [0.0; 3];
    for (slot, entry) in direction.iter_mut().enumerate() {
        let mixed = noise_mix(seed ^ noise_mix(case_key ^ noise_mix(slot as u64)));
        let unit = (mixed >> 11) as f64 / (1u64 << 53) as f64;
        // Magnitudes in [0.5, 1]: never a degenerate direction.
        let signed = 2.0 * unit - 1.0;
        *entry = (0.5 + 0.5 * signed.abs()).copysign(signed);
    }
    let norm = direction
        .iter()
        .map(|value| value * value)
        .sum::<f64>()
        .sqrt();
    if !norm.is_finite() || norm <= 0.0 {
        return Err(EvalError::TranslationOriginStressInvalid {
            reason: "degenerate_direction",
        });
    }
    Ok(direction.map(|value| value / norm))
}

/// Apply one declared transformation with offset magnitude `offset`.
pub fn stress_matched_input(
    input: &MatchedInput,
    transform: OriginStressTransform,
    offset: f64,
    seed: u64,
    case_key: u64,
) -> Result<MatchedInput, EvalError> {
    if !ORIGIN_STRESS_OFFSETS.contains(&offset) {
        return Err(EvalError::TranslationOriginStressInvalid {
            reason: "offset_not_registered",
        });
    }
    let direction = origin_stress_direction(seed, case_key)?;
    let shift = direction.map(|value| value * offset);
    let mut out = input.clone();
    match transform {
        OriginStressTransform::OriginShift => {
            for ((key_point, query_point), delta) in out
                .key_position
                .iter_mut()
                .zip(out.query_position.iter_mut())
                .zip(shift)
            {
                *key_point += delta;
                *query_point += delta;
            }
        }
        OriginStressTransform::KeyReduction => {
            let (_, key, _) = torsor_carriers(input)?;
            let target = vec3([
                input.key_position[0] + shift[0],
                input.key_position[1] + shift[1],
                input.key_position[2] + shift[2],
            ])?;
            let moved = key
                .transport(target)
                .map_err(Tdi25Error::Torsor)
                .map_err(EvalError::Bridge)?;
            let moment = moved.moment();
            out.key[3] = moment.x;
            out.key[4] = moment.y;
            out.key[5] = moment.z;
            out.key_position = [target.x, target.y, target.z];
        }
        OriginStressTransform::QueryReduction => {
            let omega = vec3([input.query[3], input.query[4], input.query[5]])?;
            let transported = omega.cross(vec3(shift)?);
            out.query[0] += transported.x;
            out.query[1] += transported.y;
            out.query[2] += transported.z;
            for (query_point, delta) in out.query_position.iter_mut().zip(shift) {
                *query_point += delta;
            }
        }
    }
    for value in out
        .query
        .iter()
        .chain(&out.key)
        .chain(&out.key_position)
        .chain(&out.query_position)
    {
        if !value.is_finite() {
            return Err(EvalError::TranslationOriginStressInvalid {
                reason: "non_finite_transform",
            });
        }
    }
    Ok(out)
}

/// Score one matched block under every declared transformation and offset for
/// T6 and C6. Order: transformation, offset, arm.
pub fn evaluate_translation_origin_stress(
    split: DataSplit,
    family: TaskFamily,
    seed_block: u64,
    n_cases: u64,
    seed: u64,
) -> Result<Vec<OriginStressCell>, EvalError> {
    let run = MatchedPrimaryRun::evaluate(split, family, seed_block, n_cases)?;
    let mut cells = Vec::new();
    for transform in ORIGIN_STRESS_TRANSFORMS {
        for offset in ORIGIN_STRESS_OFFSETS {
            for arm in SEQUENCE_SCALING_ARMS {
                let (clean, clean_scores) = match arm {
                    ComparisonArm::T6 => (run.t6_outcomes(), run.t6_scores()),
                    _ => (run.c6_outcomes(), run.c6_scores()),
                };
                let mut cell = OriginStressCell {
                    family,
                    seed_block,
                    transform,
                    offset,
                    arm,
                    n_cases: 0,
                    clean_matches: 0,
                    stressed_matches: 0,
                    flips: 0,
                    max_abs_score_change: 0.0,
                    max_abs_target_change: 0.0,
                };
                for (index, input) in run.inputs().iter().enumerate() {
                    let clean_score = arm_score(arm, input)?;
                    if clean_score.to_bits() != clean_scores[index].to_bits() {
                        return Err(EvalError::TranslationOriginStressInvalid {
                            reason: "clean_reference_drift",
                        });
                    }
                    let case_key = (seed_block << 32) | index as u64;
                    let stressed = stress_matched_input(input, transform, offset, seed, case_key)?;
                    let clean_target = common_target(input, family)?;
                    let target = common_target(&stressed, family)?;
                    let stressed_score = arm_score(arm, &stressed)?;
                    let matched = shared_match(stressed_score, target);
                    let clean_match = clean[index].matches_oracle;
                    cell.n_cases += 1;
                    cell.clean_matches += u64::from(clean_match);
                    cell.stressed_matches += u64::from(matched);
                    cell.flips += u64::from(matched != clean_match);
                    cell.max_abs_score_change = cell
                        .max_abs_score_change
                        .max((stressed_score - clean_score).abs());
                    cell.max_abs_target_change = cell
                        .max_abs_target_change
                        .max((target - clean_target).abs());
                }
                cells.push(cell);
            }
        }
    }
    Ok(cells)
}

/// Declared chiral-relevant mirror/parity transformations of the mirror/parity
/// stress suite (TDI-25 slice 44), acting on the two six-carriers with the
/// upstream TDI-24 involutions; reduction points are unchanged.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MirrorStressTransform {
    /// `(q, k) -> (Mq, Mk)`: the direct pairing is invariant and the
    /// parity-odd pairing changes sign.
    SimultaneousMirror,
    /// `(q, k) -> (Jq, Jk)`: both the direct and the parity-odd pairings are
    /// invariant in real arithmetic (`J` is orthogonal and commutes with `J`).
    ComplexStructure,
    /// `(q, k) -> (q, Mk)`: one-sided reflection of the key only; neither
    /// pairing is preserved in general.
    KeyMirror,
}

/// Declared transformations, all reported (none selected).
pub const MIRROR_STRESS_TRANSFORMS: [MirrorStressTransform; 3] = [
    MirrorStressTransform::SimultaneousMirror,
    MirrorStressTransform::ComplexStructure,
    MirrorStressTransform::KeyMirror,
];

/// Per (block, transformation, arm) match and invariance accounting.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MirrorStressCell {
    pub family: TaskFamily,
    pub seed_block: u64,
    pub transform: MirrorStressTransform,
    pub arm: ComparisonArm,
    pub n_cases: u64,
    /// Matches of the matched primary on clean inputs.
    pub clean_matches: u64,
    /// Matches on the transformed input against the common target
    /// recomputed from that same transformed input inside the evaluator.
    pub stressed_matches: u64,
    /// Cases whose match bit differs between clean and transformed inputs.
    pub flips: u64,
    /// Label-free: maximum absolute change of the arm score.
    pub max_abs_score_change: f64,
    /// Maximum absolute change of the recomputed common target.
    pub max_abs_target_change: f64,
}

/// Apply one declared mirror/parity transformation to a matched input. The
/// transformation depends on the declared transform only, never on an arm,
/// score, target or label.
pub fn mirror_stress_matched_input(
    input: &MatchedInput,
    transform: MirrorStressTransform,
) -> Result<MatchedInput, EvalError> {
    let (query, key) = chiral_carriers(input)?;
    let (query, key) = match transform {
        MirrorStressTransform::SimultaneousMirror => (query.mirror(), key.mirror()),
        MirrorStressTransform::ComplexStructure => {
            (query.complex_structure(), key.complex_structure())
        }
        MirrorStressTransform::KeyMirror => (query, key.mirror()),
    };
    let mut out = input.clone();
    out.query = query.as_array();
    out.key = key.as_array();
    Ok(out)
}

/// Score one matched block under every declared mirror/parity transformation
/// for T6 and C6. Order: transformation, arm.
pub fn evaluate_mirror_parity_stress(
    split: DataSplit,
    family: TaskFamily,
    seed_block: u64,
    n_cases: u64,
) -> Result<Vec<MirrorStressCell>, EvalError> {
    let run = MatchedPrimaryRun::evaluate(split, family, seed_block, n_cases)?;
    let mut cells = Vec::new();
    for transform in MIRROR_STRESS_TRANSFORMS {
        for arm in SEQUENCE_SCALING_ARMS {
            let (clean, clean_scores) = match arm {
                ComparisonArm::T6 => (run.t6_outcomes(), run.t6_scores()),
                _ => (run.c6_outcomes(), run.c6_scores()),
            };
            let mut cell = MirrorStressCell {
                family,
                seed_block,
                transform,
                arm,
                n_cases: 0,
                clean_matches: 0,
                stressed_matches: 0,
                flips: 0,
                max_abs_score_change: 0.0,
                max_abs_target_change: 0.0,
            };
            for (index, input) in run.inputs().iter().enumerate() {
                let clean_score = arm_score(arm, input)?;
                if clean_score.to_bits() != clean_scores[index].to_bits() {
                    return Err(EvalError::MirrorParityStressInvalid {
                        reason: "clean_reference_drift",
                    });
                }
                let stressed = mirror_stress_matched_input(input, transform)?;
                let clean_target = common_target(input, family)?;
                let target = common_target(&stressed, family)?;
                let stressed_score = arm_score(arm, &stressed)?;
                let matched = shared_match(stressed_score, target);
                let clean_match = clean[index].matches_oracle;
                cell.n_cases += 1;
                cell.clean_matches += u64::from(clean_match);
                cell.stressed_matches += u64::from(matched);
                cell.flips += u64::from(matched != clean_match);
                cell.max_abs_score_change = cell
                    .max_abs_score_change
                    .max((stressed_score - clean_score).abs());
                cell.max_abs_target_change = cell
                    .max_abs_target_change
                    .max((target - clean_target).abs());
            }
            cells.push(cell);
        }
    }
    Ok(cells)
}

/// Per (block, mirror transformation, origin transformation, offset, arm)
/// match and invariance accounting of the mixed adversarial suite (TDI-25
/// slice 44 mirror/parity classes composed with slice 43 translation/origin
/// classes).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MixedStressCell {
    pub family: TaskFamily,
    pub seed_block: u64,
    pub mirror: MirrorStressTransform,
    pub origin: OriginStressTransform,
    pub offset: f64,
    pub arm: ComparisonArm,
    pub n_cases: u64,
    /// Matches of the matched primary on clean inputs.
    pub clean_matches: u64,
    /// Matches on the composed input against the common target recomputed
    /// from that same composed input inside the evaluator.
    pub stressed_matches: u64,
    /// Cases whose match bit differs between clean and composed inputs.
    pub flips: u64,
    /// Label-free: maximum absolute change of the arm score.
    pub max_abs_score_change: f64,
    /// Maximum absolute change of the recomputed common target.
    pub max_abs_target_change: f64,
    /// Maximum absolute difference between the arm score on the composed
    /// input and on the mirror-only input (the origin component's effect).
    pub max_abs_origin_effect: f64,
}

/// Compose one declared mirror/parity transformation with one declared
/// translation/origin transformation: the mirror acts first on the carriers,
/// then the origin stress acts on the mirrored input with the slice-43
/// per-case direction. No arm, score, target or label enters the composition.
pub fn mixed_stress_matched_input(
    input: &MatchedInput,
    mirror: MirrorStressTransform,
    origin: OriginStressTransform,
    offset: f64,
    seed: u64,
    case_key: u64,
) -> Result<MatchedInput, EvalError> {
    let mirrored = mirror_stress_matched_input(input, mirror)?;
    stress_matched_input(&mirrored, origin, offset, seed, case_key)
}

/// Score one matched block under every declared composition for T6 and C6.
/// Order: mirror transformation, origin transformation, offset, arm.
pub fn evaluate_mixed_adversarial_stress(
    split: DataSplit,
    family: TaskFamily,
    seed_block: u64,
    n_cases: u64,
    seed: u64,
) -> Result<Vec<MixedStressCell>, EvalError> {
    let run = MatchedPrimaryRun::evaluate(split, family, seed_block, n_cases)?;
    let mut cells = Vec::new();
    for mirror in MIRROR_STRESS_TRANSFORMS {
        for origin in ORIGIN_STRESS_TRANSFORMS {
            for offset in ORIGIN_STRESS_OFFSETS {
                for arm in SEQUENCE_SCALING_ARMS {
                    let (clean, clean_scores) = match arm {
                        ComparisonArm::T6 => (run.t6_outcomes(), run.t6_scores()),
                        _ => (run.c6_outcomes(), run.c6_scores()),
                    };
                    let mut cell = MixedStressCell {
                        family,
                        seed_block,
                        mirror,
                        origin,
                        offset,
                        arm,
                        n_cases: 0,
                        clean_matches: 0,
                        stressed_matches: 0,
                        flips: 0,
                        max_abs_score_change: 0.0,
                        max_abs_target_change: 0.0,
                        max_abs_origin_effect: 0.0,
                    };
                    for (index, input) in run.inputs().iter().enumerate() {
                        let clean_score = arm_score(arm, input)?;
                        if clean_score.to_bits() != clean_scores[index].to_bits() {
                            return Err(EvalError::MixedAdversarialStressInvalid {
                                reason: "clean_reference_drift",
                            });
                        }
                        let case_key = (seed_block << 32) | index as u64;
                        let mirrored = mirror_stress_matched_input(input, mirror)?;
                        let composed = mixed_stress_matched_input(
                            input, mirror, origin, offset, seed, case_key,
                        )?;
                        let clean_target = common_target(input, family)?;
                        let target = common_target(&composed, family)?;
                        let mirrored_score = arm_score(arm, &mirrored)?;
                        let stressed_score = arm_score(arm, &composed)?;
                        let matched = shared_match(stressed_score, target);
                        let clean_match = clean[index].matches_oracle;
                        cell.n_cases += 1;
                        cell.clean_matches += u64::from(clean_match);
                        cell.stressed_matches += u64::from(matched);
                        cell.flips += u64::from(matched != clean_match);
                        cell.max_abs_score_change = cell
                            .max_abs_score_change
                            .max((stressed_score - clean_score).abs());
                        cell.max_abs_target_change = cell
                            .max_abs_target_change
                            .max((target - clean_target).abs());
                        cell.max_abs_origin_effect = cell
                            .max_abs_origin_effect
                            .max((stressed_score - mirrored_score).abs());
                    }
                    cells.push(cell);
                }
            }
        }
    }
    Ok(cells)
}

/// Input class of the numerical precision study (TDI-25 slice 46): the clean
/// matched input, or the slice-43 translation/origin stress at one declared
/// transformation and offset (large-magnitude, cancellation-prone inputs).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PrecisionInput {
    Clean,
    OriginStress(OriginStressTransform, f64),
}

/// Per (block, input class, arm) f64/f32 accounting.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MatchedPrecisionCell {
    pub family: TaskFamily,
    pub seed_block: u64,
    pub input: PrecisionInput,
    pub arm: ComparisonArm,
    pub n_cases: u64,
    /// Finite f32 score within the declared forward-error bound.
    pub within_tolerance: u64,
    /// Finite f32 score outside the declared bound.
    pub tolerance_failures: u64,
    /// Non-finite f32 score (overflow); never dropped.
    pub non_finite_f32: u64,
    /// Cases whose f32 score sign differs from the f64 score sign.
    pub sign_flips: u64,
    pub max_abs_error: f64,
    pub max_error_to_bound: f64,
}

fn f32_vec(data: [f64; 3]) -> [f32; 3] {
    data.map(|value| value as f32)
}

fn f32_cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn f32_dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn abs_cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        (a[1] * b[2]).abs() + (a[2] * b[1]).abs(),
        (a[2] * b[0]).abs() + (a[0] * b[2]).abs(),
        (a[0] * b[1]).abs() + (a[1] * b[0]).abs(),
    ]
}

/// T6 factorized score `(v + Q x omega).R + omega.(M + P x R)` evaluated in
/// f32 (inputs rounded to f32, reference operation order), returned with the
/// f64 condition: the sum of absolute elementary products of the expansion.
#[must_use]
pub fn t6_score_f32(input: &MatchedInput) -> (f64, f64) {
    let v = [input.query[0], input.query[1], input.query[2]];
    let omega = [input.query[3], input.query[4], input.query[5]];
    let resultant = [input.key[0], input.key[1], input.key[2]];
    let moment = [input.key[3], input.key[4], input.key[5]];
    let (q32, w32) = (f32_vec(input.query_position), f32_vec(omega));
    let (r32, p32) = (f32_vec(resultant), f32_vec(input.key_position));
    let qw = f32_cross(q32, w32);
    let v32 = f32_vec(v);
    let dual = [v32[0] + qw[0], v32[1] + qw[1], v32[2] + qw[2]];
    let pr = f32_cross(p32, r32);
    let m32 = f32_vec(moment);
    let origin = [m32[0] + pr[0], m32[1] + pr[1], m32[2] + pr[2]];
    let score = f32_dot(dual, r32) + f32_dot(w32, origin);
    let qw_abs = abs_cross(input.query_position, omega);
    let pr_abs = abs_cross(input.key_position, resultant);
    let mut condition = 0.0;
    for index in 0..3 {
        condition += (v[index].abs() + qw_abs[index]) * resultant[index].abs();
        condition += omega[index].abs() * (moment[index].abs() + pr_abs[index]);
    }
    (f64::from(score), condition)
}

/// C6 score `(1, 0, 1)` in f32 through the upstream TDI-24 slice-44 kernel.
#[must_use]
pub fn c6_score_f32(input: &MatchedInput) -> Option<(f64, f64)> {
    let (query, key) = chiral_carriers(input).ok()?;
    Some(chiral_score_f32(query, key, MATCHED_CHIRAL_WEIGHTS))
}

/// Declared input classes, all reported: clean, then every slice-43
/// transformation at every slice-43 offset.
#[must_use]
pub fn precision_inputs() -> Vec<PrecisionInput> {
    let mut inputs = vec![PrecisionInput::Clean];
    for transform in ORIGIN_STRESS_TRANSFORMS {
        for offset in ORIGIN_STRESS_OFFSETS {
            inputs.push(PrecisionInput::OriginStress(transform, offset));
        }
    }
    inputs
}

/// Score one matched block in f64 and f32 for T6 and C6 under every declared
/// input class. Order: input class, arm.
pub fn evaluate_matched_numerical_precision(
    split: DataSplit,
    family: TaskFamily,
    seed_block: u64,
    n_cases: u64,
    seed: u64,
) -> Result<Vec<MatchedPrecisionCell>, EvalError> {
    let run = MatchedPrimaryRun::evaluate(split, family, seed_block, n_cases)?;
    let tolerance = PRECISION_BOUND_FACTOR * f64::from(f32::EPSILON);
    let mut cells = Vec::new();
    for class in precision_inputs() {
        for arm in SEQUENCE_SCALING_ARMS {
            let mut cell = MatchedPrecisionCell {
                family,
                seed_block,
                input: class,
                arm,
                n_cases: 0,
                within_tolerance: 0,
                tolerance_failures: 0,
                non_finite_f32: 0,
                sign_flips: 0,
                max_abs_error: 0.0,
                max_error_to_bound: 0.0,
            };
            for (index, clean) in run.inputs().iter().enumerate() {
                let input = match class {
                    PrecisionInput::Clean => clean.clone(),
                    PrecisionInput::OriginStress(transform, offset) => {
                        let case_key = (seed_block << 32) | index as u64;
                        stress_matched_input(clean, transform, offset, seed, case_key)?
                    }
                };
                let reference = arm_score(arm, &input)?;
                let (single, condition) = match arm {
                    ComparisonArm::T6 => t6_score_f32(&input),
                    _ => c6_score_f32(&input).ok_or(EvalError::MatchedPrecisionInvalid {
                        reason: "carrier_invalid",
                    })?,
                };
                cell.n_cases += 1;
                if !single.is_finite() {
                    cell.non_finite_f32 += 1;
                    continue;
                }
                let error = (single - reference).abs();
                let bound = tolerance * condition;
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
                cell.sign_flips += u64::from(
                    (single > 0.0) != (reference > 0.0) || (single < 0.0) != (reference < 0.0),
                );
                cell.max_abs_error = cell.max_abs_error.max(error);
                cell.max_error_to_bound = cell.max_error_to_bound.max(ratio);
            }
            cells.push(cell);
        }
    }
    Ok(cells)
}

/// Public f64 T6 primary score of one matched input (timing harness only).
pub fn t6_score_f64(input: &MatchedInput) -> Result<f64, EvalError> {
    score_t6(input)
}

/// Public f64 C6 primary score of one matched input (timing harness only).
pub fn c6_score_f64(input: &MatchedInput) -> Result<f64, EvalError> {
    score_c6(input)
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
