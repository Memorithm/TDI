//! P3 protocol preflight only.
//!
//! This test does not execute any Pascal P3 scientific Development/Validation
//! cell. It verifies the prospectively frozen generator geometry and primitive
//! work-accounting arithmetic before scientific execution.

const P3_WIDTHS: [u8; 4] = [9, 11, 13, 14];
const P3_DENSITIES: [usize; 3] = [4, 32, 256];
const P3_GATE_COUNT: usize = 8;

fn gcd(mut left: usize, mut right: usize) -> usize {
    while right != 0 {
        let remainder = left % right;
        left = right;
        right = remainder;
    }
    left
}

fn p3_masks(variable_count: u8, gate: usize, density: usize) -> Vec<usize> {
    let modulus = (1usize << variable_count) - 1;
    (0..density)
        .map(|term| 1 + ((257usize * gate + 71usize * term) % modulus))
        .collect()
}

fn work_pascal(pascal_zeta_xors: u64, query_lookups: u64) -> Option<u64> {
    pascal_zeta_xors.checked_add(query_lookups)
}

fn work_generic(generic_term_tests: u64, query_lookups: u64) -> Option<u64> {
    generic_term_tests.checked_add(query_lookups)
}

fn signed_delta(control_work: u64, pascal_work: u64) -> i128 {
    i128::from(control_work) - i128::from(pascal_work)
}

#[test]
fn p3_generator_is_coprime_and_unique_for_every_declared_cell_shape() {
    for variable_count in P3_WIDTHS {
        let modulus = (1usize << variable_count) - 1;
        assert_eq!(
            gcd(71, modulus),
            1,
            "P3 step must be coprime with 2^n-1 for n={variable_count}"
        );

        for density in P3_DENSITIES {
            assert!(density < modulus);
            for gate in 0..P3_GATE_COUNT {
                let mut masks = p3_masks(variable_count, gate, density);
                assert!(masks.iter().all(|&mask| mask > 0 && mask <= modulus));
                masks.sort_unstable();
                masks.dedup();
                assert_eq!(
                    masks.len(),
                    density,
                    "duplicate P3 mask for n={variable_count}, density={density}, gate={gate}"
                );
            }
        }
    }
}

#[test]
fn p3_frozen_geometry_is_exactly_216_nonfinal_cells() {
    let widths_per_split = 2usize;
    let densities = P3_DENSITIES.len();
    let schedules = 3usize;
    let query_loads = 3usize;
    let reuse_values = 2usize;

    let cells_per_split =
        widths_per_split * densities * schedules * query_loads * reuse_values;
    assert_eq!(cells_per_split, 108);
    assert_eq!(2 * cells_per_split, 216);
}

#[test]
fn p3_work_accounting_preserves_positive_null_and_negative_deltas() {
    let pascal = work_pascal(90, 10).expect("bounded P3 Pascal work");
    let direct_negative = 80u64;
    let direct_null = 100u64;
    let direct_positive = 120u64;
    let generic = work_generic(110, 10).expect("bounded P3 generic work");

    assert_eq!(pascal, 100);
    assert_eq!(signed_delta(direct_negative, pascal), -20);
    assert_eq!(signed_delta(direct_null, pascal), 0);
    assert_eq!(signed_delta(direct_positive, pascal), 20);
    assert_eq!(signed_delta(generic, pascal), 20);
}

#[test]
fn p3_work_sums_fail_closed_on_overflow() {
    assert_eq!(work_pascal(u64::MAX, 1), None);
    assert_eq!(work_generic(u64::MAX, 1), None);
}
