//! TDI-7.2 final-holdout rejection-policy decision validator.
//!
//! The frozen TDI-7.0 protocol requires rejection counts/reasons and forbids
//! result-driven exclusions. The reviewed validator change of 2026-09-01
//! accepts exactly one frozen policy: `frozen_tdi71_typed_errors_v1`, whose
//! machine-recognized reason codes mirror the frozen TDI-7.1 evaluator's typed
//! construction failures. Any other policy stays invalid.

use std::collections::BTreeMap;
use std::fs;

const DECISION_PATH: &str = "docs/TDI-7.2-FINAL-HOLDOUT-REJECTION-POLICY.toml";
const SCHEMA_VERSION: u64 = 1;
const PREARM_MAIN_COMMIT: &str = "c98e73d52e1f23f315ca75b9303b89580b14fc45";
const NOT_AUTHORIZED: &str = "NOT_AUTHORIZED";
const UNRESOLVED: &str = "UNRESOLVED";
const FROZEN_POLICY: &str = "frozen_tdi71_typed_errors_v1";
const FROZEN_REASONS: &str =
    "invalid_mixer,invalid_intervention,recovery_extraction_failed,non_finite_target";

const ALLOWED_KEYS: [&str; 7] = [
    "schema_version",
    "policy_status",
    "prearm_main_commit",
    "rejection_policy",
    "rejection_reasons",
    "authorization_state",
    "policy_reference",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PolicyStatus {
    Unresolved,
    Frozen,
}

impl PolicyStatus {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Unresolved => "UNRESOLVED",
            Self::Frozen => "FROZEN",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct RejectionPolicyDecision {
    schema_version: u64,
    status: PolicyStatus,
    prearm_main_commit: String,
    rejection_policy: Option<String>,
    rejection_reasons: Option<String>,
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
    MissingFrozenPolicyFields,
    UnexpectedFrozenPolicyFields,
    InvalidRejectionPolicy,
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
        let status = match parse_string(required(&values, "policy_status")?)?.as_str() {
            UNRESOLVED => PolicyStatus::Unresolved,
            "FROZEN" => PolicyStatus::Frozen,
            _ => return Err(PolicyError::InvalidStatus),
        };
        let decision = Self {
            schema_version: parse_integer(required(&values, "schema_version")?)?,
            status,
            prearm_main_commit: parse_string(required(&values, "prearm_main_commit")?)?,
            rejection_policy: values
                .get("rejection_policy")
                .map(String::as_str)
                .map(parse_string)
                .transpose()?,
            rejection_reasons: values
                .get("rejection_reasons")
                .map(String::as_str)
                .map(parse_string)
                .transpose()?,
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
        let has_policy_fields = self.rejection_policy.is_some() || self.rejection_reasons.is_some();
        match self.status {
            PolicyStatus::Unresolved => {
                if has_policy_fields {
                    return Err(PolicyError::UnexpectedFrozenPolicyFields);
                }
                if self.policy_reference != UNRESOLVED {
                    return Err(PolicyError::InvalidPolicyReference);
                }
            }
            PolicyStatus::Frozen => {
                let policy = self
                    .rejection_policy
                    .as_deref()
                    .ok_or(PolicyError::MissingFrozenPolicyFields)?;
                let reasons = self
                    .rejection_reasons
                    .as_deref()
                    .ok_or(PolicyError::MissingFrozenPolicyFields)?;
                if policy != FROZEN_POLICY {
                    return Err(PolicyError::InvalidRejectionPolicy);
                }
                if reasons != FROZEN_REASONS {
                    return Err(PolicyError::InvalidRejectionPolicy);
                }
                if self.policy_reference.is_empty() || self.policy_reference == UNRESOLVED {
                    return Err(PolicyError::InvalidPolicyReference);
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
    println!("policy_status={}", decision.status.as_str());
    match &decision.rejection_policy {
        Some(policy) => println!(
            "rejection_policy={policy};rejection_reasons={}",
            decision.rejection_reasons.as_deref().unwrap_or("MISSING")
        ),
        None => println!("rejection_policy=UNRESOLVED"),
    }
    println!("prearm_main_commit={}", decision.prearm_main_commit);
    println!("authorization_state={}", decision.authorization_state);
    println!("final_holdout_accessed=false");
    println!("arming_allowed=false");

    if require_frozen && decision.status != PolicyStatus::Frozen {
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

    fn frozen() -> String {
        format!(
            "schema_version = 1\n\
             policy_status = \"FROZEN\"\n\
             prearm_main_commit = \"{PREARM_MAIN_COMMIT}\"\n\
             rejection_policy = \"{FROZEN_POLICY}\"\n\
             rejection_reasons = \"{FROZEN_REASONS}\"\n\
             authorization_state = \"NOT_AUTHORIZED\"\n\
             policy_reference = \"TDI-7.2-HUMAN-DECISION-2026-09-01-CHECKUPAUTO\"\n"
        )
    }

    #[test]
    fn unresolved_policy_is_valid_and_not_authorized() {
        let decision = RejectionPolicyDecision::parse(&unresolved()).unwrap();
        assert_eq!(decision.status, PolicyStatus::Unresolved);
        assert_eq!(decision.authorization_state, NOT_AUTHORIZED);
    }

    #[test]
    fn frozen_tdi71_typed_error_policy_is_accepted() {
        let decision = RejectionPolicyDecision::parse(&frozen()).unwrap();
        assert_eq!(decision.status, PolicyStatus::Frozen);
        assert_eq!(decision.rejection_policy.as_deref(), Some(FROZEN_POLICY));
        assert_eq!(decision.rejection_reasons.as_deref(), Some(FROZEN_REASONS));
        assert_eq!(decision.authorization_state, NOT_AUTHORIZED);
    }

    #[test]
    fn frozen_without_policy_fields_fails_closed() {
        let input = unresolved().replace(
            "policy_status = \"UNRESOLVED\"",
            "policy_status = \"FROZEN\"",
        );
        assert_eq!(
            RejectionPolicyDecision::parse(&input),
            Err(PolicyError::MissingFrozenPolicyFields)
        );
    }

    #[test]
    fn frozen_with_an_unreviewed_policy_fails_closed() {
        let input = frozen().replace(FROZEN_POLICY, "result_driven_v1");
        assert_eq!(
            RejectionPolicyDecision::parse(&input),
            Err(PolicyError::InvalidRejectionPolicy)
        );
    }

    #[test]
    fn frozen_reason_taxonomy_must_match_exactly() {
        let input = frozen().replace(
            FROZEN_REASONS,
            "invalid_mixer,invalid_intervention,recovery_extraction_failed",
        );
        assert_eq!(
            RejectionPolicyDecision::parse(&input),
            Err(PolicyError::InvalidRejectionPolicy)
        );

        let input = frozen().replace(
            FROZEN_REASONS,
            "invalid_mixer,invalid_intervention,recovery_extraction_failed,non_finite_target,low_overlap",
        );
        assert_eq!(
            RejectionPolicyDecision::parse(&input),
            Err(PolicyError::InvalidRejectionPolicy)
        );
    }

    #[test]
    fn unresolved_policy_must_not_carry_policy_fields() {
        let input = unresolved().replace(
            "policy_reference = \"UNRESOLVED\"",
            "policy_reference = \"UNRESOLVED\"\n\
             rejection_policy = \"frozen_tdi71_typed_errors_v1\"\n\
             rejection_reasons = \"invalid_mixer\"",
        );
        assert_eq!(
            RejectionPolicyDecision::parse(&input),
            Err(PolicyError::UnexpectedFrozenPolicyFields)
        );
    }

    #[test]
    fn arbitrary_reason_keys_are_still_rejected() {
        let input = format!("{}allowed_reason = \"numerical_failure\"\n", unresolved());
        assert_eq!(
            RejectionPolicyDecision::parse(&input),
            Err(PolicyError::UnknownKey)
        );
    }

    #[test]
    fn wrong_commit_authorization_or_reference_fails_closed() {
        let input = frozen().replace(PREARM_MAIN_COMMIT, &"0".repeat(40));
        assert_eq!(
            RejectionPolicyDecision::parse(&input),
            Err(PolicyError::InvalidPrearmCommit)
        );

        let input = frozen().replace("NOT_AUTHORIZED", "AUTHORIZED");
        assert_eq!(
            RejectionPolicyDecision::parse(&input),
            Err(PolicyError::AuthorizationPresent)
        );

        let input = frozen().replace(
            "policy_reference = \"TDI-7.2-HUMAN-DECISION-2026-09-01-CHECKUPAUTO\"",
            "policy_reference = \"UNRESOLVED\"",
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
