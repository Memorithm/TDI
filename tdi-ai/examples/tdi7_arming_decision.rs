//! TDI-7.2 final-holdout population decision validator.
//!
//! This example validates only the pre-arm decision record. It cannot generate,
//! read, fit, classify, or report final-holdout data and it never authorizes a
//! final run.

use std::collections::BTreeMap;
use std::fs;

const DECISION_PATH: &str = "docs/TDI-7.2-FINAL-HOLDOUT-DECISION.toml";
const SCHEMA_VERSION: u64 = 1;
const PREARM_MAIN_COMMIT: &str = "c98e73d52e1f23f315ca75b9303b89580b14fc45";
const FINAL_SEED_START: u64 = 7_100_030_000;
const FINAL_SEED_END: u64 = 7_100_039_999;
const NOT_AUTHORIZED: &str = "NOT_AUTHORIZED";
const UNRESOLVED_REFERENCE: &str = "UNRESOLVED";

const ALLOWED_KEYS: [&str; 8] = [
    "schema_version",
    "decision_status",
    "prearm_main_commit",
    "final_seed_start",
    "final_seed_end",
    "authorization_state",
    "decision_reference",
    "final_holdout_generator_count",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DecisionStatus {
    Unresolved,
    Frozen,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Decision {
    schema_version: u64,
    status: DecisionStatus,
    prearm_main_commit: String,
    final_seed_start: u64,
    final_seed_end: u64,
    authorization_state: String,
    decision_reference: String,
    final_holdout_generator_count: Option<u64>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DecisionError {
    Io,
    MalformedLine,
    DuplicateKey,
    UnknownKey,
    MissingKey,
    InvalidInteger,
    InvalidString,
    InvalidSchema,
    InvalidStatus,
    InvalidPrearmCommit,
    InvalidSeedRange,
    AuthorizationPresent,
    UnexpectedCount,
    MissingCount,
    CountOutOfRange,
    InvalidDecisionReference,
}

fn parse_map(input: &str) -> Result<BTreeMap<String, String>, DecisionError> {
    let mut values = BTreeMap::new();
    for line in input.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let (key, value) = line
            .split_once('=')
            .ok_or(DecisionError::MalformedLine)?;
        let key = key.trim();
        let value = value.trim();
        if key.is_empty() || value.is_empty() {
            return Err(DecisionError::MalformedLine);
        }
        if !ALLOWED_KEYS.contains(&key) {
            return Err(DecisionError::UnknownKey);
        }
        if values.insert(key.to_string(), value.to_string()).is_some() {
            return Err(DecisionError::DuplicateKey);
        }
    }
    Ok(values)
}

fn required<'a>(
    values: &'a BTreeMap<String, String>,
    key: &str,
) -> Result<&'a str, DecisionError> {
    values
        .get(key)
        .map(String::as_str)
        .ok_or(DecisionError::MissingKey)
}

fn parse_integer(raw: &str) -> Result<u64, DecisionError> {
    raw.parse::<u64>().map_err(|_| DecisionError::InvalidInteger)
}

fn parse_string(raw: &str) -> Result<String, DecisionError> {
    if raw.len() < 2 || !raw.starts_with('"') || !raw.ends_with('"') {
        return Err(DecisionError::InvalidString);
    }
    let inner = &raw[1..raw.len() - 1];
    if inner.contains('"') || inner.contains('\n') || inner.contains('\r') {
        return Err(DecisionError::InvalidString);
    }
    Ok(inner.to_string())
}

impl Decision {
    fn parse(input: &str) -> Result<Self, DecisionError> {
        let values = parse_map(input)?;
        let schema_version = parse_integer(required(&values, "schema_version")?)?;
        let status = match parse_string(required(&values, "decision_status")?)?.as_str() {
            "UNRESOLVED" => DecisionStatus::Unresolved,
            "FROZEN" => DecisionStatus::Frozen,
            _ => return Err(DecisionError::InvalidStatus),
        };
        let prearm_main_commit = parse_string(required(&values, "prearm_main_commit")?)?;
        let final_seed_start = parse_integer(required(&values, "final_seed_start")?)?;
        let final_seed_end = parse_integer(required(&values, "final_seed_end")?)?;
        let authorization_state = parse_string(required(&values, "authorization_state")?)?;
        let decision_reference = parse_string(required(&values, "decision_reference")?)?;
        let final_holdout_generator_count = values
            .get("final_holdout_generator_count")
            .map(|raw| parse_integer(raw))
            .transpose()?;

        let decision = Self {
            schema_version,
            status,
            prearm_main_commit,
            final_seed_start,
            final_seed_end,
            authorization_state,
            decision_reference,
            final_holdout_generator_count,
        };
        decision.validate()?;
        Ok(decision)
    }

