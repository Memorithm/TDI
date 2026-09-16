//! TDI-22.2 bounded non-final evaluator foundation.
//!
//! Implements only the semantics authorised by the frozen TDI-22.1 contract:
//! deterministic split streams, frozen geometry mappings, T0/T1/T3/T4 scoring,
//! tie handling, semantic resource accounting, and canonical non-final records.
//! No final/confirmatory surface, FLAT-ATTENTION runtime integration, or hardware
//! performance claim is introduced here.

use core::fmt;

use super::tdi22_torsor::{
    FactorizedTorsorKey, FactorizedTwistQuery, Torsor3, TorsorError, Twist3, Vec3,
};

pub const DEVELOPMENT_DOMAIN: u64 = 0x5444_4932_3244_4556;
pub const VALIDATION_DOMAIN: u64 = 0x5444_4932_3256_414c;
pub const CANDIDATES_PER_QUERY: usize = 16;
pub const QUERIES_PER_EPISODE: usize = 8;
pub const DEVELOPMENT_EPISODES_PER_CELL: u64 = 16;
pub const VALIDATION_EPISODES_PER_CELL: u64 = 32;
pub const MAX_GEOMETRY_INDEX: u64 = 255;
pub const MAX_COMPONENT_ABS: f64 = 64.0;
pub const MAX_SUPPLIED_POSITION_ABS: f64 = 1.75;
pub const SCORE_TOLERANCE_EPSILON_MULTIPLIER: f64 = 512.0;
pub const RECORD_SCHEMA_VERSION: &str = "tdi22-eval-record-v1";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Split {
    Development,
    Validation,
}

impl Split {
    #[must_use]
    pub const fn domain(self) -> u64 {
        match self {
            Self::Development => DEVELOPMENT_DOMAIN,
            Self::Validation => VALIDATION_DOMAIN,
        }
    }

