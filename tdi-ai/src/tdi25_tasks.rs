//! TDI-25 Phase-B balanced task generators.
//!
//! Slice 11 introduces deterministic torsor-favorable transport tasks while
//! keeping the oracle separate from inference inputs.
//! Slice 12 adds chiral-favorable mirrored handedness pairs with the same
//! oracle/input separation.
//! Slice 13 adds mixed geometry tasks that require both a transported
//! relation and a parity-sensitive relation.
//! Slice 14 adds neutral six-component controls that privilege neither
//! torsor nor chirality in the target construction.
//! Slice 15 registers explicit position-geometry arms (linear/helical/
//! learned/external) so no geometry is an implicit default.
//! Slice 16 adds bounded deterministic difficulty strata independent of
//! model output.
//! Slice 17 embeds a typed Development/Validation split identity on every
//! Phase-B case without changing carriers, targets, or oracles.
//! Slice 18 adds a versioned protected-label inference API so callbacks
//! receive only inference-visible inputs; oracles stay sealed outside.
//! Slice 19 adds seed-domain registration with fail-closed disjointness
//! plus stable case canonicalization/hash over inference-visible inputs.

use core::fmt;

use super::tdi22_torsor::{Torsor3, Twist3, Vec3};
use super::tdi24_chiral::{Chiral6, ChiralScoreWeights};
use super::tdi25_torsor_chiral::{
    Generic6, TaskFamily, Tdi25Error, chiral_arm_score, generic_arm_score, torsor_arm_score,
};

/// Versioned TDI-25 torsor-favorable transport task contract.
pub const TORSOR_TRANSPORT_TASK_CONTRACT: &str = "tdi25-torsor-transport-task-v1";

/// Versioned TDI-25 chiral-favorable reflection task contract.
pub const CHIRAL_REFLECTION_TASK_CONTRACT: &str = "tdi25-chiral-reflection-task-v1";

/// Versioned TDI-25 mixed-geometry task contract.
pub const MIXED_GEOMETRY_TASK_CONTRACT: &str = "tdi25-mixed-geometry-task-v1";

/// Versioned TDI-25 neutral six-component control task contract.
pub const NEUTRAL_CONTROL_TASK_CONTRACT: &str = "tdi25-neutral-control-task-v1";

/// Versioned TDI-25 position-geometry arm registry contract.
pub const POSITION_GEOMETRY_ARM_CONTRACT: &str = "tdi25-position-geometry-arm-v1";

/// Inclusive upper bound on admissible geometry indices.
pub const POSITION_GEOMETRY_INDEX_MAX: u64 = 1024;

/// Versioned TDI-25 difficulty-strata contract.
pub const DIFFICULTY_STRATA_CONTRACT: &str = "tdi25-difficulty-strata-v1";

/// Inclusive upper bound on admissible difficulty levels (`0..=MAX`).
pub const DIFFICULTY_LEVEL_MAX: u8 = 3;

/// Versioned Slice-17 Development/Validation split-manifest contract.
pub const SPLIT_MANIFEST_CONTRACT: &str = "tdi25-split-manifest-v1";

/// Typed population split carried by every Phase-B case.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DataSplit {
    /// Non-final Development population.
    Development,
    /// Non-final Validation population.
    Validation,
}

impl DataSplit {
    /// Stable lowercase label for manifests and audits.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Development => "development",
            Self::Validation => "validation",
        }
    }

    /// Fail-closed parse of a split label.
    ///
    /// Protected/final and any other unknown labels are rejected; this slice
    /// never materializes those populations.
    pub fn parse(label: &str) -> Result<Self, Tdi25TaskError> {
        match label {
            "development" => Ok(Self::Development),
            "validation" => Ok(Self::Validation),
            _ => Err(Tdi25TaskError::UnknownSplitIdentity),
        }
    }
}

/// Complete typed identity of one task case within a split.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SplitCaseIdentity {
    pub split: DataSplit,
    pub case_id: u64,
    pub split_contract: &'static str,
}

/// Build the typed split identity embedded beside a case id.
#[must_use]
pub const fn split_case_identity(split: DataSplit, case_id: u64) -> SplitCaseIdentity {
    SplitCaseIdentity {
        split,
        case_id,
        split_contract: SPLIT_MANIFEST_CONTRACT,
    }
}

/// Versioned Slice-18 protected-label inference API contract.
pub const PROTECTED_LABEL_CONTRACT: &str = "tdi25-protected-label-api-v1";

/// Sealed oracle/target retained outside the inference callback.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ProtectedLabel<T> {
    oracle: T,
}

impl<T> ProtectedLabel<T> {
    /// Seal an oracle/target away from inference callbacks.
    #[must_use]
    pub const fn seal(oracle: T) -> Self {
        Self { oracle }
    }

    /// Reveal only on the evaluation/scoring path — never passed to inference.
    #[must_use]
    pub const fn reveal_for_evaluation(&self) -> &T {
        &self.oracle
    }
}

/// Labeled case pairing an inference-visible input with a sealed oracle.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LabeledCase<I, O> {
    input: I,
    label: ProtectedLabel<O>,
    label_contract: &'static str,
}

impl<I, O> LabeledCase<I, O> {
    /// Construct a labeled case from an inference input and sealed oracle.
    #[must_use]
    pub const fn new(input: I, oracle: O) -> Self {
        Self {
            input,
            label: ProtectedLabel::seal(oracle),
            label_contract: PROTECTED_LABEL_CONTRACT,
        }
    }

    /// Borrow the inference-visible input (no oracle/target).
    #[must_use]
    pub const fn inference_input(&self) -> &I {
        &self.input
    }

    /// Borrow the sealed label for evaluation/scoring only.
    #[must_use]
    pub const fn protected_label(&self) -> &ProtectedLabel<O> {
        &self.label
    }

    /// Protected-label API contract pin carried on every sealed case.
    #[must_use]
    pub const fn label_contract(&self) -> &'static str {
        self.label_contract
    }
}

/// Invoke an inference callback that, by construction, receives only the input.
#[must_use]
pub fn run_inference_callback<I, O, R, F>(case: &LabeledCase<I, O>, callback: F) -> R
where
    F: FnOnce(&I) -> R,
{
    callback(case.inference_input())
}

/// Seal a torsor-transport input with its shared oracle.
#[must_use]
pub fn seal_torsor_transport(
    input: TorsorTransportInput,
    oracle: TorsorTransportOracle,
) -> LabeledCase<TorsorTransportInput, TorsorTransportOracle> {
    LabeledCase::new(input, oracle)
}

/// Seal a chiral-reflection input with its member oracle.
#[must_use]
pub fn seal_chiral_reflection(
    input: ChiralReflectionInput,
    oracle: ChiralReflectionOracle,
) -> LabeledCase<ChiralReflectionInput, ChiralReflectionOracle> {
    LabeledCase::new(input, oracle)
}

/// Seal a mixed-geometry input with its member oracle.
#[must_use]
pub fn seal_mixed_geometry(
    input: MixedGeometryInput,
    oracle: MixedGeometryOracle,
) -> LabeledCase<MixedGeometryInput, MixedGeometryOracle> {
    LabeledCase::new(input, oracle)
}

/// Seal a neutral-control input with its member oracle.
#[must_use]
pub fn seal_neutral_control(
    input: NeutralControlInput,
    oracle: NeutralControlOracle,
) -> LabeledCase<NeutralControlInput, NeutralControlOracle> {
    LabeledCase::new(input, oracle)
}

/// Versioned Slice-19 seed/case canonicalization contract.
pub const SEED_CASE_CANONICALIZATION_CONTRACT: &str = "tdi25-seed-case-canonicalization-v1";

/// Declared provenance-bound seed domain.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SeedDomain {
    /// Non-final Development seed namespace.
    Development,
    /// Non-final Validation seed namespace.
    Validation,
}

impl SeedDomain {
    const fn domain_tag(self) -> u64 {
        match self {
            Self::Development => 0x5444_4932_3544_4556,
            Self::Validation => 0x5444_4932_3556_414C,
        }
    }

    /// Stable lowercase domain label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Development => "development",
            Self::Validation => "validation",
        }
    }

    /// Fail-closed parse of a seed-domain label.
    pub fn parse(label: &str) -> Result<Self, Tdi25TaskError> {
        match label {
            "development" => Ok(Self::Development),
            "validation" => Ok(Self::Validation),
            _ => Err(Tdi25TaskError::UnknownSeedDomain),
        }
    }

    /// Map a typed data split onto its seed domain.
    #[must_use]
    pub const fn from_split(split: DataSplit) -> Self {
        match split {
            DataSplit::Development => Self::Development,
            DataSplit::Validation => Self::Validation,
        }
    }
}

