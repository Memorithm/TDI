//! TDI-13 bootstrap: deterministic Boolean relational architecture primitives.
//!
//! This module intentionally does not implement attention. Candidate mechanisms
//! must not depend on Q/K/V projections, pairwise similarity ranking, softmax,
//! or dense token-token score matrices.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArchitectureArm {
    B0AttentionReference,
    B1BinaryAttentionReference,
    B2BooleanDirect,
    B3BooleanAssociative,
    B4BooleanAnf,
    B5BooleanRelational,
}

impl ArchitectureArm {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ForbiddenMechanisms {
    pub qkv_projection: bool,
    pub pairwise_dot_product: bool,
    pub cosine_similarity: bool,
    pub softmax: bool,
    pub hamming_attention_score: bool,
    pub dense_pairwise_matrix: bool,
    pub attention_fallback: bool,
}

impl ForbiddenMechanisms {
    pub const fn any(self) -> bool {
        self.qkv_projection
            || self.pairwise_dot_product
            || self.cosine_similarity
            || self.softmax
            || self.hamming_attention_score
            || self.dense_pairwise_matrix
            || self.attention_fallback
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CandidateValidationError {
    ReferenceArmIsNotCandidate,
    ForbiddenAttentionMechanism,
    ZeroStateWidth,
    ZeroMemorySlots,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CandidateConfig {
    pub arm: ArchitectureArm,
    pub state_width_bits: u16,
    pub memory_slots: u16,
    pub forbidden: ForbiddenMechanisms,
}

impl CandidateConfig {
    pub const fn validate(self) -> Result<(), CandidateValidationError> {
        if !self.arm.is_boolean_candidate() {
            return Err(CandidateValidationError::ReferenceArmIsNotCandidate);
        }
        if self.forbidden.any() {
            return Err(CandidateValidationError::ForbiddenAttentionMechanism);
        }
        if self.state_width_bits == 0 {
            return Err(CandidateValidationError::ZeroStateWidth);
        }
        if self.memory_slots == 0 {
            return Err(CandidateValidationError::ZeroMemorySlots);
        }
        Ok(())
    }
}

/// Small fixed-width Boolean state for the Stage-0 reference evaluator.
///
/// The bootstrap intentionally starts with <= 64 bits so exact state and
/// accounting semantics remain auditable. Larger representations belong to a
/// later experiment after Stage-0 behavior is frozen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct BooleanState(u64);

impl BooleanState {
    pub const fn from_bits(bits: u64) -> Self {
        Self(bits)
    }

    pub const fn bits(self) -> u64 {
        self.0
    }

    pub const fn test(self, bit: u8) -> bool {
        if bit >= 64 {
            false
        } else {
            ((self.0 >> bit) & 1) != 0
        }
    }

    pub const fn xor(self, rhs: Self) -> Self {
        Self(self.0 ^ rhs.0)
    }

    pub const fn and(self, rhs: Self) -> Self {
        Self(self.0 & rhs.0)
    }

    pub const fn or(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }

    pub const fn not(self, width_bits: u8) -> Self {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Literal {
    Bit(u8),
    NotBit(u8),
}

impl Literal {
    const fn eval(self, state: BooleanState) -> bool {
        match self {
            Self::Bit(bit) => state.test(bit),
            Self::NotBit(bit) => !state.test(bit),
        }
    }
}

/// A bounded conjunction. Route predicates are explicit and deterministic;
/// there is no ranking step and no token-pair score.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Clause<const N: usize> {
    pub literals: [Literal; N],
}

impl<const N: usize> Clause<N> {
    pub fn eval(&self, state: BooleanState, counters: &mut ResourceCounters) -> bool {
        for literal in self.literals {
            counters.boolean_primitive_evals += 1;
            if !literal.eval(state) {
                return false;
            }
        }
        true
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ResourceCounters {
    pub boolean_primitive_evals: u64,
    pub route_activations: u64,
    pub memory_reads: u64,
    pub memory_writes: u64,
    pub memory_misses: u64,
    pub memory_collisions: u64,
    pub memory_replacements: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryRead {
    Hit(BooleanState),
    Miss,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Slot {
    tag: u64,
    value: BooleanState,
}

/// Deterministic bounded direct-address memory.
///
/// Address selection is O(1) in the number of stored sequence elements: no
/// scan over previous tokens is permitted. A full-width tag distinguishes a
/// genuine hit from an address collision.
#[derive(Debug, Clone, PartialEq, Eq)]
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

    pub const fn peak_dynamic_memory_bits() -> usize {
        // Conservative exact Stage-0 accounting for the semantic payload:
        // occupied flag + 64-bit tag + 64-bit Boolean state per slot.
        // Rust layout/padding is intentionally not claimed by this semantic
        // counter and can be measured separately by an implementation bench.
        SLOTS * (1 + 64 + 64)
    }
}

/// Deterministic F2-style address mixer. This is routing, not similarity:
/// one current Boolean state maps directly to one tag without comparing it to
/// any stored token.
pub const fn route_tag(state: BooleanState, salt: u64) -> u64 {
    let mut x = state.bits() ^ salt;
    x ^= x >> 33;
    x = x.wrapping_mul(0xff51afd7ed558ccd);
    x ^= x >> 33;
    x = x.wrapping_mul(0xc4ceb9fe1a85ec53);
    x ^ (x >> 33)
}

pub fn activate_route<const N: usize>(
    state: BooleanState,
    clause: &Clause<N>,
    counters: &mut ResourceCounters,
) -> bool {
    let active = clause.eval(state, counters);
    if active {
        counters.route_activations += 1;
    }
    active
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn boolean_candidates_reject_attention_mechanisms() {
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
    fn reference_arms_are_not_candidate_arms() {
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
    fn delayed_boolean_recall_is_directly_addressed() {
        let mut memory = DirectAddressMemory::<8>::default();
        let mut counters = ResourceCounters::default();
        let fact = BooleanState::from_bits(0b1010_0110);
        let key = BooleanState::from_bits(0b0011_0101);
        let tag = route_tag(key, 0x13);

        memory.write(tag, fact, &mut counters);

        // Distractors do not trigger a scan over memory.
        for distractor in 0u64..32 {
            let _ = route_tag(BooleanState::from_bits(distractor), 0x99);
        }

        assert_eq!(memory.read(tag, &mut counters), MemoryRead::Hit(fact));
        assert_eq!(counters.memory_reads, 1);
        assert_eq!(counters.memory_writes, 1);
        assert_eq!(counters.memory_misses, 0);
    }

    #[test]
    fn collision_is_explicit_and_old_tag_becomes_miss() {
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
    fn distractor_rejection_uses_clause_not_similarity() {
        let clause = Clause {
            literals: [Literal::Bit(0), Literal::NotBit(1), Literal::Bit(3)],
        };
        let target = BooleanState::from_bits(0b1001);
        let distractor = BooleanState::from_bits(0b1011);
        let mut counters = ResourceCounters::default();

        assert!(activate_route(target, &clause, &mut counters));
        assert!(!activate_route(distractor, &clause, &mut counters));
        assert_eq!(counters.route_activations, 1);
    }

    #[test]
    fn semantic_memory_accounting_is_exact() {
        assert_eq!(DirectAddressMemory::<8>::peak_dynamic_memory_bits(), 8 * 129);
    }
}
