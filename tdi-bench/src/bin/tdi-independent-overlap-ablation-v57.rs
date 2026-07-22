//! TDI-5.7 generator-robustness evaluator (does the {O1,O2}-beyond-{contraction
//! + spectral} signal survive a family of exact generators?).
//!
//! This file is derived from the frozen TDI-5.6 evaluator
//! (`tdi-independent-overlap-ablation-v56.rs`), itself derived from the frozen
//! TDI-5.5 … TDI-5.2 evaluators. TDI-5.1 … TDI-5.6 remain frozen and untouched.
//! TDI-5.7 reuses their frozen exact candidate analysis, exact
//! overlap/total-variation primitives, the two exact contraction descriptors
//! (delta, delta_bar), the two exact spectral moments (s2, s3), the layouts
//! CK/SK/SKT, ridge machinery, deterministic bootstrap engine and the four-way
//! Beneficial/Equivalent/Harmful/Inconclusive classification logic without
//! altering any of them.
//!
//! TDI-5.7 makes exactly the scientific additions its preregistration
//! (`docs/TDI-5.7-GENERATOR-ROBUSTNESS-PREREGISTRATION.md`) declares and
//! nothing else — it changes only the *generator*, holding the entire TDI-5.6
//! measurement apparatus fixed:
//!
//!   * a family of four deterministic, exact successor-mask generation rules —
//!     `F0Base` (the inherited uniform generator), `F1Sparse` (out-degree
//!     1..2), `F2Dense` (all states minus 0..1 excluded), `F3Local`
//!     (Hamming-<=1 neighbourhood) — each assembled into a system by the
//!     unchanged frozen `build_system`, each a deterministic function of the
//!     candidate seed via the inherited `splitmix64` chain, and each
//!     guaranteeing a non-empty successor set (a total, exact Noop kernel);
//!   * 48 fresh, independent seed reservations (4 families x 3 seed blocks x 4
//!     populations, 480,000 accepted records), disjoint from the TDI-5.6 blocks
//!     J/K/L and all earlier blocks, with fresh per-block and per-family
//!     bootstrap seeds;
//!   * criterion TDI-5.7A (replication: SKT vs SK classifies Beneficial at the
//!     focal horizons U3 and U6 for every family), TDI-5.7B (across-family
//!     effect-size heterogeneity), TDI-5.7C (F0->F1 transfer, descriptive) and
//!     TDI-5.7D (per-family exact-descriptor drift, descriptive); per-family
//!     per-horizon SKT-vs-SK reductions across the dense grid U3..U8 are
//!     reported.
//!
//! The full run is gated behind an explicit, exact human confirmation
//! environment variable (see `run_full_experiment` and
//! `tdi57_full_run_confirmed`). No commit, test or CI run supplies that
//! token.

use tdi_core::{
    Action, ExactRatio, State, TableSystem, analyze_branching_recovery, distribution_overlap,
    explore, uniform_branching_path_entropy_bits, uniform_branching_state_distribution,
};

const OBSERVATION_HORIZON: usize = 2;

// Dense target-horizon grid, inherited unchanged from TDI-5.5 (Section 3), so
// the overlaps' marginal value is sampled at every integer horizon 3..=8.
const TARGET_HORIZONS: [usize; 6] = [3, 4, 5, 6, 7, 8];
const TARGET_HORIZON_COUNT: usize = TARGET_HORIZONS.len();
const PRIMARY_HORIZON: usize = 6;
const PRIMARY_HORIZON_INDEX: usize = 3;

// The two focal horizons at which TDI-5.7A/5.7B classify: U3 (near, where
// TDI-5.4B found a short-horizon benefit) and the primary U6.
const FOCAL_HORIZONS: [usize; 2] = [3, 6];
const FOCAL_HORIZON_COUNT: usize = FOCAL_HORIZONS.len();

const TRAIN_WIDTH_3: u8 = 3;
const TRAIN_WIDTH_4: u8 = 4;
// Widths 5 and 6 remain supported by the inherited frozen generator and its
// exact cardinality/budget machinery, but TDI-5.7 generates no populations
// at those widths (Section 8): there are no OOD populations.
const WIDTH_5: u8 = 5;
const WIDTH_6: u8 = 6;

const TRAIN_WIDTH_3_SYSTEMS: usize = 15_000;
const TRAIN_WIDTH_4_SYSTEMS: usize = 15_000;
const HOLDOUT_WIDTH_3_SYSTEMS: usize = 5_000;
const HOLDOUT_WIDTH_4_SYSTEMS: usize = 5_000;

// TDI-5.7 runs the inherited 3-block per-generator machinery once per
// generator family (Section 7). SEED_BLOCK_COUNT is the number of blocks
// *within a family*; the four families give 12 blocks and 48 reservations.
const GENERATOR_FAMILY_COUNT: usize = 4;
const SEED_BLOCK_COUNT: usize = 3;
const POPULATIONS_PER_SEED_BLOCK: usize = 4;
const TOTAL_SEED_RESERVATIONS: usize =
    GENERATOR_FAMILY_COUNT * SEED_BLOCK_COUNT * POPULATIONS_PER_SEED_BLOCK;

const BASELINE_FEATURE_COUNT: usize = 13;
const EARLY_OVERLAP_FEATURE_COUNT: usize = 2;
// Exact contraction descriptors of the one-step Noop kernel, inherited
// unchanged from TDI-5.5 Section 5: the Dobrushin coefficient and the mean
// pairwise total variation. Both are exact rationals, computed per candidate
// system.
const CONTRACTION_FEATURE_COUNT: usize = 2;
// Exact spectral moments of the one-step Noop kernel (TDI-5.7 Section 5):
// s2 = trace(P^2) and s3 = trace(P^3), computed per candidate system as
// closed-walk sums of unit fractions with a single final rounding.
const SPECTRAL_FEATURE_COUNT: usize = 2;

// Linear layouts, inherited from TDI-5.2/5.3/5.4/5.5. In TDI-5.7 they are
// exploratory only (Section 6) and determine no confirmatory criterion.
const B0_FEATURE_COUNT: usize = BASELINE_FEATURE_COUNT;
const B1_FEATURE_COUNT: usize = BASELINE_FEATURE_COUNT + 1;
const B2_FEATURE_COUNT: usize = BASELINE_FEATURE_COUNT + 1;
const B12_FEATURE_COUNT: usize = BASELINE_FEATURE_COUNT + EARLY_OVERLAP_FEATURE_COUNT;
const BD_FEATURE_COUNT: usize = BASELINE_FEATURE_COUNT + 1;

// Confirmatory linear layouts (Section 6). CK (inherited from TDI-5.5) adds
// the two exact contraction descriptors to the baseline; SK additionally adds
// the two exact spectral moments; SKT additionally adds the two early
// overlaps. SK minus CK isolates the spectral moments' marginal value beyond
// contraction (criterion 5.7B); SKT minus SK isolates the overlaps' marginal
// value *after* both the contraction descriptors and the spectral moments are
// already present (criteria 5.7A, 5.7C).
//   CK  = baseline + delta + delta_bar                          (13 + 2 = 15)
//   SK  = baseline + delta + delta_bar + s2 + s3                (13 + 4 = 17)
//   SKT = baseline + delta + delta_bar + s2 + s3 + O1 + O2      (13 + 6 = 19)
const CK_FEATURE_COUNT: usize = BASELINE_FEATURE_COUNT + CONTRACTION_FEATURE_COUNT;
const SK_FEATURE_COUNT: usize =
    BASELINE_FEATURE_COUNT + CONTRACTION_FEATURE_COUNT + SPECTRAL_FEATURE_COUNT;
const SKT_FEATURE_COUNT: usize = BASELINE_FEATURE_COUNT
    + CONTRACTION_FEATURE_COUNT
    + SPECTRAL_FEATURE_COUNT
    + EARLY_OVERLAP_FEATURE_COUNT;

const MODEL_LAYOUT_COUNT: usize = 8;

const RIDGE_LAMBDA: f64 = 1.0;
const BOOTSTRAP_REPLICATES: usize = 4_000;
// Fresh per-family stratified-aggregate bootstrap seeds (TDI-5.7 Section 9),
// disjoint from every TDI-5.2 … 5.6 bootstrap seed. Each family aggregates its
// own three blocks with seed base + family index.
const AGGREGATE_BOOTSTRAP_SEED_BASE: u64 = 0x5444_4935_3700_4700;

fn family_aggregate_bootstrap_seed(family: GeneratorFamily) -> u64 {
    AGGREGATE_BOOTSTRAP_SEED_BASE + family.index()
}

const MAX_SUPPORTED_WIDTH: u8 = 6;
const WIDTH_3_ATTEMPT_MULTIPLIER: usize = 64;
const WIDTH_4_ATTEMPT_MULTIPLIER: usize = 96;
const WIDTH_5_ATTEMPT_MULTIPLIER: usize = 128;
const WIDTH_6_ATTEMPT_MULTIPLIER: usize = 256;
const WIDTH_3_NO_PROGRESS_LIMIT: usize = 25_000;
const WIDTH_4_NO_PROGRESS_LIMIT: usize = 50_000;
const WIDTH_5_NO_PROGRESS_LIMIT: usize = 75_000;
const WIDTH_6_NO_PROGRESS_LIMIT: usize = 100_000;

/// A seed block is one of the `SEED_BLOCK_COUNT` blocks within a generator
/// family (Section 8). Its population seeds and bootstrap seed are computed
/// deterministically from `(family, block)`; no block table is stored.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct SeedBlockId {
    family: GeneratorFamily,
    block: u8,
}

impl SeedBlockId {
    fn label(self) -> String {
        format!("{}/b{}", self.family.label(), self.block)
    }

    /// `base(f, b) = 1.4e9 + f·300e6 + b·100e6` (Section 8). The four
    /// populations start at this base + `{0, 10, 20, 30}·1e6`.
    fn population_base_seed(self) -> u64 {
        1_400_000_000 + self.family.index() * 300_000_000 + u64::from(self.block) * 100_000_000
    }

    /// `0x5444_4935_3700_0000 + (SEED_BLOCK_COUNT·f + b) + 1` (Section 9).
    fn bootstrap_seed(self) -> u64 {
        0x5444_4935_3700_0000
            + (SEED_BLOCK_COUNT as u64 * self.family.index() + u64::from(self.block))
            + 1
    }
}

