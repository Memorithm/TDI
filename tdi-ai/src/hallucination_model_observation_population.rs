//! Non-executing TDI-11.2 Development/Validation population-derivation scaffolding.
//!
//! This layer declares:
//! - the exact authorized domain taxonomy matching the TDI-11.2 pre-arm
//!   (`Development`, `Validation` only);
//! - a fail-closed caller-supplied population-derivation contract candidate;
//! - deterministic domain-separated seed-commitment framing;
//! - exact membership / coverage helpers that refuse Final/holdout leakage.
//!
//! It does **not**:
//! - invent scientific population sizes, strata, or seed lists;
//! - pin `development_validation_population_derivation`;
//! - create Final / confirmatory / holdout populations;
//! - load or execute a concrete model;
//! - set `model_execution_authorized` / `final_execution_authorized`.
//!
//! Presence of this scaffold is a non-authorizing candidate only. Distinct from
//! rejection/provenance (#213), resource-accounting (#216), and
//! registry/timing/H11-A (#218) scaffolds.

use core::fmt;

use super::hallucination_model_observation_rejections::{
    ModelObservationDomain, ModelObservationRejectionCode,
};

/// Scaffold schema marker for population-derivation contract records.
pub const MODEL_OBSERVATION_POPULATION_DERIVATION_SCHEMA: &str =
    "tdi11.2-model-observation-population-derivation-v0";

/// Scaffold schema marker for domain-separated seed-commitment records.
pub const MODEL_OBSERVATION_POPULATION_SEED_COMMITMENT_SCHEMA: &str =
    "tdi11.2-model-observation-population-seed-commitment-v0";

/// Non-authorizing candidate identifier for the Dev/Val population-derivation
/// freeze field.
///
/// Matching this string in docs/CI does **not** pin
/// `development_validation_population_derivation`. A later reviewed freeze must
/// still replace `unresolved_blocking` explicitly.
pub const CANDIDATE_POPULATION_DERIVATION_CONTRACT: &str =
    "ModelObservationPopulationDerivationContract";

/// Exact ordered authorized domain keys from the TDI-11.2 pre-arm.
///
/// These are *domain labels*, not a closed scientific population map. Selecting
/// strata / sizes / seeds remains a later reviewed freeze.
pub const AUTHORIZED_POPULATION_DOMAIN_KEYS: &[&str] = &["Development", "Validation"];

/// Exact count of authorized population domain keys.
pub const AUTHORIZED_POPULATION_DOMAIN_COUNT: usize = 2;

/// Forbidden domain / surface tokens that must never appear in population
/// scaffolding identities (fail-closed leakage guard).
pub const FORBIDDEN_POPULATION_SURFACE_TOKENS: &[&str] = &[
    "Final",
    "final_seed_list",
    "final_dataset",
    "final_runner",
    "final_result_payload",
    "final_population",
    "confirmatory_population",
    "holdout_population",
    "concrete_model_runner",
    "complete_world_hidden_truth",
    "evaluator_labels",
];

fn contains_forbidden_token(raw: &str) -> bool {
    FORBIDDEN_POPULATION_SURFACE_TOKENS
        .iter()
        .any(|token| raw.contains(token))
}

/// One caller-supplied stratum binding (domain + stratum id + seed-space key).
///
/// Scientific stratum names and seed spaces remain caller-supplied; this type
/// only enforces fail-closed integrity shape.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModelObservationPopulationStratum {
    domain: ModelObservationDomain,
    stratum_id: String,
    seed_space_key: String,
}

impl ModelObservationPopulationStratum {
    /// Construct a stratum binding; empty identities and forbidden tokens fail.
    pub fn new(
        domain: ModelObservationDomain,
        stratum_id: impl Into<String>,
        seed_space_key: impl Into<String>,
    ) -> Result<Self, ModelObservationRejectionCode> {
        let stratum_id = stratum_id.into();
        let seed_space_key = seed_space_key.into();
        if stratum_id.trim().is_empty() {
            return Err(ModelObservationRejectionCode::PopulationEmptyStratumId);
        }
        if seed_space_key.trim().is_empty() {
            return Err(ModelObservationRejectionCode::PopulationEmptySeedSpaceKey);
        }
        if contains_forbidden_token(&stratum_id) || contains_forbidden_token(&seed_space_key) {
            return Err(ModelObservationRejectionCode::PopulationForbiddenSurfaceToken);
        }
        Ok(Self {
            domain,
            stratum_id,
            seed_space_key,
        })
    }

    #[must_use]
    pub const fn domain(&self) -> ModelObservationDomain {
        self.domain
    }

    #[must_use]
    pub fn stratum_id(&self) -> &str {
        &self.stratum_id
    }

    #[must_use]
    pub fn seed_space_key(&self) -> &str {
        &self.seed_space_key
    }
}

/// Fail-closed caller-supplied Development/Validation population-derivation
/// contract candidate.
///
/// Requires exact coverage of both authorized domains, unique stratum ids, and
/// unique `(domain, seed_space_key)` pairs. Does **not** invent scientific
/// population sizes or pin the freeze field.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModelObservationPopulationDerivationContract {
    strata: Vec<ModelObservationPopulationStratum>,
}

