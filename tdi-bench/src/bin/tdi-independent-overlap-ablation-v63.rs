//! TDI-6.3 information-decomposition evaluator (how do O1 and O2 jointly
//! inform U_h?).
//!
//! This file is derived from the frozen TDI-5.6 evaluator
//! (`tdi-independent-overlap-ablation-v56.rs`): TDI-6.3 inherits TDI-5.6's
//! candidate generation, target construction and exact descriptor
//! computation verbatim, bit-exact, unmodified -- the single base generator
//! (`build_system` over uniform non-empty successor masks, widths 3 and 4);
//! observation geometry and target geometry `U_h = -log2(1 - O_h)`; the 13
//! structural/entropic baseline variables; the two early overlaps `O1, O2`;
//! the two exact contraction descriptors delta, delta_bar; the two exact
//! spectral moments s2, s3; the dense horizon grid `H = {3,4,5,6,7,8}` and
//! the focal horizons U3, U6; and the four-population-per-block
//! (training-w3, holdout-w3, training-w4, holdout-w4) seed-reservation
//! structure.
//!
//! **The single changed factor from TDI-5.6 is the confirmatory analysis
//! machinery itself** (preregistration Section 1): TDI-6.3 replaces the
//! ridge-model / four-way-classifier ablation machinery entirely with a
//! closed-form **two-source partial information decomposition (PID)** of
//! `U_h` with respect to `(O1, O2)`, computed directly from the empirical
//! joint covariance of `(O1, O2, U_h)` under a Gaussian (second-moment-only)
//! working model and Minimum-Mutual-Information (MMI) redundancy (Barrett,
//! *Phys. Rev. E* 91, 052802, 2015). TDI-6.3 does not re-test whether
//! `{O1,O2}` improves on any baseline (settled by 5.2 ... 6.5); it asks how
//! the joint predictive information `{O1,O2}` carry about `U_h` is
//! distributed between them: redundant, unique to one overlap, or
//! synergistic.
//!
//! New in TDI-6.3
//! (`docs/TDI-6.3-INFORMATION-DECOMPOSITION-PREREGISTRATION.md` Sections
//! 5-16): the PID lattice (Redundancy, Unique(O1), Unique(O2), Synergy)
//! assembled from two independently cross-checked mutual-information
//! computation methods -- a Cholesky-log-determinant path (method 1,
//! canonical) and a multiple-correlation-coefficient identity (method 2,
//! cross-check) -- the non-exact determinism discipline this requires
//! (declared FP regime, declared tolerances, tolerance-based rather than
//! byte-exact reproduction, Section 8); three fresh, independent seed blocks
//! S/T/U; a covariance-resampling bootstrap for the four PID components; and
//! the purely descriptive criteria TDI-6.3A (the decomposition at the focal
//! horizons, per block and pooled aggregate), TDI-6.3B (the decomposition
//! across the dense horizon grid, aggregate only) and TDI-6.3C (cross-block
//! dominant-component consistency). None of these criteria is a pass/fail
//! classification: TDI-6.3 uses no ridge regression, no feature layouts, and
//! no Beneficial/Equivalent/Harmful/Inconclusive classifier anywhere.
//!
//! Unlike TDI-5.6's train/holdout split, TDI-6.3 pools all four populations
//! of a block (training and holdout, widths 3 and 4) before computing that
//! block's decomposition (Section 4.5): the decomposition estimates a joint
//! covariance structure directly rather than testing out-of-sample
//! prediction error, so pooling maximizes the precision of that estimate.
//! The exact contraction/spectral descriptors are still generated (the
//! candidate machinery is inherited verbatim) and are reported as descriptive
//! context (Section 12) but are consumed by no TDI-6.3 criterion; only `O1`,
//! `O2` and `U_h` feed the decomposition.
//!
//! Frozen ancestor identities (verified at runtime and in CI): the v56
//! evaluator, the TDI-5.6 preregistration, and the full frozen chain
//! TDI-5.1 -> TDI-5.8, TDI-6.1, TDI-6.2, TDI-6.5 (every ancestor evaluator
//! and preregistration hash) are verified before any generation.
//!
//! The full run is gated behind an explicit, exact human confirmation
//! environment variable (see `run_full_experiment` and
//! `tdi63_full_run_confirmed`). No commit, test or CI run supplies that
//! token; the authoring agent never invokes `--full`.

use tdi_core::{
    Action, ExactRatio, State, TableSystem, analyze_branching_recovery, distribution_overlap,
    explore, uniform_branching_path_entropy_bits, uniform_branching_state_distribution,
};

const OBSERVATION_HORIZON: usize = 2;

// Dense target-horizon grid, inherited unchanged from TDI-5.6 (Section 10),
// so the decomposition is sampled at every integer horizon 3..=8.
const TARGET_HORIZONS: [usize; 6] = [3, 4, 5, 6, 7, 8];
const TARGET_HORIZON_COUNT: usize = TARGET_HORIZONS.len();
const PRIMARY_HORIZON: usize = 6;
const PRIMARY_HORIZON_INDEX: usize = 3;

// The two focal horizons at which TDI-6.3A/6.3C classify: U3 (near) and the
// primary U6 (Section 10).
const FOCAL_HORIZONS: [usize; 2] = [3, 6];
const FOCAL_HORIZON_COUNT: usize = FOCAL_HORIZONS.len();

const TRAIN_WIDTH_3: u8 = 3;
const TRAIN_WIDTH_4: u8 = 4;
// Widths 5 and 6 remain supported by the inherited frozen generator and its
// exact cardinality/budget machinery, but TDI-6.3 generates no populations at
// those widths (Section 7): there are no OOD populations.
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
// unchanged from TDI-5.6 Section 4.6: the Dobrushin coefficient and the mean
// pairwise total variation. Both are exact rationals, computed per candidate
// system. Reported as descriptive context (Section 12); consumed by no
// TDI-6.3 criterion.
const CONTRACTION_FEATURE_COUNT: usize = 2;
// Exact spectral moments of the one-step Noop kernel, inherited unchanged
// from TDI-5.6 Section 4.6: s2 = trace(P^2) and s3 = trace(P^3), computed per
// candidate system as closed-walk sums of unit fractions with a single final
// rounding. Reported as descriptive context (Section 12); consumed by no
// TDI-6.3 criterion.
const SPECTRAL_FEATURE_COUNT: usize = 2;

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
    S,
    T,
    U,
}

impl SeedBlockId {
    const fn label(self) -> &'static str {
        match self {
            Self::S => "S",
            Self::T => "T",
            Self::U => "U",
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

// Three fresh seed blocks S/T/U, disjoint from every prior experiment's
// blocks (preregistration Section 8: TDI-5.7 <= 2.53e9; TDI-6.1 3.0-3.23e9;
// TDI-6.2 4.0-4.23e9; TDI-6.5 5.0-6.13e9; TDI-5.8 7.0-7.81e9; TDI-6.3 starts
// at 8.0e9). `base(b) = 8_000_000_000 + b * 100_000_000` for block index
// `b in {0,1,2}`; the four populations of a block start at
// `base + {0,10,20,30} * 1_000_000` (training-w3, holdout-w3, training-w4,
// holdout-w4). New bootstrap seeds in the `0x5444_4936_3300_...` range
// (Section 11), disjoint from every prior bootstrap seed.
const SEED_BLOCKS: [SeedBlockSpec; SEED_BLOCK_COUNT] = [
    SeedBlockSpec {
        id: SeedBlockId::S,
        training_width_3_seed: 8_000_000_000,
        holdout_width_3_seed: 8_010_000_000,
        training_width_4_seed: 8_020_000_000,
        holdout_width_4_seed: 8_030_000_000,
        bootstrap_seed: 0x5444_4936_3300_0001,
    },
    SeedBlockSpec {
        id: SeedBlockId::T,
        training_width_3_seed: 8_100_000_000,
        holdout_width_3_seed: 8_110_000_000,
        training_width_4_seed: 8_120_000_000,
        holdout_width_4_seed: 8_130_000_000,
        bootstrap_seed: 0x5444_4936_3300_0002,
    },
    SeedBlockSpec {
        id: SeedBlockId::U,
        training_width_3_seed: 8_200_000_000,
        holdout_width_3_seed: 8_210_000_000,
        training_width_4_seed: 8_220_000_000,
        holdout_width_4_seed: 8_230_000_000,
        bootstrap_seed: 0x5444_4936_3300_0003,
    },
];

// Single pooled-aggregate bootstrap seed (Section 11): unlike TDI-6.5's
// per-family or TDI-5.8's per-width aggregates, TDI-6.3 has exactly one
// pooled aggregate across all three blocks.
const AGGREGATE_BOOTSTRAP_SEED: u64 = 0x5444_4936_3300_4700;

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

#[derive(Clone, Debug)]
struct Record {
    baseline: [f64; BASELINE_FEATURE_COUNT],
    early_overlap: [f64; EARLY_OVERLAP_FEATURE_COUNT],
    contraction: [f64; CONTRACTION_FEATURE_COUNT],
    spectral: [f64; SPECTRAL_FEATURE_COUNT],
    overlaps: [f64; TARGET_HORIZON_COUNT],
    targets_u: [f64; TARGET_HORIZON_COUNT],
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
    // TDI-6.3 addition (not part of the inherited TDI-5.6 candidate-
    // generation machinery above): the hard pipeline guard on a non-finite
    // partial information decomposition or bootstrap replicate. See
    // `compute_block_pid`, `compute_aggregate_pid` and
    // `guarded_pid_bootstrap` below the core module. Degeneracy is not
    // expected on genuine generator output (preregistration Section 6); this
    // variant exists so that expectation being violated fails loud, before
    // any TDI-6.3A/B/C printing, rather than silently reporting or dropping
    // a non-finite value.
    DegenerateDecomposition,
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
            Self::DegenerateDecomposition => "degenerate-decomposition",
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
/// to `f64` -- not `1.0 - overlap.as_f64()`, which would round twice and
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
/// single `as_f64()` step -- the same exactness discipline as delta, delta_bar,
/// O1 and O2. No eigenvalue, characteristic polynomial or floating-point
/// iteration is involved; both moments are exact rationals in `[0, 2^width]`.
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

fn target_values(records: &[Record], horizon_index: usize) -> Vec<f64> {
    records
        .iter()
        .map(|record| record.targets_u[horizon_index])
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
                format!("width {width} is not part of the TDI-6.3 preregistered populations"),
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
    /// All four populations of this block, pooled (preregistration Section
    /// 4.5). Unlike TDI-5.6's `combined_holdout` (which paired a training fit
    /// against a held-out test set), TDI-6.3 estimates a joint covariance
    /// directly rather than testing out-of-sample prediction, so every
    /// accepted record from every population -- training and holdout, widths
    /// 3 and 4 -- is pooled before computing that block's decomposition.
    fn combined_all_records(&self) -> Vec<Record> {
        let all_width_3 = combine_width_3_and_4(
            &self.training_width_3.report.records,
            &self.holdout_width_3.report.records,
        );
        let all_width_4 = combine_width_3_and_4(
            &self.training_width_4.report.records,
            &self.holdout_width_4.report.records,
        );

        combine_width_3_and_4(&all_width_3, &all_width_4)
    }

    /// Every population's full generation report, in `PopulationKind::ALL`
    /// order. Required-raw-output printing walks this instead of the four
    /// named fields directly. TDI-6.3 has no OOD populations (Section 7).
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

fn combine_width_3_and_4(width_3: &[Record], width_4: &[Record]) -> Vec<Record> {
    let mut combined = Vec::with_capacity(width_3.len() + width_4.len());

    combined.extend_from_slice(width_3);
    combined.extend_from_slice(width_4);

    combined
}

const FROZEN_BLOCK_ORDER: [SeedBlockId; SEED_BLOCK_COUNT] =
    [SeedBlockId::S, SeedBlockId::T, SeedBlockId::U];

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
                "requires deterministic block order S, T, U; found {} where {} was expected",
                actual.label(),
                expected.label()
            ));
        }
    }

    Ok(())
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

/// Extracts one early-overlap column (`O1` at `source_index = 0`, `O2` at
/// `source_index = 1`) across records -- the `early_overlap` analogue of the
/// frozen `target_values` above, feeding the PID core module below.
fn source_overlap_values(records: &[Record], source_index: usize) -> Vec<f64> {
    records
        .iter()
        .map(|record| record.early_overlap[source_index])
        .collect()
}

// ===== TDI-6.3 information-decomposition core module =====
// This entire module (down to the "END CORE MODULE" marker) is pre-verified,
// hand-derived numerical code. Insert it VERBATIM, character for character,
// with NO modification, no reformatting beyond what `cargo fmt` itself would
// do, and no "improvement". Every formula here was independently verified
// against two numerically distinct derivations (see the preregistration
// Section 6) before being written; do not second-guess or "simplify" any of
// it. Place it in a sensible location among the other free functions (e.g.
// right after `combine_width_3_and_4` / before the printing functions), and
// wire the pipeline/criteria/printing/tests around it as instructed
// separately.

/// Cross-method agreement tolerance and Cholesky-pivot degeneracy floor
/// (preregistration Section 6, frozen constants).
const PID_CROSS_METHOD_TOLERANCE: f64 = 1.0e-9;
const PID_DEGENERACY_PIVOT_FLOOR: f64 = 1.0e-12;

/// Bootstrap replicate count for the PID components (Section 11, inherited).
const PID_BOOTSTRAP_REPLICATES: usize = 4_000;

/// Sample means, variances and pairwise covariances of `(T, S1, S2)`,
/// accumulated in a single deterministic pass (Section 6). Uses the
/// population (divide-by-`n`) convention, matching `fit_ridge`'s scale
/// computation elsewhere in this program; every downstream mutual-
/// information/PID quantity is a function of correlations or of a ratio of
/// same-degree determinants, both of which are exactly invariant to the
/// population-vs-sample (`n` vs `n-1`) normalization choice, so this choice
/// does not affect any reported value.
#[derive(Clone, Copy, Debug)]
struct TripleCovariance {
    mean_t: f64,
    mean_s1: f64,
    mean_s2: f64,
    var_t: f64,
    var_s1: f64,
    var_s2: f64,
    cov_t_s1: f64,
    cov_t_s2: f64,
    cov_s1_s2: f64,
}

fn triple_covariance(
    target: &[f64],
    source_1: &[f64],
    source_2: &[f64],
) -> Result<TripleCovariance, String> {
    let count = target.len();

    if source_1.len() != count || source_2.len() != count {
        return Err(format!(
            "triple_covariance requires equal-length inputs: target={count}, \
             source_1={}, source_2={}",
            source_1.len(),
            source_2.len()
        ));
    }

    if count < 2 {
        return Err(format!(
            "triple_covariance requires at least 2 records, found {count}"
        ));
    }

    let n = count as f64;

    let mean_t = target.iter().sum::<f64>() / n;
    let mean_s1 = source_1.iter().sum::<f64>() / n;
    let mean_s2 = source_2.iter().sum::<f64>() / n;

    let mut var_t = 0.0_f64;
    let mut var_s1 = 0.0_f64;
    let mut var_s2 = 0.0_f64;
    let mut cov_t_s1 = 0.0_f64;
    let mut cov_t_s2 = 0.0_f64;
    let mut cov_s1_s2 = 0.0_f64;

    for index in 0..count {
        let dt = target[index] - mean_t;
        let d1 = source_1[index] - mean_s1;
        let d2 = source_2[index] - mean_s2;

        var_t += dt * dt;
        var_s1 += d1 * d1;
        var_s2 += d2 * d2;
        cov_t_s1 += dt * d1;
        cov_t_s2 += dt * d2;
        cov_s1_s2 += d1 * d2;
    }

    Ok(TripleCovariance {
        mean_t,
        mean_s1,
        mean_s2,
        var_t: var_t / n,
        var_s1: var_s1 / n,
        var_s2: var_s2 / n,
        cov_t_s1: cov_t_s1 / n,
        cov_t_s2: cov_t_s2 / n,
        cov_s1_s2: cov_s1_s2 / n,
    })
}

/// Squared Pearson correlation from a covariance and two variances, clamped
/// to `[0,1]`; `NaN` if either variance is at or below the degeneracy floor.
fn squared_correlation(covariance: f64, variance_a: f64, variance_b: f64) -> f64 {
    if variance_a <= PID_DEGENERACY_PIVOT_FLOOR || variance_b <= PID_DEGENERACY_PIVOT_FLOOR {
        return f64::NAN;
    }

    let value = (covariance * covariance) / (variance_a * variance_b);
    value.clamp(0.0, 1.0)
}

/// Bivariate Gaussian-model mutual information `-0.5*log2(1 - rho^2)`
/// (Section 5), from a squared Pearson/multiple correlation. `NaN` if the
/// input is not finite, is negative, or is within `PID_DEGENERACY_PIVOT_FLOOR`
/// of `1.0`. The floor (not a bare `>= 1.0` check) matters: without it, a
/// squared correlation numerically indistinguishable from 1 but a few ULPs
/// below it produces an enormous, meaningless "information" value from
/// `log2` of a near-zero argument — the same numerical instability the
/// Cholesky pivot floor guards against in `cholesky_log2_determinant_3x3`,
/// applied here to the bivariate closed form.
fn bivariate_mutual_information_bits(rho_squared: f64) -> f64 {
    if !rho_squared.is_finite()
        || rho_squared < 0.0
        || 1.0 - rho_squared <= PID_DEGENERACY_PIVOT_FLOOR
    {
        return f64::NAN;
    }

    -0.5 * (1.0 - rho_squared).log2()
}

