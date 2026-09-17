//! External logical-operation accounting for two frozen TDI-2.1 memories.
use tdi_ai::experimental::tdi2_intuition_cost::reference_operation_accounting;
use tdi_ai::experimental::tdi2_intuition_experience::{
    context_experience_store, motif_experience_store,
};
use tdi_ai::experimental::tdi2_intuition_tasks::{context_template_pair, motif_retrieval_case};

fn main() {
    let motif_store = motif_experience_store(4);
    let motif_case = motif_retrieval_case(1_000);
    let motif = reference_operation_accounting(&motif_store, motif_case.query());
    assert_eq!(motif.templates_considered, 17);
    assert_eq!(motif.boolean_clauses_checked, 17);
    assert_eq!(motif.candidates_selected, 1);

    let context_store = context_experience_store(4);
    let context_case = context_template_pair(3_000)[0].clone();
    let context = reference_operation_accounting(&context_store, context_case.query());
    assert_eq!(context.templates_considered, 26);
    assert_eq!(context.boolean_clauses_checked, 52);
    assert_eq!(context.candidates_selected, 1);

    println!("motif=[{}]", motif.canonical_record());
    println!("context=[{}]", context.canonical_record());
}
