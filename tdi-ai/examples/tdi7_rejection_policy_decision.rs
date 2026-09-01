//! TDI-7.2 final-holdout rejection-policy decision validator.
//!
//! The frozen TDI-7.0 protocol requires rejection counts/reasons but the
//! current TDI-7.0/TDI-7.1 surfaces do not define the exact set of rejectable
//! final-record conditions. This validator keeps that policy unresolved and
//! deliberately accepts no FROZEN policy until a reviewed code change defines
//! machine-recognized reason codes and their applicability conditions.

use std::collections::BTreeMap;
use std::fs;

const DECISION_PATH: &str = "docs/TDI-7.2-FINAL-HOLDOUT-REJECTION-POLICY.toml";
const SCHEMA_VERSION: u64 = 1;
const PREARM_MAIN_COMMIT: &str = "c98e73d52e1f23f315ca75b9303b89580b14fc45";
const NOT_AUTHORIZED: &str = "NOT_AUTHORIZED";
const UNRESOLVED: &str = "UNRESOLVED";

const ALLOWED_KEYS: [&str; 5] = [
    "schema_version",
    "policy_status",
    "prearm_main_commit",
    "authorization_state",
    "policy_reference",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PolicyStatus {
    Unresolved,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct RejectionPolicyDecision {
    schema_version: u64,
    status: PolicyStatus,
    prearm_main_commit: String,
    authorization_state: String,
    policy_reference: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PolicyError {
    Io,
    MalformedLine,
    DuplicateKey,
    UnknownKey,
    MissingKey,
    InvalidInteger,
    InvalidString,
    InvalidSchema,
    InvalidStatus,
    FrozenPolicyRequiresReviewedValidatorUpdate,
    InvalidPrearmCommit,
    AuthorizationPresent,
    InvalidPolicyReference,
}

fn parse_map(input: &str) -> Result<BTreeMap<String, String>, PolicyError> {
    let mut values = BTreeMap::new();
    for line in input.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let (key, value) = line.split_once('=').ok_or(PolicyError::MalformedLine)?;
        let key = key.trim();
        let value = value.trim();
        if key.is_empty() || value.is_empty() {
            return Err(PolicyError::MalformedLine);
        }
        if !ALLOWED_KEYS.contains(&key) {
            return Err(PolicyError::UnknownKey);
        }
        if values.insert(key.to_string(), value.to_string()).is_some() {
            return Err(PolicyError::DuplicateKey);
        }
    }
    Ok(values)
}

fn required<'a>(values: &'a BTreeMap<String, String>, key: &str) -> Result<&'a str, PolicyError> {
    values
        .get(key)
        .map(String::as_str)
        .ok_or(PolicyError::MissingKey)
}

fn parse_integer(raw: &str) -> Result<u64, PolicyError> {
    raw.parse::<u64>().map_err(|_| PolicyError::InvalidInteger)
}

fn parse_string(raw: &str) -> Result<String, PolicyError> {
    if raw.len() < 2 || !raw.starts_with('"') || !raw.ends_with('"') {
        return Err(PolicyError::InvalidString);
    }
    let inner = &raw[1..raw.len() - 1];
    if inner.contains('"') || inner.contains('\n') || inner.contains('\r') {
        return Err(PolicyError::InvalidString);
    }
    Ok(inner.to_string())
}

impl RejectionPolicyDecision {
    fn parse(input: &str) -> Result<Self, PolicyError> {
        let values = parse_map(input)?;
        let status_raw = parse_string(required(&values, "policy_status")?)?;
        let status = match status_raw.as_str() {
            UNRESOLVED => PolicyStatus::Unresolved,
            "FROZEN" => return Err(PolicyError::FrozenPolicyRequiresReviewedValidatorUpdate),
            _ => return Err(PolicyError::InvalidStatus),
        };
        let decision = Self {
            schema_version: parse_integer(required(&values, "schema_version")?)?,
            status,
            prearm_main_commit: parse_string(required(&values, "prearm_main_commit")?)?,
            authorization_state: parse_string(required(&values, "authorization_state")?)?,
            policy_reference: parse_string(required(&values, "policy_reference")?)?,
        };
        decision.validate()?;
        Ok(decision)
    }

