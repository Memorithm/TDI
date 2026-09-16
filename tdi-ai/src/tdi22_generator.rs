//! Deterministic non-final TDI-22.2 episode generation for frozen P1-P5 cells.
//!
//! The generator follows the frozen TDI-22.1 seed/draw-order contract. In
//! particular, F3 chooses the evaluator-owned target from content `(R,C)`
//! before nuisance geometry is assigned, then reconstructs `M(P)=C-P x R`.
//! This module does not execute a campaign, persist result payloads, expose a
//! final/confirmatory surface, or make a performance claim.

use core::fmt;

use super::tdi22_eval::{
    CANDIDATES_PER_QUERY, DEVELOPMENT_EPISODES_PER_CELL, EvalError, EvalKey, EvalQuery, Geometry,
    MAX_COMPONENT_ABS, PrimaryCell, QUERIES_PER_EPISODE, Split, SplitMix64,
    VALIDATION_EPISODES_PER_CELL, geometry_point, scores_tied,
};
use super::tdi22_torsor::{Torsor3, TorsorError, Twist3, Vec3};

const TARGET_RETRY_LIMIT: u8 = 32;
const GEOMETRY_MODULUS: u64 = 256;

#[derive(Clone, Debug, PartialEq)]
pub struct GeneratedQuery {
    pub query_index: u64,
    pub query: EvalQuery,
    pub candidates: Vec<EvalKey>,
    pub target_identity: u16,
    pub attempts_used: u8,
}

#[derive(Clone, Debug, PartialEq)]
pub struct GeneratedEpisode {
    pub split: Split,
    pub cell: PrimaryCell,
    pub episode_index: u64,
    pub queries: Vec<GeneratedQuery>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GeneratorError {
    Eval(EvalError),
    Algebra(TorsorError),
    EpisodeIndexOutOfRange { observed: u64, limit: u64 },
    QueryIndexOverflow,
    DerivedOutOfBounds { field: &'static str },
    AmbiguousTarget,
    RetryBudgetExhausted { query_index: u64 },
}

impl fmt::Display for GeneratorError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Eval(error) => write!(formatter, "evaluator error: {error}"),
            Self::Algebra(error) => write!(formatter, "torsor algebra error: {error}"),
            Self::EpisodeIndexOutOfRange { observed, limit } => write!(
                formatter,
                "episode index {observed} is outside frozen split bound 0..{limit}"
            ),
            Self::QueryIndexOverflow => write!(formatter, "query structural index overflow"),
            Self::DerivedOutOfBounds { field } => {
                write!(
                    formatter,
                    "derived field {field} exceeds frozen numeric domain"
                )
            }
            Self::AmbiguousTarget => write!(formatter, "evaluator-owned target is ambiguous"),
            Self::RetryBudgetExhausted { query_index } => write!(
                formatter,
                "target retry budget exhausted for query {query_index}"
            ),
        }
    }
}

impl std::error::Error for GeneratorError {}

impl From<EvalError> for GeneratorError {
    fn from(value: EvalError) -> Self {
        Self::Eval(value)
    }
}

