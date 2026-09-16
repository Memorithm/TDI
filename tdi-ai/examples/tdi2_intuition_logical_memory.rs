//! Logical memory accounting without physical-byte claims.
use tdi_ai::experimental::tdi2_intuition_cost::logical_memory_accounting;
use tdi_ai::experimental::tdi2_intuition_experience::{
    context_experience_store, motif_experience_store,
};

fn main() {
    let motif = logical_memory_accounting(&motif_experience_store(4));
    let context = logical_memory_accounting(&context_experience_store(4));
    assert_eq!(motif.templates, 17);
    assert_eq!(motif.boolean_clauses, 17);
    assert_eq!(motif.reliability_counters, 34);
    assert_eq!(context.templates, 26);
    assert_eq!(context.boolean_clauses, 52);
    assert_eq!(context.reliability_counters, 52);
    println!(
        "tdi2.1-logical-memory-v1;family=motif;templates={};clauses={};roles={};relations={};counters={}",
        motif.templates,
        motif.boolean_clauses,
        motif.roles,
        motif.relations,
        motif.reliability_counters
    );
    println!(
        "tdi2.1-logical-memory-v1;family=context;templates={};clauses={};roles={};relations={};counters={}",
        context.templates,
        context.boolean_clauses,
        context.roles,
        context.relations,
        context.reliability_counters
    );
}