const fn task_family_label(family: TaskFamily) -> &'static str {
    match family {
        TaskFamily::TorsorFavorable => "torsor_favorable",
        TaskFamily::ChiralFavorable => "chiral_favorable",
        TaskFamily::Mixed => "mixed",
        TaskFamily::Neutral => "neutral",
    }
}

const fn family_seed_tag(family: TaskFamily) -> u64 {
    match family {
        TaskFamily::TorsorFavorable => 0x5446_4631_0000_0001,
        TaskFamily::ChiralFavorable => 0x5446_4632_0000_0002,
        TaskFamily::Mixed => 0x5446_4633_0000_0003,
        TaskFamily::Neutral => 0x5446_4634_0000_0004,
    }
}

/// Domain-separated mix of a declared local seed.
#[must_use]
pub fn mix_registered_seed(domain: SeedDomain, family: TaskFamily, local_seed: u64) -> u64 {
    let mut state = local_seed
        .wrapping_add(0x9E37_79B9_7F4A_7C15)
        .wrapping_mul(domain.domain_tag() | 1);
    state ^= family_seed_tag(family).rotate_left(17);
    state = (state ^ (state >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    state = (state ^ (state >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    state ^ (state >> 31)
}

/// One registry entry binding a local seed to a domain-separated mixed seed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RegisteredSeed {
    pub domain: SeedDomain,
    pub family: TaskFamily,
    pub local_seed: u64,
    pub mixed_seed: u64,
    pub registry_contract: &'static str,
}

/// Register a local seed under a declared domain/family namespace.
#[must_use]
pub fn register_seed(domain: SeedDomain, family: TaskFamily, local_seed: u64) -> RegisteredSeed {
    RegisteredSeed {
        domain,
        family,
        local_seed,
        mixed_seed: mix_registered_seed(domain, family, local_seed),
        registry_contract: SEED_CASE_CANONICALIZATION_CONTRACT,
    }
}

/// Fail closed when mixed seeds collide across distinct declared domains.
pub fn assert_seed_domain_disjointness(seeds: &[RegisteredSeed]) -> Result<(), Tdi25TaskError> {
    for (index, left) in seeds.iter().enumerate() {
        for right in seeds.iter().skip(index + 1) {
            if left.mixed_seed == right.mixed_seed && left.domain != right.domain {
                return Err(Tdi25TaskError::SeedDomainOverlap);
            }
        }
    }
    Ok(())
}

fn format_f64_bits(value: f64) -> String {
    format!("{:016x}", value.to_bits())
}

fn format_f64_slice(values: &[f64]) -> String {
    values
        .iter()
        .copied()
        .map(format_f64_bits)
        .collect::<Vec<_>>()
        .join(",")
}

fn format_vec3(point: Vec3) -> String {
    format_f64_slice(&[point.x, point.y, point.z])
}

fn format_twist3(twist: Twist3) -> String {
    format!(
        "w={};v={}",
        format_vec3(twist.angular()),
        format_vec3(twist.linear())
    )
}

fn format_torsor3(key: Torsor3) -> String {
    format!(
        "R={};M={};ref={}",
        format_vec3(key.resultant()),
        format_vec3(key.moment()),
        format_vec3(key.reference())
    )
}

fn format_chiral6(carrier: Chiral6) -> String {
    format_f64_slice(&carrier.as_array())
}

fn format_generic6(carrier: Generic6) -> String {
    format_f64_slice(&carrier.as_array())
}

fn format_weights(weights: ChiralScoreWeights) -> String {
    format!(
        "a={};b={};g={}",
        format_f64_bits(weights.alpha),
        format_f64_bits(weights.beta),
        format_f64_bits(weights.gamma)
    )
}

/// Stable canonical record for one inference-visible torsor-transport case.
#[must_use]
pub fn canonical_torsor_transport_record(input: &TorsorTransportInput) -> String {
    format!(
        "{SEED_CASE_CANONICALIZATION_CONTRACT};family={};split={};case={:016x};query={};key={};qpos={};gen={}",
        task_family_label(input.task_family),
        input.split.as_str(),
        input.case_id,
        format_twist3(input.query),
        format_torsor3(input.key),
        format_vec3(input.query_position),
        input.generator_contract,
    )
}

/// Stable canonical record for one inference-visible chiral-reflection case.
#[must_use]
pub fn canonical_chiral_reflection_record(input: &ChiralReflectionInput) -> String {
    format!(
        "{SEED_CASE_CANONICALIZATION_CONTRACT};family={};split={};case={:016x};query={};key={};weights={};gen={}",
        task_family_label(input.task_family),
        input.split.as_str(),
        input.case_id,
        format_chiral6(input.query),
        format_chiral6(input.key),
        format_weights(input.weights),
        input.generator_contract,
    )
}

/// Stable canonical record for one inference-visible mixed-geometry case.
#[must_use]
pub fn canonical_mixed_geometry_record(input: &MixedGeometryInput) -> String {
    format!(
        "{SEED_CASE_CANONICALIZATION_CONTRACT};family={};split={};case={:016x};tq={};tk={};qpos={};cq={};ck={};weights={};gen={}",
        task_family_label(input.task_family),
        input.split.as_str(),
        input.case_id,
        format_twist3(input.torsor_query),
        format_torsor3(input.torsor_key),
        format_vec3(input.query_position),
        format_chiral6(input.chiral_query),
        format_chiral6(input.chiral_key),
        format_weights(input.weights),
        input.generator_contract,
    )
}

/// Stable canonical record for one inference-visible neutral-control case.
#[must_use]
pub fn canonical_neutral_control_record(input: &NeutralControlInput) -> String {
    format!(
        "{SEED_CASE_CANONICALIZATION_CONTRACT};family={};split={};case={:016x};query={};key={};gen={}",
        task_family_label(input.task_family),
        input.split.as_str(),
        input.case_id,
        format_generic6(input.query),
        format_generic6(input.key),
        input.generator_contract,
    )
}

/// Non-cryptographic stable digest of a canonical record.
#[must_use]
pub fn canonical_digest(record: &str) -> String {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in record.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("{hash:016x}")
}

/// Canonical record plus digest for one inference-visible case.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CanonicalCaseDigest {
    pub record: String,
    pub digest: String,
    pub contract: &'static str,
}

fn finalize_canonical_record(record: String) -> CanonicalCaseDigest {
    let digest = canonical_digest(&record);
    CanonicalCaseDigest {
        record,
        digest,
        contract: SEED_CASE_CANONICALIZATION_CONTRACT,
    }
}

/// Canonicalize a torsor-transport inference input (no oracle).
#[must_use]
pub fn canonicalize_torsor_transport_input(input: &TorsorTransportInput) -> CanonicalCaseDigest {
    finalize_canonical_record(canonical_torsor_transport_record(input))
}

/// Canonicalize a chiral-reflection inference input (no oracle).
#[must_use]
pub fn canonicalize_chiral_reflection_input(input: &ChiralReflectionInput) -> CanonicalCaseDigest {
    finalize_canonical_record(canonical_chiral_reflection_record(input))
}

/// Canonicalize a mixed-geometry inference input (no oracle).
#[must_use]
pub fn canonicalize_mixed_geometry_input(input: &MixedGeometryInput) -> CanonicalCaseDigest {
    finalize_canonical_record(canonical_mixed_geometry_record(input))
}

/// Canonicalize a neutral-control inference input (no oracle).
#[must_use]
pub fn canonicalize_neutral_control_input(input: &NeutralControlInput) -> CanonicalCaseDigest {
    finalize_canonical_record(canonical_neutral_control_record(input))
}

/// Inference-visible input for one torsor transport case.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TorsorTransportInput {
    pub case_id: u64,
    /// Typed Development/Validation population identity.
    pub split: DataSplit,
    pub query: Twist3,
    pub key: Torsor3,
    pub query_position: Vec3,
    pub task_family: TaskFamily,
    pub generator_contract: &'static str,
}

/// Oracle retained separately from inference input.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TorsorTransportOracle {
    pub pair_id: u64,
    pub expected_score: f64,
    pub generator_contract: &'static str,
}

/// Two reductions of the same physical torsor with one shared oracle.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TorsorTransportPair {
    pub original: TorsorTransportInput,
    pub transported: TorsorTransportInput,
    pub oracle: TorsorTransportOracle,
}