/// `log2(det(A))` of a 3x3 symmetric positive-definite matrix via the
/// Cholesky-Banachiewicz factorization `A = L L^T` (Section 6, method 1):
/// `log2(det(A)) = 2 * sum(log2(L_ii))`. Returns `None` if `A` is not (to
/// within the declared degeneracy floor) positive-definite. `matrix` must be
/// symmetric; only the lower-triangular + diagonal entries are read.
fn cholesky_log2_determinant_3x3(matrix: [[f64; 3]; 3]) -> Option<f64> {
    let a00 = matrix[0][0];
    let a10 = matrix[1][0];
    let a11 = matrix[1][1];
    let a20 = matrix[2][0];
    let a21 = matrix[2][1];
    let a22 = matrix[2][2];

    if a00 <= PID_DEGENERACY_PIVOT_FLOOR {
        return None;
    }
    let l00 = a00.sqrt();

    let l10 = a10 / l00;
    let diagonal_1 = a11 - l10 * l10;
    if diagonal_1 <= PID_DEGENERACY_PIVOT_FLOOR {
        return None;
    }
    let l11 = diagonal_1.sqrt();

    let l20 = a20 / l00;
    let l21 = (a21 - l20 * l10) / l11;
    let diagonal_2 = a22 - l20 * l20 - l21 * l21;
    if diagonal_2 <= PID_DEGENERACY_PIVOT_FLOOR {
        return None;
    }
    let l22 = diagonal_2.sqrt();

    Some(2.0 * (l00.log2() + l11.log2() + l22.log2()))
}

/// The three Gaussian-model mutual informations needed for the PID lattice
/// (Section 5): `I(T;S1)`, `I(T;S2)`, `I(T;{S1,S2})`, in bits.
#[derive(Clone, Copy, Debug)]
struct MutualInformationTriple {
    i_t_s1: f64,
    i_t_s2: f64,
    i_t_joint: f64,
}

/// Method 1 (canonical, Section 6): Cholesky-based log-determinant path.
fn mutual_information_method_1(cov: &TripleCovariance) -> MutualInformationTriple {
    let rho2_t_s1 = squared_correlation(cov.cov_t_s1, cov.var_t, cov.var_s1);
    let rho2_t_s2 = squared_correlation(cov.cov_t_s2, cov.var_t, cov.var_s2);

    let i_t_s1 = bivariate_mutual_information_bits(rho2_t_s1);
    let i_t_s2 = bivariate_mutual_information_bits(rho2_t_s2);

    let sigma_full = [
        [cov.var_t, cov.cov_t_s1, cov.cov_t_s2],
        [cov.cov_t_s1, cov.var_s1, cov.cov_s1_s2],
        [cov.cov_t_s2, cov.cov_s1_s2, cov.var_s2],
    ];
    let det_s1_s2 = cov.var_s1 * cov.var_s2 - cov.cov_s1_s2 * cov.cov_s1_s2;

    let i_t_joint = match cholesky_log2_determinant_3x3(sigma_full) {
        Some(log2_det_full)
            if cov.var_t > PID_DEGENERACY_PIVOT_FLOOR && det_s1_s2 > PID_DEGENERACY_PIVOT_FLOOR =>
        {
            0.5 * (cov.var_t.log2() + det_s1_s2.log2() - log2_det_full)
        }
        _ => f64::NAN,
    };

    MutualInformationTriple {
        i_t_s1,
        i_t_s2,
        i_t_joint,
    }
}

/// Method 2 (cross-check, Section 6): the classical squared
/// multiple-correlation-coefficient identity from pairwise Pearson
/// correlations, a genuinely independent arithmetic path from method 1 (not
/// merely a re-typing of the same computation). Computed on BOTH the real
/// `--full` path (Section 13/17 require the method-1/method-2 cross-check
/// table in the required raw output, per block and aggregate) and exercised
/// directly in the bounded tests.
fn mutual_information_method_2(cov: &TripleCovariance) -> MutualInformationTriple {
    let rho2_t_s1 = squared_correlation(cov.cov_t_s1, cov.var_t, cov.var_s1);
    let rho2_t_s2 = squared_correlation(cov.cov_t_s2, cov.var_t, cov.var_s2);

    let i_t_s1 = bivariate_mutual_information_bits(rho2_t_s1);
    let i_t_s2 = bivariate_mutual_information_bits(rho2_t_s2);

    let i_t_joint = if cov.var_t > PID_DEGENERACY_PIVOT_FLOOR
        && cov.var_s1 > PID_DEGENERACY_PIVOT_FLOOR
        && cov.var_s2 > PID_DEGENERACY_PIVOT_FLOOR
    {
        let rho_t_s1 = cov.cov_t_s1 / (cov.var_t * cov.var_s1).sqrt();
        let rho_t_s2 = cov.cov_t_s2 / (cov.var_t * cov.var_s2).sqrt();
        let rho_s1_s2 = cov.cov_s1_s2 / (cov.var_s1 * cov.var_s2).sqrt();

        let denominator = 1.0 - rho_s1_s2 * rho_s1_s2;

        if denominator > PID_DEGENERACY_PIVOT_FLOOR {
            let r_squared = (rho_t_s1 * rho_t_s1 + rho_t_s2 * rho_t_s2
                - 2.0 * rho_t_s1 * rho_t_s2 * rho_s1_s2)
                / denominator;

            bivariate_mutual_information_bits(r_squared.clamp(0.0, 1.0))
        } else {
            f64::NAN
        }
    } else {
        f64::NAN
    };

    MutualInformationTriple {
        i_t_s1,
        i_t_s2,
        i_t_joint,
    }
}

/// The two-source PID lattice under MMI redundancy (Barrett, 2015; Section
/// 5): `Red = min(I(T;S1), I(T;S2))`; `Un_i = I(T;Si) - Red`; `Syn =
/// I(T;{S1,S2}) - I(T;S1) - I(T;S2) + Red`. All four terms are bits and, for
/// a well-defined (non-degenerate) input, are guaranteed non-negative under
/// the Gaussian working model (Section 4.2).
#[derive(Clone, Copy, Debug)]
struct PartialInformationDecomposition {
    redundancy: f64,
    unique_1: f64,
    unique_2: f64,
    synergy: f64,
    joint: f64,
}

fn assemble_pid(mi: &MutualInformationTriple) -> PartialInformationDecomposition {
    let redundancy = mi.i_t_s1.min(mi.i_t_s2);
    let unique_1 = mi.i_t_s1 - redundancy;
    let unique_2 = mi.i_t_s2 - redundancy;
    let synergy = mi.i_t_joint - mi.i_t_s1 - mi.i_t_s2 + redundancy;

    PartialInformationDecomposition {
        redundancy,
        unique_1,
        unique_2,
        synergy,
        joint: mi.i_t_joint,
    }
}

impl PartialInformationDecomposition {
    fn is_finite(&self) -> bool {
        self.redundancy.is_finite()
            && self.unique_1.is_finite()
            && self.unique_2.is_finite()
            && self.synergy.is_finite()
            && self.joint.is_finite()
    }

    /// Each component's proportion of the joint MI, or `None` if the joint
    /// MI is non-positive or non-finite (proportions undefined).
    fn proportions(&self) -> Option<[f64; 4]> {
        if !self.is_finite() || self.joint <= 0.0 {
            return None;
        }

        Some([
            self.redundancy / self.joint,
            self.unique_1 / self.joint,
            self.unique_2 / self.joint,
            self.synergy / self.joint,
        ])
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PidComponent {
    Redundancy,
    Unique1,
    Unique2,
    Synergy,
}

impl PidComponent {
    const fn label(self) -> &'static str {
        match self {
            Self::Redundancy => "redundancy",
            Self::Unique1 => "unique(O1)",
            Self::Unique2 => "unique(O2)",
            Self::Synergy => "synergy",
        }
    }
}

/// The component with the largest point estimate (Section 13), or `None` if
/// the decomposition is not finite.
fn dominant_component(pid: &PartialInformationDecomposition) -> Option<PidComponent> {
    if !pid.is_finite() {
        return None;
    }

    let candidates = [
        (PidComponent::Redundancy, pid.redundancy),
        (PidComponent::Unique1, pid.unique_1),
        (PidComponent::Unique2, pid.unique_2),
        (PidComponent::Synergy, pid.synergy),
    ];

    candidates
        .into_iter()
        .max_by(|left, right| left.1.total_cmp(&right.1))
        .map(|(component, _)| component)
}

/// Section 6: cross-method agreement between the canonical (Cholesky/
/// log-det) and cross-check (multiple-correlation) computations, both
/// computed on the SAME covariance. Must agree to within
/// `PID_CROSS_METHOD_TOLERANCE` bits on every quantity where both methods
/// produce a finite value; if both methods report the SAME quantity as
/// non-finite (degenerate), that is agreement (they concur it is
/// undefined), but if one method is finite and the other is not, that is a
/// genuine disagreement — a bare "skip non-finite differences" check would
/// silently treat one method's undetected degeneracy as if it never
/// happened, exactly the failure mode this cross-check exists to catch.
#[derive(Clone, Copy, Debug)]
struct CrossMethodAgreement {
    max_absolute_difference: f64,
    within_tolerance: bool,
}

fn cross_method_agreement(
    method_1: &MutualInformationTriple,
    method_2: &MutualInformationTriple,
) -> CrossMethodAgreement {
    let pairs = [
        (method_1.i_t_s1, method_2.i_t_s1),
        (method_1.i_t_s2, method_2.i_t_s2),
        (method_1.i_t_joint, method_2.i_t_joint),
    ];

    let mut max_absolute_difference = 0.0_f64;
    let mut within_tolerance = true;

    for (value_1, value_2) in pairs {
        match (value_1.is_finite(), value_2.is_finite()) {
            (true, true) => {
                let difference = (value_1 - value_2).abs();
                max_absolute_difference = max_absolute_difference.max(difference);
                within_tolerance &= difference <= PID_CROSS_METHOD_TOLERANCE;
            }
            (false, false) => {
                // Both methods agree the quantity is degenerate/undefined.
            }
            (true, false) | (false, true) => {
                // One method found a well-defined value, the other did not:
                // a genuine disagreement, not a shared degeneracy.
                within_tolerance = false;
            }
        }
    }

    CrossMethodAgreement {
        max_absolute_difference,
        within_tolerance,
    }
}

/// The four PID components' (and the joint MI's) 95% bootstrap intervals
/// (Section 11): resample records with replacement, recompute the full
/// decomposition (method 1 only) on each replicate. Reuses the frozen
/// `ConfidenceInterval` / `confidence_interval` / `DeterministicRng` types.
struct PidBootstrapIntervals {
    redundancy: ConfidenceInterval,
    unique_1: ConfidenceInterval,
    unique_2: ConfidenceInterval,
    synergy: ConfidenceInterval,
    joint: ConfidenceInterval,
}

fn pid_bootstrap(
    target: &[f64],
    source_1: &[f64],
    source_2: &[f64],
    seed: u64,
    replicate_count: usize,
) -> Result<PidBootstrapIntervals, String> {
    let count = target.len();

    if count == 0 {
        return Err("pid_bootstrap requires at least one record".to_owned());
    }

    let mut generator = DeterministicRng::new(seed);

    let mut redundancy_samples = Vec::with_capacity(replicate_count);
    let mut unique_1_samples = Vec::with_capacity(replicate_count);
    let mut unique_2_samples = Vec::with_capacity(replicate_count);
    let mut synergy_samples = Vec::with_capacity(replicate_count);
    let mut joint_samples = Vec::with_capacity(replicate_count);

    for _ in 0..replicate_count {
        let mut resampled_t = Vec::with_capacity(count);
        let mut resampled_s1 = Vec::with_capacity(count);
        let mut resampled_s2 = Vec::with_capacity(count);

        for _ in 0..count {
            let index = generator.index(count);
            resampled_t.push(target[index]);
            resampled_s1.push(source_1[index]);
            resampled_s2.push(source_2[index]);
        }

        let cov = triple_covariance(&resampled_t, &resampled_s1, &resampled_s2)?;
        let mi = mutual_information_method_1(&cov);
        let pid = assemble_pid(&mi);

        redundancy_samples.push(pid.redundancy);
        unique_1_samples.push(pid.unique_1);
        unique_2_samples.push(pid.unique_2);
        synergy_samples.push(pid.synergy);
        joint_samples.push(pid.joint);
    }

    Ok(PidBootstrapIntervals {
        redundancy: confidence_interval(redundancy_samples),
        unique_1: confidence_interval(unique_1_samples),
        unique_2: confidence_interval(unique_2_samples),
        synergy: confidence_interval(synergy_samples),
        joint: confidence_interval(joint_samples),
    })
}
// ===== END CORE MODULE =====

// The frozen core module above declares `PidBootstrapIntervals` with no
// derives at all. The report-assembly code below needs to hold one computed
// aggregate decomposition in two places at once (TDI-6.3A's per-focal-horizon
// aggregate slot and TDI-6.3B's dense-grid slot reuse the very same aggregate
// computation, preregistration Section 13/14), so this crate adds a manual
// `Clone`/`Copy` impl here -- outside, not inside, the verbatim core block
// above -- rather than duplicate the bootstrap computation. Every field of
// `ConfidenceInterval` is already `Copy`, so this is a mechanical,
// content-free addition, not a change to any formula.
impl Clone for PidBootstrapIntervals {
    fn clone(&self) -> Self {
        *self
    }
}

impl Copy for PidBootstrapIntervals {}

impl std::fmt::Debug for PidBootstrapIntervals {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("PidBootstrapIntervals")
            .field("redundancy", &self.redundancy)
            .field("unique_1", &self.unique_1)
            .field("unique_2", &self.unique_2)
            .field("synergy", &self.synergy)
            .field("joint", &self.joint)
            .finish()
    }
}

/// One block's full information-decomposition result at one horizon
/// (preregistration Section 13): the covariance, both mutual-information
/// methods, their cross-check agreement, the assembled PID lattice, its
/// bootstrap intervals, and the dominant component.
#[derive(Clone, Copy, Debug)]
struct BlockPidResult {
    seed_block: SeedBlockId,
    covariance: TripleCovariance,
    method_1: MutualInformationTriple,
    method_2: MutualInformationTriple,
    agreement: CrossMethodAgreement,
    decomposition: PartialInformationDecomposition,
    bootstrap: PidBootstrapIntervals,
    dominant: Option<PidComponent>,
}

/// The pooled-aggregate analogue of `BlockPidResult` (Section 13): identical
/// fields, minus the per-block identity, computed over all three blocks'
/// pooled records.
#[derive(Clone, Copy, Debug)]
struct AggregatePidResult {
    covariance: TripleCovariance,
    method_1: MutualInformationTriple,
    method_2: MutualInformationTriple,
    agreement: CrossMethodAgreement,
    decomposition: PartialInformationDecomposition,
    bootstrap: PidBootstrapIntervals,
    dominant: Option<PidComponent>,
}

/// Runs the identical resampling procedure to the core module's
/// `pid_bootstrap` above (same primitives, same deterministic seeding, same
/// replicate count, same record-resampling scheme) but additionally enforces
/// a hard finiteness guard on EVERY individual replicate's assembled PID
/// before trusting the result. This does not modify `pid_bootstrap` -- it is
/// still inserted and used verbatim above, unconditionally, once this guard
/// has passed -- it is purely an outer check the production pipeline runs
/// first to decide whether `pid_bootstrap`'s output may be trusted at all.
///
/// Degeneracy is not expected on genuine generator output (preregistration
/// Section 6: "expected never to trigger on the real populations"). This
/// guard exists so that expectation being violated fails loud -- a hard
/// `Err`, refusing to report or print anything -- rather than silently
/// letting a single degenerate replicate be swallowed into a percentile
/// computation (`confidence_interval`'s `total_cmp` sort would place a `NaN`
/// at one extreme without ever flagging it) and shrinking the effective
/// bootstrap sample size with no record of it having happened.
fn guarded_pid_bootstrap(
    target: &[f64],
    source_1: &[f64],
    source_2: &[f64],
    seed: u64,
    replicate_count: usize,
) -> Result<PidBootstrapIntervals, String> {
    let count = target.len();

    if count == 0 {
        return Err("pid_bootstrap requires at least one record".to_owned());
    }

    let mut generator = DeterministicRng::new(seed);

    for replicate_index in 0..replicate_count {
        let mut resampled_t = Vec::with_capacity(count);
        let mut resampled_s1 = Vec::with_capacity(count);
        let mut resampled_s2 = Vec::with_capacity(count);

        for _ in 0..count {
            let index = generator.index(count);
            resampled_t.push(target[index]);
            resampled_s1.push(source_1[index]);
            resampled_s2.push(source_2[index]);
        }

        let cov = triple_covariance(&resampled_t, &resampled_s1, &resampled_s2)?;
        let pid = assemble_pid(&mutual_information_method_1(&cov));

        if !pid.is_finite() {
            return Err(format!(
                "{}: bootstrap replicate {replicate_index} of {replicate_count} produced a \
                 non-finite partial information decomposition (degenerate resampled \
                 covariance); refusing to silently drop it and shrink the effective bootstrap \
                 sample (preregistration Section 6: this is not expected on genuine generator \
                 output)",
                FailureCategory::DegenerateDecomposition
            ));
        }
    }

    // Every one of the `replicate_count` replicates produced a finite
    // decomposition above: `DeterministicRng` is a pure function of `seed`,
    // so recomputing from the same seed reproduces the identical resample
    // sequence, and it is now known safe to trust the verified core module's
    // own `pid_bootstrap` for the reported intervals.
    pid_bootstrap(target, source_1, source_2, seed, replicate_count)
}

