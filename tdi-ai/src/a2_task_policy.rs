//! Explicit bounded A2 task-routing candidate for TDI-8.1.
//!
//! PR #117 deliberately stopped before A2 because `A2Reference::step` requires
//! one read key and an optional write key on every recurrent event. This module
//! makes that missing semantic choice explicit and independently testable.
//!
//! The policy consumes only arguments already visible to `SymbolicTaskAdapter`
//! plus one runner-side distractor key derived from the immutable task instance.
//! Exact targets, source indices and generator collision classes are never arm
//! inputs. No capacity, projection seed, recurrent parameter, fusion gain,
//! horizon, population or TDI-8.2 surface is selected here.

use crate::task_encoding::{
    PayloadKeyCursor, TaskEncodingError, association_memory_key,
    distractor_read_key_for_instance, payload_memory_key,
};
use crate::task_generators::TaskInstance;

/// One deterministic A2 lookup-before-write route for one symbolic event.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct A2TaskRoute {
    read_key: u64,
    write_key: Option<u64>,
}

impl A2TaskRoute {
    /// Logical key read before the optional write.
    #[must_use]
    pub const fn read_key(self) -> u64 {
        self.read_key
    }

    /// Optional logical key written after recurrent/memory fusion.
    #[must_use]
    pub const fn write_key(self) -> Option<u64> {
        self.write_key
    }
}

/// Per-instance routing state for the bounded A2 adapter candidate.
///
/// Only T2 payload call order is persistent. The distractor key is selected
/// runner-side before execution and is guaranteed to differ from every logical
/// write key in the immutable task instance.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct A2TaskRouting {
    distractor_read_key: u64,
    payload_cursor: PayloadKeyCursor,
}

impl A2TaskRouting {
    /// Build routing for exactly one immutable task instance.
    pub fn for_instance(instance: &TaskInstance) -> Result<Self, TaskEncodingError> {
        Ok(Self {
            distractor_read_key: distractor_read_key_for_instance(instance)?,
            payload_cursor: PayloadKeyCursor::default(),
        })
    }

    /// Reset chronological payload routing while preserving this instance's
    /// prevalidated distractor key.
    pub fn reset(&mut self) {
        self.payload_cursor.reset();
    }

    /// Association write policy: lookup the same logical association key before
    /// writing the post-fusion recurrent state under that key.
    #[must_use]
    pub const fn association(self, key_code: u64) -> A2TaskRoute {
        let key = association_memory_key(key_code);
        A2TaskRoute {
            read_key: key,
            write_key: Some(key),
        }
    }

    /// Build the next T2 payload route without mutating this routing state.
    ///
    /// The returned next routing value is committed by the adapter only after
    /// the corresponding A2 reference step succeeds.
    pub fn prospective_payload(self) -> Result<(A2TaskRoute, Self), TaskEncodingError> {
        let mut next = self;
        let key = next.payload_cursor.next_write_key()?;
        Ok((
            A2TaskRoute {
                read_key: key,
                write_key: Some(key),
            },
            next,
        ))
    }

    /// Distractors never write memory and use the prevalidated runner-side key.
    #[must_use]
    pub const fn distractor(self) -> A2TaskRoute {
        A2TaskRoute {
            read_key: self.distractor_read_key,
            write_key: None,
        }
    }

    /// Association query policy: read the exact logical association key only.
    #[must_use]
    pub const fn query_association(self, key_code: u64) -> A2TaskRoute {
        A2TaskRoute {
            read_key: association_memory_key(key_code),
            write_key: None,
        }
    }

    /// Payload query policy: read the deterministic call-order payload key only.
    #[must_use]
    pub fn query_payload(self, position: u64) -> A2TaskRoute {
        A2TaskRoute {
            read_key: payload_memory_key(position),
            write_key: None,
        }
    }

    /// Runner-side distractor key retained only for routing diagnostics.
    #[must_use]
    pub const fn distractor_read_key(self) -> u64 {
        self.distractor_read_key
    }

    /// Next T2 position implied solely by successful chronological payload calls.
    #[must_use]
    pub const fn next_payload_position(self) -> u64 {
        self.payload_cursor.next_position()
    }
}

#[cfg(test)]
mod tests {
    use super::A2TaskRouting;
    use crate::task_encoding::{
        association_memory_key, distractor_read_key_for_instance, payload_memory_key,
    };
    use crate::task_generators::{
        T1Config, T2Config, generate_t1, generate_t2,
    };

    #[test]
    fn association_write_and_query_use_the_same_logical_key() {
        let task = generate_t1(21, T1Config::new(3, 1, 1).expect("config")).expect("task");
        let routing = A2TaskRouting::for_instance(&task).expect("routing");
        let key_code = 0x1234_5678_9abc_def0;
        let write = routing.association(key_code);
        let query = routing.query_association(key_code);
        assert_eq!(write.read_key(), association_memory_key(key_code));
        assert_eq!(write.write_key(), Some(association_memory_key(key_code)));
        assert_eq!(query.read_key(), association_memory_key(key_code));
        assert_eq!(query.write_key(), None);
    }

    #[test]
    fn payload_route_is_prospective_until_explicit_commit() {
        let task = generate_t2(22, T2Config::new(2, 1).expect("config")).expect("task");
        let routing = A2TaskRouting::for_instance(&task).expect("routing");
        let (first, next) = routing.prospective_payload().expect("first route");
        assert_eq!(routing.next_payload_position(), 0);
        assert_eq!(next.next_payload_position(), 1);
        assert_eq!(first.read_key(), payload_memory_key(0));
        assert_eq!(first.write_key(), Some(payload_memory_key(0)));
        assert_eq!(next.query_payload(0).read_key(), first.read_key());
    }

    #[test]
    fn distractor_route_uses_the_prevalidated_instance_key_and_never_writes() {
        let task = generate_t1(23, T1Config::new(4, 2, 2).expect("config")).expect("task");
        let routing = A2TaskRouting::for_instance(&task).expect("routing");
        let expected = distractor_read_key_for_instance(&task).expect("safe key");
        let route = routing.distractor();
        assert_eq!(route.read_key(), expected);
        assert_eq!(route.write_key(), None);
    }
}