/// The `SEED_BLOCK_COUNT` blocks of one family, in frozen order. The inherited
/// per-generator sub-pipeline runs over this array once per family.
fn frozen_block_order(family: GeneratorFamily) -> [SeedBlockId; SEED_BLOCK_COUNT] {
    std::array::from_fn(|block| SeedBlockId {
        family,
        block: block as u8,
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PopulationKind {
    TrainingWidth3,
    HoldoutWidth3,
    TrainingWidth4,
    HoldoutWidth4,
}

impl PopulationKind {
    const ALL: [Self; POPULATIONS_PER_SEED_BLOCK] = [
        Self::TrainingWidth3,
        Self::HoldoutWidth3,
        Self::TrainingWidth4,
        Self::HoldoutWidth4,
    ];

    const fn label(self) -> &'static str {
        match self {
            Self::TrainingWidth3 => "training-w3",
            Self::HoldoutWidth3 => "holdout-w3",
            Self::TrainingWidth4 => "training-w4",
            Self::HoldoutWidth4 => "holdout-w4",
        }
    }

    const fn width(self) -> u8 {
        match self {
            Self::TrainingWidth3 | Self::HoldoutWidth3 => TRAIN_WIDTH_3,
            Self::TrainingWidth4 | Self::HoldoutWidth4 => TRAIN_WIDTH_4,
        }
    }

    const fn target_count(self) -> usize {
        match self {
            Self::TrainingWidth3 => TRAIN_WIDTH_3_SYSTEMS,
            Self::HoldoutWidth3 => HOLDOUT_WIDTH_3_SYSTEMS,
            Self::TrainingWidth4 => TRAIN_WIDTH_4_SYSTEMS,
            Self::HoldoutWidth4 => HOLDOUT_WIDTH_4_SYSTEMS,
        }
    }

    /// Offset from the block base seed: 0 / 10M / 20M / 30M (Section 8).
    const fn seed_offset(self) -> u64 {
        match self {
            Self::TrainingWidth3 => 0,
            Self::HoldoutWidth3 => 10_000_000,
            Self::TrainingWidth4 => 20_000_000,
            Self::HoldoutWidth4 => 30_000_000,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct PopulationSpec {
    seed_block: SeedBlockId,
    population: PopulationKind,
    width: u8,
    seed: u64,
    target_count: usize,
}

impl PopulationSpec {
    fn from_block(seed_block: SeedBlockId, population: PopulationKind) -> Self {
        Self {
            seed_block,
            population,
            width: population.width(),
            seed: seed_block.population_base_seed() + population.seed_offset(),
            target_count: population.target_count(),
        }
    }

    fn family(self) -> GeneratorFamily {
        self.seed_block.family
    }
}

fn population_specs() -> [PopulationSpec; TOTAL_SEED_RESERVATIONS] {
    let default = PopulationSpec::from_block(
        SeedBlockId {
            family: GeneratorFamily::F0Base,
            block: 0,
        },
        PopulationKind::ALL[0],
    );
    let mut specs = [default; TOTAL_SEED_RESERVATIONS];
    let mut index = 0_usize;

    for family in GeneratorFamily::ALL {
        for block in 0..SEED_BLOCK_COUNT {
            let seed_block = SeedBlockId {
                family,
                block: block as u8,
            };
            for population in PopulationKind::ALL {
                specs[index] = PopulationSpec::from_block(seed_block, population);
                index += 1;
            }
        }
    }

    specs
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(usize)]
enum FeatureLayout {
    // Linear layouts B0..BD are exploratory in TDI-5.7. Their discriminants
    // (0..4) are preserved so `layout as usize` indexing is unchanged from
    // TDI-5.2/5.3/5.4/5.5. The confirmatory layouts CK/SK/SKT follow.
    B0,
    B1,
    B2,
    B12,
    BD,
    Ck,
    Sk,
    Skt,
}

impl FeatureLayout {
    const ALL: [Self; MODEL_LAYOUT_COUNT] = [
        Self::B0,
        Self::B1,
        Self::B2,
        Self::B12,
        Self::BD,
        Self::Ck,
        Self::Sk,
        Self::Skt,
    ];

    const fn label(self) -> &'static str {
        match self {
            Self::B0 => "B0 — BASELINE",
            Self::B1 => "B1 — BASELINE + O1",
            Self::B2 => "B2 — BASELINE + O2",
            Self::B12 => "B12 — BASELINE + O1 + O2",
            Self::BD => "BD — BASELINE + (O2 - O1), EXPLORATOIRE",
            Self::Ck => "CK — BASELINE + δ + δ̄ (contraction)",
            Self::Sk => "SK — BASELINE + δ + δ̄ + s2 + s3 (contraction + spectral)",
            Self::Skt => {
                "SKT — BASELINE + δ + δ̄ + s2 + s3 + O1 + O2 (contraction + spectral + TDI)"
            }
        }
    }

    const fn feature_count(self) -> usize {
        match self {
            Self::B0 => B0_FEATURE_COUNT,
            Self::B1 => B1_FEATURE_COUNT,
            Self::B2 => B2_FEATURE_COUNT,
            Self::B12 => B12_FEATURE_COUNT,
            Self::BD => BD_FEATURE_COUNT,
            Self::Ck => CK_FEATURE_COUNT,
            Self::Sk => SK_FEATURE_COUNT,
            Self::Skt => SKT_FEATURE_COUNT,
        }
    }
}

#[derive(Clone, Debug)]
struct Record {
    baseline: [f64; BASELINE_FEATURE_COUNT],
    early_overlap: [f64; EARLY_OVERLAP_FEATURE_COUNT],
    contraction: [f64; CONTRACTION_FEATURE_COUNT],
    spectral: [f64; SPECTRAL_FEATURE_COUNT],
    overlaps: [f64; TARGET_HORIZON_COUNT],
    targets_u: [f64; TARGET_HORIZON_COUNT],
}

#[derive(Clone, Debug)]
struct RidgeModel {
    means: Vec<f64>,
    scales: Vec<f64>,
    coefficients: Vec<f64>,
}

#[derive(Clone, Debug)]
struct HorizonModels {
    models: Vec<RidgeModel>,
}

impl HorizonModels {
    fn get(&self, horizon_index: usize, layout: FeatureLayout) -> &RidgeModel {
        let index = horizon_index * MODEL_LAYOUT_COUNT + layout as usize;

        &self.models[index]
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Metrics {
    mse: f64,
    mae: f64,
    r_squared: f64,
    spearman: f64,
    bias: f64,
    observed_mean: f64,
    predicted_mean: f64,
    calibration_intercept: f64,
    calibration_slope: f64,
    zero_fraction: f64,
    one_fraction: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct ConfidenceInterval {
    lower: f64,
    median: f64,
    upper: f64,
}

/// The four exact generator families (TDI-5.7 Section 5). Each is a
/// deterministic rule for filling a state's successor mask from the
/// `splitmix64` chain; only the rule differs, everything downstream is
/// inherited. Every rule guarantees a non-empty successor set.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(usize)]
enum GeneratorFamily {
    F0Base,
    F1Sparse,
    F2Dense,
    F3Local,
}

impl GeneratorFamily {
    const ALL: [Self; GENERATOR_FAMILY_COUNT] =
        [Self::F0Base, Self::F1Sparse, Self::F2Dense, Self::F3Local];

    const fn label(self) -> &'static str {
        match self {
            Self::F0Base => "F0-base",
            Self::F1Sparse => "F1-sparse",
            Self::F2Dense => "F2-dense",
            Self::F3Local => "F3-local",
        }
    }

    const fn index(self) -> u64 {
        self as u64
    }

    /// One-line summary of the family's successor-mask rule (Section 5),
    /// printed in the required raw output (Section 17, "the four family rules").
    const fn rule_description(self) -> &'static str {
        match self {
            Self::F0Base => {
                "uniforme sur tous les sous-ensembles successeurs non vides : \
                 mask = d0 % (2^states − 1) + 1 (générateur 5.6 inchangé)"
            }
            Self::F1Sparse => {
                "faible degré sortant d ∈ {1, 2} : d successeurs distincts \
                 tirés par rejet dans la chaîne splitmix64"
            }
            Self::F2Dense => {
                "fort degré sortant : tous les états, moins e ∈ {0, 1} bit(s) exclu(s)"
            }
            Self::F3Local => {
                "voisinage local (Hamming ≤ 1) : sous-ensemble de \
                 {s, s⊕1, s⊕2, …, s⊕2^(width−1)}, self forcé si le tirage est vide"
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct AttemptContext {
    family: GeneratorFamily,
    width: u8,
    seed: u64,
    attempt_index: usize,
}

impl AttemptContext {
    const fn new(family: GeneratorFamily, width: u8, seed: u64, attempt_index: usize) -> Self {
        Self {
            family,
            width,
            seed,
            attempt_index,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FailureCategory {
    Arithmetic,
    Cardinality,
    Structural,
    DynamicAnalysis,
    UnsupportedWidth,
    SeedRange,
    AttemptBudget,
    NoProgress,
    InvalidConfiguration,
}

impl std::fmt::Display for FailureCategory {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let label = match self {
            Self::Arithmetic => "arithmetic",
            Self::Cardinality => "cardinality",
            Self::Structural => "structural",
            Self::DynamicAnalysis => "dynamic-analysis",
            Self::UnsupportedWidth => "unsupported-width",
            Self::SeedRange => "seed-range",
            Self::AttemptBudget => "attempt-budget",
            Self::NoProgress => "no-progress",
            Self::InvalidConfiguration => "invalid-configuration",
        };

        formatter.write_str(label)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct EvaluationError {
    context: AttemptContext,
    category: FailureCategory,
    message: String,
}

impl EvaluationError {
    fn new(context: AttemptContext, category: FailureCategory, message: impl Into<String>) -> Self {
        Self {
            context,
            category,
            message: message.into(),
        }
    }
}

impl std::fmt::Display for EvaluationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "{} failure at width {}, seed {}, attempt {}: {}",
            self.category,
            self.context.width,
            self.context.seed,
            self.context.attempt_index,
            self.message
        )
    }
}

impl std::error::Error for EvaluationError {}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Cardinality {
    Exact(u128),
    TooLarge { width: u8, exponent: u128 },
    Invalid { width: u8, reason: &'static str },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum RejectionReason {
    ObservationFullyRecovered,
    InvalidObservationGeometry,
    TargetFullyRecovered { horizon: usize },
    InvalidTargetGeometry { horizon: usize },
    InvalidTransformedTarget { horizon: usize },
    NonFiniteFeature,
}

impl std::fmt::Display for RejectionReason {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ObservationFullyRecovered => formatter.write_str("observation-fully-recovered"),
            Self::InvalidObservationGeometry => formatter.write_str("invalid-observation-geometry"),
            Self::TargetFullyRecovered { horizon } => {
                write!(formatter, "target-fully-recovered-h{horizon}")
            }
            Self::InvalidTargetGeometry { horizon } => {
                write!(formatter, "invalid-target-geometry-h{horizon}")
            }
            Self::InvalidTransformedTarget { horizon } => {
                write!(formatter, "invalid-transformed-target-h{horizon}")
            }
            Self::NonFiniteFeature => formatter.write_str("non-finite-feature"),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct RejectionCounts {
    counts: std::collections::BTreeMap<RejectionReason, usize>,
}

impl RejectionCounts {
    fn record(&mut self, reason: RejectionReason) {
        let count = self.counts.entry(reason).or_insert(0);

        *count = count
            .checked_add(1)
            .expect("rejection count cannot overflow usize");
    }

    fn total(&self) -> usize {
        self.counts.values().copied().sum()
    }

    fn summary(&self) -> String {
        if self.counts.is_empty() {
            return "none".to_owned();
        }

        self.counts
            .iter()
            .map(|(reason, count)| format!("{reason}={count}"))
            .collect::<Vec<_>>()
            .join(",")
    }
}

#[derive(Clone, Debug)]
enum CandidateOutcome {
    // Boxed so the accepted variant (a full `Record`) does not dominate the
    // enum size (clippy::large_enum_variant); the record grew with the
    // contraction descriptors and the denser horizon grid.
    Accepted(Box<Record>),
    Rejected(RejectionReason),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct GenerationLimits {
    max_attempts: usize,
    no_progress_limit: usize,
}

#[derive(Clone, Debug)]
struct GenerationReport {
    records: Vec<Record>,
    next_seed: u64,
    excluded: usize,
    rejections: RejectionCounts,
    attempts: usize,
    limits: GenerationLimits,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct GenerationProgress {
    accepted: usize,
    excluded: usize,
    rejections: RejectionCounts,
    target_count: usize,
    limits: GenerationLimits,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct TerminationDiagnostic {
    context: AttemptContext,
    category: FailureCategory,
    progress: GenerationProgress,
    message: String,
}

impl TerminationDiagnostic {
    fn new(
        context: AttemptContext,
        category: FailureCategory,
        progress: GenerationProgress,
        message: impl Into<String>,
    ) -> Self {
        Self {
            context,
            category,
            progress,
            message: message.into(),
        }
    }
}

impl std::fmt::Display for TerminationDiagnostic {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "{} termination at width {}, seed {}, attempt {}: {}; accepted={}, excluded={}, rejections=[{}], target={}, max_attempts={}, no_progress_limit={}",
            self.category,
            self.context.width,
            self.context.seed,
            self.context.attempt_index,
            self.message,
            self.progress.accepted,
            self.progress.excluded,
            self.progress.rejections.summary(),
            self.progress.target_count,
            self.progress.limits.max_attempts,
            self.progress.limits.no_progress_limit
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum GenerationError {
    Evaluation(EvaluationError),
    // The diagnostic variants are boxed so `GenerationError` stays small: the
    // per-family `AttemptContext` carried by `TerminationDiagnostic` pushes the
    // unboxed enum over clippy's `result_large_err` threshold in the many
    // `Result<_, GenerationError>` return types (TDI-5.7 adds `family`).
    AttemptBudgetExhausted(Box<TerminationDiagnostic>),
    NoProgress(Box<TerminationDiagnostic>),
}

impl std::fmt::Display for GenerationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Evaluation(error) => error.fmt(formatter),
            Self::AttemptBudgetExhausted(diagnostic) | Self::NoProgress(diagnostic) => {
                diagnostic.fmt(formatter)
            }
        }
    }
}

impl std::error::Error for GenerationError {}

#[derive(Clone, Copy, Debug)]
struct DeterministicRng {
    state: u64,
}

impl DeterministicRng {
    const fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        splitmix64(self.state)
    }

    fn index(&mut self, upper: usize) -> usize {
        (self.next_u64() % upper as u64) as usize
    }
}

impl RidgeModel {
    fn predict_linear(&self, features: &[f64]) -> f64 {
        assert_eq!(features.len(), self.means.len());
        assert_eq!(features.len(), self.scales.len());
        assert_eq!(self.coefficients.len(), features.len() + 1);

        features
            .iter()
            .zip(&self.means)
            .zip(&self.scales)
            .zip(self.coefficients.iter().skip(1))
            .fold(
                self.coefficients[0],
                |accumulator, (((value, mean), scale), coefficient)| {
                    accumulator + coefficient * ((value - mean) / scale)
                },
            )
    }
}

fn splitmix64(mut value: u64) -> u64 {
    value = value.wrapping_add(0x9E37_79B9_7F4A_7C15);

    let mut mixed = value;
    mixed = (mixed ^ (mixed >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    mixed = (mixed ^ (mixed >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);

    mixed ^ (mixed >> 31)
}

fn state_count_cardinality(width: u8) -> Cardinality {
    let shift = u32::from(width);

    1_u128
        .checked_shl(shift)
        .map(Cardinality::Exact)
        .unwrap_or(Cardinality::TooLarge {
            width,
            exponent: u128::from(shift),
        })
}

fn successor_set_space_cardinality(width: u8) -> Cardinality {
    let states = match state_count_cardinality(width) {
        Cardinality::Exact(states) => states,
        other => return other,
    };

    let Ok(shift) = u32::try_from(states) else {
        return Cardinality::TooLarge {
            width,
            exponent: states,
        };
    };

    1_u128
        .checked_shl(shift)
        .map(Cardinality::Exact)
        .unwrap_or(Cardinality::TooLarge {
            width,
            exponent: states,
        })
}

fn generation_successor_set_space_cardinality(width: u8) -> Cardinality {
    if width > MAX_SUPPORTED_WIDTH {
        Cardinality::Invalid {
            width,
            reason: "width is unsupported by the u64 successor-mask evaluator",
        }
    } else {
        successor_set_space_cardinality(width)
    }
}

fn state_count(context: AttemptContext) -> Result<usize, EvaluationError> {
    if context.width > MAX_SUPPORTED_WIDTH {
        return Err(EvaluationError::new(
            context,
            FailureCategory::UnsupportedWidth,
            format!(
                "width {} exceeds maximum supported width {MAX_SUPPORTED_WIDTH}",
                context.width
            ),
        ));
    }

    match state_count_cardinality(context.width) {
        Cardinality::Exact(value) => usize::try_from(value).map_err(|_| {
            EvaluationError::new(
                context,
                FailureCategory::Cardinality,
                format!("state count {value} cannot be represented as usize"),
            )
        }),
        Cardinality::TooLarge { exponent, .. } => Err(EvaluationError::new(
            context,
            FailureCategory::Cardinality,
            format!("state count 2^{exponent} exceeds exact evaluator range"),
        )),
        Cardinality::Invalid { reason, .. } => Err(EvaluationError::new(
            context,
            FailureCategory::UnsupportedWidth,
            reason,
        )),
    }
}

fn nonempty_successor_set_count(context: AttemptContext) -> Result<u64, EvaluationError> {
    match generation_successor_set_space_cardinality(context.width) {
        Cardinality::Exact(space_count) => {
            let nonempty_count = space_count.checked_sub(1).ok_or_else(|| {
                EvaluationError::new(
                    context,
                    FailureCategory::Arithmetic,
                    "successor-mask space underflow when removing empty mask",
                )
            })?;

            u64::try_from(nonempty_count).map_err(|_| {
                EvaluationError::new(
                    context,
                    FailureCategory::Cardinality,
                    format!(
                        "non-empty successor-mask count {nonempty_count} cannot be represented as u64"
                    ),
                )
            })
        }
        Cardinality::TooLarge { exponent, .. } => Err(EvaluationError::new(
            context,
            FailureCategory::Cardinality,
            format!("successor-mask space 2^{exponent} exceeds u128 exact range"),
        )),
        Cardinality::Invalid { reason, .. } => Err(EvaluationError::new(
            context,
            FailureCategory::UnsupportedWidth,
            reason,
        )),
    }
}

/// Advances the `splitmix64` chain one step and returns the new value. The
/// families draw from this exactly like the inherited generator.
fn next_draw(generator: &mut u64) -> u64 {
    *generator = splitmix64(*generator);
    *generator
}

/// The complete `states`-bit mask (all successor slots set), used by the dense
/// family. For `states == 64` this is `u64::MAX`; a `1 << 64` shift would be
/// undefined.
fn full_successor_mask(states: usize) -> u64 {
    if states >= 64 {
        u64::MAX
    } else {
        (1_u64 << states) - 1
    }
}

/// Produces each state's successor mask under the candidate's generator family
/// (TDI-5.7 Section 5). Every rule is a deterministic function of the seed via
/// the `splitmix64` chain and guarantees a non-empty successor set; the masks
/// are assembled by the unchanged frozen `build_system`.
fn generate_family_masks(context: AttemptContext) -> Result<Vec<u64>, EvaluationError> {
    let states = state_count(context)?;
    let states_u64 = states as u64;

    let mut masks = vec![0_u64; states];
    let mut generator = context.seed;

    match context.family {
        // F0 — base: uniform over all non-empty successor subsets (inherited
        // TDI-5.6 rule, unchanged).
        GeneratorFamily::F0Base => {
            let mask_count = nonempty_successor_set_count(context)?;
            for mask in &mut masks {
                *mask = next_draw(&mut generator) % mask_count + 1;
            }
        }
        // F1 — sparse: out-degree d ∈ {1, 2}, then d distinct successors by
        // rejection.
        GeneratorFamily::F1Sparse => {
            for mask in &mut masks {
                let out_degree = 1 + next_draw(&mut generator) % 2;
                let mut selected = 0_u64;
                while u64::from(selected.count_ones()) < out_degree {
                    let position = next_draw(&mut generator) % states_u64;
                    selected |= 1_u64 << position;
                }
                *mask = selected;
            }
        }
        // F2 — dense: out-degree states or states − 1 (exclude 0 or 1 states).
        GeneratorFamily::F2Dense => {
            let full = full_successor_mask(states);
            for mask in &mut masks {
                let excluded = next_draw(&mut generator) % 2;
                let mut selected = full;
                if excluded == 1 {
                    let position = next_draw(&mut generator) % states_u64;
                    selected &= !(1_u64 << position);
                }
                *mask = selected;
            }
        }
        // F3 — local: a non-empty subset of the Hamming-≤1 neighbourhood
        // {s, s^1, s^2, …, s^(2^(width−1))}, forcing self on an empty draw.
        GeneratorFamily::F3Local => {
            let width = context.width;
            for (source_bits, mask) in masks.iter_mut().enumerate() {
                let source = source_bits as u64;
                let mut neighbours = Vec::with_capacity(width as usize + 1);
                neighbours.push(source);
                for bit in 0..width {
                    neighbours.push(source ^ (1_u64 << bit));
                }

                let draw = next_draw(&mut generator);
                let mut selected = 0_u64;
                for (slot, &neighbour) in neighbours.iter().enumerate() {
                    if draw & (1_u64 << slot) != 0 {
                        selected |= 1_u64 << neighbour;
                    }
                }
                if selected == 0 {
                    selected |= 1_u64 << neighbours[0];
                }
                *mask = selected;
            }
        }
    }

    Ok(masks)
}

fn build_system(context: AttemptContext, masks: &[u64]) -> Result<TableSystem, EvaluationError> {
    let states = state_count(context)?;

    if masks.len() != states {
        return Err(EvaluationError::new(
            context,
            FailureCategory::Structural,
            format!(
                "expected {states} successor masks, received {}",
                masks.len()
            ),
        ));
    }

    let mut system = TableSystem::new(context.width).map_err(|error| {
        EvaluationError::new(
            context,
            FailureCategory::Structural,
            format!("cannot create branching system: {error:?}"),
        )
    })?;

    for (source_bits, &mask) in masks.iter().enumerate() {
        let source = State::new(source_bits as u64, context.width).map_err(|error| {
            EvaluationError::new(
                context,
                FailureCategory::Structural,
                format!("cannot create source state {source_bits}: {error:?}"),
            )
        })?;

        let mut successors = Vec::new();

        for target in 0..states {
            let shift = u32::try_from(target).map_err(|_| {
                EvaluationError::new(
                    context,
                    FailureCategory::Arithmetic,
                    format!("successor target index {target} cannot be shifted"),
                )
            })?;

            let bit = 1_u64.checked_shl(shift).ok_or_else(|| {
                EvaluationError::new(
                    context,
                    FailureCategory::Arithmetic,
                    format!("successor target index {target} exceeds u64 mask width"),
                )
            })?;

            if mask & bit != 0 {
                successors.push(State::new(target as u64, context.width).map_err(|error| {
                    EvaluationError::new(
                        context,
                        FailureCategory::Structural,
                        format!("cannot create target state {target}: {error:?}"),
                    )
                })?);
            }
        }

        system
            .insert(source, Action::Noop, successors)
            .map_err(|error| {
                EvaluationError::new(
                    context,
                    FailureCategory::Structural,
                    format!(
                        "cannot insert branching transition for state \
                     {source_bits}: {error:?}"
                    ),
                )
            })?;
    }

    Ok(system)
}

fn entropy_profile(
    context: AttemptContext,
    system: &TableSystem,
    initial: State,
) -> Result<[f64; OBSERVATION_HORIZON], EvaluationError> {
    let mut profile = [0.0_f64; OBSERVATION_HORIZON];

    for depth in 1..=OBSERVATION_HORIZON {
        profile[depth - 1] =
            uniform_branching_path_entropy_bits(system, initial, Action::Noop, depth).map_err(
                |error| {
                    EvaluationError::new(
                        context,
                        FailureCategory::DynamicAnalysis,
                        format!("branching entropy failed at depth {depth}: {error:?}"),
                    )
                },
            )?;
    }

    Ok(profile)
}

fn topology_profile(
    context: AttemptContext,
    system: &TableSystem,
    initial: State,
) -> Result<([f64; OBSERVATION_HORIZON], [f64; OBSERVATION_HORIZON]), EvaluationError> {
    let actions = [Action::Noop; OBSERVATION_HORIZON];

    let report = explore(system, initial, &actions).map_err(|error| {
        EvaluationError::new(
            context,
            FailureCategory::DynamicAnalysis,
            format!("branching exploration failed: {error:?}"),
        )
    })?;

    let mut reachable = [0.0_f64; OBSERVATION_HORIZON];
    let mut paths = [0.0_f64; OBSERVATION_HORIZON];

    for depth in 1..=OBSERVATION_HORIZON {
        reachable[depth - 1] = report.reachable_count(depth).ok_or_else(|| {
            EvaluationError::new(
                context,
                FailureCategory::Structural,
                format!("missing reachable layer {depth}"),
            )
        })? as f64;

        paths[depth - 1] = report.path_count(depth).ok_or_else(|| {
            EvaluationError::new(
                context,
                FailureCategory::Structural,
                format!("missing path-count layer {depth}"),
            )
        })? as f64;
    }

    Ok((reachable, paths))
}

fn ratio_value(ratio: &ExactRatio) -> f64 {
    ratio.as_f64()
}

/// Exact total variation `1 - overlap`, formed as the rational
/// `(denominator - numerator) / denominator` and rounded to `f64` in a
/// single `as_f64` step, so the descriptor is the exact rational converted
/// to `f64` — not `1.0 - overlap.as_f64()`, which would round twice and
/// deviate from the overlap up to one ULP. Every overlap this experiment
/// produces (width <= 4) has `u128` components; the deterministic
/// `1.0 - as_f64` form is retained only as an unreachable fallback rather
/// than risking a panic on a hypothetical wider kernel.
fn exact_total_variation(overlap: &ExactRatio) -> f64 {
    match overlap.components_u128() {
        Some((numerator, denominator)) => ExactRatio::new(denominator - numerator, denominator)
            .map(|total_variation| total_variation.as_f64())
            .unwrap_or_else(|| 1.0 - ratio_value(overlap)),
        None => 1.0 - ratio_value(overlap),
    }
}

/// Exact contraction descriptors of the one-step Noop kernel (TDI-5.7
/// Section 5): the Dobrushin coefficient `delta = max_{i<j} TV(P_i, P_j)`
/// and the mean pairwise total variation `delta_bar`. Each `P_s` is the
/// exact uniform distribution over state `s`'s Noop successor set
/// (`uniform_branching_state_distribution(.., 1)`); `TV = 1 - overlap` uses
/// the inherited exact `distribution_overlap`. Both descriptors are exact
/// rationals in `[0, 1]`, converted to `f64` exactly like the early
/// overlaps. Every one of the `2^width` states has a defined Noop
/// transition (see `build_system`), so the kernel is total and the maximum
/// / mean range over all unordered state pairs.
fn contraction_descriptors(
    context: AttemptContext,
    system: &TableSystem,
) -> Result<[f64; CONTRACTION_FEATURE_COUNT], EvaluationError> {
    let states = state_count(context)?;

    let mut rows = Vec::with_capacity(states);

    for index in 0..states {
        let state = State::new(index as u64, context.width).map_err(|error| {
            EvaluationError::new(
                context,
                FailureCategory::Structural,
                format!("cannot create kernel state {index}: {error:?}"),
            )
        })?;

        let row = uniform_branching_state_distribution(system, state, Action::Noop, 1).map_err(
            |error| {
                EvaluationError::new(
                    context,
                    FailureCategory::DynamicAnalysis,
                    format!("one-step kernel distribution failed for state {index}: {error:?}"),
                )
            },
        )?;

        rows.push(row);
    }

    let zero = ExactRatio::new(0, 1).expect("zero is a valid exact ratio");
    let one = ExactRatio::new(1, 1).expect("one is a valid exact ratio");

    let mut min_overlap = one;
    let mut overlap_sum = zero;
    let mut pair_count = 0_u128;

    for left in 0..states {
        for right in (left + 1)..states {
            let overlap = distribution_overlap(&rows[left], &rows[right]).map_err(|error| {
                EvaluationError::new(
                    context,
                    FailureCategory::Arithmetic,
                    format!("pairwise kernel overlap failed for states {left},{right}: {error:?}"),
                )
            })?;

            let ordering = overlap.checked_cmp(&min_overlap).ok_or_else(|| {
                EvaluationError::new(
                    context,
                    FailureCategory::Arithmetic,
                    "kernel overlap comparison overflowed".to_owned(),
                )
            })?;

            if ordering == std::cmp::Ordering::Less {
                min_overlap = overlap.clone();
            }

            overlap_sum = overlap_sum.checked_add(&overlap).ok_or_else(|| {
                EvaluationError::new(
                    context,
                    FailureCategory::Arithmetic,
                    "kernel overlap sum overflowed".to_owned(),
                )
            })?;

            pair_count += 1;
        }
    }

    let dobrushin = exact_total_variation(&min_overlap);

    let mean_total_variation = if pair_count == 0 {
        // Only possible with a single state; width >= 3 guarantees pairs.
        0.0
    } else {
        let mean_overlap = overlap_sum.checked_div_u128(pair_count).ok_or_else(|| {
            EvaluationError::new(
                context,
                FailureCategory::Arithmetic,
                "kernel mean overlap division overflowed".to_owned(),
            )
        })?;

        exact_total_variation(&mean_overlap)
    };

    Ok([dobrushin, mean_total_variation])
}

/// Exact spectral moments of the one-step Noop kernel (TDI-5.7 Section 5):
/// `s2 = trace(P^2)` and `s3 = trace(P^3)`. Each `P(s, .)` is the exact
/// uniform distribution over state `s`'s `d_s` Noop successors
/// (`uniform_branching_state_distribution(.., 1)`), so `P_{s,t} = 1/d_s` for
/// each successor `t` and `0` otherwise. The traces are sums over closed
/// walks,
///
///   s2 = sum over ordered pairs (i, j) with j a successor of i and i a
///        successor of j, of 1/(d_i d_j);
///   s3 = sum over ordered triples (i, j, k) with j a successor of i, k a
///        successor of j and i a successor of k, of 1/(d_i d_j d_k).
///
/// Every summand is a unit fraction whose denominator is a product of at most
/// three branching factors (each `<= 2^width`), so it fits in `u128`. The
/// summands are accumulated with the inherited arbitrary-precision
/// `ExactRatio` addition and only the final total is rounded to `f64` in a
/// single `as_f64()` step — the same exactness discipline as δ, δ̄, O₁ and O₂.
/// No eigenvalue, characteristic polynomial or floating-point iteration is
/// involved; both moments are exact rationals in `[0, 2^width]`.
fn spectral_moments(
    context: AttemptContext,
    system: &TableSystem,
) -> Result<[f64; SPECTRAL_FEATURE_COUNT], EvaluationError> {
    let states = state_count(context)?;

    let mut state_rows: Vec<(State, std::collections::BTreeMap<State, ExactRatio>)> =
        Vec::with_capacity(states);

    for index in 0..states {
        let state = State::new(index as u64, context.width).map_err(|error| {
            EvaluationError::new(
                context,
                FailureCategory::Structural,
                format!("cannot create kernel state {index}: {error:?}"),
            )
        })?;

        let row = uniform_branching_state_distribution(system, state, Action::Noop, 1).map_err(
            |error| {
                EvaluationError::new(
                    context,
                    FailureCategory::DynamicAnalysis,
                    format!("one-step kernel distribution failed for state {index}: {error:?}"),
                )
            },
        )?;

        state_rows.push((state, row));
    }

    // Map every state to its Noop row so successor states (which are keys in
    // some row) can be resolved to their branching factor and membership in
    // constant time. Every one of the `2^width` states is present.
    let row_of: std::collections::BTreeMap<State, &std::collections::BTreeMap<State, ExactRatio>> =
        state_rows
            .iter()
            .map(|(state, row)| (*state, row))
            .collect();

    let resolve =
        |state: &State| -> Result<&std::collections::BTreeMap<State, ExactRatio>, EvaluationError> {
            row_of.get(state).copied().ok_or_else(|| {
                EvaluationError::new(
                    context,
                    FailureCategory::Structural,
                    "kernel successor state is absent from the state enumeration".to_owned(),
                )
            })
        };

    let arithmetic = |message: &str| {
        EvaluationError::new(context, FailureCategory::Arithmetic, message.to_owned())
    };

    let mut second_moment = ExactRatio::new(0, 1).expect("zero is a valid exact ratio");
    let mut third_moment = second_moment.clone();

    for (from_state, from_row) in &state_rows {
        let from_degree = from_row.len() as u128;

        for middle_state in from_row.keys() {
            let middle_row = resolve(middle_state)?;
            let middle_degree = middle_row.len() as u128;

            // Closed 2-walk i -> j -> i contributes 1 / (d_i d_j).
            if middle_row.contains_key(from_state) {
                let denominator = from_degree
                    .checked_mul(middle_degree)
                    .ok_or_else(|| arithmetic("spectral 2-walk denominator overflowed"))?;

                let term = ExactRatio::new(1, denominator)
                    .ok_or_else(|| arithmetic("spectral 2-walk term is invalid"))?;

                second_moment = second_moment
                    .checked_add(&term)
                    .ok_or_else(|| arithmetic("spectral second-moment sum overflowed"))?;
            }

            // Closed 3-walk i -> j -> k -> i contributes 1 / (d_i d_j d_k).
            for last_state in middle_row.keys() {
                let last_row = resolve(last_state)?;
                let last_degree = last_row.len() as u128;

                if last_row.contains_key(from_state) {
                    let denominator = from_degree
                        .checked_mul(middle_degree)
                        .and_then(|partial| partial.checked_mul(last_degree))
                        .ok_or_else(|| arithmetic("spectral 3-walk denominator overflowed"))?;

                    let term = ExactRatio::new(1, denominator)
                        .ok_or_else(|| arithmetic("spectral 3-walk term is invalid"))?;

                    third_moment = third_moment
                        .checked_add(&term)
                        .ok_or_else(|| arithmetic("spectral third-moment sum overflowed"))?;
                }
            }
        }
    }

    Ok([second_moment.as_f64(), third_moment.as_f64()])
}

fn target_horizon_index(horizon: usize) -> Option<usize> {
    TARGET_HORIZONS
        .iter()
        .position(|&candidate| candidate == horizon)
}

fn primary_horizon_index() -> usize {
    let index =
        target_horizon_index(PRIMARY_HORIZON).expect("primary horizon belongs to target horizons");

    debug_assert_eq!(index, PRIMARY_HORIZON_INDEX);

    index
}

fn feature_layout(record: &Record, layout: FeatureLayout) -> Vec<f64> {
    let mut features = Vec::with_capacity(layout.feature_count());
    features.extend_from_slice(&record.baseline);

    let first_overlap = record.early_overlap[0];
    let second_overlap = record.early_overlap[1];

    match layout {
        FeatureLayout::B0 => {}
        FeatureLayout::B1 => {
            features.push(first_overlap);
        }
        FeatureLayout::B2 => {
            features.push(second_overlap);
        }
        FeatureLayout::B12 => {
            features.push(first_overlap);
            features.push(second_overlap);
        }
        FeatureLayout::BD => {
            features.push(second_overlap - first_overlap);
        }
        // Confirmatory layouts (TDI-5.7 Section 6). Terms are the two exact
        // contraction descriptors (delta, delta_bar) and, for SK/SKT, the two
        // exact spectral moments (s2, s3), all already stored on the record;
        // standardization happens downstream in ridge fitting, exactly like
        // every other feature. The baseline block is untouched, so SK minus CK
        // isolates the spectral moments' marginal value beyond contraction and
        // SKT minus SK isolates the overlaps' marginal value beyond both.
        FeatureLayout::Ck => {
            features.push(record.contraction[0]);
            features.push(record.contraction[1]);
        }
        FeatureLayout::Sk => {
            features.push(record.contraction[0]);
            features.push(record.contraction[1]);
            features.push(record.spectral[0]);
            features.push(record.spectral[1]);
        }
        FeatureLayout::Skt => {
            features.push(record.contraction[0]);
            features.push(record.contraction[1]);
            features.push(record.spectral[0]);
            features.push(record.spectral[1]);
            features.push(first_overlap);
            features.push(second_overlap);
        }
    }

    debug_assert_eq!(features.len(), layout.feature_count());

    features
}

fn target_values(records: &[Record], horizon_index: usize) -> Vec<f64> {
    records
        .iter()
        .map(|record| record.targets_u[horizon_index])
        .collect()
}

fn overlap_values(records: &[Record], horizon_index: usize) -> Vec<f64> {
    records
        .iter()
        .map(|record| record.overlaps[horizon_index])
        .collect()
}

fn biguint_log2_from_u64_digits(digits: &[u64]) -> Result<f64, String> {
    let top = digits
        .last()
        .copied()
        .ok_or_else(|| "cannot calculate log2 of zero".to_owned())?;

    if top == 0 {
        return Err("invalid leading zero BigUint limb".to_owned());
    }

    let top_bits = 64_usize - top.leading_zeros() as usize;
    let bit_length = (digits.len() - 1) * 64 + top_bits;

    let combined = if digits.len() >= 2 {
        (u128::from(top) << 64) | u128::from(digits[digits.len() - 2])
    } else {
        u128::from(top)
    };

    let combined_bits = if digits.len() >= 2 {
        top_bits + 64
    } else {
        top_bits
    };

    let shift = combined_bits.saturating_sub(53);
    let significant = (combined >> shift) as u64;
    let significant_bits = combined_bits - shift;

    let mantissa = significant as f64 / 2.0_f64.powi((significant_bits - 1) as i32);

    if !mantissa.is_finite() || !(1.0..2.0).contains(&mantissa) {
        return Err("invalid normalized BigUint mantissa".to_owned());
    }

    let logarithm = (bit_length - 1) as f64 + mantissa.log2();

    if !logarithm.is_finite() {
        return Err("non-finite BigUint logarithm".to_owned());
    }

    Ok(logarithm)
}

fn exact_overlap_deficit_u(ratio: &ExactRatio) -> Result<f64, String> {
    if ratio.numerator() >= ratio.denominator() {
        return Err("conditional overlap must be strictly below one".to_owned());
    }

    let deficit_numerator = ratio.denominator() - ratio.numerator();

    let numerator_log2 = biguint_log2_from_u64_digits(&deficit_numerator.to_u64_digits())?;

    let denominator_log2 = biguint_log2_from_u64_digits(&ratio.denominator().to_u64_digits())?;

    // Finiteness/non-negativity of the transformed value is deliberately
    // not checked here: the caller (`analyze_seed`) treats an invalid
    // transform as a graceful per-candidate exclusion
    // (`RejectionReason::InvalidTransformedTarget`), not a fatal error.
    // Checking it here too would let this function's own fatal error
    // path intercept the value first, making that exclusion unreachable.
    Ok(denominator_log2 - numerator_log2)
}

// `normalized_entropy`, `normalized_reachable`, and `transformed_path_count`
// deliberately do not validate the finiteness of their own return values.
// `analyze_seed`'s baseline-feature assembly checks every value it collects
// from these functions in one place and turns a non-finite one into a
// graceful per-candidate exclusion (`RejectionReason::NonFiniteFeature`).
// A local fatal check here would intercept the value first and make that
// exclusion unreachable. `normalized_entropy`'s denominator check is kept
// because it depends only on the width (a structural property, not a
// per-candidate outcome), so a bad denominator is a genuine invariant
// violation rather than a data-quality edge case.
fn normalized_entropy(entropy_bits: f64, context: AttemptContext) -> Result<f64, EvaluationError> {
    let states = state_count(context)? as f64;
    let denominator = states.ln();

    if !denominator.is_finite() || denominator <= 0.0 {
        return Err(EvaluationError::new(
            context,
            FailureCategory::Arithmetic,
            format!("invalid entropy normalizer for width {}", context.width),
        ));
    }

    Ok(entropy_bits * std::f64::consts::LN_2 / denominator)
}

fn normalized_reachable(reachable: f64, context: AttemptContext) -> Result<f64, EvaluationError> {
    let states = state_count(context)? as f64;

    Ok(reachable / states)
}

fn transformed_path_count(path_count: f64) -> f64 {
    path_count.ln_1p()
}

fn analyze_seed(context: AttemptContext) -> Result<CandidateOutcome, EvaluationError> {
    let masks = generate_family_masks(context)?;
    let system = build_system(context, &masks)?;

    let reference = State::new(0, context.width).map_err(|error| {
        EvaluationError::new(
            context,
            FailureCategory::Structural,
            format!("cannot create reference state: {error:?}"),
        )
    })?;

    let perturbation_node = context.width.checked_sub(1).ok_or_else(|| {
        EvaluationError::new(
            context,
            FailureCategory::Structural,
            "width zero cannot define the width-1 perturbation node",
        )
    })?;

    let perturbation = Action::Flip {
        node: perturbation_node,
    };

    let perturbed = perturbation.apply(reference).map_err(|error| {
        EvaluationError::new(
            context,
            FailureCategory::Structural,
            format!("cannot apply perturbation: {error:?}"),
        )
    })?;

    let reference_entropy = entropy_profile(context, &system, reference)?;
    let perturbed_entropy = entropy_profile(context, &system, perturbed)?;

    let (reference_reachable, reference_paths) = topology_profile(context, &system, reference)?;

    let (perturbed_reachable, perturbed_paths) = topology_profile(context, &system, perturbed)?;

    let observation = analyze_branching_recovery(
        &system,
        reference,
        perturbation,
        Action::Noop,
        OBSERVATION_HORIZON,
    )
    .map_err(|error| {
        EvaluationError::new(
            context,
            FailureCategory::DynamicAnalysis,
            format!(
                "observation recovery analysis failed for width \
             {}, seed {}: {error:?}",
                context.width, context.seed
            ),
        )
    })?;

    // Critère d’exclusion préenregistré : O2 = 1.
    if observation.fully_recovered() {
        return Ok(CandidateOutcome::Rejected(
            RejectionReason::ObservationFullyRecovered,
        ));
    }

    let observation_overlaps = observation.overlap_profile();

    if observation_overlaps.len() != OBSERVATION_HORIZON {
        return Err(EvaluationError::new(
            context,
            FailureCategory::Structural,
            format!(
                "expected {OBSERVATION_HORIZON} observation overlaps, \
             received {}",
                observation_overlaps.len()
            ),
        ));
    }

    let first_overlap = ratio_value(&observation_overlaps[0]);
    let second_overlap = ratio_value(&observation_overlaps[1]);

    if !first_overlap.is_finite()
        || !second_overlap.is_finite()
        || !(0.0..=1.0).contains(&first_overlap)
        || !(0.0..1.0).contains(&second_overlap)
    {
        return Ok(CandidateOutcome::Rejected(
            RejectionReason::InvalidObservationGeometry,
        ));
    }

    let mut overlaps = [0.0_f64; TARGET_HORIZON_COUNT];
    let mut targets_u = [0.0_f64; TARGET_HORIZON_COUNT];

    for (horizon_index, &horizon) in TARGET_HORIZONS.iter().enumerate() {
        let outcome =
            analyze_branching_recovery(&system, reference, perturbation, Action::Noop, horizon)
                .map_err(|error| {
                    EvaluationError::new(
                        context,
                        FailureCategory::DynamicAnalysis,
                        format!(
                            "target recovery analysis failed at horizon {horizon} \
                 for width {}, seed {}: {error:?}",
                            context.width, context.seed
                        ),
                    )
                })?;

        // Critère d’exclusion préenregistré :
        // déficit exact nul à un horizon cible.
        if outcome.fully_recovered() {
            return Ok(CandidateOutcome::Rejected(
                RejectionReason::TargetFullyRecovered { horizon },
            ));
        }

        let overlap_ratio = outcome.final_overlap().ok_or_else(|| {
            EvaluationError::new(
                context,
                FailureCategory::Structural,
                format!(
                    "target horizon {horizon} produced no overlap \
                     for width {}, seed {}",
                    context.width, context.seed
                ),
            )
        })?;

        let overlap = ratio_value(&overlap_ratio);

        if !overlap.is_finite() || !(0.0..1.0).contains(&overlap) {
            return Ok(CandidateOutcome::Rejected(
                RejectionReason::InvalidTargetGeometry { horizon },
            ));
        }

        let target_u = exact_overlap_deficit_u(&overlap_ratio).map_err(|error| {
            EvaluationError::new(
                context,
                FailureCategory::Arithmetic,
                format!(
                    "cannot calculate U_{horizon} for width {width}, \
                     seed {seed}: {error}",
                    width = context.width,
                    seed = context.seed
                ),
            )
        })?;

        if !target_u.is_finite() || target_u < 0.0 {
            return Ok(CandidateOutcome::Rejected(
                RejectionReason::InvalidTransformedTarget { horizon },
            ));
        }

        overlaps[horizon_index] = overlap;
        targets_u[horizon_index] = target_u;
    }

    let baseline = [
        normalized_entropy(reference_entropy[0], context)?,
        normalized_entropy(reference_entropy[1], context)?,
        normalized_entropy(perturbed_entropy[0], context)?,
        normalized_entropy(perturbed_entropy[1], context)?,
        normalized_reachable(reference_reachable[0], context)?,
        normalized_reachable(reference_reachable[1], context)?,
        transformed_path_count(reference_paths[0]),
        transformed_path_count(reference_paths[1]),
        normalized_reachable(perturbed_reachable[0], context)?,
        normalized_reachable(perturbed_reachable[1], context)?,
        transformed_path_count(perturbed_paths[0]),
        transformed_path_count(perturbed_paths[1]),
        f64::from(context.width),
    ];

    let early_overlap = [first_overlap, second_overlap];
    let contraction = contraction_descriptors(context, &system)?;
    let spectral = spectral_moments(context, &system)?;

    if baseline
        .iter()
        .chain(&early_overlap)
        .chain(&contraction)
        .chain(&spectral)
        .any(|value| !value.is_finite())
    {
        return Ok(CandidateOutcome::Rejected(
            RejectionReason::NonFiniteFeature,
        ));
    }

    Ok(CandidateOutcome::Accepted(Box::new(Record {
        baseline,
        early_overlap,
        contraction,
        spectral,
        overlaps,
        targets_u,
    })))
}
fn preregistered_generation_limits(
    family: GeneratorFamily,
    width: u8,
    start_seed: u64,
    count: usize,
) -> Result<GenerationLimits, EvaluationError> {
    let context = AttemptContext::new(family, width, start_seed, 0);

    if count == 0 {
        return Err(EvaluationError::new(
            context,
            FailureCategory::InvalidConfiguration,
            "record target must be positive",
        ));
    }

    let (attempt_multiplier, no_progress_limit) = match width {
        TRAIN_WIDTH_3 => (WIDTH_3_ATTEMPT_MULTIPLIER, WIDTH_3_NO_PROGRESS_LIMIT),
        TRAIN_WIDTH_4 => (WIDTH_4_ATTEMPT_MULTIPLIER, WIDTH_4_NO_PROGRESS_LIMIT),
        WIDTH_5 => (WIDTH_5_ATTEMPT_MULTIPLIER, WIDTH_5_NO_PROGRESS_LIMIT),
        WIDTH_6 => (WIDTH_6_ATTEMPT_MULTIPLIER, WIDTH_6_NO_PROGRESS_LIMIT),
        _ => {
            return Err(EvaluationError::new(
                context,
                FailureCategory::UnsupportedWidth,
                format!("width {width} is not part of the TDI-5.7 preregistered populations"),
            ));
        }
    };

    let max_attempts = count.checked_mul(attempt_multiplier).ok_or_else(|| {
        EvaluationError::new(
            context,
            FailureCategory::Arithmetic,
            format!(
                "attempt budget overflow for target {count} and multiplier {attempt_multiplier}"
            ),
        )
    })?;

    Ok(GenerationLimits {
        max_attempts,
        no_progress_limit,
    })
}

/// Verifies that every population spec's worst-case reserved seed range
/// (`[seed, seed + max_attempts)`) is pairwise disjoint from every other
/// spec's. Generic over `specs` so both the real preregistered layout and
/// tiny test/smoke overrides can be checked with the same logic; callers
/// that specifically need the real 12-reservation contract should use
/// `validate_preregistered_seed_reservations` instead.
fn validate_seed_reservations(specs: &[PopulationSpec]) -> Result<usize, String> {
    let mut ranges = Vec::with_capacity(specs.len());

    for spec in specs {
        let label = || {
            format!(
                "block {} {}",
                spec.seed_block.label(),
                spec.population.label()
            )
        };

        let limits = preregistered_generation_limits(
            spec.family(),
            spec.width,
            spec.seed,
            spec.target_count,
        )
        .map_err(|error| format!("{}: {error}", label()))?;

        let reserved_attempts = u64::try_from(limits.max_attempts).map_err(|_| {
            format!(
                "{}: maximum-attempt budget {} cannot be represented as u64",
                label(),
                limits.max_attempts
            )
        })?;

        let end_seed = spec
            .seed
            .checked_add(reserved_attempts)
            .ok_or_else(|| format!("{}: reserved seed range overflows u64", label()))?;

        ranges.push((spec.seed, end_seed, label()));
    }

    ranges.sort_by_key(|(start_seed, _, _)| *start_seed);

    for pair in ranges.windows(2) {
        let (_, previous_end, previous_label) = &pair[0];
        let (next_start, _, next_label) = &pair[1];

        if *previous_end > *next_start {
            return Err(format!(
                "reserved seed ranges overlap: {previous_label} ends at \
                 {previous_end}, {next_label} starts at {next_start}"
            ));
        }
    }

    Ok(ranges.len())
}

fn validate_preregistered_seed_reservations() -> Result<usize, String> {
    let count = validate_seed_reservations(&population_specs())?;

    if count != TOTAL_SEED_RESERVATIONS {
        return Err(format!(
            "expected {TOTAL_SEED_RESERVATIONS} seed reservations, received {count}"
        ));
    }

    Ok(count)
}

fn generate_records_with_limits(
    family: GeneratorFamily,
    width: u8,
    start_seed: u64,
    count: usize,
    limits: GenerationLimits,
) -> Result<GenerationReport, GenerationError> {
    generate_records_with_analyzer(family, width, start_seed, count, limits, analyze_seed)
}

fn seed_for_attempt(
    family: GeneratorFamily,
    width: u8,
    start_seed: u64,
    attempt_index: usize,
) -> Result<u64, EvaluationError> {
    let attempt_offset = u64::try_from(attempt_index).map_err(|_| {
        EvaluationError::new(
            AttemptContext::new(family, width, start_seed, attempt_index),
            FailureCategory::SeedRange,
            format!("attempt index {attempt_index} cannot be represented as u64"),
        )
    })?;

    start_seed.checked_add(attempt_offset).ok_or_else(|| {
        EvaluationError::new(
            AttemptContext::new(family, width, start_seed, attempt_index),
            FailureCategory::SeedRange,
            format!("seed range overflow from start seed {start_seed} at attempt {attempt_index}"),
        )
    })
}

fn generate_records_with_analyzer<F>(
    family: GeneratorFamily,
    width: u8,
    start_seed: u64,
    count: usize,
    limits: GenerationLimits,
    mut analyzer: F,
) -> Result<GenerationReport, GenerationError>
where
    F: FnMut(AttemptContext) -> Result<CandidateOutcome, EvaluationError>,
{
    if limits.max_attempts == 0 || limits.no_progress_limit == 0 {
        return Err(GenerationError::Evaluation(EvaluationError::new(
            AttemptContext::new(family, width, start_seed, 0),
            FailureCategory::InvalidConfiguration,
            "generation limits must be positive",
        )));
    }

    if count == 0 {
        return Err(GenerationError::Evaluation(EvaluationError::new(
            AttemptContext::new(family, width, start_seed, 0),
            FailureCategory::InvalidConfiguration,
            "record target must be positive",
        )));
    }

    let mut records = Vec::with_capacity(count);
    let mut excluded = 0_usize;
    let mut rejections = RejectionCounts::default();
    let mut attempts = 0_usize;
    let mut attempts_without_progress = 0_usize;

    while records.len() < count {
        if attempts >= limits.max_attempts {
            let seed = seed_for_attempt(family, width, start_seed, attempts)
                .map_err(GenerationError::Evaluation)?;
            let diagnostic = TerminationDiagnostic::new(
                AttemptContext::new(family, width, seed, attempts),
                FailureCategory::AttemptBudget,
                GenerationProgress {
                    accepted: records.len(),
                    excluded,
                    rejections: rejections.clone(),
                    target_count: count,
                    limits,
                },
                "target record count remained unattainable before the deterministic attempt budget",
            );

            return Err(GenerationError::AttemptBudgetExhausted(Box::new(
                diagnostic,
            )));
        }

        let seed = seed_for_attempt(family, width, start_seed, attempts)
            .map_err(GenerationError::Evaluation)?;
        let context = AttemptContext::new(family, width, seed, attempts);

        match analyzer(context).map_err(GenerationError::Evaluation)? {
            CandidateOutcome::Accepted(record) => {
                records.push(*record);
                attempts_without_progress = 0;
            }
            CandidateOutcome::Rejected(reason) => {
                rejections.record(reason);
                excluded += 1;
                attempts_without_progress += 1;

                debug_assert_eq!(excluded, rejections.total());
            }
        }

        attempts += 1;

        if records.len() < count && attempts_without_progress >= limits.no_progress_limit {
            let diagnostic = TerminationDiagnostic::new(
                context,
                FailureCategory::NoProgress,
                GenerationProgress {
                    accepted: records.len(),
                    excluded,
                    rejections: rejections.clone(),
                    target_count: count,
                    limits,
                },
                format!(
                    "no accepted record observed for {attempts_without_progress} consecutive attempts"
                ),
            );

            return Err(GenerationError::NoProgress(Box::new(diagnostic)));
        }
    }

    let next_seed = seed_for_attempt(family, width, start_seed, attempts)
        .map_err(GenerationError::Evaluation)?;

    Ok(GenerationReport {
        records,
        next_seed,
        excluded,
        rejections,
        attempts,
        limits,
    })
}

#[derive(Clone, Debug)]
struct PopulationGenerationReport {
    spec: PopulationSpec,
    report: GenerationReport,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct PopulationGenerationError {
    spec: PopulationSpec,
    error: Box<GenerationError>,
}

impl std::fmt::Display for PopulationGenerationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "seed block {}, population {}: {}",
            self.spec.seed_block.label(),
            self.spec.population.label(),
            self.error
        )
    }
}

impl std::error::Error for PopulationGenerationError {}

fn generate_population_with_analyzer<F>(
    spec: PopulationSpec,
    limits: GenerationLimits,
    analyzer: F,
) -> Result<PopulationGenerationReport, PopulationGenerationError>
where
    F: FnMut(AttemptContext) -> Result<CandidateOutcome, EvaluationError>,
{
    generate_records_with_analyzer(
        spec.family(),
        spec.width,
        spec.seed,
        spec.target_count,
        limits,
        analyzer,
    )
    .map(|report| PopulationGenerationReport { spec, report })
    .map_err(|error| PopulationGenerationError {
        spec,
        error: Box::new(error),
    })
}

fn generate_population(
    spec: PopulationSpec,
) -> Result<PopulationGenerationReport, PopulationGenerationError> {
    let limits =
        preregistered_generation_limits(spec.family(), spec.width, spec.seed, spec.target_count)
            .map_err(|error| PopulationGenerationError {
                spec,
                error: Box::new(GenerationError::Evaluation(error)),
            })?;

    generate_population_with_analyzer(spec, limits, analyze_seed)
}

#[derive(Clone, Debug)]
struct BlockPopulations {
    seed_block: SeedBlockId,
    training_width_3: PopulationGenerationReport,
    holdout_width_3: PopulationGenerationReport,
    training_width_4: PopulationGenerationReport,
    holdout_width_4: PopulationGenerationReport,
}

impl BlockPopulations {
    fn combined_holdout(&self) -> Vec<Record> {
        combine_width_3_and_4(
            &self.holdout_width_3.report.records,
            &self.holdout_width_4.report.records,
        )
    }

    /// Every population's full generation report, in `PopulationKind::ALL`
    /// order. Required-raw-output printing walks this instead of the four
    /// named fields directly. TDI-5.7 has no OOD populations (Section 8).
    fn reports(&self) -> [&PopulationGenerationReport; POPULATIONS_PER_SEED_BLOCK] {
        [
            &self.training_width_3,
            &self.holdout_width_3,
            &self.training_width_4,
            &self.holdout_width_4,
        ]
    }
}

fn find_population_spec(
    specs: &[PopulationSpec],
    seed_block: SeedBlockId,
    population: PopulationKind,
) -> PopulationSpec {
    *specs
        .iter()
        .find(|spec| spec.seed_block == seed_block && spec.population == population)
        .expect("population_specs always covers every (block, population) pair")
}

fn generate_block_populations(
    seed_block: SeedBlockId,
    specs: &[PopulationSpec],
) -> Result<BlockPopulations, PopulationGenerationError> {
    let generate =
        |population: PopulationKind| -> Result<PopulationGenerationReport, PopulationGenerationError> {
            let spec = find_population_spec(specs, seed_block, population);

            generate_population(spec)
        };

    Ok(BlockPopulations {
        seed_block,
        training_width_3: generate(PopulationKind::TrainingWidth3)?,
        holdout_width_3: generate(PopulationKind::HoldoutWidth3)?,
        training_width_4: generate(PopulationKind::TrainingWidth4)?,
        holdout_width_4: generate(PopulationKind::HoldoutWidth4)?,
    })
}

fn model_features(record: &Record, layout: FeatureLayout) -> Vec<f64> {
    feature_layout(record, layout)
}

fn feature_matrix<F>(records: &[Record], feature_fn: F) -> Vec<Vec<f64>>
where
    F: Fn(&Record) -> Vec<f64>,
{
    records.iter().map(feature_fn).collect()
}

fn fit_ridge(features: &[Vec<f64>], targets: &[f64]) -> Result<RidgeModel, String> {
    if features.is_empty() {
        return Err("cannot fit ridge regression on an empty dataset".to_owned());
    }

    if features.len() != targets.len() {
        return Err(format!(
            "feature/target length mismatch: {} versus {}",
            features.len(),
            targets.len()
        ));
    }

    let feature_count = features[0].len();

    if feature_count == 0 {
        return Err("ridge regression requires at least one feature".to_owned());
    }

    if features.iter().any(|row| row.len() != feature_count) {
        return Err("inconsistent feature-vector lengths".to_owned());
    }

    let sample_count = features.len() as f64;
    let mut means = vec![0.0_f64; feature_count];

    for row in features {
        for (mean, value) in means.iter_mut().zip(row) {
            *mean += value;
        }
    }

    for mean in &mut means {
        *mean /= sample_count;
    }

    let mut scales = vec![0.0_f64; feature_count];

    for row in features {
        for ((scale, value), mean) in scales.iter_mut().zip(row).zip(&means) {
            let difference = value - mean;
            *scale += difference * difference;
        }
    }

    for scale in &mut scales {
        *scale = (*scale / sample_count).sqrt();

        if !scale.is_finite() || *scale <= 1.0e-12 {
            *scale = 1.0;
        }
    }

    let dimension = feature_count + 1;
    let mut normal = vec![vec![0.0_f64; dimension]; dimension];
    let mut right_hand_side = vec![0.0_f64; dimension];

    for (row, &target) in features.iter().zip(targets) {
        let mut standardized = Vec::with_capacity(dimension);
        standardized.push(1.0);

        standardized.extend(
            row.iter()
                .zip(&means)
                .zip(&scales)
                .map(|((value, mean), scale)| (value - mean) / scale),
        );

        for (left_index, &left_value) in standardized.iter().enumerate() {
            right_hand_side[left_index] += left_value * target;

            for (right_index, &right_value) in standardized.iter().enumerate() {
                normal[left_index][right_index] += left_value * right_value;
            }
        }
    }

    for (index, row) in normal.iter_mut().enumerate().skip(1) {
        row[index] += RIDGE_LAMBDA;
    }

    let coefficients = solve_linear_system(normal, right_hand_side)?;

    Ok(RidgeModel {
        means,
        scales,
        coefficients,
    })
}

fn fit_horizon_models(
    records: &[Record],
    target_scalers: &[TargetScaler; TARGET_HORIZON_COUNT],
) -> Result<HorizonModels, String> {
    let mut models = Vec::with_capacity(TARGET_HORIZON_COUNT * MODEL_LAYOUT_COUNT);

    for (horizon_index, scaler) in target_scalers.iter().copied().enumerate() {
        let raw_targets = target_values(records, horizon_index);

        let standardized_targets = raw_targets
            .iter()
            .map(|&value| scaler.standardize(value))
            .collect::<Vec<_>>();

        for layout in FeatureLayout::ALL {
            let matrix = feature_matrix(records, |record| model_features(record, layout));

            models.push(fit_ridge(&matrix, &standardized_targets)?);
        }
    }

    Ok(HorizonModels { models })
}

fn solve_linear_system(
    mut matrix: Vec<Vec<f64>>,
    mut right_hand_side: Vec<f64>,
) -> Result<Vec<f64>, String> {
    let dimension = matrix.len();

    if dimension == 0 || right_hand_side.len() != dimension {
        return Err("invalid linear-system dimensions".to_owned());
    }

    if matrix.iter().any(|row| row.len() != dimension) {
        return Err("linear-system matrix is not square".to_owned());
    }

    for column in 0..dimension {
        let pivot_row = (column..dimension)
            .max_by(|&left, &right| {
                matrix[left][column]
                    .abs()
                    .total_cmp(&matrix[right][column].abs())
            })
            .ok_or_else(|| "missing pivot row".to_owned())?;

        let pivot_value = matrix[pivot_row][column];

        if !pivot_value.is_finite() || pivot_value.abs() <= 1.0e-12 {
            return Err(format!(
                "singular or ill-conditioned normal matrix at column {column}"
            ));
        }

        if pivot_row != column {
            matrix.swap(pivot_row, column);
            right_hand_side.swap(pivot_row, column);
        }

        let pivot_values = matrix[column].clone();
        let pivot_denominator = pivot_values[column];
        let pivot_right_hand_side = right_hand_side[column];

        for (row_index, row_values) in matrix.iter_mut().enumerate().skip(column + 1) {
            let factor = row_values[column] / pivot_denominator;

            row_values[column] = 0.0;

            for (value, pivot_value) in row_values.iter_mut().zip(&pivot_values).skip(column + 1) {
                *value -= factor * pivot_value;
            }

            right_hand_side[row_index] -= factor * pivot_right_hand_side;
        }
    }

    let mut solution = vec![0.0_f64; dimension];

    for row in (0..dimension).rev() {
        let trailing_sum = matrix[row]
            .iter()
            .enumerate()
            .skip(row + 1)
            .map(|(column, coefficient)| coefficient * solution[column])
            .sum::<f64>();

        solution[row] = (right_hand_side[row] - trailing_sum) / matrix[row][row];

        if !solution[row].is_finite() {
            return Err(format!("non-finite linear-system solution at row {row}"));
        }
    }

    Ok(solution)
}
fn calculate_metrics(targets: &[f64], predicted: &[f64]) -> Metrics {
    assert_eq!(targets.len(), predicted.len());
    assert!(!targets.is_empty());

    let sample_count = targets.len() as f64;
    let observed_mean = targets.iter().sum::<f64>() / sample_count;
    let predicted_mean = predicted.iter().sum::<f64>() / sample_count;

    let mut squared_error = 0.0_f64;
    let mut absolute_error = 0.0_f64;
    let mut total_variance = 0.0_f64;
    let mut calibration_covariance = 0.0_f64;
    let mut prediction_variance = 0.0_f64;
    let mut zero_count = 0_usize;
    let mut one_count = 0_usize;

    for (&target, &prediction) in targets.iter().zip(predicted) {
        let residual = target - prediction;
        squared_error += residual * residual;
        absolute_error += residual.abs();

        let centered_target = target - observed_mean;
        let centered_prediction = prediction - predicted_mean;

        total_variance += centered_target * centered_target;
        calibration_covariance += centered_prediction * centered_target;
        prediction_variance += centered_prediction * centered_prediction;

        if prediction == 0.0 {
            zero_count += 1;
        }

        if prediction == 1.0 {
            one_count += 1;
        }
    }

    let r_squared = if total_variance <= 1.0e-15 {
        0.0
    } else {
        1.0 - squared_error / total_variance
    };

    let calibration_slope = if prediction_variance <= 1.0e-15 {
        0.0
    } else {
        calibration_covariance / prediction_variance
    };

    let calibration_intercept = observed_mean - calibration_slope * predicted_mean;

    Metrics {
        mse: squared_error / sample_count,
        mae: absolute_error / sample_count,
        r_squared,
        spearman: spearman_correlation(targets, predicted),
        bias: predicted_mean - observed_mean,
        observed_mean,
        predicted_mean,
        calibration_intercept,
        calibration_slope,
        zero_fraction: zero_count as f64 / sample_count,
        one_fraction: one_count as f64 / sample_count,
    }
}

fn average_ranks(values: &[f64]) -> Vec<f64> {
    let mut indices = (0..values.len()).collect::<Vec<_>>();

    indices.sort_by(|&left, &right| {
        values[left]
            .total_cmp(&values[right])
            .then_with(|| left.cmp(&right))
    });

    let mut ranks = vec![0.0_f64; values.len()];
    let mut start = 0_usize;

    while start < indices.len() {
        let mut end = start + 1;

        while end < indices.len()
            && values[indices[start]].total_cmp(&values[indices[end]]) == std::cmp::Ordering::Equal
        {
            end += 1;
        }

        let average_rank = (start + 1 + end) as f64 / 2.0;

        for &index in &indices[start..end] {
            ranks[index] = average_rank;
        }

        start = end;
    }

    ranks
}

fn pearson_correlation(left: &[f64], right: &[f64]) -> f64 {
    assert_eq!(left.len(), right.len());

    let count = left.len() as f64;
    let left_mean = left.iter().sum::<f64>() / count;
    let right_mean = right.iter().sum::<f64>() / count;

    let mut covariance = 0.0_f64;
    let mut left_variance = 0.0_f64;
    let mut right_variance = 0.0_f64;

    for (&left_value, &right_value) in left.iter().zip(right) {
        let centered_left = left_value - left_mean;
        let centered_right = right_value - right_mean;

        covariance += centered_left * centered_right;
        left_variance += centered_left * centered_left;
        right_variance += centered_right * centered_right;
    }

    let denominator = (left_variance * right_variance).sqrt();

    if denominator <= 1.0e-15 {
        0.0
    } else {
        covariance / denominator
    }
}

fn spearman_correlation(left: &[f64], right: &[f64]) -> f64 {
    let left_ranks = average_ranks(left);
    let right_ranks = average_ranks(right);

    pearson_correlation(&left_ranks, &right_ranks)
}

fn percentile(sorted: &[f64], quantile: f64) -> f64 {
    let position = quantile * (sorted.len() - 1) as f64;
    let lower = position.floor() as usize;
    let upper = position.ceil() as usize;

    if lower == upper {
        sorted[lower]
    } else {
        let weight = position - lower as f64;
        sorted[lower] * (1.0 - weight) + sorted[upper] * weight
    }
}

fn confidence_interval(mut values: Vec<f64>) -> ConfidenceInterval {
    values.sort_by(f64::total_cmp);

    ConfidenceInterval {
        lower: percentile(&values, 0.025),
        median: percentile(&values, 0.500),
        upper: percentile(&values, 0.975),
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct TargetScaler {
    mean: f64,
    scale: f64,
}

impl TargetScaler {
    fn fit(records: &[Record], horizon_index: usize) -> Result<Self, String> {
        let values = records
            .iter()
            .map(|record| record.targets_u[horizon_index])
            .collect::<Vec<_>>();

        if values.is_empty() {
            return Err("training population contains no target values".to_owned());
        }

        let count = values.len() as f64;
        let mean = values.iter().sum::<f64>() / count;

        let variance = values
            .iter()
            .map(|value| {
                let difference = value - mean;
                difference * difference
            })
            .sum::<f64>()
            / count;

        let scale = variance.sqrt();

        if !mean.is_finite() || !scale.is_finite() {
            return Err("target has invalid training geometry".to_owned());
        }

        let scale = if scale <= 1.0e-12 { 1.0 } else { scale };

        Ok(Self { mean, scale })
    }

    fn standardize(self, value: f64) -> f64 {
        (value - self.mean) / self.scale
    }

    fn unstandardize(self, value: f64) -> f64 {
        self.mean + self.scale * value
    }
}

fn fit_target_scalers(records: &[Record]) -> Result<[TargetScaler; TARGET_HORIZON_COUNT], String> {
    let mut scalers = Vec::with_capacity(TARGET_HORIZON_COUNT);

    for horizon_index in 0..TARGET_HORIZON_COUNT {
        scalers.push(TargetScaler::fit(records, horizon_index)?);
    }

    scalers.try_into().map_err(|values: Vec<TargetScaler>| {
        format!(
            "expected {TARGET_HORIZON_COUNT} target scalers, received {}",
            values.len()
        )
    })
}

#[derive(Clone, Debug)]
struct BlockModelFit {
    seed_block: SeedBlockId,
    target_scalers: [TargetScaler; TARGET_HORIZON_COUNT],
    models: HorizonModels,
}

fn combine_width_3_and_4(width_3: &[Record], width_4: &[Record]) -> Vec<Record> {
    let mut combined = Vec::with_capacity(width_3.len() + width_4.len());

    combined.extend_from_slice(width_3);
    combined.extend_from_slice(width_4);

    combined
}

fn fit_block_models(
    seed_block: SeedBlockId,
    training_width_3: &[Record],
    training_width_4: &[Record],
) -> Result<BlockModelFit, String> {
    let combined = combine_width_3_and_4(training_width_3, training_width_4);
    let target_scalers = fit_target_scalers(&combined)?;
    let models = fit_horizon_models(&combined, &target_scalers)?;

    Ok(BlockModelFit {
        seed_block,
        target_scalers,
        models,
    })
}

#[derive(Clone, Debug)]
struct AggregateModelFit {
    blocks: [BlockModelFit; SEED_BLOCK_COUNT],
}

/// Validates that `seed_blocks` is exactly one family's frozen block order
/// (`frozen_block_order(family)` for the family of its first block).
fn validate_frozen_block_order(seed_blocks: &[SeedBlockId]) -> Result<(), String> {
    if seed_blocks.len() != SEED_BLOCK_COUNT {
        return Err(format!(
            "expected {SEED_BLOCK_COUNT} seed blocks in frozen order, received {}",
            seed_blocks.len()
        ));
    }

    let family = seed_blocks[0].family;
    let expected_order = frozen_block_order(family);

    for (&actual, &expected) in seed_blocks.iter().zip(&expected_order) {
        if actual != expected {
            return Err(format!(
                "requires the deterministic block order of family {}; found {} where {} was expected",
                family.label(),
                actual.label(),
                expected.label()
            ));
        }
    }

    Ok(())
}

impl AggregateModelFit {
    fn assemble(blocks: [BlockModelFit; SEED_BLOCK_COUNT]) -> Result<Self, String> {
        let seed_blocks = blocks.each_ref().map(|fit| fit.seed_block);

        validate_frozen_block_order(&seed_blocks)
            .map_err(|error| format!("aggregate model fit {error}"))?;

        Ok(Self { blocks })
    }

    fn family(&self) -> GeneratorFamily {
        self.blocks[0].seed_block.family
    }

    fn block(&self, seed_block: SeedBlockId) -> &BlockModelFit {
        self.blocks
            .iter()
            .find(|fit| fit.seed_block == seed_block)
            .expect("AggregateModelFit always contains exactly one fit per seed block")
    }
}

fn print_model(label: &str, model: &RidgeModel) {
    println!();
    println!("{label}");
    println!("  intercept : {:.12}", model.coefficients[0]);

    for index in 0..model.means.len() {
        println!(
            "  feature {index:02} | moyenne={:.12} | \
             échelle={:.12} | coefficient={:.12}",
            model.means[index],
            model.scales[index],
            model.coefficients[index + 1],
        );
    }
}

fn print_interval(label: &str, interval: ConfidenceInterval) {
    println!(
        "{label}: [{:.9}, {:.9}] (médiane {:.9})",
        interval.lower, interval.upper, interval.median
    );
}
#[derive(Clone, Debug)]
struct Tdi52PredictionSet {
    standardized: Vec<f64>,
    reconstructed_overlap: Vec<f64>,
}

/// One fitted layout's evaluation at a horizon: its standardized-U and
/// reconstructed-O metrics and its prediction set. TDI-5.7 compares two
/// fitted layouts, so this carries no layout identity of its own.
#[derive(Clone, Debug)]
struct PredictorEvaluation {
    standardized: Metrics,
    reconstructed: Metrics,
    predictions: Tdi52PredictionSet,
}

#[derive(Clone, Copy, Debug)]
struct Tdi52BootstrapIntervals {
    standardized_mse: ConfidenceInterval,
    reconstructed_mse: ConfidenceInterval,
    reconstructed_mae: ConfidenceInterval,
    relative_standardized_mse: ConfidenceInterval,
}

fn tdi52_relative_reduction(baseline: f64, challenger: f64) -> f64 {
    if !baseline.is_finite() || !challenger.is_finite() || baseline.abs() <= 1.0e-15 {
        0.0
    } else {
        (baseline - challenger) / baseline
    }
}

fn tdi52_reconstruct_overlap(target_u: f64) -> (f64, bool) {
    let raw = 1.0 - 2.0_f64.powf(-target_u);

    if !raw.is_finite() {
        return (0.0, true);
    }

    let clipped = raw.clamp(0.0, 1.0);

    (clipped, clipped != raw)
}

fn tdi52_predict(
    records: &[Record],
    horizon_index: usize,
    layout: FeatureLayout,
    model: &RidgeModel,
    scaler: TargetScaler,
) -> Result<Tdi52PredictionSet, String> {
    let mut standardized = Vec::with_capacity(records.len());
    let mut reconstructed_overlap = Vec::with_capacity(records.len());

    for record in records {
        let features = feature_layout(record, layout);
        let prediction = model.predict_linear(&features);

        if !prediction.is_finite() {
            return Err(format!(
                "non-finite standardized prediction for {} at horizon {}",
                layout.label(),
                TARGET_HORIZONS[horizon_index],
            ));
        }

        let target_u = scaler.unstandardize(prediction);

        if !target_u.is_finite() {
            return Err(format!(
                "non-finite unstandardized prediction for {} at horizon {}",
                layout.label(),
                TARGET_HORIZONS[horizon_index],
            ));
        }

        let (overlap, _clipped) = tdi52_reconstruct_overlap(target_u);

        standardized.push(prediction);
        reconstructed_overlap.push(overlap);
    }

    Ok(Tdi52PredictionSet {
        standardized,
        reconstructed_overlap,
    })
}

/// Evaluates one fitted ridge layout at a horizon: its standardized-U and
/// reconstructed-O metrics plus its prediction set. TDI-5.7 compares two
/// fitted layouts only — the naive persistence competitor of TDI-5.5 is
/// dropped (preregistration Section 7), so every comparison runs through the
/// identical paired / stratified-aggregate bootstrap and four-way classifier.
fn evaluate_layout(
    layout: FeatureLayout,
    records: &[Record],
    horizon_index: usize,
    models: &HorizonModels,
    scaler: TargetScaler,
    standardized_targets: &[f64],
    overlap_targets: &[f64],
) -> Result<PredictorEvaluation, String> {
    let predictions = tdi52_predict(
        records,
        horizon_index,
        layout,
        models.get(horizon_index, layout),
        scaler,
    )?;

    let standardized = calculate_metrics(standardized_targets, &predictions.standardized);
    let reconstructed = calculate_metrics(overlap_targets, &predictions.reconstructed_overlap);

    Ok(PredictorEvaluation {
        standardized,
        reconstructed,
        predictions,
    })
}

fn tdi52_paired_bootstrap(
    seed_block: SeedBlockId,
    records: &[Record],
    horizon_index: usize,
    scaler: TargetScaler,
    baseline: &Tdi52PredictionSet,
    challenger: &Tdi52PredictionSet,
) -> Result<Tdi52BootstrapIntervals, String> {
    let count = records.len();

    if count == 0
        || baseline.standardized.len() != count
        || challenger.standardized.len() != count
        || baseline.reconstructed_overlap.len() != count
        || challenger.reconstructed_overlap.len() != count
    {
        return Err("invalid paired-bootstrap dimensions".to_owned());
    }

    let mut generator = DeterministicRng::new(seed_block.bootstrap_seed());

    let mut standardized_mse = Vec::with_capacity(BOOTSTRAP_REPLICATES);

    let mut reconstructed_mse = Vec::with_capacity(BOOTSTRAP_REPLICATES);

    let mut reconstructed_mae = Vec::with_capacity(BOOTSTRAP_REPLICATES);

    let mut relative_standardized_mse = Vec::with_capacity(BOOTSTRAP_REPLICATES);

    for _ in 0..BOOTSTRAP_REPLICATES {
        let mut baseline_standardized_squared = 0.0;
        let mut challenger_standardized_squared = 0.0;

        let mut baseline_overlap_squared = 0.0;
        let mut challenger_overlap_squared = 0.0;

        let mut baseline_overlap_absolute = 0.0;
        let mut challenger_overlap_absolute = 0.0;

        for _ in 0..count {
            let index = generator.index(count);
            let record = &records[index];

            let standardized_target = scaler.standardize(record.targets_u[horizon_index]);

            let baseline_standardized_residual = standardized_target - baseline.standardized[index];

            let challenger_standardized_residual =
                standardized_target - challenger.standardized[index];

            baseline_standardized_squared +=
                baseline_standardized_residual * baseline_standardized_residual;

            challenger_standardized_squared +=
                challenger_standardized_residual * challenger_standardized_residual;

            let overlap_target = record.overlaps[horizon_index];

            let baseline_overlap_residual = overlap_target - baseline.reconstructed_overlap[index];

            let challenger_overlap_residual =
                overlap_target - challenger.reconstructed_overlap[index];

            baseline_overlap_squared += baseline_overlap_residual * baseline_overlap_residual;

            challenger_overlap_squared += challenger_overlap_residual * challenger_overlap_residual;

            baseline_overlap_absolute += baseline_overlap_residual.abs();

            challenger_overlap_absolute += challenger_overlap_residual.abs();
        }

        let denominator = count as f64;

        let baseline_standardized_mse = baseline_standardized_squared / denominator;
        let challenger_standardized_mse = challenger_standardized_squared / denominator;

        standardized_mse.push(baseline_standardized_mse - challenger_standardized_mse);

        relative_standardized_mse.push(tdi52_relative_reduction(
            baseline_standardized_mse,
            challenger_standardized_mse,
        ));

        reconstructed_mse.push(
            baseline_overlap_squared / denominator - challenger_overlap_squared / denominator,
        );

        reconstructed_mae.push(
            baseline_overlap_absolute / denominator - challenger_overlap_absolute / denominator,
        );
    }

    Ok(Tdi52BootstrapIntervals {
        standardized_mse: confidence_interval(standardized_mse),
        reconstructed_mse: confidence_interval(reconstructed_mse),
        reconstructed_mae: confidence_interval(reconstructed_mae),
        relative_standardized_mse: confidence_interval(relative_standardized_mse),
    })
}

struct BlockComparisonInputs<'a> {
    seed_block: SeedBlockId,
    records: &'a [Record],
    scaler: TargetScaler,
    baseline: &'a Tdi52PredictionSet,
    challenger: &'a Tdi52PredictionSet,
}

fn aggregate_paired_bootstrap(
    horizon_index: usize,
    blocks: &[BlockComparisonInputs<'_>],
) -> Result<Tdi52BootstrapIntervals, String> {
    let seed_blocks = blocks
        .iter()
        .map(|block| block.seed_block)
        .collect::<Vec<_>>();

    validate_frozen_block_order(&seed_blocks)
        .map_err(|error| format!("aggregate bootstrap {error}"))?;

    // Every block in an aggregate belongs to the same family (validated
    // above); that family's stratified-aggregate bootstrap seed is disjoint
    // from every other family's (Section 9).
    let family = seed_blocks[0].family;

    for block in blocks {
        let count = block.records.len();

        if count == 0
            || block.baseline.standardized.len() != count
            || block.challenger.standardized.len() != count
            || block.baseline.reconstructed_overlap.len() != count
            || block.challenger.reconstructed_overlap.len() != count
        {
            return Err("invalid aggregate paired-bootstrap dimensions".to_owned());
        }
    }

    let mut generator = DeterministicRng::new(family_aggregate_bootstrap_seed(family));

    let mut standardized_mse = Vec::with_capacity(BOOTSTRAP_REPLICATES);
    let mut reconstructed_mse = Vec::with_capacity(BOOTSTRAP_REPLICATES);
    let mut reconstructed_mae = Vec::with_capacity(BOOTSTRAP_REPLICATES);
    let mut relative_standardized_mse = Vec::with_capacity(BOOTSTRAP_REPLICATES);

    for _ in 0..BOOTSTRAP_REPLICATES {
        let mut baseline_standardized_squared = 0.0;
        let mut challenger_standardized_squared = 0.0;

        let mut baseline_overlap_squared = 0.0;
        let mut challenger_overlap_squared = 0.0;

        let mut baseline_overlap_absolute = 0.0;
        let mut challenger_overlap_absolute = 0.0;

        let mut total_count = 0_usize;

        for block in blocks {
            let count = block.records.len();

            for _ in 0..count {
                let index = generator.index(count);
                let record = &block.records[index];

                let standardized_target = block.scaler.standardize(record.targets_u[horizon_index]);

                let baseline_standardized_residual =
                    standardized_target - block.baseline.standardized[index];

                let challenger_standardized_residual =
                    standardized_target - block.challenger.standardized[index];

                baseline_standardized_squared +=
                    baseline_standardized_residual * baseline_standardized_residual;

                challenger_standardized_squared +=
                    challenger_standardized_residual * challenger_standardized_residual;

                let overlap_target = record.overlaps[horizon_index];

                let baseline_overlap_residual =
                    overlap_target - block.baseline.reconstructed_overlap[index];

                let challenger_overlap_residual =
                    overlap_target - block.challenger.reconstructed_overlap[index];

                baseline_overlap_squared += baseline_overlap_residual * baseline_overlap_residual;

                challenger_overlap_squared +=
                    challenger_overlap_residual * challenger_overlap_residual;

                baseline_overlap_absolute += baseline_overlap_residual.abs();
                challenger_overlap_absolute += challenger_overlap_residual.abs();
            }

            total_count += count;
        }

        let denominator = total_count as f64;

        let baseline_standardized_mse = baseline_standardized_squared / denominator;
        let challenger_standardized_mse = challenger_standardized_squared / denominator;

        standardized_mse.push(baseline_standardized_mse - challenger_standardized_mse);

        relative_standardized_mse.push(tdi52_relative_reduction(
            baseline_standardized_mse,
            challenger_standardized_mse,
        ));

        reconstructed_mse.push(
            baseline_overlap_squared / denominator - challenger_overlap_squared / denominator,
        );

        reconstructed_mae.push(
            baseline_overlap_absolute / denominator - challenger_overlap_absolute / denominator,
        );
    }

    Ok(Tdi52BootstrapIntervals {
        standardized_mse: confidence_interval(standardized_mse),
        reconstructed_mse: confidence_interval(reconstructed_mse),
        reconstructed_mae: confidence_interval(reconstructed_mae),
        relative_standardized_mse: confidence_interval(relative_standardized_mse),
    })
}

#[derive(Clone, Debug)]
struct BlockComparison {
    seed_block: SeedBlockId,
    standardized_targets: Vec<f64>,
    overlap_targets: Vec<f64>,
    baseline: PredictorEvaluation,
    challenger: PredictorEvaluation,
    bootstrap: Tdi52BootstrapIntervals,
}

fn evaluate_block_comparison(
    seed_block: SeedBlockId,
    holdout_records: &[Record],
    horizon_index: usize,
    models: &HorizonModels,
    scalers: &[TargetScaler; TARGET_HORIZON_COUNT],
    baseline_layout: FeatureLayout,
    challenger_layout: FeatureLayout,
) -> Result<BlockComparison, String> {
    if holdout_records.is_empty() {
        return Err("cannot evaluate an empty population".to_owned());
    }

    let scaler = scalers[horizon_index];

    let standardized_targets = holdout_records
        .iter()
        .map(|record| scaler.standardize(record.targets_u[horizon_index]))
        .collect::<Vec<_>>();

    let overlap_targets = overlap_values(holdout_records, horizon_index);

    let baseline = evaluate_layout(
        baseline_layout,
        holdout_records,
        horizon_index,
        models,
        scaler,
        &standardized_targets,
        &overlap_targets,
    )?;

    let challenger = evaluate_layout(
        challenger_layout,
        holdout_records,
        horizon_index,
        models,
        scaler,
        &standardized_targets,
        &overlap_targets,
    )?;

    let bootstrap = tdi52_paired_bootstrap(
        seed_block,
        holdout_records,
        horizon_index,
        scaler,
        &baseline.predictions,
        &challenger.predictions,
    )?;

    Ok(BlockComparison {
        seed_block,
        standardized_targets,
        overlap_targets,
        baseline,
        challenger,
        bootstrap,
    })
}

fn pooled_standardized_metrics(blocks: &[BlockComparison]) -> (Metrics, Metrics) {
    let mut targets = Vec::new();
    let mut baseline_predictions = Vec::new();
    let mut challenger_predictions = Vec::new();

    for block in blocks {
        targets.extend_from_slice(&block.standardized_targets);
        baseline_predictions.extend_from_slice(&block.baseline.predictions.standardized);
        challenger_predictions.extend_from_slice(&block.challenger.predictions.standardized);
    }

    (
        calculate_metrics(&targets, &baseline_predictions),
        calculate_metrics(&targets, &challenger_predictions),
    )
}

fn pooled_reconstructed_metrics(blocks: &[BlockComparison]) -> (Metrics, Metrics) {
    let mut targets = Vec::new();
    let mut baseline_predictions = Vec::new();
    let mut challenger_predictions = Vec::new();

    for block in blocks {
        targets.extend_from_slice(&block.overlap_targets);
        baseline_predictions.extend_from_slice(&block.baseline.predictions.reconstructed_overlap);
        challenger_predictions
            .extend_from_slice(&block.challenger.predictions.reconstructed_overlap);
    }

    (
        calculate_metrics(&targets, &baseline_predictions),
        calculate_metrics(&targets, &challenger_predictions),
    )
}

#[derive(Clone, Debug)]
struct AggregateComparison {
    blocks: Vec<BlockComparison>,
    aggregate_baseline_standardized: Metrics,
    aggregate_challenger_standardized: Metrics,
    aggregate_baseline_reconstructed: Metrics,
    aggregate_challenger_reconstructed: Metrics,
    aggregate_bootstrap: Tdi52BootstrapIntervals,
}

impl AggregateComparison {
    fn family(&self) -> GeneratorFamily {
        self.blocks[0].seed_block.family
    }

    fn block(&self, seed_block: SeedBlockId) -> &BlockComparison {
        self.blocks
            .iter()
            .find(|comparison| comparison.seed_block == seed_block)
            .expect("AggregateComparison always contains exactly one comparison per seed block")
    }
}

fn evaluate_aggregate_comparison(
    horizon_index: usize,
    aggregate_fit: &AggregateModelFit,
    holdout_records: [&[Record]; SEED_BLOCK_COUNT],
    baseline_layout: FeatureLayout,
    challenger_layout: FeatureLayout,
) -> Result<AggregateComparison, String> {
    let mut blocks = Vec::with_capacity(SEED_BLOCK_COUNT);

    for (seed_block, records) in frozen_block_order(aggregate_fit.family())
        .into_iter()
        .zip(holdout_records)
    {
        let block_fit = aggregate_fit.block(seed_block);

        blocks.push(evaluate_block_comparison(
            seed_block,
            records,
            horizon_index,
            &block_fit.models,
            &block_fit.target_scalers,
            baseline_layout,
            challenger_layout,
        )?);
    }

    let (aggregate_baseline_standardized, aggregate_challenger_standardized) =
        pooled_standardized_metrics(&blocks);

    let (aggregate_baseline_reconstructed, aggregate_challenger_reconstructed) =
        pooled_reconstructed_metrics(&blocks);

    let bootstrap_inputs = blocks
        .iter()
        .zip(holdout_records)
        .map(|(comparison, records)| BlockComparisonInputs {
            seed_block: comparison.seed_block,
            records,
            scaler: aggregate_fit.block(comparison.seed_block).target_scalers[horizon_index],
            baseline: &comparison.baseline.predictions,
            challenger: &comparison.challenger.predictions,
        })
        .collect::<Vec<_>>();

    let aggregate_bootstrap = aggregate_paired_bootstrap(horizon_index, &bootstrap_inputs)?;

    Ok(AggregateComparison {
        blocks,
        aggregate_baseline_standardized,
        aggregate_challenger_standardized,
        aggregate_baseline_reconstructed,
        aggregate_challenger_reconstructed,
        aggregate_bootstrap,
    })
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CriterionCClassification {
    Beneficial,
    Equivalent,
    Harmful,
    Inconclusive,
}

impl CriterionCClassification {
    const fn label(self) -> &'static str {
        match self {
            Self::Beneficial => "beneficial",
            Self::Equivalent => "equivalent",
            Self::Harmful => "harmful",
            Self::Inconclusive => "inconclusive",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct CriterionCResult {
    classification: CriterionCClassification,
    blocks_confirming_benefit: usize,
    aggregate_relative_improvement_at_least_2_percent: bool,
    aggregate_bootstrap_lower_bound_positive: bool,
    all_block_point_estimates_within_equivalence_margin: bool,
    block_intervals_within_equivalence_margin: usize,
    aggregate_interval_within_equivalence_margin: bool,
    blocks_confirming_harm: usize,
    aggregate_relative_worsening_at_least_2_percent: bool,
    aggregate_bootstrap_upper_bound_negative: bool,
}

impl CriterionCResult {
    const MARGIN: f64 = 0.02;

    fn is_beneficial(&self) -> bool {
        self.blocks_confirming_benefit >= 2
            && self.aggregate_relative_improvement_at_least_2_percent
            && self.aggregate_bootstrap_lower_bound_positive
    }

    fn is_equivalent(&self) -> bool {
        self.all_block_point_estimates_within_equivalence_margin
            && self.block_intervals_within_equivalence_margin >= 2
            && self.aggregate_interval_within_equivalence_margin
    }

    fn is_harmful(&self) -> bool {
        self.blocks_confirming_harm >= 2
            && self.aggregate_relative_worsening_at_least_2_percent
            && self.aggregate_bootstrap_upper_bound_negative
    }

    /// Fixed precedence Beneficial > Equivalent > Harmful > Inconclusive
    /// over the already-computed sub-condition flags. Factored out so the
    /// precedence is unit-testable from a hand-built `CriterionCResult`
    /// without constructing an `AggregateComparison`.
    fn classify(&self) -> CriterionCClassification {
        if self.is_beneficial() {
            CriterionCClassification::Beneficial
        } else if self.is_equivalent() {
            CriterionCClassification::Equivalent
        } else if self.is_harmful() {
            CriterionCClassification::Harmful
        } else {
            CriterionCClassification::Inconclusive
        }
    }
}

fn evaluate_criterion_c(comparison: &AggregateComparison) -> CriterionCResult {
    let order = frozen_block_order(comparison.family());

    let block_relative_reductions = order.map(|seed_block| {
        let block = comparison.block(seed_block);

        tdi52_relative_reduction(
            block.baseline.standardized.mse,
            block.challenger.standardized.mse,
        )
    });

    let blocks_confirming_benefit = order
        .iter()
        .zip(block_relative_reductions)
        .filter(|&(&seed_block, relative_reduction)| {
            relative_reduction >= CriterionCResult::MARGIN
                && comparison
                    .block(seed_block)
                    .bootstrap
                    .standardized_mse
                    .lower
                    > 0.0
        })
        .count();

    let blocks_confirming_harm = order
        .iter()
        .zip(block_relative_reductions)
        .filter(|&(&seed_block, relative_reduction)| {
            relative_reduction <= -CriterionCResult::MARGIN
                && comparison
                    .block(seed_block)
                    .bootstrap
                    .standardized_mse
                    .upper
                    < 0.0
        })
        .count();

    let all_block_point_estimates_within_equivalence_margin =
        block_relative_reductions.iter().all(|&reduction| {
            (-CriterionCResult::MARGIN..=CriterionCResult::MARGIN).contains(&reduction)
        });

    let block_intervals_within_equivalence_margin = order
        .iter()
        .filter(|&&seed_block| {
            let interval = comparison
                .block(seed_block)
                .bootstrap
                .relative_standardized_mse;

            interval.lower >= -CriterionCResult::MARGIN
                && interval.upper <= CriterionCResult::MARGIN
        })
        .count();

    let aggregate_relative_reduction = tdi52_relative_reduction(
        comparison.aggregate_baseline_standardized.mse,
        comparison.aggregate_challenger_standardized.mse,
    );

    let aggregate_relative_improvement_at_least_2_percent =
        aggregate_relative_reduction >= CriterionCResult::MARGIN;

    let aggregate_relative_worsening_at_least_2_percent =
        aggregate_relative_reduction <= -CriterionCResult::MARGIN;

    let aggregate_bootstrap_lower_bound_positive =
        comparison.aggregate_bootstrap.standardized_mse.lower > 0.0;

    let aggregate_bootstrap_upper_bound_negative =
        comparison.aggregate_bootstrap.standardized_mse.upper < 0.0;

    let aggregate_relative_interval = comparison.aggregate_bootstrap.relative_standardized_mse;

    let aggregate_interval_within_equivalence_margin = aggregate_relative_interval.lower
        >= -CriterionCResult::MARGIN
        && aggregate_relative_interval.upper <= CriterionCResult::MARGIN;

    let mut result = CriterionCResult {
        classification: CriterionCClassification::Inconclusive,
        blocks_confirming_benefit,
        aggregate_relative_improvement_at_least_2_percent,
        aggregate_bootstrap_lower_bound_positive,
        all_block_point_estimates_within_equivalence_margin,
        block_intervals_within_equivalence_margin,
        aggregate_interval_within_equivalence_margin,
        blocks_confirming_harm,
        aggregate_relative_worsening_at_least_2_percent,
        aggregate_bootstrap_upper_bound_negative,
    };

    result.classification = result.classify();

    result
}

fn focal_horizon_indices() -> [usize; FOCAL_HORIZON_COUNT] {
    std::array::from_fn(|slot| {
        target_horizon_index(FOCAL_HORIZONS[slot])
            .expect("every focal horizon belongs to the target horizons")
    })
}

/// One horizon's comparison of a challenger predictor against a baseline
/// predictor: the aggregate comparison, its four-way classification, and the
/// aggregate relative-MSE reduction of the challenger over the baseline in
/// standardized-U space (the challenger's marginal value at that horizon).
#[derive(Clone, Debug)]
struct HorizonComparison {
    horizon: usize,
    comparison: AggregateComparison,
    result: CriterionCResult,
    aggregate_relative_reduction: f64,
}

fn evaluate_horizon_comparison(
    horizon_index: usize,
    aggregate_fit: &AggregateModelFit,
    combined_holdout_records: [&[Record]; SEED_BLOCK_COUNT],
    baseline_layout: FeatureLayout,
    challenger_layout: FeatureLayout,
) -> Result<HorizonComparison, String> {
    let comparison = evaluate_aggregate_comparison(
        horizon_index,
        aggregate_fit,
        combined_holdout_records,
        baseline_layout,
        challenger_layout,
    )?;

    let result = evaluate_criterion_c(&comparison);

    let aggregate_relative_reduction = tdi52_relative_reduction(
        comparison.aggregate_baseline_standardized.mse,
        comparison.aggregate_challenger_standardized.mse,
    );

    Ok(HorizonComparison {
        horizon: TARGET_HORIZONS[horizon_index],
        comparison,
        result,
        aggregate_relative_reduction,
    })
}

/// Number of exact descriptors summarised by TDI-5.7D: delta, delta_bar, s2, s3.
const DESCRIPTOR_MEAN_COUNT: usize = CONTRACTION_FEATURE_COUNT + SPECTRAL_FEATURE_COUNT;

/// One generator family's SKT-vs-SK result at the focal horizons (Section 13),
/// plus the exact-descriptor holdout means used by TDI-5.7D.
#[derive(Clone, Debug)]
struct FamilyReport {
    family: GeneratorFamily,
    blocks: Vec<BlockPopulations>,
    aggregate_fit: AggregateModelFit,
    /// SKT-vs-SK comparison at every grid horizon H = {3..8} (TARGET_HORIZONS
    /// order). The prereg reports per-family per-horizon reductions across the
    /// grid (Sections 12, 17); the focal horizons U3/U6 that drive criteria
    /// 5.7A/5.7B are the entries at `focal_horizon_indices()`.
    grid: Vec<HorizonComparison>,
    /// Holdout means of [delta, delta_bar, s2, s3] on this family's holdout.
    descriptor_means: [f64; DESCRIPTOR_MEAN_COUNT],
}

/// Criterion TDI-5.7A (Section 13): per-family SKT-vs-SK focal classifications
/// and the replication verdict — Beneficial at U3 and U6 for every family.
/// `non_replications` names each (family, horizon) that is not Beneficial (the
/// located non-replication).
#[derive(Clone, Debug)]
struct Tdi57CriterionA {
    per_family_focal: Vec<(
        GeneratorFamily,
        [CriterionCClassification; FOCAL_HORIZON_COUNT],
    )>,
    replicated: bool,
    non_replications: Vec<(GeneratorFamily, usize)>,
}

/// One focal horizon's across-family effect-size heterogeneity (Section 14).
#[derive(Clone, Copy, Debug, PartialEq)]
struct FocalHeterogeneity {
    horizon: usize,
    minimum: f64,
    maximum: f64,
    range: f64,
    all_exceed_margin: bool,
}

/// Criterion TDI-5.7B (Section 14): the across-family spread of the SKT-vs-SK
/// aggregate relative-MSE reduction, per focal horizon. Descriptive.
#[derive(Clone, Debug)]
struct Tdi57CriterionB {
    per_focal: Vec<FocalHeterogeneity>,
}

/// Criterion TDI-5.7C (Section 15, descriptive): SKT-vs-SK transfer from F0's
/// fitted models to F1's holdout at the focal horizons.
#[derive(Clone, Debug)]
struct Tdi57CriterionC {
    transfer_focal: Vec<HorizonComparison>,
    focal_classifications: [CriterionCClassification; FOCAL_HORIZON_COUNT],
}

/// Criterion TDI-5.7D (Section 15, descriptive): per-family exact-descriptor
/// holdout means [delta, delta_bar, s2, s3] and their across-family range.
#[derive(Clone, Debug)]
struct Tdi57CriterionD {
    per_family_means: Vec<(GeneratorFamily, [f64; DESCRIPTOR_MEAN_COUNT])>,
    ranges: [f64; DESCRIPTOR_MEAN_COUNT],
}

/// Holdout means of the four exact descriptors (delta, delta_bar, s2, s3) over
/// a family's combined holdout populations (TDI-5.7D).
fn family_descriptor_means(blocks: &[BlockPopulations]) -> [f64; DESCRIPTOR_MEAN_COUNT] {
    let mut sums = [0.0_f64; DESCRIPTOR_MEAN_COUNT];
    let mut count = 0_usize;

    for block in blocks {
        for record in block.combined_holdout() {
            sums[0] += record.contraction[0];
            sums[1] += record.contraction[1];
            sums[2] += record.spectral[0];
            sums[3] += record.spectral[1];
            count += 1;
        }
    }

    if count == 0 {
        return [0.0; DESCRIPTOR_MEAN_COUNT];
    }

    sums.map(|sum| sum / count as f64)
}

#[derive(Clone, Debug)]
struct Tdi57ExperimentReport {
    families: Vec<FamilyReport>,
    criterion_a: Tdi57CriterionA,
    criterion_b: Tdi57CriterionB,
    criterion_c: Tdi57CriterionC,
    criterion_d: Tdi57CriterionD,
}

/// Runs the full TDI-5.7 pipeline (generation of the width-3/width-4
/// populations across seed blocks J/K/L, per-block ridge fitting on the
/// contraction- and spectral-inclusive design, aggregation, and the three
/// TDI-5.7
/// criteria) over an arbitrary set of population specifications. Callers
/// control scale entirely through `population_specs`: the preregistered
/// `population_specs()` output requests the real 120,000-record run, while
/// tests and the termination smoke path pass tiny synthetic-scale specs
/// instead. This function is called with the real specs only from
/// `run_full_experiment`'s `--full` path, and only after that path's exact
/// confirmation-token check has passed; tests and the termination smoke
/// path never reach that branch.
fn run_family_pipeline(
    family: GeneratorFamily,
    population_specs: &[PopulationSpec],
) -> Result<FamilyReport, String> {
    let mut blocks = Vec::with_capacity(SEED_BLOCK_COUNT);

    for seed_block in frozen_block_order(family) {
        blocks.push(
            generate_block_populations(seed_block, population_specs)
                .map_err(|error| error.to_string())?,
        );
    }

    let mut block_fits = Vec::with_capacity(SEED_BLOCK_COUNT);

    for population in &blocks {
        block_fits.push(fit_block_models(
            population.seed_block,
            &population.training_width_3.report.records,
            &population.training_width_4.report.records,
        )?);
    }

    let block_fits: [BlockModelFit; SEED_BLOCK_COUNT] = block_fits.try_into().map_err(|_| {
        format!(
            "family {}: expected exactly {SEED_BLOCK_COUNT} block fits",
            family.label()
        )
    })?;

    let aggregate_fit = AggregateModelFit::assemble(block_fits)?;

    let combined_holdouts = blocks
        .iter()
        .map(BlockPopulations::combined_holdout)
        .collect::<Vec<_>>();

    let combined_holdout_refs: [&[Record]; SEED_BLOCK_COUNT] =
        std::array::from_fn(|index| combined_holdouts[index].as_slice());

    // SKT (challenger) vs SK (baseline) at EVERY grid horizon H = {3..8}: the
    // per-family per-horizon reductions the prereg reports (Sections 12, 17).
    // The focal horizons U3/U6 that drive criteria 5.7A/5.7B are the grid
    // entries at focal_horizon_indices().
    let mut grid = Vec::with_capacity(TARGET_HORIZON_COUNT);
    for horizon_index in 0..TARGET_HORIZON_COUNT {
        grid.push(evaluate_horizon_comparison(
            horizon_index,
            &aggregate_fit,
            combined_holdout_refs,
            FeatureLayout::Sk,
            FeatureLayout::Skt,
        )?);
    }

    let descriptor_means = family_descriptor_means(&blocks);

    Ok(FamilyReport {
        family,
        blocks,
        aggregate_fit,
        grid,
        descriptor_means,
    })
}

/// Runs the full TDI-5.7 pipeline: the inherited per-generator sub-pipeline
/// (generate 3 blocks, fit, aggregate, SKT-vs-SK at the focal horizons) once
/// per generator family F0..F3, then assembles the four cross-family criteria
/// (Sections 13-15). Callers control scale entirely through `population_specs`;
/// the real 480,000-record run is reached only from `run_full_experiment`'s
/// confirmed `--full` path.
fn run_tdi57_pipeline(
    population_specs: &[PopulationSpec],
) -> Result<Tdi57ExperimentReport, String> {
    validate_seed_reservations(population_specs)?;

    let mut families = Vec::with_capacity(GENERATOR_FAMILY_COUNT);
    for family in GeneratorFamily::ALL {
        families.push(run_family_pipeline(family, population_specs)?);
    }

    // TDI-5.7A — replication: SKT-vs-SK Beneficial at U3 and U6 for every family.
    // The focal comparisons are the grid entries at focal_horizon_indices()
    // (U3 -> grid[0], U6 -> grid[3]).
    let focal_indices = focal_horizon_indices();
    let mut per_family_focal = Vec::with_capacity(GENERATOR_FAMILY_COUNT);
    let mut non_replications = Vec::new();
    for family_report in &families {
        let focal_classifications: [CriterionCClassification; FOCAL_HORIZON_COUNT] =
            std::array::from_fn(|slot| {
                family_report.grid[focal_indices[slot]]
                    .result
                    .classification
            });

        for (slot, &classification) in focal_classifications.iter().enumerate() {
            if classification != CriterionCClassification::Beneficial {
                non_replications.push((family_report.family, FOCAL_HORIZONS[slot]));
            }
        }

        per_family_focal.push((family_report.family, focal_classifications));
    }
    let criterion_a = Tdi57CriterionA {
        per_family_focal,
        replicated: non_replications.is_empty(),
        non_replications,
    };

    // TDI-5.7B — effect-size heterogeneity across families, per focal horizon.
    let mut per_focal = Vec::with_capacity(FOCAL_HORIZON_COUNT);
    for (slot, &horizon) in FOCAL_HORIZONS.iter().enumerate() {
        let reductions = families
            .iter()
            .map(|family_report| {
                family_report.grid[focal_indices[slot]].aggregate_relative_reduction
            })
            .collect::<Vec<_>>();

        let minimum = reductions.iter().copied().fold(f64::INFINITY, f64::min);
        let maximum = reductions.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let all_exceed_margin = reductions
            .iter()
            .all(|&value| value > CriterionCResult::MARGIN);

        per_focal.push(FocalHeterogeneity {
            horizon,
            minimum,
            maximum,
            range: maximum - minimum,
            all_exceed_margin,
        });
    }
    let criterion_b = Tdi57CriterionB { per_focal };

    // TDI-5.7C — transfer: F0's fitted SK/SKT models evaluated on F1's holdout.
    let source = families
        .iter()
        .find(|report| report.family == GeneratorFamily::F0Base)
        .ok_or_else(|| "missing family F0 in the pipeline".to_owned())?;
    let target = families
        .iter()
        .find(|report| report.family == GeneratorFamily::F1Sparse)
        .ok_or_else(|| "missing family F1 in the pipeline".to_owned())?;

    let target_holdouts = target
        .blocks
        .iter()
        .map(BlockPopulations::combined_holdout)
        .collect::<Vec<_>>();
    let target_holdout_refs: [&[Record]; SEED_BLOCK_COUNT] =
        std::array::from_fn(|index| target_holdouts[index].as_slice());

    let focal_indices = focal_horizon_indices();
    let mut transfer_focal = Vec::with_capacity(FOCAL_HORIZON_COUNT);
    for &horizon_index in &focal_indices {
        transfer_focal.push(evaluate_horizon_comparison(
            horizon_index,
            &source.aggregate_fit,
            target_holdout_refs,
            FeatureLayout::Sk,
            FeatureLayout::Skt,
        )?);
    }
    let criterion_c = Tdi57CriterionC {
        focal_classifications: std::array::from_fn(|slot| {
            transfer_focal[slot].result.classification
        }),
        transfer_focal,
    };

    // TDI-5.7D — descriptor drift: per-family holdout means and their range.
    let per_family_means = families
        .iter()
        .map(|report| (report.family, report.descriptor_means))
        .collect::<Vec<_>>();
    let ranges: [f64; DESCRIPTOR_MEAN_COUNT] = std::array::from_fn(|descriptor| {
        let minimum = families
            .iter()
            .map(|report| report.descriptor_means[descriptor])
            .fold(f64::INFINITY, f64::min);
        let maximum = families
            .iter()
            .map(|report| report.descriptor_means[descriptor])
            .fold(f64::NEG_INFINITY, f64::max);
        maximum - minimum
    });
    let criterion_d = Tdi57CriterionD {
        per_family_means,
        ranges,
    };

    Ok(Tdi57ExperimentReport {
        families,
        criterion_a,
        criterion_b,
        criterion_c,
        criterion_d,
    })
}

fn tdi52_print_bootstrap_intervals(
    label: &str,
    horizon: usize,
    intervals: Tdi52BootstrapIntervals,
) {
    println!();
    println!("{label}");

    print_interval(
        &format!("  IC 95 % amélioration MSE U{horizon} standardisée"),
        intervals.standardized_mse,
    );

    print_interval(
        &format!("  IC 95 % amélioration MSE O{horizon} reconstruite"),
        intervals.reconstructed_mse,
    );

    print_interval(
        &format!("  IC 95 % amélioration MAE O{horizon} reconstruite"),
        intervals.reconstructed_mae,
    );

    print_interval(
        &format!("  IC 95 % réduction relative MSE U{horizon} standardisée"),
        intervals.relative_standardized_mse,
    );
}

fn tdi52_print_metrics(label: &str, metrics: Metrics) {
    println!("{label}");
    println!("  MSE                    : {:.12}", metrics.mse);
    println!("  MAE                    : {:.12}", metrics.mae);
    println!("  R²                     : {:.12}", metrics.r_squared);
    println!("  Spearman               : {:.12}", metrics.spearman);
    println!("  biais                  : {:.12}", metrics.bias);
    println!("  moyenne observée       : {:.12}", metrics.observed_mean);
    println!("  moyenne prédite        : {:.12}", metrics.predicted_mean);
    println!(
        "  calibration intercept  : {:.12}",
        metrics.calibration_intercept
    );
    println!(
        "  calibration pente      : {:.12}",
        metrics.calibration_slope
    );
    println!("  fraction borne basse   : {:.12}", metrics.zero_fraction);
    println!("  fraction borne haute   : {:.12}", metrics.one_fraction);
}
fn tdi52_print_models(models: &HorizonModels, scalers: &[TargetScaler; TARGET_HORIZON_COUNT]) {
    println!();
    println!("=== NORMALISATIONS ET MODÈLES ===");

    for (horizon_index, &horizon) in TARGET_HORIZONS.iter().enumerate() {
        let scaler = scalers[horizon_index];

        println!();
        println!(
            "U_{horizon} | moyenne cible={:.12} | échelle cible={:.12}",
            scaler.mean, scaler.scale,
        );

        for layout in FeatureLayout::ALL {
            print_model(
                &format!("U_{horizon} — {}", layout.label()),
                models.get(horizon_index, layout),
            );
        }
    }
}

fn tdi52_command_output(program: &str, arguments: &[&str]) -> String {
    std::process::Command::new(program)
        .args(arguments)
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_owned())
        .filter(|output| !output.is_empty())
        .unwrap_or_else(|| "indisponible".to_owned())
}
fn tdi52_repository_root() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(std::path::Path::to_path_buf)
        .unwrap_or_else(|| std::path::PathBuf::from("."))
}

/// Hashes a repository-relative file with `sha256sum`, matching the
/// shell-out convention already used by this workspace's frozen-hash
/// tests. Freeze-time artifacts (e.g. the TDI-5.3 scientific manifest) do
/// not exist yet while TDI-5.3 remains under implementation, so a missing
/// file is reported honestly rather than treated as an error.
fn tdi52_sha256_of_repo_file(relative_path: &str) -> String {
    let path = tdi52_repository_root().join(relative_path);

    if !path.is_file() {
        return format!("non généré ({relative_path} absent)");
    }

    std::process::Command::new("sha256sum")
        .arg(&path)
        .output()
        .ok()
        .filter(|output| output.status.success())
        .and_then(|output| {
            String::from_utf8_lossy(&output.stdout)
                .split_whitespace()
                .next()
                .map(str::to_owned)
        })
        .unwrap_or_else(|| "indisponible".to_owned())
}

/// Provenance and integrity (TDI-5.7 preregistration Section 17, items
/// 1-5): git commit, compiler/Cargo versions, and the SHA-256 of the v57
/// evaluator, the TDI-5.7 preregistration and the TDI-5.7 scientific
/// manifest — plus the frozen TDI-5.6, TDI-5.5, TDI-5.4, TDI-5.3, TDI-5.2
/// and TDI-5.1 ancestors' own identities, read live and printed for
/// provenance (Section 1).
fn print_tdi52_provenance() {
    println!();
    println!("=== PROVENANCE ET INTÉGRITÉ (Section 17) ===");
    println!(
        "git commit                     : {}",
        tdi52_command_output("git", &["rev-parse", "HEAD"])
    );
    println!(
        "rustc                          : {}",
        tdi52_command_output("rustc", &["--version"])
    );
    println!(
        "cargo                          : {}",
        tdi52_command_output("cargo", &["--version"])
    );
    println!(
        "évaluateur TDI-5.7 SHA-256      : {}",
        tdi52_sha256_of_repo_file("tdi-bench/src/bin/tdi-independent-overlap-ablation-v57.rs")
    );
    println!(
        "préenregistrement TDI-5.7 SHA-256 : {}",
        tdi52_sha256_of_repo_file("docs/TDI-5.7-GENERATOR-ROBUSTNESS-PREREGISTRATION.md")
    );
    println!(
        "manifeste scientifique TDI-5.7 SHA-256 : {}",
        tdi52_sha256_of_repo_file("docs/TDI-5.7-SCIENTIFIC-CODE.sha256")
    );
    println!();
    println!("--- provenance TDI-5.6 (ancêtre gelé, inchangé) ---");
    println!(
        "évaluateur TDI-5.6 SHA-256      : {}",
        tdi52_sha256_of_repo_file("tdi-bench/src/bin/tdi-independent-overlap-ablation-v56.rs")
    );
    println!(
        "préenregistrement TDI-5.6 SHA-256 : {}",
        tdi52_sha256_of_repo_file("docs/TDI-5.6-EXACT-SPECTRAL-CHALLENGE-PREREGISTRATION.md")
    );
    println!(
        "manifeste scientifique TDI-5.6 SHA-256 : {}",
        tdi52_sha256_of_repo_file("docs/TDI-5.6-SCIENTIFIC-CODE.sha256")
    );
    println!();
    println!("--- provenance TDI-5.5 (ancêtre gelé, inchangé) ---");
    println!(
        "évaluateur TDI-5.5 SHA-256      : {}",
        tdi52_sha256_of_repo_file("tdi-bench/src/bin/tdi-independent-overlap-ablation-v55.rs")
    );
    println!(
        "préenregistrement TDI-5.5 SHA-256 : {}",
        tdi52_sha256_of_repo_file("docs/TDI-5.5-OVERLAP-BASELINE-CHALLENGE-PREREGISTRATION.md")
    );
    println!(
        "manifeste scientifique TDI-5.5 SHA-256 : {}",
        tdi52_sha256_of_repo_file("docs/TDI-5.5-SCIENTIFIC-CODE.sha256")
    );
    println!();
    println!("--- provenance TDI-5.4 (ancêtre gelé, inchangé) ---");
    println!(
        "évaluateur TDI-5.4 SHA-256      : {}",
        tdi52_sha256_of_repo_file("tdi-bench/src/bin/tdi-independent-overlap-ablation-v54.rs")
    );
    println!(
        "préenregistrement TDI-5.4 SHA-256 : {}",
        tdi52_sha256_of_repo_file("docs/TDI-5.4-NONLINEAR-OVERLAP-SUFFICIENCY-PREREGISTRATION.md")
    );
    println!(
        "manifeste scientifique TDI-5.4 SHA-256 : {}",
        tdi52_sha256_of_repo_file("docs/TDI-5.4-SCIENTIFIC-CODE.sha256")
    );
    println!();
    println!("--- provenance TDI-5.3 (ancêtre gelé, inchangé) ---");
    println!(
        "évaluateur TDI-5.3 SHA-256      : {}",
        tdi52_sha256_of_repo_file("tdi-bench/src/bin/tdi-independent-overlap-ablation-v53.rs")
    );
    println!(
        "préenregistrement TDI-5.3 SHA-256 : {}",
        tdi52_sha256_of_repo_file("docs/TDI-5.3-INDEPENDENT-OVERLAP-ACTIVATION-PREREGISTRATION.md")
    );
    println!(
        "manifeste scientifique TDI-5.3 SHA-256 : {}",
        tdi52_sha256_of_repo_file("docs/TDI-5.3-SCIENTIFIC-CODE.sha256")
    );
    println!();
    println!("--- provenance TDI-5.2 (ancêtre gelé, inchangé) ---");
    println!(
        "évaluateur TDI-5.2 SHA-256      : {}",
        tdi52_sha256_of_repo_file("tdi-bench/src/bin/tdi-independent-overlap-ablation-v52.rs")
    );
    println!(
        "préenregistrement TDI-5.2 SHA-256 : {}",
        tdi52_sha256_of_repo_file("docs/TDI-5.2-INDEPENDENT-OVERLAP-ABLATION-PREREGISTRATION.md")
    );
    println!(
        "manifeste scientifique TDI-5.2 SHA-256 : {}",
        tdi52_sha256_of_repo_file("docs/TDI-5.2-SCIENTIFIC-CODE.sha256")
    );
    println!();
    println!("--- provenance TDI-5.1 (ancêtre gelé, inchangé) ---");
    println!(
        "évaluateur TDI-5.1 SHA-256      : {}",
        tdi52_sha256_of_repo_file("tdi-bench/src/bin/tdi-continuous-deficit-geometry-v51.rs")
    );
    println!(
        "préenregistrement TDI-5.1 SHA-256 : {}",
        tdi52_sha256_of_repo_file("docs/TDI-5.1-CONTINUOUS-DEFICIT-GEOMETRY-PREREGISTRATION.md")
    );
    println!(
        "manifeste scientifique TDI-5.1 SHA-256 : {}",
        tdi52_sha256_of_repo_file("docs/TDI-5.1-SCIENTIFIC-CODE.sha256")
    );
}

/// Section 17, item 6: all frozen scientific constants.
fn print_tdi52_frozen_constants() {
    println!();
    println!("=== CONSTANTES GELÉES (Section 17, item 6) ===");
    println!("horizon d'observation                    : {OBSERVATION_HORIZON}");
    println!("horizons cibles                          : {TARGET_HORIZONS:?}");
    println!("horizon principal                        : {PRIMARY_HORIZON}");
    println!("largeur maximale supportée                : {MAX_SUPPORTED_WIDTH}");
    println!(
        "espace des ensembles successeurs (largeur 6) : {}",
        match successor_set_space_cardinality(WIDTH_6) {
            Cardinality::Exact(value) => value.to_string(),
            other => format!("{other:?}"),
        }
    );
    println!("nombre de features baseline (B0)          : {BASELINE_FEATURE_COUNT}");
    println!("nombre de features early-overlap          : {EARLY_OVERLAP_FEATURE_COUNT}");
    println!("nombre de features contraction (δ, δ̄)     : {CONTRACTION_FEATURE_COUNT}");
    println!("nombre de features spectrales (s2, s3)    : {SPECTRAL_FEATURE_COUNT}");
    println!("nombre de dispositions de modèle          : {MODEL_LAYOUT_COUNT}");
    println!("features CK (baseline + δ + δ̄)            : {CK_FEATURE_COUNT}");
    println!("features SK (CK + s2 + s3)                : {SK_FEATURE_COUNT}");
    println!("features SKT (SK + O1 + O2)               : {SKT_FEATURE_COUNT}");
    println!("horizons focaux (U3, U6)                  : {FOCAL_HORIZONS:?}");
    println!("lambda ridge                              : {RIDGE_LAMBDA}");
    println!("réplicats bootstrap                       : {BOOTSTRAP_REPLICATES}");
    println!(
        "tailles de population — train w3={TRAIN_WIDTH_3_SYSTEMS}, holdout w3={HOLDOUT_WIDTH_3_SYSTEMS}, \
         train w4={TRAIN_WIDTH_4_SYSTEMS}, holdout w4={HOLDOUT_WIDTH_4_SYSTEMS} (aucune population OOD)"
    );
    println!(
        "multiplicateurs de tentatives — w3={WIDTH_3_ATTEMPT_MULTIPLIER}, w4={WIDTH_4_ATTEMPT_MULTIPLIER}, \
         w5={WIDTH_5_ATTEMPT_MULTIPLIER}, w6={WIDTH_6_ATTEMPT_MULTIPLIER}"
    );
    println!(
        "seuils sans-progrès — w3={WIDTH_3_NO_PROGRESS_LIMIT}, w4={WIDTH_4_NO_PROGRESS_LIMIT}, \
         w5={WIDTH_5_NO_PROGRESS_LIMIT}, w6={WIDTH_6_NO_PROGRESS_LIMIT}"
    );
}

/// Section 17: the four generator-family rules (Section 5).
fn print_tdi57_family_rules() {
    println!();
    println!("=== RÈGLES DES FAMILLES DE GÉNÉRATEURS (Section 17, Section 5) ===");
    for family in GeneratorFamily::ALL {
        println!("famille {} : {}", family.label(), family.rule_description());
    }
}

/// Section 17, item 7: every seed-block definition per family (the four
/// population seeds plus each block's own bootstrap seed), and each family's
/// separate stratified aggregate bootstrap seed from Section 10. All seeds are
/// derived deterministically from `(family, block, population)`; no block table
/// is stored (Section 8/9).
fn print_tdi52_seed_block_definitions() {
    println!();
    println!("=== BLOCS DE GRAINES (Section 17, item 7) ===");

    for family in GeneratorFamily::ALL {
        for seed_block in frozen_block_order(family) {
            let base = seed_block.population_base_seed();
            println!(
                "bloc {} | train w3={} | holdout w3={} | train w4={} | holdout w4={} | \
                 graine bootstrap=0x{:016X}",
                seed_block.label(),
                base + PopulationKind::TrainingWidth3.seed_offset(),
                base + PopulationKind::HoldoutWidth3.seed_offset(),
                base + PopulationKind::TrainingWidth4.seed_offset(),
                base + PopulationKind::HoldoutWidth4.seed_offset(),
                seed_block.bootstrap_seed()
            );
        }
        println!(
            "  graine bootstrap agrégat stratifié — famille {} (Section 10) : 0x{:016X}",
            family.label(),
            family_aggregate_bootstrap_seed(family)
        );
    }
}

/// Section 17, items 8-11 and 20: requested/accepted/rejected/attempted
/// counts, rejection counts by reason, final exclusive seeds, generation
/// budgets, and (for a successful run) the deterministic margin against
/// each population's termination limits.
fn print_tdi52_population_accounting(blocks: &[BlockPopulations]) {
    println!();
    println!(
        "=== POPULATIONS — COMPTAGES, RAISONS DE REJET, GRAINES FINALES, BUDGETS \
         (Section 17, items 8-11, 20) ==="
    );

    for block in blocks {
        for report in block.reports() {
            let spec = report.spec;
            let generation = &report.report;

            println!(
                "bloc {} | {:11} | demandé={} | accepté={} | rejeté={} | tenté={} | \
                 max_tentatives={} | seuil_sans_progrès={} | graine initiale={} | \
                 graine finale exclusive={} | raisons de rejet={}",
                block.seed_block.label(),
                spec.population.label(),
                spec.target_count,
                generation.records.len(),
                generation.excluded,
                generation.attempts,
                generation.limits.max_attempts,
                generation.limits.no_progress_limit,
                spec.seed,
                generation.next_seed,
                generation.rejections.summary()
            );
        }
    }
}

/// Section 17, items 14-15: every metric and every bootstrap interval
/// (per block and pooled aggregate) underlying one criterion's verdict.
fn print_tdi52_aggregate_comparison(label: &str, horizon: usize, comparison: &AggregateComparison) {
    println!();
    println!("=== {label} — métriques et intervalles bootstrap (Section 17, items 14-15) ===");

    for seed_block in frozen_block_order(comparison.family()) {
        let block = comparison.block(seed_block);

        tdi52_print_metrics(
            &format!(
                "  bloc {} — référence — espace U standardisé",
                seed_block.label()
            ),
            block.baseline.standardized,
        );
        tdi52_print_metrics(
            &format!(
                "  bloc {} — challenger — espace U standardisé",
                seed_block.label()
            ),
            block.challenger.standardized,
        );
        tdi52_print_metrics(
            &format!(
                "  bloc {} — référence — espace O reconstruit",
                seed_block.label()
            ),
            block.baseline.reconstructed,
        );
        tdi52_print_metrics(
            &format!(
                "  bloc {} — challenger — espace O reconstruit",
                seed_block.label()
            ),
            block.challenger.reconstructed,
        );
        tdi52_print_bootstrap_intervals(
            &format!(
                "  bloc {} — intervalles bootstrap appariés",
                seed_block.label()
            ),
            horizon,
            block.bootstrap,
        );
    }

    tdi52_print_metrics(
        "  agrégat — référence — espace U standardisé",
        comparison.aggregate_baseline_standardized,
    );
    tdi52_print_metrics(
        "  agrégat — challenger — espace U standardisé",
        comparison.aggregate_challenger_standardized,
    );
    tdi52_print_metrics(
        "  agrégat — référence — espace O reconstruit",
        comparison.aggregate_baseline_reconstructed,
    );
    tdi52_print_metrics(
        "  agrégat — challenger — espace O reconstruit",
        comparison.aggregate_challenger_reconstructed,
    );
    tdi52_print_bootstrap_intervals(
        "  agrégat — intervalles bootstrap stratifiés",
        horizon,
        comparison.aggregate_bootstrap,
    );
}

/// TDI-5.7 Section 17: every block-level and aggregate-level sub-condition of
/// each family's per-horizon SKT-vs-SK classification and the F0→F1 transfer
/// classification, plus the four criterion summaries (A replication, B
/// heterogeneity, C transfer, D descriptor drift), printed via `Debug` so the
/// output can never silently drift from the named fields it reflects.
fn print_tdi52_criteria_conditions(report: &Tdi57ExperimentReport) {
    println!();
    println!("=== CONDITIONS PAR CRITÈRE — niveau bloc et agrégat (Section 17) ===");

    // TDI-5.7A/B — per-family SKT-vs-SK at every grid horizon H = {3..8}. The
    // criteria classify at the focal U3/U6; all horizons are reported (§12/§17).
    for family_report in &report.families {
        for comparison in &family_report.grid {
            println!();
            println!(
                "TDI-5.7 (grille) — SKT vs SK — famille {} à U_{} : {:#?}",
                family_report.family.label(),
                comparison.horizon,
                comparison.result
            );
        }
    }

    // TDI-5.7C — transfer F0 → F1 SKT-vs-SK at each focal horizon.
    for comparison in &report.criterion_c.transfer_focal {
        println!();
        println!(
            "TDI-5.7C — transfert F0→F1 — SKT vs SK à U_{} : {:#?}",
            comparison.horizon, comparison.result
        );
    }

    println!();
    println!("TDI-5.7A (réplication) : {:#?}", report.criterion_a);
    println!();
    println!("TDI-5.7B (hétérogénéité) : {:#?}", report.criterion_b);
    println!();
    println!("TDI-5.7C (transfert F0→F1) : {:#?}", report.criterion_c);
    println!();
    println!(
        "TDI-5.7D (dérive des descripteurs) : {:#?}",
        report.criterion_d
    );
}

/// TDI-5.7 Section 17: the TDI-5.7A per-family focal classifications and
/// replication verdict, the TDI-5.7B across-family heterogeneity summary, the
/// TDI-5.7C F0→F1 transfer classification, and the TDI-5.7D descriptor-drift
/// table. All are preregistered classifications / descriptive summaries; none
/// is forced to a positive result and none is a pass/fail gate.
fn print_tdi52_final_verdicts(report: &Tdi57ExperimentReport) {
    println!();
    println!("=== VERDICTS FINAUX (Section 17) ===");

    // TDI-5.7A — per-family SKT-vs-SK focal classifications + replication verdict.
    for (family, classifications) in &report.criterion_a.per_family_focal {
        for (slot, &horizon) in FOCAL_HORIZONS.iter().enumerate() {
            println!(
                "TDI-5.7A — SKT vs SK — famille {} à U{horizon} : {}",
                family.label(),
                classifications[slot].label()
            );
        }
    }
    println!(
        "TDI-5.7A — réplication (bénéfique à U3 et U6 pour les 4 familles) : {}",
        if report.criterion_a.replicated {
            "oui"
        } else {
            "non"
        }
    );
    for (family, horizon) in &report.criterion_a.non_replications {
        println!(
            "TDI-5.7A — non-réplication localisée : famille {} à U{horizon}",
            family.label()
        );
    }

    // TDI-5.7 (grille) — per-family per-horizon SKT-vs-SK reductions across the
    // dense grid H = {3..8} (reported per Section 12; the confirmatory criteria
    // classify only at the focal horizons U3/U6).
    for family_report in &report.families {
        for comparison in &family_report.grid {
            println!(
                "TDI-5.7 (grille) — famille {} à U{} : réduction relative MSE = {:.6}, \
                 classification = {}",
                family_report.family.label(),
                comparison.horizon,
                comparison.aggregate_relative_reduction,
                comparison.result.classification.label()
            );
        }
    }

    // TDI-5.7B — effect-size heterogeneity across families, per focal horizon.
    for focal in &report.criterion_b.per_focal {
        println!(
            "TDI-5.7B — U{} : réduction relative min={:.6}, max={:.6}, étendue={:.6}, \
             les 4 familles dépassent 2 % = {}",
            focal.horizon,
            focal.minimum,
            focal.maximum,
            focal.range,
            if focal.all_exceed_margin {
                "oui"
            } else {
                "non"
            }
        );
    }

    // TDI-5.7C — transfer F0 → F1 classification, per focal horizon.
    for (slot, &horizon) in FOCAL_HORIZONS.iter().enumerate() {
        println!(
            "TDI-5.7C — transfert F0→F1 (SKT vs SK, U{horizon}) : {}",
            report.criterion_c.focal_classifications[slot].label()
        );
    }

    // TDI-5.7D — descriptor drift: per-family holdout means and across-family range.
    for (family, means) in &report.criterion_d.per_family_means {
        println!(
            "TDI-5.7D — famille {} : δ={:.6}, δ̄={:.6}, s2={:.6}, s3={:.6}",
            family.label(),
            means[0],
            means[1],
            means[2],
            means[3]
        );
    }
    println!(
        "TDI-5.7D — étendues inter-familles : δ={:.6}, δ̄={:.6}, s2={:.6}, s3={:.6}",
        report.criterion_d.ranges[0],
        report.criterion_d.ranges[1],
        report.criterion_d.ranges[2],
        report.criterion_d.ranges[3]
    );
}

/// Prints the complete TDI-5.7 required raw output (Section 17) for a
/// completed pipeline run. Purely a presentation layer over
/// `Tdi57ExperimentReport`: it has no scale-awareness of its own, so it is
/// exercised at tiny scale by the termination smoke path and by tests. It
/// only ever prints the real 120,000-record run's output when called from
/// `run_full_experiment`'s `--full` path, and only after that path's exact
/// confirmation-token check has passed.
fn print_tdi52_required_raw_output(report: &Tdi57ExperimentReport) {
    print_tdi52_provenance();
    print_tdi52_frozen_constants();
    print_tdi57_family_rules();
    print_tdi52_seed_block_definitions();

    // Per-family population accounting (counts, rejection reasons, seeds, budgets).
    for family_report in &report.families {
        print_tdi52_population_accounting(&family_report.blocks);
    }

    // CK/SK/SKT model coefficients and target scalers for every family and block.
    for family_report in &report.families {
        for seed_block in frozen_block_order(family_report.family) {
            let fit = family_report.aggregate_fit.block(seed_block);

            println!();
            println!(
                "### BLOC {} — normalisations et modèles (Section 17) ###",
                seed_block.label()
            );
            tdi52_print_models(&fit.models, &fit.target_scalers);
        }
    }

    // Per-family per-horizon SKT-vs-SK comparisons across the grid H = {3..8}
    // (metrics + bootstrap intervals); Sections 12, 17.
    for family_report in &report.families {
        for comparison in &family_report.grid {
            print_tdi52_aggregate_comparison(
                &format!(
                    "TDI-5.7 (grille) — SKT vs SK — famille {} à U_{}",
                    family_report.family.label(),
                    comparison.horizon
                ),
                comparison.horizon,
                &comparison.comparison,
            );
        }
    }

    // TDI-5.7C transfer comparisons: F0's fitted models evaluated on F1's holdout.
    for comparison in &report.criterion_c.transfer_focal {
        print_tdi52_aggregate_comparison(
            &format!(
                "TDI-5.7C — transfert F0→F1 — SKT vs SK à U_{}",
                comparison.horizon
            ),
            comparison.horizon,
            &comparison.comparison,
        );
    }

    print_tdi52_criteria_conditions(report);
    print_tdi52_final_verdicts(report);
}

fn run_termination_smoke() -> Result<(), String> {
    println!("=== TDI-5.7 TERMINATION SMOKE ===");

    // Inherited frozen invariant: the width-6 successor-set space is the
    // exact 2^64. TDI-5.7 generates no width-6 populations, but the
    // cardinality machinery is inherited unchanged and still checked.
    let width_6_space = successor_set_space_cardinality(WIDTH_6);

    if width_6_space != Cardinality::Exact(18_446_744_073_709_551_616_u128) {
        return Err(format!("unexpected width-6 cardinality: {width_6_space:?}"));
    }

    let limits = GenerationLimits {
        max_attempts: 64,
        no_progress_limit: 64,
    };

    let seed_reservation_count = validate_preregistered_seed_reservations()?;

    let smoke_block = frozen_block_order(GeneratorFamily::F0Base)[0];
    let report = generate_records_with_limits(
        GeneratorFamily::F0Base,
        TRAIN_WIDTH_3,
        smoke_block.population_base_seed() + PopulationKind::TrainingWidth3.seed_offset(),
        1,
        limits,
    )
    .map_err(|error| error.to_string())?;

    println!("width 6 successor-set space : 18446744073709551616");
    println!("reserved seed ranges         : {seed_reservation_count} disjoint");
    println!("bootstrap replicates         : {BOOTSTRAP_REPLICATES}");

    for family in GeneratorFamily::ALL {
        for seed_block in frozen_block_order(family) {
            println!(
                "block {} bootstrap seed      : 0x{:016X}",
                seed_block.label(),
                seed_block.bootstrap_seed()
            );
        }
        println!(
            "family {} aggregate bootstrap seed : 0x{:016X}",
            family.label(),
            family_aggregate_bootstrap_seed(family)
        );
    }
    println!(
        "width 3 smoke accepted       : {} in {} attempts",
        report.records.len(),
        report.attempts
    );
    // Every generated record now carries exact contraction descriptors.
    if let Some(first) = report.records.first() {
        println!(
            "width 3 smoke contraction    : delta={:.6}, delta_bar={:.6}",
            first.contraction[0], first.contraction[1]
        );
    }
    println!(
        "width 3 smoke rejections     : {}",
        report.rejections.summary()
    );

    let specs = population_specs();

    println!(
        "population specifications   : {} deterministic entries (4 per block, no OOD)",
        specs.len()
    );

    // Synthetic, bounded records exercising the confirmatory layouts
    // CK/SK/SKT without any real generation. Their contraction descriptors and
    // spectral moments are set by hand.
    let synthetic_training_width_3 = [
        Record {
            baseline: [0.0; BASELINE_FEATURE_COUNT],
            early_overlap: [0.20, 0.55],
            contraction: [0.50, 0.40],
            spectral: [1.80, 1.40],
            overlaps: [0.30; TARGET_HORIZON_COUNT],
            targets_u: [1.00, 1.10, 1.20, 1.30, 1.35, 1.40],
        },
        Record {
            baseline: [0.1; BASELINE_FEATURE_COUNT],
            early_overlap: [0.25, 0.60],
            contraction: [0.62, 0.31],
            spectral: [2.10, 1.60],
            overlaps: [0.32; TARGET_HORIZON_COUNT],
            targets_u: [1.50, 1.35, 1.25, 1.15, 1.10, 1.05],
        },
        Record {
            baseline: [0.15; BASELINE_FEATURE_COUNT],
            early_overlap: [0.30, 0.50],
            contraction: [0.44, 0.28],
            spectral: [1.50, 1.20],
            overlaps: [0.34; TARGET_HORIZON_COUNT],
            targets_u: [1.20, 1.25, 1.30, 1.35, 1.40, 1.45],
        },
    ];

    let synthetic_training_width_4 = [
        Record {
            baseline: [0.2; BASELINE_FEATURE_COUNT],
            early_overlap: [0.35, 0.70],
            contraction: [0.71, 0.52],
            spectral: [2.60, 2.10],
            overlaps: [0.36; TARGET_HORIZON_COUNT],
            targets_u: [2.00, 1.90, 1.80, 1.70, 1.65, 1.60],
        },
        Record {
            baseline: [0.05; BASELINE_FEATURE_COUNT],
            early_overlap: [0.40, 0.65],
            contraction: [0.58, 0.36],
            spectral: [2.30, 1.90],
            overlaps: [0.38; TARGET_HORIZON_COUNT],
            targets_u: [1.70, 1.75, 1.80, 1.85, 1.90, 1.95],
        },
    ];

    // The confirmatory layouts really do build the extra terms.
    let ck_features = feature_layout(&synthetic_training_width_3[0], FeatureLayout::Ck);
    let sk_features = feature_layout(&synthetic_training_width_3[0], FeatureLayout::Sk);
    let skt_features = feature_layout(&synthetic_training_width_3[0], FeatureLayout::Skt);
    println!(
        "layout feature widths        : CK={} (attendu {}), SK={} (attendu {}), SKT={} (attendu {})",
        ck_features.len(),
        CK_FEATURE_COUNT,
        sk_features.len(),
        SK_FEATURE_COUNT,
        skt_features.len(),
        SKT_FEATURE_COUNT
    );

    let f0_blocks = frozen_block_order(GeneratorFamily::F0Base);
    let block_fits = f0_blocks
        .map(|seed_block| {
            fit_block_models(
                seed_block,
                &synthetic_training_width_3,
                &synthetic_training_width_4,
            )
        })
        .into_iter()
        .collect::<Result<Vec<_>, _>>()?;

    let block_fits: [BlockModelFit; SEED_BLOCK_COUNT] = block_fits
        .try_into()
        .map_err(|_| "expected exactly three block fits".to_owned())?;

    let aggregate_fit =
        AggregateModelFit::assemble(block_fits).map_err(|error| error.to_string())?;

    println!(
        "identity smoke aggregate     : blocks {}, {}, {}",
        aggregate_fit.block(f0_blocks[0]).seed_block.label(),
        aggregate_fit.block(f0_blocks[1]).seed_block.label(),
        aggregate_fit.block(f0_blocks[2]).seed_block.label()
    );

    let combined_holdout =
        combine_width_3_and_4(&synthetic_training_width_3, &synthetic_training_width_4);
    let holdout_refs: [&[Record]; SEED_BLOCK_COUNT] = [
        combined_holdout.as_slice(),
        combined_holdout.as_slice(),
        combined_holdout.as_slice(),
    ];

    // Exercise the confirmatory SKT-vs-SK comparison and the four-way
    // classifier (criterion TDI-5.7A) at the primary horizon.
    let spectral_focal = evaluate_horizon_comparison(
        primary_horizon_index(),
        &aggregate_fit,
        holdout_refs,
        FeatureLayout::Sk,
        FeatureLayout::Skt,
    )?;

    println!(
        "identity smoke SKT vs SK CI  : [{:.6}, {:.6}]",
        spectral_focal
            .comparison
            .aggregate_bootstrap
            .standardized_mse
            .lower,
        spectral_focal
            .comparison
            .aggregate_bootstrap
            .standardized_mse
            .upper
    );
    println!(
        "identity smoke SKT vs SK     : classification={}",
        spectral_focal.result.classification.label()
    );

    // Exercise the four-way classifier on a CK-vs-SK comparison (the spectral
    // moments' marginal value beyond contraction) — a smoke sanity check only;
    // TDI-5.7 itself has no SK-vs-CK criterion (that was the frozen TDI-5.6B).
    let marginal_spectral_focal = evaluate_horizon_comparison(
        primary_horizon_index(),
        &aggregate_fit,
        holdout_refs,
        FeatureLayout::Ck,
        FeatureLayout::Sk,
    )?;

    println!(
        "identity smoke CK vs SK      : classification={}",
        marginal_spectral_focal.result.classification.label()
    );

    // The critical wiring smoke: the real pipeline entrypoint, run at tiny
    // scale by requesting exactly one accepted record per population.
    let tiny_population_specs = population_specs().map(|spec| PopulationSpec {
        target_count: 1,
        ..spec
    });

    let pipeline_report =
        run_tdi57_pipeline(&tiny_population_specs).map_err(|error| error.to_string())?;

    println!(
        "identity smoke pipeline      : familles={}, 5.7A répliqué={}, 5.7C[U3]={}",
        pipeline_report.families.len(),
        pipeline_report.criterion_a.replicated,
        pipeline_report.criterion_c.focal_classifications[0].label()
    );
    println!(
        "identity smoke pipeline fit  : famille {} bloc {} model count={}",
        pipeline_report.families[0].family.label(),
        f0_blocks[0].label(),
        pipeline_report.families[0]
            .aggregate_fit
            .block(f0_blocks[0])
            .models
            .models
            .len()
    );

    print_tdi52_required_raw_output(&pipeline_report);

    println!("bounded smoke result         : PASS");

    Ok(())
}

/// Name of the environment variable that must carry the exact TDI-5.7
/// full-run confirmation value. See TDI-5.7 preregistration Section 16.
const TDI57_FULL_RUN_CONFIRMATION_VAR: &str = "TDI57_CONFIRM_FULL_RUN";

/// The one accepted value for `TDI57_FULL_RUN_CONFIRMATION_VAR`. Any other
/// value, or the variable being unset, must refuse `--full`.
const TDI57_FULL_RUN_CONFIRMATION_VALUE: &str = "I_ACCEPT_THE_TDI57_FREEZE_RULE";

/// Pure decision function: takes the confirmation value as a plain
/// `Option<&str>` rather than reading the environment itself, so every
/// branch -- missing, wrong, and the one exact accepted value -- can be
/// unit tested directly without ever touching a real environment variable
/// or risking the accepted branch reaching `run_full_experiment` (and,
/// through it, the real pipeline).
fn tdi57_full_run_confirmed(value: Option<&str>) -> bool {
    value == Some(TDI57_FULL_RUN_CONFIRMATION_VALUE)
}

fn tdi57_usage_error() -> String {
    format!(
        "usage: tdi-independent-overlap-ablation-v57 --termination-smoke|--preflight|--full\n\
         a bare (no-argument) invocation does not start the experiment; the \
         real run additionally requires the exact environment variable \
         {TDI57_FULL_RUN_CONFIRMATION_VAR}={TDI57_FULL_RUN_CONFIRMATION_VALUE}"
    )
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Tdi57Mode {
    TerminationSmoke,
    Preflight,
    Full,
}

/// Pure command-line dispatch decision, independent of `main`'s I/O, so
/// that "a bare invocation can never select `--full`" is directly unit
/// testable against plain string slices rather than real process argv.
fn tdi57_parse_mode(arguments: &[String]) -> Result<Tdi57Mode, String> {
    match arguments {
        [flag] if flag == "--termination-smoke" => Ok(Tdi57Mode::TerminationSmoke),
        [flag] if flag == "--preflight" => Ok(Tdi57Mode::Preflight),
        [flag] if flag == "--full" => Ok(Tdi57Mode::Full),
        _ => Err(tdi57_usage_error()),
    }
}

fn main() -> Result<(), String> {
    let arguments = std::env::args().skip(1).collect::<Vec<_>>();

    match tdi57_parse_mode(&arguments)? {
        Tdi57Mode::TerminationSmoke => run_termination_smoke(),
        Tdi57Mode::Preflight => run_preflight(),
        Tdi57Mode::Full => run_full_experiment(),
    }
}

/// The TDI-5.7 full-run entrypoint. Checks the exact confirmation
/// environment variable *before* any generation, fitting or bootstrap;
/// only when it matches does this call the real full pipeline exactly
/// once, over the real preregistered `population_specs()`, and print the
/// complete required raw output. See TDI-5.7 preregistration Section 16.
fn run_full_experiment() -> Result<(), String> {
    let confirmation = std::env::var(TDI57_FULL_RUN_CONFIRMATION_VAR).ok();

    if !tdi57_full_run_confirmed(confirmation.as_deref()) {
        return Err(format!(
            "TDI-5.7 full execution requires the exact confirmation environment \
             variable {TDI57_FULL_RUN_CONFIRMATION_VAR}={TDI57_FULL_RUN_CONFIRMATION_VALUE}; \
             refusing before any generation, fitting or bootstrap"
        ));
    }

    let report = run_tdi57_pipeline(&population_specs())?;

    print_tdi52_required_raw_output(&report);

    Ok(())
}

/// TDI-5.7 preflight: verifies the complete frozen configuration (seed
/// reservations, population counts, bootstrap constants, pipeline wiring)
/// and prints identities and the exact real-run command, without ever
/// generating a scientific population. See TDI-5.7 preregistration
/// Section 16.
fn run_preflight() -> Result<(), String> {
    println!();
    println!("=== TDI-5.7 PREFLIGHT (aucune génération scientifique) ===");

    let reservation_count = validate_preregistered_seed_reservations()?;
    println!("réservations de graines vérifiées (disjointes)  : {reservation_count}");

    let specs = population_specs();

    if specs.len() != TOTAL_SEED_RESERVATIONS {
        return Err(format!(
            "expected {TOTAL_SEED_RESERVATIONS} population specifications, found {}",
            specs.len()
        ));
    }

    for family in GeneratorFamily::ALL {
        let mut family_total = 0_usize;

        for seed_block in frozen_block_order(family) {
            let block_total: usize = specs
                .iter()
                .filter(|spec| spec.seed_block == seed_block)
                .map(|spec| spec.target_count)
                .sum();

            if block_total != 40_000 {
                return Err(format!(
                    "block {} requests {block_total} accepted records, expected 40,000",
                    seed_block.label()
                ));
            }

            family_total += block_total;
        }

        if family_total != 120_000 {
            return Err(format!(
                "family {} requests {family_total} accepted records, expected 120,000",
                family.label()
            ));
        }
    }

    let grand_total: usize = specs.iter().map(|spec| spec.target_count).sum();

    if grand_total != 480_000 {
        return Err(format!(
            "total requested accepted records is {grand_total}, expected 480,000"
        ));
    }

    println!(
        "populations préenregistrées                     : {}",
        specs.len()
    );
    println!("enregistrements acceptés visés (total)          : {grand_total}");
    println!("réplicats de bootstrap par bloc                 : {BOOTSTRAP_REPLICATES}");
    for family in GeneratorFamily::ALL {
        print!("graines de bootstrap — famille {:<9} :", family.label());
        for seed_block in frozen_block_order(family) {
            print!(
                " {}=0x{:016X}",
                seed_block.label(),
                seed_block.bootstrap_seed()
            );
        }
        println!();
        println!(
            "graine de bootstrap agrégé stratifié — famille {:<9} : 0x{:016X}",
            family.label(),
            family_aggregate_bootstrap_seed(family)
        );
    }
    println!(
        "pipeline complet câblé à --full                 : oui (run_tdi57_pipeline, \
         subordonné à {TDI57_FULL_RUN_CONFIRMATION_VAR})"
    );

    print_tdi52_provenance();

    println!();
    println!("Commande requise pour l'exécution réelle (jamais lancée automatiquement) :");
    println!("  {TDI57_FULL_RUN_CONFIRMATION_VAR}={TDI57_FULL_RUN_CONFIRMATION_VALUE} \\");
    println!("    bash scripts/reproduce-tdi5.7.sh");

    println!();
    println!("=== PREFLIGHT TERMINÉ : aucun résultat produit ===");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        BASELINE_FEATURE_COUNT, BOOTSTRAP_REPLICATES, CK_FEATURE_COUNT, CONTRACTION_FEATURE_COUNT,
        Cardinality, CriterionCClassification, CriterionCResult, FOCAL_HORIZONS, FeatureLayout,
        GENERATOR_FAMILY_COUNT, GeneratorFamily, MODEL_LAYOUT_COUNT, PRIMARY_HORIZON, Record,
        SEED_BLOCK_COUNT, SK_FEATURE_COUNT, SKT_FEATURE_COUNT, SPECTRAL_FEATURE_COUNT,
        TARGET_HORIZONS, TDI57_FULL_RUN_CONFIRMATION_VALUE, TDI57_FULL_RUN_CONFIRMATION_VAR,
        TOTAL_SEED_RESERVATIONS,
    };
    use tdi_core::{Action, State, TableSystem};

    fn read_repo_file(relative_path: &str) -> String {
        std::fs::read_to_string(super::tdi52_repository_root().join(relative_path))
            .unwrap_or_else(|error| panic!("failed to read {relative_path}: {error}"))
    }

    fn evaluator_source() -> String {
        read_repo_file("tdi-bench/src/bin/tdi-independent-overlap-ablation-v57.rs")
    }

    fn record_with_overlap(o1: f64, o2: f64) -> Record {
        Record {
            baseline: [
                0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0, 1.1, 1.2, 1.3,
            ],
            early_overlap: [o1, o2],
            contraction: [(o1 + o2) / 2.0, o1 * o2],
            spectral: [1.0 + o1, 1.0 + o2],
            overlaps: [0.30; TARGET_HORIZONS.len()],
            targets_u: [1.0, 1.1, 1.2, 1.3, 1.35, 1.4],
        }
    }

    // --- Exact contraction descriptors (the exact-computation novelty, Section 5) ---

    #[test]
    fn dobrushin_and_mean_tv_are_exact_over_all_state_pairs() {
        // Width-2 one-step Noop kernel: state 0 -> {0}, state 1 -> {1},
        // states 2 and 3 -> uniform over all four states. Pairwise TV:
        // TV(P0,P1)=1; TV(P0/P1, P2/P3)=3/4 (four pairs); TV(P2,P3)=0. So the
        // Dobrushin coefficient delta = max = 1 and the mean pairwise TV
        // delta_bar = (1 + 4*(3/4) + 0) / 6 = 4/6 = 2/3.
        let mut system = TableSystem::new(2).expect("valid width");
        let state = |bits: u64| State::new(bits, 2).expect("valid state");
        system
            .insert(state(0), Action::Noop, vec![state(0)])
            .unwrap();
        system
            .insert(state(1), Action::Noop, vec![state(1)])
            .unwrap();
        let all = vec![state(0), state(1), state(2), state(3)];
        system.insert(state(2), Action::Noop, all.clone()).unwrap();
        system.insert(state(3), Action::Noop, all).unwrap();

        let context = super::AttemptContext::new(GeneratorFamily::F0Base, 2, 0, 0);
        let [delta, delta_bar] =
            super::contraction_descriptors(context, &system).expect("descriptors");

        assert!((delta - 1.0).abs() < 1e-12, "delta = {delta}");
        assert!(
            (delta_bar - 2.0 / 3.0).abs() < 1e-12,
            "delta_bar = {delta_bar}"
        );
    }

    #[test]
    fn identical_kernels_have_zero_contraction() {
        // Both states map to the same uniform distribution: every pairwise TV
        // is 0, so delta = delta_bar = 0.
        let mut system = TableSystem::new(1).expect("valid width");
        let state = |bits: u64| State::new(bits, 1).expect("valid state");
        let both = vec![state(0), state(1)];
        system.insert(state(0), Action::Noop, both.clone()).unwrap();
        system.insert(state(1), Action::Noop, both).unwrap();

        let context = super::AttemptContext::new(GeneratorFamily::F0Base, 1, 0, 0);
        let [delta, delta_bar] =
            super::contraction_descriptors(context, &system).expect("descriptors");

        assert_eq!(delta, 0.0);
        assert_eq!(delta_bar, 0.0);
    }

    // --- Contraction layouts (the confirmatory novelty, Section 6) ---

    #[test]
    fn ck_features_are_baseline_then_delta_and_delta_bar() {
        let mut record = record_with_overlap(0.4, 0.6);
        record.contraction = [0.7, 0.3];
        let features = super::feature_layout(&record, FeatureLayout::Ck);

        assert_eq!(features.len(), CK_FEATURE_COUNT);
        assert_eq!(features.len(), FeatureLayout::Ck.feature_count());
        assert_eq!(&features[..BASELINE_FEATURE_COUNT], &record.baseline);
        assert_eq!(features[BASELINE_FEATURE_COUNT], 0.7);
        assert_eq!(features[BASELINE_FEATURE_COUNT + 1], 0.3);
    }

    #[test]
    fn sk_features_add_contraction_then_the_two_spectral_moments() {
        let mut record = record_with_overlap(0.4, 0.6);
        record.contraction = [0.7, 0.3];
        record.spectral = [1.8, 1.4];
        let features = super::feature_layout(&record, FeatureLayout::Sk);

        assert_eq!(features.len(), SK_FEATURE_COUNT);
        assert_eq!(features.len(), FeatureLayout::Sk.feature_count());
        assert_eq!(&features[..BASELINE_FEATURE_COUNT], &record.baseline);
        let tail = &features[BASELINE_FEATURE_COUNT..];
        assert_eq!(tail, &[0.7, 0.3, 1.8, 1.4]);
    }

    #[test]
    fn skt_features_add_contraction_spectral_then_the_two_overlaps() {
        let (o1, o2) = (0.4, 0.6);
        let mut record = record_with_overlap(o1, o2);
        record.contraction = [0.7, 0.3];
        record.spectral = [1.8, 1.4];
        let features = super::feature_layout(&record, FeatureLayout::Skt);

        assert_eq!(features.len(), SKT_FEATURE_COUNT);
        assert_eq!(features.len(), FeatureLayout::Skt.feature_count());
        assert_eq!(&features[..BASELINE_FEATURE_COUNT], &record.baseline);
        let tail = &features[BASELINE_FEATURE_COUNT..];
        assert_eq!(tail, &[0.7, 0.3, 1.8, 1.4, o1, o2]);
    }

    #[test]
    fn confirmatory_layouts_never_perturb_the_baseline_block_and_nest_ck_sk_skt() {
        // The 13 baseline features are identical across B0, CK, SK and SKT:
        // only the appended descriptor/overlap block differs, so any
        // SKT-minus-SK signal is the overlaps' and any SK-minus-CK signal is
        // the spectral moments'. CK is a strict prefix of SK, and SK of SKT.
        let record = record_with_overlap(0.33, 0.77);
        let b0 = super::feature_layout(&record, FeatureLayout::B0);
        let ck = super::feature_layout(&record, FeatureLayout::Ck);
        let sk = super::feature_layout(&record, FeatureLayout::Sk);
        let skt = super::feature_layout(&record, FeatureLayout::Skt);

        assert_eq!(&ck[..BASELINE_FEATURE_COUNT], b0.as_slice());
        assert_eq!(&sk[..BASELINE_FEATURE_COUNT], b0.as_slice());
        assert_eq!(&skt[..BASELINE_FEATURE_COUNT], b0.as_slice());
        assert_eq!(&sk[..CK_FEATURE_COUNT], ck.as_slice());
        assert_eq!(&skt[..SK_FEATURE_COUNT], sk.as_slice());
    }

    #[test]
    fn feature_layout_enumeration_has_eight_entries_including_ck_sk_skt() {
        assert_eq!(FeatureLayout::ALL.len(), MODEL_LAYOUT_COUNT);
        assert_eq!(MODEL_LAYOUT_COUNT, 8);
        assert!(FeatureLayout::ALL.contains(&FeatureLayout::Ck));
        assert!(FeatureLayout::ALL.contains(&FeatureLayout::Sk));
        assert!(FeatureLayout::ALL.contains(&FeatureLayout::Skt));
        // Linear discriminants are preserved so `layout as usize` indexing is
        // unchanged from TDI-5.2/5.3/5.4/5.5.
        assert_eq!(FeatureLayout::B0 as usize, 0);
        assert_eq!(FeatureLayout::Ck as usize, 5);
        assert_eq!(FeatureLayout::Sk as usize, 6);
        assert_eq!(FeatureLayout::Skt as usize, 7);
    }

    #[test]
    fn confirmatory_layout_counts_are_fifteen_seventeen_and_nineteen() {
        assert_eq!(CONTRACTION_FEATURE_COUNT, 2);
        assert_eq!(SPECTRAL_FEATURE_COUNT, 2);
        assert_eq!(CK_FEATURE_COUNT, 15);
        assert_eq!(SK_FEATURE_COUNT, 17);
        assert_eq!(SKT_FEATURE_COUNT, 19);
    }

    // --- Exact spectral moments (the exact-computation novelty, Section 5) ---

    #[test]
    fn spectral_moments_are_exact_traces_of_kernel_powers() {
        // Width-2 one-step Noop kernel: a directed 3-cycle 0 -> 1 -> 2 -> 0
        // plus a fixed point 3 -> 3, every state deterministic (branching
        // factor 1). P^2 has exactly one self-return (the fixed point), so
        // trace(P^2) = 1; P^3 returns the whole 3-cycle to itself plus the
        // fixed point, so trace(P^3) = 4.
        let mut system = TableSystem::new(2).expect("valid width");
        let state = |bits: u64| State::new(bits, 2).expect("valid state");
        system
            .insert(state(0), Action::Noop, vec![state(1)])
            .unwrap();
        system
            .insert(state(1), Action::Noop, vec![state(2)])
            .unwrap();
        system
            .insert(state(2), Action::Noop, vec![state(0)])
            .unwrap();
        system
            .insert(state(3), Action::Noop, vec![state(3)])
            .unwrap();

        let context = super::AttemptContext::new(GeneratorFamily::F0Base, 2, 0, 0);
        let [s2, s3] = super::spectral_moments(context, &system).expect("moments");

        assert!((s2 - 1.0).abs() < 1e-12, "s2 = {s2}");
        assert!((s3 - 4.0).abs() < 1e-12, "s3 = {s3}");
    }

    #[test]
    fn spectral_moments_accumulate_unit_fractions_exactly() {
        // Width-2 kernel mixing branching factors: 0 -> {0, 1} (branch 2),
        // 1 -> {0}, 2 -> {2}, 3 -> {3}. By hand, trace(P^2) = 3/4 + 1/2 + 1 + 1
        // = 13/4 and trace(P^3) = 5/8 + 1/4 + 1 + 1 = 23/8, so the exact
        // closed-walk unit-fraction sums must reproduce 3.25 and 2.875.
        let mut system = TableSystem::new(2).expect("valid width");
        let state = |bits: u64| State::new(bits, 2).expect("valid state");
        system
            .insert(state(0), Action::Noop, vec![state(0), state(1)])
            .unwrap();
        system
            .insert(state(1), Action::Noop, vec![state(0)])
            .unwrap();
        system
            .insert(state(2), Action::Noop, vec![state(2)])
            .unwrap();
        system
            .insert(state(3), Action::Noop, vec![state(3)])
            .unwrap();

        let context = super::AttemptContext::new(GeneratorFamily::F0Base, 2, 0, 0);
        let [s2, s3] = super::spectral_moments(context, &system).expect("moments");

        assert!((s2 - 3.25).abs() < 1e-12, "s2 = {s2}");
        assert!((s3 - 2.875).abs() < 1e-12, "s3 = {s3}");
    }

    #[test]
    fn focal_horizon_indices_are_u3_and_u6() {
        let indices = super::focal_horizon_indices();
        assert_eq!(FOCAL_HORIZONS, [3, 6]);
        assert_eq!(TARGET_HORIZONS[indices[0]], 3);
        assert_eq!(TARGET_HORIZONS[indices[1]], 6);
        assert_eq!(indices, [0, 3]);
    }

    // --- Four-way classifier precedence (inherited, TDI-5.2 Section 13) ---

    fn base_result() -> CriterionCResult {
        CriterionCResult {
            classification: CriterionCClassification::Inconclusive,
            blocks_confirming_benefit: 0,
            aggregate_relative_improvement_at_least_2_percent: false,
            aggregate_bootstrap_lower_bound_positive: false,
            all_block_point_estimates_within_equivalence_margin: false,
            block_intervals_within_equivalence_margin: 0,
            aggregate_interval_within_equivalence_margin: false,
            blocks_confirming_harm: 0,
            aggregate_relative_worsening_at_least_2_percent: false,
            aggregate_bootstrap_upper_bound_negative: false,
        }
    }

    #[test]
    fn classify_returns_inconclusive_by_default() {
        assert_eq!(
            base_result().classify(),
            CriterionCClassification::Inconclusive
        );
    }

    #[test]
    fn classify_returns_beneficial_only_with_all_three_beneficial_conditions() {
        let mut result = base_result();
        result.blocks_confirming_benefit = 2;
        result.aggregate_relative_improvement_at_least_2_percent = true;
        result.aggregate_bootstrap_lower_bound_positive = true;
        assert_eq!(result.classify(), CriterionCClassification::Beneficial);

        result.blocks_confirming_benefit = 1;
        assert_ne!(result.classify(), CriterionCClassification::Beneficial);
    }

    #[test]
    fn classify_returns_equivalent_when_all_three_equivalence_conditions_hold() {
        let mut result = base_result();
        result.all_block_point_estimates_within_equivalence_margin = true;
        result.block_intervals_within_equivalence_margin = 2;
        result.aggregate_interval_within_equivalence_margin = true;
        assert_eq!(result.classify(), CriterionCClassification::Equivalent);

        result.block_intervals_within_equivalence_margin = 1;
        assert_eq!(result.classify(), CriterionCClassification::Inconclusive);
    }

    #[test]
    fn classify_beneficial_takes_precedence_over_equivalent() {
        let mut result = base_result();
        result.blocks_confirming_benefit = 2;
        result.aggregate_relative_improvement_at_least_2_percent = true;
        result.aggregate_bootstrap_lower_bound_positive = true;
        result.all_block_point_estimates_within_equivalence_margin = true;
        result.block_intervals_within_equivalence_margin = 3;
        result.aggregate_interval_within_equivalence_margin = true;
        assert_eq!(result.classify(), CriterionCClassification::Beneficial);
    }

    #[test]
    fn classify_returns_harmful_only_with_all_three_harmful_conditions() {
        let mut result = base_result();
        result.blocks_confirming_harm = 2;
        result.aggregate_relative_worsening_at_least_2_percent = true;
        result.aggregate_bootstrap_upper_bound_negative = true;
        assert_eq!(result.classify(), CriterionCClassification::Harmful);
    }

    // --- Full-run confirmation guard (Section 16) ---

    #[test]
    fn full_run_confirmation_accepts_only_the_exact_value() {
        assert!(super::tdi57_full_run_confirmed(Some(
            TDI57_FULL_RUN_CONFIRMATION_VALUE
        )));
        assert!(!super::tdi57_full_run_confirmed(None));
        assert!(!super::tdi57_full_run_confirmed(Some("")));
        assert!(!super::tdi57_full_run_confirmed(Some(
            "i_accept_the_tdi57_freeze_rule"
        )));
        // The frozen TDI-5.4 token must never unlock TDI-5.7.
        assert!(!super::tdi57_full_run_confirmed(Some(
            "I_ACCEPT_THE_TDI54_FREEZE_RULE"
        )));
    }

    #[test]
    fn parse_mode_rejects_a_bare_no_argument_invocation() {
        assert!(super::tdi57_parse_mode(&[]).is_err());
        assert!(super::tdi57_parse_mode(&["--full".to_owned(), "extra".to_owned()]).is_err());
    }

    #[test]
    fn parse_mode_selects_full_only_for_the_exact_single_flag() {
        assert_eq!(
            super::tdi57_parse_mode(&["--full".to_owned()]).unwrap(),
            super::Tdi57Mode::Full
        );
        assert_eq!(
            super::tdi57_parse_mode(&["--preflight".to_owned()]).unwrap(),
            super::Tdi57Mode::Preflight
        );
        assert_eq!(
            super::tdi57_parse_mode(&["--termination-smoke".to_owned()]).unwrap(),
            super::Tdi57Mode::TerminationSmoke
        );
        assert!(super::tdi57_parse_mode(&["--Full".to_owned()]).is_err());
    }

    #[test]
    fn usage_error_mentions_every_flag_and_the_confirmation_variable() {
        let usage = super::tdi57_usage_error();
        assert!(usage.contains("--termination-smoke"));
        assert!(usage.contains("--preflight"));
        assert!(usage.contains("--full"));
        assert!(usage.contains(TDI57_FULL_RUN_CONFIRMATION_VAR));
        assert!(usage.contains(TDI57_FULL_RUN_CONFIRMATION_VALUE));
    }

    #[test]
    fn full_run_refuses_before_any_work_without_the_confirmation_token() {
        // Never reach the accepted path in a test: assert the guard var is
        // absent first, then confirm the unconfirmed call returns an error
        // before any generation, fitting or bootstrap.
        if std::env::var(TDI57_FULL_RUN_CONFIRMATION_VAR).is_ok() {
            panic!("the confirmation variable must never be set during tests");
        }
        let error = super::run_full_experiment()
            .expect_err("run_full_experiment must refuse without the exact token");
        assert!(error.contains("refusing before any generation"));
    }

    #[test]
    fn run_full_experiment_is_wired_to_the_real_pipeline_on_the_accepted_path() {
        let source = evaluator_source();
        let start = source
            .find("fn run_full_experiment()")
            .expect("run_full_experiment must exist");
        let end = source[start..]
            .find("\nfn run_preflight()")
            .map(|offset| start + offset)
            .expect("run_preflight must follow run_full_experiment");
        let body = &source[start..end];

        assert!(
            body.contains("run_tdi57_pipeline(&population_specs())"),
            "accepted path must call the real pipeline over the real specs"
        );
        assert!(body.contains("tdi57_full_run_confirmed"));
        assert!(body.contains("print_tdi52_required_raw_output"));
    }

    #[test]
    fn termination_smoke_uses_only_bounded_specs_never_the_real_ones() {
        let source = evaluator_source();
        let start = source
            .find("fn run_termination_smoke()")
            .expect("run_termination_smoke must exist");
        let end = source[start..]
            .find("\nfn tdi57_full_run_confirmed")
            .map(|offset| start + offset)
            .expect("tdi57_full_run_confirmed must follow run_termination_smoke");
        let body = &source[start..end];

        assert!(body.contains("target_count: 1"));
        assert!(
            !body.contains("run_tdi57_pipeline(&population_specs())"),
            "the smoke path must never run the real-scale pipeline"
        );
    }

    // --- Populations and seed blocks (Sections 8, 9) ---

    #[test]
    fn population_specs_total_forty_eight_four_per_block_and_have_no_ood() {
        let specs = super::population_specs();
        assert_eq!(specs.len(), TOTAL_SEED_RESERVATIONS);
        assert_eq!(specs.len(), 48);
        assert_eq!(specs.len(), GENERATOR_FAMILY_COUNT * SEED_BLOCK_COUNT * 4);
        for family in GeneratorFamily::ALL {
            for block in super::frozen_block_order(family) {
                assert_eq!(specs.iter().filter(|s| s.seed_block == block).count(), 4);
            }
        }
        // No population is wider than width 4 (base composition, no OOD).
        assert!(specs.iter().all(|s| s.width <= 4));
    }

    #[test]
    fn each_block_forty_thousand_each_family_120000_and_total_is_480000() {
        let specs = super::population_specs();
        for family in GeneratorFamily::ALL {
            let mut family_total = 0_usize;
            for block in super::frozen_block_order(family) {
                let block_total: usize = specs
                    .iter()
                    .filter(|s| s.seed_block == block)
                    .map(|s| s.target_count)
                    .sum();
                assert_eq!(block_total, 40_000);
                family_total += block_total;
            }
            assert_eq!(family_total, 120_000);
        }
        let grand_total: usize = specs.iter().map(|s| s.target_count).sum();
        assert_eq!(grand_total, 480_000);
    }

    #[test]
    fn preregistered_seed_reservations_are_forty_eight_and_pairwise_disjoint() {
        assert_eq!(
            super::validate_preregistered_seed_reservations().unwrap(),
            48
        );
    }

    #[test]
    fn family_seed_blocks_are_derived_fresh_and_pairwise_distinct() {
        // Four families × three blocks, every population seed ≥ 1.4e9 — entirely
        // above the TDI-5.6 blocks J/K/L (1060M..1290M), the TDI-5.5 blocks
        // G/H/I (760M..990M) and every earlier block. All 48 population seeds,
        // all 12 block bootstrap seeds and all 4 family aggregate seeds are
        // distinct (Sections 8, 9).
        let mut population_seeds = Vec::new();
        let mut bootstrap_seeds = Vec::new();
        let mut aggregate_seeds = Vec::new();

        for family in GeneratorFamily::ALL {
            let order = super::frozen_block_order(family);
            assert_eq!(order.len(), SEED_BLOCK_COUNT);

            for (block_index, seed_block) in order.into_iter().enumerate() {
                assert_eq!(seed_block.family, family);
                assert_eq!(seed_block.block as usize, block_index);

                let base = seed_block.population_base_seed();
                for offset in [0_u64, 10_000_000, 20_000_000, 30_000_000] {
                    let seed = base + offset;
                    assert!(seed >= 1_400_000_000);
                    population_seeds.push(seed);
                }
                bootstrap_seeds.push(seed_block.bootstrap_seed());
            }
            aggregate_seeds.push(super::family_aggregate_bootstrap_seed(family));
        }

        // Anchored constants: the first and last derived bootstrap seeds and the
        // first family aggregate seed (base 0x5444_4935_3700_….., distinct from
        // the TDI-5.6 base 0x5444_4935_3600_…..).
        assert_eq!(GeneratorFamily::F0Base.index(), 0);
        assert_eq!(
            super::frozen_block_order(GeneratorFamily::F0Base)[0].bootstrap_seed(),
            0x5444_4935_3700_0001
        );
        assert_eq!(
            super::frozen_block_order(GeneratorFamily::F3Local)[SEED_BLOCK_COUNT - 1]
                .bootstrap_seed(),
            0x5444_4935_3700_000C
        );
        assert_eq!(
            super::family_aggregate_bootstrap_seed(GeneratorFamily::F0Base),
            0x5444_4935_3700_4700
        );

        // Every reserved seed — population, block bootstrap, aggregate bootstrap —
        // is distinct across the whole design.
        let mut all = population_seeds.clone();
        all.extend_from_slice(&bootstrap_seeds);
        all.extend_from_slice(&aggregate_seeds);
        let unique: std::collections::HashSet<u64> = all.iter().copied().collect();
        assert_eq!(
            unique.len(),
            all.len(),
            "all reserved seeds must be distinct"
        );
        assert_eq!(
            population_seeds.len(),
            GENERATOR_FAMILY_COUNT * SEED_BLOCK_COUNT * 4
        );
        assert_eq!(
            bootstrap_seeds.len(),
            GENERATOR_FAMILY_COUNT * SEED_BLOCK_COUNT
        );
        assert_eq!(aggregate_seeds.len(), GENERATOR_FAMILY_COUNT);
    }

    // --- Inherited frozen invariants (unchanged machinery) ---

    #[test]
    fn width_6_successor_space_is_exact_two_to_the_sixty_four() {
        assert_eq!(
            super::successor_set_space_cardinality(6),
            Cardinality::Exact(18_446_744_073_709_551_616_u128)
        );
    }

    #[test]
    fn primary_horizon_is_six_and_target_horizons_are_frozen() {
        assert_eq!(PRIMARY_HORIZON, 6);
        assert_eq!(TARGET_HORIZONS, [3, 4, 5, 6, 7, 8]);
        assert_eq!(TARGET_HORIZONS[super::primary_horizon_index()], 6);
    }

    #[test]
    fn splitmix_is_deterministic() {
        assert_eq!(super::splitmix64(0), super::splitmix64(0));
        assert_ne!(super::splitmix64(1), super::splitmix64(2));
    }

    #[test]
    fn bootstrap_replicate_count_is_four_thousand() {
        assert_eq!(BOOTSTRAP_REPLICATES, 4_000);
    }

    // --- Prediction and generation primitives ---

    #[test]
    fn generate_records_is_deterministic_and_carries_contraction_and_spectral_descriptors() {
        let family = GeneratorFamily::F0Base;
        let seed = super::frozen_block_order(family)[0].population_base_seed();
        let first = super::generate_records_with_limits(
            family,
            3,
            seed,
            4,
            super::preregistered_generation_limits(family, 3, seed, 4).unwrap(),
        )
        .expect("bounded width-3 generation");
        let second = super::generate_records_with_limits(
            family,
            3,
            seed,
            4,
            super::preregistered_generation_limits(family, 3, seed, 4).unwrap(),
        )
        .expect("bounded width-3 generation");
        assert_eq!(first.records.len(), 4);
        assert_eq!(first.next_seed, second.next_seed);
        assert_eq!(first.attempts, second.attempts);
        for (a, b) in first.records.iter().zip(second.records.iter()) {
            assert_eq!(a.early_overlap, b.early_overlap);
            assert_eq!(a.contraction, b.contraction);
            assert_eq!(a.spectral, b.spectral);
            assert_eq!(a.targets_u, b.targets_u);
        }
        // The contraction descriptors are finite and in [0, 1]; the spectral
        // moments are finite and in [0, 2^width] (here 2^3 = 8).
        for record in &first.records {
            for &value in &record.contraction {
                assert!(value.is_finite() && (0.0..=1.0).contains(&value));
            }
            for &value in &record.spectral {
                assert!(value.is_finite() && (0.0..=8.0).contains(&value));
            }
        }
    }

    #[test]
    fn skt_ridge_fit_and_prediction_are_deterministic_and_reconstruct_overlap() {
        let records: Vec<Record> = (0..24)
            .map(|i| {
                let o1 = 0.10 + 0.02 * f64::from(i % 7);
                let o2 = 0.20 + 0.015 * f64::from(i % 5);
                record_with_overlap(o1, o2)
            })
            .collect();

        let targets = super::overlap_values(&records, super::primary_horizon_index());
        let design = super::feature_matrix(&records, |record| {
            super::feature_layout(record, FeatureLayout::Skt)
        });

        let first = super::fit_ridge(&design, &targets).expect("ridge fit");
        let second = super::fit_ridge(&design, &targets).expect("ridge fit");
        assert_eq!(first.coefficients, second.coefficients);
        // Per-feature scalers cover all 19 SKT features; coefficients carry an
        // additional intercept at index 0.
        assert_eq!(first.means.len(), SKT_FEATURE_COUNT);
        assert_eq!(first.coefficients.len(), SKT_FEATURE_COUNT + 1);

        let predicted: Vec<f64> = design.iter().map(|row| first.predict_linear(row)).collect();
        assert_eq!(predicted.len(), records.len());
        assert!(predicted.iter().all(|value| value.is_finite()));

        let scaler = super::TargetScaler {
            mean: 0.0,
            scale: 1.0,
        };
        let prediction_set = super::tdi52_predict(
            &records,
            super::primary_horizon_index(),
            FeatureLayout::Skt,
            &first,
            scaler,
        )
        .expect("bounded prediction");
        assert_eq!(prediction_set.standardized.len(), records.len());
        assert!(
            prediction_set
                .reconstructed_overlap
                .iter()
                .all(|&overlap| (0.0..=1.0).contains(&overlap))
        );
    }

    // --- Reproduction script contract (Section 19) ---

    #[test]
    fn reproduction_script_invokes_full_and_requires_the_exact_token() {
        let script = read_repo_file("scripts/reproduce-tdi5.7.sh");

        assert!(
            script.contains("\"$BINARY_PATH\" --full 2>&1 | tee"),
            "the script must invoke the binary with --full, capturing combined output"
        );
        assert!(
            !script.contains("\"$BINARY_PATH\" 2>&1 | tee"),
            "the script must not invoke the binary without --full"
        );
        assert!(script.contains(TDI57_FULL_RUN_CONFIRMATION_VAR));
        assert!(script.contains(TDI57_FULL_RUN_CONFIRMATION_VALUE));
        assert!(script.contains("require_full_run_confirmation"));
    }

    #[test]
    fn reproduction_script_refuses_to_overwrite_an_existing_result_and_verifies_the_ancestors() {
        let script = read_repo_file("scripts/reproduce-tdi5.7.sh");

        assert!(script.contains("refuse_existing_output"));
        assert!(script.contains("already exists"));
        assert!(script.contains("refusing to overwrite"));
        // The reproduction must verify the full frozen chain before running.
        assert!(script.contains("FROZEN_TDI51_"));
        assert!(script.contains("FROZEN_TDI52_"));
        assert!(script.contains("FROZEN_TDI53_"));
        assert!(script.contains("FROZEN_TDI54_"));
        assert!(script.contains("FROZEN_TDI55_"));
        assert!(script.contains("FROZEN_TDI56_"));
    }

    // --- Frozen ancestors must never change under TDI-5.7 ---

    #[test]
    fn frozen_ancestor_hashes_are_unchanged() {
        let expected = [
            (
                "tdi-bench/src/bin/tdi-continuous-deficit-geometry-v51.rs",
                "d69d42fa31d973603eabd0ded8ffd8ca2f0a4b0b8fcec5f9de42ed8c7ce37444",
            ),
            (
                "docs/TDI-5.1-CONTINUOUS-DEFICIT-GEOMETRY-PREREGISTRATION.md",
                "25b65a07b7f248df3e043b9b7f63611c360f60f3d49a600a5612305440131852",
            ),
            (
                "tdi-bench/src/bin/tdi-independent-overlap-ablation-v52.rs",
                "2308607729659c7546a17530e69773f982d9a1cf41656ea7898e0123ca469ef7",
            ),
            (
                "docs/TDI-5.2-INDEPENDENT-OVERLAP-ABLATION-PREREGISTRATION.md",
                "f57a054bc95eb2e041434d6e2049509b0dce1a5397f9666d274b1bbac332be35",
            ),
            (
                "tdi-bench/src/bin/tdi-independent-overlap-ablation-v53.rs",
                "93181fb75d4882be2ca0b26c1babe9db747583d6e08e6992617ec12b7f65460f",
            ),
            (
                "docs/TDI-5.3-INDEPENDENT-OVERLAP-ACTIVATION-PREREGISTRATION.md",
                "7223128dcfd751ebeb6488c01c3512d0a10b35937ec170504984295eb421682e",
            ),
            (
                "tdi-bench/src/bin/tdi-independent-overlap-ablation-v54.rs",
                "dcf24d7eb1ccd938a81163738c38d31a693474c8a1d94046734bda243ca772bf",
            ),
            (
                "docs/TDI-5.4-NONLINEAR-OVERLAP-SUFFICIENCY-PREREGISTRATION.md",
                "229a0a8efa391c67c4dda1322b984109b142be3abf972d0a08f3c4ac742ec6ac",
            ),
            (
                "tdi-bench/src/bin/tdi-independent-overlap-ablation-v55.rs",
                "10df698d10f010b9f6c18e2a4d78042eb399d3812b8d69c2b4bb799de828b835",
            ),
            (
                "docs/TDI-5.5-OVERLAP-BASELINE-CHALLENGE-PREREGISTRATION.md",
                "37260b3349107659487e42e66c269ecad44efaf6131f8206bb28dfbcf83f9da1",
            ),
            (
                "tdi-bench/src/bin/tdi-independent-overlap-ablation-v56.rs",
                "0820274b3edb58a6e123c612dbed8dd8a1725221240365f142d9510404e1d1b2",
            ),
            (
                "docs/TDI-5.6-EXACT-SPECTRAL-CHALLENGE-PREREGISTRATION.md",
                "59e3375b82d0bb7aad7be0591b9d1eac074d4b194678dfe0e06e73c8aac89807",
            ),
        ];

        for (path, want) in expected {
            let got = super::tdi52_sha256_of_repo_file(path);
            assert_eq!(&got, want, "frozen ancestor changed: {path}");
        }
    }
}
