//! TDI-5.2 guarded implementation scaffold.
//!
//! This file is derived mechanically from the frozen TDI-5.1 evaluator.
//! The full scientific execution is deliberately disabled until the
//! preregistered TDI-5.2 layouts, seed blocks, bootstrap and criteria are
//! implemented, tested, manifested and frozen.

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
const OOD_WIDTH_5: u8 = 5;
const OOD_WIDTH_6: u8 = 6;

const TRAIN_WIDTH_3_SYSTEMS: usize = 15_000;
const TRAIN_WIDTH_4_SYSTEMS: usize = 15_000;
const HOLDOUT_WIDTH_3_SYSTEMS: usize = 5_000;
const HOLDOUT_WIDTH_4_SYSTEMS: usize = 5_000;
const OOD_WIDTH_5_SYSTEMS: usize = 10_000;
const OOD_WIDTH_6_SYSTEMS: usize = 5_000;

const SEED_BLOCK_COUNT: usize = 3;
const POPULATIONS_PER_SEED_BLOCK: usize = 6;
const TOTAL_SEED_RESERVATIONS: usize = SEED_BLOCK_COUNT * POPULATIONS_PER_SEED_BLOCK;

const BASELINE_FEATURE_COUNT: usize = 13;
const EARLY_OVERLAP_FEATURE_COUNT: usize = 2;

const B0_FEATURE_COUNT: usize = BASELINE_FEATURE_COUNT;
const B1_FEATURE_COUNT: usize = BASELINE_FEATURE_COUNT + 1;
const B2_FEATURE_COUNT: usize = BASELINE_FEATURE_COUNT + 1;
const B12_FEATURE_COUNT: usize = BASELINE_FEATURE_COUNT + EARLY_OVERLAP_FEATURE_COUNT;
const BD_FEATURE_COUNT: usize = BASELINE_FEATURE_COUNT + 1;

const MODEL_LAYOUT_COUNT: usize = 5;

const RIDGE_LAMBDA: f64 = 1.0;
const BOOTSTRAP_REPLICATES: usize = 4_000;
const AGGREGATE_BOOTSTRAP_SEED: u64 = 0x5444_4935_3241_4747;

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
    A,
    B,
    C,
}

impl SeedBlockId {
    const fn label(self) -> &'static str {
        match self {
            Self::A => "A",
            Self::B => "B",
            Self::C => "C",
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
    ood_width_5_seed: u64,
    ood_width_6_seed: u64,
    bootstrap_seed: u64,
}

const SEED_BLOCKS: [SeedBlockSpec; SEED_BLOCK_COUNT] = [
    SeedBlockSpec {
        id: SeedBlockId::A,
        training_width_3_seed: 160_000_000,
        holdout_width_3_seed: 170_000_000,
        training_width_4_seed: 180_000_000,
        holdout_width_4_seed: 190_000_000,
        ood_width_5_seed: 200_000_000,
        ood_width_6_seed: 210_000_000,
        bootstrap_seed: 0x5444_4935_3241_0001,
    },
    SeedBlockSpec {
        id: SeedBlockId::B,
        training_width_3_seed: 260_000_000,
        holdout_width_3_seed: 270_000_000,
        training_width_4_seed: 280_000_000,
        holdout_width_4_seed: 290_000_000,
        ood_width_5_seed: 300_000_000,
        ood_width_6_seed: 310_000_000,
        bootstrap_seed: 0x5444_4935_3242_0002,
    },
    SeedBlockSpec {
        id: SeedBlockId::C,
        training_width_3_seed: 360_000_000,
        holdout_width_3_seed: 370_000_000,
        training_width_4_seed: 380_000_000,
        holdout_width_4_seed: 390_000_000,
        ood_width_5_seed: 400_000_000,
        ood_width_6_seed: 410_000_000,
        bootstrap_seed: 0x5444_4935_3243_0003,
    },
];

// Aliases used only by the unreachable legacy scaffold.
// The final evaluator will orchestrate all three blocks explicitly.
const TRAIN_WIDTH_3_SEED_OFFSET: u64 = SEED_BLOCKS[0].training_width_3_seed;
const HOLDOUT_WIDTH_3_SEED_OFFSET: u64 = SEED_BLOCKS[0].holdout_width_3_seed;
const TRAIN_WIDTH_4_SEED_OFFSET: u64 = SEED_BLOCKS[0].training_width_4_seed;
const HOLDOUT_WIDTH_4_SEED_OFFSET: u64 = SEED_BLOCKS[0].holdout_width_4_seed;
const OOD_WIDTH_5_SEED_OFFSET: u64 = SEED_BLOCKS[0].ood_width_5_seed;
const OOD_WIDTH_6_SEED_OFFSET: u64 = SEED_BLOCKS[0].ood_width_6_seed;
const BOOTSTRAP_SEED: u64 = SEED_BLOCKS[0].bootstrap_seed;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PopulationKind {
    TrainingWidth3,
    HoldoutWidth3,
    TrainingWidth4,
    HoldoutWidth4,
    OodWidth5,
    OodWidth6,
}

impl PopulationKind {
    const ALL: [Self; POPULATIONS_PER_SEED_BLOCK] = [
        Self::TrainingWidth3,
        Self::HoldoutWidth3,
        Self::TrainingWidth4,
        Self::HoldoutWidth4,
        Self::OodWidth5,
        Self::OodWidth6,
    ];

    const fn label(self) -> &'static str {
        match self {
            Self::TrainingWidth3 => "training-w3",
            Self::HoldoutWidth3 => "holdout-w3",
            Self::TrainingWidth4 => "training-w4",
            Self::HoldoutWidth4 => "holdout-w4",
            Self::OodWidth5 => "ood-w5",
            Self::OodWidth6 => "ood-w6",
        }
    }

    const fn width(self) -> u8 {
        match self {
            Self::TrainingWidth3 | Self::HoldoutWidth3 => TRAIN_WIDTH_3,
            Self::TrainingWidth4 | Self::HoldoutWidth4 => TRAIN_WIDTH_4,
            Self::OodWidth5 => OOD_WIDTH_5,
            Self::OodWidth6 => OOD_WIDTH_6,
        }
    }

    const fn target_count(self) -> usize {
        match self {
            Self::TrainingWidth3 => TRAIN_WIDTH_3_SYSTEMS,
            Self::HoldoutWidth3 => HOLDOUT_WIDTH_3_SYSTEMS,
            Self::TrainingWidth4 => TRAIN_WIDTH_4_SYSTEMS,
            Self::HoldoutWidth4 => HOLDOUT_WIDTH_4_SYSTEMS,
            Self::OodWidth5 => OOD_WIDTH_5_SYSTEMS,
            Self::OodWidth6 => OOD_WIDTH_6_SYSTEMS,
        }
    }