/// Deterministically generate one transport-invariance task.
pub fn torsor_transport_pair(pair_id: u64) -> Result<TorsorTransportPair, Tdi25TaskError> {
    let original_id = pair_id
        .checked_mul(2)
        .ok_or(Tdi25TaskError::CaseIdOverflow)?;
    let transported_id = original_id
        .checked_add(1)
        .ok_or(Tdi25TaskError::CaseIdOverflow)?;

    let offset = ((pair_id % 41) as f64 + 1.0) / 128.0;
    let v = |x, y, z| {
        Vec3::new(x, y, z).map_err(|error| Tdi25TaskError::Bridge(Tdi25Error::Torsor(error)))
    };

    let resultant = v(2.0 + offset, 3.0, -4.0)?;
    let moment = v(-5.0, 7.0 + offset, 11.0)?;
    let reference = v(13.0, -17.0, 19.0 + offset)?;
    let target_reference = v(-2.0 - offset, 5.0, 7.0)?;
    let query_position = v(-23.0, 29.0 + offset, 31.0)?;
    let query = Twist3::new(v(1.0, -2.0, 3.0 + offset)?, v(0.5 + offset, 4.0, -1.0)?)
        .map_err(|error| Tdi25TaskError::Bridge(Tdi25Error::Torsor(error)))?;

    let original_key = Torsor3::new(resultant, moment, reference)
        .map_err(|error| Tdi25TaskError::Bridge(Tdi25Error::Torsor(error)))?;
    let transported_key = original_key
        .transport(target_reference)
        .map_err(|error| Tdi25TaskError::Bridge(Tdi25Error::Torsor(error)))?;
    let expected_score =
        torsor_arm_score(query, original_key, query_position).map_err(Tdi25TaskError::Bridge)?;

    let make_input = |case_id, key| TorsorTransportInput {
        case_id,
        split: DataSplit::Development,
        query,
        key,
        query_position,
        task_family: TaskFamily::TorsorFavorable,
        generator_contract: TORSOR_TRANSPORT_TASK_CONTRACT,
    };

    Ok(TorsorTransportPair {
        original: make_input(original_id, original_key),
        transported: make_input(transported_id, transported_key),
        oracle: TorsorTransportOracle {
            pair_id,
            expected_score,
            generator_contract: TORSOR_TRANSPORT_TASK_CONTRACT,
        },
    })
}

/// Handedness oracle for one member of a mirrored chiral pair.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HandednessTarget {
    /// Canonical orientation emitted first by the deterministic generator.
    Right,
    /// Simultaneously mirrored partner.
    Left,
}

/// Inference-visible input for one chiral reflection case.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ChiralReflectionInput {
    pub case_id: u64,
    /// Typed Development/Validation population identity.
    pub split: DataSplit,
    pub query: Chiral6,
    pub key: Chiral6,
    pub weights: ChiralScoreWeights,
    pub task_family: TaskFamily,
    pub generator_contract: &'static str,
}

/// Handedness/score oracle retained separately from inference input.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ChiralReflectionOracle {
    pub pair_id: u64,
    pub handedness: HandednessTarget,
    pub expected_score: f64,
    pub generator_contract: &'static str,
}

/// Mirrored handedness pair with per-member oracles kept outside inputs.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ChiralReflectionPair {
    pub right: ChiralReflectionInput,
    pub left: ChiralReflectionInput,
    pub right_oracle: ChiralReflectionOracle,
    pub left_oracle: ChiralReflectionOracle,
}

/// Deterministically generate one chiral-favorable reflection pair.
pub fn chiral_reflection_pair(pair_id: u64) -> Result<ChiralReflectionPair, Tdi25TaskError> {
    let right_id = pair_id
        .checked_mul(2)
        .ok_or(Tdi25TaskError::CaseIdOverflow)?;
    let left_id = right_id
        .checked_add(1)
        .ok_or(Tdi25TaskError::CaseIdOverflow)?;

    // Bounded offset keeps every Stage-A scalar finite and model-independent.
    let offset = ((pair_id % 29) as f64 + 1.0) / 64.0;
    // gamma != 0 so the parity-odd channel makes handedness target-relevant.
    let weights = ChiralScoreWeights::new(0.7, -0.2, 1.3)
        .map_err(|error| Tdi25TaskError::Bridge(Tdi25Error::Chiral(error)))?;
    let query = Chiral6::new([1.0 + offset, -0.75, 0.5], [0.625, -1.0 - offset, 1.5])
        .map_err(|error| Tdi25TaskError::Bridge(Tdi25Error::Chiral(error)))?;
    let key = Chiral6::new([-0.5, 1.25 + offset, -1.0], [1.75 + offset, 0.375, -0.875])
        .map_err(|error| Tdi25TaskError::Bridge(Tdi25Error::Chiral(error)))?;

    let right_score = chiral_arm_score(query, key, weights).map_err(Tdi25TaskError::Bridge)?;
    let left_query = query.mirror();
    let left_key = key.mirror();
    let left_score =
        chiral_arm_score(left_query, left_key, weights).map_err(Tdi25TaskError::Bridge)?;

    let make_input = |case_id, query, key| ChiralReflectionInput {
        case_id,
        split: DataSplit::Development,
        query,
        key,
        weights,
        task_family: TaskFamily::ChiralFavorable,
        generator_contract: CHIRAL_REFLECTION_TASK_CONTRACT,
    };

    Ok(ChiralReflectionPair {
        right: make_input(right_id, query, key),
        left: make_input(left_id, left_query, left_key),
        right_oracle: ChiralReflectionOracle {
            pair_id,
            handedness: HandednessTarget::Right,
            expected_score: right_score,
            generator_contract: CHIRAL_REFLECTION_TASK_CONTRACT,
        },
        left_oracle: ChiralReflectionOracle {
            pair_id,
            handedness: HandednessTarget::Left,
            expected_score: left_score,
            generator_contract: CHIRAL_REFLECTION_TASK_CONTRACT,
        },
    })
}

/// Inference-visible input combining torsor transport and chiral reflection.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MixedGeometryInput {
    pub case_id: u64,
    /// Typed Development/Validation population identity.
    pub split: DataSplit,
    pub torsor_query: Twist3,
    pub torsor_key: Torsor3,
    pub query_position: Vec3,
    pub chiral_query: Chiral6,
    pub chiral_key: Chiral6,
    pub weights: ChiralScoreWeights,
    pub task_family: TaskFamily,
    pub generator_contract: &'static str,
}

/// Oracle requiring both transport invariance and parity-sensitive handedness.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MixedGeometryOracle {
    pub pair_id: u64,
    pub handedness: HandednessTarget,
    pub expected_torsor_score: f64,
    pub expected_chiral_score: f64,
    pub generator_contract: &'static str,
}

/// Base vs jointly-transformed mixed geometry pair.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MixedGeometryPair {
    pub base: MixedGeometryInput,
    pub transformed: MixedGeometryInput,
    pub base_oracle: MixedGeometryOracle,
    pub transformed_oracle: MixedGeometryOracle,
}

