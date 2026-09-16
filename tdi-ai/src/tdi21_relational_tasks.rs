//! Typed deterministic relational tasks for TDI-21 development.
//!
//! Expected answers remain evaluator-side. Candidate execution receives only
//! explicit bindings and a bounded relation path query. Development and
//! Validation use disjoint identifier namespaces while preserving task topology.

use super::tdi21_relational_binding::{
    RelationalBinder, RelationalConfig, RelationalError, RelationalRead,
};
use super::tdi21_stream::StreamCounters;

pub const RELATIONAL_TASK_SEMANTICS: &str = "tdi21-relational-task-family-v1";
pub const RELATIONAL_EPISODES_PER_SPLIT: usize = 4;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RelationalSplit {
    Development,
    Validation,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RelationalFact {
    pub relation: u64,
    pub subject: u64,
    pub object: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RelationalQuery {
    pub relations: Vec<u64>,
    pub subject: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RelationalEpisode {
    pub split: RelationalSplit,
    pub case_id: u8,
    pub facts: Vec<RelationalFact>,
    pub query: RelationalQuery,
    expected: RelationalRead,
}

impl RelationalEpisode {
    #[must_use]
    pub const fn expected(&self) -> RelationalRead {
        self.expected
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DevelopmentRelationalSet(Vec<RelationalEpisode>);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ValidationRelationalSet(Vec<RelationalEpisode>);

impl DevelopmentRelationalSet {
    #[must_use]
    pub fn v1() -> Self {
        Self(build_split(RelationalSplit::Development))
    }

    #[must_use]
    pub fn episodes(&self) -> &[RelationalEpisode] {
        &self.0
    }
}

impl ValidationRelationalSet {
    #[must_use]
    pub fn v1() -> Self {
        Self(build_split(RelationalSplit::Validation))
    }

    #[must_use]
    pub fn episodes(&self) -> &[RelationalEpisode] {
        &self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RelationalEpisodeOutcome {
    pub observed: RelationalRead,
    pub expected: RelationalRead,
    pub correct: bool,
    pub counters: StreamCounters,
}

fn namespace(split: RelationalSplit) -> ([u64; 6], [u64; 4]) {
    match split {
        RelationalSplit::Development => ([3, 5, 9, 13, 17, 21], [1, 2, 4, 7]),
        RelationalSplit::Validation => ([131, 149, 167, 181, 197, 211], [31, 37, 41, 43]),
    }
}

fn fact(relation: u64, subject: u64, object: u64) -> RelationalFact {
    RelationalFact {
        relation,
        subject,
        object,
    }
}

fn episode(
    split: RelationalSplit,
    case_id: u8,
    facts: Vec<RelationalFact>,
    relations: Vec<u64>,
    subject: u64,
    expected: u64,
) -> RelationalEpisode {
    RelationalEpisode {
        split,
        case_id,
        facts,
        query: RelationalQuery { relations, subject },
        expected: RelationalRead::Hit(expected),
    }
}

fn build_split(split: RelationalSplit) -> Vec<RelationalEpisode> {
    let (e, r) = namespace(split);
    vec![
        episode(split, 0, vec![fact(r[0], e[0], e[1])], vec![r[0]], e[0], e[1]),
        episode(
            split,
            1,
            vec![fact(r[0], e[0], e[1]), fact(r[1], e[1], e[2])],
            vec![r[0], r[1]],
            e[0],
            e[2],
        ),
        episode(
            split,
            2,
            vec![
                fact(r[0], e[0], e[1]),
                fact(r[1], e[1], e[2]),
                fact(r[2], e[2], e[3]),
            ],
            vec![r[0], r[1], r[2]],
            e[0],
            e[3],
        ),
        episode(
            split,
            3,
            vec![
                fact(r[0], e[0], e[1]),
                fact(r[1], e[1], e[2]),
                fact(r[3], e[4], e[5]),
                fact(r[0], e[5], e[4]),
            ],
            vec![r[0], r[1]],
            e[0],
            e[2],
        ),
    ]
}

pub fn evaluate_relational_episode(
    config: RelationalConfig,
    episode: &RelationalEpisode,
) -> Result<RelationalEpisodeOutcome, RelationalError> {
    let mut binder = RelationalBinder::new(config)?;
    for fact in &episode.facts {
        binder.bind(fact.relation, fact.subject, fact.object)?;
    }
    let observed = binder.compose_path(&episode.query.relations, episode.query.subject)?;
    Ok(RelationalEpisodeOutcome {
        observed,
        expected: episode.expected,
        correct: observed == episode.expected,
        counters: binder.counters(),
    })
}

/// Compare only what the candidate can observe. Split metadata, case IDs and
/// evaluator-owned expected answers are deliberately excluded.
fn same_candidate_input(left: &RelationalEpisode, right: &RelationalEpisode) -> bool {
    left.facts == right.facts && left.query == right.query
}

#[must_use]
pub fn exact_split_overlap(
    development: &DevelopmentRelationalSet,
    validation: &ValidationRelationalSet,
) -> bool {
    development.episodes().iter().any(|left| {
        validation
            .episodes()
            .iter()
            .any(|right| same_candidate_input(left, right))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn overlap_detection_ignores_all_evaluator_metadata() {
        let development = DevelopmentRelationalSet::v1();
        let mut copied = development.episodes()[0].clone();
        copied.split = RelationalSplit::Validation;
        copied.case_id = 99;
        copied.expected = RelationalRead::Miss;
        let validation = ValidationRelationalSet(vec![copied]);
        assert!(exact_split_overlap(&development, &validation));
    }
}