    fn validate(&self) -> Result<(), PolicyError> {
        if self.schema_version != SCHEMA_VERSION {
            return Err(PolicyError::InvalidSchema);
        }
        if self.prearm_main_commit != PREARM_MAIN_COMMIT
            || self.prearm_main_commit.len() != 40
            || !self
                .prearm_main_commit
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit())
        {
            return Err(PolicyError::InvalidPrearmCommit);
        }
        if self.authorization_state != NOT_AUTHORIZED {
            return Err(PolicyError::AuthorizationPresent);
        }
        if self.status == PolicyStatus::Unresolved && self.policy_reference != UNRESOLVED {
            return Err(PolicyError::InvalidPolicyReference);
        }
        Ok(())
    }
}

fn main() {
    let require_frozen = match std::env::args().skip(1).collect::<Vec<_>>().as_slice() {
        [] => false,
        [flag] if flag == "--require-frozen" => true,
        _ => {
            eprintln!("usage: tdi7_rejection_policy_decision [--require-frozen]");
            std::process::exit(2);
        }
    };

    let input = match fs::read_to_string(DECISION_PATH) {
        Ok(value) => value,
        Err(_) => {
            eprintln!("TDI-7.2 rejection-policy error: {:?}", PolicyError::Io);
            std::process::exit(2);
        }
    };
    let decision = match RejectionPolicyDecision::parse(&input) {
        Ok(value) => value,
        Err(error) => {
            eprintln!("TDI-7.2 rejection-policy error: {error:?}");
            std::process::exit(2);
        }
    };

    println!("TDI-7.2 rejection-policy decision: VALID");
    println!("policy_status=UNRESOLVED");
    println!("prearm_main_commit={}", decision.prearm_main_commit);
    println!("authorization_state={}", decision.authorization_state);
    println!("final_holdout_accessed=false");
    println!("arming_allowed=false");

    if require_frozen {
        eprintln!("BLOCKED: final holdout rejection policy is UNRESOLVED");
        std::process::exit(3);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unresolved() -> String {
        format!(
            "schema_version = 1\n\
             policy_status = \"UNRESOLVED\"\n\
             prearm_main_commit = \"{PREARM_MAIN_COMMIT}\"\n\
             authorization_state = \"NOT_AUTHORIZED\"\n\
             policy_reference = \"UNRESOLVED\"\n"
        )
    }

    #[test]
    fn unresolved_policy_is_valid_and_not_authorized() {
        let decision = RejectionPolicyDecision::parse(&unresolved()).unwrap();
        assert_eq!(decision.status, PolicyStatus::Unresolved);
        assert_eq!(decision.authorization_state, NOT_AUTHORIZED);
    }

    #[test]
    fn frozen_policy_cannot_be_smuggled_without_reviewed_validator_change() {
        let input = unresolved().replace(
            "policy_status = \"UNRESOLVED\"",
            "policy_status = \"FROZEN\"",
        );
        assert_eq!(
            RejectionPolicyDecision::parse(&input),
            Err(PolicyError::FrozenPolicyRequiresReviewedValidatorUpdate)
        );
    }

    #[test]
    fn arbitrary_reason_codes_are_rejected_until_policy_is_reviewed() {
        let input = format!("{}allowed_reason = \"numerical_failure\"\n", unresolved());
        assert_eq!(
            RejectionPolicyDecision::parse(&input),
            Err(PolicyError::UnknownKey)
        );
    }

    #[test]
    fn wrong_commit_authorization_or_reference_fails_closed() {
        let input = unresolved().replace(PREARM_MAIN_COMMIT, &"0".repeat(40));
        assert_eq!(
            RejectionPolicyDecision::parse(&input),
            Err(PolicyError::InvalidPrearmCommit)
        );

        let input = unresolved().replace("NOT_AUTHORIZED", "AUTHORIZED");
        assert_eq!(
            RejectionPolicyDecision::parse(&input),
            Err(PolicyError::AuthorizationPresent)
        );

        let input = unresolved().replace(
            "policy_reference = \"UNRESOLVED\"",
            "policy_reference = \"reviewed-pr-000\"",
        );
        assert_eq!(
            RejectionPolicyDecision::parse(&input),
            Err(PolicyError::InvalidPolicyReference)
        );
    }

    #[test]
    fn source_contains_no_final_run_confirmation_secret() {
        let source = include_str!("tdi7_rejection_policy_decision.rs");
        let environment = ["TDI7_CONFIRM_FINAL_", "HOLDOUT"].concat();
        let confirmation = ["I_ACCEPT_THE_TDI7_", "HOLDOUT_FREEZE"].concat();
        assert!(!source.contains(&environment));
        assert!(!source.contains(&confirmation));
    }
}