/// Deterministically generate one mixed geometry task.
///
/// The transformed member applies torsor reduction-point transport and
/// simultaneous chiral mirror reflection together. The oracle demands the
/// shared torsor score (transport-invariant) and opposite handedness with a
/// parity-odd chiral score (reflection-sensitive).
pub fn mixed_geometry_pair(pair_id: u64) -> Result<MixedGeometryPair, Tdi25TaskError> {
    let base_id = pair_id
        .checked_mul(2)
        .ok_or(Tdi25TaskError::CaseIdOverflow)?;
    let transformed_id = base_id
        .checked_add(1)
        .ok_or(Tdi25TaskError::CaseIdOverflow)?;

    let offset = ((pair_id % 37) as f64 + 1.0) / 96.0;
    let v = |x, y, z| {
        Vec3::new(x, y, z).map_err(|error| Tdi25TaskError::Bridge(Tdi25Error::Torsor(error)))
    };

    let resultant = v(2.0 + offset, 3.0, -4.0)?;
    let moment = v(-5.0, 7.0 + offset, 11.0)?;
    let reference = v(13.0, -17.0, 19.0 + offset)?;
    let target_reference = v(-2.0 - offset, 5.0, 7.0)?;
    let query_position = v(-23.0, 29.0 + offset, 31.0)?;
    let torsor_query = Twist3::new(v(1.0, -2.0, 3.0 + offset)?, v(0.5 + offset, 4.0, -1.0)?)
        .map_err(|error| Tdi25TaskError::Bridge(Tdi25Error::Torsor(error)))?;
    let original_key = Torsor3::new(resultant, moment, reference)
        .map_err(|error| Tdi25TaskError::Bridge(Tdi25Error::Torsor(error)))?;
    let transported_key = original_key
        .transport(target_reference)
        .map_err(|error| Tdi25TaskError::Bridge(Tdi25Error::Torsor(error)))?;
    let expected_torsor_score = torsor_arm_score(torsor_query, original_key, query_position)
        .map_err(Tdi25TaskError::Bridge)?;

    let weights = ChiralScoreWeights::new(0.7, -0.2, 1.3)
        .map_err(|error| Tdi25TaskError::Bridge(Tdi25Error::Chiral(error)))?;
    let chiral_query = Chiral6::new([1.0 + offset, -0.75, 0.5], [0.625, -1.0 - offset, 1.5])
        .map_err(|error| Tdi25TaskError::Bridge(Tdi25Error::Chiral(error)))?;
    let chiral_key = Chiral6::new([-0.5, 1.25 + offset, -1.0], [1.75 + offset, 0.375, -0.875])
        .map_err(|error| Tdi25TaskError::Bridge(Tdi25Error::Chiral(error)))?;
    let base_chiral_score =
        chiral_arm_score(chiral_query, chiral_key, weights).map_err(Tdi25TaskError::Bridge)?;
    let mirrored_query = chiral_query.mirror();
    let mirrored_key = chiral_key.mirror();
    let transformed_chiral_score =
        chiral_arm_score(mirrored_query, mirrored_key, weights).map_err(Tdi25TaskError::Bridge)?;

    let make_input = |case_id, torsor_key, chiral_query, chiral_key| MixedGeometryInput {
        case_id,
        split: DataSplit::Development,
        torsor_query,
        torsor_key,
        query_position,
        chiral_query,
        chiral_key,
        weights,
        task_family: TaskFamily::Mixed,
        generator_contract: MIXED_GEOMETRY_TASK_CONTRACT,
    };

    Ok(MixedGeometryPair {
        base: make_input(base_id, original_key, chiral_query, chiral_key),
        transformed: make_input(
            transformed_id,
            transported_key,
            mirrored_query,
            mirrored_key,
        ),
        base_oracle: MixedGeometryOracle {
            pair_id,
            handedness: HandednessTarget::Right,
            expected_torsor_score,
            expected_chiral_score: base_chiral_score,
            generator_contract: MIXED_GEOMETRY_TASK_CONTRACT,
        },
        transformed_oracle: MixedGeometryOracle {
            pair_id,
            handedness: HandednessTarget::Left,
            expected_torsor_score,
            expected_chiral_score: transformed_chiral_score,
            generator_contract: MIXED_GEOMETRY_TASK_CONTRACT,
        },
    })
}

/// Structure-agnostic binary target for neutral controls.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NeutralTarget {
    /// Positive class under the deterministic sign schedule.
    ClassA,
    /// Negative class under the deterministic sign schedule.
    ClassB,
}

/// Inference-visible input for one neutral Generic6 control case.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NeutralControlInput {
    pub case_id: u64,
    /// Typed Development/Validation population identity.
    pub split: DataSplit,
    pub query: Generic6,
    pub key: Generic6,
    pub task_family: TaskFamily,
    pub generator_contract: &'static str,
}

/// Target/score oracle retained separately from inference input.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NeutralControlOracle {
    pub pair_id: u64,
    pub target: NeutralTarget,
    pub expected_score: f64,
    pub generator_contract: &'static str,
}

/// Opposite-target neutral pair that privileges neither torsor nor chirality.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NeutralControlPair {
    pub class_a: NeutralControlInput,
    pub class_b: NeutralControlInput,
    pub class_a_oracle: NeutralControlOracle,
    pub class_b_oracle: NeutralControlOracle,
}

/// Deterministically generate one neutral six-component control pair.
///
/// Both members use the matched Generic6 capacity only. The class label is
/// driven by a global sign on generic components, not by torsor transport or
/// chiral parity structure.
pub fn neutral_control_pair(pair_id: u64) -> Result<NeutralControlPair, Tdi25TaskError> {
    let class_a_id = pair_id
        .checked_mul(2)
        .ok_or(Tdi25TaskError::CaseIdOverflow)?;
    let class_b_id = class_a_id
        .checked_add(1)
        .ok_or(Tdi25TaskError::CaseIdOverflow)?;

    let offset = ((pair_id % 43) as f64 + 1.0) / 112.0;
    let make_carriers = |sign: f64| -> Result<(Generic6, Generic6), Tdi25TaskError> {
        let query = Generic6::new([
            sign * (1.0 + offset),
            0.5,
            -0.75,
            0.375 + offset,
            -0.875,
            1.25 - offset,
        ])
        .map_err(Tdi25TaskError::Bridge)?;
        let key = Generic6::new([
            1.0,
            sign * (0.625 + offset),
            0.25,
            -1.125,
            0.625 + offset,
            0.5,
        ])
        .map_err(Tdi25TaskError::Bridge)?;
        Ok((query, key))
    };

    let (query_a, key_a) = make_carriers(1.0)?;
    let (query_b, key_b) = make_carriers(-1.0)?;
    let score_a = generic_arm_score(query_a, key_a).map_err(Tdi25TaskError::Bridge)?;
    let score_b = generic_arm_score(query_b, key_b).map_err(Tdi25TaskError::Bridge)?;

    let make_input = |case_id, query, key| NeutralControlInput {
        case_id,
        split: DataSplit::Development,
        query,
        key,
        task_family: TaskFamily::Neutral,
        generator_contract: NEUTRAL_CONTROL_TASK_CONTRACT,
    };

    Ok(NeutralControlPair {
        class_a: make_input(class_a_id, query_a, key_a),
        class_b: make_input(class_b_id, query_b, key_b),
        class_a_oracle: NeutralControlOracle {
            pair_id,
            target: NeutralTarget::ClassA,
            expected_score: score_a,
            generator_contract: NEUTRAL_CONTROL_TASK_CONTRACT,
        },
        class_b_oracle: NeutralControlOracle {
            pair_id,
            target: NeutralTarget::ClassB,
            expected_score: score_b,
            generator_contract: NEUTRAL_CONTROL_TASK_CONTRACT,
        },
    })
}

/// Explicit position-geometry experimental arm.
///
/// A physical 3D coordinate is never an implicit token assumption; every
/// geometry choice carries provenance through this registry.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PositionGeometryArm {
    /// Uniform linear abscissa along x.
    Linear,
    /// Discrete helical sample on a bounded octagon with linear z.
    Helical,
    /// Deterministic frozen lookup table (not a trained embedding).
    Learned,
    /// Caller-supplied external coordinate with fail-closed validation.
    External,
}

impl PositionGeometryArm {
    /// Stable lowercase label for manifests and audits.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Linear => "linear",
            Self::Helical => "helical",
            Self::Learned => "learned",
            Self::External => "external",
        }
    }

    /// Human-readable provenance retained beside every sample.
    #[must_use]
    pub const fn provenance(self) -> &'static str {
        match self {
            Self::Linear => "tdi25-geometry-linear-abscissa-v1",
            Self::Helical => "tdi25-geometry-helical-octagon-v1",
            Self::Learned => "tdi25-geometry-learned-table-v1",
            Self::External => "tdi25-geometry-external-supplied-v1",
        }
    }

    /// Fail-closed parse of an arm label.
    pub fn parse(label: &str) -> Result<Self, Tdi25TaskError> {
        match label {
            "linear" => Ok(Self::Linear),
            "helical" => Ok(Self::Helical),
            "learned" => Ok(Self::Learned),
            "external" => Ok(Self::External),
            _ => Err(Tdi25TaskError::UnknownGeometryArm),
        }
    }
}

/// One materialized position sample with explicit arm provenance.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PositionGeometrySample {
    pub arm: PositionGeometryArm,
    pub index: u64,
    pub point: Vec3,
    pub arm_provenance: &'static str,
    pub generator_contract: &'static str,
}