    fn validate(&self) -> Result<(), DecisionError> {
        if self.schema_version != SCHEMA_VERSION {
            return Err(DecisionError::InvalidSchema);
        }
        if self.prearm_main_commit != PREARM_MAIN_COMMIT
            || self.prearm_main_commit.len() != 40
            || !self
                .prearm_main_commit
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit())
        {
            return Err(DecisionError::InvalidPrearmCommit);
        }
        if self.final_seed_start != FINAL_SEED_START || self.final_seed_end != FINAL_SEED_END {
            return Err(DecisionError::InvalidSeedRange);
        }
        if self.authorization_state != NOT_AUTHORIZED {
            return Err(DecisionError::AuthorizationPresent);
        }

        match self.status {
            DecisionStatus::Unresolved => {
                if self.final_holdout_generator_count.is_some() {
                    return Err(DecisionError::UnexpectedCount);
                }
                if self.decision_reference != UNRESOLVED_REFERENCE {
                    return Err(DecisionError::InvalidDecisionReference);
                }
            }
            DecisionStatus::Frozen => {
                let count = self
                    .final_holdout_generator_count
                    .ok_or(DecisionError::MissingCount)?;
                let capacity = self
                    .final_seed_end
                    .checked_sub(self.final_seed_start)
                    .and_then(|delta| delta.checked_add(1))
                    .ok_or(DecisionError::InvalidSeedRange)?;
                if count == 0 || count > capacity {
                    return Err(DecisionError::CountOutOfRange);
                }
                if self.decision_reference.is_empty()
                    || self.decision_reference == UNRESOLVED_REFERENCE
                {
                    return Err(DecisionError::InvalidDecisionReference);
                }
            }
        }
        Ok(())
    }
}