/// Computes one seed block's full information decomposition of `U_h`
/// (`record.targets_u[horizon_index]`) with respect to `(O1, O2)`
/// (`record.early_overlap`), at the block's own bootstrap seed
/// (preregistration Sections 6, 11, 13). Both the point-estimate
/// decomposition and every bootstrap replicate are hard-guarded against
/// non-finite values: a degenerate covariance anywhere aborts this function
/// with an `Err` instead of returning a result containing a `NaN` component.
fn compute_block_pid(
    seed_block: SeedBlockId,
    records: &[Record],
    horizon_index: usize,
    bootstrap_seed: u64,
) -> Result<BlockPidResult, String> {
    let target = target_values(records, horizon_index);
    let source_1 = source_overlap_values(records, 0);
    let source_2 = source_overlap_values(records, 1);

    let covariance = triple_covariance(&target, &source_1, &source_2)?;
    let method_1 = mutual_information_method_1(&covariance);
    let method_2 = mutual_information_method_2(&covariance);
    let agreement = cross_method_agreement(&method_1, &method_2);
    let decomposition = assemble_pid(&method_1);

    if !decomposition.is_finite() {
        return Err(format!(
            "{}: block {} at horizon index {horizon_index}: the point-estimate partial \
             information decomposition is non-finite (degenerate covariance); refusing to \
             report or print a result (preregistration Section 6: this is not expected on \
             genuine generator output)",
            FailureCategory::DegenerateDecomposition,
            seed_block.label(),
        ));
    }

    let dominant = dominant_component(&decomposition);
    let bootstrap = guarded_pid_bootstrap(
        &target,
        &source_1,
        &source_2,
        bootstrap_seed,
        PID_BOOTSTRAP_REPLICATES,
    )
    .map_err(|error| {
        format!(
            "block {} at horizon index {horizon_index}: {error}",
            seed_block.label()
        )
    })?;

    Ok(BlockPidResult {
        seed_block,
        covariance,
        method_1,
        method_2,
        agreement,
        decomposition,
        bootstrap,
        dominant,
    })
}

/// The pooled-aggregate analogue of `compute_block_pid` (Sections 6, 11,
/// 13): `records` is expected to already be the pooled concatenation of all
/// three blocks' records (see `assemble_tdi63_report`), and the single
/// pooled-aggregate bootstrap seed `AGGREGATE_BOOTSTRAP_SEED` is used in
/// place of a per-block seed. The same hard finiteness guard applies to both
/// the point estimate and every bootstrap replicate.
fn compute_aggregate_pid(
    records: &[Record],
    horizon_index: usize,
) -> Result<AggregatePidResult, String> {
    let target = target_values(records, horizon_index);
    let source_1 = source_overlap_values(records, 0);
    let source_2 = source_overlap_values(records, 1);

    let covariance = triple_covariance(&target, &source_1, &source_2)?;
    let method_1 = mutual_information_method_1(&covariance);
    let method_2 = mutual_information_method_2(&covariance);
    let agreement = cross_method_agreement(&method_1, &method_2);
    let decomposition = assemble_pid(&method_1);

    if !decomposition.is_finite() {
        return Err(format!(
            "{}: aggregate at horizon index {horizon_index}: the point-estimate partial \
             information decomposition is non-finite (degenerate covariance); refusing to \
             report or print a result (preregistration Section 6: this is not expected on \
             genuine generator output)",
            FailureCategory::DegenerateDecomposition,
        ));
    }

    let dominant = dominant_component(&decomposition);
    let bootstrap = guarded_pid_bootstrap(
        &target,
        &source_1,
        &source_2,
        AGGREGATE_BOOTSTRAP_SEED,
        PID_BOOTSTRAP_REPLICATES,
    )
    .map_err(|error| format!("aggregate at horizon index {horizon_index}: {error}"))?;

    Ok(AggregatePidResult {
        covariance,
        method_1,
        method_2,
        agreement,
        decomposition,
        bootstrap,
        dominant,
    })
}

/// Section 12 descriptor diagnostic (context only, consumed by no TDI-6.3
/// criterion): the pooled means of the four exact descriptors delta,
/// delta_bar, s2, s3. `mean_baseline_grand` and `mean_overlap_grand` are
/// additional bookkeeping, not among Section 12's four named descriptors:
/// the verbatim-transplanted `Record`/`analyze_seed` machinery still
/// populates the 13 inherited baseline features and the six raw per-horizon
/// overlaps on every record (Section 4.6), and although no TDI-6.3 criterion
/// reads them, this diagnostic reports their grand means alongside the
/// named descriptors purely so every field the frozen `Record` carries is
/// accounted for somewhere in the required raw output.
#[derive(Clone, Copy, Debug)]
struct DescriptorDiagnostic {
    mean_delta: f64,
    mean_delta_bar: f64,
    mean_s2: f64,
    mean_s3: f64,
    mean_baseline_grand: f64,
    mean_overlap_grand: f64,
}

fn descriptor_diagnostic(records: &[Record]) -> DescriptorDiagnostic {
    if records.is_empty() {
        return DescriptorDiagnostic {
            mean_delta: 0.0,
            mean_delta_bar: 0.0,
            mean_s2: 0.0,
            mean_s3: 0.0,
            mean_baseline_grand: 0.0,
            mean_overlap_grand: 0.0,
        };
    }

    let count = records.len() as f64;
    let mut delta_sum = 0.0_f64;
    let mut delta_bar_sum = 0.0_f64;
    let mut s2_sum = 0.0_f64;
    let mut s3_sum = 0.0_f64;
    let mut baseline_sum = 0.0_f64;
    let mut overlap_sum = 0.0_f64;

    for record in records {
        delta_sum += record.contraction[0];
        delta_bar_sum += record.contraction[1];
        s2_sum += record.spectral[0];
        s3_sum += record.spectral[1];
        baseline_sum += record.baseline.iter().sum::<f64>();
        overlap_sum += record.overlaps.iter().sum::<f64>();
    }

    DescriptorDiagnostic {
        mean_delta: delta_sum / count,
        mean_delta_bar: delta_bar_sum / count,
        mean_s2: s2_sum / count,
        mean_s3: s3_sum / count,
        mean_baseline_grand: baseline_sum / (count * BASELINE_FEATURE_COUNT as f64),
        mean_overlap_grand: overlap_sum / (count * TARGET_HORIZON_COUNT as f64),
    }
}

/// TDI-6.3A's per-focal-horizon bundle (Section 13): the three per-block
/// decompositions and the pooled-aggregate decomposition, all at the same
/// horizon.
#[derive(Clone, Copy, Debug)]
struct FocalHorizonPid {
    horizon: usize,
    blocks: [BlockPidResult; SEED_BLOCK_COUNT],
    aggregate: AggregatePidResult,
}

/// TDI-6.3B's per-dense-horizon bundle (Section 14): the pooled-aggregate
/// decomposition only.
#[derive(Clone, Copy, Debug)]
struct DenseHorizonPid {
    horizon: usize,
    aggregate: AggregatePidResult,
}

/// TDI-6.3C's per-focal-horizon cross-block consistency bundle (Section 15).
#[derive(Clone, Copy, Debug)]
struct FocalConsistency {
    horizon: usize,
    cross_block_dominant_component_consistent: bool,
    block_dominant: [(SeedBlockId, Option<PidComponent>); SEED_BLOCK_COUNT],
}

/// Pure core of the TDI-6.3C cross-block consistency check (Section 15):
/// true iff every block's dominant component equals the first block's
/// (vacuously true for an empty slice; `assemble_tdi63_report` always calls
/// this with exactly `SEED_BLOCK_COUNT` entries). Two blocks that both lack
/// a well-defined dominant component (`None`) count as agreeing that the
/// decomposition is degenerate there, mirroring `cross_method_agreement`'s
/// shared-degeneracy convention in the core module above.
fn cross_block_dominant_component_consistent(block_dominant: &[Option<PidComponent>]) -> bool {
    match block_dominant.first() {
        Some(first) => block_dominant.iter().all(|dominant| dominant == first),
        None => true,
    }
}

/// Pure core of the TDI-6.3B dominant-component stability summary (Section
/// 14): `stable` iff the dominant component is identical at every grid
/// horizon; `shift_horizons` lists each horizon (using the LATER horizon of
/// a changing consecutive pair, i.e. the horizon at which the shift becomes
/// visible) at which the dominant component differs from the previous
/// horizon in the grid.
fn dominant_component_shift_summary(
    horizons: &[usize],
    dominant: &[Option<PidComponent>],
) -> (bool, Vec<usize>) {
    let stable = dominant.windows(2).all(|pair| pair[0] == pair[1]);

    let shift_horizons = (1..dominant.len())
        .filter(|&index| dominant[index] != dominant[index - 1])
        .map(|index| horizons[index])
        .collect();

    (stable, shift_horizons)
}

/// Criterion TDI-6.3A (Section 13, primary, descriptive): the full
/// decomposition at both focal horizons U3 and U6, per block and pooled
/// aggregate. Purely descriptive -- no pass/fail classification.
#[derive(Clone, Copy, Debug)]
struct Tdi63CriterionA {
    focal: [FocalHorizonPid; FOCAL_HORIZON_COUNT],
}

/// Criterion TDI-6.3B (Section 14, descriptive): the pooled-aggregate
/// decomposition across the full dense horizon grid U3..U8, plus whether the
/// dominant component is stable across the grid or shifts (and, if so, at
/// which horizon(s)).
#[derive(Clone, Debug)]
struct Tdi63CriterionB {
    grid: [DenseHorizonPid; TARGET_HORIZON_COUNT],
    dominant_stable: bool,
    shift_horizons: Vec<usize>,
}

/// Criterion TDI-6.3C (Section 15, descriptive replication check): at each
/// focal horizon, whether the three blocks' independently computed dominant
/// components agree.
#[derive(Clone, Copy, Debug)]
struct Tdi63CriterionC {
    focal: [FocalConsistency; FOCAL_HORIZON_COUNT],
}

/// The complete TDI-6.3 report (preregistration Section 17): the raw
/// per-block populations (for population accounting), the Section 12
/// descriptor diagnostics (per block and pooled aggregate), and the three
/// descriptive criteria TDI-6.3A/B/C.
#[derive(Debug)]
struct Tdi63ExperimentReport {
    blocks: Vec<BlockPopulations>,
    descriptor_diagnostics: Vec<(SeedBlockId, DescriptorDiagnostic)>,
    aggregate_descriptor_diagnostic: DescriptorDiagnostic,
    criterion_a: Tdi63CriterionA,
    criterion_b: Tdi63CriterionB,
    criterion_c: Tdi63CriterionC,
}

/// Assembles the complete TDI-6.3 report from already-generated (or, for the
/// termination smoke, already-synthesized) `BlockPopulations`, in
/// `FROZEN_BLOCK_ORDER`. This is the generation-agnostic core of the
/// pipeline: `run_tdi63_pipeline` calls it after real generation;
/// `run_termination_smoke` calls it directly on bounded, in-memory synthetic
/// populations, so the entire decomposition/criteria/printing machinery is
/// exercised identically in both cases without the smoke path ever
/// generating a real candidate.
fn assemble_tdi63_report(blocks: Vec<BlockPopulations>) -> Result<Tdi63ExperimentReport, String> {
    let seed_blocks: Vec<SeedBlockId> = blocks.iter().map(|block| block.seed_block).collect();
    validate_frozen_block_order(&seed_blocks)?;

    // Section 4.5: pool all four populations of each block; no train/holdout
    // split.
    let block_records: Vec<(SeedBlockId, Vec<Record>)> = blocks
        .iter()
        .map(|block| (block.seed_block, block.combined_all_records()))
        .collect();

    let mut all_records = Vec::new();
    for (_, records) in &block_records {
        all_records.extend_from_slice(records);
    }

    // Aggregate PID across the full dense horizon grid U3..U8, computed once
    // here and shared between TDI-6.3A's two focal aggregate entries (Section
    // 13) and the whole of TDI-6.3B (Section 14): the two focal horizons are a
    // subset of the six-horizon dense grid, so the same six computations
    // cover both criteria without recomputing anything.
    let mut dense_aggregate = Vec::with_capacity(TARGET_HORIZON_COUNT);
    for horizon_index in 0..TARGET_HORIZON_COUNT {
        dense_aggregate.push(compute_aggregate_pid(&all_records, horizon_index)?);
    }

    let focal_indices = focal_horizon_indices();

    // TDI-6.3A: per-block decomposition at the two focal horizons, paired
    // with the already-computed aggregate at those same horizons.
    let mut focal = Vec::with_capacity(FOCAL_HORIZON_COUNT);
    for &horizon_index in &focal_indices {
        let mut block_results = Vec::with_capacity(SEED_BLOCK_COUNT);

        for (seed_block, records) in &block_records {
            block_results.push(compute_block_pid(
                *seed_block,
                records,
                horizon_index,
                seed_block.bootstrap_seed(),
            )?);
        }

        let block_results: [BlockPidResult; SEED_BLOCK_COUNT] = block_results
            .try_into()
            .map_err(|_| "expected exactly three block PID results".to_owned())?;

        focal.push(FocalHorizonPid {
            horizon: TARGET_HORIZONS[horizon_index],
            blocks: block_results,
            aggregate: dense_aggregate[horizon_index],
        });
    }

    let focal: [FocalHorizonPid; FOCAL_HORIZON_COUNT] = focal
        .try_into()
        .map_err(|_| "expected exactly two focal horizons".to_owned())?;

    let criterion_a = Tdi63CriterionA { focal };

    // TDI-6.3B: aggregate-only, across the full dense grid.
    let grid: [DenseHorizonPid; TARGET_HORIZON_COUNT] =
        std::array::from_fn(|index| DenseHorizonPid {
            horizon: TARGET_HORIZONS[index],
            aggregate: dense_aggregate[index],
        });

    let dense_horizons: Vec<usize> = grid.iter().map(|entry| entry.horizon).collect();
    let dense_dominant: Vec<Option<PidComponent>> =
        grid.iter().map(|entry| entry.aggregate.dominant).collect();
    let (dominant_stable, shift_horizons) =
        dominant_component_shift_summary(&dense_horizons, &dense_dominant);

    let criterion_b = Tdi63CriterionB {
        grid,
        dominant_stable,
        shift_horizons,
    };

    // TDI-6.3C: cross-block dominant-component consistency at the focal
    // horizons, reusing TDI-6.3A's per-block results.
    let focal_consistency: [FocalConsistency; FOCAL_HORIZON_COUNT] = std::array::from_fn(|slot| {
        let entry = &criterion_a.focal[slot];
        let block_dominant: [(SeedBlockId, Option<PidComponent>); SEED_BLOCK_COUNT] =
            std::array::from_fn(|block_slot| {
                (
                    entry.blocks[block_slot].seed_block,
                    entry.blocks[block_slot].dominant,
                )
            });

        let dominant_only: Vec<Option<PidComponent>> = block_dominant
            .iter()
            .map(|(_, dominant)| *dominant)
            .collect();

        FocalConsistency {
            horizon: entry.horizon,
            cross_block_dominant_component_consistent: cross_block_dominant_component_consistent(
                &dominant_only,
            ),
            block_dominant,
        }
    });

    let criterion_c = Tdi63CriterionC {
        focal: focal_consistency,
    };

    // Section 12: descriptor diagnostic, per block and pooled aggregate.
    let descriptor_diagnostics: Vec<(SeedBlockId, DescriptorDiagnostic)> = block_records
        .iter()
        .map(|(seed_block, records)| (*seed_block, descriptor_diagnostic(records)))
        .collect();
    let aggregate_descriptor_diagnostic = descriptor_diagnostic(&all_records);

    Ok(Tdi63ExperimentReport {
        blocks,
        descriptor_diagnostics,
        aggregate_descriptor_diagnostic,
        criterion_a,
        criterion_b,
        criterion_c,
    })
}

/// Runs the full TDI-6.3 pipeline (generation of the width-3/width-4
/// populations across seed blocks S/T/U, then `assemble_tdi63_report`) over
/// an arbitrary set of population specifications. Callers control scale
/// entirely through `population_specs`: the preregistered `population_specs()`
/// output requests the real 120,000-record run, while tests pass tiny
/// synthetic-scale specs instead. `--termination-smoke` never calls this
/// function at all (see `run_termination_smoke`): it exercises
/// `assemble_tdi63_report` directly on bounded, in-memory synthetic
/// populations, so no real candidate generation ever runs on the smoke path.
/// This function is called with the real specs only from
/// `run_full_experiment`'s `--full` path, and only after that path's exact
/// confirmation-token check has passed.
fn run_tdi63_pipeline(
    population_specs: &[PopulationSpec],
) -> Result<Tdi63ExperimentReport, String> {
    validate_seed_reservations(population_specs)?;

    let mut blocks = Vec::with_capacity(SEED_BLOCK_COUNT);

    for seed_block in FROZEN_BLOCK_ORDER {
        blocks.push(
            generate_block_populations(seed_block, population_specs)
                .map_err(|error| error.to_string())?,
        );
    }

    assemble_tdi63_report(blocks)
}

fn focal_horizon_indices() -> [usize; FOCAL_HORIZON_COUNT] {
    std::array::from_fn(|slot| {
        target_horizon_index(FOCAL_HORIZONS[slot])
            .expect("every focal horizon belongs to the target horizons")
    })
}