/// Resolve a position for the declared geometry arm.
///
/// `external` is required only for [`PositionGeometryArm::External`].
pub fn position_geometry_point(
    arm: PositionGeometryArm,
    index: u64,
    external: Option<Vec3>,
) -> Result<PositionGeometrySample, Tdi25TaskError> {
    if index > POSITION_GEOMETRY_INDEX_MAX {
        return Err(Tdi25TaskError::GeometryIndexOutOfRange);
    }

    let point = match arm {
        PositionGeometryArm::Linear => Vec3::new(index as f64 / 64.0, 0.0, 0.0)
            .map_err(|error| Tdi25TaskError::Bridge(Tdi25Error::Torsor(error)))?,
        PositionGeometryArm::Helical => {
            let (x, y) = match index % 8 {
                0 => (1.0, 0.0),
                1 => (1.0, 1.0),
                2 => (0.0, 1.0),
                3 => (-1.0, 1.0),
                4 => (-1.0, 0.0),
                5 => (-1.0, -1.0),
                6 => (0.0, -1.0),
                _ => (1.0, -1.0),
            };
            Vec3::new(x, y, index as f64 / 64.0)
                .map_err(|error| Tdi25TaskError::Bridge(Tdi25Error::Torsor(error)))?
        }
        PositionGeometryArm::Learned => {
            // Frozen deterministic table: cyclic unit-ish offsets, not trainable.
            let table = [
                (0.25, -0.5, 0.75),
                (-0.75, 0.25, 0.5),
                (0.5, 0.75, -0.25),
                (-0.25, -0.75, 0.125),
            ];
            let (x, y, z) = table[(index as usize) % table.len()];
            let scale = 1.0 + (index / 4) as f64 / 64.0;
            Vec3::new(x * scale, y * scale, z * scale)
                .map_err(|error| Tdi25TaskError::Bridge(Tdi25Error::Torsor(error)))?
        }
        PositionGeometryArm::External => {
            external.ok_or(Tdi25TaskError::ExternalPositionRequired)?
        }
    };

    Ok(PositionGeometrySample {
        arm,
        index,
        point,
        arm_provenance: arm.provenance(),
        generator_contract: POSITION_GEOMETRY_ARM_CONTRACT,
    })
}

/// Enumerate the frozen registry in stable order.
#[must_use]
pub const fn position_geometry_registry() -> [PositionGeometryArm; 4] {
    [
        PositionGeometryArm::Linear,
        PositionGeometryArm::Helical,
        PositionGeometryArm::Learned,
        PositionGeometryArm::External,
    ]
}

/// Bounded difficulty level independent of any model output.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DifficultyLevel {
    /// Discrete level in `0..=DIFFICULTY_LEVEL_MAX`.
    pub level: u8,
}

impl DifficultyLevel {
    /// Construct a fail-closed level.
    pub fn new(level: u8) -> Result<Self, Tdi25TaskError> {
        if level > DIFFICULTY_LEVEL_MAX {
            return Err(Tdi25TaskError::DifficultyOutOfRange);
        }
        Ok(Self { level })
    }

    /// Deterministic stratum for a seed; never consults model output.
    #[must_use]
    pub fn from_seed(seed: u64) -> Self {
        Self {
            level: (seed % u64::from(DIFFICULTY_LEVEL_MAX + 1)) as u8,
        }
    }

    /// Positive finite magnitude scale for the stratum.
    #[must_use]
    pub fn magnitude_scale(self) -> f64 {
        1.0 + f64::from(self.level) * 0.25
    }
}

/// Difficulty metadata attached to an existing Phase-B family seed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DifficultyStratum {
    pub seed: u64,
    pub level: DifficultyLevel,
    pub generator_contract: &'static str,
}

/// Assign a bounded difficulty stratum from a deterministic seed.
#[must_use]
pub fn difficulty_stratum(seed: u64) -> DifficultyStratum {
    DifficultyStratum {
        seed,
        level: DifficultyLevel::from_seed(seed),
        generator_contract: DIFFICULTY_STRATA_CONTRACT,
    }
}

/// Materialize a torsor-transport pair in one typed split.
pub fn torsor_transport_pair_in_split(
    pair_id: u64,
    split: DataSplit,
) -> Result<TorsorTransportPair, Tdi25TaskError> {
    let mut pair = torsor_transport_pair(pair_id)?;
    pair.original.split = split;
    pair.transported.split = split;
    Ok(pair)
}

/// Materialize a chiral-reflection pair in one typed split.
pub fn chiral_reflection_pair_in_split(
    pair_id: u64,
    split: DataSplit,
) -> Result<ChiralReflectionPair, Tdi25TaskError> {
    let mut pair = chiral_reflection_pair(pair_id)?;
    pair.right.split = split;
    pair.left.split = split;
    Ok(pair)
}

/// Materialize a mixed-geometry pair in one typed split.
pub fn mixed_geometry_pair_in_split(
    pair_id: u64,
    split: DataSplit,
) -> Result<MixedGeometryPair, Tdi25TaskError> {
    let mut pair = mixed_geometry_pair(pair_id)?;
    pair.base.split = split;
    pair.transformed.split = split;
    Ok(pair)
}

/// Materialize a neutral-control pair in one typed split.
pub fn neutral_control_pair_in_split(
    pair_id: u64,
    split: DataSplit,
) -> Result<NeutralControlPair, Tdi25TaskError> {
    let mut pair = neutral_control_pair(pair_id)?;
    pair.class_a.split = split;
    pair.class_b.split = split;
    Ok(pair)
}

/// TDI-25 task-generation failures.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tdi25TaskError {
    CaseIdOverflow,
    GeometryIndexOutOfRange,
    ExternalPositionRequired,
    UnknownGeometryArm,
    DifficultyOutOfRange,
    /// Split identity label is not Development or Validation.
    UnknownSplitIdentity,
    /// Seed-domain label is not Development or Validation.
    UnknownSeedDomain,
    /// Mixed seeds collide across distinct declared domains.
    SeedDomainOverlap,
    Bridge(Tdi25Error),
}

impl fmt::Display for Tdi25TaskError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CaseIdOverflow => formatter.write_str("TDI-25 task case id overflow"),
            Self::GeometryIndexOutOfRange => {
                formatter.write_str("TDI-25 position geometry index out of range")
            }
            Self::ExternalPositionRequired => {
                formatter.write_str("TDI-25 external position geometry requires a supplied point")
            }
            Self::UnknownGeometryArm => formatter.write_str("TDI-25 unknown position geometry arm"),
            Self::DifficultyOutOfRange => {
                formatter.write_str("TDI-25 difficulty level out of admissible range")
            }
            Self::UnknownSplitIdentity => {
                formatter.write_str("TDI-25 unknown split identity label")
            }
            Self::UnknownSeedDomain => formatter.write_str("TDI-25 unknown seed domain label"),
            Self::SeedDomainOverlap => formatter.write_str("TDI-25 seed domain overlap"),
            Self::Bridge(error) => write!(formatter, "TDI-25 bridge rejected task: {error}"),
        }
    }
}

impl std::error::Error for Tdi25TaskError {}

#[cfg(test)]
mod tests {
    use super::super::tdi24_chiral::observables;
    use super::*;

    fn close(lhs: f64, rhs: f64) {
        let scale = 1.0_f64.max(lhs.abs()).max(rhs.abs());
        assert!((lhs - rhs).abs() <= 128.0 * f64::EPSILON * scale);
    }

    #[test]
    fn transport_pairs_are_deterministic_and_keep_oracle_outside_inputs() {
        let pair = torsor_transport_pair(9).unwrap();
        assert_eq!(pair, torsor_transport_pair(9).unwrap());
        assert_ne!(pair.original.case_id, pair.transported.case_id);
        assert_eq!(pair.original.task_family, TaskFamily::TorsorFavorable);
        assert_eq!(pair.transported.task_family, TaskFamily::TorsorFavorable);
        assert_eq!(
            pair.oracle.generator_contract,
            TORSOR_TRANSPORT_TASK_CONTRACT
        );
    }

    #[test]
    fn alternate_reduction_preserves_physical_invariants_and_oracle_score() {
        for pair_id in 0..32 {
            let pair = torsor_transport_pair(pair_id).unwrap();
            assert_eq!(
                pair.original.key.resultant(),
                pair.transported.key.resultant()
            );
            assert_ne!(
                pair.original.key.reference(),
                pair.transported.key.reference()
            );

            let original_score = torsor_arm_score(
                pair.original.query,
                pair.original.key,
                pair.original.query_position,
            )
            .unwrap();
            let transported_score = torsor_arm_score(
                pair.transported.query,
                pair.transported.key,
                pair.transported.query_position,
            )
            .unwrap();
            close(original_score, pair.oracle.expected_score);
            close(transported_score, pair.oracle.expected_score);

            let original_origin = pair.original.key.origin_moment().unwrap();
            let transported_origin = pair.transported.key.origin_moment().unwrap();
            close(original_origin.x, transported_origin.x);
            close(original_origin.y, transported_origin.y);
            close(original_origin.z, transported_origin.z);
        }
    }