fn main() {
    let require_frozen = match std::env::args().skip(1).collect::<Vec<_>>().as_slice() {
        [] => false,
        [flag] if flag == "--require-frozen" => true,
        _ => {
            eprintln!("usage: tdi7_arming_decision [--require-frozen]");
            std::process::exit(2);
        }
    };

    let input = match fs::read_to_string(DECISION_PATH) {
        Ok(value) => value,
        Err(_) => {
            eprintln!("TDI-7.2 decision error: {:?}", DecisionError::Io);
            std::process::exit(2);
        }
    };
    let decision = match Decision::parse(&input) {
        Ok(value) => value,
        Err(error) => {
            eprintln!("TDI-7.2 decision error: {error:?}");
            std::process::exit(2);
        }
    };

    println!("TDI-7.2 final-population decision: VALID");
    println!("decision_status={:?}", decision.status);
    println!("prearm_main_commit={}", decision.prearm_main_commit);
    println!("authorization_state={}", decision.authorization_state);
    println!("final_holdout_accessed=false");

    if require_frozen && decision.status != DecisionStatus::Frozen {
        eprintln!("BLOCKED: final holdout generator count decision is UNRESOLVED");
        std::process::exit(3);
    }

    match decision.final_holdout_generator_count {
        Some(count) => println!("final_holdout_generator_count={count}"),
        None => println!("final_holdout_generator_count=UNRESOLVED"),
    }
    println!("arming_allowed=false");
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unresolved() -> String {
        format!(
            "schema_version = 1\n\
             decision_status = \"UNRESOLVED\"\n\
             prearm_main_commit = \"{PREARM_MAIN_COMMIT}\"\n\
             final_seed_start = {FINAL_SEED_START}\n\
             final_seed_end = {FINAL_SEED_END}\n\
             authorization_state = \"NOT_AUTHORIZED\"\n\
             decision_reference = \"UNRESOLVED\"\n"
        )
    }

    fn frozen(count: u64, reference: &str) -> String {
        format!(
            "schema_version = 1\n\
             decision_status = \"FROZEN\"\n\
             prearm_main_commit = \"{PREARM_MAIN_COMMIT}\"\n\
             final_seed_start = {FINAL_SEED_START}\n\
             final_seed_end = {FINAL_SEED_END}\n\
             authorization_state = \"NOT_AUTHORIZED\"\n\
             decision_reference = \"{reference}\"\n\
             final_holdout_generator_count = {count}\n"
        )
    }

    #[test]
    fn unresolved_record_is_valid_but_has_no_count() {
        let decision = Decision::parse(&unresolved()).unwrap();
        assert_eq!(decision.status, DecisionStatus::Unresolved);
        assert_eq!(decision.final_holdout_generator_count, None);
        assert_eq!(decision.authorization_state, NOT_AUTHORIZED);
    }

    #[test]
    fn unresolved_record_rejects_a_silent_count() {
        let input = format!("{}final_holdout_generator_count = 48\n", unresolved());
        assert_eq!(Decision::parse(&input), Err(DecisionError::UnexpectedCount));
    }

    #[test]
    fn frozen_record_requires_review_reference_and_count() {
        let decision = Decision::parse(&frozen(48, "reviewed-pr-000")).unwrap();
        assert_eq!(decision.status, DecisionStatus::Frozen);
        assert_eq!(decision.final_holdout_generator_count, Some(48));

        let missing_count = frozen(48, "reviewed-pr-000")
            .lines()
            .filter(|line| !line.starts_with("final_holdout_generator_count"))
            .collect::<Vec<_>>()
            .join("\n");
        assert_eq!(Decision::parse(&missing_count), Err(DecisionError::MissingCount));
        assert_eq!(
            Decision::parse(&frozen(48, UNRESOLVED_REFERENCE)),
            Err(DecisionError::InvalidDecisionReference)
        );
    }

    #[test]
    fn frozen_count_cannot_exceed_frozen_seed_capacity() {
        assert_eq!(
            Decision::parse(&frozen(10_001, "reviewed-pr-000")),
            Err(DecisionError::CountOutOfRange)
        );
        assert_eq!(
            Decision::parse(&frozen(0, "reviewed-pr-000")),
            Err(DecisionError::CountOutOfRange)
        );
    }

    #[test]
    fn authorization_cannot_be_smuggled_into_decision_record() {
        let input = unresolved().replace("NOT_AUTHORIZED", "AUTHORIZED");
        assert_eq!(
            Decision::parse(&input),
            Err(DecisionError::AuthorizationPresent)
        );
    }

    #[test]
    fn wrong_prearm_commit_or_seed_range_fails_closed() {
        let input = unresolved().replace(PREARM_MAIN_COMMIT, &"0".repeat(40));
        assert_eq!(
            Decision::parse(&input),
            Err(DecisionError::InvalidPrearmCommit)
        );

        let input = unresolved().replace(
            &format!("final_seed_start = {FINAL_SEED_START}"),
            "final_seed_start = 1",
        );
        assert_eq!(Decision::parse(&input), Err(DecisionError::InvalidSeedRange));
    }

    #[test]
    fn unknown_and_duplicate_keys_fail_closed() {
        let input = format!("{}surprise = \"value\"\n", unresolved());
        assert_eq!(Decision::parse(&input), Err(DecisionError::UnknownKey));

        let input = format!("{}schema_version = 1\n", unresolved());
        assert_eq!(Decision::parse(&input), Err(DecisionError::DuplicateKey));
    }

    #[test]
    fn source_contains_no_final_run_confirmation_secret() {
        let source = include_str!("tdi7_arming_decision.rs");
        let environment = ["TDI7_CONFIRM_FINAL_", "HOLDOUT"].concat();
        let confirmation = ["I_ACCEPT_THE_TDI7_", "HOLDOUT_FREEZE"].concat();
        assert!(!source.contains(&environment));
        assert!(!source.contains(&confirmation));
    }
}
