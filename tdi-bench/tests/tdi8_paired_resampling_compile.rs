use tdi_bench::paired_resampling_v8::{
    PairedDeficitObservation, PairedResamplingPlan, bonferroni_cell_alpha,
    paired_bootstrap_replicates,
};

#[test]
fn public_paired_resampling_surface_compiles_and_preserves_accounting() {
    let observations = [
        PairedDeficitObservation::new(1.0, 0.75).expect("pair"),
        PairedDeficitObservation::new(2.0, 1.0).expect("pair"),
        PairedDeficitObservation::new(3.0, 1.5).expect("pair"),
    ];
    let plan = PairedResamplingPlan::new(32, 0x5444_4938_5052_0001).expect("plan");
    let output = paired_bootstrap_replicates(&observations, plan).expect("bootstrap");

    assert_eq!(bonferroni_cell_alpha(), 0.05 / 9.0);
    assert_eq!(output.requested_replicates(), 32);
    assert_eq!(output.seed(), plan.seed());
    output
        .validate_complete_accounting()
        .expect("all replicates accounted");
    assert!(output.point().relative_effect().is_some());
}
