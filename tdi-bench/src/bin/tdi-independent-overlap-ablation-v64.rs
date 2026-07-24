//! TDI-6.4 causal-probe evaluator (does recovery depend on *which* node is
//! perturbed, for a fixed system?).
//!
//! This file is derived from the frozen TDI-5.6 evaluator
//! (`tdi-independent-overlap-ablation-v56.rs`): TDI-6.4 inherits TDI-5.6's
//! candidate generation, exclusion criteria and exact descriptor computation
//! verbatim, bit-exact, unmodified -- the single base generator (`build_system`
//! over uniform non-empty successor masks, widths 3 and 4); observation
//! geometry and target geometry `U_h = -log2(1 - O_h)`; the 13
//! structural/entropic baseline variables; the two exact contraction
//! descriptors delta, delta_bar; the two exact spectral moments s2, s3; the
//! dense horizon grid `H = {3,4,5,6,7,8}`; and the four-population-per-block
//! (training-w3, holdout-w3, training-w4, holdout-w4) seed-reservation
//! structure.
//!
//! **The single changed factor from TDI-5.6 is the perturbation protocol
//! itself** (preregistration Section 1): every TDI-5.x/6.x experiment to date
//! computes `analyze_branching_recovery` with exactly one fixed intervention
//! per generated system -- `Action::Flip { node: width - 1 }`. TDI-6.4 instead
//! computes `analyze_branching_recovery` once per possible intervention target
//! (`Action::Flip { node: i }` for every `i` in `0..width`) on the *same*
//! generated system, and compares the resulting recovery trajectories against
//! each other. This stays on the exact bit-exact-rational track throughout
//! (preregistration Section 1.2): `analyze_branching_recovery`,
//! `uniform_branching_state_distribution` and `distribution_overlap` are
//! called more times (once per node instead of once per system), not replaced
//! by a different, non-exact computation. The only floating-point step
//! anywhere is the same `U_h = -log2(1 - O_h)` transform and ordinary
//! descriptive statistics (means, ranges, correlations) every TDI-5.x exact
//! experiment already computes. TDI-6.4 therefore reproduces byte-for-byte,
//! exactly like TDI-5.2 ... 5.8, not tolerance-based like TDI-6.1/6.2/6.3/6.5.
//!
//! New in TDI-6.4
//! (`docs/TDI-6.4-CAUSAL-PROBE-PREREGISTRATION.md` Sections 5-6, 13-15):
//! exhaustive per-node analysis of every accepted record (Section 5), and the
//! three purely descriptive criteria built from comparing per-node results --
//! TDI-6.4A (per-system, per-horizon node-to-node heterogeneity range,
//! Section 13), TDI-6.4B (transfer of the early-to-late relationship across
//! intervention choice, per node index, Section 14), and TDI-6.4C (descriptor
//! correlates of heterogeneity, Section 15). None of these criteria is a
//! pass/fail classification: TDI-6.4 fits no ridge regression, no feature
//! layouts, and no Beneficial/Equivalent/Harmful/Inconclusive classifier
//! anywhere -- a causal-heterogeneity measurement has no natural "success" or
//! "failure" outcome.
//!
//! Unlike TDI-5.6's train/holdout split, TDI-6.4 pools all four populations of
//! a block (training and holdout, widths 3 and 4) before computing that
//! block's per-node analysis (Section 4.4, mirroring TDI-6.3's own pooling
//! rationale, Section 4.5): TDI-6.4 is a descriptive comparison across
//! intervention targets, not an out-of-sample predictive test, so there is no
//! fitting/prediction split.
//!
//! Frozen ancestor identities (verified at runtime and in CI): the v56
//! evaluator, the TDI-5.6 preregistration, and the full frozen chain
//! TDI-5.1 -> TDI-5.8, TDI-6.1, TDI-6.2, TDI-6.3, TDI-6.5 (every ancestor
//! evaluator and preregistration hash) are verified before any generation.
//!
//! The full run is gated behind an explicit, exact human confirmation
//! environment variable (see `run_full_experiment` and
//! `tdi64_full_run_confirmed`). No commit, test or CI run supplies that
//! token; the authoring agent never invokes `--full`.

use tdi_core::{
    Action, ExactRatio, State, TableSystem, analyze_branching_recovery, distribution_overlap,
    explore, uniform_branching_path_entropy_bits, uniform_branching_state_distribution,
};

const OBSERVATION_HORIZON: usize = 2;

// Dense target-horizon grid, inherited unchanged from TDI-5.6 (preregistration
// Section 3), so per-node heterogeneity is sampled at every integer horizon
// 3..=8. TDI-6.4 has no separate pair of "focal" horizons distinct from this
// grid (Section 10): Criterion 6.4A reports across the whole dense grid, and
// Criterion 6.4B/6.4C use the single primary horizon U6.
const TARGET_HORIZONS: [usize; 6] = [3, 4, 5, 6, 7, 8];
const TARGET_HORIZON_COUNT: usize = TARGET_HORIZONS.len();
const PRIMARY_HORIZON: usize = 6;
const PRIMARY_HORIZON_INDEX: usize = 3;

const TRAIN_WIDTH_3: u8 = 3;
const TRAIN_WIDTH_4: u8 = 4;
// Widths 5 and 6 remain supported by the inherited frozen generator and its
// exact cardinality/budget machinery, but TDI-6.4 generates no populations at
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
// unchanged from TDI-5.6: the Dobrushin coefficient and the mean pairwise
// total variation. Both are exact rationals, computed per candidate system.
// Reported as Section 12 descriptive context and consumed by Criterion 6.4C;
// not used to filter or classify anything.
const CONTRACTION_FEATURE_COUNT: usize = 2;
// Exact spectral moments of the one-step Noop kernel, inherited unchanged
// from TDI-5.6: s2 = trace(P^2) and s3 = trace(P^3), computed per candidate
// system as closed-walk sums of unit fractions with a single final rounding.
// Reported as Section 12 descriptive context and consumed by Criterion 6.4C.
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

const BOOTSTRAP_REPLICATES: usize = 4_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SeedBlockId {
    V,
    W,
    X,
}