impl ModelObservationPopulationDerivationContract {
    /// Construct a contract from caller-supplied strata.
    pub fn new(
        strata: Vec<ModelObservationPopulationStratum>,
    ) -> Result<Self, ModelObservationRejectionCode> {
        if strata.is_empty() {
            return Err(ModelObservationRejectionCode::PopulationEmpty);
        }

        let mut seen_stratum_ids: Vec<&str> = Vec::with_capacity(strata.len());
        let mut seen_seed_spaces: Vec<(ModelObservationDomain, &str)> =
            Vec::with_capacity(strata.len());
        let mut has_development = false;
        let mut has_validation = false;

        for stratum in &strata {
            if seen_stratum_ids.contains(&stratum.stratum_id()) {
                return Err(ModelObservationRejectionCode::PopulationDuplicateStratum);
            }
            seen_stratum_ids.push(stratum.stratum_id());

            let seed_key = (stratum.domain(), stratum.seed_space_key());
            if seen_seed_spaces
                .iter()
                .any(|(domain, key)| *domain == seed_key.0 && *key == seed_key.1)
            {
                return Err(ModelObservationRejectionCode::PopulationDuplicateSeedSpace);
            }
            seen_seed_spaces.push(seed_key);

            match stratum.domain() {
                ModelObservationDomain::Development => has_development = true,
                ModelObservationDomain::Validation => has_validation = true,
            }
        }

        if !(has_development && has_validation) {
            return Err(ModelObservationRejectionCode::PopulationDomainCoverageIncomplete);
        }

        Ok(Self { strata })
    }

    #[must_use]
    pub fn strata(&self) -> &[ModelObservationPopulationStratum] {
        &self.strata
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.strata.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.strata.is_empty()
    }

    /// Exact membership lookup by stratum id.
    pub fn lookup_stratum(
        &self,
        stratum_id: &str,
    ) -> Result<&ModelObservationPopulationStratum, ModelObservationRejectionCode> {
        self.strata
            .iter()
            .find(|entry| entry.stratum_id() == stratum_id)
            .ok_or(ModelObservationRejectionCode::PopulationUnknownStratum)
    }

    /// Exact membership check requiring domain agreement.
    pub fn require_membership(
        &self,
        domain: ModelObservationDomain,
        stratum_id: &str,
    ) -> Result<&ModelObservationPopulationStratum, ModelObservationRejectionCode> {
        let stratum = self.lookup_stratum(stratum_id)?;
        if stratum.domain() != domain {
            return Err(ModelObservationRejectionCode::PopulationDomainMismatch);
        }
        Ok(stratum)
    }

    /// Exact ordered coverage of authorized domain keys present in the contract.
    #[must_use]
    pub fn covered_domain_keys(&self) -> [&'static str; AUTHORIZED_POPULATION_DOMAIN_COUNT] {
        // Construction already requires both domains; report the authorized order.
        [
            AUTHORIZED_POPULATION_DOMAIN_KEYS[0],
            AUTHORIZED_POPULATION_DOMAIN_KEYS[1],
        ]
    }

    /// Parse a population domain label; unknown/forbidden labels fail closed.
    pub fn parse_domain(
        raw: &str,
    ) -> Result<ModelObservationDomain, ModelObservationRejectionCode> {
        match raw {
            "Development" => Ok(ModelObservationDomain::Development),
            "Validation" => Ok(ModelObservationDomain::Validation),
            _ => Err(ModelObservationRejectionCode::PopulationForbiddenDomain),
        }
    }

    /// Refuse any attempt to attach Final / confirmatory material to this contract.
    pub fn refuse_final_material(&self, token: &str) -> Result<(), ModelObservationRejectionCode> {
        if contains_forbidden_token(token) || token.trim().eq_ignore_ascii_case("final") {
            return Err(ModelObservationRejectionCode::PopulationFinalMaterialLeak);
        }
        Ok(())
    }

    /// Deterministic domain-separated seed-commitment framing (engineering only).
    ///
    /// Does **not** invent scientific seeds. Caller supplies the instance key;
    /// the commitment string is an exact integrity framing over domain + stratum
    /// + seed-space + instance key.
    pub fn seed_commitment(
        &self,
        domain: ModelObservationDomain,
        stratum_id: &str,
        instance_key: &str,
    ) -> Result<String, ModelObservationRejectionCode> {
        if instance_key.trim().is_empty() {
            return Err(ModelObservationRejectionCode::PopulationEmptySeedSpaceKey);
        }
        if contains_forbidden_token(instance_key) {
            return Err(ModelObservationRejectionCode::PopulationForbiddenSurfaceToken);
        }
        let stratum = self.require_membership(domain, stratum_id)?;
        Ok(format!(
            "{MODEL_OBSERVATION_POPULATION_SEED_COMMITMENT_SCHEMA}\ndomain:{}\nstratum_id:{}{}\nseed_space_key:{}{}\ninstance_key:{}{}",
            domain.as_str(),
            stratum.stratum_id().len(),
            stratum.stratum_id(),
            stratum.seed_space_key().len(),
            stratum.seed_space_key(),
            instance_key.len(),
            instance_key
        ))
    }

    /// Byte-length framed machine-readable contract record (scaffold only).
    #[must_use]
    pub fn canonical_record(&self) -> String {
        let mut out = format!(
            "{MODEL_OBSERVATION_POPULATION_DERIVATION_SCHEMA}\nstratum_count:{}\n",
            self.strata.len()
        );
        for stratum in &self.strata {
            out.push_str(&format!(
                "stratum:{}{}:{}:{}{}\n",
                stratum.stratum_id().len(),
                stratum.stratum_id(),
                stratum.domain().as_str(),
                stratum.seed_space_key().len(),
                stratum.seed_space_key()
            ));
        }
        out
    }
}

impl fmt::Display for ModelObservationPopulationDerivationContract {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{CANDIDATE_POPULATION_DERIVATION_CONTRACT}[{}]",
            self.strata.len()
        )
    }
}

/// Exact helper: authorized domain keys match the pre-arm allowed_domains list.
#[must_use]
pub fn authorized_population_domains_match_prearm() -> bool {
    AUTHORIZED_POPULATION_DOMAIN_KEYS == ["Development", "Validation"]
        && AUTHORIZED_POPULATION_DOMAIN_COUNT == 2
}
