//! TDI-24 Phase-B deterministic task generators.
//!
//! Slice 11 introduces reflection-discriminative paired handedness tasks.
//! Later slices extend this module with additional frozen task families.

use core::fmt;

use super::tdi24_chiral::{Chiral6, ChiralError};

/// Versioned Slice-11 reflection-discriminative generator contract.
pub const REFLECTION_DISCRIMINATIVE_CONTRACT: &str = "tdi24-reflection-discriminative-generator-v1";

/// Versioned Slice-12 reflection-nuisance generator contract.
pub const REFLECTION_NUISANCE_CONTRACT: &str = "tdi24-reflection-nuisance-generator-v1";

/// Versioned Slice-13 ordered direction/reversal generator contract.
pub const DIRECTION_REVERSAL_CONTRACT: &str = "tdi24-direction-reversal-generator-v1";

/// Versioned Slice-14 non-chiral negative-control generator contract.
pub const NON_CHIRAL_CONTROL_CONTRACT: &str = "tdi24-non-chiral-control-generator-v1";

/// Versioned Slice-15 difficulty-strata contract.
pub const DIFFICULTY_STRATA_CONTRACT: &str = "tdi24-difficulty-strata-v1";

/// Versioned Slice-16 Development/Validation split-manifest contract.
pub const SPLIT_MANIFEST_CONTRACT: &str = "tdi24-split-manifest-v1";

/// Versioned Slice-17 protected-label API contract.
pub const PROTECTED_LABEL_CONTRACT: &str = "tdi24-protected-label-api-v1";

/// Versioned Slice-18 seed-registry contract.
pub const SEED_REGISTRY_CONTRACT: &str = "tdi24-seed-registry-v1";

/// Inclusive upper bound on admissible difficulty levels (`0..=MAX`).
pub const DIFFICULTY_LEVEL_MAX: u8 = 3;

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
    pub fn parse(label: &str) -> Result<Self, Tdi24TaskError> {
        match label {
            "development" => Ok(Self::Development),
            "validation" => Ok(Self::Validation),
            _ => Err(Tdi24TaskError::UnknownSplitIdentity),
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

/// Phase-B task family identity carried on inference views.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TaskFamily {
    /// Slice-11 reflection-discriminative generator.
    ReflectionDiscriminative,
    /// Slice-12 reflection-nuisance generator.
    ReflectionNuisance,
    /// Slice-13 direction/reversal generator.
    DirectionReversal,
    /// Slice-14 non-chiral negative-control generator.
    NonChiralControl,
}

impl TaskFamily {
    /// Stable lowercase family label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ReflectionDiscriminative => "reflection_discriminative",
            Self::ReflectionNuisance => "reflection_nuisance",
            Self::DirectionReversal => "direction_reversal",
            Self::NonChiralControl => "non_chiral_control",
        }
    }

    /// Generator contract pin for this family.
    #[must_use]
    pub const fn generator_contract(self) -> &'static str {
        match self {
            Self::ReflectionDiscriminative => REFLECTION_DISCRIMINATIVE_CONTRACT,
            Self::ReflectionNuisance => REFLECTION_NUISANCE_CONTRACT,
            Self::DirectionReversal => DIRECTION_REVERSAL_CONTRACT,
            Self::NonChiralControl => NON_CHIRAL_CONTROL_CONTRACT,
        }
    }
}

/// Inference-visible carriers and identity. Deliberately omits any oracle/target.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InferenceView {
    /// Task family that produced the case.
    pub family: TaskFamily,
    /// Typed Development/Validation population identity.
    pub split: DataSplit,
    /// Globally deterministic case identity.
    pub case_id: u64,
    /// Pair id (or nuisance id for non-chiral controls).
    pub group_id: u64,
    /// Query carrier visible to inference.
    pub query: Chiral6,
    /// Key carrier visible to inference.
    pub key: Chiral6,
    /// Protected-label API contract pin.
    pub label_contract: &'static str,
}

/// Sealed expected target retained outside the inference callback.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProtectedLabel<T> {
    target: T,
}

impl<T> ProtectedLabel<T> {
    /// Seal an oracle/target away from inference callbacks.
    #[must_use]
    pub const fn seal(target: T) -> Self {
        Self { target }
    }

    /// Reveal only on the evaluation/scoring path — never passed to inference.
    #[must_use]
    pub const fn reveal_for_evaluation(&self) -> &T {
        &self.target
    }
}

/// Labeled case pairing an inference view with a sealed oracle.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LabeledCase<T> {
    view: InferenceView,
    label: ProtectedLabel<T>,
}

impl<T> LabeledCase<T> {
    /// Construct a labeled case from an inference view and sealed target.
    #[must_use]
    pub const fn new(view: InferenceView, target: T) -> Self {
        Self {
            view,
            label: ProtectedLabel::seal(target),
        }
    }

    /// Borrow the inference-visible view (no target).
    #[must_use]
    pub const fn inference_view(&self) -> &InferenceView {
        &self.view
    }

