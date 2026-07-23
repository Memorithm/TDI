//! TDI-6.2 nonlinear-sufficiency evaluator — does the overlap signal survive
//! the literal spectral gap and mixing time under a *nonlinear* model?
//!
//! This file is derived from the frozen TDI-6.1 evaluator
//! (`tdi-independent-overlap-ablation-v61.rs`), itself in the frozen line
//! TDI-5.2 → TDI-6.1. TDI-5.1 … TDI-6.1 remain frozen and untouched; the full
//! chain is verified at runtime and in CI. TDI-6.2 reuses their frozen
//! generator, exact candidate analysis, exact overlap/total-variation
//! primitives, the two exact contraction descriptors (delta, delta_bar), the
//! two exact spectral moments (s2, s3), the two NON-EXACT literal spectral
//! descriptors (`g = 1 - |λ2|`, `τ_ε`) computed by the frozen v61 eigensolver +
//! mixing-time path (with its NaN-rejection self-guard), the four feature
//! layouts CK/SK/GK/GKT, the deterministic bootstrap engine and the four-way
//! Beneficial/Equivalent/Harmful/Inconclusive classifier — without altering any
//! of them.
//!
//! TDI-6.2 changes **exactly one factor** vs TDI-6.1, as its preregistration
//! (`docs/TDI-6.2-NONLINEAR-SUFFICIENCY-PREREGISTRATION.md`) declares — the
//! model family:
//!
//!   * the linear ridge is replaced by a **degree-2 interaction-expanded
//!     ridge**: each layout's feature vector `x` of length `d` is mapped to
//!     `φ(x) = [x₁ … x_d, {xᵢ·xⱼ : 1 ≤ i ≤ j ≤ d}]` (the linear terms plus all
//!     `d(d+1)/2` pairwise products, squares included, in canonical order), then
//!     fit by the SAME deterministic ridge solve (`lambda = 1.0`) on the
//!     standardized expanded design. A genuinely nonlinear model — it can
//!     represent squares and interactions of every feature, including nonlinear
//!     functions of the literal spectral gap `g`, the mixing time `τ_ε` and the
//!     exact moments — yet it introduces NO new non-exactness: `g` and `τ_ε`
//!     remain the only non-exact quantities, and reproduction stays
//!     tolerance-based exactly as in TDI-6.1;
//!   * `GKT − GK` (both degree-2) now isolates the overlaps' *and all their
//!     pairwise interactions'* nonlinear marginal value beyond a nonlinear
//!     contraction + exact-moments + literal-spectral baseline; `GK − SK`
//!     isolates the literal spectral descriptors' nonlinear marginal value;
//!   * three fresh, independent seed blocks M/N/O — renamed P/Q/R — disjoint
//!     from every prior block (population base seeds start at 4.0e9), with fresh
//!     `TDI6`/`32`-marked bootstrap seeds;
//!   * criterion TDI-6.2A (GKT vs GK, degree-2, at U3 and U6 — the primary
//!     control refuting the "linear-model artifact" objection on TDI-6.1),
//!     criterion TDI-6.2B (GK vs SK, degree-2 — the nonlinear marginal value of
//!     the literal spectral descriptors), and criterion TDI-6.2C (the GKT-vs-GK
//!     decay across the dense grid).
//!
//! Because the four-way classifier's margin is ±2% relative MSE — many orders
//! of magnitude larger than the f64 tolerances — the criterion classifications
//! are robust to the last-digit variation of the descriptors.
//!
//! The full run is gated behind an explicit, exact human confirmation
//! environment variable (see `run_full_experiment` and
//! `tdi62_full_run_confirmed`). No commit, test or CI run supplies that
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

// The two focal horizons at which TDI-5.6A/5.6B classify: U3 (near, where
// TDI-5.4B found a short-horizon benefit) and the primary U6.
const FOCAL_HORIZONS: [usize; 2] = [3, 6];
const FOCAL_HORIZON_COUNT: usize = FOCAL_HORIZONS.len();

const TRAIN_WIDTH_3: u8 = 3;
const TRAIN_WIDTH_4: u8 = 4;
// Widths 5 and 6 remain supported by the inherited frozen generator and its
// exact cardinality/budget machinery, but TDI-5.6 generates no populations
// at those widths (Section 8): there are no OOD populations.
const WIDTH_5: u8 = 5;
const WIDTH_6: u8 = 6;

const TRAIN_WIDTH_3_SYSTEMS: usize = 15_000;
const TRAIN_WIDTH_4_SYSTEMS: usize = 15_000;
const HOLDOUT_WIDTH_3_SYSTEMS: usize = 5_000;
const HOLDOUT_WIDTH_4_SYSTEMS: usize = 5_000;

const SEED_BLOCK_COUNT: usize = 3;
const POPULATIONS_PER_SEED_BLOCK: usize = 4;
const TOTAL_SEED_RESERVATIONS: usize = SEED_BLOCK_COUNT * POPULATIONS_PER_SEED_BLOCK;

const BASELINE_FEATURE_COUNT: usize = 13;
const EARLY_OVERLAP_FEATURE_COUNT: usize = 2;
// Exact contraction descriptors of the one-step Noop kernel, inherited
// unchanged from TDI-5.5 Section 5: the Dobrushin coefficient and the mean
// pairwise total variation. Both are exact rationals, computed per candidate
// system.
const CONTRACTION_FEATURE_COUNT: usize = 2;
// Exact spectral moments of the one-step Noop kernel (TDI-5.6 Section 5):
// s2 = trace(P^2) and s3 = trace(P^3), computed per candidate system as
// closed-walk sums of unit fractions with a single final rounding.
const SPECTRAL_FEATURE_COUNT: usize = 2;
// Non-exact literal spectral descriptors of the one-step Noop kernel (TDI-6.2
// Section 6): the literal spectral gap g = 1 - |λ2| and the normalized
// ε-mixing time τ_ε / T_max. These are the ONLY non-exact features; they are
// computed in f64 under the Section 12 discipline and cross-validated by the
// three independent methods of Section 7.
const LITERAL_SPECTRAL_FEATURE_COUNT: usize = 2;

// Linear layouts, inherited from TDI-5.2/5.3/5.4/5.5. In TDI-5.6 they are
// exploratory only (Section 6) and determine no confirmatory criterion.
const B0_FEATURE_COUNT: usize = BASELINE_FEATURE_COUNT;
const B1_FEATURE_COUNT: usize = BASELINE_FEATURE_COUNT + 1;
const B2_FEATURE_COUNT: usize = BASELINE_FEATURE_COUNT + 1;
const B12_FEATURE_COUNT: usize = BASELINE_FEATURE_COUNT + EARLY_OVERLAP_FEATURE_COUNT;
const BD_FEATURE_COUNT: usize = BASELINE_FEATURE_COUNT + 1;

// Confirmatory linear layouts (Section 8). CK (inherited from TDI-5.5) adds the
// two exact contraction descriptors to the baseline; SK additionally adds the
// two exact spectral moments (the inherited exact baseline); GK additionally
// adds the two NON-EXACT literal spectral descriptors (g, τ_ε); GKT
// additionally adds the two early overlaps. GK minus SK isolates the literal
// spectral descriptors' marginal value beyond the exact moments (criterion
// 6.2B); GKT minus GK isolates the overlaps' marginal value *after* the
// contraction descriptors, the exact spectral moments, AND the literal spectral
// gap + mixing time are already present (criteria 6.2A, 6.2C). CK is carried
// forward from the ancestors for continuity of reporting and drives no 6.1
// criterion.
//   CK  = baseline + δ + δ̄                                  (13 + 2  = 15)
//   SK  = baseline + δ + δ̄ + s2 + s3                        (13 + 4  = 17)
//   GK  = baseline + δ + δ̄ + s2 + s3 + g + τ                (13 + 6  = 19)
//   GKT = baseline + δ + δ̄ + s2 + s3 + g + τ + O1 + O2      (13 + 8  = 21)
const CK_FEATURE_COUNT: usize = BASELINE_FEATURE_COUNT + CONTRACTION_FEATURE_COUNT;
const SK_FEATURE_COUNT: usize =
    BASELINE_FEATURE_COUNT + CONTRACTION_FEATURE_COUNT + SPECTRAL_FEATURE_COUNT;
const GK_FEATURE_COUNT: usize = BASELINE_FEATURE_COUNT
    + CONTRACTION_FEATURE_COUNT
    + SPECTRAL_FEATURE_COUNT
    + LITERAL_SPECTRAL_FEATURE_COUNT;
const GKT_FEATURE_COUNT: usize = BASELINE_FEATURE_COUNT
    + CONTRACTION_FEATURE_COUNT
    + SPECTRAL_FEATURE_COUNT
    + LITERAL_SPECTRAL_FEATURE_COUNT
    + EARLY_OVERLAP_FEATURE_COUNT;

const MODEL_LAYOUT_COUNT: usize = 9;

const RIDGE_LAMBDA: f64 = 1.0;
const BOOTSTRAP_REPLICATES: usize = 4_000;
// Fresh stratified-aggregate bootstrap seed (TDI-6.2 Section 10), disjoint from
// every TDI-5.x bootstrap seed (TDI6 prefix, `31` = ".1" marker).
const AGGREGATE_BOOTSTRAP_SEED: u64 = 0x5444_4936_3200_4700;

const MAX_SUPPORTED_WIDTH: u8 = 6;
const WIDTH_3_ATTEMPT_MULTIPLIER: usize = 64;
const WIDTH_4_ATTEMPT_MULTIPLIER: usize = 96;
const WIDTH_5_ATTEMPT_MULTIPLIER: usize = 128;
const WIDTH_6_ATTEMPT_MULTIPLIER: usize = 256;
const WIDTH_3_NO_PROGRESS_LIMIT: usize = 25_000;
const WIDTH_4_NO_PROGRESS_LIMIT: usize = 50_000;
const WIDTH_5_NO_PROGRESS_LIMIT: usize = 75_000;
const WIDTH_6_NO_PROGRESS_LIMIT: usize = 100_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SeedBlockId {
    P,
    Q,
    R,
}

impl SeedBlockId {
    const fn label(self) -> &'static str {
        match self {
            Self::P => "P",
            Self::Q => "Q",
            Self::R => "R",
        }
    }

    fn bootstrap_seed(self) -> u64 {
        SEED_BLOCKS
            .iter()
            .find(|block| block.id == self)
            .expect("SEED_BLOCKS contains an entry for every SeedBlockId")
            .bootstrap_seed
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct SeedBlockSpec {
    id: SeedBlockId,
    training_width_3_seed: u64,
    holdout_width_3_seed: u64,
    training_width_4_seed: u64,
    holdout_width_4_seed: u64,
    bootstrap_seed: u64,
}

// Fresh seed blocks P/Q/R (TDI-6.2 Section 9), pairwise-disjoint and disjoint
// from every prior block: TDI-6.2 consumes seeds up to ≈ 3.23×10⁹, so 6.2
// starts its population base seeds at 4.0×10⁹. Population base for block index
// b is 4_000_000_000 + b·100_000_000; the four populations start at
// base + {0, 10, 20, 30}·10⁶. New bootstrap seeds carry the `TDI6` prefix with
// the `32` = ".2" marker (0x5444_4936_3200_0000 + b + 1), disjoint from every
// `TDI5`-prefixed and `31`/`TDI6.1` seed of the frozen ancestors.
const SEED_BLOCKS: [SeedBlockSpec; SEED_BLOCK_COUNT] = [
    SeedBlockSpec {
        id: SeedBlockId::P,
        training_width_3_seed: 4_000_000_000,
        holdout_width_3_seed: 4_010_000_000,
        training_width_4_seed: 4_020_000_000,
        holdout_width_4_seed: 4_030_000_000,
        bootstrap_seed: 0x5444_4936_3200_0001,
    },
    SeedBlockSpec {
        id: SeedBlockId::Q,
        training_width_3_seed: 4_100_000_000,
        holdout_width_3_seed: 4_110_000_000,
        training_width_4_seed: 4_120_000_000,
        holdout_width_4_seed: 4_130_000_000,
        bootstrap_seed: 0x5444_4936_3200_0002,
    },
    SeedBlockSpec {
        id: SeedBlockId::R,
        training_width_3_seed: 4_200_000_000,
        holdout_width_3_seed: 4_210_000_000,
        training_width_4_seed: 4_220_000_000,
        holdout_width_4_seed: 4_230_000_000,
        bootstrap_seed: 0x5444_4936_3200_0003,
    },
];

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

    const fn initial_seed(self, block: SeedBlockSpec) -> u64 {
        match self {
            Self::TrainingWidth3 => block.training_width_3_seed,
            Self::HoldoutWidth3 => block.holdout_width_3_seed,
            Self::TrainingWidth4 => block.training_width_4_seed,
            Self::HoldoutWidth4 => block.holdout_width_4_seed,
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
    const fn from_block(block: SeedBlockSpec, population: PopulationKind) -> Self {
        Self {
            seed_block: block.id,
            population,
            width: population.width(),
            seed: population.initial_seed(block),
            target_count: population.target_count(),
        }
    }
}

fn population_specs() -> [PopulationSpec; TOTAL_SEED_RESERVATIONS] {
    let mut specs = [PopulationSpec::from_block(SEED_BLOCKS[0], PopulationKind::ALL[0]);
        TOTAL_SEED_RESERVATIONS];
    let mut index = 0_usize;

    for block in SEED_BLOCKS {
        for population in PopulationKind::ALL {
            specs[index] = PopulationSpec::from_block(block, population);
            index += 1;
        }
    }

    specs
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(usize)]
enum FeatureLayout {
    // Linear layouts B0..BD are exploratory. Their discriminants (0..4) are
    // preserved so `layout as usize` indexing is unchanged from
    // TDI-5.2/5.3/5.4/5.5/5.6. The confirmatory layouts CK/SK/GK/GKT follow.
    B0,
    B1,
    B2,
    B12,
    BD,
    Ck,
    Sk,
    Gk,
    Gkt,
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
        Self::Gk,
        Self::Gkt,
    ];

    const fn label(self) -> &'static str {
        match self {
            Self::B0 => "B0 — BASELINE",
            Self::B1 => "B1 — BASELINE + O1",
            Self::B2 => "B2 — BASELINE + O2",
            Self::B12 => "B12 — BASELINE + O1 + O2",
            Self::BD => "BD — BASELINE + (O2 - O1), EXPLORATOIRE",
            Self::Ck => "CK — BASELINE + δ + δ̄ (contraction)",
            Self::Sk => "SK — BASELINE + δ + δ̄ + s2 + s3 (contraction + spectral exact)",
            Self::Gk => {
                "GK — BASELINE + δ + δ̄ + s2 + s3 + g + τ (+ literal spectral gap & mixing time)"
            }
            Self::Gkt => "GKT — BASELINE + δ + δ̄ + s2 + s3 + g + τ + O1 + O2 (full model)",
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
            Self::Gk => GK_FEATURE_COUNT,
            Self::Gkt => GKT_FEATURE_COUNT,
        }
    }
}

