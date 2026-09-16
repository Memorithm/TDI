//! Bounded non-final TDI-22.2 record materialization.
//!
//! Consumes already-qualified generated episodes and produces canonical base
//! evaluation records plus the frozen T1 value-reconstruction companion record.
//! No record is persisted here and no aggregate scientific verdict is computed.

use core::fmt;

use super::tdi22_eval::{
    Arm, EvalError, EvalKey, EvalRecord, PrimaryCell, RejectionReason, ResourceLedger, Split,
    TorsorValue, rank_candidates, score, score_tolerance,
};
use super::tdi22_generator::{GeneratedEpisode, GeneratedQuery};

pub const T1_VALUE_SCHEMA_VERSION: &str = "tdi22-t1-value-record-v1";
const ARMS: [Arm; 4] = [Arm::T0, Arm::T1, Arm::T3, Arm::T4];

#[derive(Clone, Debug, PartialEq)]
pub struct T1ValueRecord {
    pub split: Split,
    pub cell: PrimaryCell,
    pub episode_index: u64,
    pub query_index: u64,
    pub target_identity: Option<u16>,
    pub selected_identity: Option<u16>,
    pub reconstruction_success: bool,
    pub target_value: Option<TorsorValue>,
    pub selected_value: Option<TorsorValue>,
    pub rejection_reason: Option<RejectionReason>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct EpisodeRecords {
    pub base_records: Vec<EvalRecord>,
    pub t1_value_records: Vec<T1ValueRecord>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RunnerError {
    Eval(EvalError),
    MissingTarget { identity: u16 },
    MissingSelected { identity: u16 },
}

impl fmt::Display for RunnerError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Eval(error) => write!(formatter, "evaluator error: {error}"),
            Self::MissingTarget { identity } => write!(formatter, "missing target identity {identity}"),
            Self::MissingSelected { identity } => {
                write!(formatter, "missing selected identity {identity}")
            }
        }
    }
}

impl std::error::Error for RunnerError {}

impl From<EvalError> for RunnerError {
    fn from(value: EvalError) -> Self {
        Self::Eval(value)
    }
}

fn key_by_identity(candidates: &[EvalKey], identity: u16) -> Option<EvalKey> {
    candidates
        .iter()
        .find(|candidate| candidate.identity() == identity)
        .copied()
}

fn target_metrics(
    arm: Arm,
    generated: &GeneratedQuery,
) -> Result<(f64, f64, f64), RunnerError> {
    let target = key_by_identity(&generated.candidates, generated.target_identity).ok_or(
        RunnerError::MissingTarget {
            identity: generated.target_identity,
        },
    )?;
    let target_score = score(arm, generated.query, target)?;
    let mut best_distractor = f64::NEG_INFINITY;
    for candidate in &generated.candidates {
        if candidate.identity() == generated.target_identity {
            continue;
        }
        let candidate_score = score(arm, generated.query, *candidate)?;
        best_distractor = best_distractor.max(candidate_score);
    }
    if !best_distractor.is_finite() {
        return Err(RunnerError::Eval(EvalError::NonFiniteScore));
    }
    Ok((
        target_score,
        best_distractor,
        target_score - best_distractor,
    ))
}

fn value_component_close(lhs: f64, rhs: f64) -> bool {
    lhs.is_finite() && rhs.is_finite() && (lhs - rhs).abs() <= score_tolerance(lhs, rhs)
}

fn value_close(lhs: TorsorValue, rhs: TorsorValue) -> bool {
    value_component_close(lhs.resultant.x, rhs.resultant.x)
        && value_component_close(lhs.resultant.y, rhs.resultant.y)
        && value_component_close(lhs.resultant.z, rhs.resultant.z)
        && value_component_close(lhs.origin_moment.x, rhs.origin_moment.x)
        && value_component_close(lhs.origin_moment.y, rhs.origin_moment.y)
        && value_component_close(lhs.origin_moment.z, rhs.origin_moment.z)
}

