//! TDI-7.2 final-holdout seed-selection decision validator.
//!
//! TDI-7.0 froze the final seed range but did not freeze how a future
//! population count chooses concrete seeds from that range. This validator
//! keeps that missing choice explicit and deliberately has no accepted FROZEN
//! rule until a separately reviewed implementation defines one.

use std::collections::BTreeMap;
use std::fs;

const DECISION_PATH: &str = "docs/TDI-7.2-FINAL-HOLDOUT-SELECTION.toml";
const SCHEMA_VERSION: u64 = 1;
const PREARM_MAIN_COMMIT: &str = "c98e73d52e1f23f315ca75b9303b89580b14fc45";
const FINAL_SEED_START: u64 = 7_100_030_000;
const FINAL_SEED_END: u64 = 7_100_039_999;
const NOT_AUTHORIZED: &str = "NOT_AUTHORIZED";
const UNRESOLVED: &str = "UNRESOLVED";

const ALLOWED_KEYS: [&str; 7] = [
    "schema_version",
    "selection_status",
    "prearm_main_commit",
    "final_seed_start",
    "final_seed_end",
    "authorization_state",
    "selection_reference",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SelectionStatus {
    Unresolved,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct SelectionDecision {
    schema_version: u64,
    status: SelectionStatus,
    prearm_main_commit: String,
    final_seed_start: u64,
    final_seed_end: u64,
    authorization_state: String,
    selection_reference: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SelectionError {
    Io,
    MalformedLine,
    DuplicateKey,
    UnknownKey,
    MissingKey,
    InvalidInteger,
    InvalidString,
    InvalidSchema,
    InvalidStatus,
    FrozenRuleRequiresReviewedValidatorUpdate,
    InvalidPrearmCommit,
    InvalidSeedRange,
    AuthorizationPresent,
    InvalidSelectionReference,
}

fn parse_map(input: &str) -> Result<BTreeMap<String, String>, SelectionError> {
    let mut values = BTreeMap::new();
    for line in input.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let (key, value) = line.split_once('=').ok_or(SelectionError::MalformedLine)?;
        let key = key.trim();
        let value = value.trim();
        if key.is_empty() || value.is_empty() {
            return Err(SelectionError::MalformedLine);
        }
        if !ALLOWED_KEYS.contains(&key) {
            return Err(SelectionError::UnknownKey);
        }
        if values.insert(key.to_string(), value.to_string()).is_some() {
            return Err(SelectionError::DuplicateKey);
        }
    }
    Ok(values)
}

fn required<'a>(
    values: &'a BTreeMap<String, String>,
    key: &str,
) -> Result<&'a str, SelectionError> {
    values
        .get(key)
        .map(String::as_str)
        .ok_or(SelectionError::MissingKey)
}

fn parse_integer(raw: &str) -> Result<u64, SelectionError> {
    raw.parse::<u64>()
        .map_err(|_| SelectionError::InvalidInteger)
}

fn parse_string(raw: &str) -> Result<String, SelectionError> {
    if raw.len() < 2 || !raw.starts_with('"') || !raw.ends_with('"') {
        return Err(SelectionError::InvalidString);
    }
    let inner = &raw[1..raw.len() - 1];
    if inner.contains('"') || inner.contains('\n') || inner.contains('\r') {
        return Err(SelectionError::InvalidString);
    }
    Ok(inner.to_string())
}

impl SelectionDecision {
    fn parse(input: &str) -> Result<Self, SelectionError> {
        let values = parse_map(input)?;
        let schema_version = parse_integer(required(&values, "schema_version")?)?;
        let status_raw = parse_string(required(&values, "selection_status")?)?;
        let status = match status_raw.as_str() {
            UNRESOLVED => SelectionStatus::Unresolved,
            "FROZEN" => return Err(SelectionError::FrozenRuleRequiresReviewedValidatorUpdate),
            _ => return Err(SelectionError::InvalidStatus),
        };
        let decision = Self {
            schema_version,
            status,
            prearm_main_commit: parse_string(required(&values, "prearm_main_commit")?)?,
            final_seed_start: parse_integer(required(&values, "final_seed_start")?)?,
            final_seed_end: parse_integer(required(&values, "final_seed_end")?)?,
            authorization_state: parse_string(required(&values, "authorization_state")?)?,
            selection_reference: parse_string(required(&values, "selection_reference")?)?,
        };
        decision.validate()?;
        Ok(decision)
    }