#[derive(Clone, Debug)]
struct Record {
    baseline: [f64; BASELINE_FEATURE_COUNT],
    early_overlap: [f64; EARLY_OVERLAP_FEATURE_COUNT],
    contraction: [f64; CONTRACTION_FEATURE_COUNT],
    spectral: [f64; SPECTRAL_FEATURE_COUNT],
    literal_spectral: [f64; LITERAL_SPECTRAL_FEATURE_COUNT],
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct AttemptContext {
    width: u8,
    seed: u64,
    attempt_index: usize,
}

impl AttemptContext {
    const fn new(width: u8, seed: u64, attempt_index: usize) -> Self {
        Self {
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
    AttemptBudgetExhausted(TerminationDiagnostic),
    NoProgress(TerminationDiagnostic),
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

fn generate_successor_masks(context: AttemptContext) -> Result<Vec<u64>, EvaluationError> {
    let states = state_count(context)?;
    let mask_count = nonempty_successor_set_count(context)?;

    let mut masks = vec![0_u64; states];
    let mut generator = context.seed;

    for mask in &mut masks {
        generator = splitmix64(generator);
        *mask = generator % mask_count + 1;
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

/// Exact contraction descriptors of the one-step Noop kernel (TDI-5.6
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

/// Exact spectral moments of the one-step Noop kernel (TDI-5.6 Section 5):
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

// === TDI-6.2 non-exact spectral descriptors (preregistration Sections 5-7,12) ===
//
// The literal spectral gap g = 1 - |λ2| and the ε-mixing time τ_ε of the
// one-step Noop kernel P. These are the ONLY non-exact quantities in the
// experiment (Section 4.2): the kernel is built exactly (rational rows) and
// converted to f64 once, then the eigenvalues and mixing time are computed in
// the declared single-threaded f64 regime (Section 12). The canonical
// eigensolver (method 1, Section 7) is a pure-Rust, unsafe-free Hessenberg
// reduction + shifted QR iteration in complex arithmetic; tests cross-validate
// it against power iteration and a reference crate within the declared
// tolerance.

/// Frozen non-exact regime constants (Section 12).
const EIGEN_CONVERGENCE_TOLERANCE: f64 = 1e-12;
const SPECTRAL_CROSS_METHOD_TOLERANCE: f64 = 1e-9;
const MIXING_EPSILON: f64 = 0.25;
const MIXING_TIME_CAP: usize = 4096;

/// Minimal complex number for the eigensolver — no external dependency, no
/// `unsafe`. Only the operations the shifted-QR iteration needs.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Complex64 {
    re: f64,
    im: f64,
}

impl Complex64 {
    fn new(re: f64, im: f64) -> Self {
        Self { re, im }
    }
    fn real(re: f64) -> Self {
        Self { re, im: 0.0 }
    }
    fn add(self, other: Self) -> Self {
        Self::new(self.re + other.re, self.im + other.im)
    }
    fn sub(self, other: Self) -> Self {
        Self::new(self.re - other.re, self.im - other.im)
    }
    fn mul(self, other: Self) -> Self {
        Self::new(
            self.re * other.re - self.im * other.im,
            self.re * other.im + self.im * other.re,
        )
    }
    fn div(self, other: Self) -> Self {
        let denominator = other.re * other.re + other.im * other.im;
        Self::new(
            (self.re * other.re + self.im * other.im) / denominator,
            (self.im * other.re - self.re * other.im) / denominator,
        )
    }
    fn conjugate(self) -> Self {
        Self::new(self.re, -self.im)
    }
    fn modulus(self) -> f64 {
        self.re.hypot(self.im)
    }
    /// Principal complex square root.
    fn sqrt(self) -> Self {
        let radius = self.modulus();
        if radius == 0.0 {
            return Self::real(0.0);
        }
        let re = ((radius + self.re) / 2.0).sqrt();
        let im = ((radius - self.re) / 2.0).sqrt();
        Self::new(re, if self.im < 0.0 { -im } else { im })
    }
}

/// Reduce a real square matrix to upper Hessenberg form in place via
/// Householder reflections (similarity transform: eigenvalues are preserved).
fn hessenberg_reduce(matrix: &mut [Vec<f64>]) {
    let n = matrix.len();
    for column in 0..n.saturating_sub(2) {
        let scale: f64 = matrix
            .iter()
            .skip(column + 1)
            .map(|row| row[column].abs())
            .sum();
        if scale == 0.0 {
            continue;
        }

        let mut norm_squared = 0.0;
        let mut reflector = vec![0.0; n];
        for (row_index, row) in matrix.iter().enumerate().skip(column + 1) {
            let scaled = row[column] / scale;
            reflector[row_index] = scaled;
            norm_squared += scaled * scaled;
        }

        let pivot = reflector[column + 1];
        let g = if pivot >= 0.0 {
            -norm_squared.sqrt()
        } else {
            norm_squared.sqrt()
        };
        norm_squared -= pivot * g;
        reflector[column + 1] = pivot - g;

        // A <- (I - v vᵀ/h) A, computed row-major: form w = vᵀA, then subtract
        // the outer product (v/h) wᵀ from every row (zero-reflector rows are
        // untouched because their factor is 0).
        let mut w = vec![0.0_f64; n];
        for (row_index, row) in matrix.iter().enumerate() {
            let v_row = reflector[row_index];
            if v_row == 0.0 {
                continue;
            }
            for (accumulator, &entry) in w.iter_mut().zip(row.iter()) {
                *accumulator += v_row * entry;
            }
        }
        for (row_index, row) in matrix.iter_mut().enumerate() {
            let factor = reflector[row_index] / norm_squared;
            if factor == 0.0 {
                continue;
            }
            for (entry, &weight) in row.iter_mut().zip(w.iter()) {
                *entry -= factor * weight;
            }
        }
        // A <- A (I - v vᵀ/h): each row i loses (row·v / h) · vᵀ.
        for row in matrix.iter_mut() {
            let projection: f64 = row
                .iter()
                .zip(reflector.iter())
                .map(|(&entry, &v)| entry * v)
                .sum();
            let factor = projection / norm_squared;
            for (entry, &v) in row.iter_mut().zip(reflector.iter()) {
                *entry -= factor * v;
            }
        }

        matrix[column + 1][column] = scale * g;
        for row in matrix.iter_mut().skip(column + 2) {
            row[column] = 0.0;
        }
    }
}

/// All eigenvalues of a real square matrix, by Hessenberg reduction followed by
/// shifted QR iteration in complex arithmetic (Wilkinson shift, with an
/// exceptional shift every 10 non-deflating iterations to break the cycling
/// that a degenerate shift causes on unit-modulus spectra). Deterministic and
/// dependency-free (Section 7, method 1).
fn eigenvalues(real_matrix: &[Vec<f64>]) -> Vec<Complex64> {
    let n = real_matrix.len();
    if n == 0 {
        return Vec::new();
    }
    let mut reduced = real_matrix.to_vec();
    hessenberg_reduce(&mut reduced);

    let mut h = vec![vec![Complex64::real(0.0); n]; n];
    for i in 0..n {
        for j in 0..n {
            h[i][j] = Complex64::real(reduced[i][j]);
        }
    }

    let mut eigenvalues = Vec::with_capacity(n);
    let mut active = n;
    let mut iterations = 0usize;
    let max_iterations = 100 * n + 1000;

    while active > 0 {
        if active == 1 {
            eigenvalues.push(h[0][0]);
            break;
        }

        // Find the split point: the largest index whose subdiagonal is negligible.
        let mut split = 0;
        let mut i = active - 1;
        while i >= 1 {
            let neighbour = h[i - 1][i - 1].modulus() + h[i][i].modulus();
            let tolerance = f64::max(1e-300, EIGEN_CONVERGENCE_TOLERANCE * neighbour);
            if h[i][i - 1].modulus() <= tolerance {
                split = i;
                break;
            }
            i -= 1;
        }

        if split == active - 1 {
            eigenvalues.push(h[active - 1][active - 1]);
            active -= 1;
            iterations = 0;
            continue;
        }
        if split == active - 2 {
            let (a11, a12, a21, a22) = (
                h[active - 2][active - 2],
                h[active - 2][active - 1],
                h[active - 1][active - 2],
                h[active - 1][active - 1],
            );
            let trace = a11.add(a22);
            let determinant = a11.mul(a22).sub(a12.mul(a21));
            let discriminant = trace
                .mul(trace)
                .sub(Complex64::real(4.0).mul(determinant))
                .sqrt();
            eigenvalues.push(trace.add(discriminant).div(Complex64::real(2.0)));
            eigenvalues.push(trace.sub(discriminant).div(Complex64::real(2.0)));
            active -= 2;
            iterations = 0;
            continue;
        }

        let corner = h[active - 1][active - 1];
        let subdiagonal = h[active - 1][active - 2].modulus();
        let shift = if iterations > 0 && iterations % 10 == 0 {
            corner.add(Complex64::real(1.5 * subdiagonal + 1e-12))
        } else {
            let (a11, a12, a21, a22) = (
                h[active - 2][active - 2],
                h[active - 2][active - 1],
                h[active - 1][active - 2],
                h[active - 1][active - 1],
            );
            let trace = a11.add(a22);
            let determinant = a11.mul(a22).sub(a12.mul(a21));
            let discriminant = trace
                .mul(trace)
                .sub(Complex64::real(4.0).mul(determinant))
                .sqrt();
            let mu1 = trace.add(discriminant).div(Complex64::real(2.0));
            let mu2 = trace.sub(discriminant).div(Complex64::real(2.0));
            if mu1.sub(corner).modulus() < mu2.sub(corner).modulus() {
                mu1
            } else {
                mu2
            }
        };

        for (d, row_vec) in h.iter_mut().enumerate().take(active).skip(split) {
            row_vec[d] = row_vec[d].sub(shift);
        }

        // QR of the active Hessenberg block via Givens rotations (unitary).
        let mut cosines = vec![Complex64::real(1.0); active];
        let mut sines = vec![Complex64::real(0.0); active];
        for k in split..(active - 1) {
            let x = h[k][k];
            let y = h[k + 1][k];
            let rho = (x.modulus() * x.modulus() + y.modulus() * y.modulus()).sqrt();
            let (cosine, sine) = if rho == 0.0 {
                (Complex64::real(1.0), Complex64::real(0.0))
            } else {
                (x.div(Complex64::real(rho)), y.div(Complex64::real(rho)))
            };
            cosines[k] = cosine;
            sines[k] = sine;
            // Rotate rows k and k+1 across columns k..active; borrow both rows
            // disjointly so the update stays row-major and index-free.
            let (upper_rows, lower_rows) = h.split_at_mut(k + 1);
            let row_upper = &mut upper_rows[k];
            let row_lower = &mut lower_rows[0];
            for (upper_cell, lower_cell) in row_upper
                .iter_mut()
                .zip(row_lower.iter_mut())
                .take(active)
                .skip(k)
            {
                let upper = *upper_cell;
                let lower = *lower_cell;
                *upper_cell = cosine
                    .conjugate()
                    .mul(upper)
                    .add(sine.conjugate().mul(lower));
                *lower_cell = Complex64::real(0.0)
                    .sub(sine)
                    .mul(upper)
                    .add(cosine.mul(lower));
            }
        }
        // R Q: rotate columns k and k+1 across the affected rows.
        for k in split..(active - 1) {
            let cosine = cosines[k];
            let sine = sines[k];
            let end = (k + 2).min(active);
            for row_vec in h[split..end].iter_mut() {
                let left = row_vec[k];
                let right = row_vec[k + 1];
                row_vec[k] = left.mul(cosine).add(right.mul(sine));
                row_vec[k + 1] = Complex64::real(0.0)
                    .sub(sine.conjugate())
                    .mul(left)
                    .add(cosine.conjugate().mul(right));
            }
        }

        for (d, row_vec) in h.iter_mut().enumerate().take(active).skip(split) {
            row_vec[d] = row_vec[d].add(shift);
        }

        iterations += 1;
        if iterations > max_iterations {
            // Non-convergence must never emit finite-but-wrong eigenvalues into
            // the frozen feature path. Signal failure with NaN so the descriptor
            // becomes non-finite and the candidate is rejected
            // (`NonFiniteFeature`) rather than silently mis-scored. This is
            // empirically unreachable (the exceptional shift handles unit-modulus
            // spectra), but it makes a silent eigensolver failure impossible
            // rather than merely improbable.
            for _ in 0..active {
                eigenvalues.push(Complex64::new(f64::NAN, f64::NAN));
            }
            break;
        }
    }

    eigenvalues
}

/// The second-largest eigenvalue modulus (SLEM) of a stochastic kernel: the
/// largest `|λ|` over all eigenvalues except one Perron eigenvalue (the one
/// closest to 1, removed once).
fn second_largest_modulus(eigenvalues: &[Complex64]) -> f64 {
    if eigenvalues.is_empty() {
        return 0.0;
    }
    let mut perron_index = 0;
    let mut best_distance = f64::INFINITY;
    for (index, value) in eigenvalues.iter().enumerate() {
        let distance = value.sub(Complex64::real(1.0)).modulus();
        if distance < best_distance {
            best_distance = distance;
            perron_index = index;
        }
    }
    let mut modulus = 0.0;
    for (index, value) in eigenvalues.iter().enumerate() {
        if index == perron_index {
            continue;
        }
        let candidate = value.modulus();
        // Propagate a non-finite eigenvalue instead of letting `f64::max` absorb
        // it: a NaN here signals eigensolver non-convergence and must reach the
        // `NonFiniteFeature` rejection, never be silently dropped.
        if candidate.is_nan() {
            return f64::NAN;
        }
        modulus = f64::max(modulus, candidate);
    }
    modulus
}

/// A stationary distribution `π` of the row-stochastic kernel `P` (`πP = π`,
/// `Σπ = 1`), computed by Cesàro-averaged power iteration. The Cesàro average
/// converges to a stationary distribution for *every* finite chain — robust to
/// periodicity and reducibility — so the mixing-time reference `π` is a
/// deterministic function of `P` regardless of its ergodic structure. The
/// operation order is fixed (Section 12); the iteration is bounded by the frozen
/// cap `T_max` and stops once the running average is stable within the frozen
/// convergence tolerance.
fn stationary_distribution(matrix: &[Vec<f64>]) -> Vec<f64> {
    let n = matrix.len();
    if n == 0 {
        return Vec::new();
    }
    let mut current = vec![1.0 / n as f64; n];
    let mut average = current.clone();
    for step in 1..=MIXING_TIME_CAP {
        let mut next = vec![0.0_f64; n];
        for i in 0..n {
            let weight = current[i];
            if weight == 0.0 {
                continue;
            }
            for j in 0..n {
                next[j] += weight * matrix[i][j];
            }
        }
        let denominator = step as f64 + 1.0;
        let mut drift = 0.0_f64;
        for j in 0..n {
            let updated = (average[j] * step as f64 + next[j]) / denominator;
            drift += (updated - average[j]).abs();
            average[j] = updated;
        }
        current = next;
        if drift <= EIGEN_CONVERGENCE_TOLERANCE {
            break;
        }
    }
    let sum: f64 = average.iter().sum();
    if sum > 0.0 {
        for value in average.iter_mut() {
            *value /= sum;
        }
    }
    average
}

/// Total variation distance `½ Σ_j |row_j − π_j|` between a kernel row and the
/// stationary distribution.
fn total_variation_to_stationary(row: &[f64], stationary: &[f64]) -> f64 {
    let mut sum = 0.0_f64;
    for (probability, target) in row.iter().zip(stationary.iter()) {
        sum += (probability - target).abs();
    }
    0.5 * sum
}

/// The ε-mixing time `τ_ε = min { t ≥ 1 : max_i ‖P^t(i, ·) − π‖_TV ≤ ε }` of the
/// kernel `P`, computed by direct iteration of `P^t` in `f64` (the mixing time
/// is an observable, not an eigenvalue, so all three cross-validation methods
/// use this same iteration — Section 7). The frozen threshold is `ε = 1/4`
/// (`MIXING_EPSILON`) and the iteration cap `T_max` (`MIXING_TIME_CAP`); if
/// convergence is not reached within `T_max` the declared deterministic
/// saturation `τ_ε = T_max` is returned (Section 6).
fn mixing_time(matrix: &[Vec<f64>], stationary: &[f64]) -> usize {
    let n = matrix.len();
    if n == 0 {
        return 0;
    }
    let mut powers = matrix.to_vec(); // P^1
    for step in 1..=MIXING_TIME_CAP {
        let mut worst = 0.0_f64;
        for row in &powers {
            worst = f64::max(worst, total_variation_to_stationary(row, stationary));
        }
        if worst <= MIXING_EPSILON {
            return step;
        }
        if step == MIXING_TIME_CAP {
            break;
        }
        let mut next = vec![vec![0.0_f64; n]; n];
        for i in 0..n {
            for k in 0..n {
                let weight = powers[i][k];
                if weight == 0.0 {
                    continue;
                }
                for j in 0..n {
                    next[i][j] += weight * matrix[k][j];
                }
            }
        }
        powers = next;
    }
    MIXING_TIME_CAP
}

/// Assemble the one-step `Noop` kernel `P` of a candidate system as a dense
/// `f64` matrix. `P[i][j] = 1/deg(i)` when state `j` is a `Noop` successor of
/// state `i`, else `0`; the rows come from the same exact
/// `uniform_branching_state_distribution(.., 1)` used by the contraction and
/// spectral-moment descriptors, so the kernel is built exactly (rational rows)
/// and converted to `f64` once (Section 4.2). States are enumerated in index
/// order `0..2^width` and every successor resolves to its enumeration column.
fn kernel_matrix(
    context: AttemptContext,
    system: &TableSystem,
) -> Result<Vec<Vec<f64>>, EvaluationError> {
    let states = state_count(context)?;

    let mut ordered = Vec::with_capacity(states);
    let mut position: std::collections::BTreeMap<State, usize> = std::collections::BTreeMap::new();
    for index in 0..states {
        let state = State::new(index as u64, context.width).map_err(|error| {
            EvaluationError::new(
                context,
                FailureCategory::Structural,
                format!("cannot create kernel state {index}: {error:?}"),
            )
        })?;
        ordered.push(state);
        position.insert(state, index);
    }

    let mut matrix = vec![vec![0.0_f64; states]; states];
    for (index, state) in ordered.iter().enumerate() {
        let row = uniform_branching_state_distribution(system, *state, Action::Noop, 1).map_err(
            |error| {
                EvaluationError::new(
                    context,
                    FailureCategory::DynamicAnalysis,
                    format!("one-step kernel distribution failed for state {index}: {error:?}"),
                )
            },
        )?;
        for (successor, probability) in &row {
            let column = *position.get(successor).ok_or_else(|| {
                EvaluationError::new(
                    context,
                    FailureCategory::Structural,
                    "kernel successor state is absent from the state enumeration".to_owned(),
                )
            })?;
            matrix[index][column] = probability.as_f64();
        }
    }

    Ok(matrix)
}

/// The Euclidean norm of a vector.
fn euclidean_norm(vector: &[f64]) -> f64 {
    vector.iter().map(|value| value * value).sum::<f64>().sqrt()
}

/// Remove the component along the right Perron eigenvector `1` (the all-ones
/// vector, since `P·1 = 1`) from `vector`, using the stationary left
/// eigenvector `π` as the deflation functional: `v ← v − ⟨π, v⟩·1`. Because
/// `⟨π, 1⟩ = 1`, the result satisfies `⟨π, v⟩ = 0`, i.e. it lies in the
/// `P`-invariant complement of the Perron direction.
fn deflate_against_perron(vector: &mut [f64], stationary: &[f64]) {
    let projection: f64 = vector
        .iter()
        .zip(stationary.iter())
        .map(|(value, weight)| value * weight)
        .sum();
    for value in vector.iter_mut() {
        *value -= projection;
    }
}

/// Cross-check A (Section 7, method 2): an independent witness of `|λ₂|` by
/// power iteration on the kernel deflated against the Perron (stationary)
/// direction. The right eigenvector for `λ = 1` is the all-ones vector; after
/// deflating each iterate against it (via `π`), the vector-norm growth ratio
/// `‖P v‖ / ‖v‖` converges to the second-largest eigenvalue modulus when the
/// second eigenvalue is real and modulus-dominant (the symmetric, permutation
/// and reversible birth–death families of the test battery). To avoid a single
/// deterministic start accidentally being orthogonal to the `λ₂` eigenvector —
/// which would let the iteration converge to a smaller eigenvalue — the witness
/// runs several diverse deterministic starts and returns the largest estimate.
/// Deterministic; fixed operation order.
fn power_iteration_second_modulus(matrix: &[Vec<f64>], stationary: &[f64]) -> f64 {
    let n = matrix.len();
    if n <= 1 {
        return 0.0;
    }
    // Diverse deterministic seeds: a ramp, an alternating sign pattern, a
    // period-3 pattern, and a one-hot-ish spike. At least one has a nonzero
    // component along the λ₂ eigenspace for the real-spectrum test kernels.
    let seeds: [fn(usize, usize) -> f64; 4] = [
        |i, _| (i as f64) + 1.0,
        |i, _| if i % 2 == 0 { 1.0 } else { -1.0 },
        |i, _| (i % 3) as f64 - 1.0,
        |i, n| if i == n / 2 { 1.0 } else { -1.0 / (n as f64) },
    ];
    let mut best = 0.0_f64;
    for seed in seeds {
        let start: Vec<f64> = (0..n).map(|i| seed(i, n)).collect();
        best = f64::max(best, power_iteration_from(matrix, stationary, start));
    }
    best
}

/// One deflated power-iteration run from a given start vector; returns the
/// converged vector-norm growth ratio (an estimate of a non-Perron eigenvalue
/// modulus), or 0 if the start collapses into the Perron direction.
fn power_iteration_from(matrix: &[Vec<f64>], stationary: &[f64], mut vector: Vec<f64>) -> f64 {
    let n = matrix.len();
    deflate_against_perron(&mut vector, stationary);
    let mut norm = euclidean_norm(&vector);
    if norm <= EIGEN_CONVERGENCE_TOLERANCE {
        return 0.0;
    }
    for value in vector.iter_mut() {
        *value /= norm;
    }
    let mut estimate = 0.0_f64;
    for _ in 0..MIXING_TIME_CAP {
        let mut next = vec![0.0_f64; n];
        for i in 0..n {
            let mut accumulator = 0.0_f64;
            for j in 0..n {
                accumulator += matrix[i][j] * vector[j];
            }
            next[i] = accumulator;
        }
        deflate_against_perron(&mut next, stationary);
        norm = euclidean_norm(&next);
        if norm <= EIGEN_CONVERGENCE_TOLERANCE {
            return 0.0;
        }
        for value in next.iter_mut() {
            *value /= norm;
        }
        if (norm - estimate).abs() <= EIGEN_CONVERGENCE_TOLERANCE {
            estimate = norm;
            break;
        }
        estimate = norm;
        vector = next;
    }
    estimate
}

/// The trace-consistency residual of the canonical eigensolver (method 1): the
/// maximum over `k ∈ {1, 2, 3}` of `|Σ_i λ_iᵏ − trace(Pᵏ)|`. The power sums of
/// the computed spectrum must equal the exact matrix-power traces; this is a
/// rigorous, self-contained correctness witness for the canonical path on any
/// kernel — including those with complex `λ₂`, where a scalar power iteration is
/// not a reliable modulus witness.
fn spectral_trace_residual(matrix: &[Vec<f64>], spectrum: &[Complex64]) -> f64 {
    let n = matrix.len();
    if n == 0 {
        return 0.0;
    }
    let mut power = matrix.to_vec(); // P^1
    let mut worst = 0.0_f64;
    for k in 1..=3 {
        let mut trace = 0.0_f64;
        for (i, row) in power.iter().enumerate() {
            trace += row[i];
        }
        let mut power_sum = Complex64::real(0.0);
        for eigenvalue in spectrum {
            let mut term = Complex64::real(1.0);
            for _ in 0..k {
                term = term.mul(*eigenvalue);
            }
            power_sum = power_sum.add(term);
        }
        worst = f64::max(worst, (power_sum.re - trace).abs() + power_sum.im.abs());
        if k < 3 {
            let mut next = vec![vec![0.0_f64; n]; n];
            for i in 0..n {
                for t in 0..n {
                    let weight = power[i][t];
                    if weight == 0.0 {
                        continue;
                    }
                    for j in 0..n {
                        next[i][j] += weight * matrix[t][j];
                    }
                }
            }
            power = next;
        }
    }
    worst
}

/// The two non-exact spectral descriptors of the one-step `Noop` kernel
/// (Section 6): the literal spectral gap `g = 1 − |λ₂|` and the normalized
/// ε-mixing time `τ_ε / T_max`. The canonical eigensolver (method 1) supplies
/// `|λ₂|` as the second-largest eigenvalue modulus of `P`; the mixing time is
/// obtained by direct `P^t` iteration to the stationary distribution. These are
/// the *only* non-exact quantities in the experiment.
fn literal_spectral_descriptors(
    context: AttemptContext,
    system: &TableSystem,
) -> Result<[f64; LITERAL_SPECTRAL_FEATURE_COUNT], EvaluationError> {
    let matrix = kernel_matrix(context, system)?;
    let spectrum = eigenvalues(&matrix);
    let slem = second_largest_modulus(&spectrum);
    let gap = 1.0 - slem;
    let stationary = stationary_distribution(&matrix);
    let tau = mixing_time(&matrix, &stationary);
    let normalized_tau = tau as f64 / MIXING_TIME_CAP as f64;
    Ok([gap, normalized_tau])
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
        // Confirmatory layouts (TDI-6.2 Section 8). Terms are the two exact
        // contraction descriptors (delta, delta_bar); for SK/GK/GKT the two
        // exact spectral moments (s2, s3); for GK/GKT the two NON-EXACT literal
        // spectral descriptors (g, τ_ε); for GKT the two early overlaps. All are
        // already stored on the record and standardized downstream in ridge
        // fitting, exactly like every other feature. The baseline block is
        // untouched, so GK minus SK isolates the literal spectral descriptors'
        // marginal value beyond the exact moments (6.2B), and GKT minus GK
        // isolates the overlaps' marginal value beyond contraction, exact
        // moments AND the literal spectral gap + mixing time (6.2A, 6.2C).
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
        FeatureLayout::Gk => {
            features.push(record.contraction[0]);
            features.push(record.contraction[1]);
            features.push(record.spectral[0]);
            features.push(record.spectral[1]);
            features.push(record.literal_spectral[0]);
            features.push(record.literal_spectral[1]);
        }
        FeatureLayout::Gkt => {
            features.push(record.contraction[0]);
            features.push(record.contraction[1]);
            features.push(record.spectral[0]);
            features.push(record.spectral[1]);
            features.push(record.literal_spectral[0]);
            features.push(record.literal_spectral[1]);
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
    let masks = generate_successor_masks(context)?;
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
    let literal_spectral = literal_spectral_descriptors(context, &system)?;

    if baseline
        .iter()
        .chain(&early_overlap)
        .chain(&contraction)
        .chain(&spectral)
        .chain(&literal_spectral)
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
        literal_spectral,
        overlaps,
        targets_u,
    })))
}
fn preregistered_generation_limits(
    width: u8,
    start_seed: u64,
    count: usize,
) -> Result<GenerationLimits, EvaluationError> {
    let context = AttemptContext::new(width, start_seed, 0);

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
                format!("width {width} is not part of the TDI-6.2 preregistered populations"),
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

        let limits = preregistered_generation_limits(spec.width, spec.seed, spec.target_count)
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
    width: u8,
    start_seed: u64,
    count: usize,
    limits: GenerationLimits,
) -> Result<GenerationReport, GenerationError> {
    generate_records_with_analyzer(width, start_seed, count, limits, analyze_seed)
}

fn seed_for_attempt(
    width: u8,
    start_seed: u64,
    attempt_index: usize,
) -> Result<u64, EvaluationError> {
    let attempt_offset = u64::try_from(attempt_index).map_err(|_| {
        EvaluationError::new(
            AttemptContext::new(width, start_seed, attempt_index),
            FailureCategory::SeedRange,
            format!("attempt index {attempt_index} cannot be represented as u64"),
        )
    })?;

    start_seed.checked_add(attempt_offset).ok_or_else(|| {
        EvaluationError::new(
            AttemptContext::new(width, start_seed, attempt_index),
            FailureCategory::SeedRange,
            format!("seed range overflow from start seed {start_seed} at attempt {attempt_index}"),
        )
    })
}

fn generate_records_with_analyzer<F>(
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
            AttemptContext::new(width, start_seed, 0),
            FailureCategory::InvalidConfiguration,
            "generation limits must be positive",
        )));
    }

    if count == 0 {
        return Err(GenerationError::Evaluation(EvaluationError::new(
            AttemptContext::new(width, start_seed, 0),
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
            let seed = seed_for_attempt(width, start_seed, attempts)
                .map_err(GenerationError::Evaluation)?;
            let diagnostic = TerminationDiagnostic::new(
                AttemptContext::new(width, seed, attempts),
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

            return Err(GenerationError::AttemptBudgetExhausted(diagnostic));
        }

        let seed =
            seed_for_attempt(width, start_seed, attempts).map_err(GenerationError::Evaluation)?;
        let context = AttemptContext::new(width, seed, attempts);

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

            return Err(GenerationError::NoProgress(diagnostic));
        }
    }

    let next_seed =
        seed_for_attempt(width, start_seed, attempts).map_err(GenerationError::Evaluation)?;

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
    generate_records_with_analyzer(spec.width, spec.seed, spec.target_count, limits, analyzer)
        .map(|report| PopulationGenerationReport { spec, report })
        .map_err(|error| PopulationGenerationError {
            spec,
            error: Box::new(error),
        })
}

fn generate_population(
    spec: PopulationSpec,
) -> Result<PopulationGenerationReport, PopulationGenerationError> {
    let limits = preregistered_generation_limits(spec.width, spec.seed, spec.target_count)
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
    /// named fields directly. TDI-6.2 has no OOD populations (Section 9).
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

/// The model design vector for a layout: the degree-2 interaction expansion of
/// the layout's feature set (TDI-6.2 Section 6). This is the ONLY substantive
/// change from TDI-6.1's linear model — a genuinely nonlinear model (it can
/// represent squares and pairwise interactions of every feature, including
/// nonlinear functions of the literal spectral gap and mixing time) fit by the
/// same deterministic ridge solve. Both the fit path (`feature_matrix`) and the
/// prediction path (`tdi52_predict`) route through this function, so training
/// and inference use the identical expansion. Deterministic, single-threaded,
/// fixed operation order (Section 11); it introduces no new non-exactness — `g`
/// and `τ_ε` remain the only non-exact quantities.
fn model_features(record: &Record, layout: FeatureLayout) -> Vec<f64> {
    degree2_expand(&feature_layout(record, layout))
}

/// Degree-2 interaction expansion (TDI-6.2 Section 6): maps a feature vector `x`
/// of length `d` to `[x₁ … x_d, {xᵢ·xⱼ : 1 ≤ i ≤ j ≤ d}]` of length
/// `d + d(d+1)/2` — the `d` linear terms followed by all pairwise products
/// (squares included) in a fixed canonical order (`i` outer, `j ≥ i` inner).
fn degree2_expand(features: &[f64]) -> Vec<f64> {
    let dimension = features.len();
    let mut expanded = Vec::with_capacity(dimension + dimension * (dimension + 1) / 2);
    expanded.extend_from_slice(features);
    for (i, &left) in features.iter().enumerate() {
        for &right in &features[i..] {
            expanded.push(left * right);
        }
    }
    expanded
}

/// The number of columns in a layout's degree-2 model design (excluding the
/// intercept): `d + d(d+1)/2` for a base feature count `d`.
const fn expanded_column_count(base_feature_count: usize) -> usize {
    base_feature_count + base_feature_count * (base_feature_count + 1) / 2
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

const FROZEN_BLOCK_ORDER: [SeedBlockId; SEED_BLOCK_COUNT] =
    [SeedBlockId::P, SeedBlockId::Q, SeedBlockId::R];

fn validate_frozen_block_order(seed_blocks: &[SeedBlockId]) -> Result<(), String> {
    if seed_blocks.len() != SEED_BLOCK_COUNT {
        return Err(format!(
            "expected {SEED_BLOCK_COUNT} seed blocks in frozen order, received {}",
            seed_blocks.len()
        ));
    }

    for (&actual, &expected) in seed_blocks.iter().zip(&FROZEN_BLOCK_ORDER) {
        if actual != expected {
            return Err(format!(
                "requires deterministic block order J, K, L; found {} where {} was expected",
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
/// reconstructed-O metrics and its prediction set. TDI-5.6 compares two
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
        let features = model_features(record, layout);
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
/// reconstructed-O metrics plus its prediction set. TDI-5.6 compares two
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

    let mut generator = DeterministicRng::new(AGGREGATE_BOOTSTRAP_SEED);

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

    for (seed_block, records) in FROZEN_BLOCK_ORDER.into_iter().zip(holdout_records) {
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
    let block_relative_reductions = FROZEN_BLOCK_ORDER.map(|seed_block| {
        let block = comparison.block(seed_block);

        tdi52_relative_reduction(
            block.baseline.standardized.mse,
            block.challenger.standardized.mse,
        )
    });

    let blocks_confirming_benefit = FROZEN_BLOCK_ORDER
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

    let blocks_confirming_harm = FROZEN_BLOCK_ORDER
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

    let block_intervals_within_equivalence_margin = FROZEN_BLOCK_ORDER
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

/// Criterion TDI-6.2A (Section 15, primary): the GKT-vs-GK four-way
/// classification at the focal horizons U3 and U6. GKT minus GK isolates the
/// overlaps' marginal value *after* the exact contraction descriptors, the
/// exact spectral moments, AND the literal spectral gap + mixing time are
/// present — the decisive control the series has pointed at since TDI-5.5.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Tdi62CriterionA {
    focal_classifications: [CriterionCClassification; FOCAL_HORIZON_COUNT],
}

/// Criterion TDI-6.2B (Section 16): the GK-vs-SK four-way classification at the
/// focal horizons U3 and U6. GK minus SK isolates the literal spectral
/// descriptors' (g, τ_ε) marginal value beyond the exact moments s2, s3 — the
/// control that makes 6.2A demanding rather than a baseline padded with inert
/// features.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Tdi62CriterionB {
    focal_classifications: [CriterionCClassification; FOCAL_HORIZON_COUNT],
}

/// Criterion TDI-6.2C decay-law / redundancy-horizon summary (Section 17).
/// Descriptive only: it reports the overlaps' marginal value beyond the full
/// descriptor set (contraction + exact moments + literal spectral gap + mixing
/// time) across the dense grid, whether that value is non-increasing in
/// horizon, the redundancy horizon h★ (the first horizon classified
/// Equivalent), and the successive ratios for inspecting a geometric shape.
#[derive(Clone, Debug, PartialEq)]
struct Tdi62CriterionC {
    horizons: Vec<usize>,
    relative_reductions: Vec<f64>,
    classifications: Vec<CriterionCClassification>,
    monotone_non_increasing: bool,
    first_equivalent_horizon: Option<usize>,
    successive_ratios: Vec<f64>,
}

/// Pure core of the TDI-6.2C decay-law summary, over bare per-horizon
/// reductions and classifications, so the monotonicity / redundancy-horizon
/// / ratio logic is unit-testable without building any `AggregateComparison`.
fn decay_law_summary(
    horizons: &[usize],
    relative_reductions: &[f64],
    classifications: &[CriterionCClassification],
) -> (bool, Option<usize>, Vec<f64>) {
    let monotone_non_increasing = relative_reductions
        .windows(2)
        .all(|pair| pair[1] <= pair[0]);

    let first_equivalent_horizon = classifications
        .iter()
        .zip(horizons)
        .find(|(classification, _)| **classification == CriterionCClassification::Equivalent)
        .map(|(_, &horizon)| horizon);

    let successive_ratios = relative_reductions
        .windows(2)
        .map(|pair| {
            if pair[0].abs() <= 1.0e-15 {
                0.0
            } else {
                pair[1] / pair[0]
            }
        })
        .collect();

    (
        monotone_non_increasing,
        first_equivalent_horizon,
        successive_ratios,
    )
}

#[derive(Clone, Debug)]
struct Tdi62ExperimentReport {
    blocks: Vec<BlockPopulations>,
    aggregate_fit: AggregateModelFit,
    // TDI-6.2A + 6.2C: GKT vs GK across the dense horizon grid U3..U8.
    spectral_grid: Vec<HorizonComparison>,
    criterion_a: Tdi62CriterionA,
    criterion_c: Tdi62CriterionC,
    // TDI-6.2B: GK vs SK at the focal horizons (marginal literal-spectral value).
    marginal_spectral_focal: Vec<HorizonComparison>,
    criterion_b: Tdi62CriterionB,
}

/// Runs the full TDI-6.2 pipeline (generation of the width-3/width-4
/// populations across seed blocks M/N/O, per-block ridge fitting on the
/// contraction- and spectral-inclusive design, aggregation, and the three
/// TDI-5.6
/// criteria) over an arbitrary set of population specifications. Callers
/// control scale entirely through `population_specs`: the preregistered
/// `population_specs()` output requests the real 120,000-record run, while
/// tests and the termination smoke path pass tiny synthetic-scale specs
/// instead. This function is called with the real specs only from
/// `run_full_experiment`'s `--full` path, and only after that path's exact
/// confirmation-token check has passed; tests and the termination smoke
/// path never reach that branch.
fn run_tdi62_pipeline(
    population_specs: &[PopulationSpec],
) -> Result<Tdi62ExperimentReport, String> {
    validate_seed_reservations(population_specs)?;

    let mut blocks = Vec::with_capacity(SEED_BLOCK_COUNT);

    for seed_block in FROZEN_BLOCK_ORDER {
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

    let block_fits: [BlockModelFit; SEED_BLOCK_COUNT] = block_fits
        .try_into()
        .map_err(|_| "expected exactly three block fits".to_owned())?;

    let aggregate_fit = AggregateModelFit::assemble(block_fits)?;

    let combined_holdouts = blocks
        .iter()
        .map(BlockPopulations::combined_holdout)
        .collect::<Vec<_>>();

    let combined_holdout_refs: [&[Record]; SEED_BLOCK_COUNT] = [
        combined_holdouts[0].as_slice(),
        combined_holdouts[1].as_slice(),
        combined_holdouts[2].as_slice(),
    ];

    // TDI-6.2A + 6.2C: GKT (challenger) vs GK (baseline) across the dense
    // horizon grid; the overlaps' marginal value beyond contraction, the exact
    // spectral moments, AND the literal spectral gap + mixing time.
    let mut spectral_grid = Vec::with_capacity(TARGET_HORIZON_COUNT);
    for horizon_index in 0..TARGET_HORIZON_COUNT {
        spectral_grid.push(evaluate_horizon_comparison(
            horizon_index,
            &aggregate_fit,
            combined_holdout_refs,
            FeatureLayout::Gk,
            FeatureLayout::Gkt,
        )?);
    }

    let focal_indices = focal_horizon_indices();

    let criterion_a = Tdi62CriterionA {
        focal_classifications: std::array::from_fn(|slot| {
            spectral_grid[focal_indices[slot]].result.classification
        }),
    };

    let horizons = spectral_grid
        .iter()
        .map(|entry| entry.horizon)
        .collect::<Vec<_>>();
    let relative_reductions = spectral_grid
        .iter()
        .map(|entry| entry.aggregate_relative_reduction)
        .collect::<Vec<_>>();
    let classifications = spectral_grid
        .iter()
        .map(|entry| entry.result.classification)
        .collect::<Vec<_>>();

    let (monotone_non_increasing, first_equivalent_horizon, successive_ratios) =
        decay_law_summary(&horizons, &relative_reductions, &classifications);

    let criterion_c = Tdi62CriterionC {
        horizons,
        relative_reductions,
        classifications,
        monotone_non_increasing,
        first_equivalent_horizon,
        successive_ratios,
    };

    // TDI-6.2B: GK (challenger) vs SK (baseline) at the focal horizons U3 and
    // U6; the literal spectral descriptors' marginal value beyond the exact
    // moments s2, s3 (the control that makes 6.2A demanding).
    let mut marginal_spectral_focal = Vec::with_capacity(FOCAL_HORIZON_COUNT);
    for &horizon_index in &focal_indices {
        marginal_spectral_focal.push(evaluate_horizon_comparison(
            horizon_index,
            &aggregate_fit,
            combined_holdout_refs,
            FeatureLayout::Sk,
            FeatureLayout::Gk,
        )?);
    }

    let criterion_b = Tdi62CriterionB {
        focal_classifications: std::array::from_fn(|slot| {
            marginal_spectral_focal[slot].result.classification
        }),
    };

    Ok(Tdi62ExperimentReport {
        blocks,
        aggregate_fit,
        spectral_grid,
        criterion_a,
        criterion_c,
        marginal_spectral_focal,
        criterion_b,
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

/// Provenance and integrity (TDI-6.2 preregistration Section 17): git commit,
/// compiler/Cargo versions, and the SHA-256 of the v62 evaluator, the TDI-6.2
/// preregistration and the TDI-6.2 scientific manifest — plus the full frozen
/// ancestor chain TDI-6.1 → TDI-5.1 (every ancestor evaluator, preregistration
/// and scientific manifest), read live and printed for provenance (Section 1).
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
        "évaluateur TDI-6.2 SHA-256      : {}",
        tdi52_sha256_of_repo_file("tdi-bench/src/bin/tdi-independent-overlap-ablation-v62.rs")
    );
    println!(
        "préenregistrement TDI-6.2 SHA-256 : {}",
        tdi52_sha256_of_repo_file("docs/TDI-6.2-NONLINEAR-SUFFICIENCY-PREREGISTRATION.md")
    );
    println!(
        "manifeste scientifique TDI-6.2 SHA-256 : {}",
        tdi52_sha256_of_repo_file("docs/TDI-6.2-SCIENTIFIC-CODE.sha256")
    );
    println!();
    println!("--- provenance TDI-6.1 (ancêtre gelé, inchangé) ---");
    println!(
        "évaluateur TDI-6.1 SHA-256      : {}",
        tdi52_sha256_of_repo_file("tdi-bench/src/bin/tdi-independent-overlap-ablation-v61.rs")
    );
    println!(
        "préenregistrement TDI-6.1 SHA-256 : {}",
        tdi52_sha256_of_repo_file("docs/TDI-6.1-SPECTRAL-GAP-MIXING-TIME-PREREGISTRATION.md")
    );
    println!(
        "manifeste scientifique TDI-6.1 SHA-256 : {}",
        tdi52_sha256_of_repo_file("docs/TDI-6.1-SCIENTIFIC-CODE.sha256")
    );
    println!();
    println!("--- provenance TDI-5.7 (ancêtre gelé, inchangé) ---");
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

/// Section 19, item 6: all frozen scientific constants.
fn print_tdi52_frozen_constants() {
    println!();
    println!("=== CONSTANTES GELÉES (Section 19) ===");
    println!("--- régime non-exact TDI-6.2 (Section 12) ---");
    println!(
        "régime FP                                : IEEE-754 binary64, mono-thread, ordre d'opérations fixe (pas de FMA/parallèle)"
    );
    println!("tolérance de convergence eigensolveur η  : {EIGEN_CONVERGENCE_TOLERANCE:.1e}");
    println!("tolérance d'accord inter-méthodes        : {SPECTRAL_CROSS_METHOD_TOLERANCE:.1e}");
    println!("seuil de mixing ε                        : {MIXING_EPSILON}");
    println!("plafond d'itération T_max                : {MIXING_TIME_CAP}");
    println!(
        "descripteurs non-exacts (les seuls)      : g = 1 - |λ2| ; τ_ε / T_max du noyau Noop à un pas"
    );
    println!("--- constantes exactes inchangées ---");
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
    println!("nombre de features spectrales exactes (s2, s3) : {SPECTRAL_FEATURE_COUNT}");
    println!("nombre de features spectrales littérales (g, τ): {LITERAL_SPECTRAL_FEATURE_COUNT}");
    println!("nombre de dispositions de modèle          : {MODEL_LAYOUT_COUNT}");
    println!(
        "famille de modèles (TDI-6.2 Section 6)    : ridge à expansion d'interactions degré-2, φ(x) = [x_i, x_i·x_j (i≤j)], solve ridge déterministe (λ={RIDGE_LAMBDA})"
    );
    println!(
        "features CK — base {CK_FEATURE_COUNT}, colonnes degré-2 {} (+ intercept)",
        expanded_column_count(CK_FEATURE_COUNT)
    );
    println!(
        "features SK — base {SK_FEATURE_COUNT}, colonnes degré-2 {} (+ intercept)",
        expanded_column_count(SK_FEATURE_COUNT)
    );
    println!(
        "features GK — base {GK_FEATURE_COUNT}, colonnes degré-2 {} (+ intercept)",
        expanded_column_count(GK_FEATURE_COUNT)
    );
    println!(
        "features GKT — base {GKT_FEATURE_COUNT}, colonnes degré-2 {} (+ intercept)",
        expanded_column_count(GKT_FEATURE_COUNT)
    );
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

/// Section 17, item 7: every seed-block definition (all seeds plus each
/// block's own bootstrap seed), and the separate stratified aggregate
/// bootstrap seed from Section 10.
fn print_tdi52_seed_block_definitions() {
    println!();
    println!("=== BLOCS DE GRAINES (Section 17, item 7) ===");

    for block in SEED_BLOCKS {
        println!(
            "bloc {} | train w3={} | holdout w3={} | train w4={} | holdout w4={} | \
             graine bootstrap=0x{:016X}",
            block.id.label(),
            block.training_width_3_seed,
            block.holdout_width_3_seed,
            block.training_width_4_seed,
            block.holdout_width_4_seed,
            block.bootstrap_seed
        );
    }

    println!("graine bootstrap agrégat stratifié (Section 10) : 0x{AGGREGATE_BOOTSTRAP_SEED:016X}");
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

    for seed_block in FROZEN_BLOCK_ORDER {
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

/// TDI-6.2 Section 19: every block-level and aggregate-level sub-condition of
/// the per-horizon GKT-vs-GK and focal GK-vs-SK classifications and the three
/// criterion summaries, printed via `Debug` so the output can never silently
/// drift from the named fields it reflects.
fn print_tdi52_criteria_conditions(report: &Tdi62ExperimentReport) {
    println!();
    println!("=== CONDITIONS PAR CRITÈRE — niveau bloc et agrégat (Section 19) ===");
    for entry in &report.spectral_grid {
        println!();
        println!(
            "TDI-6.2A/C — GKT vs GK à U_{} : {:#?}",
            entry.horizon, entry.result
        );
    }
    println!();
    println!(
        "TDI-6.2B — valeur marginale spectrale littérale : référence {} ; challenger {}",
        FeatureLayout::Sk.label(),
        FeatureLayout::Gk.label()
    );
    for entry in &report.marginal_spectral_focal {
        println!();
        println!(
            "TDI-6.2B — GK vs SK à U_{} : {:#?}",
            entry.horizon, entry.result
        );
    }
    println!();
    println!("TDI-6.2A (focal) : {:#?}", report.criterion_a);
    println!();
    println!("TDI-6.2B (focal) : {:#?}", report.criterion_b);
    println!();
    println!("TDI-6.2C (loi de décroissance) : {:#?}", report.criterion_c);
}

/// TDI-6.2 Section 19: the TDI-6.2A and TDI-6.2B focal classifications and the
/// TDI-6.2C decay-law / redundancy-horizon summary. All three are
/// preregistered classifications / descriptive summaries; 6.2A is the primary,
/// forced to no result; none is a pass/fail gate.
fn print_tdi52_final_verdicts(report: &Tdi62ExperimentReport) {
    println!();
    println!("=== VERDICTS FINAUX (Section 19) ===");

    for (slot, &horizon) in FOCAL_HORIZONS.iter().enumerate() {
        println!(
            "TDI-6.2A — O1/O2 au-delà de la contraction, du spectre exact ET du gap spectral littéral + mixing (GKT vs GK, U{horizon}) : {}",
            report.criterion_a.focal_classifications[slot].label()
        );
    }

    for (slot, &horizon) in FOCAL_HORIZONS.iter().enumerate() {
        println!(
            "TDI-6.2B — descripteurs spectraux littéraux au-delà des moments exacts (GK vs SK, U{horizon}) : {}",
            report.criterion_b.focal_classifications[slot].label()
        );
    }

    for index in 0..report.criterion_c.horizons.len() {
        println!(
            "TDI-6.2C — U{} : réduction relative MSE = {:.6}, classification = {}",
            report.criterion_c.horizons[index],
            report.criterion_c.relative_reductions[index],
            report.criterion_c.classifications[index].label()
        );
    }
    println!(
        "TDI-6.2C — décroissance monotone (non croissante) : {}",
        if report.criterion_c.monotone_non_increasing {
            "oui"
        } else {
            "non"
        }
    );
    println!(
        "TDI-6.2C — horizon de redondance h★ (première équivalence) : {}",
        match report.criterion_c.first_equivalent_horizon {
            Some(horizon) => format!("U{horizon}"),
            None => "aucun".to_owned(),
        }
    );
    println!(
        "TDI-6.2C — ratios successifs r_(h+1)/r_h : {:?}",
        report.criterion_c.successive_ratios
    );
}

/// Prints the complete TDI-6.2 required raw output (Section 19) for a
/// completed pipeline run. Purely a presentation layer over
/// `Tdi62ExperimentReport`: it has no scale-awareness of its own, so it is
/// exercised at tiny scale by the termination smoke path and by tests. It
/// only ever prints the real 120,000-record run's output when called from
/// `run_full_experiment`'s `--full` path, and only after that path's exact
/// confirmation-token check has passed.
fn print_tdi52_required_raw_output(report: &Tdi62ExperimentReport) {
    print_tdi52_provenance();
    print_tdi52_frozen_constants();
    print_tdi52_seed_block_definitions();
    print_tdi52_population_accounting(&report.blocks);

    for seed_block in FROZEN_BLOCK_ORDER {
        let fit = report.aggregate_fit.block(seed_block);

        println!();
        println!(
            "### BLOC {} — normalisations et modèles (Section 17) ###",
            seed_block.label()
        );
        tdi52_print_models(&fit.models, &fit.target_scalers);
    }

    for entry in &report.spectral_grid {
        print_tdi52_aggregate_comparison(
            &format!("TDI-6.2A/C — GKT vs GK à U_{}", entry.horizon),
            entry.horizon,
            &entry.comparison,
        );
    }

    for entry in &report.marginal_spectral_focal {
        print_tdi52_aggregate_comparison(
            &format!("TDI-6.2B — GK vs SK à U_{}", entry.horizon),
            entry.horizon,
            &entry.comparison,
        );
    }

    print_spectral_cross_validation_table();
    print_tdi52_criteria_conditions(report);
    print_tdi52_final_verdicts(report);
}

/// Section 19: the three-method spectral cross-validation table. For a bounded,
/// deterministic sample of real candidate kernels drawn from block M's
/// generator seeds, print the second-largest eigenvalue modulus `|λ₂|`, the
/// literal gap `g = 1 − |λ₂|` and the normalized mixing time `τ_ε / T_max` from
/// method 1 (the canonical Hessenberg + shifted-QR eigensolver, the frozen
/// feature path) and from method 2 (power iteration deflated against the Perron
/// direction), together with their disagreement and the canonical path's
/// rigorous trace-consistency residual `max_k |Σλᵢᵏ − trace(Pᵏ)|`. Method 3 —
/// the reference eigensolver dev-dependency (Section 7) — cross-checks the same
/// quantities in the bounded test suite over the closed-form known-spectra
/// battery; where offline vendoring is unavailable that suite falls back to the
/// methods-1↔2 agreement plus the known-spectra battery (Section 4.3), which
/// alone establish the canonical path's correctness. Cross-method agreement
/// within `SPECTRAL_CROSS_METHOD_TOLERANCE` is the correctness guarantee that
/// replaces bit-exact reproduction for these two non-exact descriptors.
fn print_spectral_cross_validation_table() {
    println!();
    println!("=== TABLE DE VALIDATION CROISÉE SPECTRALE (Section 19) ===");
    println!(
        "tolérance d'accord inter-méthodes η_x         : {SPECTRAL_CROSS_METHOD_TOLERANCE:.1e}"
    );
    println!(
        "méthode 1 = QR décalé canonique (chemin gelé) ; méthode 2 = itération de puissance déflatée ; \
         méthode 3 = crate de référence (dev-dependency, vérifiée dans la suite de tests / repli batterie à spectre connu)"
    );
    println!(
        "{:<7} {:>10} {:>12} {:>12} {:>12} {:>12} {:>12}",
        "largeur", "graine", "|λ2| m1", "|λ2| m2", "|m1-m2|", "g m1", "résidu-trace"
    );

    let mut worst_disagreement = 0.0_f64;
    let mut worst_residual = 0.0_f64;

    for (width, seed, matrix) in spectral_cross_validation_samples() {
        let spectrum = eigenvalues(&matrix);
        let slem_method1 = second_largest_modulus(&spectrum);
        let stationary = stationary_distribution(&matrix);
        let slem_method2 = power_iteration_second_modulus(&matrix, &stationary);
        let disagreement = (slem_method1 - slem_method2).abs();
        let residual = spectral_trace_residual(&matrix, &spectrum);
        let gap = 1.0 - slem_method1;
        let tau = mixing_time(&matrix, &stationary);
        let normalized_tau = tau as f64 / MIXING_TIME_CAP as f64;
        worst_disagreement = f64::max(worst_disagreement, disagreement);
        worst_residual = f64::max(worst_residual, residual);
        println!(
            "{width:<7} {seed:>10} {slem_method1:>12.9} {slem_method2:>12.9} \
             {disagreement:>12.2e} {gap:>12.9} {residual:>12.2e}  (τ/T_max={normalized_tau:.6})"
        );
    }

    println!(
        "résidu de trace max méthode 1 (Σλᵏ=trace(Pᵏ)) : {worst_residual:.2e}  [témoin rigoureux — niveau machine]"
    );
    println!(
        "désaccord max méthodes 1↔2 (diagnostic)       : {worst_disagreement:.2e}  [attendu élevé si λ2 complexe]"
    );
    println!(
        "NOTE : le résidu de trace (≈ niveau machine) est le témoin de correction du chemin gelé (méthode 1) ; \
         l'itération de puissance déflatée (méthode 2) n'est un témoin fiable de |λ2| que pour les noyaux à spectre \
         réel (symétriques / naissance-mort réversibles), pour lesquels l'accord 1↔2 est vérifié à {SPECTRAL_CROSS_METHOD_TOLERANCE:.0e} \
         par la batterie à spectre connu de la suite de tests (méthode 3 = crate de référence en dev-dependency, avec repli batterie). \
         Sur les candidats réels non symétriques, |m1-m2| élevé reflète un λ2 complexe et NON une erreur de la méthode 1."
    );
}

/// A bounded, deterministic sample of real candidate kernels for the spectral
/// cross-validation table: consecutive generator seeds from block M's
/// width-3 and width-4 training populations, each built through the same
/// `generate_successor_masks` → `build_system` → `kernel_matrix` path the
/// experiment uses. Kernels that fail to build are skipped; the scan is bounded
/// so the diagnostic never dominates the run.
fn spectral_cross_validation_samples() -> Vec<(u8, u64, Vec<Vec<f64>>)> {
    const SAMPLES_PER_WIDTH: usize = 6;
    const MAX_SCAN_PER_WIDTH: u64 = 256;

    let mut samples = Vec::new();
    let widths_and_bases = [
        (TRAIN_WIDTH_3, SEED_BLOCKS[0].training_width_3_seed),
        (TRAIN_WIDTH_4, SEED_BLOCKS[0].training_width_4_seed),
    ];

    for (width, base) in widths_and_bases {
        let mut collected = 0;
        let mut offset = 0_u64;
        while collected < SAMPLES_PER_WIDTH && offset < MAX_SCAN_PER_WIDTH {
            let seed = base + offset;
            offset += 1;
            let context = AttemptContext::new(width, seed, 0);
            let Ok(masks) = generate_successor_masks(context) else {
                continue;
            };
            let Ok(system) = build_system(context, &masks) else {
                continue;
            };
            let Ok(matrix) = kernel_matrix(context, &system) else {
                continue;
            };
            samples.push((width, seed, matrix));
            collected += 1;
        }
    }

    samples
}

fn run_termination_smoke() -> Result<(), String> {
    println!("=== TDI-5.6 TERMINATION SMOKE ===");

    // Inherited frozen invariant: the width-6 successor-set space is the
    // exact 2^64. TDI-5.6 generates no width-6 populations, but the
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

    let report = generate_records_with_limits(
        TRAIN_WIDTH_3,
        SEED_BLOCKS[0].training_width_3_seed,
        1,
        limits,
    )
    .map_err(|error| error.to_string())?;

    println!("width 6 successor-set space : 18446744073709551616");
    println!("reserved seed ranges         : {seed_reservation_count} disjoint");
    println!("bootstrap replicates         : {BOOTSTRAP_REPLICATES}");

    for block in SEED_BLOCKS {
        println!(
            "block {} bootstrap seed      : 0x{:016X}",
            block.id.label(),
            block.bootstrap_seed
        );
    }

    println!("aggregate bootstrap seed     : 0x{AGGREGATE_BOOTSTRAP_SEED:016X}");
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
    // CK/SK/GK/GKT without any real generation. Their contraction descriptors,
    // exact spectral moments and non-exact literal spectral descriptors (g, τ)
    // are set by hand.
    let synthetic_training_width_3 = [
        Record {
            baseline: [0.0; BASELINE_FEATURE_COUNT],
            early_overlap: [0.20, 0.55],
            contraction: [0.50, 0.40],
            spectral: [1.80, 1.40],
            literal_spectral: [0.35, 0.10],
            overlaps: [0.30; TARGET_HORIZON_COUNT],
            targets_u: [1.00, 1.10, 1.20, 1.30, 1.35, 1.40],
        },
        Record {
            baseline: [0.1; BASELINE_FEATURE_COUNT],
            early_overlap: [0.25, 0.60],
            contraction: [0.62, 0.31],
            spectral: [2.10, 1.60],
            literal_spectral: [0.28, 0.15],
            overlaps: [0.32; TARGET_HORIZON_COUNT],
            targets_u: [1.50, 1.35, 1.25, 1.15, 1.10, 1.05],
        },
        Record {
            baseline: [0.15; BASELINE_FEATURE_COUNT],
            early_overlap: [0.30, 0.50],
            contraction: [0.44, 0.28],
            spectral: [1.50, 1.20],
            literal_spectral: [0.42, 0.08],
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
            literal_spectral: [0.22, 0.20],
            overlaps: [0.36; TARGET_HORIZON_COUNT],
            targets_u: [2.00, 1.90, 1.80, 1.70, 1.65, 1.60],
        },
        Record {
            baseline: [0.05; BASELINE_FEATURE_COUNT],
            early_overlap: [0.40, 0.65],
            contraction: [0.58, 0.36],
            spectral: [2.30, 1.90],
            literal_spectral: [0.31, 0.12],
            overlaps: [0.38; TARGET_HORIZON_COUNT],
            targets_u: [1.70, 1.75, 1.80, 1.85, 1.90, 1.95],
        },
    ];

    // The confirmatory layouts really do build the extra terms.
    let ck_features = feature_layout(&synthetic_training_width_3[0], FeatureLayout::Ck);
    let sk_features = feature_layout(&synthetic_training_width_3[0], FeatureLayout::Sk);
    let gk_features = feature_layout(&synthetic_training_width_3[0], FeatureLayout::Gk);
    let gkt_features = feature_layout(&synthetic_training_width_3[0], FeatureLayout::Gkt);
    println!(
        "layout feature widths        : CK={} (attendu {}), SK={} (attendu {}), \
         GK={} (attendu {}), GKT={} (attendu {})",
        ck_features.len(),
        CK_FEATURE_COUNT,
        sk_features.len(),
        SK_FEATURE_COUNT,
        gk_features.len(),
        GK_FEATURE_COUNT,
        gkt_features.len(),
        GKT_FEATURE_COUNT
    );

    let block_fits = FROZEN_BLOCK_ORDER
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
        aggregate_fit.block(SeedBlockId::P).seed_block.label(),
        aggregate_fit.block(SeedBlockId::Q).seed_block.label(),
        aggregate_fit.block(SeedBlockId::R).seed_block.label()
    );

    let combined_holdout =
        combine_width_3_and_4(&synthetic_training_width_3, &synthetic_training_width_4);
    let holdout_refs: [&[Record]; SEED_BLOCK_COUNT] = [
        combined_holdout.as_slice(),
        combined_holdout.as_slice(),
        combined_holdout.as_slice(),
    ];

    // Exercise the confirmatory GKT-vs-GK comparison and the four-way
    // classifier (criterion TDI-6.2A, primary) at the primary horizon.
    let spectral_focal = evaluate_horizon_comparison(
        primary_horizon_index(),
        &aggregate_fit,
        holdout_refs,
        FeatureLayout::Gk,
        FeatureLayout::Gkt,
    )?;

    println!(
        "identity smoke GKT vs GK CI  : [{:.6}, {:.6}]",
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
        "identity smoke 6.2A          : classification={}",
        spectral_focal.result.classification.label()
    );

    // Exercise criterion TDI-6.2B (GK vs SK, the literal spectral descriptors'
    // marginal value beyond the exact moments) at the primary horizon.
    let marginal_spectral_focal = evaluate_horizon_comparison(
        primary_horizon_index(),
        &aggregate_fit,
        holdout_refs,
        FeatureLayout::Sk,
        FeatureLayout::Gk,
    )?;

    println!(
        "identity smoke 6.2B          : classification={}",
        marginal_spectral_focal.result.classification.label()
    );

    // The critical wiring smoke: the real pipeline entrypoint, run at tiny
    // scale by requesting exactly one accepted record per population.
    let tiny_population_specs = population_specs().map(|spec| PopulationSpec {
        target_count: 1,
        ..spec
    });

    let pipeline_report =
        run_tdi62_pipeline(&tiny_population_specs).map_err(|error| error.to_string())?;

    println!(
        "identity smoke pipeline      : grille={}, 6.2A[U3]={}, h★={:?}",
        pipeline_report.spectral_grid.len(),
        pipeline_report.criterion_a.focal_classifications[0].label(),
        pipeline_report.criterion_c.first_equivalent_horizon
    );
    println!(
        "identity smoke pipeline fit  : block M model count={}",
        pipeline_report
            .aggregate_fit
            .block(SeedBlockId::P)
            .models
            .models
            .len()
    );

    print_tdi52_required_raw_output(&pipeline_report);

    println!("bounded smoke result         : PASS");

    Ok(())
}

/// Name of the environment variable that must carry the exact TDI-6.2
/// full-run confirmation value. See TDI-6.2 preregistration Section 18.
const TDI62_FULL_RUN_CONFIRMATION_VAR: &str = "TDI62_CONFIRM_FULL_RUN";

/// The one accepted value for `TDI62_FULL_RUN_CONFIRMATION_VAR`. Any other
/// value, or the variable being unset, must refuse `--full`.
const TDI62_FULL_RUN_CONFIRMATION_VALUE: &str = "I_ACCEPT_THE_TDI62_FREEZE_RULE";

/// Pure decision function: takes the confirmation value as a plain
/// `Option<&str>` rather than reading the environment itself, so every
/// branch -- missing, wrong, and the one exact accepted value -- can be
/// unit tested directly without ever touching a real environment variable
/// or risking the accepted branch reaching `run_full_experiment` (and,
/// through it, the real pipeline).
fn tdi62_full_run_confirmed(value: Option<&str>) -> bool {
    value == Some(TDI62_FULL_RUN_CONFIRMATION_VALUE)
}

fn tdi62_usage_error() -> String {
    format!(
        "usage: tdi-independent-overlap-ablation-v56 --termination-smoke|--preflight|--full\n\
         a bare (no-argument) invocation does not start the experiment; the \
         real run additionally requires the exact environment variable \
         {TDI62_FULL_RUN_CONFIRMATION_VAR}={TDI62_FULL_RUN_CONFIRMATION_VALUE}"
    )
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Tdi62Mode {
    TerminationSmoke,
    Preflight,
    Full,
}

/// Pure command-line dispatch decision, independent of `main`'s I/O, so
/// that "a bare invocation can never select `--full`" is directly unit
/// testable against plain string slices rather than real process argv.
fn tdi62_parse_mode(arguments: &[String]) -> Result<Tdi62Mode, String> {
    match arguments {
        [flag] if flag == "--termination-smoke" => Ok(Tdi62Mode::TerminationSmoke),
        [flag] if flag == "--preflight" => Ok(Tdi62Mode::Preflight),
        [flag] if flag == "--full" => Ok(Tdi62Mode::Full),
        _ => Err(tdi62_usage_error()),
    }
}

fn main() -> Result<(), String> {
    let arguments = std::env::args().skip(1).collect::<Vec<_>>();

    match tdi62_parse_mode(&arguments)? {
        Tdi62Mode::TerminationSmoke => run_termination_smoke(),
        Tdi62Mode::Preflight => run_preflight(),
        Tdi62Mode::Full => run_full_experiment(),
    }
}

/// The TDI-5.6 full-run entrypoint. Checks the exact confirmation
/// environment variable *before* any generation, fitting or bootstrap;
/// only when it matches does this call the real full pipeline exactly
/// once, over the real preregistered `population_specs()`, and print the
/// complete required raw output. See TDI-5.6 preregistration Section 16.
fn run_full_experiment() -> Result<(), String> {
    let confirmation = std::env::var(TDI62_FULL_RUN_CONFIRMATION_VAR).ok();

    if !tdi62_full_run_confirmed(confirmation.as_deref()) {
        return Err(format!(
            "TDI-5.6 full execution requires the exact confirmation environment \
             variable {TDI62_FULL_RUN_CONFIRMATION_VAR}={TDI62_FULL_RUN_CONFIRMATION_VALUE}; \
             refusing before any generation, fitting or bootstrap"
        ));
    }

    let report = run_tdi62_pipeline(&population_specs())?;

    print_tdi52_required_raw_output(&report);

    Ok(())
}

/// TDI-5.6 preflight: verifies the complete frozen configuration (seed
/// reservations, population counts, bootstrap constants, pipeline wiring)
/// and prints identities and the exact real-run command, without ever
/// generating a scientific population. See TDI-5.6 preregistration
/// Section 16.
fn run_preflight() -> Result<(), String> {
    println!();
    println!("=== TDI-5.6 PREFLIGHT (aucune génération scientifique) ===");

    let reservation_count = validate_preregistered_seed_reservations()?;
    println!("réservations de graines vérifiées (disjointes)  : {reservation_count}");

    let specs = population_specs();

    if specs.len() != TOTAL_SEED_RESERVATIONS {
        return Err(format!(
            "expected {TOTAL_SEED_RESERVATIONS} population specifications, found {}",
            specs.len()
        ));
    }

    for seed_block in FROZEN_BLOCK_ORDER {
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
    }

    let grand_total: usize = specs.iter().map(|spec| spec.target_count).sum();

    if grand_total != 120_000 {
        return Err(format!(
            "total requested accepted records is {grand_total}, expected 120,000"
        ));
    }

    println!(
        "populations préenregistrées                     : {}",
        specs.len()
    );
    println!("enregistrements acceptés visés (total)          : {grand_total}");
    println!("réplicats de bootstrap par bloc                 : {BOOTSTRAP_REPLICATES}");
    println!(
        "graines de bootstrap par bloc                   : {}=0x{:016X} {}=0x{:016X} {}=0x{:016X}",
        SeedBlockId::P.label(),
        SeedBlockId::P.bootstrap_seed(),
        SeedBlockId::Q.label(),
        SeedBlockId::Q.bootstrap_seed(),
        SeedBlockId::R.label(),
        SeedBlockId::R.bootstrap_seed()
    );
    println!("graine de bootstrap agrégé stratifié            : 0x{AGGREGATE_BOOTSTRAP_SEED:016X}");
    println!(
        "pipeline complet câblé à --full                 : oui (run_tdi62_pipeline, \
         subordonné à {TDI62_FULL_RUN_CONFIRMATION_VAR})"
    );

    print_tdi52_provenance();

    println!();
    println!("Commande requise pour l'exécution réelle (jamais lancée automatiquement) :");
    println!("  {TDI62_FULL_RUN_CONFIRMATION_VAR}={TDI62_FULL_RUN_CONFIRMATION_VALUE} \\");
    println!("    bash scripts/reproduce-tdi5.6.sh");

    println!();
    println!("=== PREFLIGHT TERMINÉ : aucun résultat produit ===");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        AGGREGATE_BOOTSTRAP_SEED, BASELINE_FEATURE_COUNT, BOOTSTRAP_REPLICATES, CK_FEATURE_COUNT,
        CONTRACTION_FEATURE_COUNT, Cardinality, Complex64, CriterionCClassification,
        CriterionCResult, FOCAL_HORIZONS, FeatureLayout, GK_FEATURE_COUNT, GKT_FEATURE_COUNT,
        LITERAL_SPECTRAL_FEATURE_COUNT, MIXING_EPSILON, MIXING_TIME_CAP, MODEL_LAYOUT_COUNT,
        PRIMARY_HORIZON, Record, SEED_BLOCKS, SK_FEATURE_COUNT, SPECTRAL_CROSS_METHOD_TOLERANCE,
        SPECTRAL_FEATURE_COUNT, SeedBlockId, TARGET_HORIZONS, TDI62_FULL_RUN_CONFIRMATION_VALUE,
        TDI62_FULL_RUN_CONFIRMATION_VAR, TOTAL_SEED_RESERVATIONS,
    };
    use tdi_core::{Action, State, TableSystem};

    fn read_repo_file(relative_path: &str) -> String {
        std::fs::read_to_string(super::tdi52_repository_root().join(relative_path))
            .unwrap_or_else(|error| panic!("failed to read {relative_path}: {error}"))
    }

    fn evaluator_source() -> String {
        read_repo_file("tdi-bench/src/bin/tdi-independent-overlap-ablation-v62.rs")
    }

    fn record_with_overlap(o1: f64, o2: f64) -> Record {
        Record {
            baseline: [
                0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0, 1.1, 1.2, 1.3,
            ],
            early_overlap: [o1, o2],
            contraction: [(o1 + o2) / 2.0, o1 * o2],
            spectral: [1.0 + o1, 1.0 + o2],
            literal_spectral: [1.0 - o2, o1 * 0.5],
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

        let context = super::AttemptContext::new(2, 0, 0);
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

        let context = super::AttemptContext::new(1, 0, 0);
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
    fn gk_features_add_contraction_spectral_then_the_two_literal_spectral() {
        let mut record = record_with_overlap(0.4, 0.6);
        record.contraction = [0.7, 0.3];
        record.spectral = [1.8, 1.4];
        record.literal_spectral = [0.55, 0.11];
        let features = super::feature_layout(&record, FeatureLayout::Gk);

        assert_eq!(features.len(), GK_FEATURE_COUNT);
        assert_eq!(features.len(), FeatureLayout::Gk.feature_count());
        assert_eq!(&features[..BASELINE_FEATURE_COUNT], &record.baseline);
        let tail = &features[BASELINE_FEATURE_COUNT..];
        assert_eq!(tail, &[0.7, 0.3, 1.8, 1.4, 0.55, 0.11]);
    }

    #[test]
    fn gkt_features_add_contraction_spectral_literal_then_the_two_overlaps() {
        let (o1, o2) = (0.4, 0.6);
        let mut record = record_with_overlap(o1, o2);
        record.contraction = [0.7, 0.3];
        record.spectral = [1.8, 1.4];
        record.literal_spectral = [0.55, 0.11];
        let features = super::feature_layout(&record, FeatureLayout::Gkt);

        assert_eq!(features.len(), GKT_FEATURE_COUNT);
        assert_eq!(features.len(), FeatureLayout::Gkt.feature_count());
        assert_eq!(&features[..BASELINE_FEATURE_COUNT], &record.baseline);
        let tail = &features[BASELINE_FEATURE_COUNT..];
        assert_eq!(tail, &[0.7, 0.3, 1.8, 1.4, 0.55, 0.11, o1, o2]);
    }

    #[test]
    fn confirmatory_layouts_never_perturb_the_baseline_block_and_nest_ck_sk_gk_gkt() {
        // The 13 baseline features are identical across B0, CK, SK, GK and GKT:
        // only the appended descriptor/overlap block differs, so any
        // GKT-minus-GK signal is the overlaps', any GK-minus-SK signal is the
        // literal spectral descriptors', and any SK-minus-CK signal is the exact
        // moments'. CK ⊂ SK ⊂ GK ⊂ GKT as strict prefixes.
        let record = record_with_overlap(0.33, 0.77);
        let b0 = super::feature_layout(&record, FeatureLayout::B0);
        let ck = super::feature_layout(&record, FeatureLayout::Ck);
        let sk = super::feature_layout(&record, FeatureLayout::Sk);
        let gk = super::feature_layout(&record, FeatureLayout::Gk);
        let gkt = super::feature_layout(&record, FeatureLayout::Gkt);

        assert_eq!(&ck[..BASELINE_FEATURE_COUNT], b0.as_slice());
        assert_eq!(&sk[..BASELINE_FEATURE_COUNT], b0.as_slice());
        assert_eq!(&gk[..BASELINE_FEATURE_COUNT], b0.as_slice());
        assert_eq!(&gkt[..BASELINE_FEATURE_COUNT], b0.as_slice());
        assert_eq!(&sk[..CK_FEATURE_COUNT], ck.as_slice());
        assert_eq!(&gk[..SK_FEATURE_COUNT], sk.as_slice());
        assert_eq!(&gkt[..GK_FEATURE_COUNT], gk.as_slice());
    }

    #[test]
    fn feature_layout_enumeration_has_nine_entries_including_ck_sk_gk_gkt() {
        assert_eq!(FeatureLayout::ALL.len(), MODEL_LAYOUT_COUNT);
        assert_eq!(MODEL_LAYOUT_COUNT, 9);
        assert!(FeatureLayout::ALL.contains(&FeatureLayout::Ck));
        assert!(FeatureLayout::ALL.contains(&FeatureLayout::Sk));
        assert!(FeatureLayout::ALL.contains(&FeatureLayout::Gk));
        assert!(FeatureLayout::ALL.contains(&FeatureLayout::Gkt));
        // Linear discriminants are preserved so `layout as usize` indexing is
        // unchanged from TDI-5.2/5.3/5.4/5.5/5.6.
        assert_eq!(FeatureLayout::B0 as usize, 0);
        assert_eq!(FeatureLayout::Ck as usize, 5);
        assert_eq!(FeatureLayout::Sk as usize, 6);
        assert_eq!(FeatureLayout::Gk as usize, 7);
        assert_eq!(FeatureLayout::Gkt as usize, 8);
    }

    #[test]
    fn confirmatory_layout_counts_are_fifteen_seventeen_nineteen_and_twentyone() {
        assert_eq!(CONTRACTION_FEATURE_COUNT, 2);
        assert_eq!(SPECTRAL_FEATURE_COUNT, 2);
        assert_eq!(LITERAL_SPECTRAL_FEATURE_COUNT, 2);
        assert_eq!(CK_FEATURE_COUNT, 15);
        assert_eq!(SK_FEATURE_COUNT, 17);
        assert_eq!(GK_FEATURE_COUNT, 19);
        assert_eq!(GKT_FEATURE_COUNT, 21);
    }

    // --- The degree-2 interaction model (the TDI-6.2 novelty, Section 6) ---

    #[test]
    fn degree2_expand_is_linear_terms_then_canonical_pairwise_products() {
        // For x = [a, b, c] the expansion is [a, b, c, a², ab, ac, b², bc, c²]:
        // the 3 linear terms then all 6 pairwise products (i ≤ j) in canonical
        // order (i outer, j ≥ i inner). Length 3 + 3·4/2 = 9.
        let (a, b, c) = (2.0_f64, 3.0, 5.0);
        let expanded = super::degree2_expand(&[a, b, c]);
        assert_eq!(
            expanded,
            vec![a, b, c, a * a, a * b, a * c, b * b, b * c, c * c]
        );
        assert_eq!(expanded.len(), 3 + 3 * 4 / 2);
    }

    #[test]
    fn expanded_column_counts_match_the_layout_dimensions() {
        // d + d(d+1)/2 for each confirmatory layout.
        assert_eq!(super::expanded_column_count(CK_FEATURE_COUNT), 135);
        assert_eq!(super::expanded_column_count(SK_FEATURE_COUNT), 170);
        assert_eq!(super::expanded_column_count(GK_FEATURE_COUNT), 209);
        assert_eq!(super::expanded_column_count(GKT_FEATURE_COUNT), 252);
    }

    #[test]
    fn model_features_is_the_degree2_expansion_of_the_layout() {
        // The fit/predict design is exactly the degree-2 expansion of the raw
        // layout feature vector, for every confirmatory layout.
        let record = record_with_overlap(0.4, 0.6);
        for (layout, base) in [
            (FeatureLayout::Ck, CK_FEATURE_COUNT),
            (FeatureLayout::Sk, SK_FEATURE_COUNT),
            (FeatureLayout::Gk, GK_FEATURE_COUNT),
            (FeatureLayout::Gkt, GKT_FEATURE_COUNT),
        ] {
            let raw = super::feature_layout(&record, layout);
            let design = super::model_features(&record, layout);
            assert_eq!(design, super::degree2_expand(&raw));
            assert_eq!(design.len(), super::expanded_column_count(base));
            // The linear block is a prefix; the products follow.
            assert_eq!(&design[..base], raw.as_slice());
        }
    }

    #[test]
    fn gkt_minus_gk_expansion_adds_the_overlaps_and_all_their_interactions() {
        // The nonlinear marginal value of the overlaps (criteria 6.2A/6.2C) is
        // exactly the extra columns of the GKT degree-2 design over the GK one:
        // the two overlap linear terms plus every pairwise product involving an
        // overlap (O₁², O₂², O₁·O₂, and each overlap crossed with the 19 base
        // features). Raw layouts differ by the two overlaps; expanded designs
        // differ by 2 + (231 − 190) = 43 columns.
        let record = record_with_overlap(0.33, 0.77);
        let gk = super::model_features(&record, FeatureLayout::Gk);
        let gkt = super::model_features(&record, FeatureLayout::Gkt);
        assert_eq!(
            super::feature_layout(&record, FeatureLayout::Gkt).len()
                - super::feature_layout(&record, FeatureLayout::Gk).len(),
            2
        );
        assert_eq!(gkt.len() - gk.len(), 43);
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

        let context = super::AttemptContext::new(2, 0, 0);
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

        let context = super::AttemptContext::new(2, 0, 0);
        let [s2, s3] = super::spectral_moments(context, &system).expect("moments");

        assert!((s2 - 3.25).abs() < 1e-12, "s2 = {s2}");
        assert!((s3 - 2.875).abs() < 1e-12, "s3 = {s3}");
    }

    // --- Decay-law summary (Section 15) ---

    #[test]
    fn decay_law_flags_monotone_decrease_and_first_equivalent_horizon() {
        let horizons = [3, 4, 5, 6, 7, 8];
        let reductions = [0.05, 0.03, 0.01, 0.005, 0.002, 0.001];
        let classifications = [
            CriterionCClassification::Beneficial,
            CriterionCClassification::Inconclusive,
            CriterionCClassification::Equivalent,
            CriterionCClassification::Equivalent,
            CriterionCClassification::Equivalent,
            CriterionCClassification::Equivalent,
        ];

        let (monotone, first_equivalent, ratios) =
            super::decay_law_summary(&horizons, &reductions, &classifications);

        assert!(monotone);
        assert_eq!(first_equivalent, Some(5));
        assert_eq!(ratios.len(), 5);
        assert!((ratios[0] - 0.6).abs() < 1e-12); // 0.03 / 0.05
    }

    #[test]
    fn decay_law_detects_a_non_monotone_profile_and_no_equivalence() {
        let horizons = [3, 4, 5];
        let reductions = [0.01, 0.02, 0.015];
        let classifications = [
            CriterionCClassification::Beneficial,
            CriterionCClassification::Beneficial,
            CriterionCClassification::Beneficial,
        ];

        let (monotone, first_equivalent, _ratios) =
            super::decay_law_summary(&horizons, &reductions, &classifications);

        assert!(!monotone);
        assert_eq!(first_equivalent, None);
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
        assert!(super::tdi62_full_run_confirmed(Some(
            TDI62_FULL_RUN_CONFIRMATION_VALUE
        )));
        assert!(!super::tdi62_full_run_confirmed(None));
        assert!(!super::tdi62_full_run_confirmed(Some("")));
        assert!(!super::tdi62_full_run_confirmed(Some(
            "i_accept_the_tdi62_freeze_rule"
        )));
        // The frozen TDI-5.4 token must never unlock TDI-5.6.
        assert!(!super::tdi62_full_run_confirmed(Some(
            "I_ACCEPT_THE_TDI54_FREEZE_RULE"
        )));
    }

    #[test]
    fn parse_mode_rejects_a_bare_no_argument_invocation() {
        assert!(super::tdi62_parse_mode(&[]).is_err());
        assert!(super::tdi62_parse_mode(&["--full".to_owned(), "extra".to_owned()]).is_err());
    }

    #[test]
    fn parse_mode_selects_full_only_for_the_exact_single_flag() {
        assert_eq!(
            super::tdi62_parse_mode(&["--full".to_owned()]).unwrap(),
            super::Tdi62Mode::Full
        );
        assert_eq!(
            super::tdi62_parse_mode(&["--preflight".to_owned()]).unwrap(),
            super::Tdi62Mode::Preflight
        );
        assert_eq!(
            super::tdi62_parse_mode(&["--termination-smoke".to_owned()]).unwrap(),
            super::Tdi62Mode::TerminationSmoke
        );
        assert!(super::tdi62_parse_mode(&["--Full".to_owned()]).is_err());
    }

    #[test]
    fn usage_error_mentions_every_flag_and_the_confirmation_variable() {
        let usage = super::tdi62_usage_error();
        assert!(usage.contains("--termination-smoke"));
        assert!(usage.contains("--preflight"));
        assert!(usage.contains("--full"));
        assert!(usage.contains(TDI62_FULL_RUN_CONFIRMATION_VAR));
        assert!(usage.contains(TDI62_FULL_RUN_CONFIRMATION_VALUE));
    }

    #[test]
    fn full_run_refuses_before_any_work_without_the_confirmation_token() {
        // Never reach the accepted path in a test: assert the guard var is
        // absent first, then confirm the unconfirmed call returns an error
        // before any generation, fitting or bootstrap.
        if std::env::var(TDI62_FULL_RUN_CONFIRMATION_VAR).is_ok() {
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
            body.contains("run_tdi62_pipeline(&population_specs())"),
            "accepted path must call the real pipeline over the real specs"
        );
        assert!(body.contains("tdi62_full_run_confirmed"));
        assert!(body.contains("print_tdi52_required_raw_output"));
    }

    #[test]
    fn termination_smoke_uses_only_bounded_specs_never_the_real_ones() {
        let source = evaluator_source();
        let start = source
            .find("fn run_termination_smoke()")
            .expect("run_termination_smoke must exist");
        let end = source[start..]
            .find("\nfn tdi62_full_run_confirmed")
            .map(|offset| start + offset)
            .expect("tdi62_full_run_confirmed must follow run_termination_smoke");
        let body = &source[start..end];

        assert!(body.contains("target_count: 1"));
        assert!(
            !body.contains("run_tdi62_pipeline(&population_specs())"),
            "the smoke path must never run the real-scale pipeline"
        );
    }

    // --- Populations and seed blocks (Sections 8, 9) ---

    #[test]
    fn population_specs_total_twelve_four_per_block_and_have_no_ood() {
        let specs = super::population_specs();
        assert_eq!(specs.len(), TOTAL_SEED_RESERVATIONS);
        assert_eq!(specs.len(), 12);
        for block in super::FROZEN_BLOCK_ORDER {
            assert_eq!(specs.iter().filter(|s| s.seed_block == block).count(), 4);
        }
        // No population is wider than width 4 (single generator, no OOD).
        assert!(specs.iter().all(|s| s.width <= 4));
    }

    #[test]
    fn each_block_requests_forty_thousand_and_total_is_120000() {
        let specs = super::population_specs();
        for block in super::FROZEN_BLOCK_ORDER {
            let block_total: usize = specs
                .iter()
                .filter(|s| s.seed_block == block)
                .map(|s| s.target_count)
                .sum();
            assert_eq!(block_total, 40_000);
        }
        let grand_total: usize = specs.iter().map(|s| s.target_count).sum();
        assert_eq!(grand_total, 120_000);
    }

    #[test]
    fn preregistered_seed_reservations_are_twelve_and_pairwise_disjoint() {
        assert_eq!(
            super::validate_preregistered_seed_reservations().unwrap(),
            12
        );
    }

    #[test]
    fn seed_blocks_are_pqr_and_disjoint_from_every_prior_block() {
        let ids: Vec<_> = SEED_BLOCKS.iter().map(|b| b.id).collect();
        assert_eq!(ids, vec![SeedBlockId::P, SeedBlockId::Q, SeedBlockId::R]);
        // Fresh population base seeds start at 4.0e9 (Section 9): TDI-6.1
        // consumes seeds up to ≈ 3.23e9, so every 6.2 population seed is above
        // every prior block's consumed range.
        for block in SEED_BLOCKS {
            for seed in [
                block.training_width_3_seed,
                block.holdout_width_3_seed,
                block.training_width_4_seed,
                block.holdout_width_4_seed,
            ] {
                assert!(seed >= 4_000_000_000);
            }
        }
        // The population base for block index b is 4_000_000_000 + b·100_000_000
        // and the four populations start at base + {0, 10, 20, 30}·10⁶.
        for (b, block) in SEED_BLOCKS.iter().enumerate() {
            let base = 4_000_000_000 + (b as u64) * 100_000_000;
            assert_eq!(block.training_width_3_seed, base);
            assert_eq!(block.holdout_width_3_seed, base + 10_000_000);
            assert_eq!(block.training_width_4_seed, base + 20_000_000);
            assert_eq!(block.holdout_width_4_seed, base + 30_000_000);
        }
        // Bootstrap seeds carry the TDI6 prefix with the `32` = ".2" marker,
        // distinct from every TDI5-prefixed and `31`/TDI-6.1 ancestor seed.
        let boots: Vec<_> = SEED_BLOCKS.iter().map(|b| b.bootstrap_seed).collect();
        assert_eq!(
            boots,
            vec![
                0x5444_4936_3200_0001_u64,
                0x5444_4936_3200_0002,
                0x5444_4936_3200_0003
            ]
        );
        assert_eq!(AGGREGATE_BOOTSTRAP_SEED, 0x5444_4936_3200_4700);
        // The aggregate seed is distinct from every block seed.
        assert!(!boots.contains(&AGGREGATE_BOOTSTRAP_SEED));
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
        let seed = SEED_BLOCKS[0].training_width_3_seed;
        let first = super::generate_records_with_limits(
            3,
            seed,
            4,
            super::preregistered_generation_limits(3, seed, 4).unwrap(),
        )
        .expect("bounded width-3 generation");
        let second = super::generate_records_with_limits(
            3,
            seed,
            4,
            super::preregistered_generation_limits(3, seed, 4).unwrap(),
        )
        .expect("bounded width-3 generation");
        assert_eq!(first.records.len(), 4);
        assert_eq!(first.next_seed, second.next_seed);
        assert_eq!(first.attempts, second.attempts);
        for (a, b) in first.records.iter().zip(second.records.iter()) {
            assert_eq!(a.early_overlap, b.early_overlap);
            assert_eq!(a.contraction, b.contraction);
            assert_eq!(a.spectral, b.spectral);
            // The non-exact literal spectral descriptors reproduce bit-for-bit
            // on the same toolchain/architecture (Section 12): identical inputs
            // through the identical fixed-order f64 pipeline.
            assert_eq!(a.literal_spectral, b.literal_spectral);
            assert_eq!(a.targets_u, b.targets_u);
        }
        // The contraction descriptors are finite and in [0, 1]; the exact
        // spectral moments are finite and in [0, 2^width] (here 2^3 = 8); the
        // literal gap g and the normalized mixing time τ/T_max are finite and in
        // [0, 1] (up to last-digit f64 noise on g).
        for record in &first.records {
            for &value in &record.contraction {
                assert!(value.is_finite() && (0.0..=1.0).contains(&value));
            }
            for &value in &record.spectral {
                assert!(value.is_finite() && (0.0..=8.0).contains(&value));
            }
            let [gap, tau] = record.literal_spectral;
            assert!(
                gap.is_finite() && (-1e-9..=1.0).contains(&gap),
                "gap = {gap}"
            );
            assert!(tau.is_finite() && (0.0..=1.0).contains(&tau), "tau = {tau}");
        }
    }

    #[test]
    fn gkt_ridge_fit_and_prediction_are_deterministic_and_reconstruct_overlap() {
        let records: Vec<Record> = (0..24)
            .map(|i| {
                let o1 = 0.10 + 0.02 * f64::from(i % 7);
                let o2 = 0.20 + 0.015 * f64::from(i % 5);
                record_with_overlap(o1, o2)
            })
            .collect();

        let targets = super::overlap_values(&records, super::primary_horizon_index());
        // The model design is the degree-2 expansion (TDI-6.2 Section 6), exactly
        // as the fit and prediction paths use it.
        let design = super::feature_matrix(&records, |record| {
            super::model_features(record, FeatureLayout::Gkt)
        });

        let first = super::fit_ridge(&design, &targets).expect("ridge fit");
        let second = super::fit_ridge(&design, &targets).expect("ridge fit");
        assert_eq!(first.coefficients, second.coefficients);
        // Per-feature scalers cover all degree-2 expanded GKT columns
        // (21 linear + 21·22/2 = 231 interactions = 252); coefficients carry an
        // additional intercept at index 0.
        let expanded = super::expanded_column_count(GKT_FEATURE_COUNT);
        assert_eq!(expanded, 252);
        assert_eq!(first.means.len(), expanded);
        assert_eq!(first.coefficients.len(), expanded + 1);

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
            FeatureLayout::Gkt,
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
        let script = read_repo_file("scripts/reproduce-tdi6.2.sh");

        assert!(
            script.contains("\"$BINARY_PATH\" --full 2>&1 | tee"),
            "the script must invoke the binary with --full, capturing combined output"
        );
        assert!(
            !script.contains("\"$BINARY_PATH\" 2>&1 | tee"),
            "the script must not invoke the binary without --full"
        );
        assert!(script.contains(TDI62_FULL_RUN_CONFIRMATION_VAR));
        assert!(script.contains(TDI62_FULL_RUN_CONFIRMATION_VALUE));
        assert!(script.contains("require_full_run_confirmation"));
    }

    #[test]
    fn reproduction_script_refuses_to_overwrite_an_existing_result_and_verifies_the_ancestors() {
        let script = read_repo_file("scripts/reproduce-tdi6.2.sh");

        assert!(script.contains("refuse_existing_output"));
        assert!(script.contains("already exists"));
        assert!(script.contains("refusing to overwrite"));
        // The reproduction must verify the full frozen chain TDI-5.1 → 5.7
        // before running (Section 20).
        assert!(script.contains("FROZEN_TDI51_"));
        assert!(script.contains("FROZEN_TDI52_"));
        assert!(script.contains("FROZEN_TDI53_"));
        assert!(script.contains("FROZEN_TDI54_"));
        assert!(script.contains("FROZEN_TDI55_"));
        assert!(script.contains("FROZEN_TDI56_"));
        assert!(script.contains("FROZEN_TDI57_"));
        assert!(script.contains("FROZEN_TDI61_"));
    }

    // --- Frozen ancestors must never change under TDI-6.2 ---

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
            (
                "tdi-bench/src/bin/tdi-independent-overlap-ablation-v57.rs",
                "900031bc27a35e327038911d93f10d74458f913e64d9644b225963df699049ae",
            ),
            (
                "docs/TDI-5.7-GENERATOR-ROBUSTNESS-PREREGISTRATION.md",
                "2ca7d1a674d451e642beb5b01f8a0d8f08f8fadcf7f91032370e7fd5e3d91476",
            ),
            (
                "tdi-bench/src/bin/tdi-independent-overlap-ablation-v61.rs",
                "bb9d155021117b70d1483a9abbc51f45f994caddb8a17365d7fb14f02201f278",
            ),
            (
                "docs/TDI-6.1-SPECTRAL-GAP-MIXING-TIME-PREREGISTRATION.md",
                "4d754f334c95b113078c28a24069ffd8fb3e93e2ba89055001aab3bf3ee1a159",
            ),
        ];

        for (path, want) in expected {
            let got = super::tdi52_sha256_of_repo_file(path);
            assert_eq!(&got, want, "frozen ancestor changed: {path}");
        }
    }

    // --- TDI-6.2 non-exact spectral descriptors (Sections 6, 7, 12) ---

    /// The largest eigenvalue modulus over all eigenvalues (diagnostic helper).
    fn largest_modulus(spectrum: &[Complex64]) -> f64 {
        spectrum
            .iter()
            .map(|value| value.modulus())
            .fold(0.0_f64, f64::max)
    }

    /// trace(P^k) computed directly by repeated dense multiplication.
    fn trace_of_power(matrix: &[Vec<f64>], k: usize) -> f64 {
        let n = matrix.len();
        let mut power = matrix.to_vec();
        for _ in 1..k {
            let mut next = vec![vec![0.0_f64; n]; n];
            for i in 0..n {
                for t in 0..n {
                    for j in 0..n {
                        next[i][j] += power[i][t] * matrix[t][j];
                    }
                }
            }
            power = next;
        }
        (0..n).map(|i| power[i][i]).sum()
    }

    /// A deterministic, sparse-ish random row-stochastic matrix of size `n`,
    /// built from the frozen splitmix64 stream so the battery is reproducible.
    fn random_stochastic(n: usize, seed: u64) -> Vec<Vec<f64>> {
        let mut state = seed;
        let mut next = || {
            state = state.wrapping_add(0x9e37_79b9_7f4a_7c15);
            (super::splitmix64(state) >> 11) as f64 / (1u64 << 53) as f64
        };
        let mut matrix = vec![vec![0.0_f64; n]; n];
        for row in matrix.iter_mut() {
            let mut sum = 0.0;
            for cell in row.iter_mut() {
                let value = if next() < 0.5 { 0.0 } else { next() };
                *cell = value;
                sum += value;
            }
            if sum == 0.0 {
                row[0] = 1.0;
                sum = 1.0;
            }
            for cell in row.iter_mut() {
                *cell /= sum;
            }
        }
        matrix
    }

    #[test]
    fn eigenvalues_recover_a_known_diagonal_spectrum() {
        let matrix = vec![
            vec![0.5, 0.0, 0.0],
            vec![0.0, 0.2, 0.0],
            vec![0.0, 0.0, -0.3],
        ];
        let mut moduli: Vec<f64> = super::eigenvalues(&matrix)
            .iter()
            .map(|value| value.modulus())
            .collect();
        moduli.sort_by(|a, b| b.partial_cmp(a).unwrap());
        assert!((moduli[0] - 0.5).abs() < 1e-9);
        assert!((moduli[1] - 0.3).abs() < 1e-9);
        assert!((moduli[2] - 0.2).abs() < 1e-9);
    }

    #[test]
    fn eigenvalues_recover_a_symmetric_tridiagonal_spectrum() {
        // [[2,1,0],[1,2,1],[0,1,2]] has eigenvalues 2 and 2 ± √2.
        let matrix = vec![
            vec![2.0, 1.0, 0.0],
            vec![1.0, 2.0, 1.0],
            vec![0.0, 1.0, 2.0],
        ];
        let mut moduli: Vec<f64> = super::eigenvalues(&matrix)
            .iter()
            .map(|value| value.modulus())
            .collect();
        moduli.sort_by(|a, b| b.partial_cmp(a).unwrap());
        assert!((moduli[0] - (2.0 + 2.0_f64.sqrt())).abs() < 1e-9);
        assert!((moduli[1] - 2.0).abs() < 1e-9);
        assert!((moduli[2] - (2.0 - 2.0_f64.sqrt())).abs() < 1e-9);
    }

    #[test]
    fn slem_of_a_permutation_on_the_unit_circle_is_one() {
        // The 3-cycle permutation has the cube roots of unity as its spectrum:
        // all three eigenvalues have modulus 1, so removing one Perron
        // eigenvalue still leaves |λ2| = 1 (a periodic, non-mixing kernel).
        let matrix = vec![
            vec![0.0, 1.0, 0.0],
            vec![0.0, 0.0, 1.0],
            vec![1.0, 0.0, 0.0],
        ];
        let spectrum = super::eigenvalues(&matrix);
        assert!((largest_modulus(&spectrum) - 1.0).abs() < 1e-9);
        assert!((super::second_largest_modulus(&spectrum) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn slem_of_a_two_state_chain_is_the_closed_form() {
        // P = [[1-a, a], [b, 1-b]] has eigenvalues 1 and (1 - a - b), so the
        // literal second-largest modulus is |1 - a - b|.
        for (a, b) in [(0.3, 0.2), (0.7, 0.1), (0.5, 0.5), (0.9, 0.9)] {
            let matrix = vec![vec![1.0 - a, a], vec![b, 1.0 - b]];
            let slem = super::second_largest_modulus(&super::eigenvalues(&matrix));
            assert!(
                (slem - (1.0 - a - b).abs()).abs() < 1e-9,
                "a={a}, b={b}, slem={slem}"
            );
        }
    }

    #[test]
    fn slem_of_the_averaging_chain_is_zero() {
        // Rank-one uniform kernel: eigenvalues 1 and 0, so |λ2| = 0, gap = 1.
        let matrix = vec![vec![0.5, 0.5], vec![0.5, 0.5]];
        let slem = super::second_largest_modulus(&super::eigenvalues(&matrix));
        assert!(slem < 1e-9, "slem = {slem}");
    }

    #[test]
    fn spectrum_satisfies_the_trace_invariant_on_random_stochastic_kernels() {
        // The rigorous, self-contained correctness witness for the canonical
        // eigensolver: the power sums Σλᵢᵏ must equal trace(Pᵏ) exactly (up to
        // f64), for k = 1, 2, 3, on real branching-scale kernels n = 8 and 16.
        for &n in &[8_usize, 16] {
            for replicate in 0..64 {
                let matrix = random_stochastic(n, 0xA5A5_0000 ^ (n as u64) << 32 ^ replicate);
                let spectrum = super::eigenvalues(&matrix);
                assert_eq!(spectrum.len(), n);
                let residual = super::spectral_trace_residual(&matrix, &spectrum);
                assert!(
                    residual < 1e-9,
                    "n={n}, replicate={replicate}, residual={residual}"
                );
                // Independent cross-check of the production residual: compute
                // Σλᵢ² here and compare to a from-scratch trace(P²).
                let mut power_sum2 = Complex64::real(0.0);
                for value in &spectrum {
                    power_sum2 = power_sum2.add(value.mul(*value));
                }
                let direct_trace2 = trace_of_power(&matrix, 2);
                assert!((power_sum2.re - direct_trace2).abs() < 1e-9);
                assert!(power_sum2.im.abs() < 1e-9);
                // Every eigenvalue of a stochastic matrix lies in the unit disk.
                for value in &spectrum {
                    assert!(value.modulus() <= 1.0 + 1e-9);
                }
            }
        }
    }

    #[test]
    fn method_one_and_method_two_agree_on_symmetric_kernels_within_tolerance() {
        // On symmetric (hence real-spectrum) doubly-stochastic kernels the
        // canonical eigensolver (method 1) and the deflated power iteration
        // (method 2) must agree on |λ2| within the declared cross-method
        // tolerance — the Section 7 correctness guarantee for the descriptors.
        let kernels = vec![
            vec![vec![0.6, 0.4], vec![0.4, 0.6]],
            vec![vec![0.5, 0.5], vec![0.5, 0.5]],
            vec![
                vec![0.5, 0.3, 0.2],
                vec![0.3, 0.4, 0.3],
                vec![0.2, 0.3, 0.5],
            ],
            vec![
                vec![0.7, 0.1, 0.1, 0.1],
                vec![0.1, 0.7, 0.1, 0.1],
                vec![0.1, 0.1, 0.7, 0.1],
                vec![0.1, 0.1, 0.1, 0.7],
            ],
        ];
        for matrix in kernels {
            let method1 = super::second_largest_modulus(&super::eigenvalues(&matrix));
            let stationary = super::stationary_distribution(&matrix);
            let method2 = super::power_iteration_second_modulus(&matrix, &stationary);
            assert!(
                (method1 - method2).abs() <= SPECTRAL_CROSS_METHOD_TOLERANCE,
                "method1={method1}, method2={method2}"
            );
        }
    }

    #[test]
    fn reference_crate_crosscheck_falls_back_to_methods_one_two_and_known_spectra() {
        // Method 3 (Section 7) is a battle-tested reference eigensolver admitted
        // ONLY as a test-only dev-dependency, so the frozen feature path stays
        // dependency-free. No reference crate is vendored in this offline
        // environment, so — exactly as Section 4.3 declares — the cross-check
        // falls back to methods-1↔2 agreement together with the closed-form
        // known-spectra battery, which alone establish the canonical path's
        // correctness. This test enforces that fallback is always available:
        // for kernels with a KNOWN closed-form |λ2|, method 1 recovers it and
        // method 2 (where the spectrum is real) confirms it.
        // Known |λ2| = |1 - a - b| for the two-state chain.
        let (a, b): (f64, f64) = (0.3, 0.2);
        let matrix = vec![vec![1.0 - a, a], vec![b, 1.0 - b]];
        let known = (1.0 - a - b).abs();
        let method1 = super::second_largest_modulus(&super::eigenvalues(&matrix));
        let stationary = super::stationary_distribution(&matrix);
        let method2 = super::power_iteration_second_modulus(&matrix, &stationary);
        assert!((method1 - known).abs() <= SPECTRAL_CROSS_METHOD_TOLERANCE);
        assert!((method2 - known).abs() <= SPECTRAL_CROSS_METHOD_TOLERANCE);
    }

    #[test]
    fn kernel_rows_from_a_real_candidate_sum_to_one() {
        let context = super::AttemptContext::new(3, SEED_BLOCKS[0].training_width_3_seed, 0);
        let masks = super::generate_successor_masks(context).expect("masks");
        let system = super::build_system(context, &masks).expect("system");
        let matrix = super::kernel_matrix(context, &system).expect("kernel");
        assert_eq!(matrix.len(), 8); // 2^3 states
        for row in &matrix {
            let sum: f64 = row.iter().sum();
            assert!((sum - 1.0).abs() < 1e-12, "row sum = {sum}");
            assert!(row.iter().all(|&value| value >= 0.0));
        }
    }

    #[test]
    fn mixing_time_matches_a_brute_force_iteration_and_saturates() {
        // Averaging chain: P^1 already equals π, so τ_ε = 1 at any ε ≥ 0.
        let averaging = vec![vec![0.5, 0.5], vec![0.5, 0.5]];
        let stationary = super::stationary_distribution(&averaging);
        assert_eq!(super::mixing_time(&averaging, &stationary), 1);

        // A birth–death chain mixes in finite time; the library mixing time
        // must equal an independent brute-force iteration to the same π.
        let chain = vec![
            vec![0.5, 0.5, 0.0],
            vec![0.25, 0.5, 0.25],
            vec![0.0, 0.5, 0.5],
        ];
        let stationary = super::stationary_distribution(&chain);
        let library = super::mixing_time(&chain, &stationary);
        let brute = brute_force_mixing_time(&chain, &stationary);
        assert_eq!(library, brute);
        assert!((1..MIXING_TIME_CAP).contains(&library));

        // A 2-cycle is periodic: P^t alternates identity/swap and never comes
        // within ε = 1/4 of π, so τ_ε saturates deterministically at T_max.
        let periodic = vec![vec![0.0, 1.0], vec![1.0, 0.0]];
        let stationary = super::stationary_distribution(&periodic);
        assert_eq!(super::mixing_time(&periodic, &stationary), MIXING_TIME_CAP);
    }

    /// Independent brute-force ε-mixing time: iterate P^t explicitly and return
    /// the first t whose worst-start TV distance to π is ≤ ε.
    fn brute_force_mixing_time(matrix: &[Vec<f64>], stationary: &[f64]) -> usize {
        let n = matrix.len();
        let mut power = matrix.to_vec();
        for t in 1..=MIXING_TIME_CAP {
            let worst = (0..n)
                .map(|i| {
                    0.5 * (0..n)
                        .map(|j| (power[i][j] - stationary[j]).abs())
                        .sum::<f64>()
                })
                .fold(0.0_f64, f64::max);
            if worst <= MIXING_EPSILON {
                return t;
            }
            if t == MIXING_TIME_CAP {
                break;
            }
            let mut next = vec![vec![0.0_f64; n]; n];
            for i in 0..n {
                for k in 0..n {
                    for j in 0..n {
                        next[i][j] += power[i][k] * matrix[k][j];
                    }
                }
            }
            power = next;
        }
        MIXING_TIME_CAP
    }

    #[test]
    fn literal_spectral_descriptors_are_gap_and_normalized_mixing_time() {
        // Build a real width-3 candidate and check both descriptors are in
        // range, with g = 1 - |λ2| exactly matching the canonical eigensolver
        // and τ reported as τ_ε / T_max.
        let context = super::AttemptContext::new(3, SEED_BLOCKS[0].training_width_3_seed, 0);
        let masks = super::generate_successor_masks(context).expect("masks");
        let system = super::build_system(context, &masks).expect("system");
        let [gap, tau] =
            super::literal_spectral_descriptors(context, &system).expect("descriptors");

        let matrix = super::kernel_matrix(context, &system).expect("kernel");
        let slem = super::second_largest_modulus(&super::eigenvalues(&matrix));
        assert!((gap - (1.0 - slem)).abs() < 1e-12);

        let stationary = super::stationary_distribution(&matrix);
        let expected_tau = super::mixing_time(&matrix, &stationary) as f64 / MIXING_TIME_CAP as f64;
        assert!((tau - expected_tau).abs() < 1e-12);
        assert!((0.0..=1.0).contains(&tau));
        assert!(gap <= 1.0 + 1e-9);
    }

    #[test]
    fn non_convergence_signals_via_nan_and_is_rejected_not_silently_scored() {
        // Hardening (adversarial-review Findings 1 & 3): a non-finite eigenvalue
        // (the sentinel the eigensolver's non-convergence fallback emits) must
        // propagate to a NaN SLEM so `1 - |λ2|` is non-finite and the candidate
        // is rejected via the NonFiniteFeature path — never absorbed by
        // `f64::max` into a finite-but-wrong gap.
        let spectrum = vec![
            Complex64::real(1.0),
            Complex64::new(f64::NAN, f64::NAN),
            Complex64::real(0.3),
        ];
        assert!(super::second_largest_modulus(&spectrum).is_nan());
        // All-NaN (full non-convergence) also yields NaN, not 0.
        let all_nan = vec![
            Complex64::new(f64::NAN, f64::NAN),
            Complex64::new(f64::NAN, f64::NAN),
        ];
        assert!(super::second_largest_modulus(&all_nan).is_nan());
    }

    #[test]
    fn real_candidate_descriptors_are_always_finite_so_the_guard_never_false_rejects() {
        // The convergence guard must never wrongly reject a VALID candidate: for
        // a battery of real width-3 and width-4 candidate kernels the eigensolver
        // converges and `literal_spectral_descriptors` returns finite [g, τ].
        for (width, base) in [
            (3_u8, SEED_BLOCKS[0].training_width_3_seed),
            (4_u8, SEED_BLOCKS[0].training_width_4_seed),
        ] {
            for offset in 0..24_u64 {
                let context = super::AttemptContext::new(width, base + offset, 0);
                let Ok(masks) = super::generate_successor_masks(context) else {
                    continue;
                };
                let Ok(system) = super::build_system(context, &masks) else {
                    continue;
                };
                let [gap, tau] =
                    super::literal_spectral_descriptors(context, &system).expect("descriptors");
                assert!(gap.is_finite(), "width={width}, seed={}", base + offset);
                assert!(tau.is_finite(), "width={width}, seed={}", base + offset);
            }
        }
    }

    #[test]
    fn complex_arithmetic_is_consistent() {
        // The minimal complex type must behave: (1+2i)(3-i) = 5+5i, and the
        // principal square root of -1 is i.
        let product = Complex64::new(1.0, 2.0).mul(Complex64::new(3.0, -1.0));
        assert!((product.re - 5.0).abs() < 1e-12 && (product.im - 5.0).abs() < 1e-12);
        let root = Complex64::new(-1.0, 0.0).sqrt();
        assert!(root.re.abs() < 1e-9 && (root.im - 1.0).abs() < 1e-9);
    }
}