    /// Borrow the sealed label for evaluation/scoring only.
    #[must_use]
    pub const fn protected_label(&self) -> &ProtectedLabel<T> {
        &self.label
    }
}

/// Invoke an inference callback that, by construction, receives only [`InferenceView`].
#[must_use]
pub fn run_inference_callback<T, R, F>(case: &LabeledCase<T>, callback: F) -> R
where
    F: FnOnce(&InferenceView) -> R,
{
    callback(case.inference_view())
}

/// Seal a reflection-discriminative case into the protected-label API.
#[must_use]
pub fn seal_reflection_discriminative(
    case: &ReflectionDiscriminativeCase,
) -> LabeledCase<HandednessTarget> {
    LabeledCase::new(
        InferenceView {
            family: TaskFamily::ReflectionDiscriminative,
            split: case.split,
            case_id: case.case_id,
            group_id: case.pair_id,
            query: case.query,
            key: case.key,
            label_contract: PROTECTED_LABEL_CONTRACT,
        },
        case.target,
    )
}

/// Seal a reflection-nuisance case into the protected-label API.
#[must_use]
pub fn seal_reflection_nuisance(
    case: &ReflectionNuisanceCase,
) -> LabeledCase<ReflectionInvariantTarget> {
    LabeledCase::new(
        InferenceView {
            family: TaskFamily::ReflectionNuisance,
            split: case.split,
            case_id: case.case_id,
            group_id: case.pair_id,
            query: case.query,
            key: case.key,
            label_contract: PROTECTED_LABEL_CONTRACT,
        },
        case.target,
    )
}

/// Seal a direction/reversal case into the protected-label API.
#[must_use]
pub fn seal_direction_reversal(case: &DirectionReversalCase) -> LabeledCase<DirectionTarget> {
    LabeledCase::new(
        InferenceView {
            family: TaskFamily::DirectionReversal,
            split: case.split,
            case_id: case.case_id,
            group_id: case.pair_id,
            query: case.query,
            key: case.key,
            label_contract: PROTECTED_LABEL_CONTRACT,
        },
        case.target,
    )
}

/// Seal a non-chiral control case into the protected-label API.
#[must_use]
pub fn seal_non_chiral_control(case: &NonChiralControlCase) -> LabeledCase<NonChiralTarget> {
    LabeledCase::new(
        InferenceView {
            family: TaskFamily::NonChiralControl,
            split: case.split,
            case_id: case.case_id,
            group_id: case.nuisance_id,
            query: case.query,
            key: case.key,
            label_contract: PROTECTED_LABEL_CONTRACT,
        },
        case.target,
    )
}

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
            Self::Development => 0x5444_4932_3444_4556,
            Self::Validation => 0x5444_4932_3456_414C,
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
    pub fn parse(label: &str) -> Result<Self, Tdi24TaskError> {
        match label {
            "development" => Ok(Self::Development),
            "validation" => Ok(Self::Validation),
            _ => Err(Tdi24TaskError::UnknownSeedDomain),
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

fn family_seed_tag(family: TaskFamily) -> u64 {
    match family {
        TaskFamily::ReflectionDiscriminative => 0x5244_4631_0000_0001,
        TaskFamily::ReflectionNuisance => 0x5244_4632_0000_0002,
        TaskFamily::DirectionReversal => 0x5244_4633_0000_0003,
        TaskFamily::NonChiralControl => 0x5244_4634_0000_0004,
    }
}

/// Domain-separated mix of a declared local seed.
#[must_use]
pub fn mix_registered_seed(domain: SeedDomain, family: TaskFamily, local_seed: u64) -> u64 {
    // Stage the mix so domain/family tags cannot cancel inside the local-seed
    // XOR subspace (which would otherwise create systematic cross-family
    // collisions for dense local ranges).
    let mut state = local_seed
        .wrapping_add(0x9E37_79B9_7F4A_7C15)
        .wrapping_mul(domain.domain_tag() | 1);
    state ^= family_seed_tag(family).rotate_left(17);
    state = (state ^ (state >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    state = (state ^ (state >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    state ^ (state >> 31)
}

/// One registry entry binding a local seed to a domain-separated mixed seed.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
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
        registry_contract: SEED_REGISTRY_CONTRACT,
    }
}

/// Fail closed when mixed seeds collide across distinct declared domains.
pub fn assert_seed_domain_disjointness(seeds: &[RegisteredSeed]) -> Result<(), Tdi24TaskError> {
    for (index, left) in seeds.iter().enumerate() {
        for right in seeds.iter().skip(index + 1) {
            if left.mixed_seed == right.mixed_seed && left.domain != right.domain {
                return Err(Tdi24TaskError::SeedDomainOverlap);
            }
        }
    }
    Ok(())
}

const NUISANCE_CASE_ID_PREFIX: u64 = 1_u64 << 63;

/// Handedness oracle for one member of a mirrored pair.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HandednessTarget {
    /// Canonical orientation emitted first by the deterministic generator.
    Right,
    /// Simultaneously mirrored partner.
    Left,
}

/// One bounded reflection-discriminative case.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ReflectionDiscriminativeCase {
    /// Pair identity shared by both mirrored members.
    pub pair_id: u64,
    /// Typed Development/Validation population identity.
    pub split: DataSplit,
    /// Globally deterministic member identity (`2*pair_id` or `2*pair_id+1`).
    pub case_id: u64,
    /// Query carrier.
    pub query: Chiral6,
    /// Key carrier.
    pub key: Chiral6,
    /// Handedness oracle label.
    pub target: HandednessTarget,
    /// Generator contract identity.
    pub generator_contract: &'static str,
}

/// Pair of exact mirrored cases.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ReflectionDiscriminativePair {
    /// Canonical member.
    pub right: ReflectionDiscriminativeCase,
    /// Simultaneously reflected member.
    pub left: ReflectionDiscriminativeCase,
}

/// Deterministically materialize one handedness pair from a bounded pair id.
pub fn reflection_discriminative_pair(
    pair_id: u64,
) -> Result<ReflectionDiscriminativePair, Tdi24TaskError> {
    let right_id = pair_id
        .checked_mul(2)
        .ok_or(Tdi24TaskError::CaseIdOverflow)?;
    let left_id = right_id
        .checked_add(1)
        .ok_or(Tdi24TaskError::CaseIdOverflow)?;

    // The bounded offset changes examples without ever depending on model output.
    // Magnitudes stay small enough that every Stage-A scalar remains finite.
    let offset = ((pair_id % 29) as f64 + 1.0) / 64.0;
    let query = Chiral6::new([1.0 + offset, -0.75, 0.5], [0.625, -1.0 - offset, 1.5])
        .map_err(Tdi24TaskError::Chiral)?;
    let key = Chiral6::new([-0.5, 1.25 + offset, -1.0], [1.75 + offset, 0.375, -0.875])
        .map_err(Tdi24TaskError::Chiral)?;

    let right = ReflectionDiscriminativeCase {
        pair_id,
        split: DataSplit::Development,
        case_id: right_id,
        query,
        key,
        target: HandednessTarget::Right,
        generator_contract: REFLECTION_DISCRIMINATIVE_CONTRACT,
    };
    let left = ReflectionDiscriminativeCase {
        pair_id,
        split: DataSplit::Development,
        case_id: left_id,
        query: query.mirror(),
        key: key.mirror(),
        target: HandednessTarget::Left,
        generator_contract: REFLECTION_DISCRIMINATIVE_CONTRACT,
    };
    Ok(ReflectionDiscriminativePair { right, left })
}

/// Reflection-invariant binary oracle for the nuisance family.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReflectionInvariantTarget {
    /// Positive even-sector class.
    ClassA,
    /// Negative even-sector class.
    ClassB,
}