fn encode_identity(value: Option<u16>) -> String {
    value.map_or_else(|| "none".to_owned(), |value| value.to_string())
}

fn encode_component(value: Option<f64>) -> String {
    value.map_or_else(
        || "none".to_owned(),
        |value| format!("0x{:016x}", value.to_bits()),
    )
}

fn components(value: Option<TorsorValue>) -> [Option<f64>; 6] {
    match value {
        Some(value) => [
            Some(value.resultant.x),
            Some(value.resultant.y),
            Some(value.resultant.z),
            Some(value.origin_moment.x),
            Some(value.origin_moment.y),
            Some(value.origin_moment.z),
        ],
        None => [None; 6],
    }
}

impl T1ValueRecord {
    #[must_use]
    pub fn encode_line(&self) -> String {
        let target = components(self.target_value);
        let selected = components(self.selected_value);
        let rejection = self.rejection_reason.map_or("none", RejectionReason::token);
        format!(
            "schema_version={T1_VALUE_SCHEMA_VERSION}\tsplit={}\tcell={}\tepisode_index={}\tquery_index={}\ttarget_identity={}\tselected_identity={}\treconstruction_success={}\ttarget_r_x_bits={}\ttarget_r_y_bits={}\ttarget_r_z_bits={}\ttarget_c_x_bits={}\ttarget_c_y_bits={}\ttarget_c_z_bits={}\tselected_r_x_bits={}\tselected_r_y_bits={}\tselected_r_z_bits={}\tselected_c_x_bits={}\tselected_c_y_bits={}\tselected_c_z_bits={}\trejection_reason={}\n",
            self.split.token(),
            self.cell.token(),
            self.episode_index,
            self.query_index,
            encode_identity(self.target_identity),
            encode_identity(self.selected_identity),
            u8::from(self.reconstruction_success),
            encode_component(target[0]),
            encode_component(target[1]),
            encode_component(target[2]),
            encode_component(target[3]),
            encode_component(target[4]),
            encode_component(target[5]),
            encode_component(selected[0]),
            encode_component(selected[1]),
            encode_component(selected[2]),
            encode_component(selected[3]),
            encode_component(selected[4]),
            encode_component(selected[5]),
            rejection,
        )
    }
}

fn evaluate_arm(
    split: Split,
    cell: PrimaryCell,
    episode_index: u64,
    generated: &GeneratedQuery,
    arm: Arm,
) -> Result<(EvalRecord, Option<T1ValueRecord>), RunnerError> {
    let resources = ResourceLedger::for_arm(arm, generated.candidates.len())?;
    let (target_score, best_distractor, target_margin) = target_metrics(arm, generated)?;
    let target_key = key_by_identity(&generated.candidates, generated.target_identity).ok_or(
        RunnerError::MissingTarget {
            identity: generated.target_identity,
        },
    )?;
    let target_value = TorsorValue::from(target_key);

    let ranking = rank_candidates(arm, generated.query, &generated.candidates);
    if arm == Arm::T1 && ranking == Err(EvalError::AmbiguousT1Top) {
        let base = EvalRecord {
            split,
            cell,
            episode_index,
            query_index: generated.query_index,
            arm,
            target_identity: Some(generated.target_identity),
            selected_identity: None,
            exact_success: false,
            target_score: Some(target_score),
            runner_up_score: Some(best_distractor),
            target_margin: Some(target_margin),
            rejection_reason: Some(RejectionReason::AmbiguousT1Top),
            resources,
        };
        let value = T1ValueRecord {
            split,
            cell,
            episode_index,
            query_index: generated.query_index,
            target_identity: Some(generated.target_identity),
            selected_identity: None,
            reconstruction_success: false,
            target_value: Some(target_value),
            selected_value: None,
            rejection_reason: Some(RejectionReason::AmbiguousT1Top),
        };
        return Ok((base, Some(value)));
    }

    let ranking = ranking?;
    let selected_key = key_by_identity(&generated.candidates, ranking.selected_identity).ok_or(
        RunnerError::MissingSelected {
            identity: ranking.selected_identity,
        },
    )?;
    let base = EvalRecord {
        split,
        cell,
        episode_index,
        query_index: generated.query_index,
        arm,
        target_identity: Some(generated.target_identity),
        selected_identity: Some(ranking.selected_identity),
        exact_success: ranking.selected_identity == generated.target_identity,
        target_score: Some(target_score),
        runner_up_score: Some(best_distractor),
        target_margin: Some(target_margin),
        rejection_reason: None,
        resources,
    };

    let companion = if arm == Arm::T1 {
        let selected_value = TorsorValue::from(selected_key);
        Some(T1ValueRecord {
            split,
            cell,
            episode_index,
            query_index: generated.query_index,
            target_identity: Some(generated.target_identity),
            selected_identity: Some(ranking.selected_identity),
            reconstruction_success: value_close(selected_value, target_value),
            target_value: Some(target_value),
            selected_value: Some(selected_value),
            rejection_reason: None,
        })
    } else {
        None
    };
    Ok((base, companion))
}