    #[test]
    fn torsor_case_id_overflow_fails_closed() {
        assert_eq!(
            torsor_transport_pair(u64::MAX),
            Err(Tdi25TaskError::CaseIdOverflow)
        );
    }

    #[test]
    fn reflection_pairs_are_deterministic_and_keep_oracle_outside_inputs() {
        let pair = chiral_reflection_pair(11).unwrap();
        assert_eq!(pair, chiral_reflection_pair(11).unwrap());
        assert_ne!(pair.right.case_id, pair.left.case_id);
        assert_eq!(pair.right.task_family, TaskFamily::ChiralFavorable);
        assert_eq!(pair.left.task_family, TaskFamily::ChiralFavorable);
        assert_eq!(
            pair.right_oracle.generator_contract,
            CHIRAL_REFLECTION_TASK_CONTRACT
        );
        assert_eq!(pair.right_oracle.handedness, HandednessTarget::Right);
        assert_eq!(pair.left_oracle.handedness, HandednessTarget::Left);
        assert_ne!(
            pair.right_oracle.expected_score,
            pair.left_oracle.expected_score
        );
    }

    #[test]
    fn mirrored_members_are_exact_mirrors_with_parity_odd_oracle_scores() {
        for pair_id in 0..32 {
            let pair = chiral_reflection_pair(pair_id).unwrap();
            assert_eq!(pair.left.query, pair.right.query.mirror());
            assert_eq!(pair.left.key, pair.right.key.mirror());
            assert_eq!(pair.left.weights, pair.right.weights);
            assert!(pair.right.weights.gamma.abs() > 0.0);

            let right_score =
                chiral_arm_score(pair.right.query, pair.right.key, pair.right.weights).unwrap();
            let left_score =
                chiral_arm_score(pair.left.query, pair.left.key, pair.left.weights).unwrap();
            close(right_score, pair.right_oracle.expected_score);
            close(left_score, pair.left_oracle.expected_score);

            let base = observables(pair.right.query, pair.right.key).unwrap();
            let reflected = observables(pair.left.query, pair.left.key).unwrap();
            close(reflected.direct, base.direct);
            close(reflected.mirrored, base.mirrored);
            close(reflected.chiral, -base.chiral);
            // Even channels cancel in the score difference; only gamma*chi remains.
            close(
                right_score - left_score,
                2.0 * pair.right.weights.gamma * base.chiral,
            );
        }
    }

    #[test]
    fn chiral_case_id_overflow_fails_closed() {
        assert_eq!(
            chiral_reflection_pair(u64::MAX),
            Err(Tdi25TaskError::CaseIdOverflow)
        );
    }

    #[test]
    fn mixed_pairs_are_deterministic_and_keep_oracle_outside_inputs() {
        let pair = mixed_geometry_pair(7).unwrap();
        assert_eq!(pair, mixed_geometry_pair(7).unwrap());
        assert_ne!(pair.base.case_id, pair.transformed.case_id);
        assert_eq!(pair.base.task_family, TaskFamily::Mixed);
        assert_eq!(pair.transformed.task_family, TaskFamily::Mixed);
        assert_eq!(
            pair.base_oracle.generator_contract,
            MIXED_GEOMETRY_TASK_CONTRACT
        );
        assert_eq!(pair.base_oracle.handedness, HandednessTarget::Right);
        assert_eq!(pair.transformed_oracle.handedness, HandednessTarget::Left);
        assert_eq!(
            pair.base_oracle.expected_torsor_score,
            pair.transformed_oracle.expected_torsor_score
        );
        assert_ne!(
            pair.base_oracle.expected_chiral_score,
            pair.transformed_oracle.expected_chiral_score
        );
    }

    #[test]
    fn mixed_transform_preserves_torsor_score_and_flips_chiral_parity() {
        for pair_id in 0..24 {
            let pair = mixed_geometry_pair(pair_id).unwrap();
            assert_eq!(
                pair.base.torsor_key.resultant(),
                pair.transformed.torsor_key.resultant()
            );
            assert_ne!(
                pair.base.torsor_key.reference(),
                pair.transformed.torsor_key.reference()
            );
            assert_eq!(
                pair.transformed.chiral_query,
                pair.base.chiral_query.mirror()
            );
            assert_eq!(pair.transformed.chiral_key, pair.base.chiral_key.mirror());
            assert!(pair.base.weights.gamma.abs() > 0.0);

            let base_torsor = torsor_arm_score(
                pair.base.torsor_query,
                pair.base.torsor_key,
                pair.base.query_position,
            )
            .unwrap();
            let transformed_torsor = torsor_arm_score(
                pair.transformed.torsor_query,
                pair.transformed.torsor_key,
                pair.transformed.query_position,
            )
            .unwrap();
            close(base_torsor, pair.base_oracle.expected_torsor_score);
            close(
                transformed_torsor,
                pair.transformed_oracle.expected_torsor_score,
            );
            close(base_torsor, transformed_torsor);

            let base_chiral = chiral_arm_score(
                pair.base.chiral_query,
                pair.base.chiral_key,
                pair.base.weights,
            )
            .unwrap();
            let transformed_chiral = chiral_arm_score(
                pair.transformed.chiral_query,
                pair.transformed.chiral_key,
                pair.transformed.weights,
            )
            .unwrap();
            close(base_chiral, pair.base_oracle.expected_chiral_score);
            close(
                transformed_chiral,
                pair.transformed_oracle.expected_chiral_score,
            );

            let base_obs = observables(pair.base.chiral_query, pair.base.chiral_key).unwrap();
            let transformed_obs =
                observables(pair.transformed.chiral_query, pair.transformed.chiral_key).unwrap();
            close(transformed_obs.direct, base_obs.direct);
            close(transformed_obs.mirrored, base_obs.mirrored);
            close(transformed_obs.chiral, -base_obs.chiral);
        }
    }

    #[test]
    fn mixed_case_id_overflow_fails_closed() {
        assert_eq!(
            mixed_geometry_pair(u64::MAX),
            Err(Tdi25TaskError::CaseIdOverflow)
        );
    }

    #[test]
    fn neutral_pairs_are_deterministic_and_keep_oracle_outside_inputs() {
        let pair = neutral_control_pair(5).unwrap();
        assert_eq!(pair, neutral_control_pair(5).unwrap());
        assert_ne!(pair.class_a.case_id, pair.class_b.case_id);
        assert_eq!(pair.class_a.task_family, TaskFamily::Neutral);
        assert_eq!(pair.class_b.task_family, TaskFamily::Neutral);
        assert_eq!(
            pair.class_a_oracle.generator_contract,
            NEUTRAL_CONTROL_TASK_CONTRACT
        );
        assert_eq!(pair.class_a_oracle.target, NeutralTarget::ClassA);
        assert_eq!(pair.class_b_oracle.target, NeutralTarget::ClassB);
        assert_ne!(
            pair.class_a_oracle.expected_score,
            pair.class_b_oracle.expected_score
        );
    }

    #[test]
    fn neutral_targets_use_generic_capacity_without_torsor_or_chiral_privilege() {
        for pair_id in 0..24 {
            let pair = neutral_control_pair(pair_id).unwrap();
            let score_a = generic_arm_score(pair.class_a.query, pair.class_a.key).unwrap();
            let score_b = generic_arm_score(pair.class_b.query, pair.class_b.key).unwrap();
            close(score_a, pair.class_a_oracle.expected_score);
            close(score_b, pair.class_b_oracle.expected_score);

            // Class label tracks only the global generic sign schedule.
            assert_eq!(pair.class_a.query.as_array()[0].signum(), 1.0);
            assert_eq!(pair.class_b.query.as_array()[0].signum(), -1.0);
            // Shared nuisance coordinates (indices 2,3,5 on query; 0,2,5 on key)
            // stay identical across classes — no chirality/torsor channeling.
            let qa = pair.class_a.query.as_array();
            let qb = pair.class_b.query.as_array();
            let ka = pair.class_a.key.as_array();
            let kb = pair.class_b.key.as_array();
            close(qa[2], qb[2]);
            close(qa[3], qb[3]);
            close(qa[5], qb[5]);
            close(ka[0], kb[0]);
            close(ka[2], kb[2]);
            close(ka[5], kb[5]);
        }
    }

    #[test]
    fn neutral_case_id_overflow_fails_closed() {
        assert_eq!(
            neutral_control_pair(u64::MAX),
            Err(Tdi25TaskError::CaseIdOverflow)
        );
    }

