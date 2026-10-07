use tdi_ai::tdi21_pascal_p4::{LoadRegion, Schedule, Split, rounded_load, run_cell};

#[test]
fn p4_harness_matches_frozen_preflight_loads() {
    assert_eq!(rounded_load(10, 4, LoadRegion::Low), Ok(83));
    assert_eq!(rounded_load(12, 16, LoadRegion::Cross), Ok(194));
    assert_eq!(rounded_load(15, 64, LoadRegion::High), Ok(962));
}

#[test]
fn p4_matched_arms_are_exact_on_declared_development_cell() {
    let cell = run_cell(
        Split::Development,
        10,
        4,
        Schedule::Affine,
        LoadRegion::Low,
        1,
    )
    .expect("declared P4 Development cell");
    assert_eq!(cell.mismatches, [0, 0]);
    assert_eq!(cell.checksums[0], cell.checksums[1]);
    assert_eq!(cell.checksums[0], cell.checksums[2]);
    assert_eq!(cell.pairwise_token_comparisons, 0);
}