//! TDI-5.4 nonlinear-sufficiency and horizon-invariance evaluator.
//!
//! This file is derived from the frozen TDI-5.3 evaluator
//! (`tdi-independent-overlap-ablation-v53.rs`), itself derived from the
//! frozen TDI-5.2 evaluator. TDI-5.1, TDI-5.2 and TDI-5.3 remain frozen
//! and untouched. TDI-5.4 reuses their frozen generator, exact candidate
//! analysis, ridge machinery, deterministic bootstrap engine and the
//! four-way Beneficial/Equivalent/Harmful/Inconclusive classification
//! logic without altering any of them.
//!
//! TDI-5.4 makes exactly the scientific additions its preregistration
//! (`docs/TDI-5.4-NONLINEAR-OVERLAP-SUFFICIENCY-PREREGISTRATION.md`)
//! declares and nothing else:
//!
//!   * a fixed nonlinear basis over the two early-overlap predictors and
//!     two new confirmatory layouts — `N2` (baseline + O2 + O2^2) and
//!     `N12` (baseline + O1 + O2 + O1^2 + O2^2 + O1*O2). The 13 baseline
//!     variables stay linear, so any N12-minus-N2 difference is
//!     attributable solely to O1, its curvature and its interaction with
//!     O2;
//!   * three fresh, independent seed blocks D/E/F, disjoint from the
//!     TDI-5.3 blocks A/B/C;
//!   * the lean in-distribution population set only (width-3 and width-4
//!     training and holdout); no OOD populations;
//!   * criterion TDI-5.4A (the frozen classification applied to N12 vs N2
//!     at the primary horizon U6) and criterion TDI-5.4B (the same
//!     classification at each secondary horizon U3/U4/U5/U8 plus a
//!     horizon-invariance summary).
//!
//! The full run is gated behind an explicit, exact human confirmation
//! environment variable (see `run_full_experiment` and
//! `tdi54_full_run_confirmed`). No commit, test or CI run supplies that
//! token.

use tdi_core::{
    Action, ExactRatio, State, TableSystem, analyze_branching_recovery, explore,
    uniform_branching_path_entropy_bits,
};

const OBSERVATION_HORIZON: usize = 2;

const TARGET_HORIZONS: [usize; 5] = [3, 4, 5, 6, 8];
const TARGET_HORIZON_COUNT: usize = TARGET_HORIZONS.len();
const PRIMARY_HORIZON: usize = 6;
const PRIMARY_HORIZON_INDEX: usize = 3;

const TRAIN_WIDTH_3: u8 = 3;
const TRAIN_WIDTH_4: u8 = 4;
// Widths 5 and 6 remain supported by the inherited frozen generator and
// its exact cardinality/budget machinery, but TDI-5.4 generates no
// populations at those widths (Section 6): there are no OOD populations.
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

// Linear layouts, inherited from TDI-5.2/5.3. In TDI-5.4 they are
// exploratory only (Section 13) and determine no confirmatory criterion.
const B0_FEATURE_COUNT: usize = BASELINE_FEATURE_COUNT;
const B1_FEATURE_COUNT: usize = BASELINE_FEATURE_COUNT + 1;
const B2_FEATURE_COUNT: usize = BASELINE_FEATURE_COUNT + 1;
const B12_FEATURE_COUNT: usize = BASELINE_FEATURE_COUNT + EARLY_OVERLAP_FEATURE_COUNT;
const BD_FEATURE_COUNT: usize = BASELINE_FEATURE_COUNT + 1;

// Confirmatory nonlinear layouts, new in TDI-5.4 (Section 5). The 13
// baseline variables stay linear; only the overlap block gets nonlinear
// terms, so N12 minus N2 isolates O1, its curvature and its interaction.
//   N2  = baseline + O2 + O2^2                      (13 + 2 = 15)
//   N12 = baseline + O1 + O2 + O1^2 + O2^2 + O1*O2  (13 + 5 = 18)
const N2_FEATURE_COUNT: usize = BASELINE_FEATURE_COUNT + 2;
const N12_FEATURE_COUNT: usize = BASELINE_FEATURE_COUNT + 5;

const MODEL_LAYOUT_COUNT: usize = 7;

const RIDGE_LAMBDA: f64 = 1.0;
const BOOTSTRAP_REPLICATES: usize = 4_000;
const AGGREGATE_BOOTSTRAP_SEED: u64 = 0x5444_4935_3444_4747;

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
    D,
    E,
    F,
}