    #[test]
    fn position_geometry_registry_is_explicit_and_complete() {
        let registry = position_geometry_registry();
        assert_eq!(registry.len(), 4);
        assert_eq!(registry[0], PositionGeometryArm::Linear);
        assert_eq!(registry[1], PositionGeometryArm::Helical);
        assert_eq!(registry[2], PositionGeometryArm::Learned);
        assert_eq!(registry[3], PositionGeometryArm::External);
        for arm in registry {
            assert_eq!(PositionGeometryArm::parse(arm.as_str()).unwrap(), arm);
            assert!(!arm.provenance().is_empty());
        }
        assert_eq!(
            PositionGeometryArm::parse("implicit"),
            Err(Tdi25TaskError::UnknownGeometryArm)
        );
    }

    #[test]
    fn position_geometry_points_are_deterministic_with_provenance() {
        for index in 0..32 {
            let linear = position_geometry_point(PositionGeometryArm::Linear, index, None).unwrap();
            assert_eq!(
                linear,
                position_geometry_point(PositionGeometryArm::Linear, index, None).unwrap()
            );
            assert_eq!(linear.generator_contract, POSITION_GEOMETRY_ARM_CONTRACT);
            assert_eq!(
                linear.arm_provenance,
                PositionGeometryArm::Linear.provenance()
            );
            close(linear.point.y, 0.0);
            close(linear.point.z, 0.0);

            let helical =
                position_geometry_point(PositionGeometryArm::Helical, index, None).unwrap();
            assert_eq!(helical.arm, PositionGeometryArm::Helical);

            let learned =
                position_geometry_point(PositionGeometryArm::Learned, index, None).unwrap();
            assert_eq!(learned.arm, PositionGeometryArm::Learned);
            assert_ne!(learned.point, linear.point);
        }

        let supplied = Vec3::new(1.0, -2.0, 0.5).unwrap();
        let external =
            position_geometry_point(PositionGeometryArm::External, 3, Some(supplied)).unwrap();
        assert_eq!(external.point, supplied);
        assert_eq!(
            position_geometry_point(PositionGeometryArm::External, 3, None),
            Err(Tdi25TaskError::ExternalPositionRequired)
        );
        assert_eq!(
            position_geometry_point(
                PositionGeometryArm::Linear,
                POSITION_GEOMETRY_INDEX_MAX + 1,
                None
            ),
            Err(Tdi25TaskError::GeometryIndexOutOfRange)
        );
    }

    #[test]
    fn difficulty_strata_are_deterministic_bounded_and_model_independent() {
        for seed in 0..64 {
            let stratum = difficulty_stratum(seed);
            assert_eq!(stratum, difficulty_stratum(seed));
            assert!(stratum.level.level <= DIFFICULTY_LEVEL_MAX);
            assert_eq!(stratum.generator_contract, DIFFICULTY_STRATA_CONTRACT);
            assert_eq!(stratum.level, DifficultyLevel::from_seed(seed));
            let scale = stratum.level.magnitude_scale();
            assert!(scale.is_finite() && scale >= 1.0);
            assert!(scale <= 1.0 + f64::from(DIFFICULTY_LEVEL_MAX) * 0.25);
        }
        assert_eq!(
            DifficultyLevel::new(DIFFICULTY_LEVEL_MAX + 1),
            Err(Tdi25TaskError::DifficultyOutOfRange)
        );
        let mut seen = [false; (DIFFICULTY_LEVEL_MAX as usize) + 1];
        for seed in 0..32 {
            seen[difficulty_stratum(seed).level.level as usize] = true;
        }
        assert!(seen.iter().all(|hit| *hit));
    }

    #[test]
    fn development_and_validation_identities_are_typed_and_disjoint() {
        let development = split_case_identity(DataSplit::Development, 17);
        let validation = split_case_identity(DataSplit::Validation, 17);
        assert_ne!(development, validation);
        assert_eq!(development.case_id, validation.case_id);
        assert_eq!(development.split_contract, SPLIT_MANIFEST_CONTRACT);
        assert_eq!(validation.split_contract, SPLIT_MANIFEST_CONTRACT);
        assert_eq!(DataSplit::Development.as_str(), "development");
        assert_eq!(DataSplit::Validation.as_str(), "validation");
        assert_eq!(
            DataSplit::parse("development").unwrap(),
            DataSplit::Development
        );
        assert_eq!(
            DataSplit::parse("validation").unwrap(),
            DataSplit::Validation
        );
        assert_eq!(
            DataSplit::parse("protected"),
            Err(Tdi25TaskError::UnknownSplitIdentity)
        );
        assert_eq!(
            DataSplit::parse("final"),
            Err(Tdi25TaskError::UnknownSplitIdentity)
        );
        assert_eq!(
            DataSplit::parse("holdout"),
            Err(Tdi25TaskError::UnknownSplitIdentity)
        );
    }

    #[test]
    fn every_phase_b_family_embeds_the_requested_split() {
        for split in [DataSplit::Development, DataSplit::Validation] {
            let torsor = torsor_transport_pair_in_split(3, split).unwrap();
            assert_eq!(torsor.original.split, split);
            assert_eq!(torsor.transported.split, split);

            let chiral = chiral_reflection_pair_in_split(4, split).unwrap();
            assert_eq!(chiral.right.split, split);
            assert_eq!(chiral.left.split, split);

            let mixed = mixed_geometry_pair_in_split(5, split).unwrap();
            assert_eq!(mixed.base.split, split);
            assert_eq!(mixed.transformed.split, split);

            let neutral = neutral_control_pair_in_split(6, split).unwrap();
            assert_eq!(neutral.class_a.split, split);
            assert_eq!(neutral.class_b.split, split);
        }
        assert_eq!(
            torsor_transport_pair(1).unwrap().original.split,
            DataSplit::Development
        );
        assert_eq!(
            chiral_reflection_pair(1).unwrap().right.split,
            DataSplit::Development
        );
    }

    #[test]
    fn split_choice_does_not_change_task_payload_or_oracle() {
        let t_dev = torsor_transport_pair_in_split(11, DataSplit::Development).unwrap();
        let t_val = torsor_transport_pair_in_split(11, DataSplit::Validation).unwrap();
        assert_eq!(t_dev.original.query, t_val.original.query);
        assert_eq!(t_dev.original.key, t_val.original.key);
        assert_eq!(t_dev.oracle, t_val.oracle);
        assert_eq!(t_dev.original.case_id, t_val.original.case_id);
        assert_ne!(t_dev.original.split, t_val.original.split);

        let c_dev = chiral_reflection_pair_in_split(9, DataSplit::Development).unwrap();
        let c_val = chiral_reflection_pair_in_split(9, DataSplit::Validation).unwrap();
        assert_eq!(c_dev.right.query, c_val.right.query);
        assert_eq!(c_dev.right_oracle, c_val.right_oracle);
        assert_ne!(c_dev.right.split, c_val.right.split);

        let m_dev = mixed_geometry_pair_in_split(8, DataSplit::Development).unwrap();
        let m_val = mixed_geometry_pair_in_split(8, DataSplit::Validation).unwrap();
        assert_eq!(m_dev.base.torsor_query, m_val.base.torsor_query);
        assert_eq!(m_dev.base_oracle, m_val.base_oracle);
        assert_ne!(m_dev.base.split, m_val.base.split);

        let n_dev = neutral_control_pair_in_split(7, DataSplit::Development).unwrap();
        let n_val = neutral_control_pair_in_split(7, DataSplit::Validation).unwrap();
        assert_eq!(n_dev.class_a.query, n_val.class_a.query);
        assert_eq!(n_dev.class_a_oracle, n_val.class_a_oracle);
        assert_ne!(n_dev.class_a.split, n_val.class_a.split);
    }

    #[test]
    fn protected_label_api_hides_oracles_from_inference_callbacks() {
        let pair = torsor_transport_pair(21).unwrap();
        let labeled = seal_torsor_transport(pair.original, pair.oracle);
        assert_eq!(labeled.label_contract(), PROTECTED_LABEL_CONTRACT);
        assert_eq!(labeled.inference_input().case_id, pair.original.case_id);
        assert_eq!(labeled.inference_input().query, pair.original.query);
        assert_eq!(labeled.inference_input().key, pair.original.key);
        assert_eq!(
            labeled.inference_input().task_family,
            TaskFamily::TorsorFavorable
        );
        assert_eq!(
            *labeled.protected_label().reveal_for_evaluation(),
            pair.oracle
        );

        let observed = run_inference_callback(&labeled, |input| {
            assert_eq!(input.query, pair.original.query);
            assert_eq!(input.key, pair.original.key);
            format!("{input:?}")
        });
        assert!(!observed.contains("expected_score"));
        assert!(!observed.contains("oracle"));
        assert!(observed.contains("TorsorFavorable"));
    }

