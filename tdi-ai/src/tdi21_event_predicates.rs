//! Versioned present/local-state predicate adapter for future TDI-21 B4 routing.
//!
//! The adapter sees one current Write event plus a bounded local bucket
//! observation produced by the candidate's own B3 memory. It does not receive
//! stored payloads, oracle answers, future events, Validation labels or an
//! attention score. The mapping is fixed and development-only.

use super::tdi21_stream::{Event, RouteObservation};

pub const WRITE_PREDICATE_SEMANTICS: &str = "tdi21-write-local-predicates-v1";
pub const WRITE_PREDICATE_WIDTH: u8 = 6;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum WritePredicate {
    MarkerBit0 = 0,
    MarkerBit1 = 1,
    ExactTagPresent = 2,
    BucketHasAny = 3,
    BucketFull = 4,
    NextVictimSecond = 5,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WritePredicateVector {
    assignment: u64,
}

impl WritePredicateVector {
    #[must_use]
    pub const fn assignment(self) -> u64 {
        self.assignment
    }

    #[must_use]
    pub const fn test(self, predicate: WritePredicate) -> bool {
        (self.assignment & (1u64 << predicate as u8)) != 0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WritePredicateError {
    NotWriteEvent,
    InvalidMarker,
    RequiresTwoWayObservation,
    InvalidObservation,
}

fn set_if(assignment: &mut u64, predicate: WritePredicate, enabled: bool) {
    if enabled {
        *assignment |= 1u64 << predicate as u8;
    }
}

/// Encode the fixed six-bit B4 write-routing observation.
///
/// Bits are, in order: marker bit 0, marker bit 1, exact-tag-present,
/// bucket-has-any-entry, bucket-full, and next-victim-is-second-way.
/// No key bits or stored payload bits are exposed in v1.
pub fn encode_b4_write_predicates(
    event: Event,
    observation: RouteObservation,
) -> Result<WritePredicateVector, WritePredicateError> {
    let marker = match event {
        Event::Write { marker, .. } => marker.bits(),
        _ => return Err(WritePredicateError::NotWriteEvent),
    };
    if marker > 3 {
        return Err(WritePredicateError::InvalidMarker);
    }
    if observation.ways != 2 {
        return Err(WritePredicateError::RequiresTwoWayObservation);
    }
    if observation.occupied_ways > observation.ways
        || observation.bucket_full != (observation.occupied_ways == observation.ways)
        || observation.next_victim_way >= observation.ways
        || (observation.exact_present && observation.occupied_ways == 0)
    {
        return Err(WritePredicateError::InvalidObservation);
    }

    let mut assignment = 0u64;
    set_if(
        &mut assignment,
        WritePredicate::MarkerBit0,
        marker & 0b01 != 0,
    );
    set_if(
        &mut assignment,
        WritePredicate::MarkerBit1,
        marker & 0b10 != 0,
    );
    set_if(
        &mut assignment,
        WritePredicate::ExactTagPresent,
        observation.exact_present,
    );
    set_if(
        &mut assignment,
        WritePredicate::BucketHasAny,
        observation.occupied_ways != 0,
    );
    set_if(
        &mut assignment,
        WritePredicate::BucketFull,
        observation.bucket_full,
    );
    set_if(
        &mut assignment,
        WritePredicate::NextVictimSecond,
        observation.next_victim_way == 1,
    );
    Ok(WritePredicateVector { assignment })
}