/// One case where mirror reflection is a nuisance transformation and must not
/// alter the oracle target.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ReflectionNuisanceCase {
    pub pair_id: u64,
    /// Typed Development/Validation population identity.
    pub split: DataSplit,
    pub case_id: u64,
    pub query: Chiral6,
    pub key: Chiral6,
    pub target: ReflectionInvariantTarget,
    pub generator_contract: &'static str,
}

/// Exact mirrored nuisance pair with one shared target.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ReflectionNuisancePair {
    pub canonical: ReflectionNuisanceCase,
    pub reflected: ReflectionNuisanceCase,
}

/// Deterministically materialize one reflection-nuisance pair.
pub fn reflection_nuisance_pair(pair_id: u64) -> Result<ReflectionNuisancePair, Tdi24TaskError> {
    let local_base = pair_id
        .checked_mul(2)
        .filter(|value| *value < (u64::MAX >> 1))
        .ok_or(Tdi24TaskError::CaseIdOverflow)?;
    let canonical_id = NUISANCE_CASE_ID_PREFIX | local_base;
    let reflected_id = canonical_id
        .checked_add(1)
        .ok_or(Tdi24TaskError::CaseIdOverflow)?;

    let target = if pair_id % 2 == 0 {
        ReflectionInvariantTarget::ClassA
    } else {
        ReflectionInvariantTarget::ClassB
    };
    let sign = match target {
        ReflectionInvariantTarget::ClassA => 1.0,
        ReflectionInvariantTarget::ClassB => -1.0,
    };
    let offset = ((pair_id % 31) as f64 + 1.0) / 80.0;
    let query = Chiral6::new(
        [sign * (1.0 + offset), 0.5, -0.25],
        [0.75 + offset, -0.625, 1.125],
    )
    .map_err(Tdi24TaskError::Chiral)?;
    let key = Chiral6::new([1.0, -0.5 + offset, 0.875], [-1.25, 0.375 + offset, 0.5])
        .map_err(Tdi24TaskError::Chiral)?;

    let canonical = ReflectionNuisanceCase {
        pair_id,
        split: DataSplit::Development,
        case_id: canonical_id,
        query,
        key,
        target,
        generator_contract: REFLECTION_NUISANCE_CONTRACT,
    };
    let reflected = ReflectionNuisanceCase {
        pair_id,
        split: DataSplit::Development,
        case_id: reflected_id,
        query: query.mirror(),
        key: key.mirror(),
        target,
        generator_contract: REFLECTION_NUISANCE_CONTRACT,
    };
    Ok(ReflectionNuisancePair {
        canonical,
        reflected,
    })
}

