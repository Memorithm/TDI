#![cfg(feature = "experimental")]

use tdi_ai::experimental::tdi21_relational_binding::{
    RelationalAddressMode, RelationalBinder, RelationalConfig,
};
use tdi_ai::experimental::tdi21_relational_tasks::{
    DevelopmentRelationalSet, RELATIONAL_V1_ENTITY_BITS, RELATIONAL_V1_RELATION_BITS,
    RelationalEpisode, ValidationRelationalSet,
};
use tdi_ai::experimental::tdi21_stream::{MemoryMode, StreamConfig};

fn config() -> RelationalConfig {
    RelationalConfig {
        stream: StreamConfig {
            mode: MemoryMode::TwoWay,
            slots: 128,
            payload_bits: RELATIONAL_V1_ENTITY_BITS,
            max_events: 128,
            route_salt: 0x5444_4932_3100_0021,
        },
        entity_bits: RELATIONAL_V1_ENTITY_BITS,
        relation_bits: RELATIONAL_V1_RELATION_BITS,
    }
}

fn run_episode(
    mut binder: RelationalBinder,
    episode: &RelationalEpisode,
) -> (tdi_ai::experimental::tdi21_relational_binding::RelationalRead, RelationalBinder) {
    for fact in &episode.facts {
        binder.bind(fact.relation, fact.subject, fact.object).unwrap();
    }
    let answer = binder
        .compose_path(&episode.query.relations, episode.query.subject)
        .unwrap();
    (answer, binder)
}

fn assert_exact_and_anf_agree(episode: &RelationalEpisode) {
    let (exact_answer, exact) = run_episode(RelationalBinder::new(config()).unwrap(), episode);
    let (anf_answer, anf) = run_episode(
        RelationalBinder::new_anf_identity(config()).unwrap(),
        episode,
    );

    assert_eq!(exact.address_mode(), RelationalAddressMode::ExactPacked);
    assert_eq!(anf.address_mode(), RelationalAddressMode::AnfIdentity);
    assert_eq!(exact_answer, episode.expected());
    assert_eq!(anf_answer, episode.expected());
    assert_eq!(exact_answer, anf_answer);
    assert_eq!(exact.counters(), anf.counters());
    assert_eq!(exact.relational_work(), anf.relational_work());
    assert_eq!(exact.anf_work().anf_term_evals, 0);
    assert!(anf.anf_work().anf_term_evals > 0);
    assert_eq!(anf.counters().work.pairwise_comparisons, 0);
}

#[test]
fn anf_identity_matches_exact_addressing_on_all_development_episodes() {
    for episode in DevelopmentRelationalSet::v1().episodes() {
        assert_exact_and_anf_agree(episode);
    }
}

#[test]
fn anf_identity_transfers_unchanged_to_disjoint_validation_identifiers() {
    for episode in ValidationRelationalSet::v1().episodes() {
        assert_exact_and_anf_agree(episode);
    }
}
