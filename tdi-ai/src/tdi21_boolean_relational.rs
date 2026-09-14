//! TDI-21 bootstrap primitives for Boolean relational architecture research.
//!
//! Candidate arms in this module are intentionally not attention mechanisms:
//! they do not expose Q/K/V projections, pairwise similarity scoring, softmax,
//! or dense token-token score matrices.

/// Experimental architecture arms used by the TDI-21 ladder.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArchitectureArm {
    /// Conventional attention reference; baseline only.
    B0AttentionReference,
    /// Binary/Hamming attention reference; baseline only.
    B1BinaryAttentionReference,
    /// Boolean predicates with direct-address bounded memory.
    B2BooleanDirect,
    /// Boolean routing with a future bounded associative-memory extension.
    B3BooleanAssociative,
    /// Boolean routing with a future explicit GF(2)/ANF extension.
    B4BooleanAnf,
    /// Complete Boolean relational candidate family.
    B5BooleanRelational,
}

impl ArchitectureArm {
    /// Whether the arm belongs to the non-attention candidate family.
    #[must_use]
    pub const fn is_boolean_candidate(self) -> bool {
        matches!(
            self,
            Self::B2BooleanDirect
                | Self::B3BooleanAssociative
                | Self::B4BooleanAnf
                | Self::B5BooleanRelational
        )
    }
}

/// Declaration used to fail closed when a candidate attempts to enable a
/// mechanism prohibited by the TDI-21 research contract.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ForbiddenMechanisms {
    pub qkv_projection: bool,
    pub pairwise_dot_product: bool,
    pub cosine_similarity: bool,
    pub softmax: bool,
    pub hamming_attention_score: bool,
    pub dense_pairwise_matrix: bool,
    pub attention_fallback: bool,
    pub linear_history_scan_addressing: bool,
}

impl ForbiddenMechanisms {
    #[must_use]
    pub const fn any(self) -> bool {
        self.qkv_projection
            || self.pairwise_dot_product
            || self.cosine_similarity
            || self.softmax
            || self.hamming_attention_score
            || self.dense_pairwise_matrix
            || self.attention_fallback
            || self.linear_history_scan_addressing
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CandidateValidationError {
    ReferenceArmIsNotCandidate,
    ForbiddenAttentionMechanism,
    ZeroStateWidth,
    StateWidthExceedsBootstrapLimit,
    ZeroMemorySlots,
}

/// Stage-0 candidate configuration. The 64-bit state ceiling is deliberate:
/// bootstrap semantics stay small enough for exhaustive deterministic tests.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CandidateConfig {
    pub arm: ArchitectureArm,
    pub state_width_bits: u8,
    pub memory_slots: usize,
    pub forbidden: ForbiddenMechanisms,
}

impl CandidateConfig {
    pub fn validate(self) -> Result<(), CandidateValidationError> {
        if !self.arm.is_boolean_candidate() {
            return Err(CandidateValidationError::ReferenceArmIsNotCandidate);
        }
        if self.forbidden.any() {
            return Err(CandidateValidationError::ForbiddenAttentionMechanism);
        }
        if self.state_width_bits == 0 {
            return Err(CandidateValidationError::ZeroStateWidth);
        }
        if self.state_width_bits > 64 {
            return Err(CandidateValidationError::StateWidthExceedsBootstrapLimit);
        }
        if self.memory_slots == 0 {
            return Err(CandidateValidationError::ZeroMemorySlots);
        }
        Ok(())
    }
}

/// Fixed-width Stage-0 Boolean state.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BooleanState(u64);

impl BooleanState {
    #[must_use]
    pub const fn from_bits(bits: u64) -> Self {
        Self(bits)
    }

    #[must_use]
    pub const fn bits(self) -> u64 {
        self.0
    }

    #[must_use]
    pub const fn test(self, bit: u8) -> bool {
        bit < 64 && ((self.0 >> bit) & 1) != 0
    }

    #[must_use]
    pub const fn bitand(self, rhs: Self) -> Self {
        Self(self.0 & rhs.0)
    }

    #[must_use]
    pub const fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }

    #[must_use]
    pub const fn bitxor(self, rhs: Self) -> Self {
        Self(self.0 ^ rhs.0)
    }

    #[must_use]
    pub const fn bitnot(self, width_bits: u8) -> Self {
        let mask = if width_bits == 0 {
            0
        } else if width_bits >= 64 {
            u64::MAX
        } else {
            (1u64 << width_bits) - 1
        };
        Self((!self.0) & mask)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Literal {
    Bit(u8),
    NotBit(u8),
}

impl Literal {
    #[must_use]
    const fn evaluate(self, state: BooleanState) -> bool {
        match self {
            Self::Bit(bit) => state.test(bit),
            Self::NotBit(bit) => !state.test(bit),
        }
    }
}

/// A deterministic conjunction of Boolean literals.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Clause<const N: usize> {
    pub literals: [Literal; N],
}

impl<const N: usize> Clause<N> {
    pub fn evaluate(&self, state: BooleanState, counters: &mut ResourceCounters) -> bool {
        for literal in self.literals {
            counters.boolean_primitive_evals += 1;
            if !literal.evaluate(state) {
                return false;
            }
        }
        true
    }
}

/// Semantic work counters. They intentionally keep pairwise comparisons
/// separate rather than converting unlike operations to a fabricated FLOP
/// equivalent.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ResourceCounters {
    pub boolean_primitive_evals: u64,
    pub route_activations: u64,
    pub memory_reads: u64,
    pub memory_writes: u64,
    pub memory_misses: u64,
    pub memory_collisions: u64,
    pub memory_replacements: u64,
    pub pairwise_comparisons: u64,
}

