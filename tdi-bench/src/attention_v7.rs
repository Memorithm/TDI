//! Deterministic TDI-7 task and intervention mechanics shared by bounded evaluators.
//!
//! These functions preserve the TDI-7.1 software-oracle semantics. They expose
//! no final-holdout seed selection and contain no final-run authorization token.

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum TaskKind {
    AssociativeRecall,
    Copy,
}

impl TaskKind {
    #[must_use]
    pub const fn id(self) -> &'static str {
        match self {
            Self::AssociativeRecall => "associative_recall",
            Self::Copy => "copy",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TaskExample {
    seed: u64,
    kind: TaskKind,
    input: Vec<u16>,
    target: Vec<u16>,
    retrieval_distance: usize,
    distractor_count: usize,
}

impl TaskExample {
    #[must_use]
    pub const fn seed(&self) -> u64 {
        self.seed
    }

    #[must_use]
    pub const fn kind(&self) -> TaskKind {
        self.kind
    }

    #[must_use]
    pub fn input(&self) -> &[u16] {
        &self.input
    }

    #[must_use]
    pub fn target(&self) -> &[u16] {
        &self.target
    }

    #[must_use]
    pub const fn retrieval_distance(&self) -> usize {
        self.retrieval_distance
    }

    #[must_use]
    pub const fn distractor_count(&self) -> usize {
        self.distractor_count
    }
}

#[derive(Clone, Copy, Debug)]
struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    const fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    fn bounded(&mut self, upper_exclusive: u16) -> u16 {
        assert!(upper_exclusive > 0);
        (self.next_u64() % u64::from(upper_exclusive)) as u16
    }
}

fn unique_tokens(rng: &mut SplitMix64, count: usize, base: u16, width: u16) -> Vec<u16> {
    assert!(count <= usize::from(width));
    let mut out = Vec::with_capacity(count);
    while out.len() < count {
        let token = base + rng.bounded(width);
        if !out.contains(&token) {
            out.push(token);
        }
    }
    out
}

#[must_use]
pub fn generate_associative_recall(seed: u64) -> TaskExample {
    const PAIRS: usize = 4;
    const QUERY_MARKER: u16 = 250;

    let mut rng = SplitMix64::new(seed ^ 0x5444_4937_4152_0001);
    let keys = unique_tokens(&mut rng, PAIRS, 1, 64);
    let values = unique_tokens(&mut rng, PAIRS, 128, 64);
    let query_pair = usize::from(rng.bounded(PAIRS as u16));

    let mut input = Vec::with_capacity(PAIRS * 2 + 2);
    for index in 0..PAIRS {
        input.push(keys[index]);
        input.push(values[index]);
    }
    input.push(QUERY_MARKER);
    input.push(keys[query_pair]);

    let query_position = input.len() - 1;
    let value_position = query_pair * 2 + 1;

    TaskExample {
        seed,
        kind: TaskKind::AssociativeRecall,
        input,
        target: vec![values[query_pair]],
        retrieval_distance: query_position - value_position,
        distractor_count: PAIRS - 1,
    }
}

#[must_use]
pub fn generate_copy(seed: u64) -> TaskExample {
    const COPY_MARKER: u16 = 251;
    const SOURCE_LEN: usize = 4;

    let mut rng = SplitMix64::new(seed ^ 0x5444_4937_434F_0001);
    let source = unique_tokens(&mut rng, SOURCE_LEN, 16, 96);
    let distractor_count = usize::from(rng.bounded(4)) + 1;
    let distractors = unique_tokens(&mut rng, distractor_count, 160, 64);

    let mut input = Vec::with_capacity(SOURCE_LEN + distractor_count + 1);
    input.extend_from_slice(&source);
    input.extend_from_slice(&distractors);
    input.push(COPY_MARKER);

    TaskExample {
        seed,
        kind: TaskKind::Copy,
        input,
        target: source,
        retrieval_distance: distractor_count + 1,
        distractor_count,
    }
}

#[must_use]
pub fn generate_task(kind: TaskKind, seed: u64) -> TaskExample {
    match kind {
        TaskKind::AssociativeRecall => generate_associative_recall(seed),
        TaskKind::Copy => generate_copy(seed),
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum InterventionSite {
    EarlyToken,
    LateToken,
}

impl InterventionSite {
    #[must_use]
    pub const fn id(self) -> &'static str {
        match self {
            Self::EarlyToken => "early-token",
            Self::LateToken => "late-token",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct MechanisticState {
    tokens: Vec<u16>,
    target: Vec<u16>,
    activations: Vec<f64>,
}

impl MechanisticState {
    #[must_use]
    pub fn new(tokens: Vec<u16>, target: Vec<u16>) -> Self {
        let activations = tokens
            .iter()
            .map(|token| f64::from(*token) / 256.0)
            .collect();
        Self {
            tokens,
            target,
            activations,
        }
    }

    #[must_use]
    pub fn from_task(task: &TaskExample) -> Self {
        Self::new(task.input.clone(), task.target.clone())
    }

    #[must_use]
    pub fn tokens(&self) -> &[u16] {
        &self.tokens
    }

    #[must_use]
    pub fn target(&self) -> &[u16] {
        &self.target
    }

    #[must_use]
    pub fn activations(&self) -> &[f64] {
        &self.activations
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SingleSiteIntervention {
    site: InterventionSite,
    amplitude: f64,
}

impl SingleSiteIntervention {
    #[must_use]
    pub const fn new(site: InterventionSite, amplitude: f64) -> Self {
        Self { site, amplitude }
    }

    #[must_use]
    pub const fn site(self) -> InterventionSite {
        self.site
    }

    #[must_use]
    pub const fn amplitude(self) -> f64 {
        self.amplitude
    }

    #[must_use]
    pub fn index(self, len: usize) -> Option<usize> {
        match self.site {
            InterventionSite::EarlyToken => (len >= 2).then_some(1),
            InterventionSite::LateToken => (len >= 2).then_some(len - 2),
        }
    }

    pub fn apply(self, reference: &MechanisticState) -> Result<MechanisticState, InterventionError> {
        if !self.amplitude.is_finite() {
            return Err(InterventionError::NonFiniteAmplitude);
        }
        let index = self
            .index(reference.activations.len())
            .ok_or(InterventionError::StateTooShort)?;
        let mut perturbed = reference.clone();
        perturbed.activations[index] += self.amplitude;
        if !perturbed.activations[index].is_finite() {
            return Err(InterventionError::NonFiniteResult);
        }
        Ok(perturbed)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InterventionError {
    StateTooShort,
    NonFiniteAmplitude,
    NonFiniteResult,
    DuplicateSite,
}

pub fn apply_joint(
    reference: &MechanisticState,
    left: SingleSiteIntervention,
    right: SingleSiteIntervention,
) -> Result<MechanisticState, InterventionError> {
    if left.site == right.site {
        return Err(InterventionError::DuplicateSite);
    }
    let once = left.apply(reference)?;
    right.apply(&once)
}

#[must_use]
pub fn advance(state: &MechanisticState) -> MechanisticState {
    let len = state.activations.len();
    let mut next = state.clone();
    for index in 0..len {
        let left = if index == 0 {
            state.activations[index]
        } else {
            state.activations[index - 1]
        };
        let center = state.activations[index];
        let right = if index + 1 == len {
            state.activations[index]
        } else {
            state.activations[index + 1]
        };
        next.activations[index] = 0.25 * left + 0.5 * center + 0.25 * right;
    }
    next
}

#[must_use]
pub fn linf_distance(left: &MechanisticState, right: &MechanisticState) -> f64 {
    left.activations
        .iter()
        .zip(&right.activations)
        .map(|(a, b)| (a - b).abs())
        .fold(0.0, f64::max)
}

#[must_use]
pub fn reciprocal_linf_recovery(left: &MechanisticState, right: &MechanisticState) -> f64 {
    1.0 / (1.0 + linf_distance(left, right))
}

pub fn recovery_trajectory(
    reference: &MechanisticState,
    perturbed: &MechanisticState,
    horizon: usize,
) -> Vec<f64> {
    let mut reference_state = reference.clone();
    let mut perturbed_state = perturbed.clone();
    let mut trajectory = Vec::with_capacity(horizon);
    for _ in 0..horizon {
        reference_state = advance(&reference_state);
        perturbed_state = advance(&perturbed_state);
        trajectory.push(reciprocal_linf_recovery(
            &reference_state,
            &perturbed_state,
        ));
    }
    trajectory
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn task_generators_preserve_tdi71_known_fixtures() {
        let ar = generate_associative_recall(7_100_000_000);
        assert_eq!(ar.kind(), TaskKind::AssociativeRecall);
        assert_eq!(ar.input().len(), 10);
        assert_eq!(ar.target().len(), 1);
        assert_eq!(ar.distractor_count(), 3);

        let copy = generate_copy(7_100_000_001);
        assert_eq!(copy.kind(), TaskKind::Copy);
        assert_eq!(copy.target().len(), 4);
        assert_eq!(&copy.input()[..4], copy.target());
    }

    #[test]
    fn same_seed_is_bit_deterministic() {
        for kind in [TaskKind::AssociativeRecall, TaskKind::Copy] {
            assert_eq!(generate_task(kind, 42), generate_task(kind, 42));
        }
    }

    #[test]
    fn intervention_preserves_tokens_and_target() {
        let task = generate_copy(123);
        let reference = MechanisticState::from_task(&task);
        for site in [InterventionSite::EarlyToken, InterventionSite::LateToken] {
            let perturbed = SingleSiteIntervention::new(site, 0.25)
                .apply(&reference)
                .expect("fixture intervention succeeds");
            assert_eq!(perturbed.tokens(), reference.tokens());
            assert_eq!(perturbed.target(), reference.target());
        }
    }

    #[test]
    fn joint_intervention_changes_both_distinct_sites() {
        let task = generate_associative_recall(456);
        let reference = MechanisticState::from_task(&task);
        let early = SingleSiteIntervention::new(InterventionSite::EarlyToken, 0.25);
        let late = SingleSiteIntervention::new(InterventionSite::LateToken, 0.25);
        let joint = apply_joint(&reference, early, late).expect("distinct sites");
        let changed = reference
            .activations()
            .iter()
            .zip(joint.activations())
            .filter(|(left, right)| left != right)
            .count();
        assert_eq!(changed, 2);
        assert_eq!(joint.tokens(), reference.tokens());
        assert_eq!(joint.target(), reference.target());
    }

    #[test]
    fn recovery_is_bounded_and_deterministic() {
        let task = generate_associative_recall(789);
        let reference = MechanisticState::from_task(&task);
        let perturbed = SingleSiteIntervention::new(InterventionSite::EarlyToken, 0.25)
            .apply(&reference)
            .unwrap();
        let left = recovery_trajectory(&reference, &perturbed, 4);
        let right = recovery_trajectory(&reference, &perturbed, 4);
        assert_eq!(left, right);
        assert_eq!(left.len(), 4);
        assert!(left.iter().all(|value| (0.0..=1.0).contains(value)));
    }

    #[test]
    fn source_has_no_final_holdout_authorization_secret() {
        let source = include_str!("attention_v7.rs");
        let confirmation = ["I_ACCEPT_THE_TDI7_", "HOLDOUT_FREEZE"].concat();
        let environment = ["TDI7_CONFIRM_FINAL_", "HOLDOUT"].concat();
        assert!(!source.contains(&confirmation));
        assert!(!source.contains(&environment));
    }
}
