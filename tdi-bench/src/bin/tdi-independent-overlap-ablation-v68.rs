//! TDI-6.8 transportable ordering (does the overlap improve *transferred
//! ordering*, and does the ordering transfer at all?).
//!
//! This file derives from the frozen TDI-6.7 evaluator
//! (`tdi-independent-overlap-ablation-v67.rs`) by changing exactly one factor —
//! **what is measured**: a rank statistic instead of a level statistic.
//! TDI-5.1 … TDI-5.9 and TDI-6.1 … TDI-6.7 remain frozen and untouched.
//!
//! Four experiments in a row found cross-generator transfer of the deficit
//! *level* to be broken, and none of the four repairs worked. Each time, the
//! reading offered was that the *ordering* might still survive even where the
//! level does not. That reading was asserted four times and tested zero times.
//! TDI-6.8 tests it.
//!
//! **One arm, four layouts** (Section 3). No correction of any kind is applied:
//! the model fitted on the source's training populations is applied to the
//! target's holdouts carrying the source's feature statistics and the source's
//! target scaler, coefficients never refitted. TDI-6.7's B1 (observable offset)
//! and B2 (oracle target scaler) are absent by preregistration — B2 in
//! particular fitted the target scaler, and Section 12 states flatly that **no
//! target label is read anywhere in TDI-6.8, in any arm, for any criterion**.
//! The experiment has no oracle arm, because on a bounded rank scale there is no
//! scale to supply. The layouts are the ladder CK ⊂ SK ⊂ GK ⊂ GKT
//! (15 / 17 / 19 / 21 features); the primary comparison is GKT against GK.
//!
//! The statistic is Spearman's ρ **per seed block**, never pooled: pooling three
//! blocks whose deficit levels differ would rank across domains and manufacture
//! correlation from the level gap alone. `ρ̄` is the mean of the three per-block
//! values and is undefined if any block is (Section 6).
//!
//! Criteria: 6.8A (GKT vs GK on the confirmatory pair, margin `m = 0.02`
//! absolute on the bounded [−1, 1] scale — two standard errors of a single ρ at
//! 10,000 records); 6.8B (`rank_transfers` — does the ordering transfer at all,
//! which can disagree with 6.8A: an increment can be *Beneficial* while both
//! correlations sit at zero); 6.8C (`retention = ρ̄_transfer / ρ̄_within`,
//! descriptive); 6.8D (all 12 ordered pairs and the direction consistency
//! across them, with the preregistered reading rule that a *Beneficial*
//! increment accompanied by `ρ̄(GKT) ≤ 0` is **better-ordered noise, not
//! transfer**). Section 14 reports transferred ordering against the label-free
//! domain distance `|ū₂ᵀ − ū₂ˢ|` — the statistic TDI-6.7 applied as `Δ` and
//! TDI-6.8 only reports.
//!
//! No outcome is a success or a failure. *Harmful* or *Equivalent* would
//! establish that the four prior discussions were reading a pooling artifact or
//! a within-domain effect, which is a result of the same weight as confirmation.
//!
//! Everything else is inherited verbatim from TDI-6.7: the four generator
//! families, the population contract, the CK/SK/GK/GKT layouts, the linear ridge
//! (λ = 1), the horizons, the deterministic bootstrap, the four-way classifier,
//! and the non-exact determinism discipline in which `g` and `τ_ε` are the only
//! non-exact quantities. The rank statistic adds none — ranks of finite doubles
//! are exact.
//!
//! Populations and bootstrap streams are **fresh** (Section 7): the 8.6e9 seed
//! origin clears TDI-6.7's last reservation. That freshness is what lets Section
//! 1.3 admit a rank criterion whose hypothesis came from already-observed data.
//!
//! The full run is gated behind an explicit, exact human confirmation
//! environment variable (see `run_full_experiment` and
//! `tdi68_full_run_confirmed`). No commit, test or CI run supplies that token.

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

// The two focal horizons at which TDI-6.8A/6.8B classify: U3 (near, where
// TDI-5.4B found a short-horizon benefit) and the primary U6.
const FOCAL_HORIZONS: [usize; 2] = [3, 6];
const FOCAL_HORIZON_COUNT: usize = FOCAL_HORIZONS.len();

const TRAIN_WIDTH_3: u8 = 3;
const TRAIN_WIDTH_4: u8 = 4;
// Widths 5 and 6 remain supported by the inherited frozen generator and its
// exact cardinality/budget machinery, but TDI-6.7 generates no populations
// at those widths (Section 8): there are no OOD populations.
const WIDTH_5: u8 = 5;
const WIDTH_6: u8 = 6;

const TRAIN_WIDTH_3_SYSTEMS: usize = 15_000;
const TRAIN_WIDTH_4_SYSTEMS: usize = 15_000;
const HOLDOUT_WIDTH_3_SYSTEMS: usize = 5_000;
const HOLDOUT_WIDTH_4_SYSTEMS: usize = 5_000;

// TDI-6.7 runs the inherited 3-block per-generator machinery once per
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

/// Frozen non-exact regime constants (Section 13), inherited unchanged from
/// TDI-6.1 Section 12.
const EIGEN_CONVERGENCE_TOLERANCE: f64 = 1e-12;
const SPECTRAL_CROSS_METHOD_TOLERANCE: f64 = 1e-9;
const MIXING_EPSILON: f64 = 0.25;
const MIXING_TIME_CAP: usize = 4096;

// Linear layouts, inherited from TDI-5.2/5.3/5.4/5.5. In TDI-6.7 they are
// exploratory only (Section 6) and determine no confirmatory criterion.
const B0_FEATURE_COUNT: usize = BASELINE_FEATURE_COUNT;
const B1_FEATURE_COUNT: usize = BASELINE_FEATURE_COUNT + 1;
const B2_FEATURE_COUNT: usize = BASELINE_FEATURE_COUNT + 1;
const B12_FEATURE_COUNT: usize = BASELINE_FEATURE_COUNT + EARLY_OVERLAP_FEATURE_COUNT;
const BD_FEATURE_COUNT: usize = BASELINE_FEATURE_COUNT + 1;

// The two *literal* non-exact spectral descriptors of the one-step Noop kernel,
// inherited unchanged from TDI-6.1 Section 6: the spectral gap g = 1 - |λ2| and
// the normalized ε-mixing time τ_ε / T_max. These are the ONLY non-exact
// quantities in the experiment (computed in the declared f64 regime, Section
// 13); everything else remains bit-exact.
const LITERAL_SPECTRAL_FEATURE_COUNT: usize = 2;

// Confirmatory linear layouts (Section 11). CK (inherited from TDI-5.5) adds the
// two exact contraction descriptors to the baseline; SK additionally adds the
// two exact spectral moments; GK additionally adds the two literal spectral
// descriptors g and τ_ε (the TDI-6.1 baseline); GKT additionally adds the two
// early overlaps. GK minus SK isolates the literal spectral descriptors'
// marginal value in each family (TDI-6.5D, not a TDI-6.7 criterion); GKT minus GK
// isolates the overlaps' marginal value *after* the contraction descriptors, the
// exact spectral moments AND the literal spectral gap + mixing time are already
// present (the confirmatory comparison, criteria 6.8A, 6.8B, 6.8C, 6.8D).
//   CK  = baseline + delta + delta_bar                              (13 + 2 = 15)
//   SK  = baseline + delta + delta_bar + s2 + s3                    (13 + 4 = 17)
//   GK  = baseline + delta + delta_bar + s2 + s3 + g + τ_ε          (13 + 6 = 19)
//   GKT = baseline + delta + delta_bar + s2 + s3 + g + τ_ε + O1 + O2 (13 + 8 = 21)
const CK_FEATURE_COUNT: usize = BASELINE_FEATURE_COUNT + CONTRACTION_FEATURE_COUNT;
const SK_FEATURE_COUNT: usize =
    BASELINE_FEATURE_COUNT + CONTRACTION_FEATURE_COUNT + SPECTRAL_FEATURE_COUNT;
const GK_FEATURE_COUNT: usize = BASELINE_FEATURE_COUNT
    + CONTRACTION_FEATURE_COUNT
    + SPECTRAL_FEATURE_COUNT
    + LITERAL_SPECTRAL_FEATURE_COUNT;
const GKT_FEATURE_COUNT: usize = GK_FEATURE_COUNT + EARLY_OVERLAP_FEATURE_COUNT;

const MODEL_LAYOUT_COUNT: usize = 9;

const RIDGE_LAMBDA: f64 = 1.0;
const BOOTSTRAP_REPLICATES: usize = 4_000;
// Fresh per-family stratified-aggregate bootstrap seeds (TDI-6.7 Section 7),
// disjoint from every TDI-5.2 … 6.2 bootstrap seed. Each family aggregates its
// own three blocks with seed base + family index.
const AGGREGATE_BOOTSTRAP_SEED_BASE: u64 = 0x5444_4936_3800_4800;

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
/// family (Section 9). Its population seeds and bootstrap seed are computed
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

    /// `base(f, b) = 8.6e9 + f·300e6 + b·100e6` (Section 7). The four
    /// populations start at this base + `{0, 10, 20, 30}·1e6`. The 8.6e9 origin
    /// clears TDI-6.7's last reservation (8,530,005,038), so every TDI-6.8 seed
    /// is disjoint from every prior experiment's.
    ///
    /// This comment was wrong in both ancestors — it announced a 6.2e9 origin
    /// while `v67`'s code used 7.4e9 — because a copy derivation carries stale
    /// prose forward silently. The freshness it describes is not cosmetic: it is
    /// what lets Section 1.3 admit a rank criterion whose hypothesis came from
    /// already-observed data.
    fn population_base_seed(self) -> u64 {
        8_600_000_000 + self.family.index() * 300_000_000 + u64::from(self.block) * 100_000_000
    }

    /// `0x5444_4936_3800_0000 + (SEED_BLOCK_COUNT·f + b) + 1` (Section 7) — the
    /// series ASCII scheme "TDI" + "6" + "8".
    fn bootstrap_seed(self) -> u64 {
        0x5444_4936_3800_0000
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

    /// Offset from the block base seed: 0 / 10M / 20M / 30M (Section 12).
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
    // Linear layouts B0..BD are exploratory in TDI-6.7. Their discriminants
    // (0..4) are preserved so `layout as usize` indexing is unchanged from
    // TDI-5.2/5.3/5.4/5.5. The confirmatory layouts CK/SK/GK/GKT follow, with
    // strict nesting CK ⊂ SK ⊂ GK ⊂ GKT.
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
            Self::Sk => "SK — BASELINE + δ + δ̄ + s2 + s3 (contraction + spectral)",
            Self::Gk => {
                "GK — BASELINE + δ + δ̄ + s2 + s3 + g + τ_ε (contraction + spectral + littéral)"
            }
            Self::Gkt => {
                "GKT — BASELINE + δ + δ̄ + s2 + s3 + g + τ_ε + O1 + O2 \
                 (contraction + spectral + littéral + TDI)"
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

// ---- literal spectral descriptors (transplanted from TDI-6.1) ----
//
// The one-step Noop kernel is assembled exactly (rational rows) and converted to
// f64 once, then the eigenvalues and mixing time are computed in the declared
// single-threaded f64 regime (Section 13). The canonical eigensolver (method 1,
// Section 8) is a pure-Rust, unsafe-free Hessenberg reduction + shifted QR
// iteration in complex arithmetic; tests cross-validate it against power
// iteration and a reference crate within the declared tolerance.

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
/// dependency-free (Section 8, method 1).
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
/// operation order is fixed (Section 13); the iteration is bounded by the frozen
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
/// use this same iteration — Section 8). The frozen threshold is `ε = 1/4`
/// (`MIXING_EPSILON`) and the iteration cap `T_max` (`MIXING_TIME_CAP`); if
/// convergence is not reached within `T_max` the declared deterministic
/// saturation `τ_ε = T_max` is returned (Section 7).
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
/// and converted to `f64` once (Section 4.3). States are enumerated in index
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

/// Cross-check A (Section 8, method 2): an independent witness of `|λ₂|` by
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
/// (Section 7): the literal spectral gap `g = 1 − |λ₂|` and the normalized
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
        // Confirmatory layouts (TDI-6.7 Section 4). Terms are the two exact
        // contraction descriptors (delta, delta_bar); for SK/GK/GKT the two exact
        // spectral moments (s2, s3); for GK/GKT the two literal spectral
        // descriptors (g, τ_ε); and for GKT the two early overlaps (O1, O2) — all
        // already stored on the record. Standardization happens downstream in
        // ridge fitting, exactly like every other feature. The baseline block is
        // untouched and the layouts nest strictly CK ⊂ SK ⊂ GK ⊂ GKT, so GK minus
        // SK isolates the literal spectral descriptors' marginal value and GKT
        // minus GK isolates the overlaps' marginal value beyond all of them.
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
                format!("width {width} is not part of the TDI-6.7 preregistered populations"),
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

    /// The combined training populations, used by TDI-6.7 as the source of the
    /// **target** domain's re-standardization statistics (Section 4.2).
    ///
    /// Deliberately the training populations and not `combined_holdout`: the
    /// holdout is the set being scored, and standardizing on it would make the
    /// design transductive.
    fn combined_training(&self) -> Vec<Record> {
        combine_width_3_and_4(
            &self.training_width_3.report.records,
            &self.training_width_4.report.records,
        )
    }

    /// Every population's full generation report, in `PopulationKind::ALL`
    /// order. Required-raw-output printing walks this instead of the four
    /// named fields directly. TDI-6.7 has no OOD populations (Section 5).
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

/// Preregistration Section 4.4: a feature scale that is non-finite or at or
/// below this floor is replaced by `1.0`, identically in every arm.
const DEGENERATE_SCALE_FLOOR: f64 = 1.0e-12;

/// The per-feature mean and scale of a design matrix, in the frozen
/// accumulation order.
///
/// This is the **single** implementation of the standardization statistics.
/// `fit_ridge` uses it to build a model, and the transfer derivation
/// (Section 4.1) uses it to recompute those statistics on a different record
/// set. Sharing one implementation is deliberate: two copies could drift, and a
/// drift between the fitted standardization and the transfer-time
/// standardization would silently corrupt every arm comparison.
///
/// Reads features only. It has no access to a target value, which is what makes
/// the A1 arm structurally label-free (Section 4.3).
fn feature_standardization(features: &[Vec<f64>]) -> Result<(Vec<f64>, Vec<f64>), String> {
    if features.is_empty() {
        return Err("cannot standardize an empty design matrix".to_owned());
    }

    let feature_count = features[0].len();

    if feature_count == 0 {
        return Err("standardization requires at least one feature".to_owned());
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

        if !scale.is_finite() || *scale <= DEGENERATE_SCALE_FLOOR {
            *scale = 1.0;
        }
    }

    Ok((means, scales))
}

/// TDI-6.7's single changed factor (preregistration Section 3): what the
/// label-free transfer correction *is*.
///
/// TDI-6.6 aligned the feature scale and that destroyed the domain displacement
/// carrying the level (6.6 §5). TDI-6.7 leaves every feature statistic alone and
/// shifts the predicted level by a scalar estimated from the observed horizon.
/// Coefficients are never refitted in any arm (Section 3).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TransferArm {
    /// **B0** — source feature statistics and source target scaler: byte-for-byte
    /// TDI-6.6's A0, carried forward so the B1-vs-B0 comparison is paired.
    SourceStandardized,
}

impl TransferArm {
    /// TDI-6.8 has exactly **one** arm. Section 3: "No correction of any kind
    /// is applied." TDI-6.7's B1 (observable offset) and B2 (oracle target
    /// scaler) are absent by preregistration — B2 in particular fitted the
    /// target scaler, which reads target-domain `U_h`, and Section 12 states
    /// flatly that no target label is read anywhere in TDI-6.8, in any arm, for
    /// any criterion.
    const ALL: [Self; 1] = [Self::SourceStandardized];

    fn label(self) -> &'static str {
        match self {
            Self::SourceStandardized => "plain-transfer",
        }
    }
}

/// The observed-horizon deficit of a record, `u₂ = −log₂(1 - O₂)`.
///
/// Derived from `Record::early_overlap` alone — a **feature**. This function has
/// no access to a target value, which is what makes the B1 arm structurally
/// label-free (preregistration Section 3.3).
fn observed_deficit(record: &Record, horizon: ObservedHorizon) -> Result<f64, String> {
    let overlap = match horizon {
        ObservedHorizon::First => record.early_overlap[0],
        ObservedHorizon::Last => record.early_overlap[1],
    };
    let remainder = 1.0 - overlap;

    if remainder <= 0.0 {
        return Err(format!(
            "observed overlap {overlap} leaves no deficit; the frozen population \
             contract excludes fully-recovered observations, so this must not occur"
        ));
    }

    let deficit = -remainder.log2();

    if !deficit.is_finite() {
        return Err(format!(
            "observed deficit u2 is not finite for O2 = {overlap}"
        ));
    }

    Ok(deficit)
}

/// Which observed horizon the deficit proxy is read at. `Last` (`U₂`) is the
/// Section 14 path; `First` (`U₁`) is the Section 15 companion.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ObservedHorizon {
    First,
    Last,
}

fn observable_shift(
    source_training: &[&[Record]],
    target_training: &[&[Record]],
) -> Result<f64, String> {
    observable_shift_at(source_training, target_training, ObservedHorizon::Last)
}

/// Mean observed deficit pooled over a domain's blocks, in the frozen
/// accumulation order (preregistration Section 3.1, step 2).
fn pooled_observed_deficit(blocks: &[&[Record]], horizon: ObservedHorizon) -> Result<f64, String> {
    let mut total = 0.0_f64;
    let mut count = 0_usize;

    for block in blocks {
        for record in *block {
            total += observed_deficit(record, horizon)?;
            count += 1;
        }
    }

    if count == 0 {
        return Err("cannot pool the observed deficit of an empty domain".to_owned());
    }

    Ok(total / count as f64)
}