/// Ordered relation oracle.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DirectionTarget {
    Forward,
    Reverse,
}

/// One ordered query-key relation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DirectionReversalCase {
    pub pair_id: u64,
    /// Typed Development/Validation population identity.
    pub split: DataSplit,
    pub case_id: u64,
    pub query: Chiral6,
    pub key: Chiral6,
    pub target: DirectionTarget,
    pub generator_contract: &'static str,
}

/// Exact order-reversal pair.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DirectionReversalPair {
    pub forward: DirectionReversalCase,
    pub reverse: DirectionReversalCase,
}

/// Deterministically materialize one ordered relation and its reversal.
pub fn direction_reversal_pair(pair_id: u64) -> Result<DirectionReversalPair, Tdi24TaskError> {
    let local_base = pair_id
        .checked_mul(2)
        .filter(|value| *value < (1_u64 << 62))
        .ok_or(Tdi24TaskError::CaseIdOverflow)?;
    let prefix = 1_u64 << 62;
    let forward_id = prefix | local_base;
    let reverse_id = forward_id
        .checked_add(1)
        .ok_or(Tdi24TaskError::CaseIdOverflow)?;

    let offset = ((pair_id % 37) as f64 + 1.0) / 96.0;
    let query = Chiral6::new([1.0 + offset, -0.5, 0.75], [0.25, -1.125 - offset, 1.5])
        .map_err(Tdi24TaskError::Chiral)?;
    let key = Chiral6::new(
        [-0.625, 1.375 + offset, -1.25],
        [1.625 + offset, 0.5, -0.375],
    )
    .map_err(Tdi24TaskError::Chiral)?;

    Ok(DirectionReversalPair {
        forward: DirectionReversalCase {
            pair_id,
            split: DataSplit::Development,
            case_id: forward_id,
            query,
            key,
            target: DirectionTarget::Forward,
            generator_contract: DIRECTION_REVERSAL_CONTRACT,
        },
        reverse: DirectionReversalCase {
            pair_id,
            split: DataSplit::Development,
            case_id: reverse_id,
            query: key,
            key: query,
            target: DirectionTarget::Reverse,
            generator_contract: DIRECTION_REVERSAL_CONTRACT,
        },
    })
}

/// Target defined exclusively from parity-even construction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NonChiralTarget {
    ClassA,
    ClassB,
}

/// Negative-control case whose odd sectors are nuisance-only.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NonChiralControlCase {
    pub case_id: u64,
    /// Typed Development/Validation population identity.
    pub split: DataSplit,
    pub nuisance_id: u64,
    pub query: Chiral6,
    pub key: Chiral6,
    pub target: NonChiralTarget,
    pub generator_contract: &'static str,
}

/// Deterministically materialize one negative-control case.
///
/// Consecutive cases 2*n and 2*n+1 have opposite targets but exactly identical
/// query/key odd sectors. Only parity-even coordinates depend on the target.
pub fn non_chiral_control_case(case_id: u64) -> Result<NonChiralControlCase, Tdi24TaskError> {
    let nuisance_id = case_id / 2;
    let target = if case_id % 2 == 0 {
        NonChiralTarget::ClassA
    } else {
        NonChiralTarget::ClassB
    };
    let sign = match target {
        NonChiralTarget::ClassA => 1.0,
        NonChiralTarget::ClassB => -1.0,
    };
    let nuisance = ((nuisance_id % 43) as f64 + 1.0) / 112.0;

    let query_odd = [0.375 + nuisance, -0.875, 1.25 - nuisance];
    let key_odd = [-1.125, 0.625 + nuisance, 0.5];

    let query = Chiral6::new([sign * (1.0 + nuisance), 0.5, -0.75], query_odd)
        .map_err(Tdi24TaskError::Chiral)?;
    let key = Chiral6::new([1.0, sign * (0.625 + nuisance), 0.25], key_odd)
        .map_err(Tdi24TaskError::Chiral)?;

    Ok(NonChiralControlCase {
        case_id,
        split: DataSplit::Development,
        nuisance_id,
        query,
        key,
        target,
        generator_contract: NON_CHIRAL_CONTROL_CONTRACT,
    })
}

/// Bounded difficulty stratum independent of any model output.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct DifficultyLevel {
    /// Discrete level in `0..=DIFFICULTY_LEVEL_MAX`.
    pub level: u8,
}

impl DifficultyLevel {
    /// Construct a fail-closed level.
    pub fn new(level: u8) -> Result<Self, Tdi24TaskError> {
        if level > DIFFICULTY_LEVEL_MAX {
            return Err(Tdi24TaskError::DifficultyOutOfRange);
        }
        Ok(Self { level })
    }

    /// Deterministic stratum for a seed; never consults model output.
    pub fn from_seed(seed: u64) -> Self {
        Self {
            level: (seed % u64::from(DIFFICULTY_LEVEL_MAX + 1)) as u8,
        }
    }