    fn validate(&self) -> Result<(), SelectionError> {
        if self.schema_version != SCHEMA_VERSION {
            return Err(SelectionError::InvalidSchema);
        }
        if self.prearm_main_commit != PREARM_MAIN_COMMIT
            || self.prearm_main_commit.len() != 40
            || !self
                .prearm_main_commit
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit())
        {
            return Err(SelectionError::InvalidPrearmCommit);
        }
        if self.final_seed_start != FINAL_SEED_START || self.final_seed_end != FINAL_SEED_END {
            return Err(SelectionError::InvalidSeedRange);
        }
        if self.authorization_state != NOT_AUTHORIZED {
            return Err(SelectionError::AuthorizationPresent);
        }
        if self.status == SelectionStatus::Unresolved && self.selection_reference != UNRESOLVED {
            return Err(SelectionError::InvalidSelectionReference);
        }
        Ok(())
    }
}

fn main() {
    let require_frozen = match std::env::args().skip(1).collect::<Vec<_>>().as_slice() {
        [] => false,
        [flag] if flag == "--require-frozen" => true,
        _ => {
            eprintln!("usage: tdi7_seed_selection_decision [--require-frozen]");
            std::process::exit(2);
        }
    };

    let input = match fs::read_to_string(DECISION_PATH) {
        Ok(value) => value,
        Err(_) => {
            eprintln!("TDI-7.2 seed-selection error: {:?}", SelectionError::Io);
            std::process::exit(2);
        }
    };
    let decision = match SelectionDecision::parse(&input) {
        Ok(value) => value,
        Err(error) => {
            eprintln!("TDI-7.2 seed-selection error: {error:?}");
            std::process::exit(2);
        }
    };

    println!("TDI-7.2 seed-selection decision: VALID");
    println!("selection_status=UNRESOLVED");
    println!("prearm_main_commit={}", decision.prearm_main_commit);
    println!("authorization_state={}", decision.authorization_state);
    println!("final_holdout_accessed=false");
    println!("arming_allowed=false");

    if require_frozen {
        eprintln!("BLOCKED: final holdout seed-selection rule is UNRESOLVED");
        std::process::exit(3);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unresolved() -> String {
        format!(
            "schema_version = 1\n\
             selection_status = \"UNRESOLVED\"\n\
             prearm_main_commit = \"{PREARM_MAIN_COMMIT}\"\n\
             final_seed_start = {FINAL_SEED_START}\n\
             final_seed_end = {FINAL_SEED_END}\n\
             authorization_state = \"NOT_AUTHORIZED\"\n\
             selection_reference = \"UNRESOLVED\"\n"
        )
    }

    #[test]
    fn unresolved_selection_is_valid_and_not_authorized() {
        let decision = SelectionDecision::parse(&unresolved()).unwrap();
        assert_eq!(decision.status, SelectionStatus::Unresolved);
        assert_eq!(decision.authorization_state, NOT_AUTHORIZED);
    }

    #[test]
    fn frozen_status_cannot_be_smuggled_without_reviewed_validator_change() {
        let input = unresolved().replace(
            "selection_status = \"UNRESOLVED\"",
            "selection_status = \"FROZEN\"",
        );
        assert_eq!(
            SelectionDecision::parse(&input),
            Err(SelectionError::FrozenRuleRequiresReviewedValidatorUpdate)
        );
    }

    #[test]
    fn arbitrary_selection_rule_key_is_rejected() {
        let input = format!("{}selection_rule = \"first_n\"\n", unresolved());
        assert_eq!(
            SelectionDecision::parse(&input),
            Err(SelectionError::UnknownKey)
        );
    }

    #[test]
    fn wrong_range_or_commit_fails_closed() {
        let input = unresolved().replace(PREARM_MAIN_COMMIT, &"0".repeat(40));
        assert_eq!(
            SelectionDecision::parse(&input),
            Err(SelectionError::InvalidPrearmCommit)
        );

        let input = unresolved().replace(
            &format!("final_seed_end = {FINAL_SEED_END}"),
            "final_seed_end = 7100040000",
        );
        assert_eq!(
            SelectionDecision::parse(&input),
            Err(SelectionError::InvalidSeedRange)
        );
    }

    #[test]
    fn authorization_and_reference_fail_closed() {
        let input = unresolved().replace("NOT_AUTHORIZED", "AUTHORIZED");
        assert_eq!(
            SelectionDecision::parse(&input),
            Err(SelectionError::AuthorizationPresent)
        );

        let input = unresolved().replace(
            "selection_reference = \"UNRESOLVED\"",
            "selection_reference = \"reviewed-pr-000\"",
        );
        assert_eq!(
            SelectionDecision::parse(&input),
            Err(SelectionError::InvalidSelectionReference)
        );
    }

    #[test]
    fn source_contains_no_final_run_confirmation_secret() {
        let source = include_str!("tdi7_seed_selection_decision.rs");
        let environment = ["TDI7_CONFIRM_FINAL_", "HOLDOUT"].concat();
        let confirmation = ["I_ACCEPT_THE_TDI7_", "HOLDOUT_FREEZE"].concat();
        assert!(!source.contains(&environment));
        assert!(!source.contains(&confirmation));
    }
}
