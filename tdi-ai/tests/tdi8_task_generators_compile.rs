use std::collections::BTreeSet;

use tdi_ai::task_generators::{
    HorizonPlan, HorizonStratum, T1Config, T2Config, T3Config, TaskEvent, TaskFamily, generate_t1,
    generate_t2, generate_t3,
};

#[test]
fn downstream_can_build_all_three_symbolic_task_families() {
    // Synthetic software-oracle fixtures only. These values are deliberately not
    // frozen experimental TDI-8.1 dimensions or horizon choices.
    let horizons = HorizonPlan::new(2, 4, 8).expect("synthetic horizon plan");
    assert_eq!(horizons.value(HorizonStratum::Short), 2);
    assert_eq!(horizons.value(HorizonStratum::Medium), 4);
    assert_eq!(horizons.value(HorizonStratum::Long), 8);

    let t1 = generate_t1(11, T1Config::new(4, 2, 1).expect("synthetic T1 config"))
        .expect("deterministic T1");
    assert_eq!(t1.family(), TaskFamily::AssociativeRecall);
    assert!(t1
        .events()
        .iter()
        .any(|event| matches!(event, TaskEvent::QueryAssociation { .. })));

    let t2 = generate_t2(13, T2Config::new(3, 2).expect("synthetic T2 config"))
        .expect("deterministic T2");
    assert_eq!(t2.family(), TaskFamily::DelayedCopy);
    assert!(t2
        .events()
        .iter()
        .any(|event| matches!(event, TaskEvent::QueryPayload { .. })));

    let t3 = generate_t3(
        17,
        T3Config::new(6, 2, 3, 16, 2).expect("synthetic T3 config"),
    )
    .expect("deterministic T3");
    assert_eq!(t3.family(), TaskFamily::InterferenceRecall);
    let query_sources: Vec<_> = t3
        .events()
        .iter()
        .filter_map(|event| match event {
            TaskEvent::QueryAssociation { source_index, .. } => Some(*source_index),
            _ => None,
        })
        .collect();
    assert!(query_sources.contains(&0));
    assert!(query_sources.contains(&5));
}

#[test]
fn t1_public_schedule_does_not_duplicate_association_keys_in_a_bounded_fixture() {
    let instance = generate_t1(
        0x51a7_7e57,
        T1Config::new(1024, 1, 1).expect("synthetic bounded T1 config"),
    )
    .expect("deterministic bounded T1");

    let mut keys = BTreeSet::new();
    let mut association_count = 0usize;
    for event in instance.events() {
        if let TaskEvent::Associate { key, .. } = event {
            association_count += 1;
            assert!(keys.insert(key.code()), "duplicate generated T1 key");
        }
    }
    assert_eq!(association_count, 1024);
    assert_eq!(keys.len(), association_count);
}