    #[test]
    fn every_phase_b_family_seals_oracles_outside_inference_inputs() {
        let torsor = torsor_transport_pair(3).unwrap();
        let sealed_torsor = seal_torsor_transport(torsor.transported, torsor.oracle);
        assert_eq!(
            run_inference_callback(&sealed_torsor, |input| input.split),
            DataSplit::Development
        );
        assert_eq!(
            sealed_torsor
                .protected_label()
                .reveal_for_evaluation()
                .expected_score,
            torsor.oracle.expected_score
        );

        let chiral = chiral_reflection_pair(4).unwrap();
        let sealed_chiral = seal_chiral_reflection(chiral.left, chiral.left_oracle);
        assert_eq!(
            *sealed_chiral.protected_label().reveal_for_evaluation(),
            chiral.left_oracle
        );
        let leaked = run_inference_callback(&sealed_chiral, |input| format!("{input:?}"));
        assert!(!leaked.contains("handedness"));
        assert!(!leaked.contains("expected_score"));
        assert!(!leaked.contains("Left"));
        assert!(!leaked.contains("Right"));
        assert!(leaked.contains("ChiralFavorable"));

        let mixed = mixed_geometry_pair(5).unwrap();
        let sealed_mixed = seal_mixed_geometry(mixed.transformed, mixed.transformed_oracle);
        assert_eq!(
            sealed_mixed
                .protected_label()
                .reveal_for_evaluation()
                .handedness,
            HandednessTarget::Left
        );
        assert_eq!(
            run_inference_callback(&sealed_mixed, |input| input.case_id),
            mixed.transformed.case_id
        );
        let mixed_leaked = run_inference_callback(&sealed_mixed, |input| format!("{input:?}"));
        assert!(!mixed_leaked.contains("expected_torsor_score"));
        assert!(!mixed_leaked.contains("expected_chiral_score"));
        assert!(!mixed_leaked.contains("handedness"));

        let neutral = neutral_control_pair(6).unwrap();
        let sealed_neutral = seal_neutral_control(neutral.class_b, neutral.class_b_oracle);
        assert_eq!(
            *sealed_neutral.protected_label().reveal_for_evaluation(),
            neutral.class_b_oracle
        );
        let neutral_leaked = run_inference_callback(&sealed_neutral, |input| format!("{input:?}"));
        assert!(!neutral_leaked.contains("ClassA"));
        assert!(!neutral_leaked.contains("ClassB"));
        assert!(!neutral_leaked.contains("expected_score"));
        assert!(!neutral_leaked.contains("target"));
        assert_eq!(sealed_neutral.label_contract(), PROTECTED_LABEL_CONTRACT);
    }

    #[test]
    fn sealed_inference_input_preserves_split_without_exposing_oracle() {
        let case = chiral_reflection_pair_in_split(11, DataSplit::Validation).unwrap();
        let labeled = seal_chiral_reflection(case.right, case.right_oracle);
        assert_eq!(labeled.inference_input().split, DataSplit::Validation);
        assert_eq!(
            *labeled.protected_label().reveal_for_evaluation(),
            case.right_oracle
        );
        assert_eq!(PROTECTED_LABEL_CONTRACT, "tdi25-protected-label-api-v1");
    }

    #[test]
    fn seed_registry_is_deterministic_and_domain_separated() {
        let families = [
            TaskFamily::TorsorFavorable,
            TaskFamily::ChiralFavorable,
            TaskFamily::Mixed,
            TaskFamily::Neutral,
        ];
        for family in families {
            for local in 0..64 {
                let development = register_seed(SeedDomain::Development, family, local);
                let validation = register_seed(SeedDomain::Validation, family, local);
                assert_eq!(
                    development,
                    register_seed(SeedDomain::Development, family, local)
                );
                assert_eq!(
                    development.registry_contract,
                    SEED_CASE_CANONICALIZATION_CONTRACT
                );
                assert_eq!(
                    development.mixed_seed,
                    mix_registered_seed(SeedDomain::Development, family, local)
                );
                assert_ne!(development.mixed_seed, validation.mixed_seed);
                assert_eq!(
                    SeedDomain::from_split(DataSplit::Development),
                    SeedDomain::Development
                );
                assert_eq!(
                    SeedDomain::from_split(DataSplit::Validation),
                    SeedDomain::Validation
                );
            }
        }
        assert_eq!(
            SeedDomain::parse("protected"),
            Err(Tdi25TaskError::UnknownSeedDomain)
        );
        assert_eq!(
            SeedDomain::parse("final"),
            Err(Tdi25TaskError::UnknownSeedDomain)
        );
        assert_eq!(SeedDomain::Development.as_str(), "development");
    }

    #[test]
    fn declared_seed_domains_have_no_mixed_seed_overlap() {
        let families = [
            TaskFamily::TorsorFavorable,
            TaskFamily::ChiralFavorable,
            TaskFamily::Mixed,
            TaskFamily::Neutral,
        ];
        let mut seeds = Vec::new();
        for family in families {
            for local in 0..256u64 {
                seeds.push(register_seed(SeedDomain::Development, family, local));
                seeds.push(register_seed(SeedDomain::Validation, family, local));
            }
        }
        assert_eq!(seeds.len(), 2048);
        assert_eq!(assert_seed_domain_disjointness(&seeds), Ok(()));

        let mixed = seeds
            .iter()
            .map(|seed| seed.mixed_seed)
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(mixed.len(), seeds.len());

        let forged = [
            RegisteredSeed {
                domain: SeedDomain::Development,
                family: TaskFamily::TorsorFavorable,
                local_seed: 0,
                mixed_seed: 0xDEAD_BEEF,
                registry_contract: SEED_CASE_CANONICALIZATION_CONTRACT,
            },
            RegisteredSeed {
                domain: SeedDomain::Validation,
                family: TaskFamily::TorsorFavorable,
                local_seed: 1,
                mixed_seed: 0xDEAD_BEEF,
                registry_contract: SEED_CASE_CANONICALIZATION_CONTRACT,
            },
        ];
        assert_eq!(
            assert_seed_domain_disjointness(&forged),
            Err(Tdi25TaskError::SeedDomainOverlap)
        );
    }

    #[test]
    fn case_canonicalization_is_stable_and_excludes_oracles() {
        let torsor = torsor_transport_pair(17).unwrap();
        let sealed = seal_torsor_transport(torsor.original, torsor.oracle);
        let first = canonicalize_torsor_transport_input(sealed.inference_input());
        let second = canonicalize_torsor_transport_input(sealed.inference_input());
        assert_eq!(first, second);
        assert_eq!(first.contract, SEED_CASE_CANONICALIZATION_CONTRACT);
        assert_eq!(first.digest, canonical_digest(&first.record));
        assert!(
            first
                .record
                .starts_with(SEED_CASE_CANONICALIZATION_CONTRACT)
        );
        assert!(!first.record.contains("expected_score"));
        assert!(!first.record.contains("oracle"));
        assert!(first.record.contains("torsor_favorable"));
        assert!(first.record.contains("development"));

        let validation = torsor_transport_pair_in_split(17, DataSplit::Validation).unwrap();
        let validation_digest = canonicalize_torsor_transport_input(&validation.original);
        assert_ne!(first.digest, validation_digest.digest);
        assert!(validation_digest.record.contains("validation"));
    }

    #[test]
    fn every_phase_b_family_has_a_deterministic_canonical_digest() {
        let digests = [
            canonicalize_torsor_transport_input(&torsor_transport_pair(1).unwrap().original),
            canonicalize_chiral_reflection_input(&chiral_reflection_pair(2).unwrap().left),
            canonicalize_mixed_geometry_input(&mixed_geometry_pair(3).unwrap().base),
            canonicalize_neutral_control_input(&neutral_control_pair(4).unwrap().class_a),
        ];
        let mut unique = std::collections::BTreeSet::new();
        for digest in &digests {
            assert_eq!(digest.digest.len(), 16);
            assert!(!digest.record.contains("expected_score"));
            assert!(!digest.record.contains("handedness"));
            assert!(!digest.record.contains("target"));
            unique.insert(digest.digest.clone());
        }
        assert_eq!(unique.len(), 4);
        assert_eq!(
            SEED_CASE_CANONICALIZATION_CONTRACT,
            "tdi25-seed-case-canonicalization-v1"
        );
    }
}
