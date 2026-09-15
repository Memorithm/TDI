//! Versioned present/local-state predicate adapter for future TDI-21 B4 routing.
//!
//! The public adapter observes the bucket addressed by the current Write event
//! itself. Callers cannot pair metadata probed from another key with the write
//! being encoded. It does not receive stored payloads, oracle answers, future
//! events, Validation labels or an attention score. The mapping is fixed and
//! development-only.

use super::tdi21_stream::{BooleanStream, Event, MemoryMode, RouteObservation};

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

fn encode_observation(
    marker: u64,
    observation: RouteObservation,
) -> Result<WritePredicateVector, WritePredicateError> {
    if observation.ways != 2 {
        return Err(WritePredicateError::RequiresTwoWayObservation);
    }
    if observation.occupied_ways > observation.ways
        || observation.bucket_full != (observation.occupied_ways == observation.ways)
        || observation.next_victim_way >= observation.ways
        || (observation.exact_present && observation.occupied_ways == 0)
        || (!observation.bucket_full && observation.next_victim_way != 0)
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

/// Observe and encode the fixed six-bit B4 write-routing state atomically.
///
/// The bucket observation is always derived from the key carried by `event`;
/// callers cannot supply an independently probed observation. Bits are, in
/// order: marker bit 0, marker bit 1, exact-tag-present, bucket-has-any-entry,
/// bucket-full, and next-victim-is-second-way. No key bits or stored payload
/// bits are exposed in v1.
pub fn encode_b4_write_predicates(
    stream: &mut BooleanStream,
    event: Event,
) -> Result<WritePredicateVector, WritePredicateError> {
    let (key, marker) = match event {
        Event::Write { key, marker, .. } => (key, marker.bits()),
        _ => return Err(WritePredicateError::NotWriteEvent),
    };
    if marker > 3 {
        return Err(WritePredicateError::InvalidMarker);
    }
    if stream.config().mode != MemoryMode::TwoWay {
        return Err(WritePredicateError::RequiresTwoWayObservation);
    }
    let observation = stream.observe_key(key);
    encode_observation(marker, observation)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn impossible_nonfull_second_victim_is_rejected() {
        let observation = RouteObservation {
            exact_present: false,
            bucket_full: false,
            occupied_ways: 1,
            ways: 2,
            next_victim_way: 1,
        };
        assert_eq!(
            encode_observation(1, observation),
            Err(WritePredicateError::InvalidObservation)
        );
    }

    #[test]
    fn full_bucket_may_expose_either_replacement_victim() {
        for next_victim_way in [0, 1] {
            let observation = RouteObservation {
                exact_present: false,
                bucket_full: true,
                occupied_ways: 2,
                ways: 2,
                next_victim_way,
            };
            assert!(encode_observation(1, observation).is_ok());
        }
    }
}
