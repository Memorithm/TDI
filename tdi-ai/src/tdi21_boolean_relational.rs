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
    pub const fn is_boolean_candidate(&self) -> bool {
        matches!(
            *self,
            Self::B2BooleanDirect
                | Self::B3BooleanAssociative
                | Self::B4BooleanAnf
                | Self::B5BooleanRelational
        )
    }
}

/// A caller declaration, not an inspection or attestation of executed code.
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
    pub const fn any(&self) -> bool {
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

/// Stage-0 declaration. The 64-bit ceiling bounds representation width; it
/// does not make exhaustive enumeration of all 64-bit states practical.
/// Declaring B3-B5 does not by itself implement or qualify those arms.
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

/// Fixed-width Stage-0 Boolean storage. Raw operations are not instrumented;
/// callers must use counted operations when exporting work measurements.
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
    pub const fn and_state(self, rhs: Self) -> Self {
        Self(self.0 & rhs.0)
    }

    #[must_use]
    pub const fn or_state(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }

    #[must_use]
    pub const fn xor_state(self, rhs: Self) -> Self {
        Self(self.0 ^ rhs.0)
    }

    #[must_use]
    pub const fn not_state(self, width_bits: u8) -> Self {
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
            // An invalid index is not a false proposition that can be negated.
            Self::NotBit(bit) => bit < 64 && !state.test(bit),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClauseValidationError {
    InvalidWidth,
    BitOutOfRange(u8),
}

/// A deterministic conjunction of Boolean literals.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Clause<const N: usize> {
    pub literals: [Literal; N],
}

impl<const N: usize> Clause<N> {
    /// Check every literal, including those hidden behind short-circuiting.
    pub fn validate(&self, width_bits: u8) -> Result<(), ClauseValidationError> {
        if width_bits == 0 || width_bits > 64 {
            return Err(ClauseValidationError::InvalidWidth);
        }
        for literal in &self.literals {
            let (Literal::Bit(bit) | Literal::NotBit(bit)) = *literal;
            if bit >= width_bits {
                return Err(ClauseValidationError::BitOutOfRange(bit));
            }
        }
        Ok(())
    }

    pub fn evaluate(&self, state: BooleanState, counters: &mut ResourceCounters) -> bool {
        for &literal in &self.literals {
            charge(&mut counters.boolean_primitive_evals, 1);
            if !literal.evaluate(state) {
                return false;
            }
        }
        true
    }
}

/// Checked accounting: exhaustion aborts the run instead of wrapping into
/// apparently valid evidence. No saturated counter is reported as exact.
fn charge(counter: &mut u64, amount: u64) {
    *counter = counter
        .checked_add(amount)
        .expect("TDI-21 counter overflow");
}

/// Separate semantic work categories, not a fabricated FLOP or hardware cost.
/// `boolean_primitive_evals` counts literal evaluations, including negated ones.
/// `word_boolean_evals` counts explicitly instrumented u64 bitwise/shift ops.
/// Address derivations, full-tag equality checks and ANF terms are separate.
/// A zero pairwise counter is only a declaration-consistency check: arbitrary
/// caller code cannot be certified attention-free by a mutable counter.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ResourceCounters {
    pub boolean_primitive_evals: u64,
    pub word_boolean_evals: u64,
    pub address_derivations: u64,
    pub tag_equality_checks: u64,
    pub anf_term_evals: u64,
    pub route_activations: u64,
    pub memory_reads: u64,
    pub memory_writes: u64,
    pub memory_misses: u64,
    pub memory_collisions: u64,
    pub memory_replacements: u64,
    pub pairwise_comparisons: u64,
}

impl ResourceCounters {
    /// Reject recorded token-pair scoring. This is not execution attestation.
    #[must_use]
    pub const fn candidate_is_pairwise_free(&self) -> bool {
        self.pairwise_comparisons == 0
    }