fn observable_shift_at(
    source_training: &[&[Record]],
    target_training: &[&[Record]],
    horizon: ObservedHorizon,
) -> Result<f64, String> {
    Ok(pooled_observed_deficit(target_training, horizon)?
        - pooled_observed_deficit(source_training, horizon)?)
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

    let (means, scales) = feature_standardization(features)?;

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

/// Spearman's ρ that **reports** degeneracy instead of hiding it.
///
/// `pearson_correlation` returns `0.0` when either variance vanishes, so
/// `spearman_correlation` silently returns `0.0` for a constant argument. That
/// convention produced a genuinely misleading published line: TDI-5.8's
/// reconstructed-O "Spearman exactly 0.000000000" sits beside
/// `fraction borne basse = 1.0`, i.e. every prediction clamped to one value, so
/// the zero measured saturation rather than lost ordering.
///
/// TDI-6.8 Section 8 requires undefined cases to be counted, not absorbed, so
/// every criterion path uses this function and `None` propagates.
fn rank_correlation(left: &[f64], right: &[f64]) -> Option<f64> {
    assert_eq!(left.len(), right.len());

    if left.len() < 2 {
        return None;
    }

    let left_ranks = average_ranks(left);
    let right_ranks = average_ranks(right);

    // A constant argument has zero rank variance; `ρ` is undefined, not zero.
    if is_constant(&left_ranks) || is_constant(&right_ranks) {
        return None;
    }

    Some(pearson_correlation(&left_ranks, &right_ranks))
}

fn is_constant(values: &[f64]) -> bool {
    values
        .iter()
        .all(|value| value.total_cmp(&values[0]) == std::cmp::Ordering::Equal)
}

/// Kendall's τ-b — the companion rank statistic of Section 15.
///
/// Deliberately the direct `O(n²)` definition rather than a merge-sort inversion
/// count. τ-b exists so that the choice of Spearman cannot be mistaken for a
/// lever (Section 6); a subtle bug in a hand-rolled `O(n log n)` counter would
/// defeat that purpose entirely, and the direct form is checkable against
/// hand-computed cases. Measured cost at the preregistered scale
/// (n = 10,000 per block): 0.177 s per cell, ≈ 1.1 min for all 384 cells.
///
/// Returns `None` when the tie-corrected denominator vanishes — either argument
/// constant, or every pair tied — for the same reason as `rank_correlation`.
fn kendall_tau_b(left: &[f64], right: &[f64]) -> Option<f64> {
    assert_eq!(left.len(), right.len());

    let count = left.len();

    if count < 2 {
        return None;
    }

    let mut concordant = 0_u64;
    let mut discordant = 0_u64;
    let mut left_ties = 0_u64;
    let mut right_ties = 0_u64;

    for first in 0..count {
        for second in (first + 1)..count {
            let left_order = left[first].total_cmp(&left[second]);
            let right_order = right[first].total_cmp(&right[second]);

            match (left_order, right_order) {
                (std::cmp::Ordering::Equal, std::cmp::Ordering::Equal) => {
                    left_ties += 1;
                    right_ties += 1;
                }
                (std::cmp::Ordering::Equal, _) => left_ties += 1,
                (_, std::cmp::Ordering::Equal) => right_ties += 1,
                _ if left_order == right_order => concordant += 1,
                _ => discordant += 1,
            }
        }
    }

    let pairs = (count * (count - 1) / 2) as f64;
    let denominator = ((pairs - left_ties as f64) * (pairs - right_ties as f64)).sqrt();

    if denominator <= 1.0e-15 {
        return None;
    }

    Some((concordant as f64 - discordant as f64) / denominator)
}

/// The rank statistics of one (pair, layout, horizon, seed block) cell.
///
/// Section 16 requires the tie counts to be printed beside the correlations, so
/// they are carried here rather than recomputed at print time.
#[derive(Clone, Copy, Debug, PartialEq)]
struct RankStatistics {
    spearman: Option<f64>,
    kendall_tau_b: Option<f64>,
    tied_truth_pairs: u64,
    tied_prediction_pairs: u64,
}

impl RankStatistics {
    fn evaluate(truth: &[f64], prediction: &[f64]) -> Self {
        Self {
            spearman: rank_correlation(truth, prediction),
            kendall_tau_b: kendall_tau_b(truth, prediction),
            tied_truth_pairs: tied_pairs(truth),
            tied_prediction_pairs: tied_pairs(prediction),
        }
    }

    /// Whether Spearman and τ-b disagree in *direction* — the disagreement
    /// Section 15 requires to be named explicitly wherever it occurs.
    fn direction_disagreement(&self) -> bool {
        match (self.spearman, self.kendall_tau_b) {
            (Some(rho), Some(tau)) => rho * tau < 0.0,
            _ => false,
        }
    }
}

/// Number of tied pairs within a sample, counted from the tie-group sizes so the
/// cost stays `O(n log n)` rather than `O(n²)`.
fn tied_pairs(values: &[f64]) -> u64 {
    let mut sorted = values.to_vec();
    sorted.sort_by(f64::total_cmp);

    let mut total = 0_u64;
    let mut start = 0_usize;

    while start < sorted.len() {
        let mut end = start + 1;

        while end < sorted.len()
            && sorted[start].total_cmp(&sorted[end]) == std::cmp::Ordering::Equal
        {
            end += 1;
        }

        let group = (end - start) as u64;
        total += group * (group - 1) / 2;
        start = end;
    }

    total
}

/// The frozen symmetric margin of TDI-6.8 Section 10, **absolute** on the
/// bounded [−1, 1] rank scale.
///
/// Justified before any data existed: with 10,000 holdout records per block the
/// standard error of a single Spearman ρ is about `1/√(n−1) ≈ 0.010`, so an
/// increment inside ±0.02 cannot be told from sampling noise by the statistic
/// itself. It is also the direct transposition of the campaign's frozen 2 %
/// relative-MSE margin onto a bounded scale. This value may not be revisited
/// after seeing a result.
const RANK_EQUIVALENCE_MARGIN: f64 = 0.02;

/// Section 8: above this fraction of undefined bootstrap replicates a cell's
/// interval is reported as not-available and its classification forced to
/// *Indeterminate*.
const MAX_UNDEFINED_REPLICATE_FRACTION: f64 = 0.01;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RankClassification {
    Beneficial,
    Harmful,
    Equivalent,
    Indeterminate,
}

impl RankClassification {
    fn label(self) -> &'static str {
        match self {
            Self::Beneficial => "beneficial",
            Self::Harmful => "harmful",
            Self::Equivalent => "equivalent",
            Self::Indeterminate => "indeterminate",
        }
    }
}

/// Every sub-condition of the Section 10 rule, carried alongside the verdict.
///
/// The campaign prints all sub-conditions rather than the classification alone,
/// so a reader can see *which* condition decided a cell and no verdict can be
/// quoted without the evidence that produced it.
#[derive(Clone, Debug, PartialEq)]
struct RankComparison {
    classification: RankClassification,
    /// `ρ̄(challenger) − ρ̄(baseline)`, the mean of the three per-block ρ each.
    aggregate_increment: Option<f64>,
    /// Per-block increments, in frozen block order.
    block_increments: Vec<Option<f64>>,
    interval: Option<ConfidenceInterval>,
    undefined_replicates: usize,
    total_replicates: usize,
    /// Section 10 condition 1 and its mirror.
    all_blocks_favour_challenger: bool,
    all_blocks_favour_baseline: bool,
    /// Section 10 condition 2 and its mirror.
    aggregate_increment_at_least_margin: bool,
    aggregate_decrement_at_least_margin: bool,
    /// Section 10 condition 3 and its mirror.
    interval_lower_bound_positive: bool,
    interval_upper_bound_negative: bool,
    /// Section 10 equivalence, both halves.
    all_block_increments_within_margin: bool,
    interval_within_margin: bool,
}