impl ResourceCounters {
    /// B2-B5 runs fail closed if a pairwise token comparison is recorded.
    #[must_use]
    pub const fn candidate_is_pairwise_free(self) -> bool {
        self.pairwise_comparisons == 0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MemoryRead {
    Hit(BooleanState),
    Miss,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Slot {
    tag: u64,
    value: BooleanState,
}

/// Bounded deterministic direct-address memory.
///
/// Address selection is independent of sequence length. A full tag separates a
/// genuine hit from an address collision; the implementation never scans prior
/// tokens to find a match.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DirectAddressMemory<const SLOTS: usize> {
    slots: [Option<Slot>; SLOTS],
}

impl<const SLOTS: usize> Default for DirectAddressMemory<SLOTS> {
    fn default() -> Self {
        Self {
            slots: [None; SLOTS],
        }
    }
}

impl<const SLOTS: usize> DirectAddressMemory<SLOTS> {
    #[must_use]
    const fn index(tag: u64) -> Option<usize> {
        if SLOTS == 0 {
            None
        } else {
            Some((tag as usize) % SLOTS)
        }
    }

    pub fn write(
        &mut self,
        tag: u64,
        value: BooleanState,
        counters: &mut ResourceCounters,
    ) {
        counters.memory_writes += 1;
        let Some(index) = Self::index(tag) else {
            counters.memory_misses += 1;
            return;
        };

        if let Some(previous) = self.slots[index] {
            if previous.tag != tag {
                counters.memory_collisions += 1;
                counters.memory_replacements += 1;
            }
        }
        self.slots[index] = Some(Slot { tag, value });
    }

    pub fn read(&self, tag: u64, counters: &mut ResourceCounters) -> MemoryRead {
        counters.memory_reads += 1;
        let Some(index) = Self::index(tag) else {
            counters.memory_misses += 1;
            return MemoryRead::Miss;
        };

        match self.slots[index] {
            Some(slot) if slot.tag == tag => MemoryRead::Hit(slot.value),
            _ => {
                counters.memory_misses += 1;
                MemoryRead::Miss
            }
        }
    }

    /// Exact semantic payload accounting for Stage 0: one occupancy bit, one
    /// 64-bit tag, and one 64-bit Boolean state per slot. This is deliberately
    /// not a claim about Rust struct padding or physical allocator overhead.
    #[must_use]
    pub const fn peak_semantic_memory_bits() -> usize {
        SLOTS * 129
    }
}

/// Direct Boolean route mixer. XOR and rotations are deterministic bit
/// operations; this maps the current state to an address tag without comparing
/// that state against stored tokens.
#[must_use]
pub const fn route_tag(state: BooleanState, salt: u64) -> u64 {
    let mut value = state.bits() ^ salt;
    value ^= value.rotate_left(13);
    value ^= value.rotate_right(7);
    value ^ value.rotate_left(17)
}

pub fn activate_route<const N: usize>(
    state: BooleanState,
    clause: &Clause<N>,
    counters: &mut ResourceCounters,
) -> bool {
    let active = clause.evaluate(state, counters);
    if active {
        counters.route_activations += 1;
    }
    active
}

/// One algebraic-normal-form monomial over GF(2). A set bit selects a Boolean
/// variable participating in the product term.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AnfTerm {
    pub variables: u64,
}

impl AnfTerm {
    #[must_use]
    pub const fn evaluate(self, assignment: u64) -> bool {
        (assignment & self.variables) == self.variables
    }
}

/// Evaluate a Zhegalkin/algebraic-normal-form expression as XOR of Boolean
/// product terms. No numeric attention score is produced.
#[must_use]
pub fn evaluate_anf(constant: bool, terms: &[AnfTerm], assignment: u64) -> bool {
    let mut value = constant;
    for term in terms {
        value ^= term.evaluate(assignment);
    }
    value
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RunManifest {
    pub arm: ArchitectureArm,
    pub sequence_len: usize,
    pub state_width_bits: u8,
    pub memory_slots: usize,
    pub development_seed: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EvidenceRecord {
    pub manifest: RunManifest,
    pub correct: bool,
    pub resources: ResourceCounters,
}

impl EvidenceRecord {
    #[must_use]
    pub const fn candidate_structurally_valid(self) -> bool {
        self.manifest.arm.is_boolean_candidate() && self.resources.candidate_is_pairwise_free()
    }
}

/// Deterministic Stage-0 delayed recall fixture.
#[must_use]
pub fn delayed_bit_recall<const SLOTS: usize>(key: u64, value: bool) -> EvidenceRecord {
    let mut memory = DirectAddressMemory::<SLOTS>::default();
    let mut resources = ResourceCounters::default();
    let encoded = BooleanState::from_bits(if value { 1 } else { 0 });
    let tag = route_tag(BooleanState::from_bits(key), 0x5444_4932_3100_0001);

    memory.write(tag, encoded, &mut resources);
    let correct = matches!(
        memory.read(tag, &mut resources),
        MemoryRead::Hit(state) if state == encoded
    );

    EvidenceRecord {
        manifest: RunManifest {
            arm: ArchitectureArm::B2BooleanDirect,
            sequence_len: 2,
            state_width_bits: 64,
            memory_slots: SLOTS,
            development_seed: 0,
        },
        correct,
        resources,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn attention_reference_is_not_a_candidate() {
        let config = CandidateConfig {
            arm: ArchitectureArm::B1BinaryAttentionReference,
            state_width_bits: 16,
            memory_slots: 8,
            forbidden: ForbiddenMechanisms::default(),
        };
        assert_eq!(
            config.validate(),
            Err(CandidateValidationError::ReferenceArmIsNotCandidate)
        );
    }

    #[test]
    fn candidate_fails_closed_when_softmax_is_declared() {
        let config = CandidateConfig {
            arm: ArchitectureArm::B2BooleanDirect,
            state_width_bits: 16,
            memory_slots: 8,
            forbidden: ForbiddenMechanisms {
                softmax: true,
                ..ForbiddenMechanisms::default()
            },
        };
        assert_eq!(
            config.validate(),
            Err(CandidateValidationError::ForbiddenAttentionMechanism)
        );
    }

    #[test]
    fn direct_address_recall_does_not_record_pairwise_comparisons() {
        let evidence = delayed_bit_recall::<8>(0x35, true);
        assert!(evidence.correct);
        assert!(evidence.candidate_structurally_valid());
        assert_eq!(evidence.resources.memory_writes, 1);
        assert_eq!(evidence.resources.memory_reads, 1);
        assert_eq!(evidence.resources.pairwise_comparisons, 0);
    }

    #[test]
    fn collision_and_replacement_are_explicit() {
        let mut memory = DirectAddressMemory::<1>::default();
        let mut counters = ResourceCounters::default();
        let first = BooleanState::from_bits(1);
        let second = BooleanState::from_bits(2);

        memory.write(7, first, &mut counters);
        memory.write(8, second, &mut counters);

        assert_eq!(counters.memory_collisions, 1);
        assert_eq!(counters.memory_replacements, 1);
        assert_eq!(memory.read(7, &mut counters), MemoryRead::Miss);
        assert_eq!(memory.read(8, &mut counters), MemoryRead::Hit(second));
    }

    #[test]
    fn boolean_clause_rejects_a_distractor_without_similarity_scoring() {
        let clause = Clause {
            literals: [Literal::Bit(0), Literal::NotBit(1), Literal::Bit(3)],
        };
        let target = BooleanState::from_bits(0b1001);
        let distractor = BooleanState::from_bits(0b1011);
        let mut counters = ResourceCounters::default();

        assert!(activate_route(target, &clause, &mut counters));
        assert!(!activate_route(distractor, &clause, &mut counters));
        assert_eq!(counters.route_activations, 1);
        assert_eq!(counters.pairwise_comparisons, 0);
    }

    #[test]
    fn anf_truth_table_is_exact() {
        // f(x0, x1) = 1 XOR x0 XOR (x0 * x1)
        let terms = [
            AnfTerm { variables: 0b01 },
            AnfTerm { variables: 0b11 },
        ];
        assert!(evaluate_anf(true, &terms, 0b00));
        assert!(!evaluate_anf(true, &terms, 0b01));
        assert!(evaluate_anf(true, &terms, 0b11));
    }

    #[test]
    fn semantic_memory_accounting_is_exact_by_contract() {
        assert_eq!(DirectAddressMemory::<8>::peak_semantic_memory_bits(), 8 * 129);
    }

    #[test]
    fn route_mapping_is_deterministic() {
        let state = BooleanState::from_bits(0xdead_beef);
        assert_eq!(route_tag(state, 7), route_tag(state, 7));
        assert_ne!(route_tag(state, 7), route_tag(state, 8));
    }

    #[test]
    fn pairwise_work_invalidates_candidate_evidence() {
        let evidence = EvidenceRecord {
            manifest: RunManifest {
                arm: ArchitectureArm::B2BooleanDirect,
                sequence_len: 4,
                state_width_bits: 8,
                memory_slots: 4,
                development_seed: 0,
            },
            correct: true,
            resources: ResourceCounters {
                pairwise_comparisons: 1,
                ..ResourceCounters::default()
            },
        };
        assert!(!evidence.candidate_structurally_valid());
    }
}