    /// Record explicitly executed packed-word Boolean operations.
    pub fn charge_word_boolean_evals(&mut self, amount: u64) {
        charge(&mut self.word_boolean_evals, amount);
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
/// tokens to find a match. A zero-slot fixture reports misses; operational
/// constructors must reject zero capacity before execution.
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
            // Reduce BEFORE narrowing: identical semantics on 32/64-bit hosts.
            Some(((tag as u128) % (SLOTS as u128)) as usize)
        }
    }

    pub fn write(&mut self, tag: u64, value: BooleanState, counters: &mut ResourceCounters) {
        charge(&mut counters.memory_writes, 1);
        let Some(index) = Self::index(tag) else {
            charge(&mut counters.memory_misses, 1);
            return;
        };

        if let Some(previous) = self.slots[index] {
            charge(&mut counters.tag_equality_checks, 1);
            if previous.tag != tag {
                charge(&mut counters.memory_collisions, 1);
                charge(&mut counters.memory_replacements, 1);
            }
        }
        self.slots[index] = Some(Slot { tag, value });
    }

    pub fn read(&self, tag: u64, counters: &mut ResourceCounters) -> MemoryRead {
        charge(&mut counters.memory_reads, 1);
        let Some(index) = Self::index(tag) else {
            charge(&mut counters.memory_misses, 1);
            return MemoryRead::Miss;
        };

        if let Some(slot) = self.slots[index] {
            charge(&mut counters.tag_equality_checks, 1);
            if slot.tag == tag {
                return MemoryRead::Hit(slot.value);
            }
        }
        charge(&mut counters.memory_misses, 1);
        MemoryRead::Miss
    }

    /// One occupancy bit, one 64-bit tag and one 64-bit payload per slot.
    /// This is not Rust padding, allocator overhead, evaluator storage or a
    /// complete architecture-memory claim. Overflow fails in every build mode.
    #[must_use]
    pub const fn peak_semantic_memory_bits() -> usize {
        SLOTS
            .checked_mul(129)
            .expect("TDI-21 memory accounting overflow")
    }
}

/// Versioned route semantics. V1 used XOR with rotations and lost information.
pub const ROUTE_SEMANTICS: &str = "tdi21-xorshift64-v2";

/// Bijective for every fixed salt. Each XOR-shift is triangular over GF(2)
/// with unit diagonal and therefore invertible. Reducing the returned tag to a
/// memory slot can still collide; memory must retain and check the full tag.
/// This is neither cryptographic hashing nor learned semantic addressing.
#[must_use]
pub const fn route_tag(state: BooleanState, salt: u64) -> u64 {
    let mut value = state.bits() ^ salt;
    value ^= value << 13;
    value ^= value >> 7;
    value ^ (value << 17)
}

pub fn counted_route_tag(state: BooleanState, salt: u64, counters: &mut ResourceCounters) -> u64 {
    charge(&mut counters.address_derivations, 1);
    // Salt XOR, three shifts, three XORs. Not seven CPU-cycle claims.
    counters.charge_word_boolean_evals(7);
    route_tag(state, salt)
}

pub fn activate_route<const N: usize>(
    state: BooleanState,
    clause: &Clause<N>,
    counters: &mut ResourceCounters,
) -> bool {
    let active = clause.evaluate(state, counters);
    if active {
        charge(&mut counters.route_activations, 1);
    }
    active
}

/// One algebraic-normal-form monomial over GF(2). A set bit selects a Boolean
/// variable participating in the product term. The empty product is true.
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

/// Raw ANF evaluation. Use the counted variant for exported work measurements.
#[must_use]
pub fn evaluate_anf(constant: bool, terms: &[AnfTerm], assignment: u64) -> bool {
    let mut value = constant;
    for &term in terms {
        value ^= term.evaluate(assignment);
    }
    value
}

/// Counts terms separately from literal or packed-word operation categories.
pub fn counted_evaluate_anf(
    constant: bool,
    terms: &[AnfTerm],
    assignment: u64,
    counters: &mut ResourceCounters,
) -> bool {
    let mut value = constant;
    for &term in terms {
        charge(&mut counters.anf_term_evals, 1);
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
    /// Check declared dimensions and recorded pairwise work, not arbitrary code
    /// execution or the availability/qualification of the declared arm.
    #[must_use]
    pub const fn candidate_structurally_valid(&self) -> bool {
        self.manifest.arm.is_boolean_candidate()
            && self.manifest.sequence_len > 0
            && self.manifest.state_width_bits > 0
            && self.manifest.state_width_bits <= 64
            && self.manifest.memory_slots > 0
            && self.resources.candidate_is_pairwise_free()
    }
}

/// Two-event round-trip smoke fixture, NOT a long-delay recall experiment.
#[must_use]
pub fn delayed_bit_recall<const SLOTS: usize>(key: u64, value: bool) -> EvidenceRecord {
    let mut memory = DirectAddressMemory::<SLOTS>::default();
    let mut resources = ResourceCounters::default();
    let encoded = BooleanState::from_bits(if value { 1 } else { 0 });
    let tag = counted_route_tag(
        BooleanState::from_bits(key),
        0x5444_4932_3100_0001,
        &mut resources,
    );

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
        let terms = [AnfTerm { variables: 0b01 }, AnfTerm { variables: 0b11 }];
        assert!(evaluate_anf(true, &terms, 0b00));
        assert!(!evaluate_anf(true, &terms, 0b01));
        assert!(evaluate_anf(true, &terms, 0b10));
        assert!(evaluate_anf(true, &terms, 0b11));
    }

    #[test]
    fn semantic_memory_accounting_is_exact_by_contract() {
        assert_eq!(
            DirectAddressMemory::<8>::peak_semantic_memory_bits(),
            8 * 129
        );
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
