//! Versioned, non-final TDI-9.3 C3 representation fixture for ElasticXxx interop.
//!
//! This exporter is representation-only. It enumerates the existing abstract
//! nine-predicate C3 carrier and exposes no trajectory observations, thresholds,
//! Development/Validation/Final data, TDI-9.1 freeze authority, or TDI-9.2
//! confirmation authority.

use std::error::Error;

use tdi_ai::experimental::adaptive_inference::InferenceAction;
use tdi_ai::experimental::boolean_policy_carrier::{C3CarrierError, ValidatedC3PredicateRow};
use tdi_ai::experimental::boolean_policy_synthesis::{
    REFERENCE_C3_PREDICATE_COUNT, reference_c3_hand_action, reference_c3_policy,
};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

fn action_token(action: InferenceAction) -> &'static str {
    match action {
        InferenceAction::Continue => "CONTINUE",
        InferenceAction::Verify => "VERIFY",
        InferenceAction::Backtrack => "BACKTRACK",
        InferenceAction::Stop => "STOP",
    }
}

fn bit_string(row: &[bool; REFERENCE_C3_PREDICATE_COUNT]) -> String {
    row.iter()
        .map(|value| if *value { '1' } else { '0' })
        .collect()
}

fn main() -> Result<()> {
    if std::env::args_os().len() != 1 {
        return Err("fixed interop exporter accepts no arguments".into());
    }

    let policy = reference_c3_policy()?;
    println!("# schema=tdi9.3.elasticxxx-c3-carrier.v1");
    println!("# claim_boundary=non-final-representation-only");
    println!(
        "# predicate_order=BASE_STOP,VERIFY_BEFORE_STOP,CADENCE_DUE,CHECKPOINT_AVAILABLE,REMAINING_WORK,VERIFIER_VIOLATED,VERIFIER_SATISFIED,VERIFIER_INDETERMINATE,VERIFIER_ABSENT"
    );
    println!("# bit_order=p0-to-p8-left-to-right");
    println!("# missing_predicate_semantics=not-represented-by-TDI-binary-carrier");
    println!("# columns=row\tpredicates\tclassification\taction");

    let mut action_rows = 0usize;
    let mut unrecoverable = 0usize;
    let mut invalid = 0usize;

    for address in 0..(1usize << REFERENCE_C3_PREDICATE_COUNT) {
        let row: [bool; REFERENCE_C3_PREDICATE_COUNT] =
            std::array::from_fn(|bit| address & (1usize << bit) != 0);
        match ValidatedC3PredicateRow::new(&row) {
            Ok(validated) => {
                let decision = validated.decide(&policy)?;
                let hand = reference_c3_hand_action(&row)
                    .ok_or("validated carrier row has no hand-reference action")?;
                if decision.action() != hand {
                    return Err(format!("policy/hand mismatch at row {address}").into());
                }
                action_rows += 1;
                println!(
                    "{address}\t{}\tACTION\t{}",
                    bit_string(&row),
                    action_token(decision.action())
                );
            }
            Err(C3CarrierError::UnrecoverableViolation) => {
                if reference_c3_hand_action(&row).is_some() {
                    return Err(
                        format!("unrecoverable row {address} unexpectedly has action").into(),
                    );
                }
                unrecoverable += 1;
                println!("{address}\t{}\tUNRECOVERABLE\t-", bit_string(&row));
            }
            Err(C3CarrierError::InvalidVerifierEncoding { .. }) => {
                // The raw hand oracle is intentionally not an input validator and
                // may return an action for a contradictory one-hot encoding. The
                // checked carrier owns this rejection boundary, so no action is
                // exported for INVALID rows.
                invalid += 1;
                println!("{address}\t{}\tINVALID\t-", bit_string(&row));
            }
            Err(other) => {
                return Err(format!("unexpected carrier error at row {address}: {other}").into());
            }
        }
    }

    if (action_rows, unrecoverable, invalid) != (120, 8, 384) {
        return Err(format!(
            "unexpected carrier partition: action={action_rows} unrecoverable={unrecoverable} invalid={invalid}"
        )
        .into());
    }
    Ok(())
}