/// Applies the frozen Section 10 rule. Every input is already computed; this
/// function performs no statistics, so the rule and its evidence cannot drift
/// apart.
fn classify_rank_increment(
    block_increments: Vec<Option<f64>>,
    interval: Option<ConfidenceInterval>,
    undefined_replicates: usize,
    total_replicates: usize,
) -> RankComparison {
    let defined = block_increments
        .iter()
        .flatten()
        .copied()
        .collect::<Vec<_>>();
    let all_defined = defined.len() == block_increments.len() && !defined.is_empty();

    let aggregate_increment = if all_defined {
        Some(defined.iter().sum::<f64>() / defined.len() as f64)
    } else {
        None
    };

    let all_blocks_favour_challenger = all_defined && defined.iter().all(|&value| value > 0.0);
    let all_blocks_favour_baseline = all_defined && defined.iter().all(|&value| value < 0.0);

    let aggregate_increment_at_least_margin =
        aggregate_increment.is_some_and(|value| value >= RANK_EQUIVALENCE_MARGIN);
    let aggregate_decrement_at_least_margin =
        aggregate_increment.is_some_and(|value| value <= -RANK_EQUIVALENCE_MARGIN);

    let interval_lower_bound_positive = interval.is_some_and(|bounds| bounds.lower > 0.0);
    let interval_upper_bound_negative = interval.is_some_and(|bounds| bounds.upper < 0.0);

    let all_block_increments_within_margin = all_defined
        && defined
            .iter()
            .all(|value| value.abs() <= RANK_EQUIVALENCE_MARGIN);
    let interval_within_margin = interval.is_some_and(|bounds| {
        bounds.lower >= -RANK_EQUIVALENCE_MARGIN && bounds.upper <= RANK_EQUIVALENCE_MARGIN
    });

    // Section 8: too many undefined replicates forces *Indeterminate* before any
    // other condition is consulted.
    let replicates_usable = total_replicates > 0
        && (undefined_replicates as f64) / (total_replicates as f64)
            <= MAX_UNDEFINED_REPLICATE_FRACTION;

    let classification = if !replicates_usable {
        RankClassification::Indeterminate
    } else if all_blocks_favour_challenger
        && aggregate_increment_at_least_margin
        && interval_lower_bound_positive
    {
        RankClassification::Beneficial
    } else if all_blocks_favour_baseline
        && aggregate_decrement_at_least_margin
        && interval_upper_bound_negative
    {
        RankClassification::Harmful
    } else if all_block_increments_within_margin && interval_within_margin {
        RankClassification::Equivalent
    } else {
        RankClassification::Indeterminate
    };

    RankComparison {
        classification,
        aggregate_increment,
        block_increments,
        interval: if replicates_usable { interval } else { None },
        undefined_replicates,
        total_replicates,
        all_blocks_favour_challenger,
        all_blocks_favour_baseline,
        aggregate_increment_at_least_margin,
        aggregate_decrement_at_least_margin,
        interval_lower_bound_positive,
        interval_upper_bound_negative,
        all_block_increments_within_margin,
        interval_within_margin,
    }
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

#[derive(Clone, Debug)]
struct Tdi52PredictionSet {
    standardized: Vec<f64>,
    reconstructed_overlap: Vec<f64>,
}

/// One fitted layout's prediction set at a horizon.
///
/// TDI-6.8 pools its metrics across the three seed blocks (`pooled_arm_metrics`)
/// and computes every rank statistic from the predictions directly, so no
/// per-block metric is retained here.
#[derive(Clone, Debug)]
struct PredictorEvaluation {
    predictions: Tdi52PredictionSet,
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

/// Evaluates one fitted ridge layout at a horizon, yielding its prediction set.
///
/// Metrics are not computed per block: TDI-6.8 pools them across the three seed
/// blocks and derives every rank statistic from the predictions directly.
fn evaluate_layout(
    layout: FeatureLayout,
    records: &[Record],
    horizon_index: usize,
    models: &HorizonModels,
    scaler: TargetScaler,
) -> Result<PredictorEvaluation, String> {
    let predictions = tdi52_predict(
        records,
        horizon_index,
        layout,
        models.get(horizon_index, layout),
        scaler,
    )?;

    Ok(PredictorEvaluation { predictions })
}

/// Distinct stratified-bootstrap stream per ordered transfer pair.
///
/// TDI-6.5C had a single pair and could key the stream on the source family
/// alone. TDI-6.8D evaluates 12 ordered pairs, and pairs sharing a source would
/// otherwise share a resampling stream; keying on both families keeps each
/// pair's interval independent of the others. Disjoint from the per-family
/// aggregate seeds because the pair offsets start at `0x10`.
fn transfer_pair_bootstrap_seed(source: GeneratorFamily, target: GeneratorFamily) -> u64 {
    AGGREGATE_BOOTSTRAP_SEED_BASE + 0x10 * (1 + source.index()) + target.index()
}

/// One seed block's single-arm evaluation on a transfer target's holdout.
///
/// TDI-6.6 needs a *single-predictor* evaluation, which the inherited
/// comparison machinery cannot express: `evaluate_block_comparison` derives one
/// set of standardized ground-truth values from one scaler and shares it between
/// its two predictors. That is correct for A0-vs-A1 (both carry the source
/// scaler) but cannot represent A2, whose ground truth is standardized by the
/// target scaler (preregistration Section 6).
#[derive(Clone, Debug)]
struct ArmBlockEvaluation {
    seed_block: SeedBlockId,
    records_len: usize,
    standardized_targets: Vec<f64>,
    overlap_targets: Vec<f64>,
    evaluation: PredictorEvaluation,
}

/// Evaluates one arm's fit on one target family's per-block holdouts.
fn evaluate_arm_blocks(
    fit: &AggregateModelFit,
    target_holdouts: [&[Record]; SEED_BLOCK_COUNT],
    horizon_index: usize,
    layout: FeatureLayout,
) -> Result<Vec<ArmBlockEvaluation>, String> {
    let mut blocks = Vec::with_capacity(SEED_BLOCK_COUNT);

    for (seed_block, records) in frozen_block_order(fit.family())
        .into_iter()
        .zip(target_holdouts)
    {
        if records.is_empty() {
            return Err("cannot evaluate an empty transfer population".to_owned());
        }

        let block_fit = fit.block(seed_block);
        let scaler = block_fit.target_scalers[horizon_index];

        let standardized_targets = records
            .iter()
            .map(|record| scaler.standardize(record.targets_u[horizon_index]))
            .collect::<Vec<_>>();

        let overlap_targets = overlap_values(records, horizon_index);

        let evaluation =
            evaluate_layout(layout, records, horizon_index, &block_fit.models, scaler)?;

        blocks.push(ArmBlockEvaluation {
            seed_block,
            records_len: records.len(),
            standardized_targets,
            overlap_targets,
            evaluation,
        });
    }

    Ok(blocks)
}

/// Pools one arm's per-block predictions into aggregate standardized and
/// reconstructed metrics, concatenating in frozen block order exactly as the
/// inherited `pooled_*_metrics` do.
fn pooled_arm_metrics(blocks: &[ArmBlockEvaluation]) -> (Metrics, Metrics) {
    let mut standardized_targets = Vec::new();
    let mut standardized_predictions = Vec::new();
    let mut overlap_targets = Vec::new();
    let mut overlap_predictions = Vec::new();

    for block in blocks {
        standardized_targets.extend_from_slice(&block.standardized_targets);
        standardized_predictions.extend_from_slice(&block.evaluation.predictions.standardized);
        overlap_targets.extend_from_slice(&block.overlap_targets);
        overlap_predictions.extend_from_slice(&block.evaluation.predictions.reconstructed_overlap);
    }

    (
        calculate_metrics(&standardized_targets, &standardized_predictions),
        calculate_metrics(&overlap_targets, &overlap_predictions),
    )
}

/// What one shared-resample run yields, for every layout at once.
#[derive(Clone, Debug)]
struct RankBootstrapOutcome {
    /// Per layout, in the order supplied: the interval of `ρ̄` and how many
    /// replicates were undefined for that layout. Section 11 needs this
    /// per-layout interval; Section 10 needs the increments below.
    per_layout: Vec<(FeatureLayout, Option<ConfidenceInterval>, usize)>,
    /// Per requested `(challenger, baseline)` comparison: the interval of
    /// `Δρ = ρ̄(challenger) − ρ̄(baseline)` and its undefined count. A replicate
    /// counts as undefined here if *either* side was undefined at it.
    increments: Vec<(
        FeatureLayout,
        FeatureLayout,
        Option<ConfidenceInterval>,
        usize,
    )>,
    replicates: usize,
}

impl RankBootstrapOutcome {
    fn layout(&self, layout: FeatureLayout) -> (Option<ConfidenceInterval>, usize) {
        self.per_layout
            .iter()
            .find(|(candidate, _, _)| *candidate == layout)
            .map(|(_, interval, undefined)| (*interval, *undefined))
            .expect("every requested layout is present in its own bootstrap outcome")
    }

    fn increment(
        &self,
        challenger: FeatureLayout,
        baseline: FeatureLayout,
    ) -> (Option<ConfidenceInterval>, usize) {
        self.increments
            .iter()
            .find(|(high, low, _, _)| *high == challenger && *low == baseline)
            .map(|(_, _, interval, undefined)| (*interval, *undefined))
            .expect("every requested comparison is present in its own bootstrap outcome")
    }
}

/// The shared-resample rank bootstrap of TDI-6.8 Section 8.
///
/// **One resample per (pair, horizon, seed block), shared by every layout.**
/// Section 8 requires the two layouts of a comparison to be resampled with the
/// same indices so the increment is paired. Sharing a single draw across all
/// four layouts satisfies that for every comparison simultaneously, lets the
/// resampled truth ranks be computed once instead of once per layout, and makes
/// the whole descriptor ladder — CK→SK→GK→GKT — paired on identical draws
/// rather than merely comparable. Measured at the preregistered scale, the
/// shared form costs 3.09 ms per replicate against 7.72 ms for independent
/// paired comparisons.
///
/// Resampling is **within** a block (Section 8), and a replicate's aggregate
/// value is the mean of the three per-block ρ at that replicate — never a
/// resampling of a pooled sample, which Section 6 forbids outright.
///
/// Undefined-replicate counts are returned rather than silently dropped:
/// Section 8 requires them reported, and they drive the 1 % guard.
fn rank_bootstrap(
    layout_blocks: &[(FeatureLayout, &[ArmBlockEvaluation])],
    comparisons: &[(FeatureLayout, FeatureLayout)],
    seed: u64,
) -> Result<RankBootstrapOutcome, String> {
    let Some((_, reference)) = layout_blocks.first() else {
        return Err("rank bootstrap needs at least one layout".to_owned());
    };

    if reference.is_empty() {
        return Err("rank bootstrap needs a non-empty block set".to_owned());
    }

    for (_, blocks) in layout_blocks {
        if blocks.len() != reference.len() {
            return Err("rank bootstrap block counts disagree between layouts".to_owned());
        }

        for (candidate, anchor) in blocks.iter().zip(*reference) {
            if candidate.seed_block != anchor.seed_block {
                return Err("rank bootstrap block order disagrees between layouts".to_owned());
            }

            if candidate.records_len != anchor.records_len || candidate.records_len == 0 {
                return Err("rank bootstrap dimensions disagree between layouts".to_owned());
            }

            // Every layout scores the same records under the same target scaler,
            // so the standardized truth must be identical across them. Asserted
            // rather than assumed, because the shared truth ranks below — the
            // whole point of one draw — depend on it.
            if candidate.standardized_targets != anchor.standardized_targets {
                return Err("rank bootstrap layouts disagree on the standardized truth".to_owned());
            }
        }
    }

    for (challenger, baseline) in comparisons {
        for layout in [challenger, baseline] {
            if !layout_blocks
                .iter()
                .any(|(candidate, _)| candidate == layout)
            {
                return Err(format!(
                    "rank bootstrap comparison names {}, which was not supplied",
                    layout.label()
                ));
            }
        }
    }

    let count = layout_blocks.len();
    let block_count = reference.len();
    let mut generator = DeterministicRng::new(seed);

    // One `Option<f64>` per layout per replicate. Held in full because Section 10
    // pairs increments replicate-by-replicate: a layout's r-th value is only
    // meaningful beside another layout's r-th value from the same draw.
    let mut per_replicate = vec![Vec::with_capacity(BOOTSTRAP_REPLICATES); count];

    let mut indices = Vec::new();
    let mut truth = Vec::new();
    let mut predictions = Vec::new();

    for _ in 0..BOOTSTRAP_REPLICATES {
        let mut totals = vec![0.0_f64; count];
        let mut defined = vec![true; count];

        for block_index in 0..block_count {
            let records_len = reference[block_index].records_len;

            // The single shared draw. Note it happens unconditionally, before any
            // layout is consulted: the stream must not depend on which layouts
            // have already gone degenerate, or determinism would hinge on the
            // data rather than on the seed alone.
            indices.clear();
            for _ in 0..records_len {
                indices.push(generator.index(records_len));
            }

            truth.clear();
            for &index in &indices {
                truth.push(reference[block_index].standardized_targets[index]);
            }

            // Computed once for every layout — the whole point of sharing.
            let truth_ranks = average_ranks(&truth);
            let truth_defined = !is_constant(&truth_ranks);

            for (layout_index, (_, blocks)) in layout_blocks.iter().enumerate() {
                if !defined[layout_index] {
                    continue;
                }

                if !truth_defined {
                    defined[layout_index] = false;
                    continue;
                }

                predictions.clear();
                for &index in &indices {
                    predictions
                        .push(blocks[block_index].evaluation.predictions.standardized[index]);
                }

                match rank_correlation_against(&truth_ranks, &predictions) {
                    Some(rho) => totals[layout_index] += rho,
                    None => defined[layout_index] = false,
                }
            }
        }

        for layout_index in 0..count {
            per_replicate[layout_index]
                .push(defined[layout_index].then(|| totals[layout_index] / block_count as f64));
        }
    }

    let per_layout = layout_blocks
        .iter()
        .enumerate()
        .map(|(index, (layout, _))| {
            let values = per_replicate[index]
                .iter()
                .flatten()
                .copied()
                .collect::<Vec<_>>();
            let undefined = BOOTSTRAP_REPLICATES - values.len();

            (
                *layout,
                (!values.is_empty()).then(|| confidence_interval(values)),
                undefined,
            )
        })
        .collect::<Vec<_>>();

    let position = |wanted: FeatureLayout| {
        layout_blocks
            .iter()
            .position(|(candidate, _)| *candidate == wanted)
            .expect("comparison layouts were validated above")
    };

    let increments = comparisons
        .iter()
        .map(|&(challenger, baseline)| {
            let high = position(challenger);
            let low = position(baseline);
            let mut values = Vec::new();
            let mut undefined = 0_usize;

            for (high, low) in per_replicate[high].iter().zip(&per_replicate[low]) {
                match (high, low) {
                    (Some(high), Some(low)) => values.push(high - low),
                    _ => undefined += 1,
                }
            }

            (
                challenger,
                baseline,
                (!values.is_empty()).then(|| confidence_interval(values)),
                undefined,
            )
        })
        .collect::<Vec<_>>();

    Ok(RankBootstrapOutcome {
        per_layout,
        increments,
        replicates: BOOTSTRAP_REPLICATES,
    })
}

/// Spearman's ρ against an already-ranked argument, so a shared truth need not
/// be re-ranked per layout. Degeneracy propagates as `None`, exactly as in
/// `rank_correlation`.
fn rank_correlation_against(left_ranks: &[f64], right: &[f64]) -> Option<f64> {
    let right_ranks = average_ranks(right);

    if is_constant(&right_ranks) {
        return None;
    }

    Some(pearson_correlation(left_ranks, &right_ranks))
}

/// Stratified bootstrap interval for a **single** arm's standardized-U `R²`.
///
/// `R² = 1 − RSS/TSS` is recomputed inside each replicate, with `TSS` taken
/// about that replicate's own resampled mean — the resampled sample is the
/// population whose mean the model is being asked to beat.
///
/// Resampling mirrors the inherited paired bootstrap exactly: `count` draws per
/// block, in frozen block order, from one deterministic stream.
fn arm_r_squared_bootstrap(
    blocks: &[ArmBlockEvaluation],
    seed: u64,
) -> Result<ConfidenceInterval, String> {
    let seed_blocks = blocks
        .iter()
        .map(|block| block.seed_block)
        .collect::<Vec<_>>();

    validate_frozen_block_order(&seed_blocks)
        .map_err(|error| format!("arm R² bootstrap {error}"))?;

    for block in blocks {
        if block.records_len == 0
            || block.evaluation.predictions.standardized.len() != block.records_len
            || block.standardized_targets.len() != block.records_len
        {
            return Err("invalid arm R² bootstrap dimensions".to_owned());
        }
    }

    let mut generator = DeterministicRng::new(seed);
    let mut values = Vec::with_capacity(BOOTSTRAP_REPLICATES);

    for _ in 0..BOOTSTRAP_REPLICATES {
        let mut targets = Vec::new();
        let mut residual_squares = 0.0_f64;

        for block in blocks {
            for _ in 0..block.records_len {
                let index = generator.index(block.records_len);
                let target = block.standardized_targets[index];
                let residual = target - block.evaluation.predictions.standardized[index];

                residual_squares += residual * residual;
                targets.push(target);
            }
        }

        let count = targets.len() as f64;
        let mean = targets.iter().sum::<f64>() / count;
        let total_squares = targets
            .iter()
            .map(|value| {
                let deviation = value - mean;

                deviation * deviation
            })
            .sum::<f64>();

        // A degenerate resample (every target identical) has no variance to
        // explain; the frozen scale floor is reused rather than inventing a
        // second convention.
        let r_squared = if total_squares <= DEGENERATE_SCALE_FLOOR {
            0.0
        } else {
            1.0 - residual_squares / total_squares
        };

        values.push(r_squared);
    }

    Ok(confidence_interval(values))
}

fn focal_horizon_indices() -> [usize; FOCAL_HORIZON_COUNT] {
    std::array::from_fn(|slot| {
        target_horizon_index(FOCAL_HORIZONS[slot])
            .expect("every focal horizon belongs to the target horizons")
    })
}

/// Number of descriptors summarised by TDI-6.6 Section 17: the four exact descriptors
/// delta, delta_bar, s2, s3 and the two literal spectral descriptors g, τ_ε.
const DESCRIPTOR_MEAN_COUNT: usize =
    CONTRACTION_FEATURE_COUNT + SPECTRAL_FEATURE_COUNT + LITERAL_SPECTRAL_FEATURE_COUNT;

/// One generator family's populations, its fitted models, and its descriptor
/// holdout means.
///
/// TDI-6.6 carries **no** within-family comparison: every criterion is about
/// transfer between families (Sections 10-13), so TDI-6.5's per-family
/// GKT-vs-GK grid and GK-vs-SK focal diagnostic are not computed here. A family
/// exists in this experiment to be a transfer source (`aggregate_fit`), a
/// transfer target (`blocks`, supplying both the re-standardization statistics
/// and the scored holdouts), and a row of the drift table (`descriptor_means`).
#[derive(Clone, Debug)]
struct FamilyReport {
    family: GeneratorFamily,
    blocks: Vec<BlockPopulations>,
    aggregate_fit: AggregateModelFit,
    /// Holdout means of [delta, delta_bar, s2, s3, g, τ_ε] on this family's
    /// holdout (Section 15, context only).
    descriptor_means: [f64; DESCRIPTOR_MEAN_COUNT],
    /// Section 15 companion: this family's mean **observed** deficits
    /// `[u₁, u₂]` on its training populations — the quantities the offset
    /// estimator reads. Reported so each pair's `Δ` can be read against the
    /// levels it was formed from.
    observed_deficit_means: [f64; 2],
}

/// The four layout arms of Section 3, in ladder order CK ⊂ SK ⊂ GK ⊂ GKT
/// (15 / 17 / 19 / 21 features). The primary comparison is GKT against GK; CK
/// and SK are carried so the ladder is visible and any GKT effect can be read
/// against what the descriptor ladder already contributes. No criterion is
/// defined on CK or SK.
const TRANSFER_LAYOUTS: [FeatureLayout; 4] = [
    FeatureLayout::Ck,
    FeatureLayout::Sk,
    FeatureLayout::Gk,
    FeatureLayout::Gkt,
];

/// The three adjacent rungs of the Section 10 ladder, as `(challenger,
/// baseline)`. Only the first is a criterion; the other two are reported.
const LADDER_COMPARISONS: [(FeatureLayout, FeatureLayout); 3] = [
    (FeatureLayout::Gkt, FeatureLayout::Gk),
    (FeatureLayout::Gk, FeatureLayout::Sk),
    (FeatureLayout::Sk, FeatureLayout::Ck),
];

/// One arm's evaluation of one (ordered pair, layout, focal horizon).
///
/// `standardized` and `reconstructed` are pooled across the three seed blocks.
/// Only the scale-free members — `r_squared_interval`,
/// and the Spearman and calibration slope inside `standardized` — may be
/// compared across arms; `mse` and `mae` may not, because A2 standardizes its
/// ground truth with a different scaler (Section 6).
#[derive(Clone, Debug)]
struct ArmEvaluation {
    arm: TransferArm,
    standardized: Metrics,
    reconstructed: Metrics,
    r_squared_interval: ConfidenceInterval,
    /// Section 16 context: the transferred model's standardized-`R²` interval.
    /// No TDI-6.8 criterion reads it — the criteria are on ranks — but Section
    /// 16 requires it printed, and a negative `R²` beside a positive `ρ̄` is
    /// exactly the "orders but does not level" reading the experiment exists to
    /// separate from "does neither".
    /// TDI-6.8 Section 6: the rank statistics of each seed block, **never**
    /// pooled. One entry per `SEED_BLOCK_COUNT`, in frozen block order.
    ///
    /// Retained per block rather than aggregated here because Section 6 forbids
    /// a pooled rank statistic from entering any criterion, and the only way to
    /// make that structurally impossible is to never form one.
    block_rank_statistics: Vec<RankStatistics>,
}

/// One ordered transfer pair at one layout and one focal horizon.
#[derive(Clone, Debug)]
struct TransferCell {
    layout: FeatureLayout,
    horizon: usize,
    /// One entry per `TransferArm::ALL` — which, in TDI-6.8, is the single
    /// plain-transfer arm of Section 3.
    arms: Vec<ArmEvaluation>,
}

impl TransferCell {
    fn horizon(&self) -> usize {
        self.horizon
    }

    fn arm(&self, arm: TransferArm) -> &ArmEvaluation {
        self.arms
            .iter()
            .find(|entry| entry.arm == arm)
            .expect("every transfer cell holds one evaluation per arm")
    }
}

/// One ordered pair `source → target`: every (layout, focal horizon) cell.
#[derive(Clone, Debug)]
struct TransferPairReport {
    source: GeneratorFamily,
    target: GeneratorFamily,
    cells: Vec<TransferCell>,
    /// Section 14's label-free domain separation `ū₂ᵀ − ū₂ˢ`, computed from
    /// features only over the two domains' TRAINING populations. Section 14
    /// reports its magnitude; TDI-6.8 never applies it to anything.
    observable_shift: f64,
    /// Section 15 companion: the same shift computed at `U₁` instead of `U₂`.
    /// Context only; no criterion consumes it.
    observable_shift_u1: f64,
    /// Sections 11-12, one entry per (layout, focal horizon).
    rank_cells: Vec<RankCell>,
    /// Section 10's three ladder rungs at each focal horizon. Only
    /// `(GKT, GK)` carries a criterion; the other two are reported.
    ladder: Vec<LadderComparison>,
}

/// Section 16's rank record for one (pair, layout, focal horizon).
///
/// Every field the preregistration demands on a `rank_transfers` or `retention`
/// line is held here explicitly, so printing can never present a boolean
/// without the three per-block ρ and the interval that produced it.
#[derive(Clone, Debug)]
struct RankCell {
    layout: FeatureLayout,
    horizon: usize,
    /// The three per-block ρ on the **target**'s holdout, in frozen block order.
    block_rho: [Option<f64>; SEED_BLOCK_COUNT],
    /// `ρ̄` — the mean of the three, undefined if any block is (Section 6).
    mean_rho: Option<f64>,
    /// Section 8's interval for `ρ̄`, with its undefined-replicate count.
    interval: Option<ConfidenceInterval>,
    undefined_replicates: usize,
    /// Section 11: `ρ > 0` in all three blocks **and** a strictly positive
    /// bootstrap lower bound.
    rank_transfers: bool,
    /// Section 12: the same fitted model scored on the **source**'s own holdout.
    within_rho: Option<f64>,
    /// Section 12: `ρ̄_transfer / ρ̄_within` when `ρ̄_within > 0`, else
    /// not-applicable. Never a success or a failure.
    retention: Option<f64>,
}

/// One rung of the Section 10 ladder at one focal horizon.
#[derive(Clone, Debug)]
struct LadderComparison {
    challenger: FeatureLayout,
    baseline: FeatureLayout,
    horizon: usize,
    comparison: RankComparison,
}

/// Criterion TDI-6.8A (Section 10, primary): on the confirmatory F0→F1 pair, the
/// GKT-against-GK rank classification at each focal horizon, with the other two
/// ladder rungs reported beside it so any GKT effect can be read against what
/// the descriptor ladder already contributes. No criterion is defined on those.
#[derive(Clone, Debug)]
struct Tdi68CriterionA {
    per_horizon: Vec<(usize, RankComparison)>,
    ladder: Vec<LadderComparison>,
}

/// Criterion TDI-6.8B (Section 11, primary): does the ordering transfer at all?
///
/// `transfers` is the preregistered conjunction — `rank_transfers` under **GKT**
/// at **both** focal horizons on the confirmatory pair. `located_failures` names
/// every (layout, horizon) whose ordering is at or below zero, which is what the
/// preregistration asks be reported when the conjunction fails.
///
/// This can disagree with TDI-6.8A, and the disagreement is informative: an
/// increment can be *Beneficial* while both correlations sit at zero.
#[derive(Clone, Debug)]
struct Tdi68CriterionB {
    transfers: bool,
    located_failures: Vec<(FeatureLayout, usize)>,
}

/// Criterion TDI-6.8C (Section 12, descriptive): how much ordering survives the
/// domain change. `(layout, horizon, ρ̄_transfer, ρ̄_within, retention)`.
///
/// `retention` is `None` — printed `not-applicable` — whenever `ρ̄_within ≤ 0`,
/// because a ratio against a non-positive reference states nothing. Descriptive:
/// no threshold is preregistered and no value is a success.
/// `(layout, focal horizon, ρ̄_transfer, ρ̄_within, retention)` — the five values
/// Section 16 requires on every retention line, kept together so none can be
/// printed without the others.
type RetentionCell = (FeatureLayout, usize, Option<f64>, Option<f64>, Option<f64>);

#[derive(Clone, Debug)]
struct Tdi68CriterionC {
    per_cell: Vec<RetentionCell>,
}

/// Criterion TDI-6.8D (Section 13, descriptive): all twelve ordered pairs, the
/// consistency of the GKT-against-GK direction across them, and the per-family
/// descriptor drift table.
#[derive(Clone, Debug)]
struct Tdi68CriterionD {
    pairs: Vec<TransferPairReport>,
    /// True iff the GKT-against-GK classification is identical across all twelve
    /// pairs at both focal horizons (Section 13).
    direction_consistent: bool,
    /// Every `(source, target, horizon)` whose classification differs from the
    /// confirmatory pair's at U₃. The preregistration requires each to be named.
    divergent_pairs: Vec<(GeneratorFamily, GeneratorFamily, usize)>,
    per_family_means: Vec<(GeneratorFamily, [f64; DESCRIPTOR_MEAN_COUNT])>,
    ranges: [f64; DESCRIPTOR_MEAN_COUNT],
}

/// Holdout means of the six descriptors (delta, delta_bar, s2, s3, g, τ_ε) over
/// a family's combined holdout populations (TDI-6.6 Section 17).
fn family_descriptor_means(blocks: &[BlockPopulations]) -> [f64; DESCRIPTOR_MEAN_COUNT] {
    let mut sums = [0.0_f64; DESCRIPTOR_MEAN_COUNT];
    let mut count = 0_usize;

    for block in blocks {
        for record in block.combined_holdout() {
            sums[0] += record.contraction[0];
            sums[1] += record.contraction[1];
            sums[2] += record.spectral[0];
            sums[3] += record.spectral[1];
            sums[4] += record.literal_spectral[0];
            sums[5] += record.literal_spectral[1];
            count += 1;
        }
    }

    if count == 0 {
        return [0.0; DESCRIPTOR_MEAN_COUNT];
    }

    sums.map(|sum| sum / count as f64)
}

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
    let descriptor_means = family_descriptor_means(&blocks);

    let trainings = blocks
        .iter()
        .map(BlockPopulations::combined_training)
        .collect::<Vec<_>>();
    let training_refs = trainings.iter().map(Vec::as_slice).collect::<Vec<_>>();
    let observed_deficit_means = [
        pooled_observed_deficit(&training_refs, ObservedHorizon::First)?,
        pooled_observed_deficit(&training_refs, ObservedHorizon::Last)?,
    ];

    Ok(FamilyReport {
        family,
        blocks,
        aggregate_fit,
        descriptor_means,
        observed_deficit_means,
    })
}

impl FamilyReport {
    /// This family's per-block combined holdouts — the records a transfer is
    /// scored on.
    fn combined_holdouts(&self) -> Vec<Vec<Record>> {
        self.blocks
            .iter()
            .map(BlockPopulations::combined_holdout)
            .collect()
    }

    /// This family's per-block combined **training** populations — the source of
    /// the re-standardization statistics when it is a transfer *target*
    /// (Section 4.2). Deliberately not the holdout.
    fn combined_trainings(&self) -> Vec<Vec<Record>> {
        self.blocks
            .iter()
            .map(BlockPopulations::combined_training)
            .collect()
    }
}

#[derive(Clone, Debug)]
struct Tdi68ExperimentReport {
    families: Vec<FamilyReport>,
    criterion_a: Tdi68CriterionA,
    criterion_b: Tdi68CriterionB,
    criterion_c: Tdi68CriterionC,
    criterion_d: Tdi68CriterionD,
}

/// The confirmatory transfer pair of Sections 12-14: F0-base's fitted models
/// evaluated on F1-sparse's holdouts, inherited from TDI-6.5C.
const CONFIRMATORY_TRANSFER_PAIR: (GeneratorFamily, GeneratorFamily) =
    (GeneratorFamily::F0Base, GeneratorFamily::F1Sparse);

/// Every ordered pair of distinct families, in frozen order (Section 15).
fn ordered_transfer_pairs() -> Vec<(GeneratorFamily, GeneratorFamily)> {
    let mut pairs = Vec::with_capacity(GENERATOR_FAMILY_COUNT * (GENERATOR_FAMILY_COUNT - 1));

    for source in GeneratorFamily::ALL {
        for target in GeneratorFamily::ALL {
            if source != target {
                pairs.push((source, target));
            }
        }
    }

    pairs
}

/// Evaluates one ordered transfer pair under all three arms, at both layouts and
/// both focal horizons.
fn evaluate_transfer_pair(
    source: &FamilyReport,
    target: &FamilyReport,
) -> Result<TransferPairReport, String> {
    let target_holdouts = target.combined_holdouts();
    let target_holdout_refs: [&[Record]; SEED_BLOCK_COUNT] =
        std::array::from_fn(|index| target_holdouts[index].as_slice());

    // Section 4.2: the re-standardization statistics come from the target's
    // TRAINING populations, never from the holdouts scored just above.
    let target_trainings = target.combined_trainings();

    let bootstrap_seed = transfer_pair_bootstrap_seed(source.family, target.family);

    // Preregistration Section 3.1: the observable shift is one scalar per pair,
    // pooled over each domain's three training blocks. Computed from features
    // only — `observed_deficit_u2` cannot reach a target value.
    let source_trainings = source.combined_trainings();
    let source_training_refs = source_trainings
        .iter()
        .map(Vec::as_slice)
        .collect::<Vec<_>>();
    let target_training_slices = target_trainings
        .iter()
        .map(Vec::as_slice)
        .collect::<Vec<_>>();
    // Section 14's label-free domain distance. TDI-6.7 applied this quantity as
    // a correction; TDI-6.8 only *reports* its magnitude, because Section 3
    // admits no correction of any kind.
    let shift = observable_shift(&source_training_refs, &target_training_slices)?;

    // Section 3: the source fit is applied to the target's holdouts verbatim.
    // There is exactly one arm and it transforms nothing, so the fit is carried
    // through unchanged — no target training record is read here, and no target
    // scaler is fitted anywhere in TDI-6.8.
    let arm_fits = TransferArm::ALL
        .map(|arm| (arm, source.aggregate_fit.clone()))
        .to_vec();

    let focal_indices = focal_horizon_indices();
    let mut cells = Vec::with_capacity(TRANSFER_LAYOUTS.len() * FOCAL_HORIZON_COUNT);

    for layout in TRANSFER_LAYOUTS {
        for &horizon_index in &focal_indices {
            let mut arms = Vec::with_capacity(TransferArm::ALL.len());

            for (arm, fit) in &arm_fits {
                let blocks = evaluate_arm_blocks(fit, target_holdout_refs, horizon_index, layout)?;
                let (standardized, reconstructed) = pooled_arm_metrics(&blocks);
                let r_squared_interval = arm_r_squared_bootstrap(&blocks, bootstrap_seed)?;

                // TDI-6.8 Section 6: one rank statistic per seed block, computed
                // from that block's own holdout, never across the concatenation.
                let block_rank_statistics = blocks
                    .iter()
                    .map(|block| {
                        RankStatistics::evaluate(
                            &block.standardized_targets,
                            &block.evaluation.predictions.standardized,
                        )
                    })
                    .collect::<Vec<_>>();

                arms.push(ArmEvaluation {
                    arm: *arm,
                    standardized,
                    reconstructed,
                    r_squared_interval,
                    block_rank_statistics,
                });
            }

            cells.push(TransferCell {
                layout,
                horizon: TARGET_HORIZONS[horizon_index],
                arms,
            });
        }
    }

    // Section 15 companion: the same shift measured at the first observed
    // horizon instead of the last. Context only.
    let observable_shift_u1 = observable_shift_at(
        &source_training_refs,
        &target_training_slices,
        ObservedHorizon::First,
    )?;

    // Sections 10-12. All four layouts are evaluated at each focal horizon and
    // fed to ONE shared-resample bootstrap, so every rung of the ladder is paired
    // on identical draws (Section 8).
    //
    // Blocks are recomputed here rather than retained from the cell loop above:
    // holding every layout's prediction vectors alive across the whole pipeline
    // would cost over a gigabyte, and `evaluate_arm_blocks` is `O(n)`, negligible
    // beside the bootstrap it feeds.
    let plain_fit = &arm_fits
        .iter()
        .find(|(arm, _)| *arm == TransferArm::SourceStandardized)
        .expect("arm fits always contain the plain-transfer arm")
        .1;

    let source_holdouts = source.combined_holdouts();
    let source_holdout_refs: [&[Record]; SEED_BLOCK_COUNT] =
        std::array::from_fn(|index| source_holdouts[index].as_slice());

    let mut rank_cells = Vec::with_capacity(TRANSFER_LAYOUTS.len() * FOCAL_HORIZON_COUNT);
    let mut ladder = Vec::with_capacity(LADDER_COMPARISONS.len() * FOCAL_HORIZON_COUNT);

    for (position, &horizon_index) in focal_indices.iter().enumerate() {
        let horizon = FOCAL_HORIZONS[position];

        let mut transfer_blocks = Vec::with_capacity(TRANSFER_LAYOUTS.len());

        for layout in TRANSFER_LAYOUTS {
            transfer_blocks.push((
                layout,
                evaluate_arm_blocks(plain_fit, target_holdout_refs, horizon_index, layout)?,
            ));
        }

        let bootstrap_input = transfer_blocks
            .iter()
            .map(|(layout, blocks)| (*layout, blocks.as_slice()))
            .collect::<Vec<_>>();

        // Section 7 defines one bootstrap stream per ordered pair and no
        // per-horizon term, so both focal horizons re-enter the same frozen
        // stream. The blocks hold the same records at either horizon, so this
        // additionally pairs U₃ against U₆ on identical resamples.
        let outcome = rank_bootstrap(&bootstrap_input, &LADDER_COMPARISONS, bootstrap_seed)?;

        for (layout, blocks) in &transfer_blocks {
            let block_rho = per_block_rank_correlations(blocks);
            let mean_rho = mean_of_defined(&block_rho);
            let (interval, undefined_replicates) = outcome.layout(*layout);

            // Section 11, both conjuncts. The per-block condition is checked on
            // the blocks themselves; the interval condition on the bootstrap.
            let rank_transfers = block_rho.iter().all(|rho| rho.is_some_and(|rho| rho > 0.0))
                && interval.is_some_and(|interval| interval.lower > 0.0);

            // Section 12: the same fitted model, scored on the source's own
            // holdout. Source labels are available in the source domain by
            // construction and are never fitted on; no target label is read.
            let within_blocks =
                evaluate_arm_blocks(plain_fit, source_holdout_refs, horizon_index, *layout)?;
            let within_rho = mean_of_defined(&per_block_rank_correlations(&within_blocks));

            let retention = match (mean_rho, within_rho) {
                (Some(transfer), Some(within)) if within > 0.0 => Some(transfer / within),
                _ => None,
            };

            rank_cells.push(RankCell {
                layout: *layout,
                horizon,
                block_rho,
                mean_rho,
                interval,
                undefined_replicates,
                rank_transfers,
                within_rho,
                retention,
            });
        }

        for (challenger, baseline) in LADDER_COMPARISONS {
            let high = transfer_blocks
                .iter()
                .find(|(layout, _)| *layout == challenger)
                .expect("every ladder layout is evaluated above");
            let low = transfer_blocks
                .iter()
                .find(|(layout, _)| *layout == baseline)
                .expect("every ladder layout is evaluated above");

            let block_increments = per_block_rank_correlations(&high.1)
                .into_iter()
                .zip(per_block_rank_correlations(&low.1))
                .map(|(high, low)| match (high, low) {
                    (Some(high), Some(low)) => Some(high - low),
                    _ => None,
                })
                .collect::<Vec<_>>();

            let (interval, undefined) = outcome.increment(challenger, baseline);

            ladder.push(LadderComparison {
                challenger,
                baseline,
                horizon,
                comparison: classify_rank_increment(
                    block_increments,
                    interval,
                    undefined,
                    outcome.replicates,
                ),
            });
        }
    }

    Ok(TransferPairReport {
        source: source.family,
        target: target.family,
        cells,
        observable_shift: shift,
        observable_shift_u1,
        rank_cells,
        ladder,
    })
}

/// The three per-block Spearman ρ of an arm evaluation, in frozen block order.
fn per_block_rank_correlations(blocks: &[ArmBlockEvaluation]) -> [Option<f64>; SEED_BLOCK_COUNT] {
    std::array::from_fn(|index| {
        blocks.get(index).and_then(|block| {
            rank_correlation(
                &block.standardized_targets,
                &block.evaluation.predictions.standardized,
            )
        })
    })
}

/// `ρ̄` — the mean of the three per-block values, or `None` if any is undefined.
///
/// Section 6 forbids filling an undefined block with anything: a mean over two
/// of three blocks is a different statistic, and reporting it as `ρ̄` would
/// silently change the estimand.
fn mean_of_defined(values: &[Option<f64>; SEED_BLOCK_COUNT]) -> Option<f64> {
    let mut total = 0.0_f64;

    for value in values {
        total += (*value)?;
    }

    Some(total / SEED_BLOCK_COUNT as f64)
}

/// Runs the full TDI-6.6 pipeline: the inherited per-generator sub-pipeline
/// (generate 3 blocks, fit, aggregate) once per family F0..F3, then every
/// ordered transfer pair under the three arms, then the four criteria
/// (Sections 10-13). Callers control scale entirely through `population_specs`;
/// the real 480,000-record run is reached only from `run_full_experiment`'s
/// confirmed `--full` path.
fn run_tdi68_pipeline(
    population_specs: &[PopulationSpec],
) -> Result<Tdi68ExperimentReport, String> {
    let mut families = Vec::with_capacity(GENERATOR_FAMILY_COUNT);

    for family in GeneratorFamily::ALL {
        families.push(run_family_pipeline(family, population_specs)?);
    }

    let family_report = |wanted: GeneratorFamily| -> Result<&FamilyReport, String> {
        families
            .iter()
            .find(|report| report.family == wanted)
            .ok_or_else(|| format!("missing family {} in the pipeline", wanted.label()))
    };

    let mut pairs = Vec::with_capacity(GENERATOR_FAMILY_COUNT * (GENERATOR_FAMILY_COUNT - 1));

    for (source, target) in ordered_transfer_pairs() {
        pairs.push(evaluate_transfer_pair(
            family_report(source)?,
            family_report(target)?,
        )?);
    }

    let (confirmatory_source, confirmatory_target) = CONFIRMATORY_TRANSFER_PAIR;
    let confirmatory = pairs
        .iter()
        .find(|pair| pair.source == confirmatory_source && pair.target == confirmatory_target)
        .ok_or_else(|| "missing the confirmatory transfer pair in the pipeline".to_owned())?;

    let focal_horizons = FOCAL_HORIZONS;

    // TDI-6.8A — GKT against GK on the confirmatory pair (Section 10).
    let rung = |pair: &TransferPairReport, challenger, baseline, horizon| {
        pair.ladder
            .iter()
            .find(|entry| {
                entry.challenger == challenger
                    && entry.baseline == baseline
                    && entry.horizon == horizon
            })
            .map(|entry| entry.comparison.clone())
            .ok_or_else(|| {
                format!(
                    "missing ladder rung {} vs {} at U{horizon}",
                    challenger.label(),
                    baseline.label()
                )
            })
    };

    let mut per_horizon = Vec::with_capacity(FOCAL_HORIZON_COUNT);

    for horizon in focal_horizons {
        per_horizon.push((
            horizon,
            rung(confirmatory, FeatureLayout::Gkt, FeatureLayout::Gk, horizon)?,
        ));
    }

    let criterion_a = Tdi68CriterionA {
        per_horizon,
        ladder: confirmatory
            .ladder
            .iter()
            .filter(|entry| entry.challenger != FeatureLayout::Gkt)
            .cloned()
            .collect(),
    };

    // TDI-6.8B — does the ordering transfer at all (Section 11)? The verdict is
    // the GKT conjunction; every layout is scanned so the failure can be located.
    let mut located_failures = Vec::new();

    for cell in &confirmatory.rank_cells {
        if !cell.rank_transfers {
            located_failures.push((cell.layout, cell.horizon));
        }
    }

    let transfers = focal_horizons.iter().all(|&horizon| {
        confirmatory.rank_cells.iter().any(|cell| {
            cell.layout == FeatureLayout::Gkt && cell.horizon == horizon && cell.rank_transfers
        })
    });

    let criterion_b = Tdi68CriterionB {
        transfers,
        located_failures,
    };

    // TDI-6.8C — retention against the within-domain reference (Section 12).
    let criterion_c = Tdi68CriterionC {
        per_cell: confirmatory
            .rank_cells
            .iter()
            .map(|cell| {
                (
                    cell.layout,
                    cell.horizon,
                    cell.mean_rho,
                    cell.within_rho,
                    cell.retention,
                )
            })
            .collect(),
    };

    // TDI-6.8D — every ordered pair, and whether the GKT-vs-GK direction is
    // identical across all of them at both focal horizons (Section 13).
    let reference = rung(
        confirmatory,
        FeatureLayout::Gkt,
        FeatureLayout::Gk,
        focal_horizons[0],
    )?
    .classification;

    let mut divergent_pairs = Vec::new();

    for pair in &pairs {
        for horizon in focal_horizons {
            let classification =
                rung(pair, FeatureLayout::Gkt, FeatureLayout::Gk, horizon)?.classification;

            if classification != reference {
                divergent_pairs.push((pair.source, pair.target, horizon));
            }
        }
    }

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

    let criterion_d = Tdi68CriterionD {
        direction_consistent: divergent_pairs.is_empty(),
        divergent_pairs,
        pairs,
        per_family_means,
        ranges,
    };

    Ok(Tdi68ExperimentReport {
        families,
        criterion_a,
        criterion_b,
        criterion_c,
        criterion_d,
    })
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

/// Provenance and integrity (TDI-6.5 preregistration Section 21): git commit,
/// compiler/Cargo versions, and the SHA-256 of the v65 evaluator, the TDI-6.5
/// preregistration and the TDI-6.5 scientific manifest — plus the combined
/// lineage (the literal-spectral core is inherited from TDI-6.1/v61 and the
/// exact family machinery from TDI-5.7/v57) and the full frozen ancestor chain
/// TDI-6.2, TDI-6.1, TDI-5.7 … TDI-5.1, each read live and printed for
/// provenance (Section 1).
fn print_tdi52_provenance() {
    println!();
    println!("=== PROVENANCE ET INTÉGRITÉ (Section 21) ===");
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
        "évaluateur TDI-6.5 SHA-256      : {}",
        tdi52_sha256_of_repo_file("tdi-bench/src/bin/tdi-independent-overlap-ablation-v65.rs")
    );
    println!(
        "préenregistrement TDI-6.5 SHA-256 : {}",
        tdi52_sha256_of_repo_file(
            "docs/TDI-6.5-GENERATOR-FAMILY-SPECTRAL-ROBUSTNESS-PREREGISTRATION.md"
        )
    );
    println!(
        "manifeste scientifique TDI-6.5 SHA-256 : {}",
        tdi52_sha256_of_repo_file("docs/TDI-6.5-SCIENTIFIC-CODE.sha256")
    );
    println!(
        "lignée combinée                : cœur littéral-spectral hérité de TDI-6.1 (v61), \
         machinerie exacte des familles héritée de TDI-5.7 (v57)"
    );
    println!();
    println!("--- provenance TDI-6.2 (ancêtre gelé, inchangé) ---");
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
    println!("--- provenance TDI-6.1 (ancêtre gelé, cœur spectral littéral) ---");
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
    println!("--- provenance TDI-5.7 (ancêtre gelé, machinerie des familles) ---");
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
    println!("=== CONSTANTES GELÉES (Section 21) ===");
    println!("--- régime non-exact TDI-6.5 (hérité de TDI-6.1 Section 12 ; Section 13) ---");
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
    println!("nombre de features spectrales (s2, s3)    : {SPECTRAL_FEATURE_COUNT}");
    println!(
        "nombre de features spectrales littérales (g, τ_ε) : {LITERAL_SPECTRAL_FEATURE_COUNT}"
    );
    println!("nombre de dispositions de modèle          : {MODEL_LAYOUT_COUNT}");
    println!("features CK (baseline + δ + δ̄)            : {CK_FEATURE_COUNT}");
    println!("features SK (CK + s2 + s3)                : {SK_FEATURE_COUNT}");
    println!("features GK (SK + g + τ_ε)                : {GK_FEATURE_COUNT}");
    println!("features GKT (GK + O1 + O2)               : {GKT_FEATURE_COUNT}");
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
fn print_tdi65_family_rules() {
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

/// TDI-6.8 Sections 6, 10 and 15: per-seed-block rank statistics and the
/// GKT-against-GK increment.
///
/// Every correlation is printed per block. No pooled rank statistic appears
/// here, because Section 6 forbids one from entering a criterion and the
/// evaluator never forms one.
fn print_tdi68_rank_statistics(report: &Tdi68ExperimentReport) {
    println!();
    println!("=== STATISTIQUES DE RANG PAR BLOC (Sections 6, 10, 15) ===");

    let render = |value: Option<f64>| match value {
        Some(number) => format!("{number:.12}"),
        None => "indéfini".to_owned(),
    };

    for pair in &report.criterion_d.pairs {
        for &horizon in &FOCAL_HORIZONS {
            for layout in TRANSFER_LAYOUTS {
                let Some(cell) = pair
                    .cells
                    .iter()
                    .find(|cell| cell.layout == layout && cell.horizon() == horizon)
                else {
                    continue;
                };

                let arm = cell.arm(TransferArm::SourceStandardized);

                for (index, statistics) in arm.block_rank_statistics.iter().enumerate() {
                    println!(
                        "  {} → {} — {} — U{horizon} — bloc {index} : ρ = {} | τ-b = {} | \
                         paires égales vérité/prédiction = {}/{}{}",
                        pair.source.label(),
                        pair.target.label(),
                        layout.label(),
                        render(statistics.spearman),
                        render(statistics.kendall_tau_b),
                        statistics.tied_truth_pairs,
                        statistics.tied_prediction_pairs,
                        if statistics.direction_disagreement() {
                            "  [DÉSACCORD DE DIRECTION ρ / τ-b — Section 15]"
                        } else {
                            ""
                        }
                    );
                }
            }
        }

        // Section 16: every rank_transfers line prints the three per-block ρ and
        // the interval, never the boolean alone; every retention line prints
        // ρ̄_transfer and ρ̄_within separately.
        for cell in &pair.rank_cells {
            println!(
                "  {} → {} — {} — U{} : ρ par bloc = [{}] | ρ̄ = {} | IC95 = {} | \
                 réplicats indéfinis = {}/{BOOTSTRAP_REPLICATES} | rank_transfers = {}",
                pair.source.label(),
                pair.target.label(),
                cell.layout.label(),
                cell.horizon,
                cell.block_rho
                    .iter()
                    .map(|value| render(*value))
                    .collect::<Vec<_>>()
                    .join(", "),
                render(cell.mean_rho),
                render_interval(cell.interval),
                cell.undefined_replicates,
                if cell.rank_transfers { "oui" } else { "non" }
            );
            println!(
                "      Section 12 — ρ̄_transfert = {} | ρ̄_intra-domaine = {} | rétention = {}",
                render(cell.mean_rho),
                render(cell.within_rho),
                render(cell.retention)
            );
        }

        // Section 14: transferred ordering against the label-free domain distance.
        for cell in pair
            .rank_cells
            .iter()
            .filter(|cell| cell.layout == FeatureLayout::Gkt)
        {
            println!(
                "  {} → {} — Section 14 — U{} : ρ̄(GKT) = {} | distance entre domaines \
                 |ū₂ᵀ − ū₂ˢ| = {:.12}",
                pair.source.label(),
                pair.target.label(),
                cell.horizon,
                render(cell.mean_rho),
                pair.observable_shift.abs()
            );
        }

        for rung in &pair.ladder {
            println!(
                "  {} → {} — {} contre {} à U{} : incréments par bloc = [{}] | \
                 moyenne = {} | marge = ±{RANK_EQUIVALENCE_MARGIN} | IC95 = {} | \
                 réplicats indéfinis = {}/{} | classification = {}{}",
                pair.source.label(),
                pair.target.label(),
                rung.challenger.label(),
                rung.baseline.label(),
                rung.horizon,
                rung.comparison
                    .block_increments
                    .iter()
                    .map(|value| render(*value))
                    .collect::<Vec<_>>()
                    .join(", "),
                render(rung.comparison.aggregate_increment),
                render_interval(rung.comparison.interval),
                rung.comparison.undefined_replicates,
                rung.comparison.total_replicates,
                rung.comparison.classification.label(),
                reading_rule_note(pair, rung)
            );
        }
    }
}

/// Section 13's preregistered reading rule, attached to the same line as the
/// classification it qualifies.
///
/// A *Beneficial* increment whose challenger orders nothing is better-ordered
/// noise, not transfer; a *Harmful* one whose baseline ordered nothing says
/// as little. The preregistration requires the qualification inline, so it
/// cannot be separated from the verdict by a reader skimming the output.
fn reading_rule_note(pair: &TransferPairReport, rung: &LadderComparison) -> &'static str {
    let mean_of = |layout: FeatureLayout| {
        pair.rank_cells
            .iter()
            .find(|cell| cell.layout == layout && cell.horizon == rung.horizon)
            .and_then(|cell| cell.mean_rho)
    };

    match rung.comparison.classification {
        RankClassification::Beneficial
            if mean_of(rung.challenger).is_none_or(|mean| mean <= 0.0) =>
        {
            "  [BRUIT MIEUX ORDONNÉ, PAS DU TRANSFERT — Section 13]"
        }
        RankClassification::Harmful if mean_of(rung.baseline).is_none_or(|mean| mean <= 0.0) => {
            "  [LA RÉFÉRENCE N'ORDONNAIT RIEN NON PLUS — Section 13]"
        }
        _ => "",
    }
}

/// A bootstrap interval, or `indéfini` when every replicate was undefined.
fn render_interval(interval: Option<ConfidenceInterval>) -> String {
    match interval {
        Some(interval) => format!(
            "[{:.12}, {:.12}] médiane {:.12}",
            interval.lower, interval.upper, interval.median
        ),
        None => "indéfini".to_owned(),
    }
}

/// Per-criterion block-level and aggregate conditions (Section 16).
fn print_tdi68_criteria_conditions(report: &Tdi68ExperimentReport) {
    println!();
    println!("=== CONDITIONS PAR CRITÈRE — niveau bloc et agrégat (Section 16) ===");

    let (source, target) = CONFIRMATORY_TRANSFER_PAIR;
    println!();
    println!(
        "paire confirmatoire : {} → {} (Sections 10-13)",
        source.label(),
        target.label()
    );

    for (horizon, comparison) in &report.criterion_a.per_horizon {
        println!();
        println!("TDI-6.8A — GKT contre GK à U{horizon} — les trois conditions du §10 :");
        println!(
            "  1. ρ(GKT) > ρ(GK) dans les trois blocs        : {}",
            comparison.all_blocks_favour_challenger
        );
        println!(
            "  2. Δρ ≥ +{RANK_EQUIVALENCE_MARGIN}                                : {}",
            comparison.aggregate_increment_at_least_margin
        );
        println!(
            "  3. borne inférieure IC95 strictement positive  : {}",
            comparison.interval_lower_bound_positive
        );
        println!(
            "  équivalence — incréments par bloc dans ±{RANK_EQUIVALENCE_MARGIN} : {} | \
             IC95 entièrement dans ±{RANK_EQUIVALENCE_MARGIN} : {}",
            comparison.all_block_increments_within_margin, comparison.interval_within_margin
        );
        println!(
            "  réplicats indéfinis : {}/{} (garde du §8 : au-delà de 1 %, Indeterminate)",
            comparison.undefined_replicates, comparison.total_replicates
        );
    }

    for pair in &report.criterion_d.pairs {
        for cell in &pair.cells {
            println!();
            println!(
                "--- {} → {} — {} — U_{} ---",
                pair.source.label(),
                pair.target.label(),
                cell.layout.label(),
                cell.horizon()
            );

            for arm in &cell.arms {
                println!("  bras {}", arm.arm.label());
                println!(
                    "    R² (U standardisé)     : {:.12}  IC 95 % [{:.9}, {:.9}] (médiane {:.9})",
                    arm.standardized.r_squared,
                    arm.r_squared_interval.lower,
                    arm.r_squared_interval.median,
                    arm.r_squared_interval.upper
                );
                println!(
                    "    Spearman (mis en commun) : {:.12}  [MIS EN COMMUN sur les trois blocs \
                     — n'est PAS le ρ̄ du critère, Section 6]",
                    arm.standardized.spearman
                );
                println!(
                    "    pente de calibration   : {:.12}",
                    arm.standardized.calibration_slope
                );
                // Section 15: the reconstructed-O ρ is never printed without its
                // two bound fractions on the SAME line, so a saturated zero can
                // not be read as a measured collapse of the ordering.
                println!(
                    "    ρ (O reconstruit)      : {:.12}  [fraction à la borne basse = {:.6}, \
                     à la borne haute = {:.6}]  R² = {:.12}",
                    arm.reconstructed.spearman,
                    arm.reconstructed.zero_fraction,
                    arm.reconstructed.one_fraction,
                    arm.reconstructed.r_squared
                );
                println!(
                    "    MSE / MAE              : {:.12} / {:.12}",
                    arm.standardized.mse, arm.standardized.mae
                );
            }
        }
    }
}

/// Final verdict lines for TDI-6.8A, 6.8B, 6.8C and 6.8D (Section 16).
fn print_tdi68_final_verdicts(report: &Tdi68ExperimentReport) {
    println!();
    println!("=== VERDICTS FINAUX (Section 16) ===");

    let (source, target) = CONFIRMATORY_TRANSFER_PAIR;
    let pair_label = format!("{} → {}", source.label(), target.label());
    let confirmatory = report
        .criterion_d
        .pairs
        .iter()
        .find(|pair| pair.source == source && pair.target == target)
        .expect("the confirmatory pair is always evaluated");

    let render = |value: Option<f64>| match value {
        Some(number) => format!("{number:.12}"),
        None => "indéfini".to_owned(),
    };

    for (horizon, comparison) in &report.criterion_a.per_horizon {
        let rung = LadderComparison {
            challenger: FeatureLayout::Gkt,
            baseline: FeatureLayout::Gk,
            horizon: *horizon,
            comparison: comparison.clone(),
        };

        println!(
            "TDI-6.8A — {pair_label} — GKT contre GK à U{horizon} : Δρ = {}, IC95 = {}, \
             classification = {}{}",
            render(comparison.aggregate_increment),
            render_interval(comparison.interval),
            comparison.classification.label(),
            reading_rule_note(confirmatory, &rung)
        );
    }

    for rung in &report.criterion_a.ladder {
        println!(
            "TDI-6.8A — {pair_label} — {} contre {} à U{} (échelle, aucun critère) : Δρ = {}, \
             IC95 = {}, classification = {}{}",
            rung.challenger.label(),
            rung.baseline.label(),
            rung.horizon,
            render(rung.comparison.aggregate_increment),
            render_interval(rung.comparison.interval),
            rung.comparison.classification.label(),
            reading_rule_note(confirmatory, rung)
        );
    }

    println!(
        "TDI-6.8B — l'ordonnancement se transfère (GKT, aux deux horizons focaux) : {}",
        if report.criterion_b.transfers {
            "oui"
        } else {
            "non"
        }
    );

    if report.criterion_b.located_failures.is_empty() {
        println!("TDI-6.8B — échecs localisés : aucun");
    } else {
        // Section 16: a rank_transfers line never prints the boolean alone.
        for (layout, horizon) in &report.criterion_b.located_failures {
            let cell = confirmatory
                .rank_cells
                .iter()
                .find(|cell| cell.layout == *layout && cell.horizon == *horizon)
                .expect("every located failure names an evaluated cell");

            println!(
                "TDI-6.8B — échec localisé : {} à U{horizon} — ρ par bloc = [{}], ρ̄ = {}, \
                 IC95 = {}",
                layout.label(),
                cell.block_rho
                    .iter()
                    .map(|value| render(*value))
                    .collect::<Vec<_>>()
                    .join(", "),
                render(cell.mean_rho),
                render_interval(cell.interval)
            );
        }
    }

    // Section 16: every retention line prints ρ̄_transfer and ρ̄_within separately.
    for (layout, horizon, transfer, within, retention) in &report.criterion_c.per_cell {
        println!(
            "TDI-6.8C — {} à U{horizon} : ρ̄_transfert = {}, ρ̄_intra-domaine = {}, \
             rétention = {}",
            layout.label(),
            render(*transfer),
            render(*within),
            retention.map_or_else(
                || "not-applicable".to_owned(),
                |value| format!("{value:.6}")
            )
        );
    }

    for pair in &report.criterion_d.pairs {
        for cell in pair
            .rank_cells
            .iter()
            .filter(|cell| cell.layout == FeatureLayout::Gkt)
        {
            let rung = pair.ladder.iter().find(|entry| {
                entry.challenger == FeatureLayout::Gkt
                    && entry.baseline == FeatureLayout::Gk
                    && entry.horizon == cell.horizon
            });

            let Some(rung) = rung else {
                continue;
            };

            println!(
                "TDI-6.8D — {} → {} — GKT à U{} : GKT contre GK = {}, Δρ = {}, ρ̄(GKT) = {}, \
                 rank_transfers = {}{}",
                pair.source.label(),
                pair.target.label(),
                cell.horizon,
                rung.comparison.classification.label(),
                render(rung.comparison.aggregate_increment),
                render(cell.mean_rho),
                if cell.rank_transfers { "oui" } else { "non" },
                reading_rule_note(pair, rung)
            );
        }

        println!(
            "TDI-6.8D — {} → {} : distance entre domaines |ū₂ᵀ − ū₂ˢ| = {:.9}, \
             compagnon U1 = {:.9} (Sections 14-15)",
            pair.source.label(),
            pair.target.label(),
            pair.observable_shift.abs(),
            pair.observable_shift_u1
        );
    }

    println!(
        "TDI-6.8D — direction cohérente sur les 12 paires ordonnées (GKT contre GK, deux \
         horizons focaux) : {}",
        if report.criterion_d.direction_consistent {
            "oui"
        } else {
            "non"
        }
    );

    for (source, target, horizon) in &report.criterion_d.divergent_pairs {
        println!(
            "TDI-6.8D — paire divergente : {} → {} à U{horizon}",
            source.label(),
            target.label()
        );
    }

    for family_report in &report.families {
        println!(
            "TDI-6.8D — famille {} : moyenne u1 = {:.9}, moyenne u2 = {:.9} \
             (Section 15, contexte)",
            family_report.family.label(),
            family_report.observed_deficit_means[0],
            family_report.observed_deficit_means[1]
        );
    }

    for (family, means) in &report.criterion_d.per_family_means {
        println!(
            "TDI-6.8D — famille {} : δ={:.6}, δ̄={:.6}, s2={:.6}, s3={:.6}, g={:.6}, τ_ε={:.6}",
            family.label(),
            means[0],
            means[1],
            means[2],
            means[3],
            means[4],
            means[5]
        );
    }

    let ranges = report.criterion_d.ranges;
    println!(
        "TDI-6.8D — étendues inter-familles : δ={:.6}, δ̄={:.6}, s2={:.6}, s3={:.6}, g={:.6}, \
         τ_ε={:.6}",
        ranges[0], ranges[1], ranges[2], ranges[3], ranges[4], ranges[5]
    );
}

/// The three-method spectral cross-validation table (Section 21). For a bounded
/// sample of real candidate kernels drawn from EACH of the four families F0–F3
/// (Section 4.4), print `|λ2|` from method 1 (canonical shifted QR, the frozen
/// path) and method 2 (deflated power iteration), their disagreement, the gap
/// g = 1 − |λ2|, the normalized mixing time and the method-1 trace-consistency
/// residual — reported per family. Method 3 (a reference eigensolver) is a
/// test-only dev-dependency exercised over the closed-form known-spectra battery
/// in the bounded test suite; where offline vendoring is unavailable that suite
/// falls back to the methods-1↔2 agreement plus the known-spectra battery
/// (Section 4.4), which alone establish the canonical path's correctness.
/// Cross-method agreement within `SPECTRAL_CROSS_METHOD_TOLERANCE` is the
/// correctness guarantee that replaces bit-exact reproduction for these two
/// non-exact descriptors.
fn print_spectral_cross_validation_table() {
    println!();
    println!("=== TABLE DE VALIDATION CROISÉE SPECTRALE (Section 21) ===");
    println!(
        "tolérance d'accord inter-méthodes η_x         : {SPECTRAL_CROSS_METHOD_TOLERANCE:.1e}"
    );
    println!(
        "méthode 1 = QR décalé canonique (chemin gelé) ; méthode 2 = itération de puissance déflatée ; \
         méthode 3 = crate de référence (dev-dependency, vérifiée dans la suite de tests / repli batterie à spectre connu)"
    );
    println!("candidats échantillonnés dans CHACUNE des quatre familles F0–F3 (Section 4.4)");
    println!(
        "{:<10} {:<7} {:>10} {:>12} {:>12} {:>12} {:>12} {:>12}",
        "famille", "largeur", "graine", "|λ2| m1", "|λ2| m2", "|m1-m2|", "g m1", "résidu-trace"
    );

    let mut worst_disagreement = 0.0_f64;
    let mut per_family_worst_residual = [0.0_f64; GENERATOR_FAMILY_COUNT];

    for (family, width, seed, matrix) in spectral_cross_validation_samples() {
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
        let family_slot = family.index() as usize;
        per_family_worst_residual[family_slot] =
            f64::max(per_family_worst_residual[family_slot], residual);
        let family_label = family.label();
        println!(
            "{family_label:<10} {width:<7} {seed:>10} {slem_method1:>12.9} {slem_method2:>12.9} \
             {disagreement:>12.2e} {gap:>12.9} {residual:>12.2e}  (τ/T_max={normalized_tau:.6})"
        );
    }

    for family in GeneratorFamily::ALL {
        let family_label = family.label();
        let residual = per_family_worst_residual[family.index() as usize];
        println!(
            "résidu de trace max méthode 1 — famille {family_label:<9} : {residual:.2e}  \
             [témoin rigoureux — niveau machine]"
        );
    }
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
/// cross-validation table, drawn from EACH of the four generator families
/// (Section 4.4): consecutive generator seeds from each family's first block
/// (block 0) width-3 and width-4 training populations, each built through the
/// same `generate_family_masks` → `build_system` → `kernel_matrix` path the
/// experiment uses. Kernels that fail to build are skipped; the scan is bounded
/// so the diagnostic never dominates the run.
fn spectral_cross_validation_samples() -> Vec<(GeneratorFamily, u8, u64, Vec<Vec<f64>>)> {
    const SAMPLES_PER_WIDTH: usize = 3;
    const MAX_SCAN_PER_WIDTH: u64 = 256;

    let mut samples = Vec::new();
    for family in GeneratorFamily::ALL {
        let block = frozen_block_order(family)[0];
        let base = block.population_base_seed();
        let widths_and_bases = [
            (
                TRAIN_WIDTH_3,
                base + PopulationKind::TrainingWidth3.seed_offset(),
            ),
            (
                TRAIN_WIDTH_4,
                base + PopulationKind::TrainingWidth4.seed_offset(),
            ),
        ];

        for (width, seed_base) in widths_and_bases {
            let mut collected = 0;
            let mut offset = 0_u64;
            while collected < SAMPLES_PER_WIDTH && offset < MAX_SCAN_PER_WIDTH {
                let seed = seed_base + offset;
                offset += 1;
                let context = AttemptContext::new(family, width, seed, 0);
                let Ok(masks) = generate_family_masks(context) else {
                    continue;
                };
                let Ok(system) = build_system(context, &masks) else {
                    continue;
                };
                let Ok(matrix) = kernel_matrix(context, &system) else {
                    continue;
                };
                samples.push((family, width, seed, matrix));
                collected += 1;
            }
        }
    }

    samples
}

/// Required raw output in the frozen order of Section 19.
fn print_tdi68_required_raw_output(report: &Tdi68ExperimentReport) {
    print_tdi52_provenance();
    print_tdi52_frozen_constants();
    print_tdi65_family_rules();
    print_tdi52_seed_block_definitions();
    print_spectral_cross_validation_table();

    for family_report in &report.families {
        print_tdi52_population_accounting(&family_report.blocks);
    }

    for family_report in &report.families {
        for seed_block in frozen_block_order(family_report.family) {
            let fit = family_report.aggregate_fit.block(seed_block);

            println!();
            println!(
                "### BLOC {} — normalisations et modèles (Section 19) ###",
                seed_block.label()
            );
            tdi52_print_models(&fit.models, &fit.target_scalers);
        }
    }

    // The B1-vs-B0 comparison for every ordered pair, layout and focal horizon:
    // the only relative-MSE comparison the design admits (Section 6). The
    // per-arm scale-free quantities are printed by the criteria-conditions
    // section, which walks the same cells.

    print_tdi68_criteria_conditions(report);
    print_tdi68_rank_statistics(report);
    print_tdi68_final_verdicts(report);
}

fn run_termination_smoke() -> Result<(), String> {
    println!("=== TDI-6.5 TERMINATION SMOKE ===");

    // Inherited frozen invariant: the width-6 successor-set space is the
    // exact 2^64. TDI-6.5 generates no width-6 populations, but the
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
    // CK/SK/GK/GKT without any real generation. Their contraction descriptors,
    // exact spectral moments and literal spectral descriptors are set by hand.
    let synthetic_training_width_3 = [
        Record {
            baseline: [0.0; BASELINE_FEATURE_COUNT],
            early_overlap: [0.20, 0.55],
            contraction: [0.50, 0.40],
            spectral: [1.80, 1.40],
            literal_spectral: [0.70, 0.05],
            overlaps: [0.30; TARGET_HORIZON_COUNT],
            targets_u: [1.00, 1.10, 1.20, 1.30, 1.35, 1.40],
        },
        Record {
            baseline: [0.1; BASELINE_FEATURE_COUNT],
            early_overlap: [0.25, 0.60],
            contraction: [0.62, 0.31],
            spectral: [2.10, 1.60],
            literal_spectral: [0.55, 0.12],
            overlaps: [0.32; TARGET_HORIZON_COUNT],
            targets_u: [1.50, 1.35, 1.25, 1.15, 1.10, 1.05],
        },
        Record {
            baseline: [0.15; BASELINE_FEATURE_COUNT],
            early_overlap: [0.30, 0.50],
            contraction: [0.44, 0.28],
            spectral: [1.50, 1.20],
            literal_spectral: [0.82, 0.03],
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
            literal_spectral: [0.40, 0.20],
            overlaps: [0.36; TARGET_HORIZON_COUNT],
            targets_u: [2.00, 1.90, 1.80, 1.70, 1.65, 1.60],
        },
        Record {
            baseline: [0.05; BASELINE_FEATURE_COUNT],
            early_overlap: [0.40, 0.65],
            contraction: [0.58, 0.36],
            spectral: [2.30, 1.90],
            literal_spectral: [0.61, 0.09],
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
        "layout feature widths        : CK={} (attendu {}), SK={} (attendu {}), GK={} (attendu {}), \
         GKT={} (attendu {})",
        ck_features.len(),
        CK_FEATURE_COUNT,
        sk_features.len(),
        SK_FEATURE_COUNT,
        gk_features.len(),
        GK_FEATURE_COUNT,
        gkt_features.len(),
        GKT_FEATURE_COUNT
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

    // Exercise the confirmatory GKT-vs-GK rank comparison and the four-way
    // classifier (criterion TDI-6.8A) at the primary horizon, through the same
    // shared-resample bootstrap the real pipeline uses.
    let mut smoke_blocks = Vec::new();

    for layout in TRANSFER_LAYOUTS {
        smoke_blocks.push((
            layout,
            evaluate_arm_blocks(
                &aggregate_fit,
                holdout_refs,
                primary_horizon_index(),
                layout,
            )?,
        ));
    }

    let smoke_input = smoke_blocks
        .iter()
        .map(|(layout, blocks)| (*layout, blocks.as_slice()))
        .collect::<Vec<_>>();
    let smoke_outcome = rank_bootstrap(&smoke_input, &LADDER_COMPARISONS, 0x5444_4936_3800_4801)?;

    let (gkt_interval, gkt_undefined) = smoke_outcome.layout(FeatureLayout::Gkt);
    println!(
        "identity smoke ρ̄(GKT) IC     : {} (réplicats indéfinis {gkt_undefined}/{})",
        render_interval(gkt_interval),
        smoke_outcome.replicates
    );

    for (challenger, baseline) in LADDER_COMPARISONS {
        let (interval, undefined) = smoke_outcome.increment(challenger, baseline);
        println!(
            "identity smoke {} vs {} : IC95 = {} (indéfinis {undefined})",
            challenger.label(),
            baseline.label(),
            render_interval(interval)
        );
    }

    // The critical wiring smoke: the real pipeline entrypoint, run at tiny
    // scale by requesting exactly one accepted record per population.
    let tiny_population_specs = population_specs().map(|spec| PopulationSpec {
        target_count: 1,
        ..spec
    });

    let pipeline_report =
        run_tdi68_pipeline(&tiny_population_specs).map_err(|error| error.to_string())?;

    println!(
        "identity smoke pipeline      : familles={}, paires={}, 6.8A[GKT vs GK, U3]={}, \
         6.8B transfère={}",
        pipeline_report.families.len(),
        pipeline_report.criterion_d.pairs.len(),
        pipeline_report.criterion_a.per_horizon[0]
            .1
            .classification
            .label(),
        pipeline_report.criterion_b.transfers
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

    print_tdi68_required_raw_output(&pipeline_report);

    println!("bounded smoke result         : PASS");

    Ok(())
}

/// Name of the environment variable that must carry the exact TDI-6.5
/// full-run confirmation value. See TDI-6.5 preregistration Section 20.
const TDI68_FULL_RUN_CONFIRMATION_VAR: &str = "TDI68_CONFIRM_FULL_RUN";

/// The one accepted value for `TDI68_FULL_RUN_CONFIRMATION_VAR`. Any other
/// value, or the variable being unset, must refuse `--full`.
const TDI68_FULL_RUN_CONFIRMATION_VALUE: &str = "I_ACCEPT_THE_TDI68_FREEZE_RULE";

/// Pure decision function: takes the confirmation value as a plain
/// `Option<&str>` rather than reading the environment itself, so every
/// branch -- missing, wrong, and the one exact accepted value -- can be
/// unit tested directly without ever touching a real environment variable
/// or risking the accepted branch reaching `run_full_experiment` (and,
/// through it, the real pipeline).
fn tdi68_full_run_confirmed(value: Option<&str>) -> bool {
    value == Some(TDI68_FULL_RUN_CONFIRMATION_VALUE)
}

fn tdi68_usage_error() -> String {
    format!(
        "usage: tdi-independent-overlap-ablation-v68 --termination-smoke|--preflight|--full\n\
         a bare (no-argument) invocation does not start the experiment; the \
         real run additionally requires the exact environment variable \
         {TDI68_FULL_RUN_CONFIRMATION_VAR}={TDI68_FULL_RUN_CONFIRMATION_VALUE}"
    )
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Tdi68Mode {
    TerminationSmoke,
    Preflight,
    Full,
}

/// Pure command-line dispatch decision, independent of `main`'s I/O, so
/// that "a bare invocation can never select `--full`" is directly unit
/// testable against plain string slices rather than real process argv.
fn tdi68_parse_mode(arguments: &[String]) -> Result<Tdi68Mode, String> {
    match arguments {
        [flag] if flag == "--termination-smoke" => Ok(Tdi68Mode::TerminationSmoke),
        [flag] if flag == "--preflight" => Ok(Tdi68Mode::Preflight),
        [flag] if flag == "--full" => Ok(Tdi68Mode::Full),
        _ => Err(tdi68_usage_error()),
    }
}

fn main() -> Result<(), String> {
    let arguments = std::env::args().skip(1).collect::<Vec<_>>();

    match tdi68_parse_mode(&arguments)? {
        Tdi68Mode::TerminationSmoke => run_termination_smoke(),
        Tdi68Mode::Preflight => run_preflight(),
        Tdi68Mode::Full => run_full_experiment(),
    }
}

/// The TDI-6.5 full-run entrypoint. Checks the exact confirmation
/// environment variable *before* any generation, fitting or bootstrap;
/// only when it matches does this call the real full pipeline exactly
/// once, over the real preregistered `population_specs()`, and print the
/// complete required raw output. See TDI-6.5 preregistration Section 20.
fn run_full_experiment() -> Result<(), String> {
    let confirmation = std::env::var(TDI68_FULL_RUN_CONFIRMATION_VAR).ok();

    if !tdi68_full_run_confirmed(confirmation.as_deref()) {
        return Err(format!(
            "TDI-6.7 full execution requires the exact confirmation environment \
             variable {TDI68_FULL_RUN_CONFIRMATION_VAR}={TDI68_FULL_RUN_CONFIRMATION_VALUE}; \
             refusing before any generation, fitting or bootstrap"
        ));
    }

    let report = run_tdi68_pipeline(&population_specs())?;

    print_tdi68_required_raw_output(&report);

    Ok(())
}

/// TDI-6.5 preflight: verifies the complete frozen configuration (seed
/// reservations, population counts, bootstrap constants, pipeline wiring)
/// and prints identities and the exact real-run command, without ever
/// generating a scientific population. See TDI-6.5 preregistration
/// Section 20.
fn run_preflight() -> Result<(), String> {
    println!();
    println!("=== TDI-6.5 PREFLIGHT (aucune génération scientifique) ===");

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
        "pipeline complet câblé à --full                 : oui (run_tdi68_pipeline, \
         subordonné à {TDI68_FULL_RUN_CONFIRMATION_VAR})"
    );

    print_tdi52_provenance();

    println!();
    println!("Commande requise pour l'exécution réelle (jamais lancée automatiquement) :");
    println!("  {TDI68_FULL_RUN_CONFIRMATION_VAR}={TDI68_FULL_RUN_CONFIRMATION_VALUE} \\");
    println!("    bash scripts/reproduce-tdi6.5.sh");

    println!();
    println!("=== PREFLIGHT TERMINÉ : aucun résultat produit ===");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        BASELINE_FEATURE_COUNT, BOOTSTRAP_REPLICATES, CK_FEATURE_COUNT, CONTRACTION_FEATURE_COUNT,
        Cardinality, Complex64, FOCAL_HORIZONS, FeatureLayout, GENERATOR_FAMILY_COUNT,
        GK_FEATURE_COUNT, GKT_FEATURE_COUNT, GeneratorFamily, LITERAL_SPECTRAL_FEATURE_COUNT,
        MIXING_EPSILON, MIXING_TIME_CAP, MODEL_LAYOUT_COUNT, PRIMARY_HORIZON, Record,
        SEED_BLOCK_COUNT, SK_FEATURE_COUNT, SPECTRAL_CROSS_METHOD_TOLERANCE,
        SPECTRAL_FEATURE_COUNT, TARGET_HORIZONS, TDI68_FULL_RUN_CONFIRMATION_VALUE,
        TDI68_FULL_RUN_CONFIRMATION_VAR, TOTAL_SEED_RESERVATIONS,
    };
    use tdi_core::{Action, State, TableSystem};

    fn read_repo_file(relative_path: &str) -> String {
        std::fs::read_to_string(super::tdi52_repository_root().join(relative_path))
            .unwrap_or_else(|error| panic!("failed to read {relative_path}: {error}"))
    }

    /// This evaluator's **own** source.
    ///
    /// The v65 derivation left this pointing at the ancestor, so every
    /// source-inspecting guard in this file was silently validating v65 instead
    /// of itself — the guards would have stayed green while it drifted. The
    /// frozen-ancestor hash tests below read v65 deliberately and separately.
    fn evaluator_source() -> String {
        read_repo_file("tdi-bench/src/bin/tdi-independent-overlap-ablation-v68.rs")
    }

    fn record_with_overlap(o1: f64, o2: f64) -> Record {
        Record {
            baseline: [
                0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0, 1.1, 1.2, 1.3,
            ],
            early_overlap: [o1, o2],
            contraction: [(o1 + o2) / 2.0, o1 * o2],
            spectral: [1.0 + o1, 1.0 + o2],
            literal_spectral: [0.5, 0.25],
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
    fn gk_features_add_contraction_spectral_then_the_two_literal_descriptors() {
        let mut record = record_with_overlap(0.4, 0.6);
        record.contraction = [0.7, 0.3];
        record.spectral = [1.8, 1.4];
        record.literal_spectral = [0.66, 0.11];
        let features = super::feature_layout(&record, FeatureLayout::Gk);

        assert_eq!(features.len(), GK_FEATURE_COUNT);
        assert_eq!(features.len(), FeatureLayout::Gk.feature_count());
        assert_eq!(&features[..BASELINE_FEATURE_COUNT], &record.baseline);
        let tail = &features[BASELINE_FEATURE_COUNT..];
        assert_eq!(tail, &[0.7, 0.3, 1.8, 1.4, 0.66, 0.11]);
    }

    #[test]
    fn gkt_features_add_contraction_spectral_literal_then_the_two_overlaps() {
        let (o1, o2) = (0.4, 0.6);
        let mut record = record_with_overlap(o1, o2);
        record.contraction = [0.7, 0.3];
        record.spectral = [1.8, 1.4];
        record.literal_spectral = [0.66, 0.11];
        let features = super::feature_layout(&record, FeatureLayout::Gkt);

        assert_eq!(features.len(), GKT_FEATURE_COUNT);
        assert_eq!(features.len(), FeatureLayout::Gkt.feature_count());
        assert_eq!(&features[..BASELINE_FEATURE_COUNT], &record.baseline);
        let tail = &features[BASELINE_FEATURE_COUNT..];
        assert_eq!(tail, &[0.7, 0.3, 1.8, 1.4, 0.66, 0.11, o1, o2]);
    }

    #[test]
    fn confirmatory_layouts_never_perturb_the_baseline_block_and_nest_ck_sk_gk_gkt() {
        // The 13 baseline features are identical across B0, CK, SK, GK and GKT:
        // only the appended descriptor/overlap block differs, so any
        // GKT-minus-GK signal is the overlaps' and any GK-minus-SK signal is the
        // literal spectral descriptors'. The layouts nest strictly
        // CK ⊂ SK ⊂ GK ⊂ GKT.
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
        // unchanged from TDI-5.2/5.3/5.4/5.5.
        assert_eq!(FeatureLayout::B0 as usize, 0);
        assert_eq!(FeatureLayout::Ck as usize, 5);
        assert_eq!(FeatureLayout::Sk as usize, 6);
        assert_eq!(FeatureLayout::Gk as usize, 7);
        assert_eq!(FeatureLayout::Gkt as usize, 8);
    }

    #[test]
    fn confirmatory_layout_counts_are_fifteen_seventeen_nineteen_and_twenty_one() {
        assert_eq!(CONTRACTION_FEATURE_COUNT, 2);
        assert_eq!(SPECTRAL_FEATURE_COUNT, 2);
        assert_eq!(LITERAL_SPECTRAL_FEATURE_COUNT, 2);
        assert_eq!(CK_FEATURE_COUNT, 15);
        assert_eq!(SK_FEATURE_COUNT, 17);
        assert_eq!(GK_FEATURE_COUNT, 19);
        assert_eq!(GKT_FEATURE_COUNT, 21);
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

    // --- Full-run confirmation guard (Section 16) ---

    #[test]
    fn full_run_confirmation_accepts_only_the_exact_value() {
        assert!(super::tdi68_full_run_confirmed(Some(
            TDI68_FULL_RUN_CONFIRMATION_VALUE
        )));
        assert!(!super::tdi68_full_run_confirmed(None));
        assert!(!super::tdi68_full_run_confirmed(Some("")));
        assert!(!super::tdi68_full_run_confirmed(Some(
            "i_accept_the_tdi65_freeze_rule"
        )));
        // The frozen TDI-5.4 token must never unlock TDI-6.5.
        assert!(!super::tdi68_full_run_confirmed(Some(
            "I_ACCEPT_THE_TDI54_FREEZE_RULE"
        )));
    }

    #[test]
    fn parse_mode_rejects_a_bare_no_argument_invocation() {
        assert!(super::tdi68_parse_mode(&[]).is_err());
        assert!(super::tdi68_parse_mode(&["--full".to_owned(), "extra".to_owned()]).is_err());
    }

    #[test]
    fn parse_mode_selects_full_only_for_the_exact_single_flag() {
        assert_eq!(
            super::tdi68_parse_mode(&["--full".to_owned()]).unwrap(),
            super::Tdi68Mode::Full
        );
        assert_eq!(
            super::tdi68_parse_mode(&["--preflight".to_owned()]).unwrap(),
            super::Tdi68Mode::Preflight
        );
        assert_eq!(
            super::tdi68_parse_mode(&["--termination-smoke".to_owned()]).unwrap(),
            super::Tdi68Mode::TerminationSmoke
        );
        assert!(super::tdi68_parse_mode(&["--Full".to_owned()]).is_err());
    }

    #[test]
    fn usage_error_mentions_every_flag_and_the_confirmation_variable() {
        let usage = super::tdi68_usage_error();
        assert!(usage.contains("--termination-smoke"));
        assert!(usage.contains("--preflight"));
        assert!(usage.contains("--full"));
        assert!(usage.contains(TDI68_FULL_RUN_CONFIRMATION_VAR));
        assert!(usage.contains(TDI68_FULL_RUN_CONFIRMATION_VALUE));
    }

    #[test]
    fn full_run_refuses_before_any_work_without_the_confirmation_token() {
        // Never reach the accepted path in a test: assert the guard var is
        // absent first, then confirm the unconfirmed call returns an error
        // before any generation, fitting or bootstrap.
        if std::env::var(TDI68_FULL_RUN_CONFIRMATION_VAR).is_ok() {
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
            body.contains("run_tdi68_pipeline(&population_specs())"),
            "accepted path must call the real pipeline over the real specs"
        );
        assert!(body.contains("tdi68_full_run_confirmed"));
        assert!(body.contains("print_tdi68_required_raw_output"));
    }

    #[test]
    fn termination_smoke_uses_only_bounded_specs_never_the_real_ones() {
        let source = evaluator_source();
        let start = source
            .find("fn run_termination_smoke()")
            .expect("run_termination_smoke must exist");
        let end = source[start..]
            .find("\nfn tdi68_full_run_confirmed")
            .map(|offset| start + offset)
            .expect("tdi68_full_run_confirmed must follow run_termination_smoke");
        let body = &source[start..end];

        assert!(body.contains("target_count: 1"));
        assert!(
            !body.contains("run_tdi68_pipeline(&population_specs())"),
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
        // Four families × three blocks, every population seed ≥ 6.2e9 — entirely
        // above TDI-6.5's last reservation (6.13e9 + 5030) and every earlier
        // block. All 48 population seeds, all 12 block bootstrap seeds and all 4
        // family aggregate seeds are distinct (Sections 8, 9).
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
                    assert!(seed >= 8_600_000_000);
                    population_seeds.push(seed);
                }
                bootstrap_seeds.push(seed_block.bootstrap_seed());
            }
            aggregate_seeds.push(super::family_aggregate_bootstrap_seed(family));
        }

        // Anchored constants: the first and last derived bootstrap seeds and the
        // first family aggregate seed (base 0x5444_4936_3800_….., distinct from
        // the TDI-6.6 base 0x5444_4936_3700_…..).
        assert_eq!(GeneratorFamily::F0Base.index(), 0);
        assert_eq!(
            super::frozen_block_order(GeneratorFamily::F0Base)[0].bootstrap_seed(),
            0x5444_4936_3800_0001
        );
        assert_eq!(
            super::frozen_block_order(GeneratorFamily::F3Local)[SEED_BLOCK_COUNT - 1]
                .bootstrap_seed(),
            0x5444_4936_3800_000C
        );
        assert_eq!(
            super::family_aggregate_bootstrap_seed(GeneratorFamily::F0Base),
            0x5444_4936_3800_4800
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
    fn generate_records_is_deterministic_and_carries_contraction_spectral_and_literal_descriptors()
    {
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
            // The two non-exact literal spectral descriptors are reproduced
            // bit-for-bit on the same toolchain (Section 13).
            assert_eq!(a.literal_spectral, b.literal_spectral);
            assert_eq!(a.targets_u, b.targets_u);
        }
        // The contraction descriptors are finite and in [0, 1]; the spectral
        // moments are finite and in [0, 2^width] (here 2^3 = 8); the literal
        // spectral descriptors g = 1 - |λ2| and τ_ε / T_max are finite and in
        // [0, 1].
        for record in &first.records {
            for &value in &record.contraction {
                assert!(value.is_finite() && (0.0..=1.0).contains(&value));
            }
            for &value in &record.spectral {
                assert!(value.is_finite() && (0.0..=8.0).contains(&value));
            }
            for &value in &record.literal_spectral {
                assert!(value.is_finite() && (0.0..=1.0).contains(&value));
            }
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
        let design = super::feature_matrix(&records, |record| {
            super::feature_layout(record, FeatureLayout::Gkt)
        });

        let first = super::fit_ridge(&design, &targets).expect("ridge fit");
        let second = super::fit_ridge(&design, &targets).expect("ridge fit");
        assert_eq!(first.coefficients, second.coefficients);
        // Per-feature scalers cover all 21 GKT features; coefficients carry an
        // additional intercept at index 0.
        assert_eq!(first.means.len(), GKT_FEATURE_COUNT);
        assert_eq!(first.coefficients.len(), GKT_FEATURE_COUNT + 1);

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

    // --- Frozen ancestors must never change under TDI-6.5 ---

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
            (
                "tdi-bench/src/bin/tdi-independent-overlap-ablation-v62.rs",
                "793fc42d0567283c0f6c773e74597a6ff38d7278cf6e14fcdca7d60e33758a37",
            ),
            (
                "docs/TDI-6.2-NONLINEAR-SUFFICIENCY-PREREGISTRATION.md",
                "a5263642ee79fb946bc9a7aa6fea4b57c22945a91b7ffa6f2220c7e4d4a55869",
            ),
        ];

        for (path, want) in expected {
            let got = super::tdi52_sha256_of_repo_file(path);
            assert_eq!(&got, want, "frozen ancestor changed: {path}");
        }
    }

    // --- TDI-6.5 literal spectral descriptors (Sections 6, 7, 8, 13) ---

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
        // tolerance — the Section 8 correctness guarantee for the descriptors.
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
        // Method 3 (Section 8) is a battle-tested reference eigensolver admitted
        // ONLY as a test-only dev-dependency, so the frozen feature path stays
        // dependency-free. No reference crate is vendored in this offline
        // environment, so — exactly as Section 4.4 declares — the cross-check
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
        let family = GeneratorFamily::F0Base;
        let seed = super::frozen_block_order(family)[0].population_base_seed()
            + super::PopulationKind::TrainingWidth3.seed_offset();
        let context = super::AttemptContext::new(family, 3, seed, 0);
        let masks = super::generate_family_masks(context).expect("masks");
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
        let family = GeneratorFamily::F0Base;
        let seed = super::frozen_block_order(family)[0].population_base_seed()
            + super::PopulationKind::TrainingWidth3.seed_offset();
        let context = super::AttemptContext::new(family, 3, seed, 0);
        let masks = super::generate_family_masks(context).expect("masks");
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
        // a battery of real width-3 and width-4 candidate kernels drawn from ALL
        // FOUR generator families the eigensolver converges and
        // `literal_spectral_descriptors` returns finite [g, τ].
        for family in GeneratorFamily::ALL {
            let block = super::frozen_block_order(family)[0];
            let base = block.population_base_seed();
            for (width, seed_base) in [
                (
                    3_u8,
                    base + super::PopulationKind::TrainingWidth3.seed_offset(),
                ),
                (
                    4_u8,
                    base + super::PopulationKind::TrainingWidth4.seed_offset(),
                ),
            ] {
                for offset in 0..24_u64 {
                    let seed = seed_base + offset;
                    let context = super::AttemptContext::new(family, width, seed, 0);
                    let Ok(masks) = super::generate_family_masks(context) else {
                        continue;
                    };
                    let Ok(system) = super::build_system(context, &masks) else {
                        continue;
                    };
                    let [gap, tau] =
                        super::literal_spectral_descriptors(context, &system).expect("descriptors");
                    assert!(
                        gap.is_finite(),
                        "family={}, width={width}, seed={seed}",
                        family.label()
                    );
                    assert!(
                        tau.is_finite(),
                        "family={}, width={width}, seed={seed}",
                        family.label()
                    );
                }
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

    // ---- TDI-6.7 observable offset --------------------------------------

    /// **Preregistration Section 3.3, required proof.**
    ///
    /// B1 claims to read no target label. A code-reading argument is not
    /// sufficient evidence, so this asserts it behaviourally: arbitrarily
    /// perturbing every target value in both domains' records must leave B1's
    /// derived model **bit-identical**, intercepts included.
    /// The mirror: B2 *does* move under the same perturbation. Without this the
    /// test above would be satisfied by an evaluator that never read a label
    /// anywhere, and the `oracle` labelling would be a lie.
    /// B1 shifts the intercept by exactly `Δ / sˢ_h` and touches nothing else
    /// (Section 3.1, step 4).
    /// B0 is the identity, and B2 keeps the SOURCE feature statistics — the
    /// difference from TDI-6.6's A2, which also re-standardized them.
    /// `u₂` is derived from a feature and refuses a fully-recovered observation
    /// rather than propagating a non-finite value (Section 3.1).
    #[test]
    fn observed_deficit_refuses_a_fully_recovered_observation() {
        let mut record = record_with_overlap(0.3, 0.5);
        let expected = -(1.0_f64 - 0.5).log2();
        assert_eq!(
            super::observed_deficit(&record, super::ObservedHorizon::Last).unwrap(),
            expected
        );

        record.early_overlap[1] = 1.0;
        assert!(
            super::observed_deficit(&record, super::ObservedHorizon::Last)
                .unwrap_err()
                .contains("fully-recovered")
        );
    }

    /// Exactly one arm declares that it reads target labels, and exactly one
    /// applies the offset.
    /// **Preregistration Section 6 made executable.** B0 and B1 share the source
    /// scaler and are comparable by relative MSE; any comparison involving B2 is
    /// refused rather than silently computed.
    /// The 12 ordered pairs of Section 13, and their distinct bootstrap streams.
    #[test]
    fn ordered_transfer_pairs_and_seeds_are_complete_and_distinct() {
        let pairs = super::ordered_transfer_pairs();
        assert_eq!(pairs.len(), 12);
        assert!(pairs.iter().all(|(s, t)| s != t));
        assert!(pairs.contains(&super::CONFIRMATORY_TRANSFER_PAIR));

        let mut seeds = pairs
            .iter()
            .map(|(s, t)| super::transfer_pair_bootstrap_seed(*s, *t))
            .collect::<Vec<_>>();
        let total = seeds.len();
        seeds.sort_unstable();
        seeds.dedup();
        assert_eq!(seeds.len(), total);
    }

    // ---- TDI-6.8 : noyau de rang (Sections 6, 8, 10) ----

    /// Spearman on a hand-computed case: perfectly monotone but non-linear data
    /// must give exactly 1, which Pearson on the raw values would not.
    #[test]
    fn rank_correlation_is_one_for_a_monotone_nonlinear_relation() {
        let truth = [1.0_f64, 2.0, 3.0, 4.0, 5.0];
        let prediction = [1.0_f64, 4.0, 9.0, 16.0, 25.0];

        let rho = super::rank_correlation(&truth, &prediction).expect("defined");
        assert!((rho - 1.0).abs() < 1.0e-12, "rho = {rho}");

        let pearson = super::pearson_correlation(&truth, &prediction);
        assert!(pearson < 0.99, "raw Pearson must not be 1: {pearson}");
    }

    /// The degeneracy that produced TDI-5.8's misleading published zero: a
    /// constant argument must report *undefined*, never `0.0`.
    #[test]
    fn rank_correlation_reports_undefined_for_a_constant_argument() {
        let truth = [1.0_f64, 2.0, 3.0, 4.0];
        let clamped = [0.5_f64; 4];

        assert_eq!(super::rank_correlation(&truth, &clamped), None);
        assert_eq!(super::rank_correlation(&clamped, &truth), None);

        // The inherited helper still returns the misleading zero; this pins the
        // difference so the two can never be confused again.
        assert_eq!(super::spearman_correlation(&truth, &clamped), 0.0);
    }

    /// TDI-6.7 §5: an additive constant preserves rank exactly *within a block*.
    /// This is the property that makes every correction arm vacuous under a rank
    /// criterion, and therefore the reason TDI-6.8 compares layouts instead.
    #[test]
    fn rank_correlation_is_invariant_under_an_additive_shift() {
        let truth = [0.3_f64, -1.2, 4.5, 2.2, -0.7, 8.1];
        let prediction = [0.1_f64, -2.0, 3.9, 1.7, -1.4, 7.2];
        let shifted = prediction.map(|value| value - 2.003_845_989);

        let base = super::rank_correlation(&truth, &prediction).expect("defined");
        let moved = super::rank_correlation(&truth, &shifted).expect("defined");

        assert_eq!(base.to_bits(), moved.to_bits());
    }

    /// Kendall τ-b against a case small enough to count by hand: one discordant
    /// pair out of six gives (5 − 1) / 6.
    #[test]
    fn kendall_tau_b_matches_a_hand_counted_case() {
        let left = [1.0_f64, 2.0, 3.0, 4.0];
        let right = [1.0_f64, 2.0, 4.0, 3.0];

        let tau = super::kendall_tau_b(&left, &right).expect("defined");
        assert!((tau - (4.0 / 6.0)).abs() < 1.0e-12, "tau = {tau}");
    }

    #[test]
    fn kendall_tau_b_reports_undefined_when_every_pair_is_tied() {
        let constant = [2.0_f64; 5];
        let varied = [1.0_f64, 2.0, 3.0, 4.0, 5.0];

        assert_eq!(super::kendall_tau_b(&constant, &varied), None);
        assert_eq!(super::kendall_tau_b(&varied, &constant), None);
    }

    #[test]
    fn tied_pairs_counts_group_combinations() {
        // Groups of sizes 3, 2 and 1 give 3 + 1 + 0 tied pairs.
        let values = [1.0_f64, 1.0, 1.0, 2.0, 2.0, 3.0];
        assert_eq!(super::tied_pairs(&values), 4);
        assert_eq!(super::tied_pairs(&[1.0_f64, 2.0, 3.0]), 0);
    }

    #[test]
    fn rank_statistics_flag_direction_disagreement_only_when_signs_differ() {
        let truth = [1.0_f64, 2.0, 3.0, 4.0, 5.0];
        let agreeing = super::RankStatistics::evaluate(&truth, &[1.0, 2.0, 3.0, 4.0, 5.0]);
        assert!(!agreeing.direction_disagreement());

        let opposing = super::RankStatistics::evaluate(&truth, &[5.0, 4.0, 3.0, 2.0, 1.0]);
        assert!(
            !opposing.direction_disagreement(),
            "both negative is agreement"
        );

        // ρ and τ-b almost never disagree on real data, so the true branch is
        // exercised directly rather than left untested.
        let contrived = super::RankStatistics {
            spearman: Some(0.4),
            kendall_tau_b: Some(-0.1),
            tied_truth_pairs: 0,
            tied_prediction_pairs: 0,
        };
        assert!(contrived.direction_disagreement());

        // An undefined member is not a disagreement.
        let undefined = super::RankStatistics {
            spearman: None,
            kendall_tau_b: Some(-0.1),
            tied_truth_pairs: 0,
            tied_prediction_pairs: 0,
        };
        assert!(!undefined.direction_disagreement());
    }

    // ---- Section 10 : frontières exactes du classificateur ----

    fn interval(lower: f64, upper: f64) -> super::ConfidenceInterval {
        super::ConfidenceInterval {
            lower,
            median: (lower + upper) / 2.0,
            upper,
        }
    }

    #[test]
    fn rank_classifier_requires_all_three_beneficial_conditions() {
        let strong = vec![Some(0.05), Some(0.04), Some(0.03)];

        assert_eq!(
            super::classify_rank_increment(strong.clone(), Some(interval(0.01, 0.07)), 0, 4000)
                .classification,
            super::RankClassification::Beneficial
        );

        // One block below zero breaks condition 1 even though the mean clears the margin.
        let mixed = vec![Some(0.09), Some(0.06), Some(-0.01)];
        assert_eq!(
            super::classify_rank_increment(mixed, Some(interval(0.01, 0.07)), 0, 4000)
                .classification,
            super::RankClassification::Indeterminate
        );

        // A non-positive lower bound breaks condition 3.
        assert_eq!(
            super::classify_rank_increment(strong, Some(interval(-0.001, 0.07)), 0, 4000)
                .classification,
            super::RankClassification::Indeterminate
        );
    }

    #[test]
    fn rank_classifier_margin_is_inclusive_at_exactly_two_hundredths() {
        let at_margin = vec![Some(0.02), Some(0.02), Some(0.02)];
        let comparison =
            super::classify_rank_increment(at_margin, Some(interval(0.005, 0.03)), 0, 4000);

        assert!(comparison.aggregate_increment_at_least_margin);
        assert_eq!(
            comparison.classification,
            super::RankClassification::Beneficial
        );

        let below = vec![Some(0.019), Some(0.019), Some(0.019)];
        assert!(
            !super::classify_rank_increment(below, Some(interval(0.005, 0.03)), 0, 4000)
                .aggregate_increment_at_least_margin
        );
    }

    #[test]
    fn rank_classifier_is_symmetric_for_harm() {
        let comparison = super::classify_rank_increment(
            vec![Some(-0.05), Some(-0.04), Some(-0.03)],
            Some(interval(-0.07, -0.01)),
            0,
            4000,
        );

        assert_eq!(
            comparison.classification,
            super::RankClassification::Harmful
        );
        assert!(comparison.all_blocks_favour_baseline);
        assert!(comparison.interval_upper_bound_negative);
    }

    #[test]
    fn rank_classifier_declares_equivalence_only_when_blocks_and_interval_agree() {
        let tiny = vec![Some(0.001), Some(-0.002), Some(0.003)];

        assert_eq!(
            super::classify_rank_increment(tiny.clone(), Some(interval(-0.01, 0.01)), 0, 4000)
                .classification,
            super::RankClassification::Equivalent
        );

        // Blocks inside the margin but a wide interval is *not* equivalence.
        assert_eq!(
            super::classify_rank_increment(tiny, Some(interval(-0.05, 0.05)), 0, 4000)
                .classification,
            super::RankClassification::Indeterminate
        );
    }

    /// Section 8: beyond 1 % undefined replicates the cell is forced to
    /// *Indeterminate* and its interval withheld, whatever the increments say.
    #[test]
    fn rank_classifier_withholds_a_verdict_when_too_many_replicates_are_undefined() {
        let strong = vec![Some(0.05), Some(0.04), Some(0.03)];

        let usable =
            super::classify_rank_increment(strong.clone(), Some(interval(0.01, 0.07)), 40, 4000);
        assert_eq!(usable.classification, super::RankClassification::Beneficial);
        assert!(usable.interval.is_some());

        let unusable = super::classify_rank_increment(strong, Some(interval(0.01, 0.07)), 41, 4000);
        assert_eq!(
            unusable.classification,
            super::RankClassification::Indeterminate
        );
        assert!(unusable.interval.is_none(), "interval must be withheld");
    }

    #[test]
    fn rank_classifier_treats_an_undefined_block_as_undecidable() {
        let comparison = super::classify_rank_increment(
            vec![Some(0.05), None, Some(0.03)],
            Some(interval(0.01, 0.07)),
            0,
            4000,
        );

        assert_eq!(comparison.aggregate_increment, None);
        assert!(!comparison.all_blocks_favour_challenger);
        assert_eq!(
            comparison.classification,
            super::RankClassification::Indeterminate
        );
    }

    // ---- TDI-6.8 : graines fraîches et bootstrap apparié (Sections 7, 8) ----

    /// The defect this test exists to prevent was nearly shipped: the mechanical
    /// derivation renamed every *textual* TDI-6.7 identity but could not touch
    /// the seeds, which carry their identity in hex and decimal literals. Had it
    /// survived, TDI-6.8 would have regenerated TDI-6.7's exact populations and
    /// reused its bootstrap streams — destroying the freshness that Section 1.3
    /// relies on to make a rank criterion admissible at all.
    #[test]
    fn seeds_are_disjoint_from_the_tdi67_scheme() {
        const TDI67_POPULATION_ORIGIN: u64 = 7_400_000_000;
        const TDI67_LAST_RESERVATION: u64 = 8_530_005_038;
        const TDI67_BOOTSTRAP_BASE: u64 = 0x5444_4936_3700_4700;

        assert_eq!(super::AGGREGATE_BOOTSTRAP_SEED_BASE, 0x5444_4936_3800_4800);
        assert_ne!(super::AGGREGATE_BOOTSTRAP_SEED_BASE, TDI67_BOOTSTRAP_BASE);

        for family in super::GeneratorFamily::ALL {
            for block in 0..super::SEED_BLOCK_COUNT {
                let identity = super::SeedBlockId {
                    family,
                    block: block as u8,
                };
                let base = identity.population_base_seed();

                assert!(
                    base > TDI67_LAST_RESERVATION,
                    "{} block {block} base {base} must clear TDI-6.7's last reservation",
                    family.label()
                );
                assert!(base > TDI67_POPULATION_ORIGIN);
            }
        }
    }

    /// Section 7's frozen per-ordered-pair formula, checked on the exact
    /// arithmetic rather than on whatever the constant happens to be.
    #[test]
    fn transfer_pair_bootstrap_seed_follows_the_frozen_formula() {
        for source in super::GeneratorFamily::ALL {
            for target in super::GeneratorFamily::ALL {
                assert_eq!(
                    super::transfer_pair_bootstrap_seed(source, target),
                    0x5444_4936_3800_4800 + 0x10 * (1 + source.index()) + target.index()
                );
            }
        }
    }

    fn rank_block(
        seed_block: super::SeedBlockId,
        targets: Vec<f64>,
        predictions: Vec<f64>,
    ) -> super::ArmBlockEvaluation {
        let records_len = targets.len();

        super::ArmBlockEvaluation {
            seed_block,
            records_len,
            standardized_targets: targets,
            overlap_targets: vec![0.0; records_len],
            evaluation: super::PredictorEvaluation {
                predictions: super::Tdi52PredictionSet {
                    standardized: predictions,
                    reconstructed_overlap: vec![0.0; records_len],
                },
            },
        }
    }

    /// The property that makes the shared draw *correct*, not merely fast: both
    /// layouts see identical indices, so identical predictions must give an
    /// increment of exactly zero in every replicate — bound included.
    #[test]
    fn shared_resample_gives_an_exactly_zero_increment_for_identical_layouts() {
        let blocks = super::frozen_block_order(super::GeneratorFamily::F0Base);
        let targets = (0..64).map(f64::from).collect::<Vec<_>>();
        let predictions = targets
            .iter()
            .map(|value| value * 0.5 + 1.0)
            .collect::<Vec<_>>();

        let baseline = blocks
            .iter()
            .map(|&id| rank_block(id, targets.clone(), predictions.clone()))
            .collect::<Vec<_>>();
        let challenger = baseline.clone();

        let outcome = super::rank_bootstrap(
            &[
                (super::FeatureLayout::Gk, baseline.as_slice()),
                (super::FeatureLayout::Gkt, challenger.as_slice()),
            ],
            &[(super::FeatureLayout::Gkt, super::FeatureLayout::Gk)],
            0x5444_4936_3800_4811,
        )
        .expect("well-formed");

        let (interval, undefined) =
            outcome.increment(super::FeatureLayout::Gkt, super::FeatureLayout::Gk);
        let total = outcome.replicates;
        let interval = interval.expect("defined");
        assert_eq!(undefined, 0);
        assert_eq!(total, super::BOOTSTRAP_REPLICATES);
        assert_eq!(interval.lower, 0.0);
        assert_eq!(interval.median, 0.0);
        assert_eq!(interval.upper, 0.0);
    }

    /// A constant prediction makes every replicate undefined, and Section 8's
    /// 1 % guard must then withhold the verdict rather than report a zero.
    #[test]
    fn a_degenerate_layout_makes_every_replicate_undefined() {
        let blocks = super::frozen_block_order(super::GeneratorFamily::F0Base);
        let targets = (0..64).map(f64::from).collect::<Vec<_>>();

        let baseline = blocks
            .iter()
            .map(|&id| rank_block(id, targets.clone(), targets.clone()))
            .collect::<Vec<_>>();
        let challenger = blocks
            .iter()
            .map(|&id| rank_block(id, targets.clone(), vec![0.5; targets.len()]))
            .collect::<Vec<_>>();

        let outcome = super::rank_bootstrap(
            &[
                (super::FeatureLayout::Gk, baseline.as_slice()),
                (super::FeatureLayout::Gkt, challenger.as_slice()),
            ],
            &[(super::FeatureLayout::Gkt, super::FeatureLayout::Gk)],
            0x5444_4936_3800_4812,
        )
        .expect("well-formed");

        let (interval, undefined) =
            outcome.increment(super::FeatureLayout::Gkt, super::FeatureLayout::Gk);
        let total = outcome.replicates;

        assert!(interval.is_none());
        assert_eq!(undefined, super::BOOTSTRAP_REPLICATES);
        assert_eq!(total, super::BOOTSTRAP_REPLICATES);

        let comparison =
            super::classify_rank_increment(vec![None, None, None], interval, undefined, total);
        assert_eq!(
            comparison.classification,
            super::RankClassification::Indeterminate
        );
    }

    /// The shared truth ranks are only sound if both layouts really carry the
    /// same standardized truth; the guard must refuse rather than silently rank
    /// one layout's predictions against the other's truth.
    #[test]
    fn rank_bootstrap_refuses_layouts_that_disagree_on_the_truth() {
        let blocks = super::frozen_block_order(super::GeneratorFamily::F0Base);
        let targets = (0..32).map(f64::from).collect::<Vec<_>>();
        let shifted = targets.iter().map(|value| value + 1.0).collect::<Vec<_>>();

        let baseline = blocks
            .iter()
            .map(|&id| rank_block(id, targets.clone(), targets.clone()))
            .collect::<Vec<_>>();
        let challenger = blocks
            .iter()
            .map(|&id| rank_block(id, shifted.clone(), shifted.clone()))
            .collect::<Vec<_>>();

        assert!(
            super::rank_bootstrap(
                &[
                    (super::FeatureLayout::Gk, baseline.as_slice()),
                    (super::FeatureLayout::Gkt, challenger.as_slice()),
                ],
                &[(super::FeatureLayout::Gkt, super::FeatureLayout::Gk)],
                1,
            )
            .is_err()
        );
    }

    // ---- TDI-6.8 : un seul bras, quatre dispositions (Sections 3, 11-13) ----

    /// Section 3 admits exactly one arm, and Section 12 states flatly that no
    /// target label is read anywhere in TDI-6.8, in any arm, for any criterion.
    /// TDI-6.7's B1 (observable offset) and B2 (oracle target scaler) are gone;
    /// B2 in particular fitted the target scaler, which reads target `U_h`.
    #[test]
    fn the_experiment_has_exactly_one_arm_and_no_oracle() {
        assert_eq!(super::TransferArm::ALL.len(), 1);
        assert_eq!(
            super::TransferArm::ALL[0],
            super::TransferArm::SourceStandardized
        );
        assert_eq!(super::TransferArm::ALL[0].label(), "plain-transfer");
    }

    /// Section 3's ladder, in nesting order with the frozen feature counts. The
    /// inherited evaluator carried only [GK, GKT]; CK and SK must be present or
    /// the ladder the preregistration asks to be visible simply is not reported.
    #[test]
    fn transfer_layouts_are_the_four_rung_ladder() {
        assert_eq!(
            super::TRANSFER_LAYOUTS,
            [
                super::FeatureLayout::Ck,
                super::FeatureLayout::Sk,
                super::FeatureLayout::Gk,
                super::FeatureLayout::Gkt,
            ]
        );

        let counts = super::TRANSFER_LAYOUTS.map(super::FeatureLayout::feature_count);
        assert_eq!(counts, [15, 17, 19, 21]);

        // Strict nesting CK ⊂ SK ⊂ GK ⊂ GKT.
        assert!(counts.windows(2).all(|pair| pair[0] < pair[1]));
    }

    /// The three rungs of Section 10, challenger first. Only the first carries a
    /// criterion; the preregistration reports the other two.
    #[test]
    fn ladder_comparisons_are_the_three_adjacent_rungs() {
        assert_eq!(
            super::LADDER_COMPARISONS,
            [
                (super::FeatureLayout::Gkt, super::FeatureLayout::Gk),
                (super::FeatureLayout::Gk, super::FeatureLayout::Sk),
                (super::FeatureLayout::Sk, super::FeatureLayout::Ck),
            ]
        );
    }

    /// Section 6 forbids filling an undefined block: a mean over two of three is
    /// a different statistic, and calling it `ρ̄` would silently change the
    /// estimand the criterion is written about.
    #[test]
    fn rho_bar_is_undefined_when_any_block_is() {
        assert_eq!(
            super::mean_of_defined(&[Some(0.3), Some(0.6), Some(0.9)]),
            Some(0.6)
        );
        assert_eq!(super::mean_of_defined(&[Some(0.3), None, Some(0.9)]), None);
        assert_eq!(super::mean_of_defined(&[None, None, None]), None);
    }

    fn beneficial_comparison() -> super::RankComparison {
        super::classify_rank_increment(
            vec![Some(0.05), Some(0.05), Some(0.05)],
            Some(super::ConfidenceInterval {
                lower: 0.01,
                median: 0.05,
                upper: 0.09,
            }),
            0,
            super::BOOTSTRAP_REPLICATES,
        )
    }

    fn pair_with(
        mean_rho: Option<f64>,
        comparison: super::RankComparison,
    ) -> super::TransferPairReport {
        super::TransferPairReport {
            source: super::GeneratorFamily::F0Base,
            target: super::GeneratorFamily::F1Sparse,
            cells: Vec::new(),
            observable_shift: 0.0,
            observable_shift_u1: 0.0,
            rank_cells: vec![super::RankCell {
                layout: super::FeatureLayout::Gkt,
                horizon: 3,
                block_rho: [mean_rho, mean_rho, mean_rho],
                mean_rho,
                interval: None,
                undefined_replicates: 0,
                rank_transfers: false,
                within_rho: None,
                retention: None,
            }],
            ladder: vec![super::LadderComparison {
                challenger: super::FeatureLayout::Gkt,
                baseline: super::FeatureLayout::Gk,
                horizon: 3,
                comparison,
            }],
        }
    }

    /// Section 13's reading rule. A *Beneficial* increment whose challenger
    /// orders nothing is better-ordered noise, not transfer, and the
    /// qualification must ride on the same line as the classification — the trap
    /// TDI-5.8B and TDI-6.6D both had to disarm after the fact.
    ///
    #[test]
    fn a_beneficial_increment_over_unordered_noise_is_qualified_inline() {
        let comparison = beneficial_comparison();
        assert_eq!(
            comparison.classification,
            super::RankClassification::Beneficial
        );

        let ordering = pair_with(Some(0.4), comparison.clone());
        assert_eq!(super::reading_rule_note(&ordering, &ordering.ladder[0]), "");

        for unordered in [Some(0.0), Some(-0.2), None] {
            let pair = pair_with(unordered, comparison.clone());
            assert!(
                super::reading_rule_note(&pair, &pair.ladder[0]).contains("BRUIT MIEUX ORDONNÉ"),
                "ρ̄(GKT) = {unordered:?} must be qualified as better-ordered noise"
            );
        }
    }

    /// Section 11 is a conjunction, and the two halves must both bite: a layout
    /// positive in every block but whose interval reaches zero does not transfer,
    /// and neither does one with a positive bound but a negative block.
    #[test]
    fn rank_transfers_needs_both_conjuncts() {
        let blocks = super::frozen_block_order(super::GeneratorFamily::F0Base);
        let targets = (0..48).map(f64::from).collect::<Vec<_>>();

        // Perfectly ordered: every block ρ = 1, so the bound is positive too.
        let ordered = blocks
            .iter()
            .map(|&id| rank_block(id, targets.clone(), targets.clone()))
            .collect::<Vec<_>>();
        // Reversed: every block ρ = −1.
        let reversed = blocks
            .iter()
            .map(|&id| {
                let mut backwards = targets.clone();
                backwards.reverse();
                rank_block(id, targets.clone(), backwards)
            })
            .collect::<Vec<_>>();

        let outcome = super::rank_bootstrap(
            &[
                (super::FeatureLayout::Gk, reversed.as_slice()),
                (super::FeatureLayout::Gkt, ordered.as_slice()),
            ],
            &[(super::FeatureLayout::Gkt, super::FeatureLayout::Gk)],
            0x5444_4936_3800_4813,
        )
        .expect("well-formed");

        let (positive, _) = outcome.layout(super::FeatureLayout::Gkt);
        let (negative, _) = outcome.layout(super::FeatureLayout::Gk);

        assert!(positive.expect("defined").lower > 0.0);
        assert!(negative.expect("defined").upper < 0.0);
    }

    /// The shared draw must serve every layout from the same indices, not just
    /// the two named in a comparison: with four identical layouts every pairwise
    /// increment is exactly zero, bound included.
    #[test]
    fn one_draw_serves_all_four_layouts() {
        let blocks = super::frozen_block_order(super::GeneratorFamily::F0Base);
        let targets = (0..48).map(f64::from).collect::<Vec<_>>();
        let predictions = targets.iter().map(|value| value * 2.0).collect::<Vec<_>>();

        let identical = blocks
            .iter()
            .map(|&id| rank_block(id, targets.clone(), predictions.clone()))
            .collect::<Vec<_>>();

        let inputs = super::TRANSFER_LAYOUTS
            .iter()
            .map(|layout| (*layout, identical.as_slice()))
            .collect::<Vec<_>>();

        let outcome =
            super::rank_bootstrap(&inputs, &super::LADDER_COMPARISONS, 0x5444_4936_3800_4814)
                .expect("well-formed");

        assert_eq!(outcome.per_layout.len(), 4);

        for (challenger, baseline) in super::LADDER_COMPARISONS {
            let (interval, undefined) = outcome.increment(challenger, baseline);
            let interval = interval.expect("defined");

            assert_eq!(undefined, 0);
            assert_eq!(interval.lower, 0.0);
            assert_eq!(interval.upper, 0.0);
        }
    }
}
