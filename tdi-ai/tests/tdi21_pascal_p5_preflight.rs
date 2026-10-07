use tdi_ai::tdi21_pascal_p4::Split;
use tdi_ai::tdi21_pascal_p5::{
    Arm, MEASURED_ROUNDS, SEMANTICS, WARMUP_ROUNDS, declared_cell_count, declared_widths,
    median_ns_six, timing_order,
};

#[test]
fn p5_timing_geometry_is_frozen_before_execution() {
    assert_eq!(SEMANTICS, "tdi21-pascal-p5-realized-cost-v1");
    assert_eq!(WARMUP_ROUNDS, 1);
    assert_eq!(MEASURED_ROUNDS, 6);
    assert_eq!(declared_widths(Split::Development), &[10, 12]);
    assert_eq!(declared_widths(Split::Validation), &[15]);
    assert_eq!(declared_cell_count(Split::Development), 108);
    assert_eq!(declared_cell_count(Split::Validation), 54);
}

#[test]
fn p5_rotating_order_is_balanced_over_six_rounds() {
    let expected = [
        [Arm::Pascal, Arm::Direct, Arm::Generic],
        [Arm::Direct, Arm::Generic, Arm::Pascal],
        [Arm::Generic, Arm::Pascal, Arm::Direct],
        [Arm::Pascal, Arm::Direct, Arm::Generic],
        [Arm::Direct, Arm::Generic, Arm::Pascal],
        [Arm::Generic, Arm::Pascal, Arm::Direct],
    ];
    for (round, order) in expected.into_iter().enumerate() {
        assert_eq!(timing_order(round), order);
    }
}

#[test]
fn p5_median_keeps_all_six_samples() {
    assert_eq!(median_ns_six([10, 100, 30, 50, 20, 40]), 35);
    assert_eq!(median_ns_six([1, 1, 1, 9, 9, 9]), 5);
}