    /// Positive finite magnitude scale for the stratum.
    ///
    /// Scales stay small so Stage-A finite arithmetic remains admissible.
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
pub fn difficulty_stratum(seed: u64) -> DifficultyStratum {
    DifficultyStratum {
        seed,
        level: DifficultyLevel::from_seed(seed),
        generator_contract: DIFFICULTY_STRATA_CONTRACT,
    }
}

/// Materialize a reflection-discriminative pair in one typed split.
pub fn reflection_discriminative_pair_in_split(
    pair_id: u64,
    split: DataSplit,
) -> Result<ReflectionDiscriminativePair, Tdi24TaskError> {
    let mut pair = reflection_discriminative_pair(pair_id)?;
    pair.right.split = split;
    pair.left.split = split;
    Ok(pair)
}

/// Materialize a reflection-nuisance pair in one typed split.
pub fn reflection_nuisance_pair_in_split(
    pair_id: u64,
    split: DataSplit,
) -> Result<ReflectionNuisancePair, Tdi24TaskError> {
    let mut pair = reflection_nuisance_pair(pair_id)?;
    pair.canonical.split = split;
    pair.reflected.split = split;
    Ok(pair)
}

/// Materialize a direction/reversal pair in one typed split.
pub fn direction_reversal_pair_in_split(
    pair_id: u64,
    split: DataSplit,
) -> Result<DirectionReversalPair, Tdi24TaskError> {
    let mut pair = direction_reversal_pair(pair_id)?;
    pair.forward.split = split;
    pair.reverse.split = split;
    Ok(pair)
}

/// Materialize a non-chiral control in one typed split.
pub fn non_chiral_control_case_in_split(
    case_id: u64,
    split: DataSplit,
) -> Result<NonChiralControlCase, Tdi24TaskError> {
    let mut case = non_chiral_control_case(case_id)?;
    case.split = split;
    Ok(case)
}

/// Task-generation failures.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tdi24TaskError {
    /// Pair id cannot be mapped to two disjoint member ids.
    CaseIdOverflow,
    /// The underlying chiral carrier rejected a generated fixture.
    Chiral(ChiralError),
    /// Difficulty level exceeded the bounded admissible range.
    DifficultyOutOfRange,
    /// Split identity label is not Development or Validation.
    UnknownSplitIdentity,
    /// Seed-domain label is not Development or Validation.
    UnknownSeedDomain,
    /// Mixed seeds collide across distinct declared domains.
    SeedDomainOverlap,
}

impl fmt::Display for Tdi24TaskError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CaseIdOverflow => formatter.write_str("TDI-24 case id overflow"),
            Self::Chiral(error) => write!(formatter, "generated chiral fixture invalid: {error}"),
            Self::DifficultyOutOfRange => {
                formatter.write_str("TDI-24 difficulty level out of bounded range")
            }
            Self::UnknownSplitIdentity => {
                formatter.write_str("TDI-24 unknown split identity label")
            }
            Self::UnknownSeedDomain => formatter.write_str("TDI-24 unknown seed domain label"),
            Self::SeedDomainOverlap => formatter.write_str("TDI-24 seed domain overlap"),
        }
    }
}

