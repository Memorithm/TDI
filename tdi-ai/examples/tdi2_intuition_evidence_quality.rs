//! Development diagnostic separating experience quantity from empirical quality.
use tdi_ai::experimental::tdi2_intuition::TemplateId;
use tdi_ai::experimental::tdi2_intuition_experience::motif_experience_store_with_evidence;
use tdi_ai::experimental::tdi2_intuition_weight::ExperienceWeightPolicy;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let good = motif_experience_store_with_evidence(8, 2);
    let poor = motif_experience_store_with_evidence(2, 8);
    let policy = ExperienceWeightPolicy::default();
    let good_evidence = good.get(TemplateId::new(1)).expect("template").evidence();
    let poor_evidence = poor.get(TemplateId::new(1)).expect("template").evidence();
    assert_eq!(good_evidence.support(), poor_evidence.support());
    let good_weight = policy.weight(good_evidence)?;
    let poor_weight = policy.weight(poor_evidence)?;
    assert!(good_weight > poor_weight);
    println!(
        "tdi2.1-evidence-quality-v1;support={};good_successes=8;poor_successes=2;good_weight_bits={:016x};poor_weight_bits={:016x}",
        good_evidence.support(),
        good_weight.to_bits(),
        poor_weight.to_bits()
    );
    Ok(())
}
