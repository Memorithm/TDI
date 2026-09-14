//! Exact local C3 perturbation report; not a trajectory or robustness study.

use std::error::Error;

use tdi_ai::experimental::adaptive_inference::{InferenceAction, PolicyArm};
use tdi_ai::experimental::boolean_policy_search::BooleanPolicySearchCandidate;
use tdi_ai::experimental::boolean_policy_search_c3::generate_c3_single_rule_baselines;
use tdi_ai::experimental::boolean_policy_sensitivity::{
    C3_SENSITIVITY_AXES, C3_SENSITIVITY_SCHEMA, c3_policy_sensitivity,
};
use tdi_ai::experimental::boolean_policy_synthesis::{
    BooleanPolicy, SynthesisSearchEnvelope, reference_c3_policy,
};

fn main() -> Result<(), Box<dyn Error>> {
    if std::env::args_os().len() != 1 {
        return Err("fixed C3 sensitivity calibration accepts no arguments".into());
    }
    let mut candidates =
        generate_c3_single_rule_baselines(&SynthesisSearchEnvelope::reference_c3())?;
    candidates.push(BooleanPolicySearchCandidate {
        candidate_id: "full-reference".to_owned(),
        policy: reference_c3_policy()?,
    });
    candidates.push(BooleanPolicySearchCandidate {
        candidate_id: "constant-continue-negative-control".to_owned(),
        policy: BooleanPolicy::new(
            PolicyArm::C3VerificationRecovery,
            Vec::new(),
            InferenceAction::Continue,
        )?,
    });
    println!("# schema={C3_SENSITIVITY_SCHEMA}; evidence=EXACT_REPRESENTATION");
    println!("# lower sensitivity is not correctness; categorical substitutions change two bits");
    println!("# matrix order=CONTINUE,VERIFY,BACKTRACK,STOP; no trajectory or final material");
    println!("policy\taxis\tadmitted\tchanged\trejected\toracle_mismatches");
    for candidate in candidates {
        let report = c3_policy_sensitivity(&candidate.policy)?;
        println!(
            "# policy={}; valid={}; invalid={}; unrecoverable={}",
            candidate.candidate_id,
            report.valid_anchors,
            report.invalid_anchors,
            report.unrecoverable_anchors,
        );
        for (name, axis) in C3_SENSITIVITY_AXES.iter().zip(report.axes) {
            println!(
                "{}\t{name}\t{}\t{}\t{}\t{}",
                candidate.candidate_id,
                axis.admitted,
                axis.action_changes,
                axis.rejected,
                report.reference_action_mismatches,
            );
        }
        for (source, row) in report.transitions.iter().enumerate() {
            println!("# transition_row={source}; counts={row:?}");
        }
    }
    Ok(())
}