impl SeedBlockId {
    const fn label(self) -> &'static str {
        match self {
            Self::D => "D",
            Self::E => "E",
            Self::F => "F",
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

// Fresh seed blocks D/E/F, disjoint from the TDI-5.3 blocks A/B/C
// (Section 7). New bootstrap seeds, disjoint from every TDI-5.2/5.3
// bootstrap seed (Section 8).
const SEED_BLOCKS: [SeedBlockSpec; SEED_BLOCK_COUNT] = [
    SeedBlockSpec {
        id: SeedBlockId::D,
        training_width_3_seed: 460_000_000,
        holdout_width_3_seed: 470_000_000,
        training_width_4_seed: 480_000_000,
        holdout_width_4_seed: 490_000_000,
        bootstrap_seed: 0x5444_4935_3444_0001,
    },
    SeedBlockSpec {
        id: SeedBlockId::E,
        training_width_3_seed: 560_000_000,
        holdout_width_3_seed: 570_000_000,
        training_width_4_seed: 580_000_000,
        holdout_width_4_seed: 590_000_000,
        bootstrap_seed: 0x5444_4935_3445_0002,
    },
    SeedBlockSpec {
        id: SeedBlockId::F,
        training_width_3_seed: 660_000_000,
        holdout_width_3_seed: 670_000_000,
        training_width_4_seed: 680_000_000,
        holdout_width_4_seed: 690_000_000,
        bootstrap_seed: 0x5444_4935_3446_0003,
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
    // Linear layouts B0..BD are exploratory in TDI-5.4. Their discriminants
    // (0..4) are preserved so `layout as usize` indexing is unchanged from
    // TDI-5.2/5.3. The confirmatory nonlinear layouts N2/N12 follow.
    B0,
    B1,
    B2,
    B12,
    BD,
    N2,
    N12,
}

impl FeatureLayout {
    const ALL: [Self; MODEL_LAYOUT_COUNT] = [
        Self::B0,
        Self::B1,
        Self::B2,
        Self::B12,
        Self::BD,
        Self::N2,
        Self::N12,
    ];

    const fn label(self) -> &'static str {
        match self {
            Self::B0 => "B0 — BASELINE",
            Self::B1 => "B1 — BASELINE + O1",
            Self::B2 => "B2 — BASELINE + O2",
            Self::B12 => "B12 — BASELINE + O1 + O2",
            Self::BD => "BD — BASELINE + (O2 - O1), EXPLORATORY",
            Self::N2 => "N2 — BASELINE + O2 + O2^2 (nonlinéaire)",
            Self::N12 => "N12 — BASELINE + O1 + O2 + O1^2 + O2^2 + O1*O2 (nonlinéaire)",
        }
    }

    const fn feature_count(self) -> usize {
        match self {
            Self::B0 => B0_FEATURE_COUNT,
            Self::B1 => B1_FEATURE_COUNT,
            Self::B2 => B2_FEATURE_COUNT,
            Self::B12 => B12_FEATURE_COUNT,
            Self::BD => BD_FEATURE_COUNT,
            Self::N2 => N2_FEATURE_COUNT,
            Self::N12 => N12_FEATURE_COUNT,
        }
    }
}

#[derive(Clone, Debug)]
struct Record {
    baseline: [f64; BASELINE_FEATURE_COUNT],
    early_overlap: [f64; EARLY_OVERLAP_FEATURE_COUNT],
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
    Accepted(Record),
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
        // Nonlinear layouts (TDI-5.4 Section 5). Terms are built from the
        // raw overlap predictors; standardization happens downstream in
        // ridge fitting, exactly like every other feature. The baseline
        // block is never given nonlinear terms.
        FeatureLayout::N2 => {
            features.push(second_overlap);
            features.push(second_overlap * second_overlap);
        }
        FeatureLayout::N12 => {
            features.push(first_overlap);
            features.push(second_overlap);
            features.push(first_overlap * first_overlap);
            features.push(second_overlap * second_overlap);
            features.push(first_overlap * second_overlap);
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

    if baseline
        .iter()
        .chain(&early_overlap)
        .any(|value| !value.is_finite())
    {
        return Ok(CandidateOutcome::Rejected(
            RejectionReason::NonFiniteFeature,
        ));
    }

    Ok(CandidateOutcome::Accepted(Record {
        baseline,
        early_overlap,
        overlaps,
        targets_u,
    }))
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
                format!("width {width} is not part of the TDI-5.1 preregistered populations"),
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
/// that specifically need the real 18-reservation contract should use
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
                records.push(record);
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
    /// named fields directly. TDI-5.4 has no OOD populations (Section 6).
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

const FROZEN_BLOCK_ORDER: [SeedBlockId; SEED_BLOCK_COUNT] =
    [SeedBlockId::D, SeedBlockId::E, SeedBlockId::F];

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
                "requires deterministic block order A, B, C; found {} where {} was expected",
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

#[derive(Clone, Debug)]
struct Tdi52LayoutEvaluation {
    layout: FeatureLayout,
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

fn tdi52_evaluate_horizon(
    records: &[Record],
    horizon_index: usize,
    models: &HorizonModels,
    scalers: &[TargetScaler; TARGET_HORIZON_COUNT],
) -> Result<Vec<Tdi52LayoutEvaluation>, String> {
    if records.is_empty() {
        return Err("cannot evaluate an empty population".to_owned());
    }

    let scaler = scalers[horizon_index];

    let standardized_targets = records
        .iter()
        .map(|record| scaler.standardize(record.targets_u[horizon_index]))
        .collect::<Vec<_>>();

    let overlap_targets = overlap_values(records, horizon_index);

    let mut evaluations = Vec::with_capacity(MODEL_LAYOUT_COUNT);

    for layout in FeatureLayout::ALL {
        let predictions = tdi52_predict(
            records,
            horizon_index,
            layout,
            models.get(horizon_index, layout),
            scaler,
        )?;

        let standardized = calculate_metrics(&standardized_targets, &predictions.standardized);

        let reconstructed = calculate_metrics(&overlap_targets, &predictions.reconstructed_overlap);

        evaluations.push(Tdi52LayoutEvaluation {
            layout,
            standardized,
            reconstructed,
            predictions,
        });
    }

    Ok(evaluations)
}

fn tdi52_layout_evaluation(
    evaluations: &[Tdi52LayoutEvaluation],
    layout: FeatureLayout,
) -> &Tdi52LayoutEvaluation {
    evaluations
        .iter()
        .find(|evaluation| evaluation.layout == layout)
        .expect("all preregistered layouts are evaluated")
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
    baseline: Tdi52LayoutEvaluation,
    challenger: Tdi52LayoutEvaluation,
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
    let evaluations = tdi52_evaluate_horizon(holdout_records, horizon_index, models, scalers)?;
    let baseline = tdi52_layout_evaluation(&evaluations, baseline_layout).clone();
    let challenger = tdi52_layout_evaluation(&evaluations, challenger_layout).clone();

    let scaler = scalers[horizon_index];

    let standardized_targets = holdout_records
        .iter()
        .map(|record| scaler.standardize(record.targets_u[horizon_index]))
        .collect::<Vec<_>>();

    let overlap_targets = overlap_values(holdout_records, horizon_index);

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

/// Criterion TDI-5.4A: the frozen four-way Beneficial/Equivalent/Harmful/
/// Inconclusive classifier (`evaluate_criterion_c`, inherited unchanged
/// from TDI-5.2 Section 13) applied to the nonlinear layouts N12 (challenger)
/// versus N2 (baseline) at the primary horizon U6. See Section 11. N12
/// minus N2 isolates O1, its curvature O1^2 and its interaction O1*O2, so
/// the classification says whether O1 carries independent signal beyond O2
/// once nonlinearity is allowed.
fn tdi54_criterion_a(
    aggregate_fit: &AggregateModelFit,
    combined_holdout_records: [&[Record]; SEED_BLOCK_COUNT],
) -> Result<(AggregateComparison, CriterionCResult), String> {
    let comparison = evaluate_aggregate_comparison(
        primary_horizon_index(),
        aggregate_fit,
        combined_holdout_records,
        FeatureLayout::N2,
        FeatureLayout::N12,
    )?;

    let result = evaluate_criterion_c(&comparison);

    Ok((comparison, result))
}

fn secondary_horizon_indices() -> [usize; TARGET_HORIZON_COUNT - 1] {
    let mut indices = [0_usize; TARGET_HORIZON_COUNT - 1];
    let mut cursor = 0_usize;

    for horizon_index in 0..TARGET_HORIZON_COUNT {
        if horizon_index != primary_horizon_index() {
            indices[cursor] = horizon_index;
            cursor += 1;
        }
    }

    indices
}

/// One secondary horizon's N12-vs-N2 comparison and its four-way
/// classification, produced by criterion TDI-5.4B (Section 12).
#[derive(Clone, Debug)]
struct SecondaryHorizonClassification {
    horizon: usize,
    comparison: AggregateComparison,
    result: CriterionCResult,
}

/// TDI-5.4B horizon-invariance summary (Section 12). This is a
/// preregistered *descriptive* criterion, not a pass/fail gate: it reports
/// how many secondary horizons share the primary-horizon classification
/// and whether the classification is invariant across all of them. Either
/// invariance or a horizon-dependent transition is a legitimate outcome.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct HorizonInvarianceResult {
    primary_classification: CriterionCClassification,
    horizons_matching_primary_class: usize,
    secondary_horizon_count: usize,
    invariant: bool,
}

/// Pure core of the TDI-5.4B invariance summary, over bare classifications
/// so the "matching count / invariant" logic is unit-testable without
/// building any `AggregateComparison`.
fn horizon_invariance_summary(
    primary_classification: CriterionCClassification,
    secondary_classifications: &[CriterionCClassification],
) -> (usize, bool) {
    let matching = secondary_classifications
        .iter()
        .filter(|&&classification| classification == primary_classification)
        .count();

    (matching, matching == secondary_classifications.len())
}

fn evaluate_horizon_invariance(
    primary_classification: CriterionCClassification,
    secondary: &[SecondaryHorizonClassification],
) -> HorizonInvarianceResult {
    let secondary_classifications = secondary
        .iter()
        .map(|entry| entry.result.classification)
        .collect::<Vec<_>>();

    let (horizons_matching_primary_class, invariant) =
        horizon_invariance_summary(primary_classification, &secondary_classifications);

    HorizonInvarianceResult {
        primary_classification,
        horizons_matching_primary_class,
        secondary_horizon_count: secondary.len(),
        invariant,
    }
}

/// Criterion TDI-5.4B: repeat the N12-vs-N2 four-way classification of
/// TDI-5.4A, unchanged, at each secondary horizon U3/U4/U5/U8, and
/// summarise horizon-invariance against the primary-horizon class. See
/// Section 12.
fn tdi54_criterion_b(
    aggregate_fit: &AggregateModelFit,
    combined_holdout_records: [&[Record]; SEED_BLOCK_COUNT],
    primary_classification: CriterionCClassification,
) -> Result<(Vec<SecondaryHorizonClassification>, HorizonInvarianceResult), String> {
    let mut secondary = Vec::with_capacity(TARGET_HORIZON_COUNT - 1);

    for horizon_index in secondary_horizon_indices() {
        let comparison = evaluate_aggregate_comparison(
            horizon_index,
            aggregate_fit,
            combined_holdout_records,
            FeatureLayout::N2,
            FeatureLayout::N12,
        )?;

        let result = evaluate_criterion_c(&comparison);

        secondary.push(SecondaryHorizonClassification {
            horizon: TARGET_HORIZONS[horizon_index],
            comparison,
            result,
        });
    }

    let invariance = evaluate_horizon_invariance(primary_classification, &secondary);

    Ok((secondary, invariance))
}

#[derive(Clone, Debug)]
struct Tdi54ExperimentReport {
    blocks: Vec<BlockPopulations>,
    aggregate_fit: AggregateModelFit,
    criterion_a_comparison: AggregateComparison,
    criterion_a: CriterionCResult,
    criterion_b_secondary: Vec<SecondaryHorizonClassification>,
    criterion_b_invariance: HorizonInvarianceResult,
}

/// Runs the full TDI-5.4 pipeline (generation of the lean width-3/width-4
/// populations across seed blocks D/E/F, per-block ridge fitting on the
/// nonlinear-inclusive design, aggregation, and the two TDI-5.4 criteria)
/// over an arbitrary set of population specifications. Callers control
/// scale entirely through `population_specs`: the preregistered
/// `population_specs()` output requests the real 120,000-record run, while
/// tests and the termination smoke path pass tiny synthetic-scale specs
/// instead. This function is called with the real specs only from
/// `run_full_experiment`'s `--full` path, and only after that path's exact
/// confirmation-token check has passed; tests and the termination smoke
/// path never reach that branch.
fn run_tdi54_pipeline(
    population_specs: &[PopulationSpec],
) -> Result<Tdi54ExperimentReport, String> {
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

    let (criterion_a_comparison, criterion_a) =
        tdi54_criterion_a(&aggregate_fit, combined_holdout_refs)?;
    let (criterion_b_secondary, criterion_b_invariance) = tdi54_criterion_b(
        &aggregate_fit,
        combined_holdout_refs,
        criterion_a.classification,
    )?;

    Ok(Tdi54ExperimentReport {
        blocks,
        aggregate_fit,
        criterion_a_comparison,
        criterion_a,
        criterion_b_secondary,
        criterion_b_invariance,
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

/// Provenance and integrity (TDI-5.4 preregistration Section 15, items
/// 1-5): git commit, compiler/Cargo versions, and the SHA-256 of the v54
/// evaluator, the TDI-5.4 preregistration and the TDI-5.4 scientific
/// manifest — plus the frozen TDI-5.3 and TDI-5.2 ancestors' own
/// identities, read live and printed for provenance (Section 1).
fn print_tdi52_provenance() {
    println!();
    println!("=== PROVENANCE ET INTÉGRITÉ (Section 15, items 1-5) ===");
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
    println!("nombre de dispositions de modèle          : {MODEL_LAYOUT_COUNT}");
    println!("features N2 (baseline + O2 + O2^2)        : {N2_FEATURE_COUNT}");
    println!("features N12 (…+ O1 + O1^2 + O1*O2)        : {N12_FEATURE_COUNT}");
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

/// TDI-5.4 Section 15 items 16-17: every block-level and aggregate-level
/// sub-condition of the TDI-5.4A classification, plus each secondary
/// horizon's TDI-5.4B classification and the invariance summary, printed
/// via `Debug` so the output can never silently drift from the named
/// fields it reflects.
fn print_tdi52_criteria_conditions(report: &Tdi54ExperimentReport) {
    println!();
    println!("=== CONDITIONS PAR CRITÈRE — niveau bloc et agrégat (Section 15, items 16-17) ===");
    println!();
    println!("TDI-5.4A (N12 vs N2 à U6) : {:#?}", report.criterion_a);
    for entry in &report.criterion_b_secondary {
        println!();
        println!("TDI-5.4B — U_{} : {:#?}", entry.horizon, entry.result);
    }
    println!();
    println!(
        "TDI-5.4B — invariance : {:#?}",
        report.criterion_b_invariance
    );
}

/// TDI-5.4 Section 15 items 18-19: the TDI-5.4A four-way classification of
/// O1's nonlinear contribution at U6, and the TDI-5.4B horizon-invariance
/// summary. TDI-5.4A is a preregistered classification and TDI-5.4B is a
/// preregistered descriptive criterion; neither is forced to a positive
/// result and neither is a pass/fail gate.
fn print_tdi52_final_verdicts(report: &Tdi54ExperimentReport) {
    println!();
    println!("=== VERDICTS FINAUX (Section 15, items 18-19) ===");
    println!(
        "TDI-5.4A — contribution O1 nonlinéaire (N12 vs N2, U6) : {}",
        report.criterion_a.classification.label()
    );
    for entry in &report.criterion_b_secondary {
        println!(
            "TDI-5.4B — classification U_{}                          : {}",
            entry.horizon,
            entry.result.classification.label()
        );
    }
    println!(
        "TDI-5.4B — {} horizon(s) secondaire(s) sur {} concordent avec U6",
        report
            .criterion_b_invariance
            .horizons_matching_primary_class,
        report.criterion_b_invariance.secondary_horizon_count
    );
    println!(
        "TDI-5.4B — classification invariante à travers les horizons : {}",
        if report.criterion_b_invariance.invariant {
            "oui"
        } else {
            "non"
        }
    );
}

/// Prints the complete TDI-5.4 required raw output (Section 15) for a
/// completed pipeline run. Purely a presentation layer over
/// `Tdi54ExperimentReport`: it has no scale-awareness of its own, so it is
/// exercised at tiny scale by the termination smoke path and by tests. It
/// only ever prints the real 120,000-record run's output when called from
/// `run_full_experiment`'s `--full` path, and only after that path's exact
/// confirmation-token check has passed.
fn print_tdi52_required_raw_output(report: &Tdi54ExperimentReport) {
    print_tdi52_provenance();
    print_tdi52_frozen_constants();
    print_tdi52_seed_block_definitions();
    print_tdi52_population_accounting(&report.blocks);

    for seed_block in FROZEN_BLOCK_ORDER {
        let fit = report.aggregate_fit.block(seed_block);

        println!();
        println!(
            "### BLOC {} — normalisations et modèles (Section 15, items 12-13) ###",
            seed_block.label()
        );
        tdi52_print_models(&fit.models, &fit.target_scalers);
    }

    print_tdi52_aggregate_comparison(
        "TDI-5.4A — contribution O1 nonlinéaire (N12 vs N2)",
        PRIMARY_HORIZON,
        &report.criterion_a_comparison,
    );

    for entry in &report.criterion_b_secondary {
        print_tdi52_aggregate_comparison(
            &format!(
                "TDI-5.4B — N12 vs N2 à l'horizon secondaire U_{}",
                entry.horizon
            ),
            entry.horizon,
            &entry.comparison,
        );
    }

    print_tdi52_criteria_conditions(report);
    print_tdi52_final_verdicts(report);
}

fn run_termination_smoke() -> Result<(), String> {
    println!("=== TDI-5.4 TERMINATION SMOKE ===");

    // Inherited frozen invariant: the width-6 successor-set space is the
    // exact 2^64. TDI-5.4 generates no width-6 populations, but the
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
    println!(
        "width 3 smoke rejections     : {}",
        report.rejections.summary()
    );

    let specs = population_specs();

    println!(
        "population specifications   : {} deterministic entries (4 per block, no OOD)",
        specs.len()
    );

    // Synthetic, bounded records exercising the nonlinear layouts N2/N12
    // without any real generation. Three width-3 rows and two width-4 rows
    // give the combined training population enough rows for a well-posed
    // ridge fit of the 18-feature N12 design under lambda = 1.
    let synthetic_training_width_3 = [
        Record {
            baseline: [0.0; BASELINE_FEATURE_COUNT],
            early_overlap: [0.20, 0.55],
            overlaps: [0.30; TARGET_HORIZON_COUNT],
            targets_u: [1.00, 1.10, 1.20, 1.30, 1.40],
        },
        Record {
            baseline: [0.1; BASELINE_FEATURE_COUNT],
            early_overlap: [0.25, 0.60],
            overlaps: [0.32; TARGET_HORIZON_COUNT],
            targets_u: [1.50, 1.35, 1.25, 1.15, 1.05],
        },
        Record {
            baseline: [0.15; BASELINE_FEATURE_COUNT],
            early_overlap: [0.30, 0.50],
            overlaps: [0.34; TARGET_HORIZON_COUNT],
            targets_u: [1.20, 1.25, 1.30, 1.35, 1.45],
        },
    ];

    let synthetic_training_width_4 = [
        Record {
            baseline: [0.2; BASELINE_FEATURE_COUNT],
            early_overlap: [0.35, 0.70],
            overlaps: [0.36; TARGET_HORIZON_COUNT],
            targets_u: [2.00, 1.90, 1.80, 1.70, 1.60],
        },
        Record {
            baseline: [0.05; BASELINE_FEATURE_COUNT],
            early_overlap: [0.40, 0.65],
            overlaps: [0.38; TARGET_HORIZON_COUNT],
            targets_u: [1.70, 1.75, 1.80, 1.85, 1.95],
        },
    ];

    // The nonlinear layouts really do build the extra terms.
    let n2_features = feature_layout(&synthetic_training_width_3[0], FeatureLayout::N2);
    let n12_features = feature_layout(&synthetic_training_width_3[0], FeatureLayout::N12);
    println!(
        "nonlinear feature widths     : N2={} (attendu {}), N12={} (attendu {})",
        n2_features.len(),
        N2_FEATURE_COUNT,
        n12_features.len(),
        N12_FEATURE_COUNT
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
        aggregate_fit.block(SeedBlockId::D).seed_block.label(),
        aggregate_fit.block(SeedBlockId::E).seed_block.label(),
        aggregate_fit.block(SeedBlockId::F).seed_block.label()
    );

    let combined_holdout =
        combine_width_3_and_4(&synthetic_training_width_3, &synthetic_training_width_4);
    let holdout_refs: [&[Record]; SEED_BLOCK_COUNT] = [
        combined_holdout.as_slice(),
        combined_holdout.as_slice(),
        combined_holdout.as_slice(),
    ];

    // Exercise the confirmatory N12-vs-N2 comparison and the four-way
    // classifier (criterion TDI-5.4A) on the nonlinear path.
    let (comparison_a, criterion_a) = tdi54_criterion_a(&aggregate_fit, holdout_refs)?;

    println!(
        "identity smoke N2 vs N12 CI  : [{:.6}, {:.6}]",
        comparison_a.aggregate_bootstrap.standardized_mse.lower,
        comparison_a.aggregate_bootstrap.standardized_mse.upper
    );
    println!(
        "identity smoke 5.4A          : classification={}",
        criterion_a.classification.label()
    );

    // Exercise criterion TDI-5.4B (per-secondary-horizon classification and
    // invariance summary).
    let (secondary, invariance) =
        tdi54_criterion_b(&aggregate_fit, holdout_refs, criterion_a.classification)?;

    println!(
        "identity smoke 5.4B          : {} horizons secondaires, {} concordants, invariant={}",
        secondary.len(),
        invariance.horizons_matching_primary_class,
        invariance.invariant
    );

    // The critical wiring smoke: the real pipeline entrypoint, run at tiny
    // scale by requesting exactly one accepted record per population.
    let tiny_population_specs = population_specs().map(|spec| PopulationSpec {
        target_count: 1,
        ..spec
    });

    let pipeline_report =
        run_tdi54_pipeline(&tiny_population_specs).map_err(|error| error.to_string())?;

    println!(
        "identity smoke pipeline      : blocks={}, 5.4A={}, 5.4B invariant={}",
        pipeline_report.blocks.len(),
        pipeline_report.criterion_a.classification.label(),
        pipeline_report.criterion_b_invariance.invariant
    );
    println!(
        "identity smoke pipeline fit  : block D model count={}",
        pipeline_report
            .aggregate_fit
            .block(SeedBlockId::D)
            .models
            .models
            .len()
    );

    print_tdi52_required_raw_output(&pipeline_report);

    println!("bounded smoke result         : PASS");

    Ok(())
}

/// Name of the environment variable that must carry the exact TDI-5.4
/// full-run confirmation value. See TDI-5.4 preregistration Section 14.
const TDI54_FULL_RUN_CONFIRMATION_VAR: &str = "TDI54_CONFIRM_FULL_RUN";

/// The one accepted value for `TDI54_FULL_RUN_CONFIRMATION_VAR`. Any other
/// value, or the variable being unset, must refuse `--full`.
const TDI54_FULL_RUN_CONFIRMATION_VALUE: &str = "I_ACCEPT_THE_TDI54_FREEZE_RULE";

/// Pure decision function: takes the confirmation value as a plain
/// `Option<&str>` rather than reading the environment itself, so every
/// branch -- missing, wrong, and the one exact accepted value -- can be
/// unit tested directly without ever touching a real environment variable
/// or risking the accepted branch reaching `run_full_experiment` (and,
/// through it, the real pipeline).
fn tdi54_full_run_confirmed(value: Option<&str>) -> bool {
    value == Some(TDI54_FULL_RUN_CONFIRMATION_VALUE)
}

fn tdi54_usage_error() -> String {
    format!(
        "usage: tdi-independent-overlap-ablation-v54 --termination-smoke|--preflight|--full\n\
         a bare (no-argument) invocation does not start the experiment; the \
         real run additionally requires the exact environment variable \
         {TDI54_FULL_RUN_CONFIRMATION_VAR}={TDI54_FULL_RUN_CONFIRMATION_VALUE}"
    )
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Tdi54Mode {
    TerminationSmoke,
    Preflight,
    Full,
}

/// Pure command-line dispatch decision, independent of `main`'s I/O, so
/// that "a bare invocation can never select `--full`" is directly unit
/// testable against plain string slices rather than real process argv.
fn tdi54_parse_mode(arguments: &[String]) -> Result<Tdi54Mode, String> {
    match arguments {
        [flag] if flag == "--termination-smoke" => Ok(Tdi54Mode::TerminationSmoke),
        [flag] if flag == "--preflight" => Ok(Tdi54Mode::Preflight),
        [flag] if flag == "--full" => Ok(Tdi54Mode::Full),
        _ => Err(tdi54_usage_error()),
    }
}

fn main() -> Result<(), String> {
    let arguments = std::env::args().skip(1).collect::<Vec<_>>();

    match tdi54_parse_mode(&arguments)? {
        Tdi54Mode::TerminationSmoke => run_termination_smoke(),
        Tdi54Mode::Preflight => run_preflight(),
        Tdi54Mode::Full => run_full_experiment(),
    }
}

/// The TDI-5.4 full-run entrypoint. Checks the exact confirmation
/// environment variable *before* any generation, fitting or bootstrap;
/// only when it matches does this call the real full pipeline exactly
/// once, over the real preregistered `population_specs()`, and print the
/// complete required raw output. See TDI-5.4 preregistration Section 14.
fn run_full_experiment() -> Result<(), String> {
    let confirmation = std::env::var(TDI54_FULL_RUN_CONFIRMATION_VAR).ok();

    if !tdi54_full_run_confirmed(confirmation.as_deref()) {
        return Err(format!(
            "TDI-5.4 full execution requires the exact confirmation environment \
             variable {TDI54_FULL_RUN_CONFIRMATION_VAR}={TDI54_FULL_RUN_CONFIRMATION_VALUE}; \
             refusing before any generation, fitting or bootstrap"
        ));
    }

    let report = run_tdi54_pipeline(&population_specs())?;

    print_tdi52_required_raw_output(&report);

    Ok(())
}

/// TDI-5.4 preflight: verifies the complete frozen configuration (seed
/// reservations, population counts, bootstrap constants, pipeline wiring)
/// and prints identities and the exact real-run command, without ever
/// generating a scientific population. See TDI-5.4 preregistration
/// Section 14.
fn run_preflight() -> Result<(), String> {
    println!();
    println!("=== TDI-5.4 PREFLIGHT (aucune génération scientifique) ===");

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
        "graines de bootstrap par bloc                   : D=0x{:016X} E=0x{:016X} F=0x{:016X}",
        SeedBlockId::D.bootstrap_seed(),
        SeedBlockId::E.bootstrap_seed(),
        SeedBlockId::F.bootstrap_seed()
    );
    println!("graine de bootstrap agrégé stratifié            : 0x{AGGREGATE_BOOTSTRAP_SEED:016X}");
    println!(
        "pipeline complet câblé à --full                 : oui (run_tdi54_pipeline, \
         subordonné à {TDI54_FULL_RUN_CONFIRMATION_VAR})"
    );

    print_tdi52_provenance();

    println!();
    println!("Commande requise pour l'exécution réelle (jamais lancée automatiquement) :");
    println!("  {TDI54_FULL_RUN_CONFIRMATION_VAR}={TDI54_FULL_RUN_CONFIRMATION_VALUE} \\");
    println!("    bash scripts/reproduce-tdi5.4.sh");

    println!();
    println!("=== PREFLIGHT TERMINÉ : aucun résultat produit ===");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        AGGREGATE_BOOTSTRAP_SEED, BASELINE_FEATURE_COUNT, BOOTSTRAP_REPLICATES, Cardinality,
        CriterionCClassification, CriterionCResult, FeatureLayout, MODEL_LAYOUT_COUNT,
        N2_FEATURE_COUNT, N12_FEATURE_COUNT, PRIMARY_HORIZON, Record, SEED_BLOCKS, SeedBlockId,
        TARGET_HORIZONS, TDI54_FULL_RUN_CONFIRMATION_VALUE, TDI54_FULL_RUN_CONFIRMATION_VAR,
        TOTAL_SEED_RESERVATIONS,
    };

    fn read_repo_file(relative_path: &str) -> String {
        std::fs::read_to_string(super::tdi52_repository_root().join(relative_path))
            .unwrap_or_else(|error| panic!("failed to read {relative_path}: {error}"))
    }

    fn evaluator_source() -> String {
        read_repo_file("tdi-bench/src/bin/tdi-independent-overlap-ablation-v54.rs")
    }

    fn record_with_overlap(o1: f64, o2: f64) -> Record {
        Record {
            baseline: [
                0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0, 1.1, 1.2, 1.3,
            ],
            early_overlap: [o1, o2],
            overlaps: [0.30; TARGET_HORIZONS.len()],
            targets_u: [1.0, 1.1, 1.2, 1.3, 1.4],
        }
    }

    // --- Nonlinear basis (the confirmatory novelty, Section 5) ---

    #[test]
    fn n2_features_are_baseline_then_o2_and_o2_squared() {
        let record = record_with_overlap(0.4, 0.6);
        let features = super::feature_layout(&record, FeatureLayout::N2);

        assert_eq!(features.len(), N2_FEATURE_COUNT);
        assert_eq!(features.len(), FeatureLayout::N2.feature_count());
        assert_eq!(&features[..BASELINE_FEATURE_COUNT], &record.baseline);
        assert_eq!(features[BASELINE_FEATURE_COUNT], 0.6);
        assert_eq!(features[BASELINE_FEATURE_COUNT + 1], 0.6 * 0.6);
    }

    #[test]
    fn n12_features_add_o1_o2_their_squares_and_the_interaction() {
        let (o1, o2) = (0.4, 0.6);
        let record = record_with_overlap(o1, o2);
        let features = super::feature_layout(&record, FeatureLayout::N12);

        assert_eq!(features.len(), N12_FEATURE_COUNT);
        assert_eq!(features.len(), FeatureLayout::N12.feature_count());
        assert_eq!(&features[..BASELINE_FEATURE_COUNT], &record.baseline);
        let tail = &features[BASELINE_FEATURE_COUNT..];
        assert_eq!(tail, &[o1, o2, o1 * o1, o2 * o2, o1 * o2]);
    }

    #[test]
    fn nonlinear_layouts_never_perturb_the_linear_baseline_block() {
        // The 13 baseline features are identical across B0, N2 and N12: only
        // the overlap block differs, so any N12-minus-N2 signal is O1's.
        let record = record_with_overlap(0.33, 0.77);
        let b0 = super::feature_layout(&record, FeatureLayout::B0);
        let n2 = super::feature_layout(&record, FeatureLayout::N2);
        let n12 = super::feature_layout(&record, FeatureLayout::N12);

        assert_eq!(&n2[..BASELINE_FEATURE_COUNT], b0.as_slice());
        assert_eq!(&n12[..BASELINE_FEATURE_COUNT], b0.as_slice());
    }

    #[test]
    fn feature_layout_enumeration_has_seven_entries_including_the_nonlinear_pair() {
        assert_eq!(FeatureLayout::ALL.len(), MODEL_LAYOUT_COUNT);
        assert_eq!(MODEL_LAYOUT_COUNT, 7);
        assert!(FeatureLayout::ALL.contains(&FeatureLayout::N2));
        assert!(FeatureLayout::ALL.contains(&FeatureLayout::N12));
        // Linear discriminants are preserved so `layout as usize` indexing
        // is unchanged from TDI-5.2/5.3.
        assert_eq!(FeatureLayout::B0 as usize, 0);
        assert_eq!(FeatureLayout::N2 as usize, 5);
        assert_eq!(FeatureLayout::N12 as usize, 6);
    }

    #[test]
    fn nonlinear_layout_counts_are_fifteen_and_eighteen() {
        assert_eq!(N2_FEATURE_COUNT, 15);
        assert_eq!(N12_FEATURE_COUNT, 18);
    }

    // --- Criterion TDI-5.4A wiring (Section 11) ---

    #[test]
    fn criterion_a_is_wired_to_n2_baseline_and_n12_challenger_at_the_primary_horizon() {
        // Verified by source inspection: TDI-5.4A must compare the nonlinear
        // layouts (N2 baseline, N12 challenger) at the primary horizon,
        // reusing the inherited four-way classifier.
        let source = evaluator_source();
        let start = source
            .find("fn tdi54_criterion_a(")
            .expect("tdi54_criterion_a must exist");
        let end = source[start..]
            .find("\nfn ")
            .map(|offset| start + offset + 1)
            .and_then(|from| source[from..].find("\nfn ").map(|o| from + o))
            .unwrap_or(source.len());
        let body = &source[start..end];

        assert!(body.contains("primary_horizon_index()"));
        assert!(body.contains("FeatureLayout::N2"));
        assert!(body.contains("FeatureLayout::N12"));
        assert!(body.contains("evaluate_criterion_c(&comparison)"));
    }

    // --- Four-way classifier precedence (inherited, Section 13) ---

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

        // Dropping any single condition falls back below Beneficial.
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

    // --- Criterion TDI-5.4B invariance summary (Section 12) ---

    #[test]
    fn invariance_summary_counts_matches_and_flags_full_invariance() {
        let primary = CriterionCClassification::Equivalent;
        let secondary = [
            CriterionCClassification::Equivalent,
            CriterionCClassification::Equivalent,
            CriterionCClassification::Equivalent,
            CriterionCClassification::Equivalent,
        ];
        let (matching, invariant) = super::horizon_invariance_summary(primary, &secondary);
        assert_eq!(matching, 4);
        assert!(invariant);
    }

    #[test]
    fn invariance_is_not_flagged_when_a_secondary_horizon_diverges() {
        let primary = CriterionCClassification::Equivalent;
        let secondary = [
            CriterionCClassification::Equivalent,
            CriterionCClassification::Beneficial,
            CriterionCClassification::Equivalent,
            CriterionCClassification::Equivalent,
        ];
        let (matching, invariant) = super::horizon_invariance_summary(primary, &secondary);
        assert_eq!(matching, 3);
        assert!(!invariant);
    }

    // --- Full-run confirmation guard (Section 14) ---

    #[test]
    fn full_run_confirmation_accepts_only_the_exact_value() {
        assert!(super::tdi54_full_run_confirmed(Some(
            TDI54_FULL_RUN_CONFIRMATION_VALUE
        )));
        assert!(!super::tdi54_full_run_confirmed(None));
        assert!(!super::tdi54_full_run_confirmed(Some("")));
        assert!(!super::tdi54_full_run_confirmed(Some(
            "i_accept_the_tdi54_freeze_rule"
        )));
        assert!(!super::tdi54_full_run_confirmed(Some(
            "I_ACCEPT_THE_TDI53_FREEZE_RULE"
        )));
    }

    #[test]
    fn parse_mode_rejects_a_bare_no_argument_invocation() {
        assert!(super::tdi54_parse_mode(&[]).is_err());
        assert!(super::tdi54_parse_mode(&["--full".to_owned(), "extra".to_owned()]).is_err());
    }

    #[test]
    fn parse_mode_selects_full_only_for_the_exact_single_flag() {
        assert_eq!(
            super::tdi54_parse_mode(&["--full".to_owned()]).unwrap(),
            super::Tdi54Mode::Full
        );
        assert_eq!(
            super::tdi54_parse_mode(&["--preflight".to_owned()]).unwrap(),
            super::Tdi54Mode::Preflight
        );
        assert_eq!(
            super::tdi54_parse_mode(&["--termination-smoke".to_owned()]).unwrap(),
            super::Tdi54Mode::TerminationSmoke
        );
        assert!(super::tdi54_parse_mode(&["--Full".to_owned()]).is_err());
    }

    #[test]
    fn usage_error_mentions_every_flag_and_the_confirmation_variable() {
        let usage = super::tdi54_usage_error();
        assert!(usage.contains("--termination-smoke"));
        assert!(usage.contains("--preflight"));
        assert!(usage.contains("--full"));
        assert!(usage.contains(TDI54_FULL_RUN_CONFIRMATION_VAR));
        assert!(usage.contains(TDI54_FULL_RUN_CONFIRMATION_VALUE));
    }

    #[test]
    fn full_run_refuses_before_any_work_without_the_confirmation_token() {
        // Never reach the accepted path in a test: assert the guard var is
        // absent first, then confirm the unconfirmed call returns an error
        // before any generation, fitting or bootstrap.
        if std::env::var(TDI54_FULL_RUN_CONFIRMATION_VAR).is_ok() {
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
            body.contains("run_tdi54_pipeline(&population_specs())"),
            "accepted path must call the real pipeline over the real specs"
        );
        assert!(body.contains("tdi54_full_run_confirmed"));
        assert!(body.contains("print_tdi52_required_raw_output"));
    }

    #[test]
    fn termination_smoke_uses_only_bounded_specs_never_the_real_ones() {
        let source = evaluator_source();
        let start = source
            .find("fn run_termination_smoke()")
            .expect("run_termination_smoke must exist");
        let end = source[start..]
            .find("\nfn tdi54_full_run_confirmed")
            .map(|offset| start + offset)
            .expect("tdi54_full_run_confirmed must follow run_termination_smoke");
        let body = &source[start..end];

        assert!(body.contains("target_count: 1"));
        assert!(
            !body.contains("run_tdi54_pipeline(&population_specs())"),
            "the smoke path must never run the real-scale pipeline"
        );
    }

    // --- Populations and seed blocks (Sections 6, 7) ---

    #[test]
    fn population_specs_total_twelve_four_per_block_and_have_no_ood() {
        let specs = super::population_specs();
        assert_eq!(specs.len(), TOTAL_SEED_RESERVATIONS);
        assert_eq!(specs.len(), 12);
        for block in super::FROZEN_BLOCK_ORDER {
            assert_eq!(specs.iter().filter(|s| s.seed_block == block).count(), 4);
        }
        // No population is wider than width 4 (no OOD).
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
    fn seed_blocks_are_def_and_disjoint_from_the_tdi53_blocks() {
        let ids: Vec<_> = SEED_BLOCKS.iter().map(|b| b.id).collect();
        assert_eq!(ids, vec![SeedBlockId::D, SeedBlockId::E, SeedBlockId::F]);
        // Fresh offsets (460M..690M), entirely above the TDI-5.3 blocks
        // A/B/C (160M..410M).
        for block in SEED_BLOCKS {
            for seed in [
                block.training_width_3_seed,
                block.holdout_width_3_seed,
                block.training_width_4_seed,
                block.holdout_width_4_seed,
            ] {
                assert!(seed >= 460_000_000);
            }
        }
        // Bootstrap seeds are the fresh, distinct D/E/F constants.
        let boots: Vec<_> = SEED_BLOCKS.iter().map(|b| b.bootstrap_seed).collect();
        assert_eq!(
            boots,
            vec![
                0x5444_4935_3444_0001_u64,
                0x5444_4935_3445_0002,
                0x5444_4935_3446_0003
            ]
        );
        assert_eq!(AGGREGATE_BOOTSTRAP_SEED, 0x5444_4935_3444_4747);
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
        assert_eq!(TARGET_HORIZONS, [3, 4, 5, 6, 8]);
        assert_eq!(TARGET_HORIZONS[super::primary_horizon_index()], 6);
    }

    #[test]
    fn secondary_horizon_indices_exclude_the_primary_horizon() {
        let indices = super::secondary_horizon_indices();
        assert_eq!(indices.len(), TARGET_HORIZONS.len() - 1);
        assert!(!indices.contains(&super::primary_horizon_index()));
        for &index in &indices {
            assert_ne!(TARGET_HORIZONS[index], PRIMARY_HORIZON);
        }
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
    fn generate_records_is_deterministic_across_repeated_calls() {
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
            assert_eq!(a.targets_u, b.targets_u);
        }
    }

    #[test]
    fn nonlinear_ridge_fit_and_prediction_are_deterministic_and_reconstruct_overlap() {
        let records: Vec<Record> = (0..24)
            .map(|i| {
                let o1 = 0.10 + 0.02 * f64::from(i % 7);
                let o2 = 0.20 + 0.015 * f64::from(i % 5);
                record_with_overlap(o1, o2)
            })
            .collect();

        let targets = super::overlap_values(&records, super::primary_horizon_index());
        let design = super::feature_matrix(&records, |record| {
            super::feature_layout(record, FeatureLayout::N12)
        });

        let first = super::fit_ridge(&design, &targets).expect("ridge fit");
        let second = super::fit_ridge(&design, &targets).expect("ridge fit");
        assert_eq!(first.coefficients, second.coefficients);
        // Per-feature scalers cover all 18 N12 features; coefficients carry
        // an additional intercept at index 0.
        assert_eq!(first.means.len(), N12_FEATURE_COUNT);
        assert_eq!(first.coefficients.len(), N12_FEATURE_COUNT + 1);

        let predicted: Vec<f64> = design.iter().map(|row| first.predict_linear(row)).collect();
        assert_eq!(predicted.len(), records.len());
        assert!(predicted.iter().all(|value| value.is_finite()));

        // The prediction/reconstruction path yields a bounded overlap in
        // [0, 1] for every record.
        let scaler = super::TargetScaler {
            mean: 0.0,
            scale: 1.0,
        };
        let prediction_set = super::tdi52_predict(
            &records,
            super::primary_horizon_index(),
            FeatureLayout::N12,
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

    // --- Reproduction script contract (Section 17) ---

    #[test]
    fn reproduction_script_invokes_full_and_requires_the_exact_token() {
        let script = read_repo_file("scripts/reproduce-tdi5.4.sh");

        assert!(
            script.contains("\"$BINARY_PATH\" --full 2>&1 | tee"),
            "the script must invoke the binary with --full, capturing combined output"
        );
        assert!(
            !script.contains("\"$BINARY_PATH\" 2>&1 | tee"),
            "the script must not invoke the binary without --full"
        );
        assert!(script.contains(TDI54_FULL_RUN_CONFIRMATION_VAR));
        assert!(script.contains(TDI54_FULL_RUN_CONFIRMATION_VALUE));
        assert!(script.contains("require_full_run_confirmation"));
    }

    #[test]
    fn reproduction_script_refuses_to_overwrite_an_existing_result_and_verifies_the_ancestors() {
        let script = read_repo_file("scripts/reproduce-tdi5.4.sh");

        assert!(script.contains("refuse_existing_output"));
        assert!(script.contains("already exists"));
        assert!(script.contains("refusing to overwrite"));
        // The reproduction must verify the full frozen chain before running.
        assert!(script.contains("FROZEN_TDI51_"));
        assert!(script.contains("FROZEN_TDI52_"));
        assert!(script.contains("FROZEN_TDI53_"));
    }

    // --- Frozen ancestors must never change under TDI-5.4 ---

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
        ];

        for (path, want) in expected {
            let got = super::tdi52_sha256_of_repo_file(path);
            assert_eq!(&got, want, "frozen ancestor changed: {path}");
        }
    }
}