impl From<TorsorError> for GeneratorError {
    fn from(value: TorsorError) -> Self {
        Self::Algebra(value)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct ContentCandidate {
    identity: u16,
    resultant: Vec3,
    origin_moment: Vec3,
}

fn bound_vec(value: Vec3, field: &'static str) -> Result<Vec3, GeneratorError> {
    if !value.x.is_finite()
        || !value.y.is_finite()
        || !value.z.is_finite()
        || value.x.abs() > MAX_COMPONENT_ABS
        || value.y.abs() > MAX_COMPONENT_ABS
        || value.z.abs() > MAX_COMPONENT_ABS
    {
        return Err(GeneratorError::DerivedOutOfBounds { field });
    }
    Ok(value)
}

fn draw_half_vec(stream: &mut SplitMix64) -> Result<Vec3, GeneratorError> {
    Ok(Vec3::new(
        stream.half_step()?,
        stream.half_step()?,
        stream.half_step()?,
    )?)
}

fn draw_quarter_vec(stream: &mut SplitMix64) -> Result<Vec3, GeneratorError> {
    Ok(Vec3::new(
        stream.quarter_step()?,
        stream.quarter_step()?,
        stream.quarter_step()?,
    )?)
}

fn draw_twist(stream: &mut SplitMix64) -> Result<Twist3, GeneratorError> {
    Ok(Twist3::new(draw_half_vec(stream)?, draw_half_vec(stream)?)?)
}

fn episode_limit(split: Split) -> u64 {
    match split {
        Split::Development => DEVELOPMENT_EPISODES_PER_CELL,
        Split::Validation => VALIDATION_EPISODES_PER_CELL,
    }
}

fn query_global_index(episode_index: u64, query_index: u64) -> Result<u64, GeneratorError> {
    episode_index
        .checked_mul(QUERIES_PER_EPISODE as u64)
        .and_then(|value| value.checked_add(query_index))
        .filter(|value| *value < GEOMETRY_MODULUS)
        .ok_or(GeneratorError::QueryIndexOverflow)
}

fn candidate_geometry_index(global: u64, identity: u16) -> Result<u64, GeneratorError> {
    global
        .checked_add(u64::from(identity))
        .and_then(|value| value.checked_add(1))
        .map(|value| value % GEOMETRY_MODULUS)
        .ok_or(GeneratorError::QueryIndexOverflow)
}

fn unique_top(mut scored: Vec<(u16, f64)>) -> Result<u16, GeneratorError> {
    if scored.len() != CANDIDATES_PER_QUERY {
        return Err(GeneratorError::Eval(EvalError::LengthMismatch));
    }
    if scored.iter().any(|(_, score)| !score.is_finite()) {
        return Err(GeneratorError::Eval(EvalError::NonFiniteScore));
    }
    scored.sort_by(|lhs, rhs| rhs.1.total_cmp(&lhs.1));
    if scores_tied(scored[0].1, scored[1].1) {
        return Err(GeneratorError::AmbiguousTarget);
    }
    Ok(scored[0].0)
}

fn content_score(twist: Twist3, candidate: ContentCandidate) -> f64 {
    twist.linear().dot(candidate.resultant) + twist.angular().dot(candidate.origin_moment)
}

fn unique_content_target(
    twist: Twist3,
    candidates: &[ContentCandidate],
) -> Result<u16, GeneratorError> {
    unique_top(
        candidates
            .iter()
            .map(|candidate| (candidate.identity, content_score(twist, *candidate)))
            .collect(),
    )
}

fn unique_direct_target(query: EvalQuery, candidates: &[EvalKey]) -> Result<u16, GeneratorError> {
    let mut scored = Vec::with_capacity(candidates.len());
    for candidate in candidates {
        let transported = bound_vec(
            candidate.torsor().moment_at(query.position())?,
            "transported_moment",
        )?;
        let score = query.twist().linear().dot(candidate.torsor().resultant())
            + query.twist().angular().dot(transported);
        scored.push((candidate.identity(), score));
    }
    unique_top(scored)
}

fn generate_f1_attempt(
    stream: &mut SplitMix64,
    query_index: u64,
) -> Result<GeneratedQuery, GeneratorError> {
    let twist = draw_twist(stream)?;
    let query_position = draw_quarter_vec(stream)?;
    let query = EvalQuery::new(twist, query_position)?;
    let mut candidates = Vec::with_capacity(CANDIDATES_PER_QUERY);
    for identity in 0..CANDIDATES_PER_QUERY {
        let resultant = draw_half_vec(stream)?;
        let local_moment = draw_half_vec(stream)?;
        let reference = draw_quarter_vec(stream)?;
        let torsor = Torsor3::new(resultant, local_moment, reference)?;
        candidates.push(EvalKey::new(identity as u16, torsor)?);
    }
    let target_identity = unique_direct_target(query, &candidates)?;
    Ok(GeneratedQuery {
        query_index,
        query,
        candidates,
        target_identity,
        attempts_used: 1,
    })
}

fn draw_content_candidates(
    stream: &mut SplitMix64,
) -> Result<Vec<ContentCandidate>, GeneratorError> {
    let mut candidates = Vec::with_capacity(CANDIDATES_PER_QUERY);
    for identity in 0..CANDIDATES_PER_QUERY {
        candidates.push(ContentCandidate {
            identity: identity as u16,
            resultant: draw_half_vec(stream)?,
            origin_moment: draw_half_vec(stream)?,
        });
    }
    Ok(candidates)
}

fn materialize_content_at(
    candidate: ContentCandidate,
    reference: Vec3,
) -> Result<EvalKey, GeneratorError> {
    let cross = bound_vec(
        reference.cross(candidate.resultant),
        "position_cross_resultant",
    )?;
    let local_moment = bound_vec(
        candidate.origin_moment.minus(cross),
        "reconstructed_local_moment",
    )?;
    Ok(EvalKey::new(
        candidate.identity,
        Torsor3::new(candidate.resultant, local_moment, reference)?,
    )?)
}

fn generate_f2_attempt(
    stream: &mut SplitMix64,
    query_index: u64,
) -> Result<GeneratedQuery, GeneratorError> {
    let twist = draw_twist(stream)?;
    let content = draw_content_candidates(stream)?;
    let target_identity = unique_content_target(twist, &content)?;
    let query = EvalQuery::new(twist, Vec3::zero())?;
    let candidates = content
        .into_iter()
        .map(|candidate| materialize_content_at(candidate, Vec3::zero()))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(GeneratedQuery {
        query_index,
        query,
        candidates,
        target_identity,
        attempts_used: 1,
    })
}

fn materialize_f3(
    stream: &mut SplitMix64,
    cell: PrimaryCell,
    episode_index: u64,
    query_index: u64,
    twist: Twist3,
    content: &[ContentCandidate],
    target_identity: u16,
) -> Result<GeneratedQuery, GeneratorError> {
    let global = query_global_index(episode_index, query_index)?;
    let (geometry, query_supplied) = match cell {
        PrimaryCell::P3 => (Geometry::G1Linear, None),
        PrimaryCell::P4 => (Geometry::G2Helix, None),
        PrimaryCell::P5 => (Geometry::G3Supplied, Some(draw_quarter_vec(stream)?)),
        _ => return Err(GeneratorError::Eval(EvalError::LengthMismatch)),
    };
    let query_position = geometry_point(geometry, global, query_supplied)?;
    let query = EvalQuery::new(twist, query_position)?;

    let mut candidates = Vec::with_capacity(CANDIDATES_PER_QUERY);
    for candidate in content {
        let supplied = if cell == PrimaryCell::P5 {
            Some(draw_quarter_vec(stream)?)
        } else {
            None
        };
        let geometry_index = candidate_geometry_index(global, candidate.identity)?;
        let reference = geometry_point(geometry, geometry_index, supplied)?;
        candidates.push(materialize_content_at(*candidate, reference)?);
    }

    // The factorized T3 query is a derived vector and must satisfy the same
    // frozen numeric-domain bound before the episode is admitted.
    let factorized = query.twist().factorized_at(query.position())?;
    bound_vec(factorized.resultant_dual, "factorized_resultant_dual")?;
    bound_vec(factorized.moment_dual, "factorized_moment_dual")?;

    Ok(GeneratedQuery {
        query_index,
        query,
        candidates,
        target_identity,
        attempts_used: 1,
    })
}

fn generate_f3_attempt(
    stream: &mut SplitMix64,
    cell: PrimaryCell,
    episode_index: u64,
    query_index: u64,
) -> Result<GeneratedQuery, GeneratorError> {
    // Frozen causal ordering: target selection occurs entirely in content
    // space before any nuisance-position word is consumed or position assigned.
    let twist = draw_twist(stream)?;
    let content = draw_content_candidates(stream)?;
    let target_identity = unique_content_target(twist, &content)?;
    materialize_f3(
        stream,
        cell,
        episode_index,
        query_index,
        twist,
        &content,
        target_identity,
    )
}

fn generate_query_attempt(
    stream: &mut SplitMix64,
    cell: PrimaryCell,
    episode_index: u64,
    query_index: u64,
) -> Result<GeneratedQuery, GeneratorError> {
    match cell {
        PrimaryCell::P1 => generate_f1_attempt(stream, query_index),
        PrimaryCell::P2 => generate_f2_attempt(stream, query_index),
        PrimaryCell::P3 | PrimaryCell::P4 | PrimaryCell::P5 => {
            generate_f3_attempt(stream, cell, episode_index, query_index)
        }
    }
}

pub fn generate_episode(
    split: Split,
    cell: PrimaryCell,
    episode_index: u64,
) -> Result<GeneratedEpisode, GeneratorError> {
    let limit = episode_limit(split);
    if episode_index >= limit {
        return Err(GeneratorError::EpisodeIndexOutOfRange {
            observed: episode_index,
            limit,
        });
    }

    let mut stream = SplitMix64::for_episode(split, cell, episode_index)?;
    let mut queries = Vec::with_capacity(QUERIES_PER_EPISODE);
    for query_index in 0..QUERIES_PER_EPISODE as u64 {
        let mut accepted = None;
        for attempt in 1..=TARGET_RETRY_LIMIT {
            match generate_query_attempt(&mut stream, cell, episode_index, query_index) {
                Ok(mut query) => {
                    query.attempts_used = attempt;
                    accepted = Some(query);
                    break;
                }
                Err(GeneratorError::AmbiguousTarget) => continue,
                Err(error) => return Err(error),
            }
        }
        queries.push(accepted.ok_or(GeneratorError::RetryBudgetExhausted { query_index })?);
    }

    Ok(GeneratedEpisode {
        split,
        cell,
        episode_index,
        queries,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn target_by_content(query: EvalQuery, candidates: &[EvalKey]) -> u16 {
        let mut scored: Vec<_> = candidates
            .iter()
            .map(|candidate| {
                (
                    candidate.identity(),
                    query.twist().linear().dot(candidate.torsor().resultant())
                        + query.twist().angular().dot(candidate.origin_moment()),
                )
            })
            .collect();
        scored.sort_by(|lhs, rhs| rhs.1.total_cmp(&lhs.1));
        assert!(!scores_tied(scored[0].1, scored[1].1));
        scored[0].0
    }

    #[test]
    fn replay_is_semantically_identical_for_every_primary_cell() {
        for cell in [
            PrimaryCell::P1,
            PrimaryCell::P2,
            PrimaryCell::P3,
            PrimaryCell::P4,
            PrimaryCell::P5,
        ] {
            let first = generate_episode(Split::Development, cell, 0).unwrap();
            let second = generate_episode(Split::Development, cell, 0).unwrap();
            assert_eq!(first, second);
            assert_eq!(first.queries.len(), QUERIES_PER_EPISODE);
            assert!(
                first
                    .queries
                    .iter()
                    .all(|query| query.candidates.len() == CANDIDATES_PER_QUERY)
            );
        }
    }

    #[test]
    fn f3_target_remains_the_content_target_after_nuisance_materialization() {
        for cell in [PrimaryCell::P3, PrimaryCell::P4, PrimaryCell::P5] {
            let episode = generate_episode(Split::Development, cell, 1).unwrap();
            for generated in episode.queries {
                assert_eq!(
                    generated.target_identity,
                    target_by_content(generated.query, &generated.candidates)
                );
            }
        }
    }

    #[test]
    fn f3_reconstruction_preserves_origin_reduced_content() {
        let episode = generate_episode(Split::Validation, PrimaryCell::P5, 3).unwrap();
        for generated in episode.queries {
            for candidate in generated.candidates {
                let recomputed = candidate.torsor().origin_moment().unwrap();
                assert_eq!(recomputed, candidate.origin_moment());
            }
        }
    }

    #[test]
    fn p1_target_is_unique_under_direct_transported_pairing() {
        let episode = generate_episode(Split::Development, PrimaryCell::P1, 2).unwrap();
        for generated in episode.queries {
            assert_eq!(
                generated.target_identity,
                unique_direct_target(generated.query, &generated.candidates).unwrap()
            );
        }
    }

    #[test]
    fn split_population_bounds_fail_closed() {
        assert!(matches!(
            generate_episode(
                Split::Development,
                PrimaryCell::P2,
                DEVELOPMENT_EPISODES_PER_CELL
            ),
            Err(GeneratorError::EpisodeIndexOutOfRange { .. })
        ));
        assert!(matches!(
            generate_episode(
                Split::Validation,
                PrimaryCell::P2,
                VALIDATION_EPISODES_PER_CELL
            ),
            Err(GeneratorError::EpisodeIndexOutOfRange { .. })
        ));
    }
}