/// A tiny, bounded, in-memory synthetic record set exercising the PID
/// machinery without any real candidate generation (the termination-smoke
/// contract, preregistration Section 16). `S1` varies linearly in the record
/// index and `S2` varies through an arithmetically distinct (not affinely
/// related) modular pattern, so the two are not collinear; `T` is built from
/// `S1`, `S2` AND a third, likewise arithmetically distinct component `C`
/// with a horizon-dependent weight, giving `T` a genuinely independent
/// component beyond `S1`, `S2` (see this file's own degeneracy tests for why
/// a `T` expressible purely as a function of the same draws as `S1`, `S2`
/// would be rank-deficient -- correct NaN-producing behaviour, not a bug --
/// rather than the finite, non-degenerate decomposition this fixture is
/// built to demonstrate): `T_h = w1*S1 + w2*(2*S2 - S1) + w3(h)*C`, matching
/// the preregistration Section 6 working model. Verified numerically before
/// being committed (non-degenerate 3x3 covariance at every horizon, method-1/
/// method-2 agreement to floating-point precision, a genuine dominant-
/// component shift across the grid).
fn synthetic_smoke_records() -> Vec<Record> {
    const COUNT: usize = 12;

    let mut records = Vec::with_capacity(COUNT);

    for i in 0..COUNT {
        let index = i as f64;
        let s1 = 0.10 + 0.05 * index;
        let s2 = 0.20 + 0.02 * ((i * i) % 7) as f64;
        let c = 0.30 + 0.04 * ((i * 3 + 1) % 5) as f64;

        let mut targets_u = [0.0_f64; TARGET_HORIZON_COUNT];
        for (horizon_index, target) in targets_u.iter_mut().enumerate() {
            let w3 = 0.15 + 0.05 * horizon_index as f64;
            *target = 0.5 * s1 + 0.3 * (2.0 * s2 - s1) + w3 * c;
        }

        records.push(Record {
            baseline: std::array::from_fn(|slot| 0.05 * (slot as f64 + index)),
            early_overlap: [s1, s2],
            contraction: [0.3 + 0.01 * index, 0.2 + 0.01 * index],
            spectral: [1.5 + 0.02 * index, 1.2 + 0.02 * index],
            overlaps: [0.30; TARGET_HORIZON_COUNT],
            targets_u,
        });
    }

    records
}

/// Builds a synthetic (not preregistered) `PopulationSpec` for the
/// termination smoke: same shape as `PopulationSpec::from_block`, but with
/// an arbitrary small seed/count divorced from `population_specs()`'s real
/// reservations, so the smoke path never reports a requested/accepted
/// mismatch against the real preregistered counts.
fn synthetic_population_spec(
    seed_block: SeedBlockId,
    population: PopulationKind,
    seed: u64,
    target_count: usize,
) -> PopulationSpec {
    PopulationSpec {
        seed_block,
        population,
        width: population.width(),
        seed,
        target_count,
    }
}

/// Wraps a synthetic record set into a `PopulationGenerationReport` shape,
/// without ever calling `analyze_seed`/`generate_successor_masks`/
/// `build_system` -- i.e. without any real candidate generation.
fn synthetic_population_report(
    spec: PopulationSpec,
    records: Vec<Record>,
) -> PopulationGenerationReport {
    let attempts = records.len();

    PopulationGenerationReport {
        spec,
        report: GenerationReport {
            records,
            next_seed: spec.seed + attempts as u64,
            excluded: 0,
            rejections: RejectionCounts::default(),
            attempts,
            limits: GenerationLimits {
                max_attempts: attempts.max(1),
                no_progress_limit: attempts.max(1),
            },
        },
    }
}

/// Builds one block's four synthetic populations (all four populated with
/// the same synthetic `records`) for the termination smoke.
fn synthetic_block_populations(
    seed_block: SeedBlockId,
    base_seed: u64,
    records: &[Record],
) -> BlockPopulations {
    let count = records.len();

    BlockPopulations {
        seed_block,
        training_width_3: synthetic_population_report(
            synthetic_population_spec(seed_block, PopulationKind::TrainingWidth3, base_seed, count),
            records.to_vec(),
        ),
        holdout_width_3: synthetic_population_report(
            synthetic_population_spec(
                seed_block,
                PopulationKind::HoldoutWidth3,
                base_seed + 1,
                count,
            ),
            records.to_vec(),
        ),
        training_width_4: synthetic_population_report(
            synthetic_population_spec(
                seed_block,
                PopulationKind::TrainingWidth4,
                base_seed + 2,
                count,
            ),
            records.to_vec(),
        ),
        holdout_width_4: synthetic_population_report(
            synthetic_population_spec(
                seed_block,
                PopulationKind::HoldoutWidth4,
                base_seed + 3,
                count,
            ),
            records.to_vec(),
        ),
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
/// tests. Freeze-time artifacts (e.g. the TDI-6.3 scientific manifest) do
/// not exist yet while TDI-6.3 remains under implementation, so a missing
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

/// The full frozen ancestor chain TDI-5.1 -> TDI-5.8, TDI-6.1, TDI-6.2,
/// TDI-6.5 (preregistration Sections 1.3, 17, 22). Each entry is an
/// (identifier, evaluator path, evaluator SHA-256, preregistration path,
/// preregistration SHA-256) tuple, mirroring TDI-5.8's own
/// `FROZEN_ANCESTOR_CHAIN`. TDI-6.3 prints this chain for provenance and
/// asserts, in a bounded test, that every hash is unchanged -- a frozen
/// ancestor changing would be a freeze violation.
///
/// Section 1.3 distinguishes three roles that must not be conflated: TDI-5.6
/// is TDI-6.3's direct scientific/code ancestor (v56.rs is transplanted
/// verbatim); TDI-5.8 and TDI-6.5 are the rolling scientific-code-manifest
/// ancestors (chain-of-custody bookkeeping, the two most-recently-merged
/// experiments at TDI-6.3's build time); and this full eleven-entry list is
/// the complete verified chain non-regression check, re-verified here and in
/// `reproduce-tdi6.3.sh`/`tdi63-ci.yml` before any generation, exactly as
/// every prior experiment's own full chain is. TDI-6.1 and TDI-6.2 are
/// included in this full chain (not named scientific ancestors of TDI-6.3,
/// but frozen predecessors whose integrity TDI-6.3 -- built after them --
/// still attests to, same as TDI-5.8's own chain includes them). Hashes were
/// computed with `sha256sum` against the actual committed files (not
/// guessed) before being pinned here and in the
/// `frozen_ancestor_hashes_are_unchanged` test.
const FROZEN_ANCESTOR_CHAIN: [(&str, &str, &str, &str, &str); 11] = [
    (
        "TDI-5.1",
        "tdi-bench/src/bin/tdi-continuous-deficit-geometry-v51.rs",
        "d69d42fa31d973603eabd0ded8ffd8ca2f0a4b0b8fcec5f9de42ed8c7ce37444",
        "docs/TDI-5.1-CONTINUOUS-DEFICIT-GEOMETRY-PREREGISTRATION.md",
        "25b65a07b7f248df3e043b9b7f63611c360f60f3d49a600a5612305440131852",
    ),
    (
        "TDI-5.2",
        "tdi-bench/src/bin/tdi-independent-overlap-ablation-v52.rs",
        "2308607729659c7546a17530e69773f982d9a1cf41656ea7898e0123ca469ef7",
        "docs/TDI-5.2-INDEPENDENT-OVERLAP-ABLATION-PREREGISTRATION.md",
        "f57a054bc95eb2e041434d6e2049509b0dce1a5397f9666d274b1bbac332be35",
    ),
    (
        "TDI-5.3",
        "tdi-bench/src/bin/tdi-independent-overlap-ablation-v53.rs",
        "93181fb75d4882be2ca0b26c1babe9db747583d6e08e6992617ec12b7f65460f",
        "docs/TDI-5.3-INDEPENDENT-OVERLAP-ACTIVATION-PREREGISTRATION.md",
        "7223128dcfd751ebeb6488c01c3512d0a10b35937ec170504984295eb421682e",
    ),
    (
        "TDI-5.4",
        "tdi-bench/src/bin/tdi-independent-overlap-ablation-v54.rs",
        "dcf24d7eb1ccd938a81163738c38d31a693474c8a1d94046734bda243ca772bf",
        "docs/TDI-5.4-NONLINEAR-OVERLAP-SUFFICIENCY-PREREGISTRATION.md",
        "229a0a8efa391c67c4dda1322b984109b142be3abf972d0a08f3c4ac742ec6ac",
    ),
    (
        "TDI-5.5",
        "tdi-bench/src/bin/tdi-independent-overlap-ablation-v55.rs",
        "10df698d10f010b9f6c18e2a4d78042eb399d3812b8d69c2b4bb799de828b835",
        "docs/TDI-5.5-OVERLAP-BASELINE-CHALLENGE-PREREGISTRATION.md",
        "37260b3349107659487e42e66c269ecad44efaf6131f8206bb28dfbcf83f9da1",
    ),
    (
        "TDI-5.6",
        "tdi-bench/src/bin/tdi-independent-overlap-ablation-v56.rs",
        "0820274b3edb58a6e123c612dbed8dd8a1725221240365f142d9510404e1d1b2",
        "docs/TDI-5.6-EXACT-SPECTRAL-CHALLENGE-PREREGISTRATION.md",
        "59e3375b82d0bb7aad7be0591b9d1eac074d4b194678dfe0e06e73c8aac89807",
    ),
    (
        "TDI-5.7",
        "tdi-bench/src/bin/tdi-independent-overlap-ablation-v57.rs",
        "900031bc27a35e327038911d93f10d74458f913e64d9644b225963df699049ae",
        "docs/TDI-5.7-GENERATOR-ROBUSTNESS-PREREGISTRATION.md",
        "2ca7d1a674d451e642beb5b01f8a0d8f08f8fadcf7f91032370e7fd5e3d91476",
    ),
    (
        "TDI-5.8",
        "tdi-bench/src/bin/tdi-independent-overlap-ablation-v58.rs",
        "e58d07e9ee01ab447be90fc90913661a0cbacd765e02f4670ada01965556f53a",
        "docs/TDI-5.8-CROSS-WIDTH-INVARIANCE-PREREGISTRATION.md",
        "981dc709ae87f9191548bf6c31b4b0558b9550d196c6caa69220206101b9c0de",
    ),
    (
        "TDI-6.1",
        "tdi-bench/src/bin/tdi-independent-overlap-ablation-v61.rs",
        "bb9d155021117b70d1483a9abbc51f45f994caddb8a17365d7fb14f02201f278",
        "docs/TDI-6.1-SPECTRAL-GAP-MIXING-TIME-PREREGISTRATION.md",
        "4d754f334c95b113078c28a24069ffd8fb3e93e2ba89055001aab3bf3ee1a159",
    ),
    (
        "TDI-6.2",
        "tdi-bench/src/bin/tdi-independent-overlap-ablation-v62.rs",
        "793fc42d0567283c0f6c773e74597a6ff38d7278cf6e14fcdca7d60e33758a37",
        "docs/TDI-6.2-NONLINEAR-SUFFICIENCY-PREREGISTRATION.md",
        "a5263642ee79fb946bc9a7aa6fea4b57c22945a91b7ffa6f2220c7e4d4a55869",
    ),
    (
        "TDI-6.5",
        "tdi-bench/src/bin/tdi-independent-overlap-ablation-v65.rs",
        "75bd5198486e7e3c6072deebbdebd256aa3152a7b43b60054349f8e181c200f0",
        "docs/TDI-6.5-GENERATOR-FAMILY-SPECTRAL-ROBUSTNESS-PREREGISTRATION.md",
        "f44eb21446ffdc6897c76818f4d4b22ecf266cf4f2707a4a8d995b0479acd589",
    ),
];

/// Provenance and integrity (TDI-6.3 preregistration Section 17, items
/// 1-5): git commit, compiler/Cargo versions, and the SHA-256 of the v63
/// evaluator, the TDI-6.3 preregistration and the TDI-6.3 scientific
/// manifest -- plus the full frozen ancestor chain (TDI-5.1 -> TDI-5.8,
/// TDI-6.1, TDI-6.2, TDI-6.5 -- Section 1.3), read live and printed for
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
        "évaluateur TDI-6.3 SHA-256      : {}",
        tdi52_sha256_of_repo_file("tdi-bench/src/bin/tdi-independent-overlap-ablation-v63.rs")
    );
    println!(
        "préenregistrement TDI-6.3 SHA-256 : {}",
        tdi52_sha256_of_repo_file("docs/TDI-6.3-INFORMATION-DECOMPOSITION-PREREGISTRATION.md")
    );
    println!(
        "manifeste scientifique TDI-6.3 SHA-256 : {}",
        tdi52_sha256_of_repo_file("docs/TDI-6.3-SCIENTIFIC-CODE.sha256")
    );

    for (label, evaluator, _evaluator_hash, prereg, _prereg_hash) in FROZEN_ANCESTOR_CHAIN {
        println!();
        println!("--- provenance {label} (ancêtre gelé, inchangé) ---");
        println!(
            "évaluateur {label} SHA-256      : {}",
            tdi52_sha256_of_repo_file(evaluator)
        );
        println!(
            "préenregistrement {label} SHA-256 : {}",
            tdi52_sha256_of_repo_file(prereg)
        );
    }
}

/// Section 17, item 6: the PID-relevant frozen constants (the declared
/// tolerance/degeneracy floor/bootstrap-replicate constants from the core
/// module above, plus the inherited horizon/width/seed/feature-count
/// constants). Ridge/layout-specific constants (lambda, model-layout count,
/// CK/SK/SKT feature counts) do not exist in TDI-6.3 and so are not printed.
fn print_tdi52_frozen_constants() {
    println!();
    println!("=== CONSTANTES GELÉES (Section 17, item 6) ===");
    println!("horizon d'observation                      : {OBSERVATION_HORIZON}");
    println!("horizons cibles                            : {TARGET_HORIZONS:?}");
    println!("horizon principal                          : {PRIMARY_HORIZON}");
    println!("horizons focaux (U3, U6)                    : {FOCAL_HORIZONS:?}");
    println!("largeur maximale supportée                  : {MAX_SUPPORTED_WIDTH}");
    println!(
        "espace des ensembles successeurs (largeur 6) : {}",
        match successor_set_space_cardinality(WIDTH_6) {
            Cardinality::Exact(value) => value.to_string(),
            other => format!("{other:?}"),
        }
    );
    println!("nombre de features baseline                 : {BASELINE_FEATURE_COUNT}");
    println!("nombre de features early-overlap            : {EARLY_OVERLAP_FEATURE_COUNT}");
    println!("nombre de features contraction (δ, δ̄)       : {CONTRACTION_FEATURE_COUNT}");
    println!("nombre de features spectrales (s2, s3)      : {SPECTRAL_FEATURE_COUNT}");
    println!(
        "tailles de population — train w3={TRAIN_WIDTH_3_SYSTEMS}, holdout w3={HOLDOUT_WIDTH_3_SYSTEMS}, \
         train w4={TRAIN_WIDTH_4_SYSTEMS}, holdout w4={HOLDOUT_WIDTH_4_SYSTEMS} (aucune population OOD ; \
         toutes les populations d'un bloc sont regroupées, Section 4.5)"
    );
    println!(
        "multiplicateurs de tentatives — w3={WIDTH_3_ATTEMPT_MULTIPLIER}, w4={WIDTH_4_ATTEMPT_MULTIPLIER}, \
         w5={WIDTH_5_ATTEMPT_MULTIPLIER}, w6={WIDTH_6_ATTEMPT_MULTIPLIER}"
    );
    println!(
        "seuils sans-progrès — w3={WIDTH_3_NO_PROGRESS_LIMIT}, w4={WIDTH_4_NO_PROGRESS_LIMIT}, \
         w5={WIDTH_5_NO_PROGRESS_LIMIT}, w6={WIDTH_6_NO_PROGRESS_LIMIT}"
    );
    println!("tolérance d'accord inter-méthodes (bits)    : {PID_CROSS_METHOD_TOLERANCE:e}");
    println!("plancher de dégénérescence (pivot Cholesky) : {PID_DEGENERACY_PIVOT_FLOOR:e}");
    println!("réplicats bootstrap PID                     : {PID_BOOTSTRAP_REPLICATES}");
    println!(
        "régime FP                                  : IEEE-754 binary64, mono-thread, ordre \
         d'opérations fixe (pas de FMA/parallèle)"
    );
}

/// Section 17, item 7: every seed-block definition (all seeds plus each
/// block's own bootstrap seed), and the single pooled-aggregate bootstrap
/// seed from Section 11.
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

    println!("graine bootstrap agrégat (Section 11) : 0x{AGGREGATE_BOOTSTRAP_SEED:016X}");
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

/// A borrowed view over one PID result's fields, shared by `BlockPidResult`
/// and `AggregatePidResult` (which differ only in whether they carry a
/// `seed_block`), so `print_tdi63_pid_block` takes one bundled argument
/// instead of one parameter per field (clippy::too_many_arguments).
struct PidResultView<'a> {
    covariance: &'a TripleCovariance,
    method_1: &'a MutualInformationTriple,
    method_2: &'a MutualInformationTriple,
    agreement: &'a CrossMethodAgreement,
    decomposition: &'a PartialInformationDecomposition,
    bootstrap: &'a PidBootstrapIntervals,
    dominant: Option<PidComponent>,
}

impl BlockPidResult {
    fn as_view(&self) -> PidResultView<'_> {
        PidResultView {
            covariance: &self.covariance,
            method_1: &self.method_1,
            method_2: &self.method_2,
            agreement: &self.agreement,
            decomposition: &self.decomposition,
            bootstrap: &self.bootstrap,
            dominant: self.dominant,
        }
    }
}

impl AggregatePidResult {
    fn as_view(&self) -> PidResultView<'_> {
        PidResultView {
            covariance: &self.covariance,
            method_1: &self.method_1,
            method_2: &self.method_2,
            agreement: &self.agreement,
            decomposition: &self.decomposition,
            bootstrap: &self.bootstrap,
            dominant: self.dominant,
        }
    }
}

