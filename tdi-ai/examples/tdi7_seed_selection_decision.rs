//! TDI-7.2 final-holdout seed-selection decision validator.
//!
//! TDI-7.0 froze the final seed range but did not freeze how a future
//! population count chooses concrete seeds from that range. The reviewed
//! validator change of 2026-09-01 accepts exactly one frozen rule:
//! `contiguous_ascending_v1`, a contiguous ascending enumeration that must
//! cover the entire frozen final seed range. Any other rule stays invalid.

use std::collections::BTreeMap;
use std::fs;

const DECISION_PATH: &str = "docs/TDI-7.2-FINAL-HOLDOUT-SELECTION.toml";
const SCHEMA_VERSION: u64 = 1;
const PREARM_MAIN_COMMIT: &str = "c98e73d52e1f23f315ca75b9303b89580b14fc45";
const FINAL_SEED_START: u64 = 7_100_030_000;
const FINAL_SEED_END: u64 = 7_100_039_999;
const NOT_AUTHORIZED: &str = "NOT_AUTHORIZED";
const UNRESOLVED: &str = "UNRESOLVED";
const FROZEN_RULE: &str = "contiguous_ascending_v1";

const ALLOWED_KEYS: [&str; 10] = [
    "schema_version",
    "selection_status",
    "prearm_main_commit",
    "final_seed_start",
    "final_seed_end",
    "selection_rule",
    "selection_start",
    "selection_count",
    "authorization_state",
    "selection_reference",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SelectionStatus {
    Unresolved,
    Frozen,
}

impl SelectionStatus {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Unresolved => "UNRESOLVED",
            Self::Frozen => "FROZEN",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct SelectionDecision {
    schema_version: u64,
    status: SelectionStatus,
    prearm_main_commit: String,
    final_seed_start: u64,
    final_seed_end: u64,
    selection_rule: Option<String>,
    selection_start: Option<u64>,
    selection_count: Option<u64>,
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
    MissingFrozenRuleFields,
    UnexpectedFrozenRuleFields,
    InvalidSelectionRule,
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
        let status = match parse_string(required(&values, "selection_status")?)?.as_str() {
            UNRESOLVED => SelectionStatus::Unresolved,
            "FROZEN" => SelectionStatus::Frozen,
            _ => return Err(SelectionError::InvalidStatus),
        };
        let decision = Self {
            schema_version,
            status,
            prearm_main_commit: parse_string(required(&values, "prearm_main_commit")?)?,
            final_seed_start: parse_integer(required(&values, "final_seed_start")?)?,
            final_seed_end: parse_integer(required(&values, "final_seed_end")?)?,
            selection_rule: values
                .get("selection_rule")
                .map(String::as_str)
                .map(parse_string)
                .transpose()?,
            selection_start: values
                .get("selection_start")
                .map(|raw| parse_integer(raw))
                .transpose()?,
            selection_count: values
                .get("selection_count")
                .map(|raw| parse_integer(raw))
                .transpose()?,
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
        let has_rule_fields = self.selection_rule.is_some()
            || self.selection_start.is_some()
            || self.selection_count.is_some();
        match self.status {
            SelectionStatus::Unresolved => {
                if has_rule_fields {
                    return Err(SelectionError::UnexpectedFrozenRuleFields);
                }
                if self.selection_reference != UNRESOLVED {
                    return Err(SelectionError::InvalidSelectionReference);
                }
            }
            SelectionStatus::Frozen => {
                let rule = self
                    .selection_rule
                    .as_deref()
                    .ok_or(SelectionError::MissingFrozenRuleFields)?;
                let start = self
                    .selection_start
                    .ok_or(SelectionError::MissingFrozenRuleFields)?;
                let count = self
                    .selection_count
                    .ok_or(SelectionError::MissingFrozenRuleFields)?;
                if rule != FROZEN_RULE {
                    return Err(SelectionError::InvalidSelectionRule);
                }
                let capacity = self
                    .final_seed_end
                    .checked_sub(self.final_seed_start)
                    .and_then(|delta| delta.checked_add(1))
                    .ok_or(SelectionError::InvalidSeedRange)?;
                if count != capacity || start != self.final_seed_start {
                    return Err(SelectionError::InvalidSelectionRule);
                }
                if self.selection_reference.is_empty() || self.selection_reference == UNRESOLVED {
                    return Err(SelectionError::InvalidSelectionReference);
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
    println!("selection_status={}", decision.status.as_str());
    match (&decision.selection_rule, decision.selection_count) {
        (Some(rule), Some(count)) => println!("selection_rule={rule};selection_count={count}"),
        _ => println!("selection_rule=UNRESOLVED"),
    }
    println!("prearm_main_commit={}", decision.prearm_main_commit);
    println!("authorization_state={}", decision.authorization_state);
    println!("final_holdout_accessed=false");
    println!("arming_allowed=false");

    if require_frozen && decision.status != SelectionStatus::Frozen {
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

    fn frozen() -> String {
        format!(
            "schema_version = 1\n\
             selection_status = \"FROZEN\"\n\
             prearm_main_commit = \"{PREARM_MAIN_COMMIT}\"\n\
             final_seed_start = {FINAL_SEED_START}\n\
             final_seed_end = {FINAL_SEED_END}\n\
             selection_rule = \"contiguous_ascending_v1\"\n\
             selection_start = {FINAL_SEED_START}\n\
             selection_count = {}\n\
             authorization_state = \"NOT_AUTHORIZED\"\n\
             selection_reference = \"TDI-7.2-HUMAN-DECISION-2026-09-01-CHECKUPAUTO\"\n",
            FINAL_SEED_END - FINAL_SEED_START + 1
        )
    }

    #[test]
    fn unresolved_selection_is_valid_and_not_authorized() {
        let decision = SelectionDecision::parse(&unresolved()).unwrap();
        assert_eq!(decision.status, SelectionStatus::Unresolved);
        assert_eq!(decision.authorization_state, NOT_AUTHORIZED);
    }

    #[test]
    fn frozen_contiguous_ascending_rule_is_accepted() {
        let decision = SelectionDecision::parse(&frozen()).unwrap();
        assert_eq!(decision.status, SelectionStatus::Frozen);
        assert_eq!(decision.selection_rule.as_deref(), Some(FROZEN_RULE));
        assert_eq!(
            decision.selection_count,
            Some(FINAL_SEED_END - FINAL_SEED_START + 1)
        );
        assert_eq!(decision.authorization_state, NOT_AUTHORIZED);
    }

    #[test]
    fn frozen_without_rule_fields_fails_closed() {
        let input = unresolved().replace(
            "selection_status = \"UNRESOLVED\"",
            "selection_status = \"FROZEN\"",
        );
        assert_eq!(
            SelectionDecision::parse(&input),
            Err(SelectionError::MissingFrozenRuleFields)
        );
    }

    #[test]
    fn frozen_with_an_unreviewed_rule_fails_closed() {
        let input = frozen().replace(FROZEN_RULE, "first_n");
        assert_eq!(
            SelectionDecision::parse(&input),
            Err(SelectionError::InvalidSelectionRule)
        );
    }

    #[test]
    fn frozen_rule_must_cover_the_entire_frozen_range() {
        let input = frozen().replace(
            &format!(
                "selection_count = {}",
                FINAL_SEED_END - FINAL_SEED_START + 1
            ),
            "selection_count = 4000",
        );
        assert_eq!(
            SelectionDecision::parse(&input),
            Err(SelectionError::InvalidSelectionRule)
        );

        let input = frozen().replace(
            &format!("selection_start = {FINAL_SEED_START}"),
            &format!("selection_start = {}", FINAL_SEED_START + 1),
        );
        assert_eq!(
            SelectionDecision::parse(&input),
            Err(SelectionError::InvalidSelectionRule)
        );
    }

    #[test]
    fn unresolved_selection_must_not_carry_rule_fields() {
        let input = unresolved().replace(
            "selection_reference = \"UNRESOLVED\"",
            "selection_reference = \"UNRESOLVED\"\n\
             selection_rule = \"contiguous_ascending_v1\"\n\
             selection_start = 7100030000\n\
             selection_count = 10000",
        );
        assert_eq!(
            SelectionDecision::parse(&input),
            Err(SelectionError::UnexpectedFrozenRuleFields)
        );
    }

    #[test]
    fn arbitrary_selection_rule_key_is_rejected_when_unrecognized() {
        let input = format!("{}selection_rule_variant = \"first_n\"\n", unresolved());
        assert_eq!(
            SelectionDecision::parse(&input),
            Err(SelectionError::UnknownKey)
        );
    }

    #[test]
    fn wrong_range_or_commit_fails_closed() {
        let input = frozen().replace(PREARM_MAIN_COMMIT, &"0".repeat(40));
        assert_eq!(
            SelectionDecision::parse(&input),
            Err(SelectionError::InvalidPrearmCommit)
        );

        let input = frozen().replace(
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
        let input = frozen().replace("NOT_AUTHORIZED", "AUTHORIZED");
        assert_eq!(
            SelectionDecision::parse(&input),
            Err(SelectionError::AuthorizationPresent)
        );

        let input = frozen().replace(
            "selection_reference = \"TDI-7.2-HUMAN-DECISION-2026-09-01-CHECKUPAUTO\"",
            "selection_reference = \"UNRESOLVED\"",
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