    const fn initial_seed(self, block: SeedBlockSpec) -> u64 {
        match self {
            Self::TrainingWidth3 => block.training_width_3_seed,
            Self::HoldoutWidth3 => block.holdout_width_3_seed,
            Self::TrainingWidth4 => block.training_width_4_seed,
            Self::HoldoutWidth4 => block.holdout_width_4_seed,
            Self::OodWidth5 => block.ood_width_5_seed,
            Self::OodWidth6 => block.ood_width_6_seed,
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
    B0,
    B1,
    B2,
    B12,
    BD,
}

impl FeatureLayout {
    const ALL: [Self; MODEL_LAYOUT_COUNT] = [Self::B0, Self::B1, Self::B2, Self::B12, Self::BD];

    const fn label(self) -> &'static str {
        match self {
            Self::B0 => "B0 — BASELINE",
            Self::B1 => "B1 — BASELINE + O1",
            Self::B2 => "B2 — BASELINE + O2",
            Self::B12 => "B12 — BASELINE + O1 + O2",
            Self::BD => "BD — BASELINE + (O2 - O1), EXPLORATORY",
        }
    }

    const fn feature_count(self) -> usize {
        match self {
            Self::B0 => B0_FEATURE_COUNT,
            Self::B1 => B1_FEATURE_COUNT,
            Self::B2 => B2_FEATURE_COUNT,
            Self::B12 => B12_FEATURE_COUNT,
            Self::BD => BD_FEATURE_COUNT,
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

    fn predict_probability(&self, features: &[f64]) -> f64 {
        self.predict_linear(features).clamp(0.0, 1.0)
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

    let transformed = denominator_log2 - numerator_log2;

    if !transformed.is_finite() || transformed < 0.0 {
        return Err(format!(
            "invalid conditional target geometry: {transformed}"
        ));
    }

    Ok(transformed)
}

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

    let normalized = entropy_bits * std::f64::consts::LN_2 / denominator;

    if !normalized.is_finite() {
        return Err(EvaluationError::new(
            context,
            FailureCategory::Arithmetic,
            "non-finite normalized entropy",
        ));
    }

    Ok(normalized)
}

fn normalized_reachable(reachable: f64, context: AttemptContext) -> Result<f64, EvaluationError> {
    let states = state_count(context)? as f64;
    let normalized = reachable / states;

    if !normalized.is_finite() {
        return Err(EvaluationError::new(
            context,
            FailureCategory::Arithmetic,
            format!("non-finite reachable fraction for width {}", context.width),
        ));
    }

    Ok(normalized)
}

fn transformed_path_count(
    path_count: f64,
    context: AttemptContext,
) -> Result<f64, EvaluationError> {
    let transformed = path_count.ln_1p();

    if !transformed.is_finite() {
        return Err(EvaluationError::new(
            context,
            FailureCategory::Arithmetic,
            "non-finite transformed path count",
        ));
    }

    Ok(transformed)
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
        transformed_path_count(reference_paths[0], context)?,
        transformed_path_count(reference_paths[1], context)?,
        normalized_reachable(perturbed_reachable[0], context)?,
        normalized_reachable(perturbed_reachable[1], context)?,
        transformed_path_count(perturbed_paths[0], context)?,
        transformed_path_count(perturbed_paths[1], context)?,
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

fn generate_records(
    width: u8,
    start_seed: u64,
    count: usize,
) -> Result<GenerationReport, GenerationError> {
    let limits = preregistered_generation_limits(width, start_seed, count)
        .map_err(GenerationError::Evaluation)?;

    generate_records_with_limits(width, start_seed, count, limits)
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
        OOD_WIDTH_5 => (WIDTH_5_ATTEMPT_MULTIPLIER, WIDTH_5_NO_PROGRESS_LIMIT),
        OOD_WIDTH_6 => (WIDTH_6_ATTEMPT_MULTIPLIER, WIDTH_6_NO_PROGRESS_LIMIT),
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

fn validate_preregistered_seed_reservations() -> Result<usize, String> {
    let mut ranges = Vec::with_capacity(TOTAL_SEED_RESERVATIONS);

    for spec in population_specs() {
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

    if ranges.len() != TOTAL_SEED_RESERVATIONS {
        return Err(format!(
            "expected {TOTAL_SEED_RESERVATIONS} seed reservations, received {}",
            ranges.len()
        ));
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
    ood_width_5: PopulationGenerationReport,
    ood_width_6: PopulationGenerationReport,
}

impl BlockPopulations {
    fn combined_holdout(&self) -> Vec<Record> {
        combine_width_3_and_4(
            &self.holdout_width_3.report.records,
            &self.holdout_width_4.report.records,
        )
    }

    /// Every population's full generation report, in `PopulationKind::ALL`
    /// order. Required-raw-output printing (Section 17 items 8-11 and 20)
    /// walks this instead of the six named fields directly.
    fn reports(&self) -> [&PopulationGenerationReport; POPULATIONS_PER_SEED_BLOCK] {
        [
            &self.training_width_3,
            &self.holdout_width_3,
            &self.training_width_4,
            &self.holdout_width_4,
            &self.ood_width_5,
            &self.ood_width_6,
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
        ood_width_5: generate(PopulationKind::OodWidth5)?,
        ood_width_6: generate(PopulationKind::OodWidth6)?,
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

fn predictions(model: &RidgeModel, features: &[Vec<f64>]) -> Vec<f64> {
    features
        .iter()
        .map(|row| model.predict_probability(row))
        .collect()
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
    [SeedBlockId::A, SeedBlockId::B, SeedBlockId::C];

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

fn ensure_seed_ranges(ranges: &[(u64, u64, &str)]) -> Result<(), String> {
    for pair in ranges.windows(2) {
        let (_, previous_end, previous_label) = pair[0];
        let (next_start, _, next_label) = pair[1];

        if previous_end > next_start {
            return Err(format!(
                "seed ranges overlap: {previous_label} ends at \
                 {previous_end}, {next_label} starts at {next_start}"
            ));
        }
    }

    Ok(())
}

#[derive(Clone, Debug)]
struct Tdi52PredictionSet {
    standardized: Vec<f64>,
    reconstructed_overlap: Vec<f64>,
    clipped_overlap_count: usize,
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

    let mut clipped_overlap_count = 0_usize;

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

        let (overlap, clipped) = tdi52_reconstruct_overlap(target_u);

        clipped_overlap_count += usize::from(clipped);
        standardized.push(prediction);
        reconstructed_overlap.push(overlap);
    }

    Ok(Tdi52PredictionSet {
        standardized,
        reconstructed_overlap,
        clipped_overlap_count,
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

fn median_of_three(mut values: [f64; SEED_BLOCK_COUNT]) -> f64 {
    values.sort_by(f64::total_cmp);
    values[1]
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct CriterionAResult {
    lower_mse_in_every_block: bool,
    block_bootstrap_lower_bounds_positive: bool,
    median_relative_reduction_at_least_15_percent: bool,
    aggregate_relative_reduction_at_least_15_percent: bool,
    aggregate_bootstrap_lower_bound_positive: bool,
    spearman_improves_in_every_block: bool,
    aggregate_bias_not_worse_than_0_02: bool,
}

impl CriterionAResult {
    const BIAS_MARGIN: f64 = 0.02;
    const RELATIVE_REDUCTION_THRESHOLD: f64 = 0.15;

    fn succeeded(self) -> bool {
        self.lower_mse_in_every_block
            && self.block_bootstrap_lower_bounds_positive
            && self.median_relative_reduction_at_least_15_percent
            && self.aggregate_relative_reduction_at_least_15_percent
            && self.aggregate_bootstrap_lower_bound_positive
            && self.spearman_improves_in_every_block
            && self.aggregate_bias_not_worse_than_0_02
    }
}

fn evaluate_criterion_a(comparison: &AggregateComparison) -> CriterionAResult {
    let block_relative_reductions = FROZEN_BLOCK_ORDER.map(|seed_block| {
        let block = comparison.block(seed_block);

        tdi52_relative_reduction(
            block.baseline.standardized.mse,
            block.challenger.standardized.mse,
        )
    });

    let lower_mse_in_every_block = FROZEN_BLOCK_ORDER.iter().all(|&seed_block| {
        let block = comparison.block(seed_block);

        block.challenger.standardized.mse < block.baseline.standardized.mse
    });

    let block_bootstrap_lower_bounds_positive = FROZEN_BLOCK_ORDER.iter().all(|&seed_block| {
        comparison
            .block(seed_block)
            .bootstrap
            .standardized_mse
            .lower
            > 0.0
    });

    let median_relative_reduction_at_least_15_percent = median_of_three(block_relative_reductions)
        >= CriterionAResult::RELATIVE_REDUCTION_THRESHOLD;

    let aggregate_relative_reduction = tdi52_relative_reduction(
        comparison.aggregate_baseline_standardized.mse,
        comparison.aggregate_challenger_standardized.mse,
    );

    let aggregate_relative_reduction_at_least_15_percent =
        aggregate_relative_reduction >= CriterionAResult::RELATIVE_REDUCTION_THRESHOLD;

    let aggregate_bootstrap_lower_bound_positive =
        comparison.aggregate_bootstrap.standardized_mse.lower > 0.0;

    let spearman_improves_in_every_block = FROZEN_BLOCK_ORDER.iter().all(|&seed_block| {
        let block = comparison.block(seed_block);

        block.challenger.standardized.spearman > block.baseline.standardized.spearman
    });

    let aggregate_bias_not_worse_than_0_02 =
        comparison.aggregate_challenger_standardized.bias.abs()
            <= comparison.aggregate_baseline_standardized.bias.abs()
                + CriterionAResult::BIAS_MARGIN;

    CriterionAResult {
        lower_mse_in_every_block,
        block_bootstrap_lower_bounds_positive,
        median_relative_reduction_at_least_15_percent,
        aggregate_relative_reduction_at_least_15_percent,
        aggregate_bootstrap_lower_bound_positive,
        spearman_improves_in_every_block,
        aggregate_bias_not_worse_than_0_02,
    }
}

fn tdi52_criterion_a(
    aggregate_fit: &AggregateModelFit,
    combined_holdout_records: [&[Record]; SEED_BLOCK_COUNT],
) -> Result<(AggregateComparison, CriterionAResult), String> {
    let comparison = evaluate_aggregate_comparison(
        primary_horizon_index(),
        aggregate_fit,
        combined_holdout_records,
        FeatureLayout::B0,
        FeatureLayout::B12,
    )?;

    let result = evaluate_criterion_a(&comparison);

    Ok((comparison, result))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct CriterionBResult {
    lower_mse_in_every_block: bool,
    block_bootstrap_lower_bounds_positive: bool,
    median_relative_reduction_at_least_10_percent: bool,
    aggregate_relative_reduction_at_least_10_percent: bool,
    spearman_not_lower_in_any_block: bool,
    aggregate_bias_not_worse_than_0_02: bool,
}

impl CriterionBResult {
    const BIAS_MARGIN: f64 = 0.02;
    const RELATIVE_REDUCTION_THRESHOLD: f64 = 0.10;

    fn succeeded(self) -> bool {
        self.lower_mse_in_every_block
            && self.block_bootstrap_lower_bounds_positive
            && self.median_relative_reduction_at_least_10_percent
            && self.aggregate_relative_reduction_at_least_10_percent
            && self.spearman_not_lower_in_any_block
            && self.aggregate_bias_not_worse_than_0_02
    }
}

fn evaluate_criterion_b(comparison: &AggregateComparison) -> CriterionBResult {
    let block_relative_reductions = FROZEN_BLOCK_ORDER.map(|seed_block| {
        let block = comparison.block(seed_block);

        tdi52_relative_reduction(
            block.baseline.standardized.mse,
            block.challenger.standardized.mse,
        )
    });

    let lower_mse_in_every_block = FROZEN_BLOCK_ORDER.iter().all(|&seed_block| {
        let block = comparison.block(seed_block);

        block.challenger.standardized.mse < block.baseline.standardized.mse
    });

    let block_bootstrap_lower_bounds_positive = FROZEN_BLOCK_ORDER.iter().all(|&seed_block| {
        comparison
            .block(seed_block)
            .bootstrap
            .standardized_mse
            .lower
            > 0.0
    });

    let median_relative_reduction_at_least_10_percent = median_of_three(block_relative_reductions)
        >= CriterionBResult::RELATIVE_REDUCTION_THRESHOLD;

    let aggregate_relative_reduction = tdi52_relative_reduction(
        comparison.aggregate_baseline_standardized.mse,
        comparison.aggregate_challenger_standardized.mse,
    );

    let aggregate_relative_reduction_at_least_10_percent =
        aggregate_relative_reduction >= CriterionBResult::RELATIVE_REDUCTION_THRESHOLD;

    let spearman_not_lower_in_any_block = FROZEN_BLOCK_ORDER.iter().all(|&seed_block| {
        let block = comparison.block(seed_block);

        block.challenger.standardized.spearman >= block.baseline.standardized.spearman
    });

    let aggregate_bias_not_worse_than_0_02 =
        comparison.aggregate_challenger_standardized.bias.abs()
            <= comparison.aggregate_baseline_standardized.bias.abs()
                + CriterionBResult::BIAS_MARGIN;

    CriterionBResult {
        lower_mse_in_every_block,
        block_bootstrap_lower_bounds_positive,
        median_relative_reduction_at_least_10_percent,
        aggregate_relative_reduction_at_least_10_percent,
        spearman_not_lower_in_any_block,
        aggregate_bias_not_worse_than_0_02,
    }
}

fn tdi52_criterion_b(
    aggregate_fit: &AggregateModelFit,
    combined_holdout_records: [&[Record]; SEED_BLOCK_COUNT],
) -> Result<(AggregateComparison, CriterionBResult), String> {
    let comparison = evaluate_aggregate_comparison(
        primary_horizon_index(),
        aggregate_fit,
        combined_holdout_records,
        FeatureLayout::B1,
        FeatureLayout::B12,
    )?;

    let result = evaluate_criterion_b(&comparison);

    Ok((comparison, result))
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

    result.classification = if result.is_beneficial() {
        CriterionCClassification::Beneficial
    } else if result.is_equivalent() {
        CriterionCClassification::Equivalent
    } else if result.is_harmful() {
        CriterionCClassification::Harmful
    } else {
        CriterionCClassification::Inconclusive
    };

    result
}

fn tdi52_criterion_c(
    aggregate_fit: &AggregateModelFit,
    combined_holdout_records: [&[Record]; SEED_BLOCK_COUNT],
) -> Result<(AggregateComparison, CriterionCResult), String> {
    let comparison = evaluate_aggregate_comparison(
        primary_horizon_index(),
        aggregate_fit,
        combined_holdout_records,
        FeatureLayout::B2,
        FeatureLayout::B12,
    )?;

    let result = evaluate_criterion_c(&comparison);

    Ok((comparison, result))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct CriterionDWidth5Result {
    positive_mse_improvement_in_every_block: bool,
    bootstrap_lower_bound_positive_in_every_block: bool,
    median_relative_reduction_at_least_20_percent: bool,
    positive_challenger_spearman_in_every_block: bool,
    spearman_not_worse_in_every_block: bool,
    aggregate_bias_strictly_lower: bool,
    positive_aggregate_reconstructed_mse_improvement: bool,
    positive_aggregate_reconstructed_mae_improvement: bool,
}

impl CriterionDWidth5Result {
    const RELATIVE_REDUCTION_THRESHOLD: f64 = 0.20;

    fn succeeded(self) -> bool {
        self.positive_mse_improvement_in_every_block
            && self.bootstrap_lower_bound_positive_in_every_block
            && self.median_relative_reduction_at_least_20_percent
            && self.positive_challenger_spearman_in_every_block
            && self.spearman_not_worse_in_every_block
            && self.aggregate_bias_strictly_lower
            && self.positive_aggregate_reconstructed_mse_improvement
            && self.positive_aggregate_reconstructed_mae_improvement
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct CriterionDWidth6Result {
    positive_mse_improvement_in_every_block: bool,
    bootstrap_lower_bound_positive_in_at_least_two_blocks: bool,
    aggregate_bootstrap_lower_bound_positive: bool,
    positive_challenger_spearman_in_every_block: bool,
    aggregate_spearman_not_worse: bool,
    aggregate_bias_not_worse: bool,
    positive_aggregate_reconstructed_mse_improvement: bool,
}

impl CriterionDWidth6Result {
    fn succeeded(self) -> bool {
        self.positive_mse_improvement_in_every_block
            && self.bootstrap_lower_bound_positive_in_at_least_two_blocks
            && self.aggregate_bootstrap_lower_bound_positive
            && self.positive_challenger_spearman_in_every_block
            && self.aggregate_spearman_not_worse
            && self.aggregate_bias_not_worse
            && self.positive_aggregate_reconstructed_mse_improvement
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct CriterionDResult {
    width_5: CriterionDWidth5Result,
    width_6: CriterionDWidth6Result,
}

impl CriterionDResult {
    fn succeeded(self) -> bool {
        self.width_5.succeeded() && self.width_6.succeeded()
    }
}

fn evaluate_criterion_d_width_5(comparison: &AggregateComparison) -> CriterionDWidth5Result {
    let block_relative_reductions = FROZEN_BLOCK_ORDER.map(|seed_block| {
        let block = comparison.block(seed_block);

        tdi52_relative_reduction(
            block.baseline.standardized.mse,
            block.challenger.standardized.mse,
        )
    });

    let positive_mse_improvement_in_every_block = FROZEN_BLOCK_ORDER.iter().all(|&seed_block| {
        let block = comparison.block(seed_block);

        block.baseline.standardized.mse - block.challenger.standardized.mse > 0.0
    });

    let bootstrap_lower_bound_positive_in_every_block =
        FROZEN_BLOCK_ORDER.iter().all(|&seed_block| {
            comparison
                .block(seed_block)
                .bootstrap
                .standardized_mse
                .lower
                > 0.0
        });

    let median_relative_reduction_at_least_20_percent = median_of_three(block_relative_reductions)
        >= CriterionDWidth5Result::RELATIVE_REDUCTION_THRESHOLD;

    let positive_challenger_spearman_in_every_block =
        FROZEN_BLOCK_ORDER.iter().all(|&seed_block| {
            comparison
                .block(seed_block)
                .challenger
                .standardized
                .spearman
                > 0.0
        });

    let spearman_not_worse_in_every_block = FROZEN_BLOCK_ORDER.iter().all(|&seed_block| {
        let block = comparison.block(seed_block);

        block.challenger.standardized.spearman >= block.baseline.standardized.spearman
    });

    let aggregate_bias_strictly_lower = comparison.aggregate_challenger_standardized.bias.abs()
        < comparison.aggregate_baseline_standardized.bias.abs();

    let positive_aggregate_reconstructed_mse_improvement =
        comparison.aggregate_baseline_reconstructed.mse
            - comparison.aggregate_challenger_reconstructed.mse
            > 0.0;

    let positive_aggregate_reconstructed_mae_improvement =
        comparison.aggregate_baseline_reconstructed.mae
            - comparison.aggregate_challenger_reconstructed.mae
            > 0.0;

    CriterionDWidth5Result {
        positive_mse_improvement_in_every_block,
        bootstrap_lower_bound_positive_in_every_block,
        median_relative_reduction_at_least_20_percent,
        positive_challenger_spearman_in_every_block,
        spearman_not_worse_in_every_block,
        aggregate_bias_strictly_lower,
        positive_aggregate_reconstructed_mse_improvement,
        positive_aggregate_reconstructed_mae_improvement,
    }
}

fn evaluate_criterion_d_width_6(comparison: &AggregateComparison) -> CriterionDWidth6Result {
    let positive_mse_improvement_in_every_block = FROZEN_BLOCK_ORDER.iter().all(|&seed_block| {
        let block = comparison.block(seed_block);

        block.baseline.standardized.mse - block.challenger.standardized.mse > 0.0
    });

    let bootstrap_lower_bound_positive_in_at_least_two_blocks = FROZEN_BLOCK_ORDER
        .iter()
        .filter(|&&seed_block| {
            comparison
                .block(seed_block)
                .bootstrap
                .standardized_mse
                .lower
                > 0.0
        })
        .count()
        >= 2;

    let aggregate_bootstrap_lower_bound_positive =
        comparison.aggregate_bootstrap.standardized_mse.lower > 0.0;

    let positive_challenger_spearman_in_every_block =
        FROZEN_BLOCK_ORDER.iter().all(|&seed_block| {
            comparison
                .block(seed_block)
                .challenger
                .standardized
                .spearman
                > 0.0
        });

    let aggregate_spearman_not_worse = comparison.aggregate_challenger_standardized.spearman
        >= comparison.aggregate_baseline_standardized.spearman;

    let aggregate_bias_not_worse = comparison.aggregate_challenger_standardized.bias.abs()
        <= comparison.aggregate_baseline_standardized.bias.abs();

    let positive_aggregate_reconstructed_mse_improvement =
        comparison.aggregate_baseline_reconstructed.mse
            - comparison.aggregate_challenger_reconstructed.mse
            > 0.0;

    CriterionDWidth6Result {
        positive_mse_improvement_in_every_block,
        bootstrap_lower_bound_positive_in_at_least_two_blocks,
        aggregate_bootstrap_lower_bound_positive,
        positive_challenger_spearman_in_every_block,
        aggregate_spearman_not_worse,
        aggregate_bias_not_worse,
        positive_aggregate_reconstructed_mse_improvement,
    }
}

fn tdi52_criterion_d(
    aggregate_fit: &AggregateModelFit,
    width_5_ood_records: [&[Record]; SEED_BLOCK_COUNT],
    width_6_ood_records: [&[Record]; SEED_BLOCK_COUNT],
) -> Result<((AggregateComparison, AggregateComparison), CriterionDResult), String> {
    let width_5_comparison = evaluate_aggregate_comparison(
        primary_horizon_index(),
        aggregate_fit,
        width_5_ood_records,
        FeatureLayout::B0,
        FeatureLayout::B12,
    )?;

    let width_6_comparison = evaluate_aggregate_comparison(
        primary_horizon_index(),
        aggregate_fit,
        width_6_ood_records,
        FeatureLayout::B0,
        FeatureLayout::B12,
    )?;

    let result = CriterionDResult {
        width_5: evaluate_criterion_d_width_5(&width_5_comparison),
        width_6: evaluate_criterion_d_width_6(&width_6_comparison),
    };

    Ok(((width_5_comparison, width_6_comparison), result))
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

#[derive(Clone, Debug)]
struct HorizonComparison {
    horizon_index: usize,
    comparison: AggregateComparison,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct CriterionEResult {
    horizons_improving_in_every_block: usize,
    u8_improves_in_every_block: bool,
    no_block_horizon_reduction_below_minus_5_percent: bool,
    average_secondary_reduction_positive_in_every_block: bool,
    aggregate_reduction_positive_at_every_secondary_horizon: bool,
}

impl CriterionEResult {
    const RELATIVE_REDUCTION_FLOOR: f64 = -0.05;

    fn succeeded(self) -> bool {
        self.horizons_improving_in_every_block >= 3
            && self.u8_improves_in_every_block
            && self.no_block_horizon_reduction_below_minus_5_percent
            && self.average_secondary_reduction_positive_in_every_block
            && self.aggregate_reduction_positive_at_every_secondary_horizon
    }
}

fn evaluate_criterion_e(horizon_comparisons: &[HorizonComparison]) -> CriterionEResult {
    let horizon_improves_in_every_block = |entry: &HorizonComparison| {
        FROZEN_BLOCK_ORDER.iter().all(|&seed_block| {
            let block = entry.comparison.block(seed_block);

            block.baseline.standardized.mse > block.challenger.standardized.mse
        })
    };

    let horizons_improving_in_every_block = horizon_comparisons
        .iter()
        .filter(|entry| horizon_improves_in_every_block(entry))
        .count();

    let u8_horizon_index = TARGET_HORIZONS
        .iter()
        .position(|&horizon| horizon == 8)
        .expect("U_8 is a frozen target horizon");

    let u8_entry = horizon_comparisons
        .iter()
        .find(|entry| entry.horizon_index == u8_horizon_index)
        .expect("U_8 is evaluated as a secondary horizon");

    let u8_improves_in_every_block = horizon_improves_in_every_block(u8_entry);

    let no_block_horizon_reduction_below_minus_5_percent =
        horizon_comparisons.iter().all(|entry| {
            FROZEN_BLOCK_ORDER.iter().all(|&seed_block| {
                let block = entry.comparison.block(seed_block);
                let relative_reduction = tdi52_relative_reduction(
                    block.baseline.standardized.mse,
                    block.challenger.standardized.mse,
                );

                relative_reduction >= CriterionEResult::RELATIVE_REDUCTION_FLOOR
            })
        });

    let average_secondary_reduction_positive_in_every_block =
        FROZEN_BLOCK_ORDER.iter().all(|&seed_block| {
            let total = horizon_comparisons
                .iter()
                .map(|entry| {
                    let block = entry.comparison.block(seed_block);

                    tdi52_relative_reduction(
                        block.baseline.standardized.mse,
                        block.challenger.standardized.mse,
                    )
                })
                .sum::<f64>();

            total / horizon_comparisons.len() as f64 > 0.0
        });

    let aggregate_reduction_positive_at_every_secondary_horizon =
        horizon_comparisons.iter().all(|entry| {
            tdi52_relative_reduction(
                entry.comparison.aggregate_baseline_standardized.mse,
                entry.comparison.aggregate_challenger_standardized.mse,
            ) > 0.0
        });

    CriterionEResult {
        horizons_improving_in_every_block,
        u8_improves_in_every_block,
        no_block_horizon_reduction_below_minus_5_percent,
        average_secondary_reduction_positive_in_every_block,
        aggregate_reduction_positive_at_every_secondary_horizon,
    }
}

fn tdi52_criterion_e(
    aggregate_fit: &AggregateModelFit,
    combined_holdout_records: [&[Record]; SEED_BLOCK_COUNT],
) -> Result<(Vec<HorizonComparison>, CriterionEResult), String> {
    let mut horizon_comparisons = Vec::with_capacity(TARGET_HORIZON_COUNT - 1);

    for horizon_index in secondary_horizon_indices() {
        let comparison = evaluate_aggregate_comparison(
            horizon_index,
            aggregate_fit,
            combined_holdout_records,
            FeatureLayout::B0,
            FeatureLayout::B12,
        )?;

        horizon_comparisons.push(HorizonComparison {
            horizon_index,
            comparison,
        });
    }

    let result = evaluate_criterion_e(&horizon_comparisons);

    Ok((horizon_comparisons, result))
}

#[derive(Clone, Debug)]
struct Tdi52ExperimentReport {
    blocks: Vec<BlockPopulations>,
    aggregate_fit: AggregateModelFit,
    criterion_a_comparison: AggregateComparison,
    criterion_a: CriterionAResult,
    criterion_b_comparison: AggregateComparison,
    criterion_b: CriterionBResult,
    criterion_c_comparison: AggregateComparison,
    criterion_c: CriterionCResult,
    criterion_d_comparisons: (AggregateComparison, AggregateComparison),
    criterion_d: CriterionDResult,
    criterion_e_horizon_comparisons: Vec<HorizonComparison>,
    criterion_e: CriterionEResult,
}

/// Runs the full TDI-5.2 pipeline (generation, per-block fitting,
/// aggregation, all five criteria) over an arbitrary set of population
/// specifications. Callers control scale entirely through
/// `population_specs`: the preregistered `population_specs()` output
/// requests the real 165,000-record run, while tests and the
/// termination smoke path pass tiny synthetic-scale specs instead.
/// This function is never called with the real specs anywhere in this
/// file; `run_full_experiment` remains the sole, unconditional guard.
fn run_tdi52_pipeline(
    population_specs: &[PopulationSpec],
) -> Result<Tdi52ExperimentReport, String> {
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

    let ood_width_5_refs: [&[Record]; SEED_BLOCK_COUNT] = [
        blocks[0].ood_width_5.report.records.as_slice(),
        blocks[1].ood_width_5.report.records.as_slice(),
        blocks[2].ood_width_5.report.records.as_slice(),
    ];

    let ood_width_6_refs: [&[Record]; SEED_BLOCK_COUNT] = [
        blocks[0].ood_width_6.report.records.as_slice(),
        blocks[1].ood_width_6.report.records.as_slice(),
        blocks[2].ood_width_6.report.records.as_slice(),
    ];

    let (criterion_a_comparison, criterion_a) =
        tdi52_criterion_a(&aggregate_fit, combined_holdout_refs)?;
    let (criterion_b_comparison, criterion_b) =
        tdi52_criterion_b(&aggregate_fit, combined_holdout_refs)?;
    let (criterion_c_comparison, criterion_c) =
        tdi52_criterion_c(&aggregate_fit, combined_holdout_refs)?;
    let (criterion_d_comparisons, criterion_d) =
        tdi52_criterion_d(&aggregate_fit, ood_width_5_refs, ood_width_6_refs)?;
    let (criterion_e_horizon_comparisons, criterion_e) =
        tdi52_criterion_e(&aggregate_fit, combined_holdout_refs)?;

    Ok(Tdi52ExperimentReport {
        blocks,
        aggregate_fit,
        criterion_a_comparison,
        criterion_a,
        criterion_b_comparison,
        criterion_b,
        criterion_c_comparison,
        criterion_c,
        criterion_d_comparisons,
        criterion_d,
        criterion_e_horizon_comparisons,
        criterion_e,
    })
}

fn tdi52_print_bootstrap_intervals(label: &str, intervals: Tdi52BootstrapIntervals) {
    println!();
    println!("{label}");

    print_interval(
        "  IC 95 % amélioration MSE U6 standardisée",
        intervals.standardized_mse,
    );

    print_interval(
        "  IC 95 % amélioration MSE O6 reconstruite",
        intervals.reconstructed_mse,
    );

    print_interval(
        "  IC 95 % amélioration MAE O6 reconstruite",
        intervals.reconstructed_mae,
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

fn tdi52_print_evaluations(
    population_label: &str,
    horizon_index: usize,
    evaluations: &[Tdi52LayoutEvaluation],
) {
    println!();
    println!(
        "=== {population_label} — U_{} ===",
        TARGET_HORIZONS[horizon_index]
    );

    for evaluation in evaluations {
        println!();
        println!("{}", evaluation.layout.label());

        tdi52_print_metrics("  espace U standardisé", evaluation.standardized);

        tdi52_print_metrics("  espace O reconstruit", evaluation.reconstructed);

        println!(
            "  prédictions O ramenées aux bornes : {} / {}",
            evaluation.predictions.clipped_overlap_count,
            evaluation.predictions.reconstructed_overlap.len(),
        );
    }

    let baseline = tdi52_layout_evaluation(evaluations, FeatureLayout::B0);

    let challenger = tdi52_layout_evaluation(evaluations, FeatureLayout::B12);

    println!(
        "  réduction relative MSE U B0→B12 : {:.9} %",
        tdi52_relative_reduction(baseline.standardized.mse, challenger.standardized.mse,) * 100.0
    );

    println!(
        "  amélioration MSE O B0→B12       : {:.12}",
        baseline.reconstructed.mse - challenger.reconstructed.mse
    );

    println!(
        "  amélioration MAE O B0→B12       : {:.12}",
        baseline.reconstructed.mae - challenger.reconstructed.mae
    );
}

fn tdi52_print_population_geometry(label: &str, records: &[Record]) {
    println!();
    println!("=== GÉOMÉTRIE — {label} ===");
    println!("systèmes : {}", records.len());

    for (horizon_index, &horizon) in TARGET_HORIZONS.iter().enumerate() {
        let values = target_values(records, horizon_index);
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

        let minimum = values
            .iter()
            .copied()
            .min_by(f64::total_cmp)
            .expect("non-empty population");

        let maximum = values
            .iter()
            .copied()
            .max_by(f64::total_cmp)
            .expect("non-empty population");

        println!(
            "  U_{horizon} | moyenne={mean:.12} | \
             écart-type={:.12} | min={minimum:.12} | max={maximum:.12}",
            variance.sqrt()
        );
    }
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

fn tdi52_fit_direct_models(training: &[Record]) -> Result<(RidgeModel, RidgeModel), String> {
    let horizon_index = primary_horizon_index();
    let targets = overlap_values(training, horizon_index);

    let baseline = feature_matrix(training, |record| feature_layout(record, FeatureLayout::B0));

    let challenger = feature_matrix(training, |record| {
        feature_layout(record, FeatureLayout::B12)
    });

    Ok((
        fit_ridge(&baseline, &targets)?,
        fit_ridge(&challenger, &targets)?,
    ))
}

fn tdi52_print_direct_comparator(
    label: &str,
    records: &[Record],
    baseline_model: &RidgeModel,
    challenger_model: &RidgeModel,
) {
    let horizon_index = primary_horizon_index();
    let targets = overlap_values(records, horizon_index);

    let baseline_features =
        feature_matrix(records, |record| feature_layout(record, FeatureLayout::B0));

    let challenger_features =
        feature_matrix(records, |record| feature_layout(record, FeatureLayout::B12));

    let baseline_predictions = predictions(baseline_model, &baseline_features);

    let challenger_predictions = predictions(challenger_model, &challenger_features);

    let baseline = calculate_metrics(&targets, &baseline_predictions);

    let challenger = calculate_metrics(&targets, &challenger_predictions);

    println!();
    println!("=== COMPARATEUR DIRECT O6 — {label} ===");

    tdi52_print_metrics("B0 direct", baseline);
    tdi52_print_metrics("B12 direct", challenger);

    println!(
        "  réduction relative MSE directe : {:.9} %",
        tdi52_relative_reduction(baseline.mse, challenger.mse,) * 100.0
    );
}

fn tdi52_repository_root() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(std::path::Path::to_path_buf)
        .unwrap_or_else(|| std::path::PathBuf::from("."))
}

/// Hashes a repository-relative file with `sha256sum`, matching the
/// shell-out convention already used by this workspace's frozen-hash
/// tests. Freeze-time artifacts (e.g. the scientific manifest) do not
/// exist yet while TDI-5.2 remains under implementation, so a missing
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

/// Section 17, items 1-5: git commit, compiler/Cargo versions, and the
/// SHA-256 of the evaluator, the preregistration, and the scientific
/// manifest.
fn print_tdi52_provenance() {
    println!();
    println!("=== PROVENANCE ET INTÉGRITÉ (Section 17, items 1-5) ===");
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
        "évaluateur SHA-256              : {}",
        tdi52_sha256_of_repo_file("tdi-bench/src/bin/tdi-independent-overlap-ablation-v52.rs")
    );
    println!(
        "préenregistrement SHA-256       : {}",
        tdi52_sha256_of_repo_file("docs/TDI-5.2-INDEPENDENT-OVERLAP-ABLATION-PREREGISTRATION.md")
    );
    println!(
        "manifeste scientifique SHA-256  : {}",
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
        match successor_set_space_cardinality(OOD_WIDTH_6) {
            Cardinality::Exact(value) => value.to_string(),
            other => format!("{other:?}"),
        }
    );
    println!("nombre de features baseline (B0)          : {BASELINE_FEATURE_COUNT}");
    println!("nombre de features early-overlap          : {EARLY_OVERLAP_FEATURE_COUNT}");
    println!("nombre de dispositions de modèle          : {MODEL_LAYOUT_COUNT}");
    println!("lambda ridge                              : {RIDGE_LAMBDA}");
    println!("réplicats bootstrap                       : {BOOTSTRAP_REPLICATES}");
    println!(
        "tailles de population — train w3={TRAIN_WIDTH_3_SYSTEMS}, holdout w3={HOLDOUT_WIDTH_3_SYSTEMS}, \
         train w4={TRAIN_WIDTH_4_SYSTEMS}, holdout w4={HOLDOUT_WIDTH_4_SYSTEMS}, \
         OOD w5={OOD_WIDTH_5_SYSTEMS}, OOD w6={OOD_WIDTH_6_SYSTEMS}"
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
             OOD w5={} | OOD w6={} | graine bootstrap=0x{:016X}",
            block.id.label(),
            block.training_width_3_seed,
            block.holdout_width_3_seed,
            block.training_width_4_seed,
            block.holdout_width_4_seed,
            block.ood_width_5_seed,
            block.ood_width_6_seed,
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
fn print_tdi52_aggregate_comparison(label: &str, comparison: &AggregateComparison) {
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
        comparison.aggregate_bootstrap,
    );
}

/// Section 17, items 16-17: every block-level and aggregate-level
/// sub-condition considered by each criterion, printed via `Debug` so the
/// output can never silently drift from the named fields it reflects.
fn print_tdi52_criteria_conditions(report: &Tdi52ExperimentReport) {
    println!();
    println!("=== CONDITIONS PAR CRITÈRE — niveau bloc et agrégat (Section 17, items 16-17) ===");
    println!();
    println!("TDI-5.2A : {:#?}", report.criterion_a);
    println!();
    println!("TDI-5.2B : {:#?}", report.criterion_b);
    println!();
    println!("TDI-5.2C : {:#?}", report.criterion_c);
    println!();
    println!("TDI-5.2D : {:#?}", report.criterion_d);
    println!();
    println!("TDI-5.2E : {:#?}", report.criterion_e);
}

fn tdi52_verdict_label(succeeded: bool) -> &'static str {
    if succeeded { "RÉUSSI" } else { "ÉCHOUÉ" }
}

/// Section 17, items 18-19: the final TDI-5.2A through TDI-5.2E verdicts,
/// with TDI-5.2C's four-way classification reported on its own line.
fn print_tdi52_final_verdicts(report: &Tdi52ExperimentReport) {
    println!();
    println!("=== VERDICTS FINAUX (Section 17, items 18-19) ===");
    println!(
        "TDI-5.2A — signal joint                    : {}",
        tdi52_verdict_label(report.criterion_a.succeeded())
    );
    println!(
        "TDI-5.2B — signal O2 indépendant            : {}",
        tdi52_verdict_label(report.criterion_b.succeeded())
    );
    println!(
        "TDI-5.2C — classification O1 indépendante   : {}",
        report.criterion_c.classification.label()
    );
    println!(
        "TDI-5.2D — transfert OOD (largeurs 5 et 6)  : {}",
        tdi52_verdict_label(report.criterion_d.succeeded())
    );
    println!(
        "TDI-5.2E — trajectoire multi-horizon        : {}",
        tdi52_verdict_label(report.criterion_e.succeeded())
    );
}

/// Prints the complete TDI-5.2 Section 17 "required raw output" checklist
/// (all 20 items) for a completed pipeline run. Purely a presentation
/// layer over `Tdi52ExperimentReport`: it has no scale-awareness of its
/// own, so it is exercised at tiny scale by the termination smoke path
/// and by tests. It would only ever print the real 165,000-record run's
/// output if `run_full_experiment` were wired to call `run_tdi52_pipeline`
/// with the real `population_specs()` — which it deliberately is not;
/// `run_full_experiment` remains the sole, unconditional guard.
fn print_tdi52_required_raw_output(report: &Tdi52ExperimentReport) {
    print_tdi52_provenance();
    print_tdi52_frozen_constants();
    print_tdi52_seed_block_definitions();
    print_tdi52_population_accounting(&report.blocks);

    for seed_block in FROZEN_BLOCK_ORDER {
        let fit = report.aggregate_fit.block(seed_block);

        println!();
        println!(
            "### BLOC {} — normalisations et modèles (Section 17, items 12-13) ###",
            seed_block.label()
        );
        tdi52_print_models(&fit.models, &fit.target_scalers);
    }

    print_tdi52_aggregate_comparison("TDI-5.2A — signal joint", &report.criterion_a_comparison);
    print_tdi52_aggregate_comparison(
        "TDI-5.2B — signal O2 indépendant",
        &report.criterion_b_comparison,
    );
    print_tdi52_aggregate_comparison(
        "TDI-5.2C — classification O1 indépendante",
        &report.criterion_c_comparison,
    );
    print_tdi52_aggregate_comparison(
        "TDI-5.2D — transfert OOD largeur 5",
        &report.criterion_d_comparisons.0,
    );
    print_tdi52_aggregate_comparison(
        "TDI-5.2D — transfert OOD largeur 6",
        &report.criterion_d_comparisons.1,
    );

    for horizon_comparison in &report.criterion_e_horizon_comparisons {
        print_tdi52_aggregate_comparison(
            &format!(
                "TDI-5.2E — trajectoire secondaire U_{}",
                TARGET_HORIZONS[horizon_comparison.horizon_index]
            ),
            &horizon_comparison.comparison,
        );
    }

    print_tdi52_criteria_conditions(report);
    print_tdi52_final_verdicts(report);
}

fn run_termination_smoke() -> Result<(), String> {
    println!("=== TDI-5.2 TERMINATION SMOKE ===");

    let width_6_space = successor_set_space_cardinality(OOD_WIDTH_6);

    if width_6_space != Cardinality::Exact(18_446_744_073_709_551_616_u128) {
        return Err(format!("unexpected width-6 cardinality: {width_6_space:?}"));
    }

    let limits = GenerationLimits {
        max_attempts: 64,
        no_progress_limit: 64,
    };

    let seed_reservation_count = validate_preregistered_seed_reservations()?;

    let report = generate_records_with_limits(TRAIN_WIDTH_3, TRAIN_WIDTH_3_SEED_OFFSET, 1, limits)
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
    println!(
        "rejection accounting total   : {}",
        report.rejections.total()
    );

    let specs = population_specs();

    println!(
        "population specifications   : {} deterministic entries",
        specs.len()
    );

    let identity_spec = PopulationSpec {
        target_count: 1,
        ..specs[0]
    };

    let identity_report = generate_population(identity_spec).map_err(|error| error.to_string())?;

    println!(
        "identity smoke seed block   : {}",
        identity_report.spec.seed_block.label()
    );
    println!(
        "identity smoke population   : {}",
        identity_report.spec.population.label()
    );
    println!(
        "identity smoke accepted     : {} in {} attempts",
        identity_report.report.records.len(),
        identity_report.report.attempts
    );

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
    ];

    let synthetic_training_width_4 = [Record {
        baseline: [0.2; BASELINE_FEATURE_COUNT],
        early_overlap: [0.35, 0.70],
        overlaps: [0.36; TARGET_HORIZON_COUNT],
        targets_u: [2.00, 1.90, 1.80, 1.70, 1.60],
    }];

    let block_fit = fit_block_models(
        SeedBlockId::A,
        &synthetic_training_width_3,
        &synthetic_training_width_4,
    )
    .map_err(|error| error.to_string())?;

    println!(
        "identity smoke model block  : {}",
        block_fit.seed_block.label()
    );
    println!(
        "identity smoke model count  : {}",
        block_fit.models.models.len()
    );
    println!(
        "identity smoke target mean  : {:.6}",
        block_fit.target_scalers[primary_horizon_index()].mean
    );

    let bootstrap_horizon = primary_horizon_index();
    let bootstrap_scaler = block_fit.target_scalers[bootstrap_horizon];

    let bootstrap_baseline = tdi52_predict(
        &synthetic_training_width_3,
        bootstrap_horizon,
        FeatureLayout::B0,
        block_fit.models.get(bootstrap_horizon, FeatureLayout::B0),
        bootstrap_scaler,
    )
    .map_err(|error| error.to_string())?;

    let bootstrap_challenger = tdi52_predict(
        &synthetic_training_width_3,
        bootstrap_horizon,
        FeatureLayout::B12,
        block_fit.models.get(bootstrap_horizon, FeatureLayout::B12),
        bootstrap_scaler,
    )
    .map_err(|error| error.to_string())?;

    let bootstrap_intervals = tdi52_paired_bootstrap(
        SeedBlockId::A,
        &synthetic_training_width_3,
        bootstrap_horizon,
        bootstrap_scaler,
        &bootstrap_baseline,
        &bootstrap_challenger,
    )
    .map_err(|error| error.to_string())?;

    println!(
        "identity smoke bootstrap seed: 0x{:016X}",
        SeedBlockId::A.bootstrap_seed()
    );
    println!(
        "identity smoke bootstrap CI  : [{:.6}, {:.6}]",
        bootstrap_intervals.standardized_mse.lower, bootstrap_intervals.standardized_mse.upper
    );

    let block_fit_b = fit_block_models(
        SeedBlockId::B,
        &synthetic_training_width_3,
        &synthetic_training_width_4,
    )
    .map_err(|error| error.to_string())?;

    let block_fit_c = fit_block_models(
        SeedBlockId::C,
        &synthetic_training_width_3,
        &synthetic_training_width_4,
    )
    .map_err(|error| error.to_string())?;

    let aggregate_fit = AggregateModelFit::assemble([block_fit, block_fit_b, block_fit_c])
        .map_err(|error| error.to_string())?;

    println!(
        "identity smoke aggregate     : blocks {}, {}, {}",
        aggregate_fit.block(SeedBlockId::A).seed_block.label(),
        aggregate_fit.block(SeedBlockId::B).seed_block.label(),
        aggregate_fit.block(SeedBlockId::C).seed_block.label()
    );

    let aggregate_comparison = evaluate_aggregate_comparison(
        primary_horizon_index(),
        &aggregate_fit,
        [
            synthetic_training_width_3.as_slice(),
            synthetic_training_width_3.as_slice(),
            synthetic_training_width_3.as_slice(),
        ],
        FeatureLayout::B0,
        FeatureLayout::B12,
    )
    .map_err(|error| error.to_string())?;

    println!(
        "identity smoke aggregate CI  : [{:.6}, {:.6}]",
        aggregate_comparison
            .aggregate_bootstrap
            .standardized_mse
            .lower,
        aggregate_comparison
            .aggregate_bootstrap
            .standardized_mse
            .upper
    );
    println!(
        "identity smoke pooled blocks : {} standardized, {} reconstructed baseline mean, {} reconstructed challenger mean",
        aggregate_comparison.blocks.len(),
        aggregate_comparison
            .aggregate_baseline_reconstructed
            .observed_mean,
        aggregate_comparison
            .aggregate_challenger_reconstructed
            .observed_mean
    );
    println!(
        "identity smoke pooled MSE    : baseline={:.6}, challenger={:.6}",
        aggregate_comparison.aggregate_baseline_standardized.mse,
        aggregate_comparison.aggregate_challenger_standardized.mse
    );
    println!(
        "identity smoke block A CI    : [{:.6}, {:.6}]",
        aggregate_comparison
            .block(SeedBlockId::A)
            .bootstrap
            .standardized_mse
            .lower,
        aggregate_comparison
            .block(SeedBlockId::A)
            .bootstrap
            .standardized_mse
            .upper
    );

    let criterion_a = evaluate_criterion_a(&aggregate_comparison);

    println!(
        "identity smoke criterion A   : succeeded={}",
        criterion_a.succeeded()
    );

    let (_, criterion_a_direct) = tdi52_criterion_a(
        &aggregate_fit,
        [
            synthetic_training_width_3.as_slice(),
            synthetic_training_width_3.as_slice(),
            synthetic_training_width_3.as_slice(),
        ],
    )
    .map_err(|error| error.to_string())?;

    println!(
        "identity smoke criterion A2  : succeeded={}",
        criterion_a_direct.succeeded()
    );

    let (_, criterion_b) = tdi52_criterion_b(
        &aggregate_fit,
        [
            synthetic_training_width_3.as_slice(),
            synthetic_training_width_3.as_slice(),
            synthetic_training_width_3.as_slice(),
        ],
    )
    .map_err(|error| error.to_string())?;

    println!(
        "identity smoke criterion B   : succeeded={}",
        criterion_b.succeeded()
    );

    let (_, criterion_c) = tdi52_criterion_c(
        &aggregate_fit,
        [
            synthetic_training_width_3.as_slice(),
            synthetic_training_width_3.as_slice(),
            synthetic_training_width_3.as_slice(),
        ],
    )
    .map_err(|error| error.to_string())?;

    println!(
        "identity smoke criterion C   : classification={}",
        criterion_c.classification.label()
    );

    let (_, criterion_d) = tdi52_criterion_d(
        &aggregate_fit,
        [
            synthetic_training_width_3.as_slice(),
            synthetic_training_width_3.as_slice(),
            synthetic_training_width_3.as_slice(),
        ],
        [
            synthetic_training_width_4.as_slice(),
            synthetic_training_width_4.as_slice(),
            synthetic_training_width_4.as_slice(),
        ],
    )
    .map_err(|error| error.to_string())?;

    println!(
        "identity smoke criterion D   : width5={}, width6={}, succeeded={}",
        criterion_d.width_5.succeeded(),
        criterion_d.width_6.succeeded(),
        criterion_d.succeeded()
    );

    let (_, criterion_e) = tdi52_criterion_e(
        &aggregate_fit,
        [
            synthetic_training_width_3.as_slice(),
            synthetic_training_width_3.as_slice(),
            synthetic_training_width_3.as_slice(),
        ],
    )
    .map_err(|error| error.to_string())?;

    println!(
        "identity smoke criterion E   : horizons_improving={}, u8_improves={}, succeeded={}",
        criterion_e.horizons_improving_in_every_block,
        criterion_e.u8_improves_in_every_block,
        criterion_e.succeeded()
    );

    let tiny_population_specs = population_specs().map(|spec| PopulationSpec {
        target_count: 1,
        ..spec
    });

    let pipeline_report =
        run_tdi52_pipeline(&tiny_population_specs).map_err(|error| error.to_string())?;

    println!(
        "identity smoke pipeline      : blocks={}, A={}, B={}, C={}, D={}, E={}",
        pipeline_report.blocks.len(),
        pipeline_report.criterion_a.succeeded(),
        pipeline_report.criterion_b.succeeded(),
        pipeline_report.criterion_c.classification.label(),
        pipeline_report.criterion_d.succeeded(),
        pipeline_report.criterion_e.succeeded()
    );
    println!(
        "identity smoke pipeline fit  : block A model count={}",
        pipeline_report
            .aggregate_fit
            .block(SeedBlockId::A)
            .models
            .models
            .len()
    );

    print_tdi52_required_raw_output(&pipeline_report);

    println!("bounded smoke result         : PASS");

    Ok(())
}

fn main() -> Result<(), String> {
    let arguments = std::env::args().skip(1).collect::<Vec<_>>();

    match arguments.as_slice() {
        [] => run_full_experiment(),
        [flag] if flag == "--termination-smoke" => run_termination_smoke(),
        _ => Err("usage: tdi-independent-overlap-ablation-v52 [--termination-smoke]".to_owned()),
    }
}

fn run_full_experiment() -> Result<(), String> {
    Err("TDI-5.2 full execution is disabled while the evaluator is under implementation".to_owned())
}

#[allow(dead_code)]
fn run_legacy_scaffold_full_experiment() -> Result<(), String> {
    println!("Generating preregistered TDI-5.1 width-3 training systems...");

    let GenerationReport {
        records: training_width_3,
        next_seed: training_width_3_next_seed,
        excluded: training_width_3_excluded,
        attempts: training_width_3_attempts,
        limits: training_width_3_limits,
        rejections: _,
    } = generate_records(
        TRAIN_WIDTH_3,
        TRAIN_WIDTH_3_SEED_OFFSET,
        TRAIN_WIDTH_3_SYSTEMS,
    )
    .map_err(|error| error.to_string())?;

    println!("Generating untouched TDI-5.1 width-3 holdout systems...");

    let GenerationReport {
        records: holdout_width_3,
        next_seed: holdout_width_3_next_seed,
        excluded: holdout_width_3_excluded,
        attempts: holdout_width_3_attempts,
        limits: holdout_width_3_limits,
        rejections: _,
    } = generate_records(
        TRAIN_WIDTH_3,
        HOLDOUT_WIDTH_3_SEED_OFFSET,
        HOLDOUT_WIDTH_3_SYSTEMS,
    )
    .map_err(|error| error.to_string())?;

    println!("Generating preregistered TDI-5.1 width-4 training systems...");

    let GenerationReport {
        records: training_width_4,
        next_seed: training_width_4_next_seed,
        excluded: training_width_4_excluded,
        attempts: training_width_4_attempts,
        limits: training_width_4_limits,
        rejections: _,
    } = generate_records(
        TRAIN_WIDTH_4,
        TRAIN_WIDTH_4_SEED_OFFSET,
        TRAIN_WIDTH_4_SYSTEMS,
    )
    .map_err(|error| error.to_string())?;

    println!("Generating untouched TDI-5.1 width-4 holdout systems...");

    let GenerationReport {
        records: holdout_width_4,
        next_seed: holdout_width_4_next_seed,
        excluded: holdout_width_4_excluded,
        attempts: holdout_width_4_attempts,
        limits: holdout_width_4_limits,
        rejections: _,
    } = generate_records(
        TRAIN_WIDTH_4,
        HOLDOUT_WIDTH_4_SEED_OFFSET,
        HOLDOUT_WIDTH_4_SYSTEMS,
    )
    .map_err(|error| error.to_string())?;

    println!("Generating untouched TDI-5.1 width-5 OOD systems...");

    let GenerationReport {
        records: holdout_width_5,
        next_seed: holdout_width_5_next_seed,
        excluded: holdout_width_5_excluded,
        attempts: holdout_width_5_attempts,
        limits: holdout_width_5_limits,
        rejections: _,
    } = generate_records(OOD_WIDTH_5, OOD_WIDTH_5_SEED_OFFSET, OOD_WIDTH_5_SYSTEMS)
        .map_err(|error| error.to_string())?;

    println!("Generating untouched TDI-5.1 width-6 extreme OOD systems...");

    let GenerationReport {
        records: holdout_width_6,
        next_seed: holdout_width_6_next_seed,
        excluded: holdout_width_6_excluded,
        attempts: holdout_width_6_attempts,
        limits: holdout_width_6_limits,
        rejections: _,
    } = generate_records(OOD_WIDTH_6, OOD_WIDTH_6_SEED_OFFSET, OOD_WIDTH_6_SYSTEMS)
        .map_err(|error| error.to_string())?;

    ensure_seed_ranges(&[
        (
            TRAIN_WIDTH_3_SEED_OFFSET,
            training_width_3_next_seed,
            "train w3",
        ),
        (
            HOLDOUT_WIDTH_3_SEED_OFFSET,
            holdout_width_3_next_seed,
            "holdout w3",
        ),
        (
            TRAIN_WIDTH_4_SEED_OFFSET,
            training_width_4_next_seed,
            "train w4",
        ),
        (
            HOLDOUT_WIDTH_4_SEED_OFFSET,
            holdout_width_4_next_seed,
            "holdout w4",
        ),
        (OOD_WIDTH_5_SEED_OFFSET, holdout_width_5_next_seed, "OOD w5"),
        (OOD_WIDTH_6_SEED_OFFSET, holdout_width_6_next_seed, "OOD w6"),
    ])?;

    let mut training = training_width_3.clone();
    training.extend(training_width_4.iter().cloned());

    let mut holdout_combined = holdout_width_3.clone();
    holdout_combined.extend(holdout_width_4.iter().cloned());

    let target_scalers = fit_target_scalers(&training)?;
    let models = fit_horizon_models(&training, &target_scalers)?;

    println!();
    println!("=== IDENTITÉ TDI-5.1 ===");
    println!(
        "git HEAD : {}",
        tdi52_command_output("git", &["rev-parse", "HEAD"])
    );
    println!(
        "rustc    : {}",
        tdi52_command_output("rustc", &["--version"])
    );
    println!(
        "cargo    : {}",
        tdi52_command_output("cargo", &["--version"])
    );
    println!("observation horizon : {OBSERVATION_HORIZON}");
    println!("target horizons     : {:?}", TARGET_HORIZONS);
    println!("primary horizon     : {PRIMARY_HORIZON}");
    println!("ridge lambda        : {RIDGE_LAMBDA}");
    println!("bootstrap replicates: {BOOTSTRAP_REPLICATES}");
    println!("bootstrap seed      : 0x{BOOTSTRAP_SEED:016X}");
    println!("max supported width : {MAX_SUPPORTED_WIDTH}");
    println!(
        "width 6 successor-set space : {}",
        match successor_set_space_cardinality(OOD_WIDTH_6) {
            Cardinality::Exact(value) => value.to_string(),
            other => format!("{other:?}"),
        }
    );

    println!();
    println!("=== POPULATIONS, GRAINES ET BORNES ===");

    for (label, accepted, excluded, attempts, limits, initial_seed, final_seed) in [
        (
            "train w3",
            training_width_3.len(),
            training_width_3_excluded,
            training_width_3_attempts,
            training_width_3_limits,
            TRAIN_WIDTH_3_SEED_OFFSET,
            training_width_3_next_seed,
        ),
        (
            "holdout w3",
            holdout_width_3.len(),
            holdout_width_3_excluded,
            holdout_width_3_attempts,
            holdout_width_3_limits,
            HOLDOUT_WIDTH_3_SEED_OFFSET,
            holdout_width_3_next_seed,
        ),
        (
            "train w4",
            training_width_4.len(),
            training_width_4_excluded,
            training_width_4_attempts,
            training_width_4_limits,
            TRAIN_WIDTH_4_SEED_OFFSET,
            training_width_4_next_seed,
        ),
        (
            "holdout w4",
            holdout_width_4.len(),
            holdout_width_4_excluded,
            holdout_width_4_attempts,
            holdout_width_4_limits,
            HOLDOUT_WIDTH_4_SEED_OFFSET,
            holdout_width_4_next_seed,
        ),
        (
            "OOD w5",
            holdout_width_5.len(),
            holdout_width_5_excluded,
            holdout_width_5_attempts,
            holdout_width_5_limits,
            OOD_WIDTH_5_SEED_OFFSET,
            holdout_width_5_next_seed,
        ),
        (
            "OOD w6",
            holdout_width_6.len(),
            holdout_width_6_excluded,
            holdout_width_6_attempts,
            holdout_width_6_limits,
            OOD_WIDTH_6_SEED_OFFSET,
            holdout_width_6_next_seed,
        ),
    ] {
        println!(
            "{label:12} | acceptés={accepted} | exclus={excluded} | tentatives={attempts} | \
             max_tentatives={} | seuil_sans_progrès={} | graine initiale={initial_seed} | \
             finale exclusive={final_seed}",
            limits.max_attempts, limits.no_progress_limit
        );
    }

    let populations: [(&str, &[Record]); 8] = [
        ("train combiné w3+w4", &training),
        ("holdout w3", &holdout_width_3),
        ("holdout w4", &holdout_width_4),
        ("holdout combiné w3+w4", &holdout_combined),
        ("OOD w5", &holdout_width_5),
        ("OOD extrême w6", &holdout_width_6),
        ("train w3", &training_width_3),
        ("train w4", &training_width_4),
    ];

    for &(label, records) in &populations {
        tdi52_print_population_geometry(label, records);
    }

    tdi52_print_models(&models, &target_scalers);

    let evaluation_populations: [(&str, &[Record]); 5] = [
        ("holdout w3", &holdout_width_3),
        ("holdout w4", &holdout_width_4),
        ("holdout combiné w3+w4", &holdout_combined),
        ("OOD w5", &holdout_width_5),
        ("OOD extrême w6", &holdout_width_6),
    ];

    for &(population_label, records) in &evaluation_populations {
        for horizon_index in 0..TARGET_HORIZON_COUNT {
            let evaluations =
                tdi52_evaluate_horizon(records, horizon_index, &models, &target_scalers)?;

            tdi52_print_evaluations(population_label, horizon_index, &evaluations);
        }
    }

    let primary_index = primary_horizon_index();
    let primary_scaler = target_scalers[primary_index];

    let combined_primary =
        tdi52_evaluate_horizon(&holdout_combined, primary_index, &models, &target_scalers)?;

    let width_3_primary =
        tdi52_evaluate_horizon(&holdout_width_3, primary_index, &models, &target_scalers)?;

    let width_4_primary =
        tdi52_evaluate_horizon(&holdout_width_4, primary_index, &models, &target_scalers)?;

    let width_5_primary =
        tdi52_evaluate_horizon(&holdout_width_5, primary_index, &models, &target_scalers)?;

    let width_6_primary =
        tdi52_evaluate_horizon(&holdout_width_6, primary_index, &models, &target_scalers)?;

    let combined_b0 = tdi52_layout_evaluation(&combined_primary, FeatureLayout::B0);

    let combined_b12 = tdi52_layout_evaluation(&combined_primary, FeatureLayout::B12);

    let width_3_b0 = tdi52_layout_evaluation(&width_3_primary, FeatureLayout::B0);

    let width_3_b12 = tdi52_layout_evaluation(&width_3_primary, FeatureLayout::B12);

    let width_4_b0 = tdi52_layout_evaluation(&width_4_primary, FeatureLayout::B0);

    let width_4_b12 = tdi52_layout_evaluation(&width_4_primary, FeatureLayout::B12);

    let width_5_b0 = tdi52_layout_evaluation(&width_5_primary, FeatureLayout::B0);

    let width_5_b12 = tdi52_layout_evaluation(&width_5_primary, FeatureLayout::B12);

    let width_6_b0 = tdi52_layout_evaluation(&width_6_primary, FeatureLayout::B0);

    let width_6_b12 = tdi52_layout_evaluation(&width_6_primary, FeatureLayout::B12);

    let combined_bootstrap = tdi52_paired_bootstrap(
        SeedBlockId::A,
        &holdout_combined,
        primary_index,
        primary_scaler,
        &combined_b0.predictions,
        &combined_b12.predictions,
    )?;

    let width_3_bootstrap = tdi52_paired_bootstrap(
        SeedBlockId::A,
        &holdout_width_3,
        primary_index,
        primary_scaler,
        &width_3_b0.predictions,
        &width_3_b12.predictions,
    )?;

    let width_4_bootstrap = tdi52_paired_bootstrap(
        SeedBlockId::A,
        &holdout_width_4,
        primary_index,
        primary_scaler,
        &width_4_b0.predictions,
        &width_4_b12.predictions,
    )?;

    let width_5_bootstrap = tdi52_paired_bootstrap(
        SeedBlockId::A,
        &holdout_width_5,
        primary_index,
        primary_scaler,
        &width_5_b0.predictions,
        &width_5_b12.predictions,
    )?;

    let width_6_bootstrap = tdi52_paired_bootstrap(
        SeedBlockId::A,
        &holdout_width_6,
        primary_index,
        primary_scaler,
        &width_6_b0.predictions,
        &width_6_b12.predictions,
    )?;

    println!();
    println!("=== INTERVALLES BOOTSTRAP U6 ===");

    for (label, intervals) in [
        ("holdout combiné w3+w4", combined_bootstrap),
        ("holdout w3", width_3_bootstrap),
        ("holdout w4", width_4_bootstrap),
        ("OOD principal w5", width_5_bootstrap),
        ("OOD extrême w6", width_6_bootstrap),
    ] {
        tdi52_print_bootstrap_intervals(label, intervals);
    }

    let criterion_a =
        tdi52_relative_reduction(combined_b0.standardized.mse, combined_b12.standardized.mse)
            >= 0.10
            && combined_bootstrap.standardized_mse.lower > 0.0
            && width_3_b0.standardized.mse - width_3_b12.standardized.mse > 0.0
            && width_4_b0.standardized.mse - width_4_b12.standardized.mse > 0.0
            && width_3_bootstrap.standardized_mse.lower > 0.0
            && width_4_bootstrap.standardized_mse.lower > 0.0
            && combined_b12.standardized.spearman > combined_b0.standardized.spearman
            && width_3_b12.standardized.spearman > 0.0
            && width_4_b12.standardized.spearman > 0.0
            && combined_b12.standardized.bias.abs() <= combined_b0.standardized.bias.abs() + 0.02;

    let criterion_b =
        tdi52_relative_reduction(width_5_b0.standardized.mse, width_5_b12.standardized.mse) >= 0.20
            && width_5_bootstrap.standardized_mse.lower > 0.0
            && width_5_b12.standardized.spearman > 0.0
            && width_5_b12.standardized.spearman >= width_5_b0.standardized.spearman
            && width_5_b12.standardized.r_squared > width_5_b0.standardized.r_squared
            && width_5_b12.standardized.bias.abs() < width_5_b0.standardized.bias.abs()
            && width_5_b0.reconstructed.mse - width_5_b12.reconstructed.mse > 0.0
            && width_5_b0.reconstructed.mae - width_5_b12.reconstructed.mae > 0.0;

    let criterion_c = width_6_b0.standardized.mse - width_6_b12.standardized.mse > 0.0
        && width_6_bootstrap.standardized_mse.lower > 0.0
        && width_6_b12.standardized.spearman > 0.0
        && width_6_b12.standardized.spearman >= width_6_b0.standardized.spearman
        && width_6_b12.standardized.bias.abs() <= width_6_b0.standardized.bias.abs()
        && width_6_b0.reconstructed.mse - width_6_b12.reconstructed.mse > 0.0;

    let secondary_horizons = [0_usize, 1, 2, 4];
    let mut positive_count = 0_usize;
    let mut reductions = Vec::with_capacity(4);
    let mut u8_positive = false;

    println!();
    println!("=== TRAJECTOIRE SECONDAIRE ===");

    for horizon_index in secondary_horizons {
        let evaluations =
            tdi52_evaluate_horizon(&holdout_combined, horizon_index, &models, &target_scalers)?;

        let baseline = tdi52_layout_evaluation(&evaluations, FeatureLayout::B0);

        let challenger = tdi52_layout_evaluation(&evaluations, FeatureLayout::B12);

        let delta = baseline.standardized.mse - challenger.standardized.mse;

        let reduction =
            tdi52_relative_reduction(baseline.standardized.mse, challenger.standardized.mse);

        positive_count += usize::from(delta > 0.0);
        reductions.push(reduction);

        if TARGET_HORIZONS[horizon_index] == 8 {
            u8_positive = delta > 0.0;
        }

        println!(
            "U_{} | Δ MSE={delta:.12} | réduction={:.9} %",
            TARGET_HORIZONS[horizon_index],
            reduction * 100.0,
        );
    }

    let average_reduction = reductions.iter().sum::<f64>() / reductions.len() as f64;

    let criterion_d = positive_count >= 3
        && u8_positive
        && reductions.iter().all(|reduction| *reduction >= -0.05)
        && average_reduction > 0.0;

    let (direct_baseline_model, direct_challenger_model) = tdi52_fit_direct_models(&training)?;

    println!();
    println!("=== MODÈLES DU COMPARATEUR DIRECT O6 ===");

    print_model("comparateur direct B0", &direct_baseline_model);

    print_model("comparateur direct B12", &direct_challenger_model);

    for &(label, records) in &evaluation_populations {
        tdi52_print_direct_comparator(
            label,
            records,
            &direct_baseline_model,
            &direct_challenger_model,
        );
    }

    println!();
    println!(
        "CRITÈRE PRINCIPAL TDI-5.1A : {}",
        if criterion_a { "RÉUSSI" } else { "ÉCHOUÉ" }
    );
    println!(
        "CRITÈRE TRANSFERT TDI-5.1B : {}",
        if criterion_b { "RÉUSSI" } else { "ÉCHOUÉ" }
    );
    println!(
        "CRITÈRE TRANSFERT EXTRÊME TDI-5.1C : {}",
        if criterion_c { "RÉUSSI" } else { "ÉCHOUÉ" }
    );
    println!(
        "CRITÈRE TRAJECTOIRE TDI-5.1D : {}",
        if criterion_d { "RÉUSSI" } else { "ÉCHOUÉ" }
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        BASELINE_FEATURE_COUNT, BOOTSTRAP_REPLICATES, BOOTSTRAP_SEED, ConfidenceInterval,
        DeterministicRng, EARLY_OVERLAP_FEATURE_COUNT, FeatureLayout, HOLDOUT_WIDTH_3_SEED_OFFSET,
        HOLDOUT_WIDTH_3_SYSTEMS, HOLDOUT_WIDTH_4_SEED_OFFSET, HOLDOUT_WIDTH_4_SYSTEMS, Metrics,
        OOD_WIDTH_5_SEED_OFFSET, OOD_WIDTH_5_SYSTEMS, OOD_WIDTH_6_SEED_OFFSET, OOD_WIDTH_6_SYSTEMS,
        PRIMARY_HORIZON, RIDGE_LAMBDA, Record, TARGET_HORIZON_COUNT, TARGET_HORIZONS,
        TRAIN_WIDTH_3, TRAIN_WIDTH_3_SEED_OFFSET, TRAIN_WIDTH_3_SYSTEMS, TRAIN_WIDTH_4_SEED_OFFSET,
        TRAIN_WIDTH_4_SYSTEMS, TargetScaler, average_ranks, calculate_metrics, confidence_interval,
        primary_horizon_index, splitmix64,
    };

    #[test]
    fn deterministic_rng_is_reproducible() {
        let mut first = DeterministicRng::new(BOOTSTRAP_SEED);
        let mut second = DeterministicRng::new(BOOTSTRAP_SEED);

        for _ in 0..100 {
            assert_eq!(first.next_u64(), second.next_u64());
        }
    }

    #[test]
    fn splitmix_is_deterministic() {
        assert_eq!(splitmix64(42), splitmix64(42));
        assert_ne!(splitmix64(42), splitmix64(43));
    }

    #[test]
    fn exact_successor_cardinalities_are_represented_through_width_6() {
        let expected_state_counts = [1_u128, 2, 4, 8, 16, 32, 64];
        let expected_successor_spaces = [
            2_u128,
            4,
            16,
            256,
            65_536,
            4_294_967_296,
            18_446_744_073_709_551_616,
        ];

        for (width, (&state_count, &successor_space)) in expected_state_counts
            .iter()
            .zip(&expected_successor_spaces)
            .enumerate()
        {
            let width = u8::try_from(width).expect("test width fits u8");

            assert_eq!(
                super::state_count_cardinality(width),
                super::Cardinality::Exact(state_count)
            );
            assert_eq!(
                super::successor_set_space_cardinality(width),
                super::Cardinality::Exact(successor_space)
            );
        }
    }

    #[test]
    fn width_6_successor_space_is_exact_u128() {
        assert_eq!(
            super::successor_set_space_cardinality(6),
            super::Cardinality::Exact(18_446_744_073_709_551_616_u128)
        );

        let context = super::AttemptContext::new(6, super::SEED_BLOCKS[0].ood_width_6_seed, 0);

        assert_eq!(
            super::nonempty_successor_set_count(context).expect("width 6 non-empty masks fit u64"),
            u64::MAX
        );
    }

    #[test]
    fn unsupported_widths_return_typed_errors() {
        assert_eq!(
            super::successor_set_space_cardinality(7),
            super::Cardinality::TooLarge {
                width: 7,
                exponent: 128,
            }
        );
        assert_eq!(
            super::generation_successor_set_space_cardinality(7),
            super::Cardinality::Invalid {
                width: 7,
                reason: "width is unsupported by the u64 successor-mask evaluator",
            }
        );

        let context = super::AttemptContext::new(7, 0, 0);
        let error =
            super::nonempty_successor_set_count(context).expect_err("width 7 is unsupported");

        assert_eq!(error.context, context);
        assert_eq!(error.category, super::FailureCategory::UnsupportedWidth);
    }

    #[test]
    fn arithmetic_and_structural_errors_keep_attempt_context() {
        let context = super::AttemptContext::new(3, 123, 4);

        let arithmetic = super::normalized_entropy(f64::INFINITY, context)
            .expect_err("non-finite normalized entropy is arithmetic failure");

        assert_eq!(arithmetic.context, context);
        assert_eq!(arithmetic.category, super::FailureCategory::Arithmetic);

        let structural =
            super::build_system(context, &[]).expect_err("wrong mask count is structural failure");

        assert_eq!(structural.context, context);
        assert_eq!(structural.category, super::FailureCategory::Structural);
    }

    #[test]
    fn evaluator_errors_are_not_converted_to_rejections() {
        let limits = super::GenerationLimits {
            max_attempts: 4,
            no_progress_limit: 4,
        };

        let error = super::generate_records_with_analyzer(
            TRAIN_WIDTH_3,
            TRAIN_WIDTH_3_SEED_OFFSET,
            1,
            limits,
            |context| {
                Err(super::EvaluationError::new(
                    context,
                    super::FailureCategory::Arithmetic,
                    "forced arithmetic failure",
                ))
            },
        )
        .expect_err("evaluator failure must propagate");

        match error {
            super::GenerationError::Evaluation(error) => {
                assert_eq!(error.context.width, TRAIN_WIDTH_3);
                assert_eq!(error.context.seed, TRAIN_WIDTH_3_SEED_OFFSET);
                assert_eq!(error.context.attempt_index, 0);
                assert_eq!(error.category, super::FailureCategory::Arithmetic);
            }
            other => panic!("unexpected generation error: {other:?}"),
        }
    }

    #[test]
    fn attempt_budget_exhaustion_is_deterministic() {
        let limits = super::GenerationLimits {
            max_attempts: 2,
            no_progress_limit: 10,
        };

        let error = super::generate_records_with_analyzer(
            TRAIN_WIDTH_3,
            TRAIN_WIDTH_3_SEED_OFFSET,
            1,
            limits,
            |_context| {
                Ok(super::CandidateOutcome::Rejected(
                    super::RejectionReason::InvalidObservationGeometry,
                ))
            },
        )
        .expect_err("budget must be exhausted deterministically");

        match error {
            super::GenerationError::AttemptBudgetExhausted(diagnostic) => {
                assert_eq!(diagnostic.context.width, TRAIN_WIDTH_3);
                assert_eq!(diagnostic.context.seed, TRAIN_WIDTH_3_SEED_OFFSET + 2);
                assert_eq!(diagnostic.context.attempt_index, 2);
                assert_eq!(diagnostic.progress.accepted, 0);
                assert_eq!(diagnostic.progress.excluded, 2);
                assert_eq!(diagnostic.progress.rejections.total(), 2);
                assert_eq!(
                    diagnostic
                        .progress
                        .rejections
                        .counts
                        .get(&super::RejectionReason::InvalidObservationGeometry),
                    Some(&2)
                );
                assert_eq!(diagnostic.progress.limits, limits);
            }
            other => panic!("unexpected generation error: {other:?}"),
        }
    }

    #[test]
    fn no_progress_termination_is_deterministic() {
        let limits = super::GenerationLimits {
            max_attempts: 10,
            no_progress_limit: 3,
        };

        let error = super::generate_records_with_analyzer(
            TRAIN_WIDTH_3,
            TRAIN_WIDTH_3_SEED_OFFSET,
            1,
            limits,
            |_context| {
                Ok(super::CandidateOutcome::Rejected(
                    super::RejectionReason::InvalidObservationGeometry,
                ))
            },
        )
        .expect_err("no-progress threshold must terminate deterministically");

        match error {
            super::GenerationError::NoProgress(diagnostic) => {
                assert_eq!(diagnostic.context.width, TRAIN_WIDTH_3);
                assert_eq!(diagnostic.context.seed, TRAIN_WIDTH_3_SEED_OFFSET + 2);
                assert_eq!(diagnostic.context.attempt_index, 2);
                assert_eq!(diagnostic.progress.accepted, 0);
                assert_eq!(diagnostic.progress.excluded, 3);
                assert_eq!(diagnostic.progress.rejections.total(), 3);
                assert_eq!(
                    diagnostic
                        .progress
                        .rejections
                        .counts
                        .get(&super::RejectionReason::InvalidObservationGeometry),
                    Some(&3)
                );
                assert_eq!(diagnostic.progress.limits, limits);
            }
            other => panic!("unexpected generation error: {other:?}"),
        }
    }

    #[test]
    fn rejection_accounting_is_deterministic_by_reason() {
        let limits = super::GenerationLimits {
            max_attempts: 8,
            no_progress_limit: 8,
        };

        let report = super::generate_records_with_analyzer(
            TRAIN_WIDTH_3,
            TRAIN_WIDTH_3_SEED_OFFSET,
            1,
            limits,
            |context| match context.attempt_index {
                0 => Ok(super::CandidateOutcome::Rejected(
                    super::RejectionReason::ObservationFullyRecovered,
                )),
                1 | 2 => Ok(super::CandidateOutcome::Rejected(
                    super::RejectionReason::TargetFullyRecovered { horizon: 3 },
                )),
                _ => Ok(super::CandidateOutcome::Accepted(Record {
                    baseline: [0.0; BASELINE_FEATURE_COUNT],
                    early_overlap: [0.25, 0.75],
                    overlaps: [0.5; TARGET_HORIZON_COUNT],
                    targets_u: [1.0; TARGET_HORIZON_COUNT],
                })),
            },
        )
        .expect("synthetic generation must succeed");

        assert_eq!(report.records.len(), 1);
        assert_eq!(report.attempts, 4);
        assert_eq!(report.excluded, 3);
        assert_eq!(report.rejections.total(), 3);

        assert_eq!(
            report
                .rejections
                .counts
                .get(&super::RejectionReason::ObservationFullyRecovered),
            Some(&1)
        );

        assert_eq!(
            report
                .rejections
                .counts
                .get(&super::RejectionReason::TargetFullyRecovered { horizon: 3 }),
            Some(&2)
        );

        assert_eq!(
            report.rejections.summary(),
            "observation-fully-recovered=1,target-fully-recovered-h3=2"
        );
    }

    #[test]
    fn small_record_generation_succeeds_with_bounded_limits() {
        let limits = super::GenerationLimits {
            max_attempts: 256,
            no_progress_limit: 256,
        };

        let report = super::generate_records_with_limits(
            TRAIN_WIDTH_3,
            TRAIN_WIDTH_3_SEED_OFFSET,
            1,
            limits,
        )
        .expect("width 3 should produce a small accepted record under bounded limits");

        assert_eq!(report.records.len(), 1);
        assert!(report.attempts <= limits.max_attempts);
        assert_eq!(report.excluded, report.rejections.total());
        assert_eq!(
            report.next_seed,
            TRAIN_WIDTH_3_SEED_OFFSET + u64::try_from(report.attempts).expect("attempts fit u64")
        );
    }

    #[test]
    fn frozen_tdi52_protocol_hashes_are_unchanged() {
        let expected_hashes = [
            (
                ".github/workflows/tdi5-ci.yml",
                "90b1d45625c8a13bc5dd14d6e98107a6ff9d85cca912b2e883d366a6ad9eed2c",
            ),
            (
                "docs/TDI-5-CONTINUOUS-DEFICIT-GEOMETRY-EVALUATOR.sha256",
                "708dbeab3ad3c509702d1b0f9eb749eb7f11e2be2424d15908560cafda09829a",
            ),
            (
                "docs/TDI-5-CONTINUOUS-DEFICIT-GEOMETRY-PREREGISTRATION.md",
                "8481d730ae5c47506284f67ce7d75586abb1412f617fbd88029106d7c26986ef",
            ),
            (
                "docs/TDI-5-CONTINUOUS-DEFICIT-GEOMETRY-PREREGISTRATION.sha256",
                "a674b1cbe9faa19e97bfd790e086b21bd3b4ac904394cce945b2796de6321b9a",
            ),
            (
                "docs/TDI-5-SCIENTIFIC-CODE.sha256",
                "a644945cb283af2b168d6f783b5caf9d7694887d386ace2b8f549eb1963c98d0",
            ),
            (
                "scripts/reproduce-tdi5.sh",
                "d06cde6a604f7cba9754c848881a999e680968c56afbf995a52f64ca3eb09a8d",
            ),
            (
                "tdi-bench/src/bin/tdi-continuous-deficit-geometry.rs",
                "3bfd370944fd48b1c9ef0bd106de51143bbd37c3419fe254d12baee34619532a",
            ),
        ];

        let repository_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("tdi-bench has workspace parent");

        for (relative_path, expected_hash) in expected_hashes {
            let path = repository_root.join(relative_path);
            let output = std::process::Command::new("sha256sum")
                .arg(&path)
                .output()
                .expect("sha256sum is available");

            assert!(
                output.status.success(),
                "sha256sum failed for {relative_path}: {}",
                String::from_utf8_lossy(&output.stderr)
            );

            let stdout = String::from_utf8_lossy(&output.stdout);
            let actual_hash = stdout
                .split_whitespace()
                .next()
                .expect("sha256sum prints hash");

            assert_eq!(actual_hash, expected_hash, "{relative_path} changed");
        }
    }

    #[test]
    fn exact_deficit_geometry_is_correct() {
        let ratio = tdi_core::ExactRatio::new(7, 8).expect("valid ratio");

        let transformed =
            super::exact_overlap_deficit_u(&ratio).expect("valid conditional geometry");

        assert!((transformed - 3.0).abs() < 1.0e-12);
    }

    #[test]
    fn biguint_logarithm_supports_more_than_128_bits() {
        let digits = [0_u64, 0_u64, 1_u64];

        let logarithm =
            super::biguint_log2_from_u64_digits(&digits).expect("large integer logarithm");

        assert!((logarithm - 128.0).abs() < 1.0e-12);
    }

    #[test]
    fn target_scaler_round_trips() {
        let records = [
            Record {
                baseline: [0.0; BASELINE_FEATURE_COUNT],
                early_overlap: [0.0; EARLY_OVERLAP_FEATURE_COUNT],
                overlaps: [0.5; TARGET_HORIZON_COUNT],
                targets_u: [1.0; TARGET_HORIZON_COUNT],
            },
            Record {
                baseline: [0.0; BASELINE_FEATURE_COUNT],
                early_overlap: [0.0; EARLY_OVERLAP_FEATURE_COUNT],
                overlaps: [0.75; TARGET_HORIZON_COUNT],
                targets_u: [2.0; TARGET_HORIZON_COUNT],
            },
        ];

        let scaler = TargetScaler::fit(&records, primary_horizon_index()).expect("valid scaler");
        let value = 1.75;

        assert!((scaler.unstandardize(scaler.standardize(value)) - value).abs() < 1.0e-12);
    }

    #[test]
    fn reconstruction_respects_unit_interval() {
        assert_eq!(super::tdi52_reconstruct_overlap(-1000.0), (0.0, true));

        assert_eq!(super::tdi52_reconstruct_overlap(0.0), (0.0, false));

        let (reconstructed, clipped) = super::tdi52_reconstruct_overlap(3.0);

        assert!(!clipped);
        assert!((0.0..=1.0).contains(&reconstructed));
        assert!((reconstructed - 0.875).abs() < 1.0e-12);
    }
    #[test]
    fn identity_metrics_are_exact() {
        let values = [0.1, 0.3, 0.6, 0.9];

        assert_eq!(
            calculate_metrics(&values, &values),
            Metrics {
                mse: 0.0,
                mae: 0.0,
                r_squared: 1.0,
                spearman: 1.0,
                bias: 0.0,
                observed_mean: 0.475,
                predicted_mean: 0.475,
                calibration_intercept: 0.0,
                calibration_slope: 1.0,
                zero_fraction: 0.0,
                one_fraction: 0.0,
            }
        );
    }

    #[test]
    fn ranks_handle_ties() {
        assert_eq!(
            average_ranks(&[3.0, 1.0, 1.0, 2.0]),
            vec![4.0, 1.5, 1.5, 3.0]
        );
    }

    #[test]
    fn confidence_interval_is_ordered() {
        let interval = confidence_interval(vec![3.0, 1.0, 4.0, 2.0]);

        assert!(interval.lower <= interval.median);
        assert!(interval.median <= interval.upper);

        let _ = ConfidenceInterval {
            lower: interval.lower,
            median: interval.median,
            upper: interval.upper,
        };
    }

    #[test]
    fn tdi52_predictor_storage_matches_preregistration() {
        assert_eq!(BASELINE_FEATURE_COUNT, 13);
        assert_eq!(EARLY_OVERLAP_FEATURE_COUNT, 2);
        assert_eq!(RIDGE_LAMBDA, 1.0);
    }

    #[test]
    fn tdi52_target_horizons_are_frozen() {
        assert_eq!(TARGET_HORIZONS, [3, 4, 5, 6, 8]);
    }

    #[test]
    fn tdi52_primary_horizon_is_six() {
        assert_eq!(PRIMARY_HORIZON, 6);
        assert_eq!(primary_horizon_index(), 3);
        assert_eq!(TARGET_HORIZONS[primary_horizon_index()], PRIMARY_HORIZON);
    }

    #[test]
    fn tdi52_feature_layouts_match_preregistration() {
        assert_eq!(super::MODEL_LAYOUT_COUNT, 5);
        assert_eq!(
            FeatureLayout::ALL,
            [
                FeatureLayout::B0,
                FeatureLayout::B1,
                FeatureLayout::B2,
                FeatureLayout::B12,
                FeatureLayout::BD,
            ]
        );

        assert_eq!(FeatureLayout::B0.feature_count(), 13);
        assert_eq!(FeatureLayout::B1.feature_count(), 14);
        assert_eq!(FeatureLayout::B2.feature_count(), 14);
        assert_eq!(FeatureLayout::B12.feature_count(), 15);
        assert_eq!(FeatureLayout::BD.feature_count(), 14);
    }

    #[test]
    fn tdi52_layout_vectors_have_exact_semantics() {
        let record = Record {
            baseline: [0.0; BASELINE_FEATURE_COUNT],
            early_overlap: [0.25, 0.75],
            overlaps: [0.5; TARGET_HORIZON_COUNT],
            targets_u: [1.0; TARGET_HORIZON_COUNT],
        };

        let b0 = super::feature_layout(&record, FeatureLayout::B0);
        let b1 = super::feature_layout(&record, FeatureLayout::B1);
        let b2 = super::feature_layout(&record, FeatureLayout::B2);
        let b12 = super::feature_layout(&record, FeatureLayout::B12);
        let bd = super::feature_layout(&record, FeatureLayout::BD);

        assert_eq!(b0.len(), 13);
        assert_eq!(&b1[BASELINE_FEATURE_COUNT..], &[0.25]);
        assert_eq!(&b2[BASELINE_FEATURE_COUNT..], &[0.75]);
        assert_eq!(&b12[BASELINE_FEATURE_COUNT..], &[0.25, 0.75]);
        assert_eq!(&bd[BASELINE_FEATURE_COUNT..], &[0.50]);
    }

    #[test]
    fn tdi52_delta_is_exploratory_only() {
        let record = Record {
            baseline: [0.0; BASELINE_FEATURE_COUNT],
            early_overlap: [0.25, 0.75],
            overlaps: [0.5; TARGET_HORIZON_COUNT],
            targets_u: [1.0; TARGET_HORIZON_COUNT],
        };

        for layout in [
            FeatureLayout::B0,
            FeatureLayout::B1,
            FeatureLayout::B2,
            FeatureLayout::B12,
        ] {
            let features = super::feature_layout(&record, layout);

            assert!(
                !features[BASELINE_FEATURE_COUNT..].contains(&0.50),
                "{} contient illicitement O2-O1",
                layout.label()
            );
        }

        let exploratory = super::feature_layout(&record, FeatureLayout::BD);
        assert_eq!(&exploratory[BASELINE_FEATURE_COUNT..], &[0.50]);
    }

    #[test]
    fn tdi52_bootstrap_contract_matches_preregistration() {
        assert_eq!(RIDGE_LAMBDA, 1.0);
        assert_eq!(BOOTSTRAP_REPLICATES, 4_000);
        assert_eq!(BOOTSTRAP_SEED, 0x5444_4935_3241_0001);

        assert_eq!(
            super::SEED_BLOCKS.map(|block| block.bootstrap_seed),
            [
                0x5444_4935_3241_0001,
                0x5444_4935_3242_0002,
                0x5444_4935_3243_0003,
            ]
        );

        assert_eq!(super::AGGREGATE_BOOTSTRAP_SEED, 0x5444_4935_3241_4747);
    }

    #[test]
    fn tdi52_seed_block_contract_matches_preregistration() {
        assert_eq!(TRAIN_WIDTH_3_SYSTEMS, 15_000);
        assert_eq!(TRAIN_WIDTH_4_SYSTEMS, 15_000);
        assert_eq!(HOLDOUT_WIDTH_3_SYSTEMS, 5_000);
        assert_eq!(HOLDOUT_WIDTH_4_SYSTEMS, 5_000);
        assert_eq!(OOD_WIDTH_5_SYSTEMS, 10_000);
        assert_eq!(OOD_WIDTH_6_SYSTEMS, 5_000);

        assert_eq!(super::SEED_BLOCKS.len(), 3);

        let block_a = super::SEED_BLOCKS[0];
        let block_b = super::SEED_BLOCKS[1];
        let block_c = super::SEED_BLOCKS[2];

        assert_eq!(block_a.id, super::SeedBlockId::A);
        assert_eq!(block_a.training_width_3_seed, 160_000_000);
        assert_eq!(block_a.holdout_width_3_seed, 170_000_000);
        assert_eq!(block_a.training_width_4_seed, 180_000_000);
        assert_eq!(block_a.holdout_width_4_seed, 190_000_000);
        assert_eq!(block_a.ood_width_5_seed, 200_000_000);
        assert_eq!(block_a.ood_width_6_seed, 210_000_000);

        assert_eq!(block_b.id, super::SeedBlockId::B);
        assert_eq!(block_b.training_width_3_seed, 260_000_000);
        assert_eq!(block_b.holdout_width_3_seed, 270_000_000);
        assert_eq!(block_b.training_width_4_seed, 280_000_000);
        assert_eq!(block_b.holdout_width_4_seed, 290_000_000);
        assert_eq!(block_b.ood_width_5_seed, 300_000_000);
        assert_eq!(block_b.ood_width_6_seed, 310_000_000);

        assert_eq!(block_c.id, super::SeedBlockId::C);
        assert_eq!(block_c.training_width_3_seed, 360_000_000);
        assert_eq!(block_c.holdout_width_3_seed, 370_000_000);
        assert_eq!(block_c.training_width_4_seed, 380_000_000);
        assert_eq!(block_c.holdout_width_4_seed, 390_000_000);
        assert_eq!(block_c.ood_width_5_seed, 400_000_000);
        assert_eq!(block_c.ood_width_6_seed, 410_000_000);

        assert_eq!(TRAIN_WIDTH_3_SEED_OFFSET, 160_000_000);
        assert_eq!(HOLDOUT_WIDTH_3_SEED_OFFSET, 170_000_000);
        assert_eq!(TRAIN_WIDTH_4_SEED_OFFSET, 180_000_000);
        assert_eq!(HOLDOUT_WIDTH_4_SEED_OFFSET, 190_000_000);
        assert_eq!(OOD_WIDTH_5_SEED_OFFSET, 200_000_000);
        assert_eq!(OOD_WIDTH_6_SEED_OFFSET, 210_000_000);

        assert_eq!(
            super::validate_preregistered_seed_reservations()
                .expect("all maximum-attempt seed reservations are disjoint"),
            18
        );
    }

    #[test]
    fn population_specifications_total_exactly_eighteen() {
        assert_eq!(super::population_specs().len(), 18);
        assert_eq!(super::TOTAL_SEED_RESERVATIONS, 18);
    }

    #[test]
    fn population_specifications_are_ordered_block_a_then_b_then_c() {
        let specs = super::population_specs();
        let per_block = super::POPULATIONS_PER_SEED_BLOCK;

        for spec in &specs[0..per_block] {
            assert_eq!(spec.seed_block, super::SeedBlockId::A);
        }

        for spec in &specs[per_block..2 * per_block] {
            assert_eq!(spec.seed_block, super::SeedBlockId::B);
        }

        for spec in &specs[2 * per_block..3 * per_block] {
            assert_eq!(spec.seed_block, super::SeedBlockId::C);
        }
    }

    #[test]
    fn every_seed_block_contains_exactly_six_populations() {
        let specs = super::population_specs();

        for block_id in [
            super::SeedBlockId::A,
            super::SeedBlockId::B,
            super::SeedBlockId::C,
        ] {
            let count = specs
                .iter()
                .filter(|spec| spec.seed_block == block_id)
                .count();

            assert_eq!(count, super::POPULATIONS_PER_SEED_BLOCK);
        }
    }

    #[test]
    fn every_seed_block_requests_exactly_fifty_five_thousand_accepted_records() {
        let specs = super::population_specs();

        for block_id in [
            super::SeedBlockId::A,
            super::SeedBlockId::B,
            super::SeedBlockId::C,
        ] {
            let total: usize = specs
                .iter()
                .filter(|spec| spec.seed_block == block_id)
                .map(|spec| spec.target_count)
                .sum();

            assert_eq!(total, 55_000);
        }
    }

    #[test]
    fn all_seed_blocks_together_request_exactly_165_000_accepted_records() {
        let total: usize = super::population_specs()
            .iter()
            .map(|spec| spec.target_count)
            .sum();

        assert_eq!(total, 165_000);
    }

    #[test]
    fn population_specifications_match_preregistered_width_seed_and_count() {
        let specs = super::population_specs();

        let expected = [
            (
                super::SeedBlockId::A,
                super::PopulationKind::TrainingWidth3,
                3u8,
                160_000_000u64,
                15_000usize,
            ),
            (
                super::SeedBlockId::A,
                super::PopulationKind::HoldoutWidth3,
                3,
                170_000_000,
                5_000,
            ),
            (
                super::SeedBlockId::A,
                super::PopulationKind::TrainingWidth4,
                4,
                180_000_000,
                15_000,
            ),
            (
                super::SeedBlockId::A,
                super::PopulationKind::HoldoutWidth4,
                4,
                190_000_000,
                5_000,
            ),
            (
                super::SeedBlockId::A,
                super::PopulationKind::OodWidth5,
                5,
                200_000_000,
                10_000,
            ),
            (
                super::SeedBlockId::A,
                super::PopulationKind::OodWidth6,
                6,
                210_000_000,
                5_000,
            ),
            (
                super::SeedBlockId::B,
                super::PopulationKind::TrainingWidth3,
                3,
                260_000_000,
                15_000,
            ),
            (
                super::SeedBlockId::B,
                super::PopulationKind::HoldoutWidth3,
                3,
                270_000_000,
                5_000,
            ),
            (
                super::SeedBlockId::B,
                super::PopulationKind::TrainingWidth4,
                4,
                280_000_000,
                15_000,
            ),
            (
                super::SeedBlockId::B,
                super::PopulationKind::HoldoutWidth4,
                4,
                290_000_000,
                5_000,
            ),
            (
                super::SeedBlockId::B,
                super::PopulationKind::OodWidth5,
                5,
                300_000_000,
                10_000,
            ),
            (
                super::SeedBlockId::B,
                super::PopulationKind::OodWidth6,
                6,
                310_000_000,
                5_000,
            ),
            (
                super::SeedBlockId::C,
                super::PopulationKind::TrainingWidth3,
                3,
                360_000_000,
                15_000,
            ),
            (
                super::SeedBlockId::C,
                super::PopulationKind::HoldoutWidth3,
                3,
                370_000_000,
                5_000,
            ),
            (
                super::SeedBlockId::C,
                super::PopulationKind::TrainingWidth4,
                4,
                380_000_000,
                15_000,
            ),
            (
                super::SeedBlockId::C,
                super::PopulationKind::HoldoutWidth4,
                4,
                390_000_000,
                5_000,
            ),
            (
                super::SeedBlockId::C,
                super::PopulationKind::OodWidth5,
                5,
                400_000_000,
                10_000,
            ),
            (
                super::SeedBlockId::C,
                super::PopulationKind::OodWidth6,
                6,
                410_000_000,
                5_000,
            ),
        ];

        assert_eq!(specs.len(), expected.len());

        for (spec, (block, population, width, seed, count)) in specs.iter().zip(expected) {
            assert_eq!(spec.seed_block, block);
            assert_eq!(spec.population, population);
            assert_eq!(spec.width, width);
            assert_eq!(spec.seed, seed);
            assert_eq!(spec.target_count, count);
        }
    }

    #[test]
    fn population_spec_seed_reservations_remain_pairwise_disjoint() {
        let mut ranges: Vec<(u64, u64)> = super::population_specs()
            .iter()
            .map(|spec| {
                let limits = super::preregistered_generation_limits(
                    spec.width,
                    spec.seed,
                    spec.target_count,
                )
                .expect("preregistered populations carry valid generation limits");
                let reserved = u64::try_from(limits.max_attempts).expect("attempt budget fits u64");
                let end = spec
                    .seed
                    .checked_add(reserved)
                    .expect("reserved range fits u64");

                (spec.seed, end)
            })
            .collect();

        ranges.sort_by_key(|(start, _)| *start);

        for pair in ranges.windows(2) {
            assert!(
                pair[0].1 <= pair[1].0,
                "seed reservations overlap: {:?} vs {:?}",
                pair[0],
                pair[1]
            );
        }

        assert_eq!(ranges.len(), super::TOTAL_SEED_RESERVATIONS);
    }

    #[test]
    fn find_population_spec_returns_the_matching_spec() {
        let specs = super::population_specs();

        let spec = super::find_population_spec(
            &specs,
            super::SeedBlockId::B,
            super::PopulationKind::OodWidth5,
        );

        assert_eq!(spec.seed_block, super::SeedBlockId::B);
        assert_eq!(spec.population, super::PopulationKind::OodWidth5);
        assert_eq!(spec.seed, 300_000_000);
        assert_eq!(spec.target_count, 10_000);
    }

    #[test]
    fn generate_block_populations_generates_and_routes_every_population_kind() {
        // Distinct per-kind counts double as a routing fingerprint: the
        // real analyzer (widths 3-6) is expensive, so this test calls
        // `generate_block_populations` exactly once rather than
        // recomputing each population a second time for comparison. A
        // copy-paste swap between two kinds (e.g. holdout_width_3 and
        // training_width_4) would surface as a length mismatch below.
        let tiny_specs = super::population_specs().map(|spec| super::PopulationSpec {
            target_count: match spec.population {
                super::PopulationKind::TrainingWidth3 => 1,
                super::PopulationKind::HoldoutWidth3 => 2,
                super::PopulationKind::TrainingWidth4 => 3,
                super::PopulationKind::HoldoutWidth4 => 4,
                super::PopulationKind::OodWidth5 => 1,
                super::PopulationKind::OodWidth6 => 1,
            },
            ..spec
        });

        let populations = super::generate_block_populations(super::SeedBlockId::B, &tiny_specs)
            .expect("tiny per-population counts must generate successfully");

        assert_eq!(populations.seed_block, super::SeedBlockId::B);
        assert_eq!(populations.training_width_3.report.records.len(), 1);
        assert_eq!(populations.holdout_width_3.report.records.len(), 2);
        assert_eq!(populations.training_width_4.report.records.len(), 3);
        assert_eq!(populations.holdout_width_4.report.records.len(), 4);
        assert_eq!(populations.ood_width_5.report.records.len(), 1);
        assert_eq!(populations.ood_width_6.report.records.len(), 1);
    }

    #[test]
    fn synthetic_successful_generation_preserves_block_and_population() {
        let spec = super::PopulationSpec {
            seed_block: super::SeedBlockId::B,
            population: super::PopulationKind::HoldoutWidth4,
            width: super::TRAIN_WIDTH_4,
            seed: 4_242,
            target_count: 1,
        };

        let limits = super::GenerationLimits {
            max_attempts: 4,
            no_progress_limit: 4,
        };

        let report = super::generate_population_with_analyzer(spec, limits, |_context| {
            Ok(super::CandidateOutcome::Accepted(Record {
                baseline: [0.0; BASELINE_FEATURE_COUNT],
                early_overlap: [0.1, 0.2],
                overlaps: [0.5; TARGET_HORIZON_COUNT],
                targets_u: [1.0; TARGET_HORIZON_COUNT],
            }))
        })
        .expect("synthetic analyzer must accept immediately");

        assert_eq!(report.spec.seed_block, super::SeedBlockId::B);
        assert_eq!(report.spec.population, super::PopulationKind::HoldoutWidth4);
        assert_eq!(report.spec.width, super::TRAIN_WIDTH_4);
        assert_eq!(report.spec.seed, 4_242);
        assert_eq!(report.report.records.len(), 1);
        assert_eq!(report.report.attempts, 1);
    }

    #[test]
    fn synthetic_attempt_budget_failure_preserves_block_and_population() {
        let spec = super::PopulationSpec {
            seed_block: super::SeedBlockId::A,
            population: super::PopulationKind::OodWidth5,
            width: super::OOD_WIDTH_5,
            seed: 777,
            target_count: 1,
        };

        let limits = super::GenerationLimits {
            max_attempts: 2,
            no_progress_limit: 10,
        };

        let failure = super::generate_population_with_analyzer(spec, limits, |_context| {
            Ok(super::CandidateOutcome::Rejected(
                super::RejectionReason::InvalidObservationGeometry,
            ))
        })
        .expect_err("attempt budget must be exhausted deterministically");

        assert_eq!(failure.spec.seed_block, super::SeedBlockId::A);
        assert_eq!(failure.spec.population, super::PopulationKind::OodWidth5);

        match *failure.error {
            super::GenerationError::AttemptBudgetExhausted(diagnostic) => {
                assert_eq!(diagnostic.context.width, super::OOD_WIDTH_5);
                assert_eq!(diagnostic.context.seed, 779);
                assert_eq!(diagnostic.context.attempt_index, 2);
                assert_eq!(diagnostic.progress.accepted, 0);
                assert_eq!(diagnostic.progress.excluded, 2);
            }
            other => panic!("unexpected generation error: {other:?}"),
        }
    }

    #[test]
    fn synthetic_no_progress_failure_preserves_block_and_population() {
        let spec = super::PopulationSpec {
            seed_block: super::SeedBlockId::C,
            population: super::PopulationKind::TrainingWidth3,
            width: super::TRAIN_WIDTH_3,
            seed: 555,
            target_count: 1,
        };

        let limits = super::GenerationLimits {
            max_attempts: 10,
            no_progress_limit: 3,
        };

        let failure = super::generate_population_with_analyzer(spec, limits, |_context| {
            Ok(super::CandidateOutcome::Rejected(
                super::RejectionReason::InvalidObservationGeometry,
            ))
        })
        .expect_err("no-progress threshold must terminate deterministically");

        assert_eq!(failure.spec.seed_block, super::SeedBlockId::C);
        assert_eq!(
            failure.spec.population,
            super::PopulationKind::TrainingWidth3
        );

        match *failure.error {
            super::GenerationError::NoProgress(diagnostic) => {
                assert_eq!(diagnostic.context.width, super::TRAIN_WIDTH_3);
                assert_eq!(diagnostic.context.seed, 557);
                assert_eq!(diagnostic.context.attempt_index, 2);
                assert_eq!(diagnostic.progress.accepted, 0);
                assert_eq!(diagnostic.progress.excluded, 3);
            }
            other => panic!("unexpected generation error: {other:?}"),
        }
    }

    #[test]
    fn formatted_failure_output_contains_all_preregistered_fields() {
        let spec = super::PopulationSpec {
            seed_block: super::SeedBlockId::B,
            population: super::PopulationKind::OodWidth6,
            width: super::OOD_WIDTH_6,
            seed: 999,
            target_count: 1,
        };

        let limits = super::GenerationLimits {
            max_attempts: 2,
            no_progress_limit: 10,
        };

        let failure = super::generate_population_with_analyzer(spec, limits, |_context| {
            Ok(super::CandidateOutcome::Rejected(
                super::RejectionReason::NonFiniteFeature,
            ))
        })
        .expect_err("attempt budget must be exhausted deterministically");

        let formatted = failure.to_string();

        for expected_fragment in [
            "block B",
            "population ood-w6",
            "width 6",
            "seed 1001",
            "attempt 2",
            "accepted=0",
            "excluded=2",
            "target=1",
            "max_attempts=2",
            "no_progress_limit=10",
            "attempt-budget",
            "non-finite-feature=2",
        ] {
            assert!(
                formatted.contains(expected_fragment),
                "formatted failure {formatted:?} missing {expected_fragment:?}"
            );
        }
    }

    #[test]
    fn population_evaluator_failures_are_not_converted_to_rejections() {
        let spec = super::PopulationSpec {
            seed_block: super::SeedBlockId::A,
            population: super::PopulationKind::TrainingWidth3,
            width: super::TRAIN_WIDTH_3,
            seed: 1,
            target_count: 1,
        };

        let limits = super::GenerationLimits {
            max_attempts: 4,
            no_progress_limit: 4,
        };

        let failure = super::generate_population_with_analyzer(spec, limits, |context| {
            Err(super::EvaluationError::new(
                context,
                super::FailureCategory::Arithmetic,
                "forced arithmetic failure",
            ))
        })
        .expect_err("evaluator failure must propagate");

        assert_eq!(failure.spec.seed_block, super::SeedBlockId::A);
        assert_eq!(
            failure.spec.population,
            super::PopulationKind::TrainingWidth3
        );

        match *failure.error {
            super::GenerationError::Evaluation(evaluation_error) => {
                assert_eq!(
                    evaluation_error.category,
                    super::FailureCategory::Arithmetic
                );
            }
            other => {
                panic!("evaluator failures must not become rejections or terminations: {other:?}")
            }
        }
    }

    #[test]
    fn full_execution_guard_is_unaffected_by_population_identity_additions() {
        assert_eq!(
            super::population_specs().len(),
            super::TOTAL_SEED_RESERVATIONS
        );

        let error =
            super::run_full_experiment().expect_err("full TDI-5.2 execution must remain disabled");

        assert_eq!(
            error,
            "TDI-5.2 full execution is disabled while the evaluator is under implementation"
        );
    }

    #[test]
    fn target_scaler_uses_unit_scale_for_constant_targets() {
        let record = Record {
            baseline: [0.0; BASELINE_FEATURE_COUNT],
            early_overlap: [0.0; EARLY_OVERLAP_FEATURE_COUNT],
            overlaps: [0.5; TARGET_HORIZON_COUNT],
            targets_u: [2.0; TARGET_HORIZON_COUNT],
        };

        let records = [record.clone(), record];

        let scaler = TargetScaler::fit(&records, primary_horizon_index())
            .expect("constant target must remain valid");

        assert_eq!(scaler.mean, 2.0);
        assert_eq!(scaler.scale, 1.0);
    }

    #[test]
    fn combine_width_3_and_4_preserves_width_3_then_width_4_order() {
        let width_3 = [
            Record {
                baseline: [0.0; BASELINE_FEATURE_COUNT],
                early_overlap: [0.10, 0.20],
                overlaps: [0.3; TARGET_HORIZON_COUNT],
                targets_u: [1.0; TARGET_HORIZON_COUNT],
            },
            Record {
                baseline: [0.0; BASELINE_FEATURE_COUNT],
                early_overlap: [0.15, 0.25],
                overlaps: [0.35; TARGET_HORIZON_COUNT],
                targets_u: [1.5; TARGET_HORIZON_COUNT],
            },
        ];

        let width_4 = [Record {
            baseline: [0.0; BASELINE_FEATURE_COUNT],
            early_overlap: [0.40, 0.50],
            overlaps: [0.6; TARGET_HORIZON_COUNT],
            targets_u: [2.0; TARGET_HORIZON_COUNT],
        }];

        let combined = super::combine_width_3_and_4(&width_3, &width_4);

        assert_eq!(combined.len(), 3);
        assert_eq!(combined[0].early_overlap, width_3[0].early_overlap);
        assert_eq!(combined[1].early_overlap, width_3[1].early_overlap);
        assert_eq!(combined[2].early_overlap, width_4[0].early_overlap);
    }

    fn sample_population_generation_report(
        population: super::PopulationKind,
        records: Vec<Record>,
    ) -> super::PopulationGenerationReport {
        let count = records.len();

        super::PopulationGenerationReport {
            spec: super::PopulationSpec {
                seed_block: super::SeedBlockId::A,
                population,
                width: population.width(),
                seed: 0,
                target_count: count,
            },
            report: super::GenerationReport {
                records,
                next_seed: 0,
                excluded: 0,
                rejections: super::RejectionCounts::default(),
                attempts: count,
                limits: super::GenerationLimits {
                    max_attempts: count.max(1),
                    no_progress_limit: count.max(1),
                },
            },
        }
    }

    #[test]
    fn block_populations_combined_holdout_preserves_width_3_then_width_4_order() {
        let holdout_width_3 = vec![Record {
            baseline: [0.0; BASELINE_FEATURE_COUNT],
            early_overlap: [0.11, 0.22],
            overlaps: [0.33; TARGET_HORIZON_COUNT],
            targets_u: [1.0; TARGET_HORIZON_COUNT],
        }];

        let holdout_width_4 = vec![Record {
            baseline: [0.0; BASELINE_FEATURE_COUNT],
            early_overlap: [0.44, 0.55],
            overlaps: [0.66; TARGET_HORIZON_COUNT],
            targets_u: [2.0; TARGET_HORIZON_COUNT],
        }];

        let populations = super::BlockPopulations {
            seed_block: super::SeedBlockId::A,
            training_width_3: sample_population_generation_report(
                super::PopulationKind::TrainingWidth3,
                Vec::new(),
            ),
            holdout_width_3: sample_population_generation_report(
                super::PopulationKind::HoldoutWidth3,
                holdout_width_3.clone(),
            ),
            training_width_4: sample_population_generation_report(
                super::PopulationKind::TrainingWidth4,
                Vec::new(),
            ),
            holdout_width_4: sample_population_generation_report(
                super::PopulationKind::HoldoutWidth4,
                holdout_width_4.clone(),
            ),
            ood_width_5: sample_population_generation_report(
                super::PopulationKind::OodWidth5,
                Vec::new(),
            ),
            ood_width_6: sample_population_generation_report(
                super::PopulationKind::OodWidth6,
                Vec::new(),
            ),
        };

        let combined = populations.combined_holdout();

        assert_eq!(combined.len(), 2);
        assert_eq!(combined[0].early_overlap, holdout_width_3[0].early_overlap);
        assert_eq!(combined[1].early_overlap, holdout_width_4[0].early_overlap);
    }

    #[test]
    fn fit_block_models_preserves_seed_block_identity_and_model_count() {
        let training_width_3 = [
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
        ];

        let training_width_4 = [Record {
            baseline: [0.2; BASELINE_FEATURE_COUNT],
            early_overlap: [0.35, 0.70],
            overlaps: [0.36; TARGET_HORIZON_COUNT],
            targets_u: [2.00, 1.90, 1.80, 1.70, 1.60],
        }];

        let fit =
            super::fit_block_models(super::SeedBlockId::B, &training_width_3, &training_width_4)
                .expect("tiny synthetic training set must fit");

        assert_eq!(fit.seed_block, super::SeedBlockId::B);
        assert_eq!(
            fit.models.models.len(),
            TARGET_HORIZON_COUNT * super::MODEL_LAYOUT_COUNT
        );

        for layout in FeatureLayout::ALL {
            let _ = fit.models.get(primary_horizon_index(), layout);
        }
    }

    #[test]
    fn fit_block_models_target_scaler_reflects_combined_training_population() {
        let training_width_3 = [Record {
            baseline: [0.0; BASELINE_FEATURE_COUNT],
            early_overlap: [0.1, 0.2],
            overlaps: [0.3; TARGET_HORIZON_COUNT],
            targets_u: [1.0; TARGET_HORIZON_COUNT],
        }];

        let training_width_4 = [Record {
            baseline: [0.0; BASELINE_FEATURE_COUNT],
            early_overlap: [0.4, 0.5],
            overlaps: [0.6; TARGET_HORIZON_COUNT],
            targets_u: [3.0; TARGET_HORIZON_COUNT],
        }];

        let fit =
            super::fit_block_models(super::SeedBlockId::C, &training_width_3, &training_width_4)
                .expect("tiny synthetic training set must fit");

        let expected_mean = (1.0 + 3.0) / 2.0;

        for scaler in fit.target_scalers {
            assert!((scaler.mean - expected_mean).abs() < 1.0e-12);
        }
    }

    #[test]
    fn fit_block_models_is_deterministic() {
        let training_width_3 = [Record {
            baseline: [0.0; BASELINE_FEATURE_COUNT],
            early_overlap: [0.2, 0.6],
            overlaps: [0.3; TARGET_HORIZON_COUNT],
            targets_u: [1.1, 1.2, 1.3, 1.4, 1.5],
        }];

        let training_width_4 = [Record {
            baseline: [0.3; BASELINE_FEATURE_COUNT],
            early_overlap: [0.4, 0.8],
            overlaps: [0.5; TARGET_HORIZON_COUNT],
            targets_u: [2.1, 2.2, 2.3, 2.4, 2.5],
        }];

        let first =
            super::fit_block_models(super::SeedBlockId::A, &training_width_3, &training_width_4)
                .expect("tiny synthetic training set must fit");

        let second =
            super::fit_block_models(super::SeedBlockId::A, &training_width_3, &training_width_4)
                .expect("tiny synthetic training set must fit");

        for (first_model, second_model) in first.models.models.iter().zip(&second.models.models) {
            assert_eq!(first_model.coefficients, second_model.coefficients);
        }
    }

    #[test]
    fn aggregate_model_fit_assembles_blocks_in_frozen_order() {
        let training_width_3 = [Record {
            baseline: [0.0; BASELINE_FEATURE_COUNT],
            early_overlap: [0.2, 0.6],
            overlaps: [0.3; TARGET_HORIZON_COUNT],
            targets_u: [1.1, 1.2, 1.3, 1.4, 1.5],
        }];

        let training_width_4 = [Record {
            baseline: [0.3; BASELINE_FEATURE_COUNT],
            early_overlap: [0.4, 0.8],
            overlaps: [0.5; TARGET_HORIZON_COUNT],
            targets_u: [2.1, 2.2, 2.3, 2.4, 2.5],
        }];

        let block_a =
            super::fit_block_models(super::SeedBlockId::A, &training_width_3, &training_width_4)
                .expect("tiny synthetic training set must fit");
        let block_b =
            super::fit_block_models(super::SeedBlockId::B, &training_width_3, &training_width_4)
                .expect("tiny synthetic training set must fit");
        let block_c =
            super::fit_block_models(super::SeedBlockId::C, &training_width_3, &training_width_4)
                .expect("tiny synthetic training set must fit");

        let aggregate = super::AggregateModelFit::assemble([block_a, block_b, block_c])
            .expect("blocks in frozen A, B, C order must assemble");

        assert_eq!(
            aggregate.block(super::SeedBlockId::A).seed_block,
            super::SeedBlockId::A
        );
        assert_eq!(
            aggregate.block(super::SeedBlockId::B).seed_block,
            super::SeedBlockId::B
        );
        assert_eq!(
            aggregate.block(super::SeedBlockId::C).seed_block,
            super::SeedBlockId::C
        );
    }

    #[test]
    fn aggregate_model_fit_rejects_out_of_order_blocks() {
        let training_width_3 = [Record {
            baseline: [0.0; BASELINE_FEATURE_COUNT],
            early_overlap: [0.2, 0.6],
            overlaps: [0.3; TARGET_HORIZON_COUNT],
            targets_u: [1.1, 1.2, 1.3, 1.4, 1.5],
        }];

        let training_width_4 = [Record {
            baseline: [0.3; BASELINE_FEATURE_COUNT],
            early_overlap: [0.4, 0.8],
            overlaps: [0.5; TARGET_HORIZON_COUNT],
            targets_u: [2.1, 2.2, 2.3, 2.4, 2.5],
        }];

        let block_a =
            super::fit_block_models(super::SeedBlockId::A, &training_width_3, &training_width_4)
                .expect("tiny synthetic training set must fit");
        let block_b =
            super::fit_block_models(super::SeedBlockId::B, &training_width_3, &training_width_4)
                .expect("tiny synthetic training set must fit");
        let block_c =
            super::fit_block_models(super::SeedBlockId::C, &training_width_3, &training_width_4)
                .expect("tiny synthetic training set must fit");

        let error = super::AggregateModelFit::assemble([block_b, block_a, block_c])
            .expect_err("out-of-order blocks must be rejected");

        assert!(error.contains("deterministic block order"));
    }

    #[test]
    fn aggregate_model_fit_rejects_duplicated_block() {
        let training_width_3 = [Record {
            baseline: [0.0; BASELINE_FEATURE_COUNT],
            early_overlap: [0.2, 0.6],
            overlaps: [0.3; TARGET_HORIZON_COUNT],
            targets_u: [1.1, 1.2, 1.3, 1.4, 1.5],
        }];

        let training_width_4 = [Record {
            baseline: [0.3; BASELINE_FEATURE_COUNT],
            early_overlap: [0.4, 0.8],
            overlaps: [0.5; TARGET_HORIZON_COUNT],
            targets_u: [2.1, 2.2, 2.3, 2.4, 2.5],
        }];

        let block_a_first =
            super::fit_block_models(super::SeedBlockId::A, &training_width_3, &training_width_4)
                .expect("tiny synthetic training set must fit");
        let block_a_second =
            super::fit_block_models(super::SeedBlockId::A, &training_width_3, &training_width_4)
                .expect("tiny synthetic training set must fit");
        let block_c =
            super::fit_block_models(super::SeedBlockId::C, &training_width_3, &training_width_4)
                .expect("tiny synthetic training set must fit");

        let error = super::AggregateModelFit::assemble([block_a_first, block_a_second, block_c])
            .expect_err("a duplicated block must be rejected");

        assert!(error.contains("deterministic block order"));
    }

    #[test]
    fn seed_block_bootstrap_seed_matches_seed_blocks_table() {
        assert_eq!(
            super::SeedBlockId::A.bootstrap_seed(),
            0x5444_4935_3241_0001
        );
        assert_eq!(
            super::SeedBlockId::B.bootstrap_seed(),
            0x5444_4935_3242_0002
        );
        assert_eq!(
            super::SeedBlockId::C.bootstrap_seed(),
            0x5444_4935_3243_0003
        );
    }

    #[test]
    fn seed_block_bootstrap_seeds_produce_different_draw_sequences() {
        let mut rng_a = DeterministicRng::new(super::SeedBlockId::A.bootstrap_seed());
        let mut rng_b = DeterministicRng::new(super::SeedBlockId::B.bootstrap_seed());
        let mut rng_c = DeterministicRng::new(super::SeedBlockId::C.bootstrap_seed());

        let draws_a = (0..16).map(|_| rng_a.index(1_000)).collect::<Vec<_>>();
        let draws_b = (0..16).map(|_| rng_b.index(1_000)).collect::<Vec<_>>();
        let draws_c = (0..16).map(|_| rng_c.index(1_000)).collect::<Vec<_>>();

        assert_ne!(draws_a, draws_b);
        assert_ne!(draws_b, draws_c);
        assert_ne!(draws_a, draws_c);
    }

    #[test]
    fn paired_bootstrap_uses_the_seed_blocks_own_seed() {
        const SAMPLE_COUNT: usize = 16;

        let records = (0..SAMPLE_COUNT)
            .map(|index| {
                let value = 1.0 + 0.13 * index as f64;

                Record {
                    baseline: [0.0; BASELINE_FEATURE_COUNT],
                    early_overlap: [0.1, 0.2],
                    overlaps: [0.3; TARGET_HORIZON_COUNT],
                    targets_u: [value; TARGET_HORIZON_COUNT],
                }
            })
            .collect::<Vec<_>>();

        let scaler = TargetScaler::fit(&records, primary_horizon_index()).expect("valid scaler");

        let baseline = super::Tdi52PredictionSet {
            standardized: (0..SAMPLE_COUNT).map(|index| 0.10 * index as f64).collect(),
            reconstructed_overlap: (0..SAMPLE_COUNT)
                .map(|index| 0.020 * index as f64)
                .collect(),
            clipped_overlap_count: 0,
        };

        let challenger = super::Tdi52PredictionSet {
            standardized: (0..SAMPLE_COUNT)
                .map(|index| 0.11 * index as f64 + 0.05)
                .collect(),
            reconstructed_overlap: (0..SAMPLE_COUNT)
                .map(|index| 0.021 * index as f64 + 0.01)
                .collect(),
            clipped_overlap_count: 0,
        };

        let block_a = super::tdi52_paired_bootstrap(
            super::SeedBlockId::A,
            &records,
            primary_horizon_index(),
            scaler,
            &baseline,
            &challenger,
        )
        .expect("bootstrap must succeed");

        let block_b = super::tdi52_paired_bootstrap(
            super::SeedBlockId::B,
            &records,
            primary_horizon_index(),
            scaler,
            &baseline,
            &challenger,
        )
        .expect("bootstrap must succeed");

        assert_ne!(
            block_a.standardized_mse.median, block_b.standardized_mse.median,
            "different seed blocks resampling identical data must not collapse to the same draw sequence"
        );
    }

    #[test]
    fn paired_bootstrap_is_deterministic_for_a_given_seed_block() {
        let records = [
            Record {
                baseline: [0.0; BASELINE_FEATURE_COUNT],
                early_overlap: [0.1, 0.2],
                overlaps: [0.3; TARGET_HORIZON_COUNT],
                targets_u: [1.0; TARGET_HORIZON_COUNT],
            },
            Record {
                baseline: [0.0; BASELINE_FEATURE_COUNT],
                early_overlap: [0.2, 0.3],
                overlaps: [0.4; TARGET_HORIZON_COUNT],
                targets_u: [1.5; TARGET_HORIZON_COUNT],
            },
        ];

        let scaler = TargetScaler::fit(&records, primary_horizon_index()).expect("valid scaler");

        let baseline = super::Tdi52PredictionSet {
            standardized: vec![0.0, 0.1],
            reconstructed_overlap: vec![0.3, 0.35],
            clipped_overlap_count: 0,
        };

        let challenger = super::Tdi52PredictionSet {
            standardized: vec![0.05, 0.05],
            reconstructed_overlap: vec![0.32, 0.33],
            clipped_overlap_count: 0,
        };

        let first = super::tdi52_paired_bootstrap(
            super::SeedBlockId::C,
            &records,
            primary_horizon_index(),
            scaler,
            &baseline,
            &challenger,
        )
        .expect("bootstrap must succeed");

        let second = super::tdi52_paired_bootstrap(
            super::SeedBlockId::C,
            &records,
            primary_horizon_index(),
            scaler,
            &baseline,
            &challenger,
        )
        .expect("bootstrap must succeed");

        assert_eq!(first.standardized_mse.lower, second.standardized_mse.lower);
        assert_eq!(
            first.standardized_mse.median,
            second.standardized_mse.median
        );
        assert_eq!(first.standardized_mse.upper, second.standardized_mse.upper);
    }

    #[test]
    fn aggregate_bootstrap_seed_differs_from_every_block_seed() {
        let mut aggregate_rng = DeterministicRng::new(super::AGGREGATE_BOOTSTRAP_SEED);
        let mut block_a_rng = DeterministicRng::new(super::SeedBlockId::A.bootstrap_seed());
        let mut block_b_rng = DeterministicRng::new(super::SeedBlockId::B.bootstrap_seed());
        let mut block_c_rng = DeterministicRng::new(super::SeedBlockId::C.bootstrap_seed());

        let aggregate_draws = (0..16)
            .map(|_| aggregate_rng.index(1_000))
            .collect::<Vec<_>>();
        let block_a_draws = (0..16)
            .map(|_| block_a_rng.index(1_000))
            .collect::<Vec<_>>();
        let block_b_draws = (0..16)
            .map(|_| block_b_rng.index(1_000))
            .collect::<Vec<_>>();
        let block_c_draws = (0..16)
            .map(|_| block_c_rng.index(1_000))
            .collect::<Vec<_>>();

        assert_ne!(aggregate_draws, block_a_draws);
        assert_ne!(aggregate_draws, block_b_draws);
        assert_ne!(aggregate_draws, block_c_draws);
    }

    #[test]
    fn aggregate_paired_bootstrap_rejects_out_of_order_blocks() {
        let records = [Record {
            baseline: [0.0; BASELINE_FEATURE_COUNT],
            early_overlap: [0.1, 0.2],
            overlaps: [0.3; TARGET_HORIZON_COUNT],
            targets_u: [1.0; TARGET_HORIZON_COUNT],
        }];

        let scaler = TargetScaler::fit(&records, primary_horizon_index()).expect("valid scaler");

        let predictions = super::Tdi52PredictionSet {
            standardized: vec![0.0],
            reconstructed_overlap: vec![0.3],
            clipped_overlap_count: 0,
        };

        let inputs = [
            super::BlockComparisonInputs {
                seed_block: super::SeedBlockId::B,
                records: &records,
                scaler,
                baseline: &predictions,
                challenger: &predictions,
            },
            super::BlockComparisonInputs {
                seed_block: super::SeedBlockId::A,
                records: &records,
                scaler,
                baseline: &predictions,
                challenger: &predictions,
            },
            super::BlockComparisonInputs {
                seed_block: super::SeedBlockId::C,
                records: &records,
                scaler,
                baseline: &predictions,
                challenger: &predictions,
            },
        ];

        let error = super::aggregate_paired_bootstrap(primary_horizon_index(), &inputs)
            .expect_err("out-of-order blocks must be rejected");

        assert!(error.contains("deterministic block order"));
    }

    #[test]
    fn aggregate_paired_bootstrap_is_deterministic() {
        let records = [
            Record {
                baseline: [0.0; BASELINE_FEATURE_COUNT],
                early_overlap: [0.1, 0.2],
                overlaps: [0.3; TARGET_HORIZON_COUNT],
                targets_u: [1.0; TARGET_HORIZON_COUNT],
            },
            Record {
                baseline: [0.0; BASELINE_FEATURE_COUNT],
                early_overlap: [0.2, 0.3],
                overlaps: [0.4; TARGET_HORIZON_COUNT],
                targets_u: [1.4; TARGET_HORIZON_COUNT],
            },
        ];

        let scaler = TargetScaler::fit(&records, primary_horizon_index()).expect("valid scaler");

        let baseline = super::Tdi52PredictionSet {
            standardized: vec![0.0, 0.2],
            reconstructed_overlap: vec![0.3, 0.35],
            clipped_overlap_count: 0,
        };

        let challenger = super::Tdi52PredictionSet {
            standardized: vec![0.1, 0.1],
            reconstructed_overlap: vec![0.32, 0.33],
            clipped_overlap_count: 0,
        };

        let build_inputs = || {
            [
                super::BlockComparisonInputs {
                    seed_block: super::SeedBlockId::A,
                    records: &records,
                    scaler,
                    baseline: &baseline,
                    challenger: &challenger,
                },
                super::BlockComparisonInputs {
                    seed_block: super::SeedBlockId::B,
                    records: &records,
                    scaler,
                    baseline: &baseline,
                    challenger: &challenger,
                },
                super::BlockComparisonInputs {
                    seed_block: super::SeedBlockId::C,
                    records: &records,
                    scaler,
                    baseline: &baseline,
                    challenger: &challenger,
                },
            ]
        };

        let first = super::aggregate_paired_bootstrap(primary_horizon_index(), &build_inputs())
            .expect("aggregate bootstrap must succeed");

        let second = super::aggregate_paired_bootstrap(primary_horizon_index(), &build_inputs())
            .expect("aggregate bootstrap must succeed");

        assert_eq!(first.standardized_mse.lower, second.standardized_mse.lower);
        assert_eq!(
            first.standardized_mse.median,
            second.standardized_mse.median
        );
        assert_eq!(first.standardized_mse.upper, second.standardized_mse.upper);
    }

    #[test]
    fn evaluate_aggregate_comparison_pools_all_three_blocks() {
        let training_width_3 = [Record {
            baseline: [0.0; BASELINE_FEATURE_COUNT],
            early_overlap: [0.2, 0.6],
            overlaps: [0.3; TARGET_HORIZON_COUNT],
            targets_u: [1.1, 1.2, 1.3, 1.4, 1.5],
        }];

        let training_width_4 = [Record {
            baseline: [0.3; BASELINE_FEATURE_COUNT],
            early_overlap: [0.4, 0.8],
            overlaps: [0.5; TARGET_HORIZON_COUNT],
            targets_u: [2.1, 2.2, 2.3, 2.4, 2.5],
        }];

        let block_a =
            super::fit_block_models(super::SeedBlockId::A, &training_width_3, &training_width_4)
                .expect("tiny synthetic training set must fit");
        let block_b =
            super::fit_block_models(super::SeedBlockId::B, &training_width_3, &training_width_4)
                .expect("tiny synthetic training set must fit");
        let block_c =
            super::fit_block_models(super::SeedBlockId::C, &training_width_3, &training_width_4)
                .expect("tiny synthetic training set must fit");

        let aggregate_fit = super::AggregateModelFit::assemble([block_a, block_b, block_c])
            .expect("blocks in frozen order must assemble");

        let holdout_a = vec![Record {
            baseline: [0.05; BASELINE_FEATURE_COUNT],
            early_overlap: [0.25, 0.65],
            overlaps: [0.35; TARGET_HORIZON_COUNT],
            targets_u: [1.15, 1.25, 1.35, 1.45, 1.55],
        }];

        let holdout_b = vec![Record {
            baseline: [0.15; BASELINE_FEATURE_COUNT],
            early_overlap: [0.30, 0.70],
            overlaps: [0.40; TARGET_HORIZON_COUNT],
            targets_u: [1.60, 1.65, 1.70, 1.75, 1.80],
        }];

        let holdout_c = vec![Record {
            baseline: [0.25; BASELINE_FEATURE_COUNT],
            early_overlap: [0.35, 0.75],
            overlaps: [0.45; TARGET_HORIZON_COUNT],
            targets_u: [2.00, 1.95, 1.90, 1.85, 1.80],
        }];

        let comparison = super::evaluate_aggregate_comparison(
            primary_horizon_index(),
            &aggregate_fit,
            [
                holdout_a.as_slice(),
                holdout_b.as_slice(),
                holdout_c.as_slice(),
            ],
            FeatureLayout::B0,
            FeatureLayout::B12,
        )
        .expect("tiny synthetic aggregate comparison must succeed");

        assert_eq!(
            comparison.block(super::SeedBlockId::A).seed_block,
            super::SeedBlockId::A
        );
        assert_eq!(
            comparison.block(super::SeedBlockId::B).seed_block,
            super::SeedBlockId::B
        );
        assert_eq!(
            comparison.block(super::SeedBlockId::C).seed_block,
            super::SeedBlockId::C
        );

        let pooled_count = comparison
            .blocks
            .iter()
            .map(|block| block.standardized_targets.len())
            .sum::<usize>();

        assert_eq!(pooled_count, 3);
        assert!(
            comparison
                .aggregate_baseline_standardized
                .observed_mean
                .is_finite()
        );
        assert!(
            comparison
                .aggregate_challenger_standardized
                .observed_mean
                .is_finite()
        );
        assert!(
            comparison
                .aggregate_bootstrap
                .standardized_mse
                .lower
                .is_finite()
        );
        assert!(
            comparison
                .aggregate_baseline_reconstructed
                .observed_mean
                .is_finite()
        );
        assert!(
            comparison
                .aggregate_challenger_reconstructed
                .observed_mean
                .is_finite()
        );
        assert!(
            comparison
                .block(super::SeedBlockId::A)
                .bootstrap
                .standardized_mse
                .lower
                .is_finite()
        );
    }

    fn full_metrics(mse: f64, mae: f64, spearman: f64, bias: f64) -> Metrics {
        Metrics {
            mse,
            mae,
            r_squared: 0.0,
            spearman,
            bias,
            observed_mean: 0.0,
            predicted_mean: 0.0,
            calibration_intercept: 0.0,
            calibration_slope: 1.0,
            zero_fraction: 0.0,
            one_fraction: 0.0,
        }
    }

    fn sample_metrics(mse: f64, spearman: f64, bias: f64) -> Metrics {
        full_metrics(mse, 0.0, spearman, bias)
    }

    fn sample_bootstrap_intervals(standardized_mse_lower: f64) -> super::Tdi52BootstrapIntervals {
        sample_bootstrap_intervals_with_relative(
            standardized_mse_lower,
            ConfidenceInterval {
                lower: 0.0,
                median: 0.0,
                upper: 0.0,
            },
        )
    }

    fn sample_bootstrap_intervals_with_relative(
        standardized_mse_lower: f64,
        relative_standardized_mse: ConfidenceInterval,
    ) -> super::Tdi52BootstrapIntervals {
        super::Tdi52BootstrapIntervals {
            standardized_mse: ConfidenceInterval {
                lower: standardized_mse_lower,
                median: standardized_mse_lower,
                upper: standardized_mse_lower + 0.1,
            },
            reconstructed_mse: ConfidenceInterval {
                lower: 0.0,
                median: 0.0,
                upper: 0.0,
            },
            reconstructed_mae: ConfidenceInterval {
                lower: 0.0,
                median: 0.0,
                upper: 0.0,
            },
            relative_standardized_mse,
        }
    }

    fn sample_layout_evaluation(
        layout: FeatureLayout,
        mse: f64,
        spearman: f64,
    ) -> super::Tdi52LayoutEvaluation {
        super::Tdi52LayoutEvaluation {
            layout,
            standardized: sample_metrics(mse, spearman, 0.0),
            reconstructed: sample_metrics(mse, spearman, 0.0),
            predictions: super::Tdi52PredictionSet {
                standardized: vec![],
                reconstructed_overlap: vec![],
                clipped_overlap_count: 0,
            },
        }
    }

    fn sample_block_comparison(
        seed_block: super::SeedBlockId,
        baseline: (FeatureLayout, f64, f64),
        challenger: (FeatureLayout, f64, f64),
        bootstrap_lower: f64,
    ) -> super::BlockComparison {
        let (baseline_layout, baseline_mse, baseline_spearman) = baseline;
        let (challenger_layout, challenger_mse, challenger_spearman) = challenger;

        super::BlockComparison {
            seed_block,
            standardized_targets: vec![],
            overlap_targets: vec![],
            baseline: sample_layout_evaluation(baseline_layout, baseline_mse, baseline_spearman),
            challenger: sample_layout_evaluation(
                challenger_layout,
                challenger_mse,
                challenger_spearman,
            ),
            bootstrap: sample_bootstrap_intervals(bootstrap_lower),
        }
    }

    fn sample_aggregate_comparison(
        blocks: Vec<super::BlockComparison>,
        aggregate_baseline_mse: f64,
        aggregate_challenger_mse: f64,
        aggregate_baseline_bias: f64,
        aggregate_challenger_bias: f64,
        aggregate_bootstrap_lower: f64,
    ) -> super::AggregateComparison {
        super::AggregateComparison {
            blocks,
            aggregate_baseline_standardized: sample_metrics(
                aggregate_baseline_mse,
                0.0,
                aggregate_baseline_bias,
            ),
            aggregate_challenger_standardized: sample_metrics(
                aggregate_challenger_mse,
                0.0,
                aggregate_challenger_bias,
            ),
            aggregate_baseline_reconstructed: sample_metrics(0.0, 0.0, 0.0),
            aggregate_challenger_reconstructed: sample_metrics(0.0, 0.0, 0.0),
            aggregate_bootstrap: sample_bootstrap_intervals(aggregate_bootstrap_lower),
        }
    }

    fn favorable_criterion_a_comparison() -> super::AggregateComparison {
        let blocks = [
            super::SeedBlockId::A,
            super::SeedBlockId::B,
            super::SeedBlockId::C,
        ]
        .map(|seed_block| {
            sample_block_comparison(
                seed_block,
                (FeatureLayout::B0, 1.0, 0.5),
                (FeatureLayout::B12, 0.8, 0.6),
                0.05,
            )
        })
        .to_vec();

        sample_aggregate_comparison(blocks, 1.0, 0.8, 0.01, 0.01, 0.05)
    }

    #[test]
    fn median_of_three_computes_the_middle_value() {
        assert_eq!(super::median_of_three([0.3, 0.1, 0.2]), 0.2);
        assert_eq!(super::median_of_three([-1.0, 5.0, 2.0]), 2.0);
    }

    #[test]
    fn criterion_a_succeeds_when_every_condition_holds() {
        let comparison = favorable_criterion_a_comparison();
        let result = super::evaluate_criterion_a(&comparison);

        assert!(result.lower_mse_in_every_block);
        assert!(result.block_bootstrap_lower_bounds_positive);
        assert!(result.median_relative_reduction_at_least_15_percent);
        assert!(result.aggregate_relative_reduction_at_least_15_percent);
        assert!(result.aggregate_bootstrap_lower_bound_positive);
        assert!(result.spearman_improves_in_every_block);
        assert!(result.aggregate_bias_not_worse_than_0_02);
        assert!(result.succeeded());
    }

    #[test]
    fn criterion_a_fails_when_median_relative_reduction_is_below_threshold() {
        let blocks = [
            super::SeedBlockId::A,
            super::SeedBlockId::B,
            super::SeedBlockId::C,
        ]
        .map(|seed_block| {
            sample_block_comparison(
                seed_block,
                (FeatureLayout::B0, 1.0, 0.5),
                (FeatureLayout::B12, 0.9, 0.6),
                0.05,
            )
        })
        .to_vec();

        let comparison = sample_aggregate_comparison(blocks, 1.0, 0.9, 0.01, 0.01, 0.05);
        let result = super::evaluate_criterion_a(&comparison);

        assert!(!result.median_relative_reduction_at_least_15_percent);
        assert!(!result.succeeded());
    }

    #[test]
    fn criterion_a_fails_when_a_block_bootstrap_lower_bound_is_not_positive() {
        let mut comparison = favorable_criterion_a_comparison();
        comparison.blocks[1].bootstrap.standardized_mse.lower = -0.01;

        let result = super::evaluate_criterion_a(&comparison);

        assert!(!result.block_bootstrap_lower_bounds_positive);
        assert!(!result.succeeded());
    }

    #[test]
    fn criterion_a_fails_when_aggregate_bootstrap_lower_bound_is_not_positive() {
        let mut comparison = favorable_criterion_a_comparison();
        comparison.aggregate_bootstrap.standardized_mse.lower = 0.0;

        let result = super::evaluate_criterion_a(&comparison);

        assert!(!result.aggregate_bootstrap_lower_bound_positive);
        assert!(!result.succeeded());
    }

    #[test]
    fn criterion_a_fails_when_spearman_does_not_improve_in_every_block() {
        let mut comparison = favorable_criterion_a_comparison();
        comparison.blocks[2].challenger.standardized.spearman =
            comparison.blocks[2].baseline.standardized.spearman;

        let result = super::evaluate_criterion_a(&comparison);

        assert!(!result.spearman_improves_in_every_block);
        assert!(!result.succeeded());
    }

    #[test]
    fn criterion_a_fails_when_aggregate_bias_worsens_beyond_margin() {
        let mut comparison = favorable_criterion_a_comparison();
        comparison.aggregate_challenger_standardized.bias =
            comparison.aggregate_baseline_standardized.bias + 0.03;

        let result = super::evaluate_criterion_a(&comparison);

        assert!(!result.aggregate_bias_not_worse_than_0_02);
        assert!(!result.succeeded());
    }

    fn favorable_criterion_b_comparison() -> super::AggregateComparison {
        let blocks = [
            super::SeedBlockId::A,
            super::SeedBlockId::B,
            super::SeedBlockId::C,
        ]
        .map(|seed_block| {
            sample_block_comparison(
                seed_block,
                (FeatureLayout::B1, 1.0, 0.5),
                (FeatureLayout::B12, 0.85, 0.5),
                0.05,
            )
        })
        .to_vec();

        sample_aggregate_comparison(blocks, 1.0, 0.85, 0.01, 0.01, 0.05)
    }

    #[test]
    fn criterion_b_succeeds_when_every_condition_holds() {
        let comparison = favorable_criterion_b_comparison();
        let result = super::evaluate_criterion_b(&comparison);

        assert!(result.lower_mse_in_every_block);
        assert!(result.block_bootstrap_lower_bounds_positive);
        assert!(result.median_relative_reduction_at_least_10_percent);
        assert!(result.aggregate_relative_reduction_at_least_10_percent);
        assert!(result.spearman_not_lower_in_any_block);
        assert!(result.aggregate_bias_not_worse_than_0_02);
        assert!(result.succeeded());
    }

    #[test]
    fn criterion_b_fails_when_median_relative_reduction_is_below_threshold() {
        let blocks = [
            super::SeedBlockId::A,
            super::SeedBlockId::B,
            super::SeedBlockId::C,
        ]
        .map(|seed_block| {
            sample_block_comparison(
                seed_block,
                (FeatureLayout::B1, 1.0, 0.5),
                (FeatureLayout::B12, 0.95, 0.5),
                0.05,
            )
        })
        .to_vec();

        let comparison = sample_aggregate_comparison(blocks, 1.0, 0.95, 0.01, 0.01, 0.05);
        let result = super::evaluate_criterion_b(&comparison);

        assert!(!result.median_relative_reduction_at_least_10_percent);
        assert!(!result.succeeded());
    }

    #[test]
    fn criterion_b_fails_when_a_block_bootstrap_lower_bound_is_not_positive() {
        let mut comparison = favorable_criterion_b_comparison();
        comparison.blocks[0].bootstrap.standardized_mse.lower = -0.01;

        let result = super::evaluate_criterion_b(&comparison);

        assert!(!result.block_bootstrap_lower_bounds_positive);
        assert!(!result.succeeded());
    }

    #[test]
    fn criterion_b_fails_when_spearman_is_lower_in_a_block() {
        let mut comparison = favorable_criterion_b_comparison();
        comparison.blocks[2].challenger.standardized.spearman =
            comparison.blocks[2].baseline.standardized.spearman - 0.01;

        let result = super::evaluate_criterion_b(&comparison);

        assert!(!result.spearman_not_lower_in_any_block);
        assert!(!result.succeeded());
    }

    #[test]
    fn criterion_b_fails_when_aggregate_bias_worsens_beyond_margin() {
        let mut comparison = favorable_criterion_b_comparison();
        comparison.aggregate_challenger_standardized.bias =
            comparison.aggregate_baseline_standardized.bias + 0.03;

        let result = super::evaluate_criterion_b(&comparison);

        assert!(!result.aggregate_bias_not_worse_than_0_02);
        assert!(!result.succeeded());
    }

    #[test]
    fn criterion_b_does_not_require_aggregate_bootstrap() {
        // Section 12 lists six sub-conditions, unlike A's seven: no
        // aggregate-bootstrap-lower-bound requirement. A hostile
        // aggregate interval must not affect the verdict.
        let mut comparison = favorable_criterion_b_comparison();
        comparison.aggregate_bootstrap.standardized_mse.lower = -100.0;

        let result = super::evaluate_criterion_b(&comparison);

        assert!(result.succeeded());
    }

    fn sample_block_comparison_full(
        seed_block: super::SeedBlockId,
        baseline: (FeatureLayout, f64, f64),
        challenger: (FeatureLayout, f64, f64),
        absolute_interval: ConfidenceInterval,
        relative_interval: ConfidenceInterval,
    ) -> super::BlockComparison {
        let (baseline_layout, baseline_mse, baseline_spearman) = baseline;
        let (challenger_layout, challenger_mse, challenger_spearman) = challenger;

        super::BlockComparison {
            seed_block,
            standardized_targets: vec![],
            overlap_targets: vec![],
            baseline: sample_layout_evaluation(baseline_layout, baseline_mse, baseline_spearman),
            challenger: sample_layout_evaluation(
                challenger_layout,
                challenger_mse,
                challenger_spearman,
            ),
            bootstrap: super::Tdi52BootstrapIntervals {
                standardized_mse: absolute_interval,
                reconstructed_mse: ConfidenceInterval {
                    lower: 0.0,
                    median: 0.0,
                    upper: 0.0,
                },
                reconstructed_mae: ConfidenceInterval {
                    lower: 0.0,
                    median: 0.0,
                    upper: 0.0,
                },
                relative_standardized_mse: relative_interval,
            },
        }
    }

    fn sample_aggregate_comparison_full(
        blocks: Vec<super::BlockComparison>,
        aggregate_baseline_mse: f64,
        aggregate_challenger_mse: f64,
        aggregate_absolute_interval: ConfidenceInterval,
        aggregate_relative_interval: ConfidenceInterval,
    ) -> super::AggregateComparison {
        super::AggregateComparison {
            blocks,
            aggregate_baseline_standardized: sample_metrics(aggregate_baseline_mse, 0.0, 0.0),
            aggregate_challenger_standardized: sample_metrics(aggregate_challenger_mse, 0.0, 0.0),
            aggregate_baseline_reconstructed: sample_metrics(0.0, 0.0, 0.0),
            aggregate_challenger_reconstructed: sample_metrics(0.0, 0.0, 0.0),
            aggregate_bootstrap: super::Tdi52BootstrapIntervals {
                standardized_mse: aggregate_absolute_interval,
                reconstructed_mse: ConfidenceInterval {
                    lower: 0.0,
                    median: 0.0,
                    upper: 0.0,
                },
                reconstructed_mae: ConfidenceInterval {
                    lower: 0.0,
                    median: 0.0,
                    upper: 0.0,
                },
                relative_standardized_mse: aggregate_relative_interval,
            },
        }
    }

    const ZERO_INTERVAL: ConfidenceInterval = ConfidenceInterval {
        lower: 0.0,
        median: 0.0,
        upper: 0.0,
    };

    #[test]
    fn criterion_c_classifies_beneficial_when_conditions_hold() {
        let positive_interval = ConfidenceInterval {
            lower: 0.05,
            median: 0.1,
            upper: 0.15,
        };

        let blocks = [
            super::SeedBlockId::A,
            super::SeedBlockId::B,
            super::SeedBlockId::C,
        ]
        .map(|seed_block| {
            sample_block_comparison_full(
                seed_block,
                (FeatureLayout::B2, 1.0, 0.0),
                (FeatureLayout::B12, 0.9, 0.0),
                positive_interval,
                ZERO_INTERVAL,
            )
        })
        .to_vec();

        let comparison =
            sample_aggregate_comparison_full(blocks, 1.0, 0.9, positive_interval, ZERO_INTERVAL);

        let result = super::evaluate_criterion_c(&comparison);

        assert_eq!(result.blocks_confirming_benefit, 3);
        assert!(result.aggregate_relative_improvement_at_least_2_percent);
        assert!(result.aggregate_bootstrap_lower_bound_positive);
        assert_eq!(
            result.classification,
            super::CriterionCClassification::Beneficial
        );
    }

    #[test]
    fn criterion_c_beneficial_requires_at_least_two_blocks() {
        let positive_interval = ConfidenceInterval {
            lower: 0.05,
            median: 0.1,
            upper: 0.15,
        };

        let blocks = vec![
            sample_block_comparison_full(
                super::SeedBlockId::A,
                (FeatureLayout::B2, 1.0, 0.0),
                (FeatureLayout::B12, 0.9, 0.0),
                positive_interval,
                ZERO_INTERVAL,
            ),
            sample_block_comparison_full(
                super::SeedBlockId::B,
                (FeatureLayout::B2, 1.0, 0.0),
                (FeatureLayout::B12, 0.995, 0.0),
                ZERO_INTERVAL,
                ZERO_INTERVAL,
            ),
            sample_block_comparison_full(
                super::SeedBlockId::C,
                (FeatureLayout::B2, 1.0, 0.0),
                (FeatureLayout::B12, 0.995, 0.0),
                ZERO_INTERVAL,
                ZERO_INTERVAL,
            ),
        ];

        let comparison =
            sample_aggregate_comparison_full(blocks, 1.0, 0.9, positive_interval, ZERO_INTERVAL);

        let result = super::evaluate_criterion_c(&comparison);

        assert_eq!(result.blocks_confirming_benefit, 1);
        assert_ne!(
            result.classification,
            super::CriterionCClassification::Beneficial
        );
    }

    #[test]
    fn criterion_c_classifies_equivalent_when_conditions_hold() {
        let narrow_relative_interval = ConfidenceInterval {
            lower: -0.01,
            median: 0.0,
            upper: 0.01,
        };

        let blocks = [
            super::SeedBlockId::A,
            super::SeedBlockId::B,
            super::SeedBlockId::C,
        ]
        .map(|seed_block| {
            sample_block_comparison_full(
                seed_block,
                (FeatureLayout::B2, 1.0, 0.0),
                (FeatureLayout::B12, 1.005, 0.0),
                ZERO_INTERVAL,
                narrow_relative_interval,
            )
        })
        .to_vec();

        let comparison = sample_aggregate_comparison_full(
            blocks,
            1.0,
            1.005,
            ZERO_INTERVAL,
            narrow_relative_interval,
        );

        let result = super::evaluate_criterion_c(&comparison);

        assert!(result.all_block_point_estimates_within_equivalence_margin);
        assert_eq!(result.block_intervals_within_equivalence_margin, 3);
        assert!(result.aggregate_interval_within_equivalence_margin);
        assert_eq!(
            result.classification,
            super::CriterionCClassification::Equivalent
        );
    }

    #[test]
    fn criterion_c_equivalent_checks_the_relative_interval_not_the_absolute_one() {
        // The absolute interval here is wide enough that using it by
        // mistake for the equivalence check would still look "within
        // margin" by coincidence; only the relative interval must
        // decide it, and it is deliberately outside the 2% margin.
        let wide_relative_interval = ConfidenceInterval {
            lower: -0.5,
            median: 0.0,
            upper: 0.5,
        };

        let blocks = [
            super::SeedBlockId::A,
            super::SeedBlockId::B,
            super::SeedBlockId::C,
        ]
        .map(|seed_block| {
            sample_block_comparison_full(
                seed_block,
                (FeatureLayout::B2, 1.0, 0.0),
                (FeatureLayout::B12, 1.005, 0.0),
                ZERO_INTERVAL,
                wide_relative_interval,
            )
        })
        .to_vec();

        let comparison = sample_aggregate_comparison_full(
            blocks,
            1.0,
            1.005,
            ZERO_INTERVAL,
            wide_relative_interval,
        );

        let result = super::evaluate_criterion_c(&comparison);

        assert_eq!(result.block_intervals_within_equivalence_margin, 0);
        assert!(!result.aggregate_interval_within_equivalence_margin);
        assert_ne!(
            result.classification,
            super::CriterionCClassification::Equivalent
        );
    }

    #[test]
    fn criterion_c_classifies_harmful_when_conditions_hold() {
        let negative_interval = ConfidenceInterval {
            lower: -0.15,
            median: -0.1,
            upper: -0.05,
        };

        let blocks = [
            super::SeedBlockId::A,
            super::SeedBlockId::B,
            super::SeedBlockId::C,
        ]
        .map(|seed_block| {
            sample_block_comparison_full(
                seed_block,
                (FeatureLayout::B2, 1.0, 0.0),
                (FeatureLayout::B12, 1.1, 0.0),
                negative_interval,
                ZERO_INTERVAL,
            )
        })
        .to_vec();

        let comparison =
            sample_aggregate_comparison_full(blocks, 1.0, 1.1, negative_interval, ZERO_INTERVAL);

        let result = super::evaluate_criterion_c(&comparison);

        assert_eq!(result.blocks_confirming_harm, 3);
        assert!(result.aggregate_relative_worsening_at_least_2_percent);
        assert!(result.aggregate_bootstrap_upper_bound_negative);
        assert_eq!(
            result.classification,
            super::CriterionCClassification::Harmful
        );
    }

    #[test]
    fn criterion_c_classifies_inconclusive_by_default() {
        // Point estimates sit inside the +/-2% margin (ruling out
        // beneficial/harmful), but the bootstrap intervals are too
        // wide to confirm equivalence either: neither extreme applies
        // and the interval-based equivalence check fails, so the
        // outcome must fall through to inconclusive.
        let wide_relative_interval = ConfidenceInterval {
            lower: -0.5,
            median: 0.0,
            upper: 0.5,
        };

        let blocks = [
            super::SeedBlockId::A,
            super::SeedBlockId::B,
            super::SeedBlockId::C,
        ]
        .map(|seed_block| {
            sample_block_comparison_full(
                seed_block,
                (FeatureLayout::B2, 1.0, 0.0),
                (FeatureLayout::B12, 1.0, 0.0),
                ZERO_INTERVAL,
                wide_relative_interval,
            )
        })
        .to_vec();

        let comparison = sample_aggregate_comparison_full(
            blocks,
            1.0,
            1.0,
            ZERO_INTERVAL,
            wide_relative_interval,
        );

        let result = super::evaluate_criterion_c(&comparison);

        assert_eq!(
            result.classification,
            super::CriterionCClassification::Inconclusive
        );
    }

    fn favorable_criterion_d_width_5_comparison() -> super::AggregateComparison {
        let blocks = [
            super::SeedBlockId::A,
            super::SeedBlockId::B,
            super::SeedBlockId::C,
        ]
        .map(|seed_block| {
            sample_block_comparison(
                seed_block,
                (FeatureLayout::B0, 1.0, 0.5),
                (FeatureLayout::B12, 0.7, 0.6),
                0.05,
            )
        })
        .to_vec();

        super::AggregateComparison {
            blocks,
            aggregate_baseline_standardized: full_metrics(1.0, 0.0, 0.5, 0.05),
            aggregate_challenger_standardized: full_metrics(0.7, 0.0, 0.6, 0.03),
            aggregate_baseline_reconstructed: full_metrics(0.5, 0.4, 0.0, 0.0),
            aggregate_challenger_reconstructed: full_metrics(0.3, 0.2, 0.0, 0.0),
            aggregate_bootstrap: sample_bootstrap_intervals(0.05),
        }
    }

    fn favorable_criterion_d_width_6_comparison() -> super::AggregateComparison {
        let blocks = [
            super::SeedBlockId::A,
            super::SeedBlockId::B,
            super::SeedBlockId::C,
        ]
        .map(|seed_block| {
            sample_block_comparison(
                seed_block,
                (FeatureLayout::B0, 1.0, 0.5),
                (FeatureLayout::B12, 0.8, 0.6),
                0.05,
            )
        })
        .to_vec();

        super::AggregateComparison {
            blocks,
            aggregate_baseline_standardized: full_metrics(1.0, 0.0, 0.5, 0.05),
            aggregate_challenger_standardized: full_metrics(0.8, 0.0, 0.6, 0.05),
            aggregate_baseline_reconstructed: full_metrics(0.5, 0.0, 0.0, 0.0),
            aggregate_challenger_reconstructed: full_metrics(0.3, 0.0, 0.0, 0.0),
            aggregate_bootstrap: sample_bootstrap_intervals(0.05),
        }
    }

    #[test]
    fn criterion_d_width_5_succeeds_when_every_condition_holds() {
        let comparison = favorable_criterion_d_width_5_comparison();
        let result = super::evaluate_criterion_d_width_5(&comparison);

        assert!(result.positive_mse_improvement_in_every_block);
        assert!(result.bootstrap_lower_bound_positive_in_every_block);
        assert!(result.median_relative_reduction_at_least_20_percent);
        assert!(result.positive_challenger_spearman_in_every_block);
        assert!(result.spearman_not_worse_in_every_block);
        assert!(result.aggregate_bias_strictly_lower);
        assert!(result.positive_aggregate_reconstructed_mse_improvement);
        assert!(result.positive_aggregate_reconstructed_mae_improvement);
        assert!(result.succeeded());
    }

    #[test]
    fn criterion_d_width_6_succeeds_when_every_condition_holds() {
        let comparison = favorable_criterion_d_width_6_comparison();
        let result = super::evaluate_criterion_d_width_6(&comparison);

        assert!(result.positive_mse_improvement_in_every_block);
        assert!(result.bootstrap_lower_bound_positive_in_at_least_two_blocks);
        assert!(result.aggregate_bootstrap_lower_bound_positive);
        assert!(result.positive_challenger_spearman_in_every_block);
        assert!(result.aggregate_spearman_not_worse);
        assert!(result.aggregate_bias_not_worse);
        assert!(result.positive_aggregate_reconstructed_mse_improvement);
        assert!(result.succeeded());
    }

    #[test]
    fn criterion_d_width_5_requires_bootstrap_positive_in_every_block() {
        let mut comparison = favorable_criterion_d_width_5_comparison();
        comparison.blocks[2].bootstrap.standardized_mse.lower = -0.01;

        let result = super::evaluate_criterion_d_width_5(&comparison);

        assert!(!result.bootstrap_lower_bound_positive_in_every_block);
        assert!(!result.succeeded());
    }

    #[test]
    fn criterion_d_width_6_only_requires_bootstrap_positive_in_two_blocks() {
        let mut comparison = favorable_criterion_d_width_6_comparison();
        comparison.blocks[2].bootstrap.standardized_mse.lower = -0.01;

        let result = super::evaluate_criterion_d_width_6(&comparison);

        assert!(result.bootstrap_lower_bound_positive_in_at_least_two_blocks);
        assert!(result.succeeded());
    }

    #[test]
    fn criterion_d_width_5_requires_strictly_lower_aggregate_bias() {
        let mut comparison = favorable_criterion_d_width_5_comparison();
        comparison.aggregate_challenger_standardized.bias =
            comparison.aggregate_baseline_standardized.bias;

        let result = super::evaluate_criterion_d_width_5(&comparison);

        assert!(!result.aggregate_bias_strictly_lower);
        assert!(!result.succeeded());
    }

    #[test]
    fn criterion_d_width_6_allows_equal_aggregate_bias() {
        // The favorable width-6 fixture already has equal baseline
        // and challenger bias (0.05 vs 0.05); width 5's stricter,
        // non-equal requirement is checked separately above.
        let comparison = favorable_criterion_d_width_6_comparison();
        let result = super::evaluate_criterion_d_width_6(&comparison);

        assert!(result.aggregate_bias_not_worse);
        assert!(result.succeeded());
    }

    #[test]
    fn criterion_d_width_5_checks_spearman_per_block_while_width_6_checks_aggregate_only() {
        let mut width_5 = favorable_criterion_d_width_5_comparison();
        width_5.blocks[2].challenger.standardized.spearman =
            width_5.blocks[2].baseline.standardized.spearman - 0.1;

        let width_5_result = super::evaluate_criterion_d_width_5(&width_5);

        assert!(!width_5_result.spearman_not_worse_in_every_block);
        assert!(!width_5_result.succeeded());

        let mut width_6 = favorable_criterion_d_width_6_comparison();
        width_6.blocks[2].challenger.standardized.spearman =
            width_6.blocks[2].baseline.standardized.spearman - 0.1;

        let width_6_result = super::evaluate_criterion_d_width_6(&width_6);

        assert!(width_6_result.aggregate_spearman_not_worse);
        assert!(width_6_result.succeeded());
    }

    #[test]
    fn criterion_d_succeeds_only_when_both_widths_succeed() {
        let width_5 =
            super::evaluate_criterion_d_width_5(&favorable_criterion_d_width_5_comparison());
        let width_6 =
            super::evaluate_criterion_d_width_6(&favorable_criterion_d_width_6_comparison());

        assert!(super::CriterionDResult { width_5, width_6 }.succeeded());

        let mut failing_width_5 = width_5;
        failing_width_5.positive_mse_improvement_in_every_block = false;

        assert!(
            !super::CriterionDResult {
                width_5: failing_width_5,
                width_6,
            }
            .succeeded()
        );

        let mut failing_width_6 = width_6;
        failing_width_6.aggregate_bootstrap_lower_bound_positive = false;

        assert!(
            !super::CriterionDResult {
                width_5,
                width_6: failing_width_6,
            }
            .succeeded()
        );
    }

    #[test]
    fn secondary_horizon_indices_exclude_the_primary_horizon() {
        let indices = super::secondary_horizon_indices();

        assert_eq!(indices.len(), 4);
        assert!(!indices.contains(&primary_horizon_index()));
        assert_eq!(indices, [0, 1, 2, 4]);

        for index in indices {
            assert!(index < TARGET_HORIZON_COUNT);
        }
    }

    fn sample_horizon_comparison(
        horizon_index: usize,
        baseline_mse: f64,
        challenger_mse: f64,
    ) -> super::HorizonComparison {
        let blocks = [
            super::SeedBlockId::A,
            super::SeedBlockId::B,
            super::SeedBlockId::C,
        ]
        .map(|seed_block| {
            sample_block_comparison(
                seed_block,
                (FeatureLayout::B0, baseline_mse, 0.0),
                (FeatureLayout::B12, challenger_mse, 0.0),
                0.05,
            )
        })
        .to_vec();

        let comparison =
            sample_aggregate_comparison(blocks, baseline_mse, challenger_mse, 0.0, 0.0, 0.05);

        super::HorizonComparison {
            horizon_index,
            comparison,
        }
    }

    fn favorable_criterion_e_horizon_comparisons() -> Vec<super::HorizonComparison> {
        super::secondary_horizon_indices()
            .into_iter()
            .map(|horizon_index| sample_horizon_comparison(horizon_index, 1.0, 0.9))
            .collect()
    }

    #[test]
    fn criterion_e_succeeds_when_every_condition_holds() {
        let horizon_comparisons = favorable_criterion_e_horizon_comparisons();
        let result = super::evaluate_criterion_e(&horizon_comparisons);

        assert_eq!(result.horizons_improving_in_every_block, 4);
        assert!(result.u8_improves_in_every_block);
        assert!(result.no_block_horizon_reduction_below_minus_5_percent);
        assert!(result.average_secondary_reduction_positive_in_every_block);
        assert!(result.aggregate_reduction_positive_at_every_secondary_horizon);
        assert!(result.succeeded());
    }

    #[test]
    fn criterion_e_fails_when_fewer_than_three_horizons_improve_in_every_block() {
        let mut horizon_comparisons = favorable_criterion_e_horizon_comparisons();

        // Break improvement for block A at U_3 and U_4 (positions 0
        // and 1), leaving only U_5 and U_8 improving in every block.
        for entry in horizon_comparisons.iter_mut().take(2) {
            entry.comparison.blocks[0].baseline.standardized.mse = 1.0;
            entry.comparison.blocks[0].challenger.standardized.mse = 1.01;
        }

        let result = super::evaluate_criterion_e(&horizon_comparisons);

        assert_eq!(result.horizons_improving_in_every_block, 2);
        assert!(!result.succeeded());
    }

    #[test]
    fn criterion_e_fails_when_u8_does_not_improve_in_every_block() {
        let mut horizon_comparisons = favorable_criterion_e_horizon_comparisons();

        // U_8 sits at position 3 (secondary_horizon_indices() is
        // [0, 1, 2, 4]). Breaking only it still leaves U_3, U_4, U_5
        // satisfying "at least three horizons," proving U_8 is
        // checked as its own independent, mandatory condition.
        horizon_comparisons[3].comparison.blocks[0]
            .baseline
            .standardized
            .mse = 1.0;
        horizon_comparisons[3].comparison.blocks[0]
            .challenger
            .standardized
            .mse = 1.01;

        let result = super::evaluate_criterion_e(&horizon_comparisons);

        assert_eq!(result.horizons_improving_in_every_block, 3);
        assert!(!result.u8_improves_in_every_block);
        assert!(!result.succeeded());
    }

    #[test]
    fn criterion_e_fails_when_a_block_horizon_reduction_is_below_minus_5_percent() {
        let mut horizon_comparisons = favorable_criterion_e_horizon_comparisons();
        horizon_comparisons[0].comparison.blocks[1]
            .baseline
            .standardized
            .mse = 1.0;
        horizon_comparisons[0].comparison.blocks[1]
            .challenger
            .standardized
            .mse = 1.10;

        let result = super::evaluate_criterion_e(&horizon_comparisons);

        assert!(!result.no_block_horizon_reduction_below_minus_5_percent);
        assert!(!result.succeeded());
    }

    #[test]
    fn criterion_e_fails_when_average_secondary_reduction_is_not_positive_for_a_block() {
        let mut horizon_comparisons = favorable_criterion_e_horizon_comparisons();

        // Block C sits just inside the -5% floor at U_3 (position 0,
        // satisfying condition 3 with margin to spare against
        // floating-point rounding at the exact boundary) and only
        // mildly positive elsewhere, dragging its own average
        // negative without disturbing any other block or condition.
        horizon_comparisons[0].comparison.blocks[2]
            .baseline
            .standardized
            .mse = 1.0;
        horizon_comparisons[0].comparison.blocks[2]
            .challenger
            .standardized
            .mse = 1.049;

        for entry in horizon_comparisons.iter_mut().skip(1) {
            entry.comparison.blocks[2].baseline.standardized.mse = 1.0;
            entry.comparison.blocks[2].challenger.standardized.mse = 0.99;
        }

        let result = super::evaluate_criterion_e(&horizon_comparisons);

        assert_eq!(result.horizons_improving_in_every_block, 3);
        assert!(result.u8_improves_in_every_block);
        assert!(result.no_block_horizon_reduction_below_minus_5_percent);
        assert!(!result.average_secondary_reduction_positive_in_every_block);
        assert!(!result.succeeded());
    }

    #[test]
    fn criterion_e_fails_when_aggregate_reduction_is_not_positive_at_a_secondary_horizon() {
        let mut horizon_comparisons = favorable_criterion_e_horizon_comparisons();
        horizon_comparisons[1]
            .comparison
            .aggregate_baseline_standardized
            .mse = 1.0;
        horizon_comparisons[1]
            .comparison
            .aggregate_challenger_standardized
            .mse = 1.0;

        let result = super::evaluate_criterion_e(&horizon_comparisons);

        assert!(!result.aggregate_reduction_positive_at_every_secondary_horizon);
        assert!(!result.succeeded());
    }

    fn tiny_pipeline_specs() -> [super::PopulationSpec; 18] {
        super::population_specs().map(|spec| super::PopulationSpec {
            target_count: 1,
            ..spec
        })
    }

    // `run_tdi52_pipeline` unconditionally exercises the real analyzer
    // (`analyze_seed`) for every preregistered width, and that analyzer's
    // cost grows steeply with width (empirically ~1.6s at width 5 and
    // ~11s at width 6 per accepted record, even with `target_count: 1`).
    // A single tiny run is therefore already an expensive integration
    // test; determinism of every stage it calls (generation's seed
    // indexing, ridge fitting, paired/aggregate bootstrap) already has
    // its own dedicated, cheap unit test elsewhere, so this is
    // deliberately the only test that runs the full pipeline.
    #[test]
    fn run_tdi52_pipeline_succeeds_end_to_end_at_tiny_scale() {
        let report = super::run_tdi52_pipeline(&tiny_pipeline_specs())
            .expect("tiny end-to-end pipeline run must succeed");

        assert_eq!(report.blocks.len(), super::SEED_BLOCK_COUNT);

        let seed_blocks = report
            .blocks
            .iter()
            .map(|block| block.seed_block)
            .collect::<Vec<_>>();

        assert_eq!(seed_blocks, super::FROZEN_BLOCK_ORDER.to_vec());

        for seed_block in super::FROZEN_BLOCK_ORDER {
            let fit = report.aggregate_fit.block(seed_block);

            assert_eq!(fit.seed_block, seed_block);
            assert_eq!(
                fit.models.models.len(),
                TARGET_HORIZON_COUNT * super::MODEL_LAYOUT_COUNT
            );
        }

        // All five criteria must be reachable and internally consistent
        // from a single orchestrated run, even though tiny-scale data
        // is not expected to satisfy their success conditions.
        let _ = report.criterion_a.succeeded();
        let _ = report.criterion_b.succeeded();
        let _ = report.criterion_c.classification.label();
        let _ = report.criterion_d.succeeded();

        assert!(
            report.criterion_e.horizons_improving_in_every_block < TARGET_HORIZON_COUNT,
            "criterion E cannot report more improving horizons than secondary horizons exist"
        );
        let _ = report.criterion_e.succeeded();

        // Reuses this same report rather than running the pipeline a
        // second time: the Section 17 printer is pure presentation over
        // `Tdi52ExperimentReport`, so this only needs to prove it does
        // not panic on a real (if tiny) report shape.
        super::print_tdi52_required_raw_output(&report);
    }

    #[test]
    fn tdi52_full_execution_is_disabled_during_scaffold() {
        let error = super::run_full_experiment()
            .expect_err("the unfinished TDI-5.2 full execution must remain disabled");

        assert_eq!(
            error,
            "TDI-5.2 full execution is disabled while the evaluator is under implementation"
        );
    }
}
