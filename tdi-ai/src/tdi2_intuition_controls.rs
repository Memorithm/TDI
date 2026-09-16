//! Matched deterministic controls for TDI-2.1 candidate ranking.

use super::tdi2_intuition::TemplateId;

/// Produce a reproducible pseudo-random ranking over an unchanged candidate set.
///
/// This control changes only ordering. It does not add or remove templates and
/// therefore supports matched comparisons against experience-weight ranking.
#[must_use]
pub fn pseudo_random_ranking(template_ids: &[TemplateId], seed: u64) -> Vec<TemplateId> {
    let mut ranked = template_ids.to_vec();
    ranked.sort_unstable_by_key(|template_id| splitmix64(seed ^ template_id.raw()));
    ranked
}

fn splitmix64(mut value: u64) -> u64 {
    value = value.wrapping_add(0x9e37_79b9_7f4a_7c15);
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}

#[cfg(test)]
mod tests {
    use super::pseudo_random_ranking;
    use crate::experimental::tdi2_intuition::TemplateId;

    #[test]
    fn control_is_reproducible_and_preserves_candidate_set() {
        let ids = [TemplateId::new(1), TemplateId::new(2), TemplateId::new(3)];
        let first = pseudo_random_ranking(&ids, 17);
        let second = pseudo_random_ranking(&ids, 17);
        assert_eq!(first, second);
        let mut sorted = first;
        sorted.sort_unstable();
        assert_eq!(sorted, ids);
    }
}
