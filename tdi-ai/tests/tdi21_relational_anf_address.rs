#![cfg(feature = "experimental")]

use tdi_ai::experimental::tdi21_relational_binding::{
    RelationalAddressMode, RelationalBinder, RelationalConfig, RelationalError, RelationalRead,
};
use tdi_ai::experimental::tdi21_stream::{MemoryMode, StreamConfig};

fn config(max_events: u64) -> RelationalConfig {
    RelationalConfig {
        stream: StreamConfig {
            mode: MemoryMode::TwoWay,
            slots: 128,
            payload_bits: 4,
            max_events,
            route_salt: 0x5444_4932_3100_0021,
        },
        entity_bits: 4,
        relation_bits: 4,
    }
}

#[test]
fn anf_identity_matches_exact_packing_for_every_v1_input_assignment() {
    let mut exact = RelationalBinder::new(config(16)).unwrap();
    let mut anf = RelationalBinder::new_anf_identity(config(16)).unwrap();
    assert_eq!(exact.address_mode(), RelationalAddressMode::ExactPacked);
    assert_eq!(anf.address_mode(), RelationalAddressMode::AnfIdentity);

    for relation in 0..16 {
        for subject in 0..16 {
            let object = (relation ^ subject) & 0x0f;
            exact.bind(relation, subject, object).unwrap();
            anf.bind(relation, subject, object).unwrap();
            assert_eq!(
                exact.recall(relation, subject),
                anf.recall(relation, subject)
            );
            assert_eq!(
                exact.recall(relation, subject).unwrap(),
                RelationalRead::Hit(object)
            );
            assert_eq!(
                anf.recall(relation, subject).unwrap(),
                RelationalRead::Hit(object)
            );

            assert_eq!(exact.relational_work().address_derivations, 3);
            assert_eq!(anf.relational_work().address_derivations, 3);
            assert_eq!(exact.anf_work().anf_term_evals, 0);
            // Eight identity output bits, one monomial each, evaluated once per
            // bind/recall address derivation: three derivations in this fixture.
            assert_eq!(anf.anf_work().anf_term_evals, 24);
            assert_eq!(exact.counters().work.pairwise_comparisons, 0);
            assert_eq!(anf.counters().work.pairwise_comparisons, 0);

            exact.reset();
            anf.reset();
        }
    }
}

#[test]
fn anf_identity_rejects_address_width_above_exact_synthesis_ceiling() {
    let mut wide = config(16);
    wide.entity_bits = 8;
    wide.relation_bits = 8;
    wide.stream.payload_bits = 8;
    assert_eq!(
        RelationalBinder::new_anf_identity(wide),
        Err(RelationalError::TooManyAnfAddressVariables)
    );
    assert!(RelationalBinder::new(wide).is_ok());
}

#[test]
fn rejected_identifier_does_not_commit_anf_or_address_work() {
    let mut anf = RelationalBinder::new_anf_identity(config(16)).unwrap();
    let before_stream = anf.counters();
    let before_relational = anf.relational_work();
    let before_anf = anf.anf_work();

    assert_eq!(anf.bind(16, 1, 2), Err(RelationalError::RelationOutOfRange));
    assert_eq!(anf.counters(), before_stream);
    assert_eq!(anf.relational_work(), before_relational);
    assert_eq!(anf.anf_work(), before_anf);
}

#[test]
fn exhausted_event_budget_does_not_commit_staged_anf_work() {
    let mut anf = RelationalBinder::new_anf_identity(config(1)).unwrap();
    anf.bind(1, 1, 2).unwrap();
    let before_relational = anf.relational_work();
    let before_anf = anf.anf_work();
    assert!(matches!(anf.recall(1, 1), Err(RelationalError::Stream(_))));
    assert_eq!(anf.relational_work(), before_relational);
    assert_eq!(anf.anf_work(), before_anf);
}