/// Shared printer for one block's or the aggregate's full PID result
/// (preregistration Section 13): the covariance and correlations, both
/// mutual-information methods and their cross-check agreement, the four
/// components and their proportions, their bootstrap intervals, and the
/// dominant component. Used once per block and once for the aggregate at
/// every horizon TDI-6.3A/B reports, so every field of every core-module
/// result type is read here.
fn print_tdi63_pid_block(label: &str, view: PidResultView<'_>) {
    let PidResultView {
        covariance,
        method_1,
        method_2,
        agreement,
        decomposition,
        bootstrap,
        dominant,
    } = view;

    println!();
    println!("--- {label} ---");
    println!(
        "covariance : mean_t={:.9} mean_s1={:.9} mean_s2={:.9} var_t={:.9} var_s1={:.9} \
         var_s2={:.9} cov_t_s1={:.9} cov_t_s2={:.9} cov_s1_s2={:.9}",
        covariance.mean_t,
        covariance.mean_s1,
        covariance.mean_s2,
        covariance.var_t,
        covariance.var_s1,
        covariance.var_s2,
        covariance.cov_t_s1,
        covariance.cov_t_s2,
        covariance.cov_s1_s2,
    );
    println!(
        "corrélations : rho_t_s1={:.9} rho_t_s2={:.9} rho_s1_s2={:.9}",
        covariance.cov_t_s1 / (covariance.var_t * covariance.var_s1).sqrt(),
        covariance.cov_t_s2 / (covariance.var_t * covariance.var_s2).sqrt(),
        covariance.cov_s1_s2 / (covariance.var_s1 * covariance.var_s2).sqrt(),
    );
    println!(
        "méthode 1 (canonique)  : I(T;S1)={:.9} I(T;S2)={:.9} I(T;{{S1,S2}})={:.9}",
        method_1.i_t_s1, method_1.i_t_s2, method_1.i_t_joint
    );
    println!(
        "méthode 2 (cross-check): I(T;S1)={:.9} I(T;S2)={:.9} I(T;{{S1,S2}})={:.9}",
        method_2.i_t_s1, method_2.i_t_s2, method_2.i_t_joint
    );
    println!(
        "accord inter-méthodes  : écart absolu max={:.12} | dans la tolérance ({:e})={}",
        agreement.max_absolute_difference, PID_CROSS_METHOD_TOLERANCE, agreement.within_tolerance
    );

    let proportions = decomposition.proportions();
    let proportion_text = |value: Option<f64>| match value {
        Some(value) => format!("{value:.9}"),
        None => "indéfinie".to_owned(),
    };

    println!(
        "Redundancy  : {:.9} bits (proportion {})",
        decomposition.redundancy,
        proportion_text(proportions.map(|values| values[0]))
    );
    println!(
        "Unique(O1)  : {:.9} bits (proportion {})",
        decomposition.unique_1,
        proportion_text(proportions.map(|values| values[1]))
    );
    println!(
        "Unique(O2)  : {:.9} bits (proportion {})",
        decomposition.unique_2,
        proportion_text(proportions.map(|values| values[2]))
    );
    println!(
        "Synergy     : {:.9} bits (proportion {})",
        decomposition.synergy,
        proportion_text(proportions.map(|values| values[3]))
    );
    println!(
        "I(T;{{S1,S2}}) (méthode 1) : {:.9} bits",
        decomposition.joint
    );

    print_interval("  IC 95 % bootstrap Redundancy", bootstrap.redundancy);
    print_interval("  IC 95 % bootstrap Unique(O1)", bootstrap.unique_1);
    print_interval("  IC 95 % bootstrap Unique(O2)", bootstrap.unique_2);
    print_interval("  IC 95 % bootstrap Synergy", bootstrap.synergy);
    print_interval("  IC 95 % bootstrap I(T;{S1,S2})", bootstrap.joint);

    println!(
        "composante dominante   : {}",
        dominant
            .map(PidComponent::label)
            .unwrap_or("aucune (décomposition non finie)")
    );
}

fn print_interval(label: &str, interval: ConfidenceInterval) {
    println!(
        "{label}: [{:.9}, {:.9}] (médiane {:.9})",
        interval.lower, interval.upper, interval.median
    );
}

/// Criterion TDI-6.3A (Section 13, primary, descriptive): the full
/// decomposition at both focal horizons, per block and pooled aggregate.
fn print_tdi63_criterion_a(criterion_a: &Tdi63CriterionA) {
    println!();
    println!("=== TDI-6.3A — décomposition aux horizons focaux (Section 13) ===");

    for entry in &criterion_a.focal {
        println!();
        println!("--- U_{} ---", entry.horizon);

        for block in &entry.blocks {
            print_tdi63_pid_block(
                &format!("bloc {} — U_{}", block.seed_block.label(), entry.horizon),
                block.as_view(),
            );
        }

        print_tdi63_pid_block(
            &format!("agrégat — U_{}", entry.horizon),
            entry.aggregate.as_view(),
        );
    }
}

/// Criterion TDI-6.3B (Section 14, descriptive): the pooled-aggregate
/// decomposition across the dense horizon grid U3..U8, plus whether the
/// dominant component is stable across the grid or shifts (and, if so, at
/// which horizon(s)).
fn print_tdi63_criterion_b(criterion_b: &Tdi63CriterionB) {
    println!();
    println!("=== TDI-6.3B — décomposition sur la grille dense (Section 14) ===");

    for entry in &criterion_b.grid {
        print_tdi63_pid_block(
            &format!("agrégat — U_{}", entry.horizon),
            entry.aggregate.as_view(),
        );
    }

    println!();
    if criterion_b.dominant_stable {
        println!(
            "TDI-6.3B — composante dominante stable : identique à tous les horizons de la grille"
        );
    } else {
        println!(
            "TDI-6.3B — composante dominante variable : change au(x) horizon(s) {}",
            criterion_b
                .shift_horizons
                .iter()
                .map(|horizon| format!("U{horizon}"))
                .collect::<Vec<_>>()
                .join(", ")
        );
    }
}

/// Criterion TDI-6.3C (Section 15, descriptive replication check): at each
/// focal horizon, whether the three blocks' independently computed dominant
/// components agree, naming every block's dominant component either way.
fn print_tdi63_criterion_c(criterion_c: &Tdi63CriterionC) {
    println!();
    println!("=== TDI-6.3C — cohérence inter-blocs de la composante dominante (Section 15) ===");

    for entry in &criterion_c.focal {
        println!();
        println!(
            "U_{} — cross_block_dominant_component_consistent : {}",
            entry.horizon, entry.cross_block_dominant_component_consistent
        );

        for (seed_block, dominant) in entry.block_dominant {
            println!(
                "  bloc {} — composante dominante : {}",
                seed_block.label(),
                dominant
                    .map(PidComponent::label)
                    .unwrap_or("aucune (décomposition non finie)")
            );
        }
    }
}

/// Section 12 descriptor diagnostic printer (context only, consumed by no
/// TDI-6.3 criterion): the pooled means of delta, delta_bar, s2, s3 per
/// block and for the pooled aggregate, plus the bookkeeping grand means of
/// the inherited baseline features and raw overlaps.
fn print_tdi63_descriptor_diagnostics(report: &Tdi63ExperimentReport) {
    println!();
    println!("=== DIAGNOSTIC DESCRIPTEURS — Section 12 (contexte, aucun critère) ===");

    for (seed_block, diagnostic) in &report.descriptor_diagnostics {
        println!(
            "bloc {} | moyenne δ={:.9} | moyenne δ̄={:.9} | moyenne s2={:.9} | moyenne s3={:.9} \
             | (hors Section 12, bookkeeping) moyenne baseline={:.9} | moyenne overlap brut={:.9}",
            seed_block.label(),
            diagnostic.mean_delta,
            diagnostic.mean_delta_bar,
            diagnostic.mean_s2,
            diagnostic.mean_s3,
            diagnostic.mean_baseline_grand,
            diagnostic.mean_overlap_grand,
        );
    }

    let diagnostic = &report.aggregate_descriptor_diagnostic;
    println!(
        "agrégat | moyenne δ={:.9} | moyenne δ̄={:.9} | moyenne s2={:.9} | moyenne s3={:.9} | \
         (hors Section 12, bookkeeping) moyenne baseline={:.9} | moyenne overlap brut={:.9}",
        diagnostic.mean_delta,
        diagnostic.mean_delta_bar,
        diagnostic.mean_s2,
        diagnostic.mean_s3,
        diagnostic.mean_baseline_grand,
        diagnostic.mean_overlap_grand,
    );
}