pub fn evaluate_episode(episode: &GeneratedEpisode) -> Result<EpisodeRecords, RunnerError> {
    let mut base_records = Vec::with_capacity(episode.queries.len() * ARMS.len());
    let mut t1_value_records = Vec::with_capacity(episode.queries.len());
    for generated in &episode.queries {
        for arm in ARMS {
            let (base, companion) =
                evaluate_arm(episode.split, episode.cell, episode.episode_index, generated, arm)?;
            base_records.push(base);
            if let Some(companion) = companion {
                t1_value_records.push(companion);
            }
        }
    }
    Ok(EpisodeRecords {
        base_records,
        t1_value_records,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::experimental::tdi22_generator::generate_episode;

    #[test]
    fn episode_materializes_exact_frozen_record_cardinality() {
        let episode = generate_episode(Split::Development, PrimaryCell::P1, 0).unwrap();
        let records = evaluate_episode(&episode).unwrap();
        assert_eq!(records.base_records.len(), 32);
        assert_eq!(records.t1_value_records.len(), 8);
        assert!(records
            .base_records
            .iter()
            .all(|record| record.encode_line().trim_end().split('\t').count() == 25));
        assert!(records
            .t1_value_records
            .iter()
            .all(|record| record.encode_line().trim_end().split('\t').count() == 21));
    }

    #[test]
    fn companion_join_keys_match_exactly_one_t1_base_record() {
        let episode = generate_episode(Split::Validation, PrimaryCell::P5, 0).unwrap();
        let records = evaluate_episode(&episode).unwrap();
        for companion in &records.t1_value_records {
            let matches = records
                .base_records
                .iter()
                .filter(|base| {
                    base.arm == Arm::T1
                        && base.query_index == companion.query_index
                        && base.episode_index == companion.episode_index
                        && base.cell == companion.cell
                        && base.split == companion.split
                })
                .count();
            assert_eq!(matches, 1);
        }
    }

    #[test]
    fn base_target_margin_is_target_minus_best_distractor() {
        let episode = generate_episode(Split::Development, PrimaryCell::P3, 0).unwrap();
        let records = evaluate_episode(&episode).unwrap();
        for record in &records.base_records {
            let target = record.target_score.unwrap();
            let distractor = record.runner_up_score.unwrap();
            assert_eq!(record.target_margin.unwrap(), target - distractor);
        }
    }

    #[test]
    fn t1_companion_is_byte_deterministic_on_replay() {
        let episode = generate_episode(Split::Development, PrimaryCell::P2, 1).unwrap();
        let first = evaluate_episode(&episode).unwrap();
        let second = evaluate_episode(&episode).unwrap();
        let first_lines: Vec<_> = first.t1_value_records.iter().map(T1ValueRecord::encode_line).collect();
        let second_lines: Vec<_> = second.t1_value_records.iter().map(T1ValueRecord::encode_line).collect();
        assert_eq!(first_lines, second_lines);
    }
}
