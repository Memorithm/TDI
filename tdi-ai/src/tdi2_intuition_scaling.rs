//! Deterministic scaling schedules for TDI-2.1 development/validation.

/// Powers-of-two support schedule ending exactly at `max_support`.
///
/// Zero means no accumulated experience and is always included.
#[must_use]
pub fn support_schedule(max_support: u64) -> Vec<u64> {
    let mut values = vec![0];
    if max_support == 0 {
        return values;
    }
    let mut value = 1u64;
    while value < max_support {
        values.push(value);
        match value.checked_mul(2) {
            Some(next) => value = next,
            None => break,
        }
    }
    if values.last().copied() != Some(max_support) {
        values.push(max_support);
    }
    values
}

/// Positive powers-of-two memory-capacity schedule ending at `max_capacity`.
#[must_use]
pub fn capacity_schedule(max_capacity: usize) -> Vec<usize> {
    if max_capacity == 0 {
        return Vec::new();
    }
    let mut values = Vec::new();
    let mut value = 1usize;
    while value < max_capacity {
        values.push(value);
        match value.checked_mul(2) {
            Some(next) => value = next,
            None => break,
        }
    }
    if values.last().copied() != Some(max_capacity) {
        values.push(max_capacity);
    }
    values
}

#[cfg(test)]
mod tests {
    use super::{capacity_schedule, support_schedule};

    #[test]
    fn support_schedule_is_bounded_and_includes_endpoints() {
        assert_eq!(support_schedule(10), vec![0, 1, 2, 4, 8, 10]);
        assert_eq!(support_schedule(0), vec![0]);
    }

    #[test]
    fn capacity_schedule_is_positive_and_bounded() {
        assert_eq!(capacity_schedule(10), vec![1, 2, 4, 8, 10]);
        assert!(capacity_schedule(0).is_empty());
    }
}