impl SeedBlockId {
    const fn label(self) -> &'static str {
        match self {
            Self::V => "V",
            Self::W => "W",
            Self::X => "X",
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

// Three fresh seed blocks V/W/X, disjoint from every prior experiment's
// blocks (preregistration Section 8: TDI-5.7 <= 2.53e9; TDI-6.1 3.0-3.23e9;
// TDI-6.2 4.0-4.23e9; TDI-6.5 5.0-6.13e9; TDI-5.8 7.0-7.81e9; TDI-6.3
// 8.0-8.24e9 approx.; TDI-6.4 starts at 9.0e9). `base(b) = 9_000_000_000 + b *
// 100_000_000` for block index `b in {0,1,2}`; the four populations of a
// block start at `base + {0,10,20,30} * 1_000_000` (training-w3, holdout-w3,
// training-w4, holdout-w4), identical in structure to TDI-6.3 Section 8. New
// bootstrap seeds in the `0x5444_4936_3400_...` range (Section 11), disjoint
// from every prior bootstrap seed.
const SEED_BLOCKS: [SeedBlockSpec; SEED_BLOCK_COUNT] = [
    SeedBlockSpec {
        id: SeedBlockId::V,
        training_width_3_seed: 9_000_000_000,
        holdout_width_3_seed: 9_010_000_000,
        training_width_4_seed: 9_020_000_000,
        holdout_width_4_seed: 9_030_000_000,
        bootstrap_seed: 0x5444_4936_3400_0001,
    },
    SeedBlockSpec {
        id: SeedBlockId::W,
        training_width_3_seed: 9_100_000_000,
        holdout_width_3_seed: 9_110_000_000,
        training_width_4_seed: 9_120_000_000,
        holdout_width_4_seed: 9_130_000_000,
        bootstrap_seed: 0x5444_4936_3400_0002,
    },
    SeedBlockSpec {
        id: SeedBlockId::X,
        training_width_3_seed: 9_200_000_000,
        holdout_width_3_seed: 9_210_000_000,
        training_width_4_seed: 9_220_000_000,
        holdout_width_4_seed: 9_230_000_000,
        bootstrap_seed: 0x5444_4936_3400_0003,
    },
];

// Single pooled-aggregate bootstrap seed (Section 11): a plain, non-stratified
// resample of the pooled records across all three blocks, mirroring TDI-6.3
// Section 11's precedent and rationale (TDI-6.4's aggregate is likewise a
// direct pooled-record estimate, not a paired-prediction comparison).
const AGGREGATE_BOOTSTRAP_SEED: u64 = 0x5444_4936_3400_4700;

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

/// One node's causal-probe analysis (preregistration Sections 5, 5.1): the
/// early overlaps `[O_i(1), O_i(2)]` (always defined -- only the transformed
/// target can diverge, never the raw overlap), the raw target overlaps
/// `[O_i(h)]` across the dense grid (always defined), and the transformed
/// targets `[U_i(h)]` across the dense grid, each `None` exactly when node
/// `node`'s perturbation fully recovers by horizon `h` (a legitimate outcome,
/// never a rejection reason for any node other than the historical
/// `i* = width - 1`).
#[derive(Clone, Debug, PartialEq)]
struct NodeAnalysis {
    node: u8,
    early_overlap: [f64; EARLY_OVERLAP_FEATURE_COUNT],
    target_overlaps: [f64; TARGET_HORIZON_COUNT],
    target_u: [Option<f64>; TARGET_HORIZON_COUNT],
}

/// One accepted candidate. `baseline`, `early_overlap`, `contraction`,
/// `spectral`, `overlaps` and `targets_u` are the historical node's
/// (`i* = width - 1`) values, inherited unchanged from TDI-5.6 -- they are
/// computed identically to how TDI-5.6 computes its own single-perturbation
/// `Record`, and drive acceptance/rejection exactly as before. `width` and
/// `per_node` are TDI-6.4's only addition (Section 5): `per_node` has one
/// entry per node `0..width`, including a recomputed entry for the historical
/// node itself, which the Section 19 consistency check (`tests` module)
/// asserts is bit-identical to the inherited fields above.
#[derive(Clone, Debug)]
struct Record {
    baseline: [f64; BASELINE_FEATURE_COUNT],
    early_overlap: [f64; EARLY_OVERLAP_FEATURE_COUNT],
    contraction: [f64; CONTRACTION_FEATURE_COUNT],
    spectral: [f64; SPECTRAL_FEATURE_COUNT],
    overlaps: [f64; TARGET_HORIZON_COUNT],
    targets_u: [f64; TARGET_HORIZON_COUNT],
    width: u8,
    per_node: Vec<NodeAnalysis>,
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
    // Boxed so the accepted variant (a full `Record`, now carrying a per-node
    // `Vec`) does not dominate the enum size (clippy::large_enum_variant).
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

/// Exact contraction descriptors of the one-step Noop kernel, inherited
/// unchanged from TDI-5.6: the Dobrushin coefficient `delta = max_{i<j}
/// TV(P_i, P_j)` and the mean pairwise total variation `delta_bar`. Each
/// `P_s` is the exact uniform distribution over state `s`'s Noop successor
/// set (`uniform_branching_state_distribution(.., 1)`); `TV = 1 - overlap`
/// uses the inherited exact `distribution_overlap`. Both descriptors are
/// exact rationals in `[0, 1]`, converted to `f64` exactly like the early
/// overlaps.
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

/// Exact spectral moments of the one-step Noop kernel, inherited unchanged
/// from TDI-5.6: `s2 = trace(P^2)` and `s3 = trace(P^3)`, computed as
/// closed-walk sums of unit fractions accumulated with arbitrary-precision
/// `ExactRatio` addition and rounded once at the end.
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

/// Exact `U = -log2(1 - O)` transform, inherited unchanged from TDI-5.6: the
/// deficit `1 - O` is formed as an exact `BigUint` rational before any
/// floating-point conversion, and both logarithms are computed directly from
/// `BigUint` digits. TDI-6.4 calls this exact same function once per node per
/// horizon (Section 5) instead of once per system, so every `U_i(h)`
/// (historical or not) has bit-identical precision to TDI-5.6's own `U_h`.
fn exact_overlap_deficit_u(ratio: &ExactRatio) -> Result<f64, String> {
    if ratio.numerator() >= ratio.denominator() {
        return Err("conditional overlap must be strictly below one".to_owned());
    }

    let deficit_numerator = ratio.denominator() - ratio.numerator();

    let numerator_log2 = biguint_log2_from_u64_digits(&deficit_numerator.to_u64_digits())?;

    let denominator_log2 = biguint_log2_from_u64_digits(&ratio.denominator().to_u64_digits())?;

    // Finiteness/non-negativity of the transformed value is deliberately not
    // checked here: callers treat an invalid transform as a graceful
    // per-candidate exclusion (historical node) or a hard defect (any other
    // node, which should be unreachable given exact arithmetic), never here.
    Ok(denominator_log2 - numerator_log2)
}

// `normalized_entropy`, `normalized_reachable`, and `transformed_path_count`
// deliberately do not validate the finiteness of their own return values.
// `analyze_seed`'s baseline-feature assembly checks every value it collects
// from these functions in one place and turns a non-finite one into a
// graceful per-candidate exclusion (`RejectionReason::NonFiniteFeature`).
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

/// TDI-6.4's only scientific addition (preregistration Sections 5, 5.1):
/// analyzes ONE node's `Flip` perturbation, exhaustively over the dense
/// horizon grid, reusing the exact same frozen primitives
/// (`analyze_branching_recovery`, `ratio_value`, `exact_overlap_deficit_u`)
/// the historical node already uses in `analyze_seed`. Full recovery at this
/// node/horizon is a legitimate, expected outcome (Section 5.1): it is
/// recorded as `target_u[horizon_index] = None`, never turned into a
/// candidate rejection (only the historical node's full recovery, checked
/// earlier in `analyze_seed`, can reject the whole candidate). Any other kind
/// of invalid geometry (non-finite, out of `[0, 1]`) is a hard `EvaluationError`
/// rather than a graceful exclusion: given exact rational arithmetic and
/// widths <= 4 this should be unreachable, so hitting it indicates a defect in
/// this evaluator, not a legitimate rare outcome (mirroring TDI-6.3's
/// `DegenerateDecomposition` philosophy for its own unreachable-in-practice
/// guard).
fn analyze_node(
    context: AttemptContext,
    system: &TableSystem,
    reference: State,
    node: u8,
) -> Result<NodeAnalysis, EvaluationError> {
    let perturbation = Action::Flip { node };

    let observation = analyze_branching_recovery(
        system,
        reference,
        perturbation,
        Action::Noop,
        OBSERVATION_HORIZON,
    )
    .map_err(|error| {
        EvaluationError::new(
            context,
            FailureCategory::DynamicAnalysis,
            format!("per-node observation recovery failed for node {node}: {error:?}"),
        )
    })?;

    let observation_overlaps = observation.overlap_profile();

    if observation_overlaps.len() != OBSERVATION_HORIZON {
        return Err(EvaluationError::new(
            context,
            FailureCategory::Structural,
            format!(
                "expected {OBSERVATION_HORIZON} per-node observation overlaps for node {node}, \
                 received {}",
                observation_overlaps.len()
            ),
        ));
    }

    let node_first_overlap = ratio_value(&observation_overlaps[0]);
    let node_second_overlap = ratio_value(&observation_overlaps[1]);

    // Section 5.1: the raw early overlap is always defined for every node,
    // whether or not that node's own trajectory happens to fully recover by
    // the observation horizon -- only the log-transform of a *target*-horizon
    // overlap can diverge. A non-finite or out-of-range value here would be a
    // defect (unreachable given exact arithmetic at widths <= 4), so it is a
    // hard error, not a per-node `None`.
    for (label, value) in [
        ("first", node_first_overlap),
        ("second", node_second_overlap),
    ] {
        if !value.is_finite() || !(0.0..=1.0).contains(&value) {
            return Err(EvaluationError::new(
                context,
                FailureCategory::Arithmetic,
                format!(
                    "node {node} {label} observation overlap {value} is outside the valid \
                     [0, 1] range"
                ),
            ));
        }
    }

    let mut target_overlaps = [0.0_f64; TARGET_HORIZON_COUNT];
    let mut target_u: [Option<f64>; TARGET_HORIZON_COUNT] = [None; TARGET_HORIZON_COUNT];

    for (horizon_index, &horizon) in TARGET_HORIZONS.iter().enumerate() {
        let outcome =
            analyze_branching_recovery(system, reference, perturbation, Action::Noop, horizon)
                .map_err(|error| {
                    EvaluationError::new(
                        context,
                        FailureCategory::DynamicAnalysis,
                        format!(
                            "per-node target recovery failed at horizon {horizon} for node \
                             {node}: {error:?}"
                        ),
                    )
                })?;

        let overlap_ratio = outcome.final_overlap().ok_or_else(|| {
            EvaluationError::new(
                context,
                FailureCategory::Structural,
                format!("node {node} horizon {horizon} produced no overlap"),
            )
        })?;

        if outcome.fully_recovered() {
            // Section 5.1: full recovery at a non-historical node is a valid,
            // legitimate outcome, not a computational defect. Record the raw
            // overlap (exactly 1.0) and leave `target_u` as `None` for this
            // (node, horizon) cell only; never reject the whole record.
            let overlap = ratio_value(&overlap_ratio);

            if !overlap.is_finite() || overlap != 1.0 {
                return Err(EvaluationError::new(
                    context,
                    FailureCategory::Arithmetic,
                    format!(
                        "node {node} horizon {horizon} reported full recovery but its raw \
                         overlap {overlap} is not exactly 1.0"
                    ),
                ));
            }

            target_overlaps[horizon_index] = overlap;
            target_u[horizon_index] = None;
            continue;
        }

        let overlap = ratio_value(&overlap_ratio);

        if !overlap.is_finite() || !(0.0..1.0).contains(&overlap) {
            return Err(EvaluationError::new(
                context,
                FailureCategory::Arithmetic,
                format!(
                    "node {node} horizon {horizon} overlap {overlap} is outside the valid \
                     [0, 1) range for a not-fully-recovered outcome"
                ),
            ));
        }

        let node_target_u = exact_overlap_deficit_u(&overlap_ratio).map_err(|error| {
            EvaluationError::new(
                context,
                FailureCategory::Arithmetic,
                format!("cannot calculate per-node U at node {node}, horizon {horizon}: {error}"),
            )
        })?;

        if !node_target_u.is_finite() || node_target_u < 0.0 {
            return Err(EvaluationError::new(
                context,
                FailureCategory::Arithmetic,
                format!(
                    "node {node} horizon {horizon} produced a non-finite or negative \
                     transformed target {node_target_u} despite a not-fully-recovered overlap"
                ),
            ));
        }

        target_overlaps[horizon_index] = overlap;
        target_u[horizon_index] = Some(node_target_u);
    }

    Ok(NodeAnalysis {
        node,
        early_overlap: [node_first_overlap, node_second_overlap],
        target_overlaps,
        target_u,
    })
}

/// Candidate analysis, inherited unchanged from TDI-5.6 through acceptance/
/// rejection (the historical node `i* = width - 1` alone decides whether this
/// candidate is accepted), with TDI-6.4's one addition appended at the very
/// end: once a candidate would be accepted under TDI-5.6's own rule, analyze
/// every node `0..width` (Section 5), not only `i*`.
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

    // Historical-node exclusion criterion, inherited unchanged from TDI-5.6:
    // O2 = 1 at the historical node i* = width - 1.
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

        // Historical-node exclusion criterion, inherited unchanged from
        // TDI-5.6: exact deficit is zero at a target horizon.
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

    // TDI-6.4's only scientific addition (preregistration Sections 5-6): this
    // candidate is accepted under the historical node's criteria exactly as
    // TDI-5.6 would accept it; now analyze EVERY node 0..width, including a
    // recomputed entry for the historical node itself (Section 19: the
    // `tests` module asserts this recomputed entry is bit-identical to the
    // `early_overlap`/`overlaps`/`targets_u` fields already computed above).
    let mut per_node = Vec::with_capacity(usize::from(context.width));

    for node in 0..context.width {
        per_node.push(analyze_node(context, &system, reference, node)?);
    }

    // Preregistration Section 19: "the evaluator will assert that U_{i*}(h)
    // ... computed here is bit-identical to what TDI-5.6's own formula would
    // produce on the same records ... any divergence would indicate a defect
    // in this evaluator, not a scientific finding." This is a genuine
    // runtime self-check (not merely a unit test -- the `tests` module below
    // additionally exercises it against freshly generated records): the
    // historical node's recomputed `NodeAnalysis` entry must be bit-identical
    // to the `early_overlap`/`overlaps`/`targets_u` fields already computed
    // above via the unchanged TDI-5.6 logic.
    let historical_index = usize::from(context.width) - 1;
    let historical = &per_node[historical_index];

    assert_eq!(
        historical.early_overlap, early_overlap,
        "Section 19 consistency check failed: historical node's recomputed early_overlap \
         diverges from the inherited TDI-5.6 computation (width {}, seed {})",
        context.width, context.seed
    );
    assert_eq!(
        historical.target_overlaps, overlaps,
        "Section 19 consistency check failed: historical node's recomputed target_overlaps \
         diverges from the inherited TDI-5.6 computation (width {}, seed {})",
        context.width, context.seed
    );
    for (horizon_index, &value) in targets_u.iter().enumerate() {
        assert_eq!(
            historical.target_u[horizon_index],
            Some(value),
            "Section 19 consistency check failed: historical node's recomputed target_u at \
             horizon index {horizon_index} diverges from the inherited TDI-5.6 computation \
             (width {}, seed {})",
            context.width,
            context.seed
        );
    }

    Ok(CandidateOutcome::Accepted(Box::new(Record {
        baseline,
        early_overlap,
        contraction,
        spectral,
        overlaps,
        targets_u,
        width: context.width,
        per_node,
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
                format!("width {width} is not part of the TDI-6.4 preregistered populations"),
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
/// tiny test/smoke overrides can be checked with the same logic; callers that
/// specifically need the real 12-reservation contract should use
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
    /// 4.4, mirroring TDI-6.3 Section 4.5): TDI-6.4 measures per-node
    /// heterogeneity directly rather than testing out-of-sample prediction,
    /// so every accepted record from every population -- training and
    /// holdout, widths 3 and 4 -- is pooled before computing that block's
    /// analysis.
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
    /// named fields directly. TDI-6.4 has no OOD populations (Section 7).
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
    [SeedBlockId::V, SeedBlockId::W, SeedBlockId::X];

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
                "requires deterministic block order V, W, X; found {} where {} was expected",
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

/// Simple bivariate Pearson correlation, `f64`, matching the descriptive-
/// statistics precedent already established by every exact TDI-5.x/6.x
/// experiment (preregistration Section 4.3): not a ridge fit, not a
/// classifier.
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

// ===== TDI-6.4 new analysis: per-system/per-horizon heterogeneity, transfer,
// and descriptor correlates (preregistration Sections 13-15) =====

/// Splits a pooled record population by width. TDI-6.4 generates only widths
/// 3 and 4 (Section 7: single base generator, no OOD populations), so this
/// simple filter accounts for every record; the debug assertion catches a
/// defect (a record of any other width) during testing without cost in a
/// release build.
fn partition_by_width(records: &[Record]) -> (Vec<&Record>, Vec<&Record>) {
    let width_3: Vec<&Record> = records
        .iter()
        .filter(|r| r.width == TRAIN_WIDTH_3)
        .collect();
    let width_4: Vec<&Record> = records
        .iter()
        .filter(|r| r.width == TRAIN_WIDTH_4)
        .collect();

    debug_assert_eq!(width_3.len() + width_4.len(), records.len());

    (width_3, width_4)
}

/// Resamples `values` with replacement (`replicate_count` times) and
/// recomputes the mean of each resample, exactly as preregistration Section
/// 11 specifies ("recompute the mean range(h) ... on each replicate"). A
/// plain, non-stratified resample of pooled records/values, mirroring TDI-6.3
/// Section 11's precedent (used both for a block's own records, keyed by that
/// block's own bootstrap seed, and for the pooled aggregate, keyed by
/// `AGGREGATE_BOOTSTRAP_SEED`).
fn bootstrap_mean_ci(values: &[f64], seed: u64, replicate_count: usize) -> ConfidenceInterval {
    let count = values.len();
    assert!(count > 0, "bootstrap_mean_ci requires at least one value");

    let mut generator = DeterministicRng::new(seed);
    let mut means = Vec::with_capacity(replicate_count);

    for _ in 0..replicate_count {
        let mut sum = 0.0_f64;

        for _ in 0..count {
            let index = generator.index(count);
            sum += values[index];
        }

        means.push(sum / count as f64);
    }

    confidence_interval(means)
}

/// Resamples paired rows `(x1[i], x2[i], y[i])` with replacement and
/// recomputes both `corr(x1, y)` and `corr(x2, y)` on each replicate, sharing
/// one deterministic resample sequence per replicate (the same "several
/// statistics from one replicate loop" pattern TDI-5.6's own paired bootstrap
/// already uses). Used for Criterion 6.4B's `(O_i(1), O_i(2))` vs `U_i(6)`
/// pair and for Criterion 6.4C's `(delta, delta_bar)` / `(s2, s3)` pairs.
fn bootstrap_two_correlations(
    x1: &[f64],
    x2: &[f64],
    y: &[f64],
    seed: u64,
    replicate_count: usize,
) -> (ConfidenceInterval, ConfidenceInterval) {
    let count = y.len();
    assert_eq!(x1.len(), count);
    assert_eq!(x2.len(), count);
    assert!(
        count > 0,
        "bootstrap_two_correlations requires at least one row"
    );

    let mut generator = DeterministicRng::new(seed);
    let mut first = Vec::with_capacity(replicate_count);
    let mut second = Vec::with_capacity(replicate_count);

    for _ in 0..replicate_count {
        let mut resampled_x1 = Vec::with_capacity(count);
        let mut resampled_x2 = Vec::with_capacity(count);
        let mut resampled_y = Vec::with_capacity(count);

        for _ in 0..count {
            let index = generator.index(count);
            resampled_x1.push(x1[index]);
            resampled_x2.push(x2[index]);
            resampled_y.push(y[index]);
        }

        first.push(pearson_correlation(&resampled_x1, &resampled_y));
        second.push(pearson_correlation(&resampled_x2, &resampled_y));
    }

    (confidence_interval(first), confidence_interval(second))
}

/// Pools, for one horizon, every system's `range(h) = max_i U_i(h) - min_i
/// U_i(h)` (Section 5, definitions) plus every individually-defined `U_i(h)`
/// value across every node of every system (used only for the descriptive
/// median-scale reference of Section 13). A system contributes to
/// `range_values` only when every one of its nodes has a defined `U_i(h)` at
/// this horizon; otherwise it is counted in `excluded_count` and contributes
/// its own still-defined per-node values to `all_defined_u` but not a
/// `range` (Section 5.1).
struct HorizonPool {
    range_values: Vec<f64>,
    excluded_count: usize,
    all_defined_u: Vec<f64>,
}

fn horizon_pool(records: &[Record], horizon_index: usize) -> HorizonPool {
    let mut range_values = Vec::new();
    let mut excluded_count = 0_usize;
    let mut all_defined_u = Vec::new();

    for record in records {
        let mut max_u = f64::NEG_INFINITY;
        let mut min_u = f64::INFINITY;
        let mut any_missing = false;

        for node in &record.per_node {
            match node.target_u[horizon_index] {
                Some(value) => {
                    all_defined_u.push(value);
                    max_u = max_u.max(value);
                    min_u = min_u.min(value);
                }
                None => any_missing = true,
            }
        }

        if any_missing {
            excluded_count += 1;
        } else {
            range_values.push(max_u - min_u);
        }
    }

    HorizonPool {
        range_values,
        excluded_count,
        all_defined_u,
    }
}

/// One horizon's Criterion 6.4A summary (Section 13): the descriptive
/// distribution (median, IQR, min, max, mean) of `range(h)` across the
/// population, its 95% bootstrap interval on the mean (Section 11), the
/// count of systems excluded from this horizon's statistic under Section
/// 5.1's full-recovery rule, and the purely descriptive reference threshold
/// (the population's own median `U_i(h)` scale, pooled across every node of
/// every system at this horizon) plus the proportion of included systems
/// whose `range(h)` exceeds it.
#[derive(Clone, Debug, PartialEq)]
struct HorizonRangeSummary {
    horizon: usize,
    included_count: usize,
    excluded_count: usize,
    median: f64,
    iqr_lower: f64,
    iqr_upper: f64,
    min: f64,
    max: f64,
    mean: f64,
    bootstrap_mean: ConfidenceInterval,
    median_u_scale: f64,
    proportion_exceeding_u_scale: f64,
}

fn summarize_horizon_range(
    horizon: usize,
    records: &[Record],
    horizon_index: usize,
    bootstrap_seed: u64,
) -> Result<HorizonRangeSummary, String> {
    let pool = horizon_pool(records, horizon_index);

    if pool.range_values.is_empty() {
        return Err(format!(
            "horizon {horizon}: no systems with every node's U defined; cannot summarize range(h) \
             (preregistration Section 5.1 would exclude every system from this horizon's statistic)"
        ));
    }

    let mut sorted = pool.range_values.clone();
    sorted.sort_by(f64::total_cmp);

    let median = percentile(&sorted, 0.50);
    let iqr_lower = percentile(&sorted, 0.25);
    let iqr_upper = percentile(&sorted, 0.75);
    let min = sorted[0];
    let max = sorted[sorted.len() - 1];
    let mean = pool.range_values.iter().sum::<f64>() / pool.range_values.len() as f64;

    let median_u_scale = if pool.all_defined_u.is_empty() {
        0.0
    } else {
        let mut u_sorted = pool.all_defined_u.clone();
        u_sorted.sort_by(f64::total_cmp);
        percentile(&u_sorted, 0.50)
    };

    let exceeding = pool
        .range_values
        .iter()
        .filter(|&&value| value > median_u_scale)
        .count();
    let proportion_exceeding_u_scale = exceeding as f64 / pool.range_values.len() as f64;

    let bootstrap_mean =
        bootstrap_mean_ci(&pool.range_values, bootstrap_seed, BOOTSTRAP_REPLICATES);

    Ok(HorizonRangeSummary {
        horizon,
        included_count: pool.range_values.len(),
        excluded_count: pool.excluded_count,
        median,
        iqr_lower,
        iqr_upper,
        min,
        max,
        mean,
        bootstrap_mean,
        median_u_scale,
        proportion_exceeding_u_scale,
    })
}

/// Criterion TDI-6.4A (Section 13, primary, descriptive): per block and
/// pooled aggregate, the `HorizonRangeSummary` at every horizon of the dense
/// grid.
#[derive(Clone, Debug, PartialEq)]
struct Tdi64CriterionA {
    blocks: Vec<(SeedBlockId, Vec<HorizonRangeSummary>)>,
    aggregate: Vec<HorizonRangeSummary>,
}

fn compute_criterion_a(
    block_records: &[(SeedBlockId, Vec<Record>)],
    all_records: &[Record],
) -> Result<Tdi64CriterionA, String> {
    let mut blocks = Vec::with_capacity(block_records.len());

    for (seed_block, records) in block_records {
        let mut summaries = Vec::with_capacity(TARGET_HORIZON_COUNT);

        for (horizon_index, &horizon) in TARGET_HORIZONS.iter().enumerate() {
            summaries.push(summarize_horizon_range(
                horizon,
                records,
                horizon_index,
                seed_block.bootstrap_seed(),
            )?);
        }

        blocks.push((*seed_block, summaries));
    }

    let mut aggregate = Vec::with_capacity(TARGET_HORIZON_COUNT);

    for (horizon_index, &horizon) in TARGET_HORIZONS.iter().enumerate() {
        aggregate.push(summarize_horizon_range(
            horizon,
            all_records,
            horizon_index,
            AGGREGATE_BOOTSTRAP_SEED,
        )?);
    }

    Ok(Tdi64CriterionA { blocks, aggregate })
}

/// One node index's early-to-late transfer summary (Section 14):
/// `corr(O_i(1), U_i(6))` and `corr(O_i(2), U_i(6))` across the subset of a
/// fixed-width population where `U_i(6)` is defined for this node, plus the
/// excluded count and 95% bootstrap intervals on each correlation.
#[derive(Clone, Debug, PartialEq)]
struct NodeCorrelation {
    node: u8,
    included_count: usize,
    excluded_count: usize,
    corr_o1_u6: f64,
    corr_o2_u6: f64,
    bootstrap_o1_u6: ConfidenceInterval,
    bootstrap_o2_u6: ConfidenceInterval,
}

fn node_correlation(records: &[&Record], node: u8, seed: u64) -> Result<NodeCorrelation, String> {
    let primary_index = primary_horizon_index();
    let mut o1 = Vec::new();
    let mut o2 = Vec::new();
    let mut u6 = Vec::new();
    let mut excluded = 0_usize;

    for record in records {
        let node_analysis = &record.per_node[usize::from(node)];

        match node_analysis.target_u[primary_index] {
            Some(value) => {
                o1.push(node_analysis.early_overlap[0]);
                o2.push(node_analysis.early_overlap[1]);
                u6.push(value);
            }
            None => excluded += 1,
        }
    }

    if u6.is_empty() {
        return Err(format!(
            "node {node}: no systems with a defined U_i(6); cannot compute Criterion 6.4B \
             correlations (not expected on genuine generator output)"
        ));
    }

    let corr_o1_u6 = pearson_correlation(&o1, &u6);
    let corr_o2_u6 = pearson_correlation(&o2, &u6);

    let (bootstrap_o1_u6, bootstrap_o2_u6) =
        bootstrap_two_correlations(&o1, &o2, &u6, seed, BOOTSTRAP_REPLICATES);

    Ok(NodeCorrelation {
        node,
        included_count: u6.len(),
        excluded_count: excluded,
        corr_o1_u6,
        corr_o2_u6,
        bootstrap_o1_u6,
        bootstrap_o2_u6,
    })
}

/// True iff two 95% confidence intervals overlap. Used only to operationalize
/// Section 14's descriptive "stable vs. shift" language against quantities
/// already computed (the per-node bootstrap intervals), without inventing any
/// new numeric threshold.
fn confidence_intervals_overlap(left: ConfidenceInterval, right: ConfidenceInterval) -> bool {
    left.lower <= right.upper && right.lower <= left.upper
}

/// Section 14's descriptive stability summary for one width's node
/// correlations: whether every non-historical node's bootstrap interval
/// overlaps the historical node's (`stable`), or lists which node(s) do not
/// (`shift`). `nodes` must be in node-index order `0..width`, so the last
/// entry is the historical node `i* = width - 1`.
#[derive(Clone, Debug, PartialEq)]
struct NodeCorrelationStability {
    stable_o1: bool,
    stable_o2: bool,
    shifted_nodes_o1: Vec<u8>,
    shifted_nodes_o2: Vec<u8>,
}

fn node_correlation_stability(nodes: &[NodeCorrelation]) -> NodeCorrelationStability {
    let historical = nodes.last().expect("at least one node per width");
    let mut shifted_nodes_o1 = Vec::new();
    let mut shifted_nodes_o2 = Vec::new();

    for node in &nodes[..nodes.len() - 1] {
        if !confidence_intervals_overlap(historical.bootstrap_o1_u6, node.bootstrap_o1_u6) {
            shifted_nodes_o1.push(node.node);
        }

        if !confidence_intervals_overlap(historical.bootstrap_o2_u6, node.bootstrap_o2_u6) {
            shifted_nodes_o2.push(node.node);
        }
    }

    NodeCorrelationStability {
        stable_o1: shifted_nodes_o1.is_empty(),
        stable_o2: shifted_nodes_o2.is_empty(),
        shifted_nodes_o1,
        shifted_nodes_o2,
    }
}

/// Criterion TDI-6.4B (Section 14, descriptive): the pooled-aggregate-only
/// (no per-block breakdown; preregistration Section 14 reports "across the
/// pooled population" with no per-block language, unlike Sections 13 and 15)
/// per-node-index transfer summary, computed separately for width-3 systems
/// (nodes 0-2) and width-4 systems (nodes 0-3).
#[derive(Clone, Debug, PartialEq)]
struct Tdi64CriterionB {
    width_3: Vec<NodeCorrelation>,
    width_4: Vec<NodeCorrelation>,
    width_3_stability: NodeCorrelationStability,
    width_4_stability: NodeCorrelationStability,
}

fn compute_criterion_b(all_records: &[Record]) -> Result<Tdi64CriterionB, String> {
    let (width_3, width_4) = partition_by_width(all_records);

    let mut width_3_correlations = Vec::with_capacity(usize::from(TRAIN_WIDTH_3));

    for node in 0..TRAIN_WIDTH_3 {
        width_3_correlations.push(node_correlation(&width_3, node, AGGREGATE_BOOTSTRAP_SEED)?);
    }

    let mut width_4_correlations = Vec::with_capacity(usize::from(TRAIN_WIDTH_4));

    for node in 0..TRAIN_WIDTH_4 {
        width_4_correlations.push(node_correlation(&width_4, node, AGGREGATE_BOOTSTRAP_SEED)?);
    }

    let width_3_stability = node_correlation_stability(&width_3_correlations);
    let width_4_stability = node_correlation_stability(&width_4_correlations);

    Ok(Tdi64CriterionB {
        width_3: width_3_correlations,
        width_4: width_4_correlations,
        width_3_stability,
        width_4_stability,
    })
}

/// Section 12's pooled-mean descriptor diagnostic (context only, consumed by
/// no other TDI-6.4 criterion): the pooled means of the four exact
/// descriptors delta, delta_bar, s2, s3, plus `mean_baseline_grand` and
/// `mean_overlap_grand` bookkeeping. Those last two are not among Section
/// 12's four named descriptors: the verbatim-transplanted `Record`/
/// `analyze_seed` machinery still populates the 13 inherited baseline
/// features and the historical node's six raw per-horizon overlaps on every
/// record, and although no TDI-6.4 criterion reads them, this diagnostic
/// reports their grand means alongside the named descriptors purely so every
/// field the frozen `Record` carries is accounted for somewhere in the
/// required raw output (mirroring TDI-6.3's own `DescriptorDiagnostic`).
#[derive(Clone, Copy, Debug, PartialEq)]
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

/// Criterion TDI-6.4C (Section 15, descriptive): the simple correlation of
/// per-system `range(6)` against each system's own four exact descriptors
/// (delta, delta_bar, s2, s3; Section 12), over the same included-at-U6
/// subset Criterion 6.4A already establishes for horizon 6, with 95%
/// bootstrap intervals.
#[derive(Clone, Debug, PartialEq)]
struct DescriptorCorrelations {
    included_count: usize,
    excluded_count: usize,
    delta: f64,
    delta_bar: f64,
    s2: f64,
    s3: f64,
    bootstrap_delta: ConfidenceInterval,
    bootstrap_delta_bar: ConfidenceInterval,
    bootstrap_s2: ConfidenceInterval,
    bootstrap_s3: ConfidenceInterval,
}

fn descriptor_correlations(
    records: &[Record],
    seed: u64,
) -> Result<DescriptorCorrelations, String> {
    let primary_index = primary_horizon_index();

    let mut ranges = Vec::new();
    let mut deltas = Vec::new();
    let mut delta_bars = Vec::new();
    let mut s2s = Vec::new();
    let mut s3s = Vec::new();
    let mut excluded = 0_usize;

    for record in records {
        let mut max_u = f64::NEG_INFINITY;
        let mut min_u = f64::INFINITY;
        let mut any_missing = false;

        for node in &record.per_node {
            match node.target_u[primary_index] {
                Some(value) => {
                    max_u = max_u.max(value);
                    min_u = min_u.min(value);
                }
                None => any_missing = true,
            }
        }

        if any_missing {
            excluded += 1;
            continue;
        }

        ranges.push(max_u - min_u);
        deltas.push(record.contraction[0]);
        delta_bars.push(record.contraction[1]);
        s2s.push(record.spectral[0]);
        s3s.push(record.spectral[1]);
    }

    if ranges.is_empty() {
        return Err(
            "no systems with a defined range(6); cannot compute Criterion 6.4C correlations"
                .to_owned(),
        );
    }

    let delta = pearson_correlation(&ranges, &deltas);
    let delta_bar = pearson_correlation(&ranges, &delta_bars);
    let s2 = pearson_correlation(&ranges, &s2s);
    let s3 = pearson_correlation(&ranges, &s3s);

    let (bootstrap_delta, bootstrap_delta_bar) =
        bootstrap_two_correlations(&deltas, &delta_bars, &ranges, seed, BOOTSTRAP_REPLICATES);
    let (bootstrap_s2, bootstrap_s3) =
        bootstrap_two_correlations(&s2s, &s3s, &ranges, seed, BOOTSTRAP_REPLICATES);

    Ok(DescriptorCorrelations {
        included_count: ranges.len(),
        excluded_count: excluded,
        delta,
        delta_bar,
        s2,
        s3,
        bootstrap_delta,
        bootstrap_delta_bar,
        bootstrap_s2,
        bootstrap_s3,
    })
}

/// Criterion TDI-6.4C (Section 15, descriptive): per block and pooled
/// aggregate.
#[derive(Clone, Debug, PartialEq)]
struct Tdi64CriterionC {
    blocks: Vec<(SeedBlockId, DescriptorCorrelations)>,
    aggregate: DescriptorCorrelations,
}

fn compute_criterion_c(
    block_records: &[(SeedBlockId, Vec<Record>)],
    all_records: &[Record],
) -> Result<Tdi64CriterionC, String> {
    let mut blocks = Vec::with_capacity(block_records.len());

    for (seed_block, records) in block_records {
        blocks.push((
            *seed_block,
            descriptor_correlations(records, seed_block.bootstrap_seed())?,
        ));
    }

    let aggregate = descriptor_correlations(all_records, AGGREGATE_BOOTSTRAP_SEED)?;

    Ok(Tdi64CriterionC { blocks, aggregate })
}

/// The complete TDI-6.4 report (preregistration Section 17): the raw
/// per-block populations (for population accounting), the Section 12
/// descriptor diagnostics (per block and pooled aggregate), and the three
/// descriptive criteria TDI-6.4A/B/C.
#[derive(Debug)]
struct Tdi64ExperimentReport {
    blocks: Vec<BlockPopulations>,
    descriptor_diagnostics: Vec<(SeedBlockId, DescriptorDiagnostic)>,
    aggregate_descriptor_diagnostic: DescriptorDiagnostic,
    criterion_a: Tdi64CriterionA,
    criterion_b: Tdi64CriterionB,
    criterion_c: Tdi64CriterionC,
}

/// Assembles the complete TDI-6.4 report from already-generated (or, for the
/// termination smoke, already-synthesized) `BlockPopulations`, in
/// `FROZEN_BLOCK_ORDER`. This is the generation-agnostic core of the
/// pipeline: `run_tdi64_pipeline` calls it after real generation;
/// `run_termination_smoke` calls it directly on bounded, in-memory synthetic
/// populations, so the entire heterogeneity/transfer/descriptor-correlation
/// machinery is exercised identically in both cases without the smoke path
/// ever generating a real candidate.
fn assemble_tdi64_report(blocks: Vec<BlockPopulations>) -> Result<Tdi64ExperimentReport, String> {
    let seed_blocks: Vec<SeedBlockId> = blocks.iter().map(|block| block.seed_block).collect();
    validate_frozen_block_order(&seed_blocks)?;

    // Section 4.4: pool all four populations of each block; no train/holdout
    // split.
    let block_records: Vec<(SeedBlockId, Vec<Record>)> = blocks
        .iter()
        .map(|block| (block.seed_block, block.combined_all_records()))
        .collect();

    let mut all_records = Vec::new();
    for (_, records) in &block_records {
        all_records.extend_from_slice(records);
    }

    let criterion_a = compute_criterion_a(&block_records, &all_records)?;
    let criterion_b = compute_criterion_b(&all_records)?;
    let criterion_c = compute_criterion_c(&block_records, &all_records)?;

    let descriptor_diagnostics: Vec<(SeedBlockId, DescriptorDiagnostic)> = block_records
        .iter()
        .map(|(seed_block, records)| (*seed_block, descriptor_diagnostic(records)))
        .collect();
    let aggregate_descriptor_diagnostic = descriptor_diagnostic(&all_records);

    Ok(Tdi64ExperimentReport {
        blocks,
        descriptor_diagnostics,
        aggregate_descriptor_diagnostic,
        criterion_a,
        criterion_b,
        criterion_c,
    })
}

/// Runs the full TDI-6.4 pipeline (generation of the width-3/width-4
/// populations across seed blocks V/W/X, then `assemble_tdi64_report`) over
/// an arbitrary set of population specifications. Callers control scale
/// entirely through `population_specs`: the preregistered `population_specs()`
/// output requests the real 120,000-record run, while tests pass tiny
/// synthetic-scale specs instead. `--termination-smoke` never calls this
/// function at all (see `run_termination_smoke`): it exercises
/// `assemble_tdi64_report` directly on bounded, in-memory synthetic
/// populations, so no real candidate generation ever runs on the smoke path.
/// This function is called with the real specs only from
/// `run_full_experiment`'s `--full` path, and only after that path's exact
/// confirmation-token check has passed.
fn run_tdi64_pipeline(
    population_specs: &[PopulationSpec],
) -> Result<Tdi64ExperimentReport, String> {
    validate_seed_reservations(population_specs)?;

    let mut blocks = Vec::with_capacity(SEED_BLOCK_COUNT);

    for seed_block in FROZEN_BLOCK_ORDER {
        blocks.push(
            generate_block_populations(seed_block, population_specs)
                .map_err(|error| error.to_string())?,
        );
    }

    assemble_tdi64_report(blocks)
}

// ===== Termination-smoke synthetic fixtures (no real generation) =====

/// A tiny, bounded, in-memory synthetic record set for one width, exercising
/// TDI-6.4's per-node heterogeneity/transfer/descriptor-correlation machinery
/// without any real candidate generation (the termination-smoke contract,
/// preregistration Section 16). Values vary with both the record index and
/// the node index so that Criterion 6.4A's range(h), Criterion 6.4B's
/// per-node correlations, and Criterion 6.4C's descriptor correlations are
/// all non-degenerate (nonzero variance). Exactly one deliberately chosen
/// cell (the last record's node 0, at the primary horizon U6) fully recovers,
/// so the Section 5.1 exclusion-counting path is exercised even within this
/// bounded smoke fixture, without every node hitting full recovery.
fn synthetic_smoke_records_for_width(width: u8, count: usize, base_offset: f64) -> Vec<Record> {
    let mut records = Vec::with_capacity(count);

    for k in 0..count {
        let index = k as f64 + base_offset;

        let mut per_node = Vec::with_capacity(usize::from(width));

        for node in 0..width {
            let node_index = f64::from(node);
            let mut target_overlaps = [0.0_f64; TARGET_HORIZON_COUNT];
            let mut target_u: [Option<f64>; TARGET_HORIZON_COUNT] = [None; TARGET_HORIZON_COUNT];

            for (horizon_index, &horizon) in TARGET_HORIZONS.iter().enumerate() {
                let deliberately_fully_recovered =
                    node == 0 && horizon_index == PRIMARY_HORIZON_INDEX && k + 1 == count;

                if deliberately_fully_recovered {
                    target_overlaps[horizon_index] = 1.0;
                    target_u[horizon_index] = None;
                } else {
                    let overlap =
                        0.20 + 0.01 * node_index + 0.003 * index + 0.005 * horizon_index as f64;
                    target_overlaps[horizon_index] = overlap.clamp(0.01, 0.95);
                    let target = 0.50 + 0.12 * node_index + 0.04 * index + 0.03 * horizon as f64
                        - 0.02 * (node_index * index).sin();
                    target_u[horizon_index] = Some(target.max(0.01));
                }
            }

            per_node.push(NodeAnalysis {
                node,
                early_overlap: [
                    0.10 + 0.03 * node_index + 0.006 * index,
                    0.20 + 0.02 * node_index + 0.004 * index,
                ],
                target_overlaps,
                target_u,
            });
        }

        let historical = per_node[usize::from(width) - 1].clone();

        records.push(Record {
            baseline: std::array::from_fn(|slot| 0.05 * (slot as f64 + index)),
            early_overlap: historical.early_overlap,
            contraction: [0.30 + 0.01 * index, 0.20 + 0.01 * index],
            spectral: [1.50 + 0.02 * index, 1.20 + 0.02 * index],
            overlaps: historical.target_overlaps,
            targets_u: std::array::from_fn(|horizon_index| {
                historical.target_u[horizon_index].unwrap_or(0.01)
            }),
            width,
            per_node,
        });
    }

    records
}

/// Builds a synthetic (not preregistered) `PopulationSpec` for the
/// termination smoke: same shape as `PopulationSpec::from_block`, but with an
/// arbitrary small seed/count divorced from `population_specs()`'s real
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

/// Builds one block's four synthetic populations for the termination smoke.
/// Unlike TDI-6.3's `synthetic_block_populations` (which reused one
/// width-agnostic record set for all four populations), TDI-6.4's `Record`
/// carries its own `width`, and Criterion 6.4B partitions by that width, so
/// the width-3 populations must contain only width-3 records and the width-4
/// populations only width-4 records.
fn synthetic_block_populations(
    seed_block: SeedBlockId,
    base_seed: u64,
    width_3_records: &[Record],
    width_4_records: &[Record],
) -> BlockPopulations {
    BlockPopulations {
        seed_block,
        training_width_3: synthetic_population_report(
            synthetic_population_spec(
                seed_block,
                PopulationKind::TrainingWidth3,
                base_seed,
                width_3_records.len(),
            ),
            width_3_records.to_vec(),
        ),
        holdout_width_3: synthetic_population_report(
            synthetic_population_spec(
                seed_block,
                PopulationKind::HoldoutWidth3,
                base_seed + 1,
                width_3_records.len(),
            ),
            width_3_records.to_vec(),
        ),
        training_width_4: synthetic_population_report(
            synthetic_population_spec(
                seed_block,
                PopulationKind::TrainingWidth4,
                base_seed + 2,
                width_4_records.len(),
            ),
            width_4_records.to_vec(),
        ),
        holdout_width_4: synthetic_population_report(
            synthetic_population_spec(
                seed_block,
                PopulationKind::HoldoutWidth4,
                base_seed + 3,
                width_4_records.len(),
            ),
            width_4_records.to_vec(),
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
/// shell-out convention already used by this workspace's frozen-hash tests.
/// Freeze-time artifacts (e.g. the TDI-6.4 scientific manifest) do not exist
/// yet while TDI-6.4 remains under implementation, so a missing file is
/// reported honestly rather than treated as an error.
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
/// TDI-6.3, TDI-6.5 (preregistration Sections 1, 3, 17, 20, 22; twelve
/// entries -- TDI-6.3 is itself now a frozen ancestor, having merged since
/// TDI-6.3 was built). Each entry is an (identifier, evaluator path,
/// evaluator SHA-256, preregistration path, preregistration SHA-256) tuple,
/// mirroring TDI-6.3's own `FROZEN_ANCESTOR_CHAIN`. TDI-6.4 prints this chain
/// for provenance and asserts, in a bounded test, that every hash is
/// unchanged -- a frozen ancestor changing would be a freeze violation.
///
/// TDI-5.6 is TDI-6.4's direct scientific/code ancestor (v56.rs is
/// transplanted verbatim, Section 3); every other entry is a frozen
/// predecessor whose integrity TDI-6.4 -- built after all of them -- still
/// attests to, same as TDI-6.3's own chain. Hashes were computed with
/// `sha256sum` against the actual committed files in this repository (not
/// guessed) before being pinned here and in the
/// `frozen_ancestor_hashes_are_unchanged` test.
const FROZEN_ANCESTOR_CHAIN: [(&str, &str, &str, &str, &str); 12] = [
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
        "TDI-6.3",
        "tdi-bench/src/bin/tdi-independent-overlap-ablation-v63.rs",
        "7e61baa85ae9d4b48cb1d6f527497cbb3980ae655f7f5aca40745d9bcb69e893",
        "docs/TDI-6.3-INFORMATION-DECOMPOSITION-PREREGISTRATION.md",
        "bd220fe6621d35099e729d54fa7befa18c7bf9287fe4bb1e63a449de0b96097e",
    ),
    (
        "TDI-6.5",
        "tdi-bench/src/bin/tdi-independent-overlap-ablation-v65.rs",
        "75bd5198486e7e3c6072deebbdebd256aa3152a7b43b60054349f8e181c200f0",
        "docs/TDI-6.5-GENERATOR-FAMILY-SPECTRAL-ROBUSTNESS-PREREGISTRATION.md",
        "f44eb21446ffdc6897c76818f4d4b22ecf266cf4f2707a4a8d995b0479acd589",
    ),
];

/// Provenance and integrity (TDI-6.4 preregistration Section 17, items 1-5):
/// git commit, compiler/Cargo versions, and the SHA-256 of the v64 evaluator,
/// the TDI-6.4 preregistration and the TDI-6.4 scientific manifest -- plus
/// the full frozen ancestor chain (TDI-5.1 -> TDI-5.8, TDI-6.1, TDI-6.2,
/// TDI-6.3, TDI-6.5), read live and printed for provenance.
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
        "évaluateur TDI-6.4 SHA-256      : {}",
        tdi52_sha256_of_repo_file("tdi-bench/src/bin/tdi-independent-overlap-ablation-v64.rs")
    );
    println!(
        "préenregistrement TDI-6.4 SHA-256 : {}",
        tdi52_sha256_of_repo_file("docs/TDI-6.4-CAUSAL-PROBE-PREREGISTRATION.md")
    );
    println!(
        "manifeste scientifique TDI-6.4 SHA-256 : {}",
        tdi52_sha256_of_repo_file("docs/TDI-6.4-SCIENTIFIC-CODE.sha256")
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

/// Section 17, item 6: the frozen constants. No ridge lambda, no
/// classification-model-layout counts, and no non-exact declared tolerances
/// exist in TDI-6.4 (it fits no model and stays exact throughout, Section
/// 1.2), so none is printed.
fn print_tdi52_frozen_constants() {
    println!();
    println!("=== CONSTANTES GELÉES (Section 17, item 6) ===");
    println!("horizon d'observation                       : {OBSERVATION_HORIZON}");
    println!("horizons cibles                             : {TARGET_HORIZONS:?}");
    println!("horizon principal (U6, utilisé par 6.4B/6.4C) : {PRIMARY_HORIZON}");
    println!("largeur maximale supportée                   : {MAX_SUPPORTED_WIDTH}");
    println!(
        "espace des ensembles successeurs (largeur 6) : {}",
        match successor_set_space_cardinality(WIDTH_6) {
            Cardinality::Exact(value) => value.to_string(),
            other => format!("{other:?}"),
        }
    );
    println!("nombre de features baseline                  : {BASELINE_FEATURE_COUNT}");
    println!("nombre de features early-overlap             : {EARLY_OVERLAP_FEATURE_COUNT}");
    println!("nombre de features contraction (δ, δ̄)        : {CONTRACTION_FEATURE_COUNT}");
    println!("nombre de features spectrales (s2, s3)       : {SPECTRAL_FEATURE_COUNT}");
    println!(
        "tailles de population — train w3={TRAIN_WIDTH_3_SYSTEMS}, holdout w3={HOLDOUT_WIDTH_3_SYSTEMS}, \
         train w4={TRAIN_WIDTH_4_SYSTEMS}, holdout w4={HOLDOUT_WIDTH_4_SYSTEMS} (aucune population OOD ; \
         toutes les populations d'un bloc sont regroupées, Section 4.4)"
    );
    println!(
        "multiplicateurs de tentatives — w3={WIDTH_3_ATTEMPT_MULTIPLIER}, w4={WIDTH_4_ATTEMPT_MULTIPLIER}, \
         w5={WIDTH_5_ATTEMPT_MULTIPLIER}, w6={WIDTH_6_ATTEMPT_MULTIPLIER}"
    );
    println!(
        "seuils sans-progrès — w3={WIDTH_3_NO_PROGRESS_LIMIT}, w4={WIDTH_4_NO_PROGRESS_LIMIT}, \
         w5={WIDTH_5_NO_PROGRESS_LIMIT}, w6={WIDTH_6_NO_PROGRESS_LIMIT}"
    );
    println!("réplicats bootstrap                          : {BOOTSTRAP_REPLICATES}");
    println!(
        "régime FP                                    : IEEE-754 binary64, mono-thread, ordre \
         d'opérations fixe (pas de FMA/parallèle) ; reproduction octet-exacte (Section 1.2, 20)"
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
/// counts, rejection counts by reason, final exclusive seeds, and each
/// population's generation budgets (max attempts, no-progress limit) --
/// printed alongside the raw attempted count, from which a reader can derive
/// the margin against those limits directly.
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

/// Section 12 descriptor diagnostic printer (context; also underlies
/// Criterion 6.4C): the pooled means of delta, delta_bar, s2, s3 per block
/// and for the pooled aggregate.
fn print_tdi64_descriptor_diagnostics(report: &Tdi64ExperimentReport) {
    println!();
    println!("=== DIAGNOSTIC DESCRIPTEURS — Section 12 (contexte ; sous-jacent à TDI-6.4C) ===");

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

fn print_interval(label: &str, interval: ConfidenceInterval) {
    println!(
        "{label}: [{:.9}, {:.9}] (médiane {:.9})",
        interval.lower, interval.upper, interval.median
    );
}

fn print_horizon_range_summary(label: &str, summary: &HorizonRangeSummary) {
    println!();
    println!("--- {label} — U_{} ---", summary.horizon);
    println!(
        "range(h) : n_inclus={} | n_exclus_pleine_récupération={} | médiane={:.9} | \
         IQR=[{:.9}, {:.9}] | min={:.9} | max={:.9} | moyenne={:.9}",
        summary.included_count,
        summary.excluded_count,
        summary.median,
        summary.iqr_lower,
        summary.iqr_upper,
        summary.min,
        summary.max,
        summary.mean,
    );
    print_interval(
        "  IC 95 % bootstrap (moyenne de range(h))",
        summary.bootstrap_mean,
    );
    println!(
        "  seuil descriptif (médiane U_i(h) de la population)={:.9} | proportion dépassant ce \
         seuil={:.9} (contexte seulement, non un critère pass/fail)",
        summary.median_u_scale, summary.proportion_exceeding_u_scale
    );
}

/// Criterion TDI-6.4A (Section 13, primary, descriptive): per block and
/// pooled aggregate, at every horizon of the dense grid.
fn print_tdi64_criterion_a(criterion_a: &Tdi64CriterionA) {
    println!();
    println!("=== TDI-6.4A — hétérogénéité nœud-à-nœud par système (Section 13, descriptif) ===");

    for (seed_block, summaries) in &criterion_a.blocks {
        for summary in summaries {
            print_horizon_range_summary(&format!("bloc {}", seed_block.label()), summary);
        }
    }

    for summary in &criterion_a.aggregate {
        print_horizon_range_summary("agrégat", summary);
    }
}

fn print_node_correlation(label: &str, correlation: &NodeCorrelation) {
    println!();
    println!("--- {label} — nœud {} ---", correlation.node);
    println!(
        "n_inclus={} | n_exclus (U_i(6) indéfini)={}",
        correlation.included_count, correlation.excluded_count
    );
    println!("corr(O_i(1), U_i(6))={:.9}", correlation.corr_o1_u6);
    print_interval(
        "  IC 95 % bootstrap corr(O_i(1), U_i(6))",
        correlation.bootstrap_o1_u6,
    );
    println!("corr(O_i(2), U_i(6))={:.9}", correlation.corr_o2_u6);
    print_interval(
        "  IC 95 % bootstrap corr(O_i(2), U_i(6))",
        correlation.bootstrap_o2_u6,
    );
}

fn print_stability_summary(width: u8, stability: &NodeCorrelationStability) {
    println!();
    println!(
        "largeur {width} — stabilité inter-nœuds (Section 14 ; \"stable\" opérationnalisé comme \
         chevauchement de l'IC bootstrap du nœud i* = largeur - 1 avec celui de chaque autre \
         nœud ; descriptif seulement, aucun seuil numérique nouveau) :"
    );
    println!(
        "  corr(O_i(1), U_i(6)) : {} (nœuds différant de i* : {:?})",
        if stability.stable_o1 {
            "stable"
        } else {
            "variable"
        },
        stability.shifted_nodes_o1
    );
    println!(
        "  corr(O_i(2), U_i(6)) : {} (nœuds différant de i* : {:?})",
        if stability.stable_o2 {
            "stable"
        } else {
            "variable"
        },
        stability.shifted_nodes_o2
    );
}

/// Criterion TDI-6.4B (Section 14, descriptive): the pooled-aggregate,
/// per-node-index transfer summary, separately for width 3 and width 4.
fn print_tdi64_criterion_b(criterion_b: &Tdi64CriterionB) {
    println!();
    println!(
        "=== TDI-6.4B — transfert précoce→tardif par nœud d'intervention (Section 14, \
         descriptif ; agrégat uniquement) ==="
    );

    println!();
    println!("--- largeur 3 (nœuds 0-2) ---");
    for correlation in &criterion_b.width_3 {
        print_node_correlation("agrégat, largeur 3", correlation);
    }
    print_stability_summary(TRAIN_WIDTH_3, &criterion_b.width_3_stability);

    println!();
    println!("--- largeur 4 (nœuds 0-3) ---");
    for correlation in &criterion_b.width_4 {
        print_node_correlation("agrégat, largeur 4", correlation);
    }
    print_stability_summary(TRAIN_WIDTH_4, &criterion_b.width_4_stability);
}

/// One node's mean `U_i(h)` at one horizon, over the pooled aggregate
/// (Section 17: "per-node U_i(h) summary statistics at every horizon" --
/// distinct from Criterion 6.4A's cross-node `range(h)` and from Criterion
/// 6.4B's per-node correlations at horizon 6 only; this is the plain
/// per-node-per-horizon mean the required raw output separately promises).
/// `mean_u` is `0.0` when `included_count` is zero (not expected on the real
/// 120,000-record population; the field is still well-defined and printed).
#[derive(Clone, Copy, Debug, PartialEq)]
struct NodeHorizonSummary {
    node: u8,
    horizon: usize,
    included_count: usize,
    excluded_count: usize,
    mean_u: f64,
}

fn node_horizon_summary(records: &[&Record], node: u8, horizon_index: usize) -> NodeHorizonSummary {
    let mut included_count = 0_usize;
    let mut excluded_count = 0_usize;
    let mut sum = 0.0_f64;

    for record in records {
        match record.per_node[usize::from(node)].target_u[horizon_index] {
            Some(value) => {
                included_count += 1;
                sum += value;
            }
            None => excluded_count += 1,
        }
    }

    let mean_u = if included_count > 0 {
        sum / included_count as f64
    } else {
        0.0
    };

    NodeHorizonSummary {
        node,
        horizon: TARGET_HORIZONS[horizon_index],
        included_count,
        excluded_count,
        mean_u,
    }
}

/// Prints the per-node, per-horizon `U_i(h)` mean table (Section 17) for one
/// width's records, over the pooled aggregate -- mirroring Criterion 6.4B's
/// own aggregate-only scope for per-node data (preregistration Section 14
/// reports "across the pooled population", not per block).
fn print_node_horizon_table(label: &str, records: &[&Record], width: u8) {
    println!();
    println!("--- {label} : moyenne U_i(h) par nœud et par horizon ---");

    for node in 0..width {
        let summaries: Vec<NodeHorizonSummary> = (0..TARGET_HORIZON_COUNT)
            .map(|horizon_index| node_horizon_summary(records, node, horizon_index))
            .collect();

        let cells: Vec<String> = summaries
            .iter()
            .map(|summary| {
                format!(
                    "U_{}={:.6} (n_inclus={}, n_exclus={})",
                    summary.horizon, summary.mean_u, summary.included_count, summary.excluded_count
                )
            })
            .collect();

        println!("nœud {node} : {}", cells.join(" | "));
    }
}

fn print_descriptor_correlations(label: &str, correlations: &DescriptorCorrelations) {
    println!();
    println!("--- {label} ---");
    println!(
        "n_inclus (range(6) défini)={} | n_exclus={}",
        correlations.included_count, correlations.excluded_count
    );
    println!("corr(range(6), δ)      ={:.9}", correlations.delta);
    print_interval(
        "  IC 95 % bootstrap corr(range(6), δ)",
        correlations.bootstrap_delta,
    );
    println!("corr(range(6), δ̄)      ={:.9}", correlations.delta_bar);
    print_interval(
        "  IC 95 % bootstrap corr(range(6), δ̄)",
        correlations.bootstrap_delta_bar,
    );
    println!("corr(range(6), s2)     ={:.9}", correlations.s2);
    print_interval(
        "  IC 95 % bootstrap corr(range(6), s2)",
        correlations.bootstrap_s2,
    );
    println!("corr(range(6), s3)     ={:.9}", correlations.s3);
    print_interval(
        "  IC 95 % bootstrap corr(range(6), s3)",
        correlations.bootstrap_s3,
    );
}

/// Criterion TDI-6.4C (Section 15, descriptive): per block and pooled
/// aggregate.
fn print_tdi64_criterion_c(criterion_c: &Tdi64CriterionC) {
    println!();
    println!(
        "=== TDI-6.4C — corrélats descripteurs de l'hétérogénéité (Section 15, descriptif) ==="
    );

    for (seed_block, correlations) in &criterion_c.blocks {
        print_descriptor_correlations(&format!("bloc {}", seed_block.label()), correlations);
    }

    print_descriptor_correlations("agrégat", &criterion_c.aggregate);
}

/// TDI-6.4 Section 17: the TDI-6.4A/B/C descriptive summaries. None of these
/// is a pass/fail verdict -- a causal-heterogeneity measurement has no
/// natural "success" or "failure" outcome (preregistration Section 2) -- so,
/// like TDI-6.3's own final-verdicts printer, nothing here is a
/// Beneficial/Equivalent/Harmful/Inconclusive classification.
fn print_tdi52_final_verdicts(report: &Tdi64ExperimentReport) {
    println!();
    println!("=== VERDICTS FINAUX (Section 17) ===");

    println!(
        "TDI-6.4A — hétérogénéité nœud-à-nœud : descriptive uniquement, aucun verdict pass/fail \
         (Section 13) ; voir la distribution de range(h) par bloc et agrégat ci-dessus."
    );
    for summary in &report.criterion_a.aggregate {
        println!(
            "TDI-6.4A — U{} — agrégat : médiane range(h)={:.6}, IC95%[{:.6}, {:.6}] sur la \
             moyenne, exclusions pleine récupération={}",
            summary.horizon,
            summary.median,
            summary.bootstrap_mean.lower,
            summary.bootstrap_mean.upper,
            summary.excluded_count
        );
    }

    println!(
        "TDI-6.4B — transfert précoce→tardif par nœud : descriptif uniquement (Section 14) ; \
         voir corr(O_i(1), U_i(6)) et corr(O_i(2), U_i(6)) par nœud ci-dessus."
    );
    println!(
        "TDI-6.4B — largeur 3 : stable(O1)={}, stable(O2)={}",
        report.criterion_b.width_3_stability.stable_o1,
        report.criterion_b.width_3_stability.stable_o2
    );
    println!(
        "TDI-6.4B — largeur 4 : stable(O1)={}, stable(O2)={}",
        report.criterion_b.width_4_stability.stable_o1,
        report.criterion_b.width_4_stability.stable_o2
    );

    println!(
        "TDI-6.4C — corrélats descripteurs de l'hétérogénéité : descriptif uniquement \
         (Section 15) ; voir corr(range(6), δ/δ̄/s2/s3) par bloc et agrégat ci-dessus."
    );
    println!(
        "TDI-6.4C — agrégat : corr(range(6), δ)={:.6}, corr(range(6), δ̄)={:.6}, \
         corr(range(6), s2)={:.6}, corr(range(6), s3)={:.6}",
        report.criterion_c.aggregate.delta,
        report.criterion_c.aggregate.delta_bar,
        report.criterion_c.aggregate.s2,
        report.criterion_c.aggregate.s3,
    );
}

/// Prints the complete TDI-6.4 required raw output (Section 17) for a
/// completed pipeline run. Purely a presentation layer over
/// `Tdi64ExperimentReport`: it has no scale-awareness of its own, so it is
/// exercised at tiny scale by the termination smoke path and by tests. It
/// only ever prints the real 120,000-record run's output when called from
/// `run_full_experiment`'s `--full` path, and only after that path's exact
/// confirmation-token check has passed.
/// Section 19's consistency check itself is a runtime `assert_eq!` inside
/// `analyze_seed` (fired once per accepted record, comparing that record's
/// recomputed historical-node `NodeAnalysis` entry against the fields
/// TDI-5.6's own unchanged logic already computed): a divergence panics
/// immediately, before this function -- or any required output -- is ever
/// reached. Reaching this print statement at all is therefore already proof
/// every accepted record's check passed; this line makes that fact an
/// explicit, positive part of the required raw output (Section 17) rather
/// than something a reader can only infer from the absence of a panic.
fn print_section_19_consistency_confirmation(total_records: usize) {
    println!();
    println!(
        "Section 19 -- cohérence du nœud historique : SUCCÈS pour les {total_records} \
         enregistrements acceptés (chaque assert_eq! dans analyze_seed aurait paniqué \
         avant d'atteindre cette impression en cas de divergence)"
    );
}

fn print_tdi52_required_raw_output(report: &Tdi64ExperimentReport) {
    print_tdi52_provenance();
    print_tdi52_frozen_constants();
    print_tdi52_seed_block_definitions();
    print_tdi52_population_accounting(&report.blocks);

    let all_records: Vec<Record> = report
        .blocks
        .iter()
        .flat_map(BlockPopulations::combined_all_records)
        .collect();

    print_section_19_consistency_confirmation(all_records.len());

    print_tdi64_descriptor_diagnostics(report);
    print_tdi64_criterion_a(&report.criterion_a);
    print_tdi64_criterion_b(&report.criterion_b);

    let (width_3, width_4) = partition_by_width(&all_records);
    println!();
    println!(
        "=== TABLE COMPLÉMENTAIRE — moyenne U_i(h) par nœud et par horizon (Section 17 ; \
         agrégat uniquement, mêmes largeurs que TDI-6.4B) ==="
    );
    print_node_horizon_table("agrégat, largeur 3", &width_3, TRAIN_WIDTH_3);
    print_node_horizon_table("agrégat, largeur 4", &width_4, TRAIN_WIDTH_4);

    print_tdi64_criterion_c(&report.criterion_c);
    print_tdi52_final_verdicts(report);
}

/// TDI-6.4's termination smoke (preregistration Section 16): exercises the
/// complete per-node heterogeneity/transfer/descriptor-correlation pipeline
/// -- `compute_criterion_a`, `compute_criterion_b`, `compute_criterion_c`,
/// `assemble_tdi64_report`'s report assembly, and the full required-raw-
/// output printing -- on a tiny, bounded, fully in-memory synthetic record
/// set (`synthetic_smoke_records_for_width`), covering both width-3 and
/// width-4 node counts. Mirrors TDI-6.3's termination smoke: a single, tiny,
/// bounded REAL candidate generation (`generate_records_with_limits`)
/// confirms the inherited generation machinery -- now including the new
/// per-node analysis -- still works, entirely separate from, and prior to,
/// the criteria-pipeline exercise, which uses only the synthetic records and
/// never calls `analyze_seed`, `generate_block_populations` or
/// `run_tdi64_pipeline`.
fn run_termination_smoke() -> Result<(), String> {
    println!("=== TDI-6.4 TERMINATION SMOKE ===");

    // Inherited frozen invariant: the width-6 successor-set space is the
    // exact 2^64. TDI-6.4 generates no width-6 populations, but the
    // cardinality machinery is inherited unchanged and still checked here.
    let width_6_space = successor_set_space_cardinality(WIDTH_6);

    if width_6_space != Cardinality::Exact(18_446_744_073_709_551_616_u128) {
        return Err(format!("unexpected width-6 cardinality: {width_6_space:?}"));
    }

    println!("width 6 successor-set space   : 18446744073709551616");

    let seed_reservation_count = validate_preregistered_seed_reservations()?;
    println!("reserved seed ranges           : {seed_reservation_count} disjoint");

    println!("bootstrap replicates           : {BOOTSTRAP_REPLICATES}");

    for block in SEED_BLOCKS {
        println!(
            "block {} bootstrap seed        : 0x{:016X}",
            block.id.label(),
            block.bootstrap_seed
        );
    }

    println!("aggregate bootstrap seed       : 0x{AGGREGATE_BOOTSTRAP_SEED:016X}");

    let specs = population_specs();
    println!(
        "population specifications     : {} deterministic entries (4 per block, no OOD) -- \
         consulted here only for seed-reservation arithmetic, never for real record counts",
        specs.len()
    );

    // Inherited-machinery sanity check: a single, tiny, bounded REAL
    // candidate generation confirming `analyze_seed`/
    // `generate_records_with_limits`, the exact contraction descriptors, and
    // the new per-node analysis all still work together, entirely separate
    // from -- and prior to -- the criteria-pipeline exercise below.
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
        "inherited generation sanity    : width 3, {} accepted record in {} attempts",
        inherited_generation.records.len(),
        inherited_generation.attempts
    );

    if let Some(first) = inherited_generation.records.first() {
        println!(
            "inherited generation δ, δ̄      : {:.6}, {:.6}",
            first.contraction[0], first.contraction[1]
        );

        if first.per_node.len() != usize::from(TRAIN_WIDTH_3) {
            return Err(format!(
                "expected {TRAIN_WIDTH_3} per-node entries on a width-3 record, found {}",
                first.per_node.len()
            ));
        }

        println!(
            "inherited generation per-node  : {} nodes analyzed",
            first.per_node.len()
        );

        // Section 19 sanity (the dedicated `tests` module repeats this
        // against a larger, freshly generated set): the historical node's
        // recomputed entry must match the inherited fields exactly.
        let historical = &first.per_node[usize::from(TRAIN_WIDTH_3) - 1];

        if historical.early_overlap != first.early_overlap {
            return Err(
                "historical per-node early_overlap diverges from the inherited field".to_owned(),
            );
        }

        if historical.target_overlaps != first.overlaps {
            return Err(
                "historical per-node target_overlaps diverges from the inherited field".to_owned(),
            );
        }

        for (horizon_index, &value) in first.targets_u.iter().enumerate() {
            if historical.target_u[horizon_index] != Some(value) {
                return Err(format!(
                    "historical per-node target_u at horizon index {horizon_index} diverges \
                     from the inherited targets_u field"
                ));
            }
        }
    }

    // Synthetic, bounded records exercising the heterogeneity/transfer/
    // descriptor-correlation machinery without any real generation.
    let width_3_records = synthetic_smoke_records_for_width(TRAIN_WIDTH_3, 8, 0.0);
    let width_4_records = synthetic_smoke_records_for_width(TRAIN_WIDTH_4, 8, 100.0);

    println!(
        "synthetic smoke records        : {} width-3, {} width-4",
        width_3_records.len(),
        width_4_records.len()
    );

    // The critical wiring smoke: the real report-assembly entrypoint
    // (`assemble_tdi64_report`), over synthetic (never generated) block
    // populations built entirely in memory.
    let blocks = vec![
        synthetic_block_populations(SeedBlockId::V, 1, &width_3_records, &width_4_records),
        synthetic_block_populations(SeedBlockId::W, 101, &width_3_records, &width_4_records),
        synthetic_block_populations(SeedBlockId::X, 201, &width_3_records, &width_4_records),
    ];

    let report = assemble_tdi64_report(blocks)?;

    println!(
        "identity smoke criterion A     : blocks={}, aggregate horizons={}",
        report.criterion_a.blocks.len(),
        report.criterion_a.aggregate.len()
    );
    println!(
        "identity smoke criterion B     : width-3 nodes={}, width-4 nodes={}",
        report.criterion_b.width_3.len(),
        report.criterion_b.width_4.len()
    );
    println!(
        "identity smoke criterion C     : blocks={}",
        report.criterion_c.blocks.len()
    );

    print_tdi52_required_raw_output(&report);

    println!("bounded smoke result           : PASS");

    Ok(())
}

/// Name of the environment variable that must carry the exact TDI-6.4
/// full-run confirmation value. See TDI-6.4 preregistration Section 16.
const TDI64_FULL_RUN_CONFIRMATION_VAR: &str = "TDI64_CONFIRM_FULL_RUN";

/// The one accepted value for `TDI64_FULL_RUN_CONFIRMATION_VAR`. Any other
/// value, or the variable being unset, must refuse `--full`.
const TDI64_FULL_RUN_CONFIRMATION_VALUE: &str = "I_ACCEPT_THE_TDI64_FREEZE_RULE";

/// Pure decision function: takes the confirmation value as a plain
/// `Option<&str>` rather than reading the environment itself, so every
/// branch -- missing, wrong, and the one exact accepted value -- can be unit
/// tested directly without ever touching a real environment variable or
/// risking the accepted branch reaching `run_full_experiment` (and, through
/// it, the real pipeline).
fn tdi64_full_run_confirmed(value: Option<&str>) -> bool {
    value == Some(TDI64_FULL_RUN_CONFIRMATION_VALUE)
}

fn tdi64_usage_error() -> String {
    format!(
        "usage: tdi-independent-overlap-ablation-v64 --termination-smoke|--preflight|--full\n\
         a bare (no-argument) invocation does not start the experiment; the \
         real run additionally requires the exact environment variable \
         {TDI64_FULL_RUN_CONFIRMATION_VAR}={TDI64_FULL_RUN_CONFIRMATION_VALUE}"
    )
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Tdi64Mode {
    TerminationSmoke,
    Preflight,
    Full,
}

/// Pure command-line dispatch decision, independent of `main`'s I/O, so that
/// "a bare invocation can never select `--full`" is directly unit testable
/// against plain string slices rather than real process argv.
fn tdi64_parse_mode(arguments: &[String]) -> Result<Tdi64Mode, String> {
    match arguments {
        [flag] if flag == "--termination-smoke" => Ok(Tdi64Mode::TerminationSmoke),
        [flag] if flag == "--preflight" => Ok(Tdi64Mode::Preflight),
        [flag] if flag == "--full" => Ok(Tdi64Mode::Full),
        _ => Err(tdi64_usage_error()),
    }
}

fn main() -> Result<(), String> {
    let arguments = std::env::args().skip(1).collect::<Vec<_>>();

    match tdi64_parse_mode(&arguments)? {
        Tdi64Mode::TerminationSmoke => run_termination_smoke(),
        Tdi64Mode::Preflight => run_preflight(),
        Tdi64Mode::Full => run_full_experiment(),
    }
}

/// The TDI-6.4 full-run entrypoint. Checks the exact confirmation
/// environment variable *before* any generation or computation; only when it
/// matches does this call the real full pipeline exactly once, over the real
/// preregistered `population_specs()`, and print the complete required raw
/// output. See TDI-6.4 preregistration Section 16.
fn run_full_experiment() -> Result<(), String> {
    let confirmation = std::env::var(TDI64_FULL_RUN_CONFIRMATION_VAR).ok();

    if !tdi64_full_run_confirmed(confirmation.as_deref()) {
        return Err(format!(
            "TDI-6.4 full execution requires the exact confirmation environment \
             variable {TDI64_FULL_RUN_CONFIRMATION_VAR}={TDI64_FULL_RUN_CONFIRMATION_VALUE}; \
             refusing before any generation or computation"
        ));
    }

    let report = run_tdi64_pipeline(&population_specs())?;

    print_tdi52_required_raw_output(&report);

    Ok(())
}

/// TDI-6.4 preflight: verifies the complete frozen configuration (seed
/// reservations, population counts, bootstrap constants, pipeline wiring)
/// and prints identities and the exact real-run command, without ever
/// generating a scientific population. See TDI-6.4 preregistration Section
/// 16.
fn run_preflight() -> Result<(), String> {
    println!();
    println!("=== TDI-6.4 PREFLIGHT (aucune génération scientifique) ===");

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
    println!("réplicats de bootstrap par bloc/agrégat         : {BOOTSTRAP_REPLICATES}");
    println!(
        "graines de bootstrap par bloc                   : {}=0x{:016X} {}=0x{:016X} {}=0x{:016X}",
        SeedBlockId::V.label(),
        SeedBlockId::V.bootstrap_seed(),
        SeedBlockId::W.label(),
        SeedBlockId::W.bootstrap_seed(),
        SeedBlockId::X.label(),
        SeedBlockId::X.bootstrap_seed()
    );
    println!("graine de bootstrap agrégat                     : 0x{AGGREGATE_BOOTSTRAP_SEED:016X}");
    println!(
        "régime FP                                       : IEEE-754 binary64, mono-thread, \
         ordre d'opérations fixe (pas de FMA/parallèle) ; reproduction octet-exacte"
    );
    println!(
        "pipeline complet câblé à --full                 : oui (run_tdi64_pipeline, \
         subordonné à {TDI64_FULL_RUN_CONFIRMATION_VAR})"
    );

    print_tdi52_provenance();

    println!();
    println!("Commande requise pour l'exécution réelle (jamais lancée automatiquement) :");
    println!("  {TDI64_FULL_RUN_CONFIRMATION_VAR}={TDI64_FULL_RUN_CONFIRMATION_VALUE} \\");
    println!("    bash scripts/reproduce-tdi6.4.sh");

    println!();
    println!("=== PREFLIGHT TERMINÉ : aucun résultat produit ===");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        AGGREGATE_BOOTSTRAP_SEED, BASELINE_FEATURE_COUNT, BOOTSTRAP_REPLICATES,
        CONTRACTION_FEATURE_COUNT, Cardinality, ConfidenceInterval, NodeAnalysis, NodeCorrelation,
        PRIMARY_HORIZON, Record, SEED_BLOCKS, SPECTRAL_FEATURE_COUNT, SeedBlockId,
        TARGET_HORIZON_COUNT, TARGET_HORIZONS, TDI64_FULL_RUN_CONFIRMATION_VALUE,
        TDI64_FULL_RUN_CONFIRMATION_VAR, TOTAL_SEED_RESERVATIONS,
    };
    use tdi_core::{Action, State, TableSystem};

    fn read_repo_file(relative_path: &str) -> String {
        std::fs::read_to_string(super::tdi52_repository_root().join(relative_path))
            .unwrap_or_else(|error| panic!("failed to read {relative_path}: {error}"))
    }

    fn evaluator_source() -> String {
        read_repo_file("tdi-bench/src/bin/tdi-independent-overlap-ablation-v64.rs")
    }

    /// A hand-built `Record` for one width, with every `per_node` cell
    /// defined except, optionally, node 0's horizon-index-0 cell (used to
    /// exercise Section 5.1's exclusion-counting behaviour without any real
    /// candidate generation).
    fn make_node_analysis_record(width: u8, degenerate_at_node0_h0: bool) -> Record {
        let mut per_node = Vec::with_capacity(usize::from(width));

        for node in 0..width {
            let base = 1.0 + f64::from(node);
            let mut target_overlaps = [0.30_f64; TARGET_HORIZON_COUNT];
            let mut target_u: [Option<f64>; TARGET_HORIZON_COUNT] =
                [Some(base); TARGET_HORIZON_COUNT];

            if degenerate_at_node0_h0 && node == 0 {
                target_overlaps[0] = 1.0;
                target_u[0] = None;
            }

            per_node.push(NodeAnalysis {
                node,
                early_overlap: [0.10 + 0.01 * f64::from(node), 0.20 + 0.01 * f64::from(node)],
                target_overlaps,
                target_u,
            });
        }

        let historical = per_node[usize::from(width) - 1].clone();

        Record {
            baseline: [0.0; BASELINE_FEATURE_COUNT],
            early_overlap: historical.early_overlap,
            contraction: [0.30, 0.20],
            spectral: [1.50, 1.20],
            overlaps: historical.target_overlaps,
            targets_u: std::array::from_fn(|horizon_index| {
                historical.target_u[horizon_index].unwrap_or(0.0)
            }),
            width,
            per_node,
        }
    }

    // --- Exact contraction descriptors / spectral moments (inherited from
    // TDI-5.6 unchanged; re-verified here since this is a fresh file) ---

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

    #[test]
    fn spectral_moments_are_exact_traces_of_kernel_powers() {
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

    // --- New primitive: analyze_node (Section 5, 5.1) ---

    #[test]
    fn analyze_node_records_none_on_full_recovery_without_rejecting_anything() {
        // The exact fixture tdi-core's own `detects_complete_reconvergence`
        // test uses: a deterministic (single-successor) width-2 system where
        // flipping node 0 from state `zero` fully reconverges by horizon 2,
        // and (being deterministic) stays reconverged at every later depth.
        let zero = State::new(0b00, 2).expect("valid state");
        let one = State::new(0b01, 2).expect("valid state");
        let two = State::new(0b10, 2).expect("valid state");
        let three = State::new(0b11, 2).expect("valid state");

        let mut system = TableSystem::new(2).expect("valid system");
        system.insert(zero, Action::Noop, vec![two]).unwrap();
        system.insert(one, Action::Noop, vec![three]).unwrap();
        system.insert(two, Action::Noop, vec![zero]).unwrap();
        system.insert(three, Action::Noop, vec![zero]).unwrap();

        let context = super::AttemptContext::new(2, 0, 0);
        let analysis =
            super::analyze_node(context, &system, zero, 0).expect("analyze_node succeeds");

        assert_eq!(analysis.node, 0);
        // Every dense-grid horizon (>= 3) is well past the horizon-2
        // reconvergence point, so every cell is a legitimate full recovery:
        // `None`, never an error, never a rejection (analyze_node has no
        // rejection concept at all -- it always returns `Ok`).
        for target_u in analysis.target_u {
            assert_eq!(target_u, None);
        }
        for overlap in analysis.target_overlaps {
            assert_eq!(overlap, 1.0);
        }
    }

    #[test]
    fn per_node_analysis_never_introduces_a_new_rejection_path() {
        // Structural guarantee (Section 5.1): once `analyze_seed` has decided
        // to accept a candidate (based on the historical node alone), the
        // per-node loop that follows must never itself construct a
        // `RejectionReason` -- only propagate a fatal `EvaluationError` via
        // `?`, which would indicate a defect, not a graceful exclusion.
        let source = evaluator_source();
        let start = source
            .find("let mut per_node = Vec::with_capacity(usize::from(context.width));")
            .expect("the per-node loop must exist in analyze_seed");
        let end = source[start..]
            .find("\nfn preregistered_generation_limits")
            .map(|offset| start + offset)
            .expect("preregistered_generation_limits must follow analyze_seed");
        let body = &source[start..end];

        assert!(
            !body.contains("RejectionReason::"),
            "the per-node analysis (and the Section 19 consistency assertion that follows it) \
             must never construct a new RejectionReason"
        );
    }

    // --- Section 19: historical-node consistency check ---

    #[test]
    fn historical_node_matches_inherited_fields_on_real_generated_records() {
        let seed_3 = SEED_BLOCKS[0].training_width_3_seed;
        let limits_3 = super::preregistered_generation_limits(3, seed_3, 6).unwrap();
        let report_3 = super::generate_records_with_limits(3, seed_3, 6, limits_3)
            .expect("bounded width-3 generation");

        let seed_4 = SEED_BLOCKS[0].training_width_4_seed;
        let limits_4 = super::preregistered_generation_limits(4, seed_4, 6).unwrap();
        let report_4 = super::generate_records_with_limits(4, seed_4, 6, limits_4)
            .expect("bounded width-4 generation");

        for record in report_3.records.iter().chain(report_4.records.iter()) {
            assert_eq!(record.per_node.len(), usize::from(record.width));

            let historical_index = usize::from(record.width) - 1;
            let historical = &record.per_node[historical_index];

            assert_eq!(historical.node, record.width - 1);
            assert_eq!(
                historical.early_overlap, record.early_overlap,
                "historical per-node early_overlap must be bit-identical to the inherited field"
            );
            assert_eq!(
                historical.target_overlaps, record.overlaps,
                "historical per-node target_overlaps must be bit-identical to the inherited field"
            );

            for (horizon_index, &value) in record.targets_u.iter().enumerate() {
                assert_eq!(
                    historical.target_u[horizon_index],
                    Some(value),
                    "historical per-node target_u at horizon index {horizon_index} must be \
                     bit-identical to the inherited targets_u field"
                );
            }
        }
    }

    // --- Section 5.1: full-recovery exclusion counting on a constructed
    // degenerate fixture ---

    #[test]
    fn full_recovery_at_a_non_historical_node_excludes_the_system_from_that_horizon_only() {
        let normal = make_node_analysis_record(3, false);
        let degenerate = make_node_analysis_record(3, true);
        let records = vec![normal, degenerate];

        let pool_h0 = super::horizon_pool(&records, 0);
        assert_eq!(
            pool_h0.range_values.len(),
            1,
            "the degenerate record must be excluded from horizon index 0's range(h) values"
        );
        assert_eq!(pool_h0.excluded_count, 1);

        for horizon_index in 1..TARGET_HORIZON_COUNT {
            let pool = super::horizon_pool(&records, horizon_index);
            assert_eq!(
                pool.range_values.len(),
                2,
                "horizon index {horizon_index} must not exclude either record (Section 5.1: \
                 exclusion is horizon-local)"
            );
            assert_eq!(pool.excluded_count, 0);
        }
    }

    #[test]
    fn summarize_horizon_range_reports_the_exclusion_count_and_never_silently_drops_it() {
        let normal = make_node_analysis_record(3, false);
        let degenerate = make_node_analysis_record(3, true);
        let records = vec![normal, degenerate];

        let summary = super::summarize_horizon_range(TARGET_HORIZONS[0], &records, 0, 0xAAAA)
            .expect("summary at horizon index 0");

        assert_eq!(summary.included_count, 1);
        assert_eq!(summary.excluded_count, 1);

        let summary_other = super::summarize_horizon_range(TARGET_HORIZONS[1], &records, 1, 0xAAAA)
            .expect("summary at horizon index 1");

        assert_eq!(summary_other.included_count, 2);
        assert_eq!(summary_other.excluded_count, 0);
    }

    #[test]
    fn summarize_horizon_range_errors_rather_than_silently_reporting_when_every_system_is_excluded()
    {
        let mut degenerate = make_node_analysis_record(3, false);
        for node in &mut degenerate.per_node {
            node.target_u[0] = None;
        }

        let result = super::summarize_horizon_range(TARGET_HORIZONS[0], &[degenerate], 0, 0xAAAA);
        assert!(result.is_err());
    }

    #[test]
    fn descriptor_correlations_reports_included_and_excluded_counts() {
        let mut records: Vec<Record> = (0..6)
            .map(|i| {
                let mut record = make_node_analysis_record(3, false);
                record.contraction = [0.10 * f64::from(i), 0.20 * f64::from(i)];
                record.spectral = [1.0 + 0.1 * f64::from(i), 2.0 + 0.1 * f64::from(i)];
                for node in &mut record.per_node {
                    node.target_u[super::primary_horizon_index()] =
                        Some(1.0 + 0.3 * f64::from(i) + f64::from(node.node));
                }
                record
            })
            .collect();

        let mut degenerate = make_node_analysis_record(3, false);
        degenerate.per_node[0].target_u[super::primary_horizon_index()] = None;
        records.push(degenerate);

        let result = super::descriptor_correlations(&records, 0xBEEF).expect("correlations");
        assert_eq!(result.included_count, 6);
        assert_eq!(result.excluded_count, 1);
        for value in [result.delta, result.delta_bar, result.s2, result.s3] {
            assert!(value.is_finite());
            assert!((-1.0..=1.0).contains(&value));
        }
    }

    // --- New descriptive-statistics primitives ---

    #[test]
    fn pearson_correlation_of_perfectly_correlated_data_is_one() {
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let y = vec![2.0, 4.0, 6.0, 8.0, 10.0];
        assert!((super::pearson_correlation(&x, &y) - 1.0).abs() < 1e-12);
    }

    #[test]
    fn pearson_correlation_of_constant_data_is_zero() {
        let x = vec![1.0, 1.0, 1.0, 1.0];
        let y = vec![2.0, 4.0, 6.0, 8.0];
        assert_eq!(super::pearson_correlation(&x, &y), 0.0);
    }

    #[test]
    fn bootstrap_mean_ci_is_deterministic() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let first = super::bootstrap_mean_ci(&values, 0xABCD, 200);
        let second = super::bootstrap_mean_ci(&values, 0xABCD, 200);
        assert_eq!(first, second);
        assert!(first.lower <= first.median);
        assert!(first.median <= first.upper);
    }

    #[test]
    fn bootstrap_two_correlations_is_deterministic_and_directionally_sane() {
        let x1 = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
        let x2 = vec![6.0, 5.0, 4.0, 3.0, 2.0, 1.0];
        let y = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0];

        let (first_a, first_b) = super::bootstrap_two_correlations(&x1, &x2, &y, 0x1234, 100);
        let (second_a, second_b) = super::bootstrap_two_correlations(&x1, &x2, &y, 0x1234, 100);

        assert_eq!(first_a, second_a);
        assert_eq!(first_b, second_b);
        assert!(first_a.median > 0.9);
        assert!(first_b.median < -0.9);
    }

    #[test]
    fn confidence_intervals_overlap_detects_overlap_and_non_overlap() {
        let a = ConfidenceInterval {
            lower: 0.0,
            median: 0.5,
            upper: 1.0,
        };
        let b = ConfidenceInterval {
            lower: 0.9,
            median: 1.0,
            upper: 1.1,
        };
        let c = ConfidenceInterval {
            lower: 2.0,
            median: 2.5,
            upper: 3.0,
        };

        assert!(super::confidence_intervals_overlap(a, b));
        assert!(super::confidence_intervals_overlap(b, a));
        assert!(!super::confidence_intervals_overlap(a, c));
    }

    #[test]
    fn partition_by_width_splits_records_correctly() {
        let width_3 = make_node_analysis_record(3, false);
        let width_4 = make_node_analysis_record(4, false);
        let records = vec![width_3.clone(), width_4.clone(), width_3];

        let (three, four) = super::partition_by_width(&records);
        assert_eq!(three.len(), 2);
        assert_eq!(four.len(), 1);
        assert!(three.iter().all(|record| record.width == 3));
        assert!(four.iter().all(|record| record.width == 4));
    }

    #[test]
    fn node_correlation_stability_flags_only_the_diverging_nodes() {
        let ci = |lower: f64, upper: f64| ConfidenceInterval {
            lower,
            median: (lower + upper) / 2.0,
            upper,
        };
        let make = |node: u8, lower: f64, upper: f64| NodeCorrelation {
            node,
            included_count: 10,
            excluded_count: 0,
            corr_o1_u6: (lower + upper) / 2.0,
            corr_o2_u6: (lower + upper) / 2.0,
            bootstrap_o1_u6: ci(lower, upper),
            bootstrap_o2_u6: ci(lower, upper),
        };

        let nodes = vec![
            make(0, 0.0, 0.2),   // does not overlap the historical node's interval
            make(1, 0.55, 0.75), // overlaps the historical node's interval
            make(2, 0.60, 0.80), // historical node i* = width - 1 = 2
        ];

        let stability = super::node_correlation_stability(&nodes);
        assert!(!stability.stable_o1);
        assert!(!stability.stable_o2);
        assert_eq!(stability.shifted_nodes_o1, vec![0]);
        assert_eq!(stability.shifted_nodes_o2, vec![0]);
    }

    #[test]
    fn node_correlation_stability_reports_stable_when_every_interval_overlaps() {
        let ci = |lower: f64, upper: f64| ConfidenceInterval {
            lower,
            median: (lower + upper) / 2.0,
            upper,
        };
        let make = |node: u8| NodeCorrelation {
            node,
            included_count: 10,
            excluded_count: 0,
            corr_o1_u6: 0.7,
            corr_o2_u6: 0.7,
            bootstrap_o1_u6: ci(0.6, 0.8),
            bootstrap_o2_u6: ci(0.6, 0.8),
        };

        let nodes = vec![make(0), make(1), make(2)];
        let stability = super::node_correlation_stability(&nodes);
        assert!(stability.stable_o1);
        assert!(stability.stable_o2);
        assert!(stability.shifted_nodes_o1.is_empty());
        assert!(stability.shifted_nodes_o2.is_empty());
    }

    // --- Termination-smoke synthetic fixture ---

    #[test]
    fn synthetic_smoke_records_are_non_degenerate() {
        let width_3 = super::synthetic_smoke_records_for_width(3, 8, 0.0);
        let width_4 = super::synthetic_smoke_records_for_width(4, 8, 100.0);
        assert_eq!(width_3.len(), 8);
        assert_eq!(width_4.len(), 8);

        for records in [&width_3, &width_4] {
            let mut any_defined = false;
            let mut any_none = false;

            for record in records.iter() {
                assert_eq!(record.per_node.len(), usize::from(record.width));

                for node in &record.per_node {
                    for value in node.target_u {
                        match value {
                            Some(_) => any_defined = true,
                            None => any_none = true,
                        }
                    }
                }
            }

            assert!(any_defined, "fixture must not have every cell undefined");
            assert!(
                any_none,
                "fixture must exercise the Section 5.1 full-recovery/None path"
            );
        }
    }

    // --- Frozen ancestors must never change under TDI-6.4 ---

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
        assert_eq!(super::FROZEN_ANCESTOR_CHAIN.len(), 12);
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
    fn seed_blocks_are_vwx_and_start_at_nine_billion_disjoint_from_every_prior_block() {
        let ids: Vec<_> = SEED_BLOCKS.iter().map(|b| b.id).collect();
        assert_eq!(ids, vec![SeedBlockId::V, SeedBlockId::W, SeedBlockId::X]);

        for block in SEED_BLOCKS {
            for seed in [
                block.training_width_3_seed,
                block.holdout_width_3_seed,
                block.training_width_4_seed,
                block.holdout_width_4_seed,
            ] {
                assert!(seed >= 9_000_000_000);
                // TDI-6.3 occupies roughly 8.0e9 .. 8.24e9; TDI-6.4 must
                // start strictly above it, and above every other prior
                // block (TDI-5.7 <= 2.53e9; TDI-6.1 3.0-3.23e9; TDI-6.2
                // 4.0-4.23e9; TDI-6.5 5.0-6.13e9; TDI-5.8 7.0-7.81e9).
                assert!(seed > 8_240_000_000);
            }
        }

        let boots: Vec<_> = SEED_BLOCKS.iter().map(|b| b.bootstrap_seed).collect();
        assert_eq!(
            boots,
            vec![
                0x5444_4936_3400_0001_u64,
                0x5444_4936_3400_0002,
                0x5444_4936_3400_0003
            ]
        );
        assert_eq!(AGGREGATE_BOOTSTRAP_SEED, 0x5444_4936_3400_4700);
        assert!(!boots.contains(&AGGREGATE_BOOTSTRAP_SEED));
    }

    #[test]
    fn feature_counts_match_the_preregistration() {
        assert_eq!(BASELINE_FEATURE_COUNT, 13);
        assert_eq!(super::EARLY_OVERLAP_FEATURE_COUNT, 2);
        assert_eq!(CONTRACTION_FEATURE_COUNT, 2);
        assert_eq!(SPECTRAL_FEATURE_COUNT, 2);
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

    #[test]
    fn generate_records_is_deterministic_and_carries_per_node_analysis() {
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
            assert_eq!(a.width, b.width);
            assert_eq!(a.per_node.len(), b.per_node.len());
            for (node_a, node_b) in a.per_node.iter().zip(b.per_node.iter()) {
                assert_eq!(node_a, node_b);
            }
        }

        for record in &first.records {
            assert_eq!(record.width, 3);
            assert_eq!(record.per_node.len(), 3);

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
    fn run_tdi64_pipeline_wires_all_criteria_on_tiny_specs() {
        // A tiny, bounded count -- matching this program's established
        // end-to-end wiring-test precedent (TDI-6.3 uses 3; TDI-5.8/TDI-6.5
        // use 1). Every assertion below is structural (lengths, counts), not
        // sensitive to sample size, so a larger count buys nothing here; the
        // exclusion-counting *logic* itself is meaningfully exercised
        // separately, on hand-built degenerate fixtures, by
        // `summarize_horizon_range_reports_the_exclusion_count_and_never_
        // silently_drops_it` and `descriptor_correlations_reports_included_
        // and_excluded_counts` elsewhere in this module.
        let tiny_specs = super::population_specs().map(|spec| super::PopulationSpec {
            target_count: 3,
            ..spec
        });

        let report = super::run_tdi64_pipeline(&tiny_specs).expect("tiny end-to-end pipeline run");

        assert_eq!(report.blocks.len(), super::SEED_BLOCK_COUNT);
        assert_eq!(report.descriptor_diagnostics.len(), super::SEED_BLOCK_COUNT);
        assert_eq!(report.criterion_a.blocks.len(), super::SEED_BLOCK_COUNT);
        assert_eq!(report.criterion_a.aggregate.len(), TARGET_HORIZON_COUNT);
        assert_eq!(report.criterion_b.width_3.len(), 3);
        assert_eq!(report.criterion_b.width_4.len(), 4);
        assert_eq!(report.criterion_c.blocks.len(), super::SEED_BLOCK_COUNT);

        // Section 5.1's exclusion accounting must be a true partition of
        // every pooled record at every horizon: included_count +
        // excluded_count must equal the total accepted record count exactly,
        // never merely "no smaller than" (which would hold vacuously for any
        // non-negative counts and could never catch a real defect).
        let total_records: usize = report
            .blocks
            .iter()
            .map(|block| block.combined_all_records().len())
            .sum();

        for summary in &report.criterion_a.aggregate {
            assert_eq!(
                summary.included_count + summary.excluded_count,
                total_records,
                "included_count + excluded_count must exactly partition every pooled record"
            );
        }
    }

    // --- Full-run confirmation guard (Section 16) ---

    #[test]
    fn full_run_confirmation_accepts_only_the_exact_value() {
        assert!(super::tdi64_full_run_confirmed(Some(
            TDI64_FULL_RUN_CONFIRMATION_VALUE
        )));
        assert!(!super::tdi64_full_run_confirmed(None));
        assert!(!super::tdi64_full_run_confirmed(Some("")));
        assert!(!super::tdi64_full_run_confirmed(Some(
            "i_accept_the_tdi64_freeze_rule"
        )));
        // Cross-experiment tokens (the direct ancestor TDI-5.6, and the most
        // recently built sibling TDI-6.3) must never unlock TDI-6.4.
        assert!(!super::tdi64_full_run_confirmed(Some(
            "I_ACCEPT_THE_TDI56_FREEZE_RULE"
        )));
        assert!(!super::tdi64_full_run_confirmed(Some(
            "I_ACCEPT_THE_TDI63_FREEZE_RULE"
        )));
    }

    #[test]
    fn parse_mode_rejects_a_bare_no_argument_invocation() {
        assert!(super::tdi64_parse_mode(&[]).is_err());
        assert!(super::tdi64_parse_mode(&["--full".to_owned(), "extra".to_owned()]).is_err());
    }

    #[test]
    fn parse_mode_selects_full_only_for_the_exact_single_flag() {
        assert_eq!(
            super::tdi64_parse_mode(&["--full".to_owned()]).unwrap(),
            super::Tdi64Mode::Full
        );
        assert_eq!(
            super::tdi64_parse_mode(&["--preflight".to_owned()]).unwrap(),
            super::Tdi64Mode::Preflight
        );
        assert_eq!(
            super::tdi64_parse_mode(&["--termination-smoke".to_owned()]).unwrap(),
            super::Tdi64Mode::TerminationSmoke
        );
        assert!(super::tdi64_parse_mode(&["--Full".to_owned()]).is_err());
    }

    #[test]
    fn usage_error_mentions_every_flag_and_the_confirmation_variable() {
        let usage = super::tdi64_usage_error();
        assert!(usage.contains("--termination-smoke"));
        assert!(usage.contains("--preflight"));
        assert!(usage.contains("--full"));
        assert!(usage.contains(TDI64_FULL_RUN_CONFIRMATION_VAR));
        assert!(usage.contains(TDI64_FULL_RUN_CONFIRMATION_VALUE));
    }

    #[test]
    fn full_run_refuses_before_any_work_without_the_confirmation_token() {
        // Never reach the accepted path in a test: assert the guard var is
        // absent first, then confirm the unconfirmed call returns an error
        // before any generation or computation.
        if std::env::var(TDI64_FULL_RUN_CONFIRMATION_VAR).is_ok() {
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
            body.contains("run_tdi64_pipeline(&population_specs())"),
            "accepted path must call the real pipeline over the real specs"
        );
        assert!(body.contains("tdi64_full_run_confirmed"));
        assert!(body.contains("print_tdi52_required_raw_output"));
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
            .find("\nfn tdi64_full_run_confirmed")
            .map(|offset| start + offset)
            .expect("tdi64_full_run_confirmed must follow run_termination_smoke");
        let body = &source[start..end];

        // A single tiny (1 accepted record), bounded, real candidate
        // generation via `generate_records_with_limits` IS present, as an
        // inherited-machinery sanity check separate from the criteria
        // exercise -- exactly mirroring TDI-5.6/TDI-6.3's own termination
        // smoke -- so that call is deliberately not asserted against here.
        assert!(
            !body.contains("run_tdi64_pipeline("),
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
            body.contains("synthetic_smoke_records_for_width("),
            "the smoke path must exercise the criteria pipeline via the synthetic record set"
        );
        assert!(
            body.contains("assemble_tdi64_report("),
            "the smoke path must exercise the real report-assembly entrypoint"
        );
    }

    #[test]
    fn preflight_runs_without_generating_any_population_and_mentions_the_real_run_command() {
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
        assert!(body.contains("reproduce-tdi6.4.sh"));
    }

    // --- Required raw output substrings (grepped by the reproduction script) ---

    #[test]
    fn required_output_phrases_are_present_in_the_evaluator_source() {
        let source = evaluator_source();
        assert!(source.contains("VERDICTS FINAUX"));
        assert!(source.contains("TDI-6.4A"));
        assert!(source.contains("TDI-6.4B"));
        assert!(source.contains("TDI-6.4C"));
    }
}