    #[must_use]
    pub const fn token(self) -> &'static str {
        match self {
            Self::Development => "development",
            Self::Validation => "validation",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum PrimaryCell {
    P1 = 1,
    P2 = 2,
    P3 = 3,
    P4 = 4,
    P5 = 5,
}

impl PrimaryCell {
    #[must_use]
    pub const fn id(self) -> u8 {
        self as u8
    }

    #[must_use]
    pub const fn token(self) -> &'static str {
        match self {
            Self::P1 => "P1",
            Self::P2 => "P2",
            Self::P3 => "P3",
            Self::P4 => "P4",
            Self::P5 => "P5",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Geometry {
    G0Origin,
    G1Linear,
    G2Helix,
    G3Supplied,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Arm {
    T0,
    T1,
    T3,
    T4,
}

impl Arm {
    #[must_use]
    pub const fn token(self) -> &'static str {
        match self {
            Self::T0 => "T0",
            Self::T1 => "T1",
            Self::T3 => "T3",
            Self::T4 => "T4",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EvalQuery {
    twist: Twist3,
    position: Vec3,
}

impl EvalQuery {
    pub fn new(twist: Twist3, position: Vec3) -> Result<Self, EvalError> {
        bound_vec(twist.linear(), "query_linear", MAX_COMPONENT_ABS)?;
        bound_vec(twist.angular(), "query_angular", MAX_COMPONENT_ABS)?;
        bound_vec(position, "query_position", MAX_COMPONENT_ABS)?;
        Ok(Self { twist, position })
    }

    #[must_use]
    pub const fn twist(self) -> Twist3 {
        self.twist
    }

    #[must_use]
    pub const fn position(self) -> Vec3 {
        self.position
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EvalKey {
    identity: u16,
    torsor: Torsor3,
    origin_moment: Vec3,
}

impl EvalKey {
    pub fn new(identity: u16, torsor: Torsor3) -> Result<Self, EvalError> {
        bound_vec(torsor.resultant(), "key_resultant", MAX_COMPONENT_ABS)?;
        bound_vec(torsor.moment(), "key_local_moment", MAX_COMPONENT_ABS)?;
        bound_vec(torsor.reference(), "key_reference", MAX_COMPONENT_ABS)?;
        let origin_moment = torsor.origin_moment().map_err(EvalError::Algebra)?;
        bound_vec(origin_moment, "key_origin_moment", MAX_COMPONENT_ABS)?;
        Ok(Self {
            identity,
            torsor,
            origin_moment,
        })
    }

    #[must_use]
    pub const fn identity(self) -> u16 {
        self.identity
    }

    #[must_use]
    pub const fn torsor(self) -> Torsor3 {
        self.torsor
    }

    #[must_use]
    pub const fn origin_moment(self) -> Vec3 {
        self.origin_moment
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TorsorValue {
    pub resultant: Vec3,
    pub origin_moment: Vec3,
}

impl From<EvalKey> for TorsorValue {
    fn from(key: EvalKey) -> Self {
        Self {
            resultant: key.torsor.resultant(),
            origin_moment: key.origin_moment,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ResourceLedger {
    pub query_bits: u64,
    pub key_bits: u64,
    pub value_bits: u64,
    pub position_bits: u64,
    pub dynamic_state_bits: u64,
    pub static_parameter_bits: u64,
    pub temporary_slots: u64,
    pub add_count: u64,
    pub mul_count: u64,
    pub cross_count: u64,
    pub dot_lane_count: u64,
    pub comparison_count: u64,
}

impl ResourceLedger {
    pub fn for_arm(arm: Arm, candidate_count: usize) -> Result<Self, EvalError> {
        let n = u64::try_from(candidate_count).map_err(|_| EvalError::ArithmeticOverflow)?;
        if n == 0 {
            return Err(EvalError::LengthMismatch);
        }
        let key_bits = 384_u64
            .checked_mul(n)
            .ok_or(EvalError::ArithmeticOverflow)?;
        let dot_lane_count = 6_u64.checked_mul(n).ok_or(EvalError::ArithmeticOverflow)?;
        let comparison_count = n.checked_sub(1).ok_or(EvalError::ArithmeticOverflow)?;
        let base_mul = 6_u64.checked_mul(n).ok_or(EvalError::ArithmeticOverflow)?;
        let base_add = 5_u64.checked_mul(n).ok_or(EvalError::ArithmeticOverflow)?;

        let (value_bits, position_bits, temporary_slots, add_count, mul_count, cross_count) =
            match arm {
                Arm::T0 | Arm::T4 => (0, 0, 2, base_add, base_mul, 0),
                Arm::T1 => (key_bits, 0, 2, base_add, base_mul, 0),
                Arm::T3 => (
                    0,
                    192,
                    8,
                    base_add
                        .checked_add(6)
                        .ok_or(EvalError::ArithmeticOverflow)?,
                    base_mul
                        .checked_add(6)
                        .ok_or(EvalError::ArithmeticOverflow)?,
                    1,
                ),
            };

        Ok(Self {
            query_bits: 384,
            key_bits,
            value_bits,
            position_bits,
            dynamic_state_bits: 0,
            static_parameter_bits: 0,
            temporary_slots,
            add_count,
            mul_count,
            cross_count,
            dot_lane_count,
            comparison_count,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ranking {
    pub selected_identity: u16,
    pub top_score: f64,
    pub runner_up_score: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RejectionReason {
    MalformedEpisode,
    NonFiniteInput,
    NonFiniteDerived,
    InvalidGeometry,
    OutOfBounds,
    MissingTarget,
    DuplicateTarget,
    LengthMismatch,
    UnresolvedProtocolParameter,
    ArithmeticOverflow,
    UnexpectedExtraOutput,
    AmbiguousTarget,
    RetryBudgetExhausted,
    AmbiguousT1Top,
}

impl RejectionReason {
    #[must_use]
    pub const fn token(self) -> &'static str {
        match self {
            Self::MalformedEpisode => "malformed_episode",
            Self::NonFiniteInput => "non_finite_input",
            Self::NonFiniteDerived => "non_finite_derived",
            Self::InvalidGeometry => "invalid_geometry",
            Self::OutOfBounds => "out_of_bounds",
            Self::MissingTarget => "missing_target",
            Self::DuplicateTarget => "duplicate_target",
            Self::LengthMismatch => "length_mismatch",
            Self::UnresolvedProtocolParameter => "unresolved_protocol_parameter",
            Self::ArithmeticOverflow => "arithmetic_overflow",
            Self::UnexpectedExtraOutput => "unexpected_extra_output",
            Self::AmbiguousTarget => "ambiguous_target",
            Self::RetryBudgetExhausted => "retry_budget_exhausted",
            Self::AmbiguousT1Top => "ambiguous_t1_top",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EvalError {
    Algebra(TorsorError),
    OutOfBounds { field: &'static str },
    GeometryIndex { index: u64 },
    SuppliedPositionRequired,
    NonQuarterSuppliedPosition,
    LengthMismatch,
    DuplicateIdentity { identity: u16 },
    AmbiguousTarget,
    AmbiguousT1Top,
    ArithmeticOverflow,
    NonFiniteScore,
}

impl fmt::Display for EvalError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Algebra(error) => write!(formatter, "torsor algebra error: {error}"),
            Self::OutOfBounds { field } => write!(formatter, "{field} exceeds frozen bounds"),
            Self::GeometryIndex { index } => {
                write!(formatter, "geometry index {index} exceeds frozen bound")
            }
            Self::SuppliedPositionRequired => {
                write!(formatter, "G3 requires an evaluator-supplied position")
            }
            Self::NonQuarterSuppliedPosition => write!(
                formatter,
                "G3 position must use exact quarter-integer components"
            ),
            Self::LengthMismatch => write!(
                formatter,
                "candidate count does not match the frozen contract"
            ),
            Self::DuplicateIdentity { identity } => {
                write!(formatter, "duplicate candidate identity {identity}")
            }
            Self::AmbiguousTarget => write!(
                formatter,
                "top score is ambiguous under the frozen tolerance"
            ),
            Self::AmbiguousT1Top => write!(
                formatter,
                "T1 top score is ambiguous under the frozen tolerance"
            ),
            Self::ArithmeticOverflow => write!(formatter, "arithmetic overflow"),
            Self::NonFiniteScore => write!(formatter, "candidate score is non-finite"),
        }
    }
}

impl std::error::Error for EvalError {}

fn bound_vec(value: Vec3, field: &'static str, bound: f64) -> Result<(), EvalError> {
    if !value.x.is_finite() || !value.y.is_finite() || !value.z.is_finite() {
        return Err(EvalError::OutOfBounds { field });
    }
    if value.x.abs() > bound || value.y.abs() > bound || value.z.abs() > bound {
        return Err(EvalError::OutOfBounds { field });
    }
    Ok(())
}

fn is_exact_quarter(value: f64) -> bool {
    let scaled = value * 4.0;
    scaled.is_finite() && scaled == scaled.round()
}

pub fn geometry_point(
    geometry: Geometry,
    index: u64,
    supplied: Option<Vec3>,
) -> Result<Vec3, EvalError> {
    if index > MAX_GEOMETRY_INDEX {
        return Err(EvalError::GeometryIndex { index });
    }

    let point = match geometry {
        Geometry::G0Origin => Vec3::zero(),
        Geometry::G1Linear => {
            Vec3::new(index as f64 / 64.0, 0.0, 0.0).map_err(EvalError::Algebra)?
        }
        Geometry::G2Helix => {
            let (x, y) = match index % 8 {
                0 => (1.0, 0.0),
                1 => (1.0, 1.0),
                2 => (0.0, 1.0),
                3 => (-1.0, 1.0),
                4 => (-1.0, 0.0),
                5 => (-1.0, -1.0),
                6 => (0.0, -1.0),
                7 => (1.0, -1.0),
                _ => unreachable!(),
            };
            Vec3::new(x, y, index as f64 / 64.0).map_err(EvalError::Algebra)?
        }
        Geometry::G3Supplied => {
            let value = supplied.ok_or(EvalError::SuppliedPositionRequired)?;
            if !is_exact_quarter(value.x)
                || !is_exact_quarter(value.y)
                || !is_exact_quarter(value.z)
            {
                return Err(EvalError::NonQuarterSuppliedPosition);
            }
            bound_vec(value, "supplied_position", MAX_SUPPLIED_POSITION_ABS)?;
            value
        }
    };

    bound_vec(point, "geometry_point", MAX_COMPONENT_ABS)?;
    Ok(point)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    pub fn for_episode(
        split: Split,
        cell: PrimaryCell,
        episode_index: u64,
    ) -> Result<Self, EvalError> {
        if episode_index >= (1_u64 << 48) {
            return Err(EvalError::ArithmeticOverflow);
        }
        let selector = (u64::from(cell.id()) << 56) | episode_index;
        Ok(Self {
            state: split.domain() ^ selector,
        })
    }

    #[must_use]
    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }

    pub fn half_step(&mut self) -> Result<f64, EvalError> {
        let residue =
            i64::try_from(self.next_u64() % 33).map_err(|_| EvalError::ArithmeticOverflow)?;
        Ok((residue - 16) as f64 / 2.0)
    }

    pub fn quarter_step(&mut self) -> Result<f64, EvalError> {
        let residue =
            i64::try_from(self.next_u64() % 15).map_err(|_| EvalError::ArithmeticOverflow)?;
        Ok((residue - 7) as f64 / 4.0)
    }
}

fn factorized_t3_query(query: EvalQuery) -> Result<FactorizedTwistQuery, EvalError> {
    let factorized = query
        .twist
        .factorized_at(query.position)
        .map_err(EvalError::Algebra)?;
    bound_vec(
        factorized.resultant_dual,
        "factorized_resultant_dual",
        MAX_COMPONENT_ABS,
    )?;
    bound_vec(
        factorized.moment_dual,
        "factorized_moment_dual",
        MAX_COMPONENT_ABS,
    )?;
    Ok(factorized)
}

fn score_t3_factorized(factorized: FactorizedTwistQuery, key: EvalKey) -> Result<f64, EvalError> {
    factorized
        .pair(FactorizedTorsorKey {
            resultant: key.torsor.resultant(),
            origin_moment: key.origin_moment,
        })
        .map_err(EvalError::Algebra)
}

pub fn score(arm: Arm, query: EvalQuery, key: EvalKey) -> Result<f64, EvalError> {
    let linear = query.twist.linear();
    let angular = query.twist.angular();
    let resultant = key.torsor.resultant();

    let value = match arm {
        Arm::T0 | Arm::T1 => linear.dot(resultant) + angular.dot(key.torsor.moment()),
        Arm::T4 => linear.dot(resultant) + angular.dot(key.origin_moment),
        Arm::T3 => score_t3_factorized(factorized_t3_query(query)?, key)?,
    };

    if value.is_finite() {
        Ok(value)
    } else {
        Err(EvalError::NonFiniteScore)
    }
}

#[must_use]
pub fn score_tolerance(lhs: f64, rhs: f64) -> f64 {
    SCORE_TOLERANCE_EPSILON_MULTIPLIER * f64::EPSILON * 1.0_f64.max(lhs.abs()).max(rhs.abs())
}

#[must_use]
pub fn scores_tied(lhs: f64, rhs: f64) -> bool {
    lhs.is_finite() && rhs.is_finite() && (lhs - rhs).abs() <= score_tolerance(lhs, rhs)
}

pub fn rank_candidates(
    arm: Arm,
    query: EvalQuery,
    candidates: &[EvalKey],
) -> Result<Ranking, EvalError> {
    if candidates.len() != CANDIDATES_PER_QUERY {
        return Err(EvalError::LengthMismatch);
    }
    for (index, candidate) in candidates.iter().enumerate() {
        if candidates[..index]
            .iter()
            .any(|prior| prior.identity == candidate.identity)
        {
            return Err(EvalError::DuplicateIdentity {
                identity: candidate.identity,
            });
        }
    }

    let factorized_t3 = if arm == Arm::T3 {
        Some(factorized_t3_query(query)?)
    } else {
        None
    };

    let mut scored = Vec::with_capacity(candidates.len());
    for candidate in candidates {
        let candidate_score = if let Some(factorized) = factorized_t3 {
            score_t3_factorized(factorized, *candidate)?
        } else {
            score(arm, query, *candidate)?
        };
        scored.push((candidate.identity, candidate_score));
    }
    scored.sort_by(|lhs, rhs| rhs.1.total_cmp(&lhs.1));

    if arm == Arm::T1 && scores_tied(scored[0].1, scored[1].1) {
        return Err(EvalError::AmbiguousT1Top);
    }

    Ok(Ranking {
        selected_identity: scored[0].0,
        top_score: scored[0].1,
        runner_up_score: scored[1].1,
    })
}

pub fn t1_value_readout(
    query: EvalQuery,
    candidates: &[EvalKey],
) -> Result<(Ranking, TorsorValue), EvalError> {
    let ranking = rank_candidates(Arm::T1, query, candidates)?;
    let selected = candidates
        .iter()
        .find(|candidate| candidate.identity == ranking.selected_identity)
        .copied()
        .ok_or(EvalError::LengthMismatch)?;
    Ok((ranking, selected.into()))
}

#[derive(Clone, Debug, PartialEq)]
pub struct EvalRecord {
    pub split: Split,
    pub cell: PrimaryCell,
    pub episode_index: u64,
    pub query_index: u64,
    pub arm: Arm,
    pub target_identity: Option<u16>,
    pub selected_identity: Option<u16>,
    pub exact_success: bool,
    pub target_score: Option<f64>,
    pub runner_up_score: Option<f64>,
    pub target_margin: Option<f64>,
    pub rejection_reason: Option<RejectionReason>,
    pub resources: ResourceLedger,
}

fn encode_identity(value: Option<u16>) -> String {
    value.map_or_else(|| "none".to_owned(), |value| value.to_string())
}

fn encode_score(value: Option<f64>) -> String {
    value.map_or_else(
        || "none".to_owned(),
        |value| format!("0x{:016x}", value.to_bits()),
    )
}

impl EvalRecord {
    #[must_use]
    pub fn encode_line(&self) -> String {
        let rejection = self.rejection_reason.map_or("none", RejectionReason::token);
        format!(
            "schema_version={RECORD_SCHEMA_VERSION}\tsplit={}\tcell={}\tepisode_index={}\tquery_index={}\tarm={}\ttarget_identity={}\tselected_identity={}\texact_success={}\ttarget_score_bits={}\trunner_up_score_bits={}\ttarget_margin_bits={}\trejection_reason={}\tquery_bits={}\tkey_bits={}\tvalue_bits={}\tposition_bits={}\tdynamic_state_bits={}\tstatic_parameter_bits={}\ttemporary_slots={}\tadd_count={}\tmul_count={}\tcross_count={}\tdot_lane_count={}\tcomparison_count={}\n",
            self.split.token(),
            self.cell.token(),
            self.episode_index,
            self.query_index,
            self.arm.token(),
            encode_identity(self.target_identity),
            encode_identity(self.selected_identity),
            u8::from(self.exact_success),
            encode_score(self.target_score),
            encode_score(self.runner_up_score),
            encode_score(self.target_margin),
            rejection,
            self.resources.query_bits,
            self.resources.key_bits,
            self.resources.value_bits,
            self.resources.position_bits,
            self.resources.dynamic_state_bits,
            self.resources.static_parameter_bits,
            self.resources.temporary_slots,
            self.resources.add_count,
            self.resources.mul_count,
            self.resources.cross_count,
            self.resources.dot_lane_count,
            self.resources.comparison_count,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(x: f64, y: f64, z: f64) -> Vec3 {
        Vec3::new(x, y, z).unwrap()
    }

    fn query(position: Vec3) -> EvalQuery {
        EvalQuery::new(
            Twist3::new(v(1.0, -2.0, 0.5), v(0.25, 1.5, -0.75)).unwrap(),
            position,
        )
        .unwrap()
    }

    fn key(identity: u16, scale: f64, reference: Vec3) -> EvalKey {
        EvalKey::new(
            identity,
            Torsor3::new(
                v(scale, 0.5 * scale, -0.25 * scale),
                v(0.75 * scale, -0.5 * scale, 0.125 * scale),
                reference,
            )
            .unwrap(),
        )
        .unwrap()
    }

    #[test]
    fn split_streams_are_deterministic_and_domain_separated() {
        let mut a = SplitMix64::for_episode(Split::Development, PrimaryCell::P1, 3).unwrap();
        let mut b = SplitMix64::for_episode(Split::Development, PrimaryCell::P1, 3).unwrap();
        let mut c = SplitMix64::for_episode(Split::Validation, PrimaryCell::P1, 3).unwrap();
        assert_eq!(a.next_u64(), b.next_u64());
        assert_ne!(a.next_u64(), c.next_u64());
    }

    #[test]
    fn dyadic_generators_match_frozen_lattices() {
        let mut stream = SplitMix64::for_episode(Split::Development, PrimaryCell::P2, 0).unwrap();
        for _ in 0..128 {
            let half = stream.half_step().unwrap();
            assert!((-8.0..=8.0).contains(&half));
            assert_eq!(half * 2.0, (half * 2.0).round());
            let quarter = stream.quarter_step().unwrap();
            assert!((-1.75..=1.75).contains(&quarter));
            assert_eq!(quarter * 4.0, (quarter * 4.0).round());
        }
    }

    #[test]
    fn geometry_registry_matches_frozen_definitions() {
        assert_eq!(
            geometry_point(Geometry::G0Origin, 9, None).unwrap(),
            Vec3::zero()
        );
        assert_eq!(
            geometry_point(Geometry::G1Linear, 64, None).unwrap(),
            v(1.0, 0.0, 0.0)
        );
        assert_eq!(
            geometry_point(Geometry::G2Helix, 9, None).unwrap(),
            v(1.0, 1.0, 9.0 / 64.0)
        );
        assert_eq!(
            geometry_point(Geometry::G3Supplied, 0, Some(v(1.75, -1.5, 0.25))).unwrap(),
            v(1.75, -1.5, 0.25)
        );
    }

    #[test]
    fn t3_minus_t4_is_exact_transport_term_within_roundoff() {
        let query = query(v(2.0, -3.0, 1.0));
        let key = key(0, 2.0, v(1.0, 1.0, -1.0));
        let t3 = score(Arm::T3, query, key).unwrap();
        let t4 = score(Arm::T4, query, key).unwrap();
        let expected = query
            .position()
            .cross(query.twist().angular())
            .dot(key.torsor().resultant());
        assert!((t3 - t4 - expected).abs() <= score_tolerance(t3 - t4, expected));
    }

    #[test]
    fn t0_and_t4_fixture_has_nonzero_moment_basis_difference() {
        let query = EvalQuery::new(
            Twist3::new(v(0.0, 0.0, 0.0), v(1.0, 0.0, 0.0)).unwrap(),
            Vec3::zero(),
        )
        .unwrap();
        let key = EvalKey::new(
            0,
            Torsor3::new(v(1.0, 2.0, 0.0), v(0.0, 0.0, 0.0), v(0.0, 0.0, 1.0)).unwrap(),
        )
        .unwrap();
        assert_ne!(
            score(Arm::T0, query, key).unwrap(),
            score(Arm::T4, query, key).unwrap()
        );
    }

    #[test]
    fn t3_rejects_out_of_bound_factorized_query_before_scoring() {
        let query = EvalQuery::new(
            Twist3::new(Vec3::zero(), v(8.0, -8.0, 8.0)).unwrap(),
            v(32.0, 32.0, 32.0),
        )
        .unwrap();
        let key = key(0, 1.0, Vec3::zero());
        assert!(matches!(
            score(Arm::T3, query, key),
            Err(EvalError::OutOfBounds {
                field: "factorized_resultant_dual"
            })
        ));
    }

    #[test]
    fn t1_tie_is_fail_closed_with_specific_error() {
        let query = query(Vec3::zero());
        let mut candidates = Vec::new();
        for identity in 0..CANDIDATES_PER_QUERY {
            candidates.push(key(identity as u16, 1.0, Vec3::zero()));
        }
        assert_eq!(
            rank_candidates(Arm::T1, query, &candidates),
            Err(EvalError::AmbiguousT1Top)
        );
    }

    #[test]
    fn non_t1_top_ties_remain_admissible() {
        let query = query(Vec3::zero());
        let mut candidates = Vec::new();
        for identity in 0..CANDIDATES_PER_QUERY {
            candidates.push(key(identity as u16, 1.0, Vec3::zero()));
        }

        for arm in [Arm::T0, Arm::T3, Arm::T4] {
            assert!(rank_candidates(arm, query, &candidates).is_ok());
        }
    }

    #[test]
    fn resource_ledger_exposes_all_frozen_fields() {
        let t3 = ResourceLedger::for_arm(Arm::T3, CANDIDATES_PER_QUERY).unwrap();
        assert_eq!(t3.cross_count, 1);
        assert_eq!(t3.dynamic_state_bits, 0);
        assert_eq!(t3.static_parameter_bits, 0);
        assert_eq!(t3.position_bits, 192);
    }

    #[test]
    fn canonical_record_has_exact_25_field_order_and_none_encoding() {
        let record = EvalRecord {
            split: Split::Development,
            cell: PrimaryCell::P1,
            episode_index: 2,
            query_index: 3,
            arm: Arm::T3,
            target_identity: None,
            selected_identity: None,
            exact_success: false,
            target_score: None,
            runner_up_score: None,
            target_margin: None,
            rejection_reason: Some(RejectionReason::MissingTarget),
            resources: ResourceLedger::default(),
        };
        let line = record.encode_line();
        assert_eq!(line.trim_end_matches('\n').split('\t').count(), 25);
        assert!(line.contains("\ttarget_identity=none\tselected_identity=none\texact_success=0\t"));
        assert!(line.contains("\trejection_reason=missing_target\t"));
        assert!(line.ends_with('\n'));
    }
}
