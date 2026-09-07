//! Explicit development provenance. Caller declarations are not attestations.
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ManifestError {
    EmptyField,
    DuplicateField(String),
    MissingField(&'static str),
    InvalidCommit,
}
/// Canonical versioned UTF-8 fields; embed exact configuration/lockfile content
/// or a separately verified content identity. No implicit environment discovery.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExperimentManifest {
    fields: BTreeMap<String, String>,
}
impl ExperimentManifest {
    pub fn new(fields: impl IntoIterator<Item = (String, String)>) -> Result<Self, ManifestError> {
        let mut map = BTreeMap::new();
        for (key, value) in fields {
            if key.trim().is_empty() || value.trim().is_empty() {
                return Err(ManifestError::EmptyField);
            }
            if map.insert(key.clone(), value).is_some() {
                return Err(ManifestError::DuplicateField(key));
            }
        }
        for key in [
            "source_commit",
            "dependency_identity",
            "backend_identity",
            "configuration",
            "generator_identity",
            "metric_identity",
            "accounting_identity",
        ] {
            if !map.contains_key(key) {
                return Err(ManifestError::MissingField(key));
            }
        }
        let commit = &map["source_commit"];
        if !matches!(commit.len(), 40 | 64) || !commit.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err(ManifestError::InvalidCommit);
        }
        Ok(Self { fields: map })
    }
    #[must_use]
    pub fn fields(&self) -> &BTreeMap<String, String> {
        &self.fields
    }
    /// Byte-length framing makes embedded delimiters/newlines unambiguous.
    #[must_use]
    pub fn canonical_record(&self) -> String {
        let mut out = String::from("tdi-development-manifest/v1\n");
        for (key, value) in &self.fields {
            out.push_str(&format!("{}:{}{}:{}", key.len(), key, value.len(), value));
        }
        out
    }
}
/// Local campaign namespace only; does not redefine a series' frozen seed rules.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DevelopmentDomain {
    Development,
    Validation,
}
impl DevelopmentDomain {
    pub fn seed(self, index: u64) -> Option<u64> {
        (index < (1u64 << 63)).then_some(
            index
                | if self == Self::Validation {
                    1u64 << 63
                } else {
                    0
                },
        )
    }
}
/// Optional measured hardware evidence, separate from semantic bit accounting.
/// Missing values mean unmeasured. Sensor identity must name collection method.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PhysicalTelemetry {
    pub sensor_identity: String,
    pub resident_bytes: Option<u64>,
    pub accelerator_bytes: Option<u64>,
    pub elapsed_nanoseconds: Option<u128>,
}