impl std::error::Error for Tdi24TaskError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::experimental::tdi24_chiral::observables;

    #[test]
    fn reflection_discriminative_pairs_are_deterministic_and_disjoint() {
        let first = reflection_discriminative_pair(7).unwrap();
        let second = reflection_discriminative_pair(7).unwrap();
        assert_eq!(first, second);
        assert_eq!(first.right.case_id, 14);
        assert_eq!(first.left.case_id, 15);
        assert_ne!(first.right.case_id, first.left.case_id);
        assert_eq!(first.right.pair_id, first.left.pair_id);
    }

    #[test]
    fn pair_members_are_exact_simultaneous_reflections_with_opposite_targets() {
        for pair_id in 0..64 {
            let pair = reflection_discriminative_pair(pair_id).unwrap();
            assert_eq!(pair.left.query, pair.right.query.mirror());
            assert_eq!(pair.left.key, pair.right.key.mirror());
            assert_eq!(pair.right.target, HandednessTarget::Right);
            assert_eq!(pair.left.target, HandednessTarget::Left);

            let right = observables(pair.right.query, pair.right.key).unwrap();
            let left = observables(pair.left.query, pair.left.key).unwrap();
            assert_eq!(left.direct, right.direct);
            assert_eq!(left.mirrored, right.mirrored);
            assert_eq!(left.chiral, -right.chiral);
            assert_ne!(right.chiral, 0.0);
        }
    }

    #[test]
    fn generator_contract_is_stable_and_overflow_fails_closed() {
        let pair = reflection_discriminative_pair(1).unwrap();
        assert_eq!(
            pair.right.generator_contract,
            REFLECTION_DISCRIMINATIVE_CONTRACT
        );
        assert_eq!(
            reflection_discriminative_pair(u64::MAX),
            Err(Tdi24TaskError::CaseIdOverflow)
        );
    }
    #[test]
    fn reflection_nuisance_pairs_are_deterministic_target_invariant_and_namespaced() {
        for pair_id in 0..64 {
            let pair = reflection_nuisance_pair(pair_id).unwrap();
            assert_eq!(pair, reflection_nuisance_pair(pair_id).unwrap());
            assert_eq!(pair.canonical.target, pair.reflected.target);
            assert_eq!(pair.reflected.query, pair.canonical.query.mirror());
            assert_eq!(pair.reflected.key, pair.canonical.key.mirror());
            assert_ne!(pair.canonical.case_id, pair.reflected.case_id);
            assert_ne!(
                pair.canonical.case_id,
                reflection_discriminative_pair(pair_id)
                    .unwrap()
                    .right
                    .case_id
            );
            assert_ne!(pair.canonical.case_id & NUISANCE_CASE_ID_PREFIX, 0);
        }
    }

    #[test]
    fn nuisance_reflection_changes_only_the_odd_observable_not_the_target() {
        for pair_id in 0..32 {
            let pair = reflection_nuisance_pair(pair_id).unwrap();
            let canonical = observables(pair.canonical.query, pair.canonical.key).unwrap();
            let reflected = observables(pair.reflected.query, pair.reflected.key).unwrap();
            assert_eq!(canonical.direct, reflected.direct);
            assert_eq!(canonical.mirrored, reflected.mirrored);
            assert_eq!(canonical.chiral, -reflected.chiral);
            assert_eq!(pair.canonical.target, pair.reflected.target);
        }
    }

    #[test]
    fn nuisance_case_id_overflow_fails_closed() {
        assert_eq!(
            reflection_nuisance_pair(u64::MAX),
            Err(Tdi24TaskError::CaseIdOverflow)
        );
    }
    #[test]
    fn direction_reversal_pairs_swap_order_and_oracle_without_mutating_values() {
        for pair_id in 0..64 {
            let pair = direction_reversal_pair(pair_id).unwrap();
            assert_eq!(pair.reverse.query, pair.forward.key);
            assert_eq!(pair.reverse.key, pair.forward.query);
            assert_eq!(pair.forward.target, DirectionTarget::Forward);
            assert_eq!(pair.reverse.target, DirectionTarget::Reverse);
            assert_ne!(pair.forward.case_id, pair.reverse.case_id);

            let forward = observables(pair.forward.query, pair.forward.key).unwrap();
            let reverse = observables(pair.reverse.query, pair.reverse.key).unwrap();
            assert_eq!(forward.direct, reverse.direct);
            assert_eq!(forward.mirrored, reverse.mirrored);
            assert_eq!(forward.chiral, -reverse.chiral);
            assert_ne!(forward.chiral, 0.0);
        }
    }

    #[test]
    fn direction_reversal_generator_is_deterministic_and_fails_closed_on_id_overflow() {
        assert_eq!(
            direction_reversal_pair(19).unwrap(),
            direction_reversal_pair(19).unwrap()
        );
        assert_eq!(
            direction_reversal_pair(u64::MAX),
            Err(Tdi24TaskError::CaseIdOverflow)
        );
        assert_eq!(
            direction_reversal_pair(1)
                .unwrap()
                .forward
                .generator_contract,
            DIRECTION_REVERSAL_CONTRACT
        );
    }
    #[test]
    fn non_chiral_control_pairs_opposite_targets_with_identical_odd_sectors() {
        for nuisance_id in 0..64 {
            let a = non_chiral_control_case(2 * nuisance_id).unwrap();
            let b = non_chiral_control_case(2 * nuisance_id + 1).unwrap();
            assert_eq!(a.nuisance_id, b.nuisance_id);
            assert_eq!(a.query.odd(), b.query.odd());
            assert_eq!(a.key.odd(), b.key.odd());
            assert_ne!(a.target, b.target);
            assert_ne!(a.query.even(), b.query.even());
            assert_eq!(a.generator_contract, NON_CHIRAL_CONTROL_CONTRACT);
            assert_eq!(b.generator_contract, NON_CHIRAL_CONTROL_CONTRACT);
        }
    }

    #[test]
    fn non_chiral_target_is_reflection_invariant_by_construction() {
        for case_id in 0..128 {
            let case = non_chiral_control_case(case_id).unwrap();
            assert_eq!(case.query.mirror().even(), case.query.even());
            assert_eq!(case.key.mirror().even(), case.key.even());
            assert_eq!(
                case.target,
                non_chiral_control_case(case_id).unwrap().target
            );
        }
    }

    #[test]
    fn odd_sector_carries_no_target_bit_within_paired_nuisance_strata() {
        for nuisance_id in 0..128 {
            let class_a = non_chiral_control_case(2 * nuisance_id).unwrap();
            let class_b = non_chiral_control_case(2 * nuisance_id + 1).unwrap();
            assert_eq!(class_a.query.odd(), class_b.query.odd());
            assert_eq!(class_a.key.odd(), class_b.key.odd());
        }
    }

    #[test]
    fn difficulty_strata_are_bounded_deterministic_and_model_independent() {
        for seed in 0..64 {
            let stratum = difficulty_stratum(seed);
            assert_eq!(stratum, difficulty_stratum(seed));
            assert!(stratum.level.level <= DIFFICULTY_LEVEL_MAX);
            assert_eq!(stratum.generator_contract, DIFFICULTY_STRATA_CONTRACT);
            assert_eq!(stratum.level, DifficultyLevel::from_seed(seed));
            let scale = stratum.level.magnitude_scale();
            assert!(scale.is_finite());
            assert!(scale >= 1.0);
            assert!(scale <= 1.0 + f64::from(DIFFICULTY_LEVEL_MAX) * 0.25);
        }
        assert_eq!(
            DifficultyLevel::new(DIFFICULTY_LEVEL_MAX + 1),
            Err(Tdi24TaskError::DifficultyOutOfRange)
        );
        // Every admissible level appears in a full residue cycle.
        let mut seen = [false; 4];
        for seed in 0..4 {
            seen[difficulty_stratum(seed).level.level as usize] = true;
        }
        assert_eq!(seen, [true, true, true, true]);
    }

    #[test]
    fn difficulty_scale_is_independent_of_existing_family_oracles() {
        for pair_id in 0..32 {
            let stratum = difficulty_stratum(pair_id);
            let pair = reflection_discriminative_pair(pair_id).unwrap();
            assert_eq!(pair.right.target, HandednessTarget::Right);
            assert_eq!(pair.left.target, HandednessTarget::Left);
            assert_eq!(
                pair.right.query,
                reflection_discriminative_pair(pair_id).unwrap().right.query
            );
            let _ = stratum.level.magnitude_scale();
        }
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
            Err(Tdi24TaskError::UnknownSplitIdentity)
        );
        assert_eq!(
            DataSplit::parse("final"),
            Err(Tdi24TaskError::UnknownSplitIdentity)
        );
    }

    #[test]
    fn every_phase_b_family_embeds_the_requested_split() {
        for split in [DataSplit::Development, DataSplit::Validation] {
            let discriminative = reflection_discriminative_pair_in_split(3, split).unwrap();
            assert_eq!(discriminative.right.split, split);
            assert_eq!(discriminative.left.split, split);

            let nuisance = reflection_nuisance_pair_in_split(4, split).unwrap();
            assert_eq!(nuisance.canonical.split, split);
            assert_eq!(nuisance.reflected.split, split);

            let direction = direction_reversal_pair_in_split(5, split).unwrap();
            assert_eq!(direction.forward.split, split);
            assert_eq!(direction.reverse.split, split);

            let control = non_chiral_control_case_in_split(6, split).unwrap();
            assert_eq!(control.split, split);
        }
        assert_eq!(
            reflection_discriminative_pair(1).unwrap().right.split,
            DataSplit::Development
        );
    }

    #[test]
    fn split_choice_does_not_change_task_payload_or_oracle() {
        let dev = reflection_discriminative_pair_in_split(11, DataSplit::Development).unwrap();
        let val = reflection_discriminative_pair_in_split(11, DataSplit::Validation).unwrap();
        assert_eq!(dev.right.query, val.right.query);
        assert_eq!(dev.right.key, val.right.key);
        assert_eq!(dev.right.target, val.right.target);
        assert_eq!(dev.right.case_id, val.right.case_id);
        assert_ne!(dev.right.split, val.right.split);

        let n_dev = reflection_nuisance_pair_in_split(9, DataSplit::Development).unwrap();
        let n_val = reflection_nuisance_pair_in_split(9, DataSplit::Validation).unwrap();
        assert_eq!(n_dev.canonical.target, n_val.canonical.target);
        assert_eq!(n_dev.canonical.query, n_val.canonical.query);

        let d_dev = direction_reversal_pair_in_split(8, DataSplit::Development).unwrap();
        let d_val = direction_reversal_pair_in_split(8, DataSplit::Validation).unwrap();
        assert_eq!(d_dev.forward.target, d_val.forward.target);
        assert_eq!(d_dev.forward.query, d_val.forward.query);

        let c_dev = non_chiral_control_case_in_split(7, DataSplit::Development).unwrap();
        let c_val = non_chiral_control_case_in_split(7, DataSplit::Validation).unwrap();
        assert_eq!(c_dev.target, c_val.target);
        assert_eq!(c_dev.query, c_val.query);
        assert_ne!(c_dev.split, c_val.split);
    }

    #[test]
    fn protected_label_api_hides_targets_from_inference_callbacks() {
        let pair = reflection_discriminative_pair(21).unwrap();
        let labeled = seal_reflection_discriminative(&pair.right);
        assert_eq!(
            labeled.inference_view().label_contract,
            PROTECTED_LABEL_CONTRACT
        );
        assert_eq!(
            labeled.inference_view().family,
            TaskFamily::ReflectionDiscriminative
        );
        assert_eq!(labeled.inference_view().query, pair.right.query);
        assert_eq!(labeled.inference_view().key, pair.right.key);
        assert_eq!(labeled.inference_view().case_id, pair.right.case_id);
        assert_eq!(labeled.inference_view().group_id, pair.right.pair_id);
        assert_eq!(
            *labeled.protected_label().reveal_for_evaluation(),
            HandednessTarget::Right
        );

        let observed = run_inference_callback(&labeled, |view| {
            // Callback surface exposes carriers/identity only.
            assert_eq!(view.query, pair.right.query);
            assert_eq!(view.key, pair.right.key);
            format!("{:?}", view)
        });
        assert!(!observed.contains("target"));
        assert!(!observed.contains("Right"));
        assert!(!observed.contains("Left"));
        assert!(observed.contains("ReflectionDiscriminative"));
    }

    #[test]
    fn every_phase_b_family_seals_targets_outside_inference_views() {
        let discriminative =
            seal_reflection_discriminative(&reflection_discriminative_pair(3).unwrap().left);
        assert_eq!(
            *discriminative.protected_label().reveal_for_evaluation(),
            HandednessTarget::Left
        );
        assert_eq!(
            discriminative.inference_view().family.generator_contract(),
            REFLECTION_DISCRIMINATIVE_CONTRACT
        );

        let nuisance = seal_reflection_nuisance(&reflection_nuisance_pair(4).unwrap().canonical);
        assert_eq!(
            run_inference_callback(&nuisance, |view| {
                assert_eq!(view.split, DataSplit::Development);
                view.family
            }),
            TaskFamily::ReflectionNuisance
        );
        assert_eq!(
            *nuisance.protected_label().reveal_for_evaluation(),
            reflection_nuisance_pair(4).unwrap().canonical.target
        );

        let direction = seal_direction_reversal(&direction_reversal_pair(5).unwrap().reverse);
        assert_eq!(
            *direction.protected_label().reveal_for_evaluation(),
            DirectionTarget::Reverse
        );
        assert_eq!(
            run_inference_callback(&direction, |view| view.case_id),
            direction_reversal_pair(5).unwrap().reverse.case_id
        );

        let control = seal_non_chiral_control(&non_chiral_control_case(6).unwrap());
        assert_eq!(
            control.inference_view().group_id,
            non_chiral_control_case(6).unwrap().nuisance_id
        );
        assert_eq!(
            *control.protected_label().reveal_for_evaluation(),
            non_chiral_control_case(6).unwrap().target
        );
        let leaked = run_inference_callback(&control, |view| format!("{view:?}"));
        assert!(!leaked.contains("ClassA"));
        assert!(!leaked.contains("ClassB"));
        assert!(!leaked.contains("target"));
    }

    #[test]
    fn sealed_inference_view_preserves_split_without_exposing_oracle() {
        let case = reflection_discriminative_pair_in_split(11, DataSplit::Validation)
            .unwrap()
            .right;
        let labeled = seal_reflection_discriminative(&case);
        assert_eq!(labeled.inference_view().split, DataSplit::Validation);
        assert_eq!(
            *labeled.protected_label().reveal_for_evaluation(),
            case.target
        );
        assert_eq!(TaskFamily::NonChiralControl.as_str(), "non_chiral_control");
    }

    #[test]
    fn seed_registry_is_deterministic_and_domain_separated() {
        let families = [
            TaskFamily::ReflectionDiscriminative,
            TaskFamily::ReflectionNuisance,
            TaskFamily::DirectionReversal,
            TaskFamily::NonChiralControl,
        ];
        for family in families {
            for local in 0..64 {
                let development = register_seed(SeedDomain::Development, family, local);
                let validation = register_seed(SeedDomain::Validation, family, local);
                assert_eq!(
                    development,
                    register_seed(SeedDomain::Development, family, local)
                );
                assert_eq!(development.registry_contract, SEED_REGISTRY_CONTRACT);
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
            Err(Tdi24TaskError::UnknownSeedDomain)
        );
        assert_eq!(
            SeedDomain::parse("final"),
            Err(Tdi24TaskError::UnknownSeedDomain)
        );
        assert_eq!(SeedDomain::Development.as_str(), "development");
    }

    #[test]
    fn declared_seed_domains_have_no_mixed_seed_overlap() {
        let families = [
            TaskFamily::ReflectionDiscriminative,
            TaskFamily::ReflectionNuisance,
            TaskFamily::DirectionReversal,
            TaskFamily::NonChiralControl,
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
                family: TaskFamily::ReflectionDiscriminative,
                local_seed: 0,
                mixed_seed: 0xDEAD_BEEF,
                registry_contract: SEED_REGISTRY_CONTRACT,
            },
            RegisteredSeed {
                domain: SeedDomain::Validation,
                family: TaskFamily::ReflectionDiscriminative,
                local_seed: 1,
                mixed_seed: 0xDEAD_BEEF,
                registry_contract: SEED_REGISTRY_CONTRACT,
            },
        ];
        assert_eq!(
            assert_seed_domain_disjointness(&forged),
            Err(Tdi24TaskError::SeedDomainOverlap)
        );
    }
}