/// TDI-6.3 Section 17: the TDI-6.3A/B/C descriptive summaries. None of these
/// is a pass/fail verdict -- a partial information decomposition has no
/// natural "success" or "failure" outcome (preregistration Sections 2, 13-15)
/// -- so unlike TDI-5.6's `print_tdi52_final_verdicts`, nothing here is a
/// Beneficial/Equivalent/Harmful/Inconclusive classification; this prints
/// the descriptive headline of each criterion.
fn print_tdi52_final_verdicts(report: &Tdi63ExperimentReport) {
    println!();
    println!("=== VERDICTS FINAUX (Section 17) ===");
    println!(
        "TDI-6.3A — décomposition aux horizons focaux : descriptive uniquement, aucun \
         verdict pass/fail (Section 13) ; voir la décomposition par bloc et agrégat ci-dessus."
    );

    for entry in &report.criterion_a.focal {
        println!(
            "TDI-6.3A — U{} — composante dominante agrégat : {}",
            entry.horizon,
            entry
                .aggregate
                .dominant
                .map(PidComponent::label)
                .unwrap_or("aucune (décomposition non finie)")
        );
    }

    println!(
        "TDI-6.3B — composante dominante sur la grille dense U3..U8 : {} (Section 14)",
        if report.criterion_b.dominant_stable {
            "stable".to_owned()
        } else {
            format!(
                "variable, change au(x) horizon(s) {}",
                report
                    .criterion_b
                    .shift_horizons
                    .iter()
                    .map(|horizon| format!("U{horizon}"))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        }
    );

    for entry in &report.criterion_c.focal {
        println!(
            "TDI-6.3C — U{} — cross_block_dominant_component_consistent = {} (Section 15)",
            entry.horizon, entry.cross_block_dominant_component_consistent
        );
    }
}

/// Prints the complete TDI-6.3 required raw output (Section 17) for a
/// completed pipeline run. Purely a presentation layer over
/// `Tdi63ExperimentReport`: it has no scale-awareness of its own, so it is
/// exercised at tiny scale by the termination smoke path and by tests. It
/// only ever prints the real 120,000-record run's output when called from
/// `run_full_experiment`'s `--full` path, and only after that path's exact
/// confirmation-token check has passed AND `run_tdi63_pipeline` returned
/// `Ok` -- a non-finite decomposition or bootstrap replicate anywhere aborts
/// with an `Err` before this function is ever reached (see
/// `compute_block_pid` / `compute_aggregate_pid` / `guarded_pid_bootstrap`).
fn print_tdi52_required_raw_output(report: &Tdi63ExperimentReport) {
    print_tdi52_provenance();
    print_tdi52_frozen_constants();
    print_tdi52_seed_block_definitions();
    print_tdi52_population_accounting(&report.blocks);
    print_tdi63_descriptor_diagnostics(report);
    print_tdi63_criterion_a(&report.criterion_a);
    print_tdi63_criterion_b(&report.criterion_b);
    print_tdi63_criterion_c(&report.criterion_c);
    print_tdi52_final_verdicts(report);
}

/// TDI-6.3's termination smoke (preregistration Section 16): exercises the
/// complete PID pipeline -- `compute_block_pid`, `compute_aggregate_pid`,
/// the hard finiteness guard, `assemble_tdi63_report`'s criteria assembly,
/// and the full required-raw-output printing -- on a tiny, bounded, fully
/// in-memory synthetic record set (`synthetic_smoke_records`). Unlike
/// TDI-5.6's termination smoke (which requested `target_count: 1` from the
/// real `population_specs()` and so still ran one real, if tiny, candidate
/// generation), this smoke path never calls `analyze_seed`,
/// `generate_successor_masks`, `build_system`, `generate_block_populations`
/// or `run_tdi63_pipeline`: `population_specs()` is consulted only for its
/// deterministic seed-reservation arithmetic (`validate_preregistered_seed_
/// reservations`, itself no generation), never for its real record counts.
fn run_termination_smoke() -> Result<(), String> {
    println!("=== TDI-6.3 TERMINATION SMOKE ===");

    // Inherited frozen invariant: the width-6 successor-set space is the
    // exact 2^64. TDI-6.3 generates no width-6 populations, but the
    // cardinality machinery is inherited unchanged and still checked here,
    // without generating any record.
    let width_6_space = successor_set_space_cardinality(WIDTH_6);

    if width_6_space != Cardinality::Exact(18_446_744_073_709_551_616_u128) {
        return Err(format!("unexpected width-6 cardinality: {width_6_space:?}"));
    }

    println!("width 6 successor-set space  : 18446744073709551616");

    let seed_reservation_count = validate_preregistered_seed_reservations()?;
    println!("reserved seed ranges          : {seed_reservation_count} disjoint");

    println!("PID bootstrap replicates      : {PID_BOOTSTRAP_REPLICATES}");

    for block in SEED_BLOCKS {
        println!(
            "block {} bootstrap seed       : 0x{:016X}",
            block.id.label(),
            block.bootstrap_seed
        );
    }

    println!("aggregate bootstrap seed      : 0x{AGGREGATE_BOOTSTRAP_SEED:016X}");

    let specs = population_specs();
    println!(
        "population specifications    : {} deterministic entries (4 per block, no OOD) -- \
         consulted here only for seed-reservation arithmetic, never for real record counts",
        specs.len()
    );

    // Inherited-machinery sanity check (unchanged from TDI-5.6's own smoke
    // test): a single, tiny, bounded REAL candidate generation confirming
    // `analyze_seed`/`generate_records_with_limits` and the exact
    // contraction descriptors still work, entirely separate from -- and
    // prior to -- the PID-pipeline exercise below, which uses only the
    // synthetic records built by `synthetic_smoke_records` and never
    // generates a real candidate.
    let inherited_limits = GenerationLimits {
        max_attempts: 64,
        no_progress_limit: 64,
    };
    let inherited_generation = generate_records_with_limits(
        TRAIN_WIDTH_3,
        SEED_BLOCKS[0].training_width_3_seed,
        1,
        inherited_limits,
    )
    .map_err(|error| error.to_string())?;
    println!(
        "inherited generation sanity   : width 3, {} accepted record in {} attempts",
        inherited_generation.records.len(),
        inherited_generation.attempts
    );
    if let Some(first) = inherited_generation.records.first() {
        println!(
            "inherited generation δ, δ̄     : {:.6}, {:.6}",
            first.contraction[0], first.contraction[1]
        );
    }

    // Synthetic, bounded records exercising the PID machinery without any
    // real generation (see `synthetic_smoke_records`'s own documentation for
    // why T carries a genuinely independent component beyond S1, S2).
    let synthetic_records = synthetic_smoke_records();
    println!(
        "synthetic smoke records       : {}",
        synthetic_records.len()
    );

    // Exercise compute_block_pid directly at the primary horizon, as a
    // targeted identity smoke before the full report assembly below.
    let primary_index = primary_horizon_index();
    let block_pid = compute_block_pid(
        SeedBlockId::S,
        &synthetic_records,
        primary_index,
        SeedBlockId::S.bootstrap_seed(),
    )?;
    println!(
        "identity smoke block PID      : I(T;S1)={:.6}, I(T;S2)={:.6}, I(T;joint)={:.6}, \
         cross-method agreement within tolerance={}",
        block_pid.method_1.i_t_s1,
        block_pid.method_1.i_t_s2,
        block_pid.method_1.i_t_joint,
        block_pid.agreement.within_tolerance
    );
    println!(
        "identity smoke dominant       : {}",
        block_pid
            .dominant
            .map(PidComponent::label)
            .unwrap_or("none (non-finite)")
    );

    // The critical wiring smoke: the real report-assembly entrypoint
    // (`assemble_tdi63_report`), over synthetic (never generated) block
    // populations built entirely in memory.
    let blocks = vec![
        synthetic_block_populations(SeedBlockId::S, 1, &synthetic_records),
        synthetic_block_populations(SeedBlockId::T, 101, &synthetic_records),
        synthetic_block_populations(SeedBlockId::U, 201, &synthetic_records),
    ];

    let report = assemble_tdi63_report(blocks)?;

    println!(
        "identity smoke pipeline       : criterion A focal horizons={}, criterion B grid \
         entries={}, criterion B dominant stable={}",
        report.criterion_a.focal.len(),
        report.criterion_b.grid.len(),
        report.criterion_b.dominant_stable
    );
    println!(
        "identity smoke criterion C    : U{} consistent={}",
        report.criterion_c.focal[0].horizon,
        report.criterion_c.focal[0].cross_block_dominant_component_consistent
    );

    print_tdi52_required_raw_output(&report);

    println!("bounded smoke result          : PASS");

    Ok(())
}

/// Name of the environment variable that must carry the exact TDI-6.3
/// full-run confirmation value. See TDI-6.3 preregistration Section 16.
const TDI63_FULL_RUN_CONFIRMATION_VAR: &str = "TDI63_CONFIRM_FULL_RUN";

/// The one accepted value for `TDI63_FULL_RUN_CONFIRMATION_VAR`. Any other
/// value, or the variable being unset, must refuse `--full`.
const TDI63_FULL_RUN_CONFIRMATION_VALUE: &str = "I_ACCEPT_THE_TDI63_FREEZE_RULE";

/// Pure decision function: takes the confirmation value as a plain
/// `Option<&str>` rather than reading the environment itself, so every
/// branch -- missing, wrong, and the one exact accepted value -- can be
/// unit tested directly without ever touching a real environment variable
/// or risking the accepted branch reaching `run_full_experiment` (and,
/// through it, the real pipeline).
fn tdi63_full_run_confirmed(value: Option<&str>) -> bool {
    value == Some(TDI63_FULL_RUN_CONFIRMATION_VALUE)
}

fn tdi63_usage_error() -> String {
    format!(
        "usage: tdi-independent-overlap-ablation-v63 --termination-smoke|--preflight|--full\n\
         a bare (no-argument) invocation does not start the experiment; the \
         real run additionally requires the exact environment variable \
         {TDI63_FULL_RUN_CONFIRMATION_VAR}={TDI63_FULL_RUN_CONFIRMATION_VALUE}"
    )
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Tdi63Mode {
    TerminationSmoke,
    Preflight,
    Full,
}

/// Pure command-line dispatch decision, independent of `main`'s I/O, so
/// that "a bare invocation can never select `--full`" is directly unit
/// testable against plain string slices rather than real process argv.
fn tdi63_parse_mode(arguments: &[String]) -> Result<Tdi63Mode, String> {
    match arguments {
        [flag] if flag == "--termination-smoke" => Ok(Tdi63Mode::TerminationSmoke),
        [flag] if flag == "--preflight" => Ok(Tdi63Mode::Preflight),
        [flag] if flag == "--full" => Ok(Tdi63Mode::Full),
        _ => Err(tdi63_usage_error()),
    }
}

fn main() -> Result<(), String> {
    let arguments = std::env::args().skip(1).collect::<Vec<_>>();

    match tdi63_parse_mode(&arguments)? {
        Tdi63Mode::TerminationSmoke => run_termination_smoke(),
        Tdi63Mode::Preflight => run_preflight(),
        Tdi63Mode::Full => run_full_experiment(),
    }
}

/// The TDI-6.3 full-run entrypoint. Checks the exact confirmation
/// environment variable *before* any generation, decomposition or bootstrap;
/// only when it matches does this call the real full pipeline exactly once,
/// over the real preregistered `population_specs()`, and print the complete
/// required raw output. A non-finite point-estimate decomposition or
/// bootstrap replicate anywhere in `run_tdi63_pipeline` aborts with an `Err`
/// -- propagated here by `?`, so `main` returns `Err` and the process exits
/// non-zero -- strictly before `print_tdi52_required_raw_output` is ever
/// reached (see TDI-6.3 preregistration Section 16 and the hard finiteness
/// guard in `compute_block_pid` / `compute_aggregate_pid` /
/// `guarded_pid_bootstrap`).
fn run_full_experiment() -> Result<(), String> {
    let confirmation = std::env::var(TDI63_FULL_RUN_CONFIRMATION_VAR).ok();

    if !tdi63_full_run_confirmed(confirmation.as_deref()) {
        return Err(format!(
            "TDI-6.3 full execution requires the exact confirmation environment \
             variable {TDI63_FULL_RUN_CONFIRMATION_VAR}={TDI63_FULL_RUN_CONFIRMATION_VALUE}; \
             refusing before any generation, decomposition or bootstrap"
        ));
    }

    let report = run_tdi63_pipeline(&population_specs())?;

    print_tdi52_required_raw_output(&report);

    Ok(())
}

/// TDI-6.3 preflight: verifies the complete frozen configuration (seed
/// reservations, population counts, bootstrap constants, the declared
/// tolerances) and prints identities and the exact real-run command, without
/// ever generating a scientific population. See TDI-6.3 preregistration
/// Section 16.
fn run_preflight() -> Result<(), String> {
    println!();
    println!("=== TDI-6.3 PREFLIGHT (aucune génération scientifique) ===");

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
    println!("réplicats de bootstrap PID par bloc/agrégat     : {PID_BOOTSTRAP_REPLICATES}");
    println!(
        "graines de bootstrap par bloc                   : {}=0x{:016X} {}=0x{:016X} {}=0x{:016X}",
        SeedBlockId::S.label(),
        SeedBlockId::S.bootstrap_seed(),
        SeedBlockId::T.label(),
        SeedBlockId::T.bootstrap_seed(),
        SeedBlockId::U.label(),
        SeedBlockId::U.bootstrap_seed()
    );
    println!("graine de bootstrap agrégat                     : 0x{AGGREGATE_BOOTSTRAP_SEED:016X}");
    println!("tolérance d'accord inter-méthodes (bits)         : {PID_CROSS_METHOD_TOLERANCE:e}");
    println!("plancher de dégénérescence (pivot Cholesky)      : {PID_DEGENERACY_PIVOT_FLOOR:e}");
    println!(
        "régime FP                                       : IEEE-754 binary64, mono-thread, \
         ordre d'opérations fixe (pas de FMA/parallèle)"
    );
    println!(
        "pipeline complet câblé à --full                 : oui (run_tdi63_pipeline, \
         subordonné à {TDI63_FULL_RUN_CONFIRMATION_VAR}) ; garde de finitude difficile activée \
         (compute_block_pid, compute_aggregate_pid, guarded_pid_bootstrap)"
    );

    print_tdi52_provenance();

    println!();
    println!("Commande requise pour l'exécution réelle (jamais lancée automatiquement) :");
    println!("  {TDI63_FULL_RUN_CONFIRMATION_VAR}={TDI63_FULL_RUN_CONFIRMATION_VALUE} \\");
    println!("    bash scripts/reproduce-tdi6.3.sh");

    println!();
    println!("=== PREFLIGHT TERMINÉ : aucun résultat produit ===");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        AGGREGATE_BOOTSTRAP_SEED, BASELINE_FEATURE_COUNT, FOCAL_HORIZON_COUNT, FOCAL_HORIZONS,
        PID_BOOTSTRAP_REPLICATES, PID_CROSS_METHOD_TOLERANCE, PID_DEGENERACY_PIVOT_FLOOR,
        PRIMARY_HORIZON, PidComponent, Record, SEED_BLOCKS, SeedBlockId, TARGET_HORIZON_COUNT,
        TARGET_HORIZONS, TDI63_FULL_RUN_CONFIRMATION_VALUE, TDI63_FULL_RUN_CONFIRMATION_VAR,
        TOTAL_SEED_RESERVATIONS,
    };

    fn read_repo_file(relative_path: &str) -> String {
        std::fs::read_to_string(super::tdi52_repository_root().join(relative_path))
            .unwrap_or_else(|error| panic!("failed to read {relative_path}: {error}"))
    }

    fn evaluator_source() -> String {
        read_repo_file("tdi-bench/src/bin/tdi-independent-overlap-ablation-v63.rs")
    }

    fn record_with(o1: f64, o2: f64, target_u: f64) -> Record {
        Record {
            baseline: [0.0; BASELINE_FEATURE_COUNT],
            early_overlap: [o1, o2],
            contraction: [0.3, 0.2],
            spectral: [1.5, 1.2],
            overlaps: [0.30; TARGET_HORIZON_COUNT],
            targets_u: [target_u; TARGET_HORIZON_COUNT],
        }
    }

    // --- triple_covariance (Section 6) ---

    #[test]
    fn triple_covariance_matches_hand_computed_values() {
        let target = vec![1.0, 2.0, 3.0, 4.0];
        let source_1 = vec![2.0, 4.0, 6.0, 8.0]; // = 2 * target
        let source_2 = vec![1.0, 1.0, 1.0, 1.0]; // constant, zero variance

        let cov = super::triple_covariance(&target, &source_1, &source_2).expect("covariance");

        assert!((cov.mean_t - 2.5).abs() < 1e-12);
        assert!((cov.mean_s1 - 5.0).abs() < 1e-12);
        assert!((cov.mean_s2 - 1.0).abs() < 1e-12);
        assert!((cov.var_t - 1.25).abs() < 1e-12);
        assert!((cov.var_s1 - 5.0).abs() < 1e-12); // 4 * var_t
        assert!(cov.var_s2.abs() < 1e-12);
        assert!((cov.cov_t_s1 - 2.5).abs() < 1e-12); // 2 * var_t
        assert!(cov.cov_t_s2.abs() < 1e-12);
        assert!(cov.cov_s1_s2.abs() < 1e-12);
    }

    #[test]
    fn triple_covariance_rejects_mismatched_lengths_and_too_few_records() {
        assert!(super::triple_covariance(&[1.0], &[1.0], &[1.0]).is_err());
        assert!(super::triple_covariance(&[1.0, 2.0], &[1.0], &[1.0, 2.0]).is_err());
    }

    // --- cholesky_log2_determinant_3x3 (Section 6, method 1) ---

    #[test]
    fn cholesky_log2_determinant_matches_hand_computed_values() {
        // Identity: det = 1, log2(1) = 0.
        let identity = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
        let log2_det = super::cholesky_log2_determinant_3x3(identity).expect("positive definite");
        assert!(log2_det.abs() < 1e-12);

        // Diagonal(2, 4, 8): det = 64, log2(64) = 6.
        let diagonal = [[2.0, 0.0, 0.0], [0.0, 4.0, 0.0], [0.0, 0.0, 8.0]];
        let log2_det = super::cholesky_log2_determinant_3x3(diagonal).expect("positive definite");
        assert!((log2_det - 6.0).abs() < 1e-12);

        // A known non-diagonal SPD matrix [[4,2,0],[2,5,1],[0,1,3]]:
        // det = 4*(15-1) - 2*(6-0) + 0 = 44.
        let matrix = [[4.0, 2.0, 0.0], [2.0, 5.0, 1.0], [0.0, 1.0, 3.0]];
        let log2_det = super::cholesky_log2_determinant_3x3(matrix).expect("positive definite");
        assert!((log2_det - 44.0_f64.log2()).abs() < 1e-9);
    }

    #[test]
    fn cholesky_log2_determinant_rejects_non_positive_definite_matrices() {
        // Zero matrix: a00 <= floor.
        assert!(super::cholesky_log2_determinant_3x3([[0.0; 3]; 3]).is_none());

        // Repeated first row/column: rank-deficient (singular) despite a00 > 0.
        let singular = [[1.0, 1.0, 0.0], [1.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
        assert!(super::cholesky_log2_determinant_3x3(singular).is_none());
    }

    // --- both MI methods agree on non-degenerate synthetic covariances ---

    #[test]
    fn both_mi_methods_agree_on_the_well_behaved_synthetic_fixture_at_every_horizon() {
        let records = super::synthetic_smoke_records();

        for horizon_index in 0..TARGET_HORIZON_COUNT {
            let target = super::target_values(&records, horizon_index);
            let source_1 = super::source_overlap_values(&records, 0);
            let source_2 = super::source_overlap_values(&records, 1);

            let cov = super::triple_covariance(&target, &source_1, &source_2).unwrap();
            let m1 = super::mutual_information_method_1(&cov);
            let m2 = super::mutual_information_method_2(&cov);

            assert!(m1.i_t_s1.is_finite(), "horizon {horizon_index}");
            assert!(m1.i_t_s2.is_finite(), "horizon {horizon_index}");
            assert!(m1.i_t_joint.is_finite(), "horizon {horizon_index}");

            let agreement = super::cross_method_agreement(&m1, &m2);
            assert!(
                agreement.within_tolerance,
                "horizon {horizon_index}: max diff {}",
                agreement.max_absolute_difference
            );
            assert!(agreement.max_absolute_difference <= PID_CROSS_METHOD_TOLERANCE);
        }
    }

    // --- degenerate case: perfectly collinear S1, S2 (Section 6) ---

    /// S2 = 2*S1 exactly; T carries its own independent-ish noise beyond
    /// S1/S2, so I(T;S1) and I(T;S2) remain finite (and, since S2 is an
    /// affine function of S1, identical) while only the joint term is
    /// undefined -- verified numerically before being committed here.
    fn collinear_s1_s2_records() -> Vec<Record> {
        (0..12)
            .map(|i| {
                let s1 = 0.1 + 0.05 * i as f64;
                let s2 = 2.0 * s1;
                let noise = 0.02 * ((i * 7 + 3) % 5) as f64;
                let t = 0.5 * s1 + 0.9 * noise;
                record_with(s1, s2, t)
            })
            .collect()
    }

    #[test]
    fn collinear_s1_s2_produces_nan_joint_in_both_methods_but_finite_finite_marginals() {
        let records = collinear_s1_s2_records();
        let target = super::target_values(&records, 0);
        let source_1 = super::source_overlap_values(&records, 0);
        let source_2 = super::source_overlap_values(&records, 1);

        let cov = super::triple_covariance(&target, &source_1, &source_2).unwrap();
        let det_s1_s2 = cov.var_s1 * cov.var_s2 - cov.cov_s1_s2 * cov.cov_s1_s2;
        assert!(
            det_s1_s2.abs() < 1e-9,
            "S1,S2 must be collinear: {det_s1_s2}"
        );

        let m1 = super::mutual_information_method_1(&cov);
        let m2 = super::mutual_information_method_2(&cov);

        assert!(m1.i_t_s1.is_finite());
        assert!(m1.i_t_s2.is_finite());
        assert!((m1.i_t_s1 - m1.i_t_s2).abs() < 1e-9);
        assert!(m1.i_t_joint.is_nan(), "method 1 joint must be NaN");
        assert!(m2.i_t_joint.is_nan(), "method 2 joint must be NaN");

        let pid = super::assemble_pid(&m1);
        assert!(!pid.is_finite());
        assert_eq!(super::dominant_component(&pid), None);
    }

    // --- rank-deficient 3-variable case (Section 6) ---

    /// `T`, `S1`, `S2` are all deterministic functions of only two
    /// independent underlying draws `a`, `b` (no independent component of
    /// `T`'s own): the 3-variable system is rank <= 2, so the FULL 3x3
    /// covariance is singular even though the 2x2 (S1, S2) sub-block is not
    /// -- verified numerically before being committed here. This is correct
    /// NaN-producing behaviour (Section 6), not a bug: see
    /// `synthetic_smoke_records`'s documentation for the contrasting
    /// well-behaved construction.
    fn rank_deficient_records() -> Vec<Record> {
        (0..12)
            .map(|i| {
                let a = 0.1 + 0.03 * i as f64;
                let b = 0.2 + 0.02 * ((i * i) % 5) as f64;
                let s1 = a + b;
                let s2 = 2.0 * a - b;
                let t = 0.4 * a + 0.6 * b;
                record_with(s1, s2, t)
            })
            .collect()
    }

    #[test]
    fn rank_deficient_three_variable_system_produces_shared_nan_joint_and_agreement() {
        let records = rank_deficient_records();
        let target = super::target_values(&records, 0);
        let source_1 = super::source_overlap_values(&records, 0);
        let source_2 = super::source_overlap_values(&records, 1);

        let cov = super::triple_covariance(&target, &source_1, &source_2).unwrap();
        let det_s1_s2 = cov.var_s1 * cov.var_s2 - cov.cov_s1_s2 * cov.cov_s1_s2;
        assert!(
            det_s1_s2 > PID_DEGENERACY_PIVOT_FLOOR,
            "S1, S2 alone must NOT be collinear: det_s1_s2={det_s1_s2}"
        );

        let m1 = super::mutual_information_method_1(&cov);
        let m2 = super::mutual_information_method_2(&cov);

        assert!(m1.i_t_s1.is_finite());
        assert!(m1.i_t_s2.is_finite());
        assert!(
            m1.i_t_joint.is_nan(),
            "method 1 joint must be NaN (full 3x3 is singular)"
        );
        assert!(
            m2.i_t_joint.is_nan(),
            "method 2 joint must be NaN (full 3x3 is singular)"
        );

        // Both methods report I(T;S1)/I(T;S2) identically (the same
        // bivariate formula) and both report the joint as degenerate: a
        // shared degeneracy, not a disagreement.
        let agreement = super::cross_method_agreement(&m1, &m2);
        assert!(
            agreement.within_tolerance,
            "a shared NaN joint must count as agreement, not disagreement"
        );
    }

    // --- PID identity and non-negativity on non-degenerate inputs ---

    #[test]
    fn pid_identity_holds_and_all_components_are_non_negative_at_every_horizon() {
        let records = super::synthetic_smoke_records();

        for horizon_index in 0..TARGET_HORIZON_COUNT {
            let target = super::target_values(&records, horizon_index);
            let source_1 = super::source_overlap_values(&records, 0);
            let source_2 = super::source_overlap_values(&records, 1);

            let cov = super::triple_covariance(&target, &source_1, &source_2).unwrap();
            let mi = super::mutual_information_method_1(&cov);
            let pid = super::assemble_pid(&mi);

            assert!(pid.is_finite(), "horizon {horizon_index}");
            assert!(pid.redundancy >= -1e-9, "horizon {horizon_index}");
            assert!(pid.unique_1 >= -1e-9, "horizon {horizon_index}");
            assert!(pid.unique_2 >= -1e-9, "horizon {horizon_index}");
            assert!(pid.synergy >= -1e-9, "horizon {horizon_index}");

            let sum = pid.redundancy + pid.unique_1 + pid.unique_2 + pid.synergy;
            assert!(
                (sum - pid.joint).abs() < 1e-9,
                "horizon {horizon_index}: Red+Un1+Un2+Syn={sum} joint={}",
                pid.joint
            );

            let proportions = pid.proportions().expect("positive joint MI");
            let proportion_sum: f64 = proportions.iter().sum();
            assert!(
                (proportion_sum - 1.0).abs() < 1e-9,
                "horizon {horizon_index}"
            );
        }
    }

    #[test]
    fn dominant_component_returns_none_on_a_non_finite_decomposition() {
        let non_finite = super::PartialInformationDecomposition {
            redundancy: f64::NAN,
            unique_1: 0.0,
            unique_2: 0.0,
            synergy: 0.0,
            joint: f64::NAN,
        };

        assert_eq!(super::dominant_component(&non_finite), None);
    }

    #[test]
    fn dominant_component_picks_the_largest_finite_component() {
        let pid = super::PartialInformationDecomposition {
            redundancy: 0.1,
            unique_1: 0.9,
            unique_2: 0.05,
            synergy: 0.2,
            joint: 1.25,
        };

        assert_eq!(super::dominant_component(&pid), Some(PidComponent::Unique1));
    }

    // --- pid_bootstrap (Section 11) ---

    #[test]
    fn pid_bootstrap_ci_contains_or_is_close_to_the_point_estimate() {
        let records = super::synthetic_smoke_records();
        let target = super::target_values(&records, PRIMARY_HORIZON_TEST_INDEX);
        let source_1 = super::source_overlap_values(&records, 0);
        let source_2 = super::source_overlap_values(&records, 1);

        let cov = super::triple_covariance(&target, &source_1, &source_2).unwrap();
        let point = super::assemble_pid(&super::mutual_information_method_1(&cov));
        assert!(point.is_finite());

        let bootstrap =
            super::pid_bootstrap(&target, &source_1, &source_2, 0x1234_5678_9abc_def0, 2_000)
                .expect("bootstrap");

        // A tight fallback (not the point estimate's own CI, which by
        // construction always contains it in a healthy bootstrap): allows a
        // little slack for bootstrap sampling noise without being so loose
        // that a substantially wrong bootstrap could still pass.
        let contains_or_close = |value: f64, interval: super::ConfidenceInterval| {
            (interval.lower..=interval.upper).contains(&value)
                || (value - interval.median).abs() < 0.05
        };

        assert!(contains_or_close(point.redundancy, bootstrap.redundancy));
        // Unique(O2) is exactly 0 here (and will be for ANY input where
        // I(T;S1) != I(T;S2): under MMI redundancy, Un_i = I(T;S_i) -
        // min(I1,I2), so the smaller of the two marginal mutual informations
        // always has Unique = 0 exactly -- an inherent property of the PID
        // definition itself, preregistration Section 5, not a fixture
        // artifact). The assertion below still checks the bootstrap
        // reproduces that same exact degeneracy rather than assuming it.
        assert!(contains_or_close(point.unique_1, bootstrap.unique_1));
        assert_eq!(point.unique_2, 0.0);
        assert_eq!(bootstrap.unique_2.lower, 0.0);
        assert_eq!(bootstrap.unique_2.upper, 0.0);
        assert!(contains_or_close(point.synergy, bootstrap.synergy));
        assert!(contains_or_close(point.joint, bootstrap.joint));
    }

    const PRIMARY_HORIZON_TEST_INDEX: usize = 3;

    #[test]
    fn guarded_pid_bootstrap_matches_raw_pid_bootstrap_on_healthy_data() {
        let records = super::synthetic_smoke_records();
        let target = super::target_values(&records, PRIMARY_HORIZON_TEST_INDEX);
        let source_1 = super::source_overlap_values(&records, 0);
        let source_2 = super::source_overlap_values(&records, 1);

        let seed = 0x9999_1111_2222_3333;
        let raw = super::pid_bootstrap(&target, &source_1, &source_2, seed, 300).expect("raw");
        let guarded = super::guarded_pid_bootstrap(&target, &source_1, &source_2, seed, 300)
            .expect("guarded bootstrap must succeed on healthy data");

        assert_eq!(raw.redundancy, guarded.redundancy);
        assert_eq!(raw.unique_1, guarded.unique_1);
        assert_eq!(raw.unique_2, guarded.unique_2);
        assert_eq!(raw.synergy, guarded.synergy);
        assert_eq!(raw.joint, guarded.joint);
    }

    // --- Hard finiteness guard (fail loud, before any output) ---

    #[test]
    fn compute_block_pid_hard_fails_on_a_degenerate_point_estimate_instead_of_reporting_nan() {
        let records = collinear_s1_s2_records();

        let error = super::compute_block_pid(SeedBlockId::S, &records, 0, 0x1)
            .expect_err("a collinear S1,S2 covariance must hard-fail, not report a NaN result");

        assert!(error.contains("degenerate-decomposition"));
    }

    #[test]
    fn compute_aggregate_pid_hard_fails_on_a_degenerate_point_estimate_instead_of_reporting_nan() {
        let records = rank_deficient_records();

        let error = super::compute_aggregate_pid(&records, 0)
            .expect_err("a rank-deficient covariance must hard-fail, not report a NaN result");

        assert!(error.contains("degenerate-decomposition"));
    }

    #[test]
    fn failure_category_display_includes_the_new_degenerate_decomposition_label() {
        assert_eq!(
            super::FailureCategory::DegenerateDecomposition.to_string(),
            "degenerate-decomposition"
        );
    }

    // --- Criteria assembly pure helpers (Sections 14, 15) ---

    #[test]
    fn cross_block_consistency_true_when_all_blocks_agree() {
        let dominant = [
            Some(PidComponent::Synergy),
            Some(PidComponent::Synergy),
            Some(PidComponent::Synergy),
        ];
        assert!(super::cross_block_dominant_component_consistent(&dominant));
    }

    #[test]
    fn cross_block_consistency_false_when_any_block_disagrees() {
        let dominant = [
            Some(PidComponent::Synergy),
            Some(PidComponent::Unique1),
            Some(PidComponent::Synergy),
        ];
        assert!(!super::cross_block_dominant_component_consistent(&dominant));
    }

    #[test]
    fn cross_block_consistency_treats_shared_none_as_agreement() {
        let dominant = [None, None, None];
        assert!(super::cross_block_dominant_component_consistent(&dominant));
    }

    #[test]
    fn dominant_shift_summary_detects_stability() {
        let horizons = [3, 4, 5, 6, 7, 8];
        let dominant = [Some(PidComponent::Synergy); 6];
        let (stable, shifts) = super::dominant_component_shift_summary(&horizons, &dominant);
        assert!(stable);
        assert!(shifts.is_empty());
    }

    #[test]
    fn dominant_shift_summary_reports_the_shift_horizon() {
        let horizons = [3, 4, 5, 6, 7, 8];
        let dominant = [
            Some(PidComponent::Synergy),
            Some(PidComponent::Unique1),
            Some(PidComponent::Unique1),
            Some(PidComponent::Unique1),
            Some(PidComponent::Unique1),
            Some(PidComponent::Unique1),
        ];
        let (stable, shifts) = super::dominant_component_shift_summary(&horizons, &dominant);
        assert!(!stable);
        assert_eq!(shifts, vec![4]);
    }

    #[test]
    fn dominant_shift_summary_reports_every_shift_horizon_when_it_oscillates() {
        let horizons = [3, 4, 5, 6, 7, 8];
        let dominant = [
            Some(PidComponent::Redundancy),
            Some(PidComponent::Synergy),
            Some(PidComponent::Synergy),
            Some(PidComponent::Redundancy),
            Some(PidComponent::Redundancy),
            Some(PidComponent::Unique2),
        ];
        let (stable, shifts) = super::dominant_component_shift_summary(&horizons, &dominant);
        assert!(!stable);
        assert_eq!(shifts, vec![4, 6, 8]);
    }

    // --- BlockPopulations::combined_all_records (Section 4.5) ---

    #[test]
    fn combined_all_records_pools_all_four_populations_in_order() {
        let records = super::synthetic_smoke_records();
        let block = super::synthetic_block_populations(SeedBlockId::S, 1, &records);
        let combined = block.combined_all_records();

        let expected_len = 4 * records.len();
        assert_eq!(combined.len(), expected_len);

        let overlaps_combined: Vec<[f64; 2]> = combined.iter().map(|r| r.early_overlap).collect();
        let overlaps_expected: Vec<[f64; 2]> = records.iter().map(|r| r.early_overlap).collect();

        let n = records.len();
        assert_eq!(&overlaps_combined[..n], overlaps_expected.as_slice());
        assert_eq!(&overlaps_combined[n..2 * n], overlaps_expected.as_slice());
        assert_eq!(
            &overlaps_combined[2 * n..3 * n],
            overlaps_expected.as_slice()
        );
        assert_eq!(&overlaps_combined[3 * n..], overlaps_expected.as_slice());
    }

    // --- Populations and seed blocks (Sections 7, 8) ---

    #[test]
    fn population_specs_total_twelve_four_per_block_and_have_no_ood() {
        let specs = super::population_specs();
        assert_eq!(specs.len(), TOTAL_SEED_RESERVATIONS);
        assert_eq!(specs.len(), 12);
        for block in super::FROZEN_BLOCK_ORDER {
            assert_eq!(specs.iter().filter(|s| s.seed_block == block).count(), 4);
        }
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
    fn seed_blocks_are_stu_and_start_at_eight_billion_disjoint_from_every_prior_block() {
        let ids: Vec<_> = SEED_BLOCKS.iter().map(|b| b.id).collect();
        assert_eq!(ids, vec![SeedBlockId::S, SeedBlockId::T, SeedBlockId::U]);

        for block in SEED_BLOCKS {
            for seed in [
                block.training_width_3_seed,
                block.holdout_width_3_seed,
                block.training_width_4_seed,
                block.holdout_width_4_seed,
            ] {
                assert!(seed >= 8_000_000_000);
                // TDI-5.8 tops out at 7.81e9; TDI-6.3 must start strictly above it.
                assert!(seed > 7_810_000_000);
            }
        }

        let boots: Vec<_> = SEED_BLOCKS.iter().map(|b| b.bootstrap_seed).collect();
        assert_eq!(
            boots,
            vec![
                0x5444_4936_3300_0001_u64,
                0x5444_4936_3300_0002,
                0x5444_4936_3300_0003
            ]
        );
        assert_eq!(AGGREGATE_BOOTSTRAP_SEED, 0x5444_4936_3300_4700);
        assert!(!boots.contains(&AGGREGATE_BOOTSTRAP_SEED));
    }

    // --- Inherited frozen invariants (unchanged machinery) ---

    #[test]
    fn width_6_successor_space_is_exact_two_to_the_sixty_four() {
        assert_eq!(
            super::successor_set_space_cardinality(6),
            super::Cardinality::Exact(18_446_744_073_709_551_616_u128)
        );
    }

    #[test]
    fn primary_horizon_is_six_and_target_horizons_are_frozen() {
        assert_eq!(PRIMARY_HORIZON, 6);
        assert_eq!(TARGET_HORIZONS, [3, 4, 5, 6, 7, 8]);
        assert_eq!(TARGET_HORIZONS[super::primary_horizon_index()], 6);
    }

    #[test]
    fn focal_horizon_indices_are_u3_and_u6() {
        let indices = super::focal_horizon_indices();
        assert_eq!(FOCAL_HORIZONS, [3, 6]);
        assert_eq!(TARGET_HORIZONS[indices[0]], 3);
        assert_eq!(TARGET_HORIZONS[indices[1]], 6);
        assert_eq!(indices, [0, 3]);
        assert_eq!(indices.len(), FOCAL_HORIZON_COUNT);
    }

    #[test]
    fn splitmix_is_deterministic() {
        assert_eq!(super::splitmix64(0), super::splitmix64(0));
        assert_ne!(super::splitmix64(1), super::splitmix64(2));
    }

    #[test]
    fn bootstrap_replicate_count_is_four_thousand() {
        assert_eq!(PID_BOOTSTRAP_REPLICATES, 4_000);
    }

    #[test]
    fn declared_tolerances_match_the_preregistration() {
        assert_eq!(PID_CROSS_METHOD_TOLERANCE, 1.0e-9);
        assert_eq!(PID_DEGENERACY_PIVOT_FLOOR, 1.0e-12);
    }

    // --- Generation determinism (inherited machinery) ---

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
            assert_eq!(a.targets_u, b.targets_u);
        }

        for record in &first.records {
            for &value in &record.contraction {
                assert!(value.is_finite() && (0.0..=1.0).contains(&value));
            }
            for &value in &record.spectral {
                assert!(value.is_finite() && (0.0..=8.0).contains(&value));
            }
        }
    }

    // --- End-to-end pipeline wiring (tiny, bounded, real generation) ---

    #[test]
    fn run_tdi63_pipeline_wires_all_criteria_on_tiny_specs() {
        let tiny_specs = super::population_specs().map(|spec| super::PopulationSpec {
            target_count: 3,
            ..spec
        });

        let report = super::run_tdi63_pipeline(&tiny_specs).expect("tiny end-to-end pipeline run");

        assert_eq!(report.criterion_a.focal.len(), FOCAL_HORIZON_COUNT);
        assert_eq!(report.criterion_b.grid.len(), TARGET_HORIZON_COUNT);
        assert_eq!(report.criterion_c.focal.len(), FOCAL_HORIZON_COUNT);
        assert_eq!(report.blocks.len(), super::SEED_BLOCK_COUNT);
        assert_eq!(report.descriptor_diagnostics.len(), super::SEED_BLOCK_COUNT);
    }

    // --- Termination smoke (Section 16) ---

    #[test]
    fn termination_smoke_runs_to_completion() {
        super::run_termination_smoke().expect("termination smoke must succeed");
    }

    #[test]
    fn termination_smoke_never_calls_the_real_pipeline_or_the_real_preregistered_counts() {
        let source = evaluator_source();
        let start = source
            .find("fn run_termination_smoke()")
            .expect("run_termination_smoke must exist");
        let end = source[start..]
            .find("\nfn tdi63_full_run_confirmed")
            .map(|offset| start + offset)
            .expect("tdi63_full_run_confirmed must follow run_termination_smoke");
        let body = &source[start..end];

        // The PID-pipeline exercise (report assembly, criteria, printing)
        // must run only on the synthetic records built in-memory below;
        // it must never call the real pipeline entrypoint, generate a real
        // *population* (`generate_block_populations`), or derive its scale
        // from the real preregistered record counts. A single tiny (1
        // accepted record), bounded, real candidate generation via
        // `generate_records_with_limits` IS present, as an inherited-
        // machinery sanity check separate from the PID exercise -- exactly
        // mirroring TDI-5.6's own termination smoke -- so that call is
        // deliberately not asserted against here.
        assert!(
            !body.contains("run_tdi63_pipeline("),
            "the smoke path must never run the real pipeline"
        );
        assert!(
            !body.contains("generate_block_populations("),
            "the smoke path must never generate a real population"
        );
        assert!(
            !body.contains("population_specs().map"),
            "the smoke path must never derive tiny specs from the real preregistered counts"
        );
        assert!(
            body.contains("synthetic_smoke_records()"),
            "the smoke path must exercise the PID pipeline via the synthetic record set"
        );
        assert!(
            body.contains("assemble_tdi63_report("),
            "the smoke path must exercise the real report-assembly entrypoint"
        );
    }

    // --- Full-run confirmation guard (Section 16) ---

    #[test]
    fn full_run_confirmation_accepts_only_the_exact_value() {
        assert!(super::tdi63_full_run_confirmed(Some(
            TDI63_FULL_RUN_CONFIRMATION_VALUE
        )));
        assert!(!super::tdi63_full_run_confirmed(None));
        assert!(!super::tdi63_full_run_confirmed(Some("")));
        assert!(!super::tdi63_full_run_confirmed(Some(
            "i_accept_the_tdi63_freeze_rule"
        )));
        // The frozen TDI-5.6 token must never unlock TDI-6.3.
        assert!(!super::tdi63_full_run_confirmed(Some(
            "I_ACCEPT_THE_TDI56_FREEZE_RULE"
        )));
    }

    #[test]
    fn parse_mode_rejects_a_bare_no_argument_invocation() {
        assert!(super::tdi63_parse_mode(&[]).is_err());
        assert!(super::tdi63_parse_mode(&["--full".to_owned(), "extra".to_owned()]).is_err());
    }

    #[test]
    fn parse_mode_selects_full_only_for_the_exact_single_flag() {
        assert_eq!(
            super::tdi63_parse_mode(&["--full".to_owned()]).unwrap(),
            super::Tdi63Mode::Full
        );
        assert_eq!(
            super::tdi63_parse_mode(&["--preflight".to_owned()]).unwrap(),
            super::Tdi63Mode::Preflight
        );
        assert_eq!(
            super::tdi63_parse_mode(&["--termination-smoke".to_owned()]).unwrap(),
            super::Tdi63Mode::TerminationSmoke
        );
        assert!(super::tdi63_parse_mode(&["--Full".to_owned()]).is_err());
    }

    #[test]
    fn usage_error_mentions_every_flag_and_the_confirmation_variable() {
        let usage = super::tdi63_usage_error();
        assert!(usage.contains("--termination-smoke"));
        assert!(usage.contains("--preflight"));
        assert!(usage.contains("--full"));
        assert!(usage.contains(TDI63_FULL_RUN_CONFIRMATION_VAR));
        assert!(usage.contains(TDI63_FULL_RUN_CONFIRMATION_VALUE));
    }

    #[test]
    fn full_run_refuses_before_any_work_without_the_confirmation_token() {
        // Never reach the accepted path in a test: assert the guard var is
        // absent first, then confirm the unconfirmed call returns an error
        // before any generation, decomposition or bootstrap.
        if std::env::var(TDI63_FULL_RUN_CONFIRMATION_VAR).is_ok() {
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
            body.contains("run_tdi63_pipeline(&population_specs())"),
            "accepted path must call the real pipeline over the real specs"
        );
        assert!(body.contains("tdi63_full_run_confirmed"));
        assert!(body.contains("print_tdi52_required_raw_output"));
    }

    #[test]
    fn preflight_runs_without_generating_any_population_and_mentions_the_real_run_command() {
        // `run_preflight` never calls `generate_block_populations` /
        // `run_tdi63_pipeline`; confirm it completes and prints the exact
        // real-run command line.
        super::run_preflight().expect("preflight must succeed without generating anything");

        let source = evaluator_source();
        let start = source
            .find("fn run_preflight()")
            .expect("run_preflight must exist");
        let end = source[start..]
            .find("\n#[cfg(test)]")
            .map(|offset| start + offset)
            .expect("the test module must follow run_preflight");
        let body = &source[start..end];

        assert!(!body.contains("generate_block_populations("));
        assert!(body.contains("reproduce-tdi6.3.sh"));
    }

    // --- Required raw output substrings (grepped by the reproduction script) ---

    #[test]
    fn required_output_phrases_are_present_in_the_termination_smoke_source_wiring() {
        // `print_tdi52_required_raw_output` (exercised by both `--full` and
        // `--termination-smoke`) must print all four phrases the
        // reproduction script's completion check greps for.
        let source = evaluator_source();
        assert!(source.contains("VERDICTS FINAUX"));
        assert!(source.contains("TDI-6.3A"));
        assert!(source.contains("TDI-6.3B"));
        assert!(source.contains("TDI-6.3C"));
    }

    // --- Frozen ancestors must never change under TDI-6.3 (TDI-5.1 -> TDI-5.8, TDI-6.1, TDI-6.2, TDI-6.5) ---

    #[test]
    fn frozen_ancestor_hashes_are_unchanged() {
        for (label, evaluator, evaluator_hash, prereg, prereg_hash) in super::FROZEN_ANCESTOR_CHAIN
        {
            assert_eq!(
                &super::tdi52_sha256_of_repo_file(evaluator),
                evaluator_hash,
                "frozen ancestor evaluator changed: {label}"
            );
            assert_eq!(
                &super::tdi52_sha256_of_repo_file(prereg),
                prereg_hash,
                "frozen ancestor preregistration changed: {label}"
            );
        }
        assert_eq!(super::FROZEN_ANCESTOR_CHAIN.len(), 11);
    }

    // --- Analytical PID regression cases (precomputed ground truth,
    // independently verified in Python against these exact Gaussian/MMI
    // formulas before being hardcoded here; see preregistration Sections 5-6).
    // These are implementation-oracle tests exercising the frozen formulas
    // directly on hand-picked covariances -- distinct from, and not to be
    // confused with, TDI-6.3's confirmatory descriptive criteria (Sections
    // 13-15), which run only on real generated data and are never adapted to
    // match these fixtures or any other observed result. ---

    fn unit_variance_covariance(
        cov_t_s1: f64,
        cov_t_s2: f64,
        cov_s1_s2: f64,
    ) -> super::TripleCovariance {
        super::TripleCovariance {
            mean_t: 0.0,
            mean_s1: 0.0,
            mean_s2: 0.0,
            var_t: 1.0,
            var_s1: 1.0,
            var_s2: 1.0,
            cov_t_s1,
            cov_t_s2,
            cov_s1_s2,
        }
    }

    fn pid_from_covariance(
        cov: &super::TripleCovariance,
    ) -> super::PartialInformationDecomposition {
        super::assemble_pid(&super::mutual_information_method_1(cov))
    }

    /// Case A' -- near-identical (not exactly identical) sources: rho_s1_s2 =
    /// 0.999, both sources equally correlated with T. Clean
    /// redundancy-dominant case: Red approx= I1 approx= I2, Un1 approx= Un2
    /// approx= 0, and a small but strictly positive Syn (NOT exactly zero --
    /// see the exact-duplicate case immediately below for the boundary this
    /// sits just short of).
    #[test]
    fn case_a_prime_near_identical_sources_is_redundancy_dominant_with_small_positive_synergy() {
        let cov = unit_variance_covariance(0.5, 0.5, 0.999);
        let pid = pid_from_covariance(&cov);

        assert!(pid.is_finite());
        assert!((pid.redundancy - 0.207518749639422).abs() < 1e-9);
        assert!(pid.unique_1.abs() < 1e-9);
        assert!(pid.unique_2.abs() < 1e-9);
        assert!((pid.synergy - 0.00012029475896555).abs() < 1e-9);
        assert!(
            pid.synergy > 0.0,
            "synergy must be strictly positive, not exactly zero"
        );
        assert!((pid.joint - 0.207639044398387).abs() < 1e-9);
    }

    /// Case A -- EXACT source duplication (S2 = S1 exactly, rho_s1_s2 = 1.0
    /// exactly): the degenerate boundary case that Case A' deliberately sits
    /// just short of. det(Sigma_{S1,S2}) = 1 - 1*1 = 0 exactly, so the joint
    /// mutual information (and hence the whole PID) is correctly NaN under
    /// both methods -- this is the intended behaviour (preregistration
    /// Section 6), not a bug, and is distinct from Case A' precisely because
    /// exact duplication, unlike near-duplication, admits no well-defined
    /// joint decomposition at all.
    #[test]
    fn case_a_exact_duplicate_sources_is_the_degenerate_boundary_not_a_finite_case() {
        let cov = unit_variance_covariance(0.5, 0.5, 1.0);
        let det_s1_s2 = cov.var_s1 * cov.var_s2 - cov.cov_s1_s2 * cov.cov_s1_s2;
        assert!(
            det_s1_s2.abs() < 1e-12,
            "S1, S2 must be exactly collinear here"
        );

        let m1 = super::mutual_information_method_1(&cov);
        let m2 = super::mutual_information_method_2(&cov);
        assert!(m1.i_t_s1.is_finite() && m1.i_t_s2.is_finite());
        assert!(
            m1.i_t_joint.is_nan(),
            "method 1 joint must be NaN on exact duplication"
        );
        assert!(
            m2.i_t_joint.is_nan(),
            "method 2 joint must be NaN on exact duplication"
        );

        let pid = pid_from_covariance(&cov);
        assert!(!pid.is_finite());
        assert_eq!(super::dominant_component(&pid), None);
    }

    /// Case B -- swapping S1 and S2 must leave Red, Syn and Ijoint invariant
    /// (the PID lattice is symmetric under source permutation) while Un1 and
    /// Un2 swap with each other exactly.
    #[test]
    fn case_b_swapping_sources_swaps_unique_terms_and_preserves_everything_else() {
        let original = unit_variance_covariance(0.5, 0.3, 0.2);
        let swapped = unit_variance_covariance(0.3, 0.5, 0.2);

        let pid_original = pid_from_covariance(&original);
        let pid_swapped = pid_from_covariance(&swapped);

        assert!(pid_original.is_finite() && pid_swapped.is_finite());
        assert!((pid_original.redundancy - 0.0680307747880142).abs() < 1e-9);
        assert!((pid_original.unique_1 - 0.139487974851408).abs() < 1e-9);
        assert!(pid_original.unique_2.abs() < 1e-9);
        assert!((pid_original.synergy - 0.0412310800959864).abs() < 1e-9);
        assert!((pid_original.joint - 0.248749829735408).abs() < 1e-9);

        assert!((pid_original.redundancy - pid_swapped.redundancy).abs() < 1e-9);
        assert!((pid_original.synergy - pid_swapped.synergy).abs() < 1e-9);
        assert!((pid_original.joint - pid_swapped.joint).abs() < 1e-9);
        assert!((pid_original.unique_1 - pid_swapped.unique_2).abs() < 1e-9);
        assert!((pid_original.unique_2 - pid_swapped.unique_1).abs() < 1e-9);
    }

    /// Case C -- one uninformative source: S2 shares no covariance with T or
    /// with S1. I(T;S2), Redundancy, Unique(O2) and Synergy must all vanish;
    /// Unique(O1) must equal the full I(T;S1).
    #[test]
    fn case_c_one_uninformative_source_yields_pure_unique_information() {
        let cov = unit_variance_covariance(0.4, 0.0, 0.0);
        let pid = pid_from_covariance(&cov);

        assert!(pid.is_finite());
        assert!((pid.unique_1 - 0.125769383497982).abs() < 1e-9);
        assert!(pid.redundancy.abs() < 1e-9);
        assert!(pid.unique_2.abs() < 1e-9);
        assert!(pid.synergy.abs() < 1e-9);
        assert!((pid.unique_1 - pid.joint).abs() < 1e-9);
    }

    /// Case E -- positive synergy: two sources independent of each other but
    /// equally correlated with T. The Gaussian/MMI model produces genuine
    /// positive synergy here without any XOR-style construction.
    #[test]
    fn case_e_independent_equally_informative_sources_produce_positive_synergy() {
        let cov = unit_variance_covariance(0.4, 0.4, 0.0);
        let pid = pid_from_covariance(&cov);

        assert!(pid.is_finite());
        assert!((pid.redundancy - 0.125769383497982).abs() < 1e-9);
        assert!(pid.unique_1.abs() < 1e-9);
        assert!(pid.unique_2.abs() < 1e-9);
        assert!((pid.synergy - 0.15242729076421).abs() < 1e-9);
        assert!(pid.synergy > pid.redundancy, "synergy must dominate here");
    }

    /// Case F -- high/pure redundancy: two strongly correlated sources,
    /// equally correlated with T. Redundancy must dominate, Un1/Un2/Syn small.
    #[test]
    fn case_f_strongly_correlated_equally_informative_sources_produce_pure_redundancy() {
        let cov = unit_variance_covariance(0.6, 0.6, 0.95);
        let pid = pid_from_covariance(&cov);

        assert!(pid.is_finite());
        assert!((pid.redundancy - 0.321928094887362).abs() < 1e-9);
        assert!(pid.unique_1.abs() < 1e-9);
        assert!(pid.unique_2.abs() < 1e-9);
        assert!((pid.synergy - 0.0104798093178233).abs() < 1e-9);
        assert!(
            pid.redundancy > 10.0 * pid.synergy,
            "redundancy must dominate here"
        );
    }

    /// Case G -- conditionally-redundant-only source: S2's correlation with T
    /// is exactly what S1 alone predicts of it (cov_t_s2 = cov_t_s1 *
    /// cov_s1_s2, unit variances), i.e. S2 carries no information about T
    /// beyond what S1 already gives. Ijoint must equal I1 exactly (S2 adds
    /// nothing), Unique(O2) and Synergy must vanish, and Unique(O1) must
    /// dominate.
    #[test]
    fn case_g_conditionally_redundant_only_source_adds_no_unique_or_synergistic_information() {
        let cov = unit_variance_covariance(0.6, 0.3, 0.5);
        let pid = pid_from_covariance(&cov);

        assert!(pid.is_finite());
        assert!((pid.joint - 0.321928094887362).abs() < 1e-9);
        assert!((pid.unique_1 - 0.253897320099348).abs() < 1e-9);
        assert!(pid.unique_2.abs() < 1e-9);
        assert!(pid.synergy.abs() < 1e-9);
    }

    /// Case I -- affine (location-scale) invariance (preregistration Section
    /// 4.4): rescaling T, S1 and S2 independently by arbitrary positive
    /// factors, with the covariance matrix scaled accordingly, must leave
    /// every PID component exactly unchanged (to floating-point precision).
    #[test]
    fn case_i_affine_rescaling_of_all_three_variables_leaves_the_pid_unchanged() {
        let base = super::TripleCovariance {
            mean_t: 0.0,
            mean_s1: 0.0,
            mean_s2: 0.0,
            var_t: 1.0,
            var_s1: 1.0,
            var_s2: 1.0,
            cov_t_s1: 0.5,
            cov_t_s2: 0.35,
            cov_s1_s2: 0.25,
        };
        let (scale_t, scale_s1, scale_s2) = (2.0_f64, 3.0_f64, 0.5_f64);
        let rescaled = super::TripleCovariance {
            mean_t: 0.0,
            mean_s1: 0.0,
            mean_s2: 0.0,
            var_t: scale_t * scale_t * base.var_t,
            var_s1: scale_s1 * scale_s1 * base.var_s1,
            var_s2: scale_s2 * scale_s2 * base.var_s2,
            cov_t_s1: scale_t * scale_s1 * base.cov_t_s1,
            cov_t_s2: scale_t * scale_s2 * base.cov_t_s2,
            cov_s1_s2: scale_s1 * scale_s2 * base.cov_s1_s2,
        };

        let pid_base = pid_from_covariance(&base);
        let pid_rescaled = pid_from_covariance(&rescaled);

        assert!(pid_base.is_finite() && pid_rescaled.is_finite());
        assert!((pid_base.redundancy - pid_rescaled.redundancy).abs() < 1e-9);
        assert!((pid_base.unique_1 - pid_rescaled.unique_1).abs() < 1e-9);
        assert!((pid_base.unique_2 - pid_rescaled.unique_2).abs() < 1e-9);
        assert!((pid_base.synergy - pid_rescaled.synergy).abs() < 1e-9);
        assert!((pid_base.joint - pid_rescaled.joint).abs() < 1e-9);
    }

    /// PID identity, restated directly on hand-picked covariances (in
    /// addition to `pid_identity_holds_and_all_components_are_non_negative_
    /// at_every_horizon`'s check on generated data): Red + Un1 + Un2 + Syn
    /// must equal Ijoint exactly (to floating-point precision) on every
    /// non-degenerate case above.
    #[test]
    fn pid_identity_holds_on_every_analytical_regression_case() {
        let cases = [
            unit_variance_covariance(0.5, 0.5, 0.999),
            unit_variance_covariance(0.5, 0.3, 0.2),
            unit_variance_covariance(0.3, 0.5, 0.2),
            unit_variance_covariance(0.4, 0.0, 0.0),
            unit_variance_covariance(0.4, 0.4, 0.0),
            unit_variance_covariance(0.6, 0.6, 0.95),
            unit_variance_covariance(0.6, 0.3, 0.5),
        ];

        for cov in cases {
            let pid = pid_from_covariance(&cov);
            assert!(pid.is_finite());
            let sum = pid.redundancy + pid.unique_1 + pid.unique_2 + pid.synergy;
            assert!((sum - pid.joint).abs() < 1e-9, "cov={cov:?}");
            assert!(pid.redundancy >= -1e-9 && pid.unique_1 >= -1e-9);
            assert!(pid.unique_2 >= -1e-9 && pid.synergy >= -1e-9);
        }
    }

    /// Near-duplicate noise sweep: as rho_s1_s2 approaches 1 (but never
    /// reaches it), the decomposition must stay finite throughout, Redundancy
    /// must stay pinned at I1 (since I1 = I2 by this symmetric construction),
    /// Un1/Un2 must stay at (approximately) zero, and Synergy must shrink
    /// monotonically toward zero -- verified numerically in Python before
    /// being hardcoded here.
    #[test]
    fn near_duplicate_noise_sweep_stays_finite_and_synergy_shrinks_monotonically_toward_zero() {
        let rhos = [0.9, 0.99, 0.999, 0.9999, 0.99999];
        let mut previous_synergy = f64::INFINITY;

        for rho in rhos {
            let cov = unit_variance_covariance(0.5, 0.5, rho);
            let pid = pid_from_covariance(&cov);

            assert!(pid.is_finite(), "rho_s1_s2={rho}");
            assert!(
                (pid.redundancy - 0.207518749639422).abs() < 1e-9,
                "rho_s1_s2={rho}"
            );
            assert!(pid.unique_1.abs() < 1e-9, "rho_s1_s2={rho}");
            assert!(pid.unique_2.abs() < 1e-9, "rho_s1_s2={rho}");
            assert!(
                pid.synergy > 0.0,
                "rho_s1_s2={rho}: synergy must stay strictly positive"
            );
            assert!(
                pid.synergy < previous_synergy,
                "rho_s1_s2={rho}: synergy must shrink monotonically as sources near-duplicate"
            );
            previous_synergy = pid.synergy;
        }

        assert!(
            previous_synergy < 1e-5,
            "synergy must have shrunk close to zero by rho=0.99999"
        );
    }

    // --- Degeneracy rejection: determinism and non-propagation (explicit,
    // beyond compute_block_pid_hard_fails_.../compute_aggregate_pid_hard_
    // fails_...) ---

    /// The same invalid (collinear S1, S2) fixture, run through
    /// `compute_block_pid` twice independently, must produce the identical
    /// failure both times -- proving the rejection is a deterministic
    /// function of the input, not a flaky or environment-dependent check.
    #[test]
    fn the_same_degenerate_fixture_always_produces_the_same_failure() {
        let records = collinear_s1_s2_records();

        let first = super::compute_block_pid(SeedBlockId::S, &records, 0, 0x1)
            .expect_err("must fail on collinear S1, S2");
        let second = super::compute_block_pid(SeedBlockId::S, &records, 0, 0x1)
            .expect_err("must fail identically on a second, independent call");

        assert_eq!(
            first, second,
            "the same invalid fixture must fail identically every time"
        );
    }

    /// A degenerate aggregate must never silently downgrade to a partial or
    /// default result: `compute_aggregate_pid`'s `Err` must be the ONLY
    /// outcome (no `Ok` variant carrying a non-finite `AggregatePidResult` can
    /// exist), so a non-finite decomposition structurally cannot reach
    /// `assemble_tdi63_report`, the printed criteria, or a completion marker.
    #[test]
    fn a_degenerate_aggregate_can_never_reach_the_report_assembly_as_an_ok_value() {
        let records = rank_deficient_records();
        let result = super::compute_aggregate_pid(&records, 0);

        assert!(
            result.is_err(),
            "degenerate aggregate covariance must be Err, never Ok"
        );
        assert!(
            result.unwrap_err().contains("degenerate-decomposition"),
            "the failure must be tagged with the DegenerateDecomposition category"
        );
    }
}
