//! Non-final Pascal P4 preregistration preflight only.

const WIDTHS: [u8; 3] = [10, 12, 15];
const DENSITIES: [usize; 3] = [4, 16, 64];

fn gcd(mut a: usize, mut b: usize) -> usize {
    while b != 0 {
        let r = a % b;
        a = b;
        b = r;
    }
    a
}

fn masks(n: u8, gate: usize, density: usize) -> Vec<usize> {
    let modulus = (1usize << n) - 1;
    (0..density)
        .map(|term| 1 + ((257 * gate + 71 * term) % modulus))
        .collect()
}

fn rounded_load(n: u8, density: usize, numerator: u128, denominator: u128) -> usize {
    let top = u128::from(n) * (1u128 << n) * numerator;
    let bottom = 2u128 * (8u128 * density as u128 - 1) * denominator;
    usize::try_from((top + bottom / 2) / bottom)
        .expect("bounded P4 load")
        .max(1)
}

#[test]
fn p4_declared_generators_are_unique() {
    for n in WIDTHS {
        let modulus = (1usize << n) - 1;
        assert_eq!(gcd(71, modulus), 1);
        for density in DENSITIES {
            for gate in 0..8 {
                let mut values = masks(n, gate, density);
                values.sort_unstable();
                values.dedup();
                assert_eq!(values.len(), density);
            }
        }
    }
}

#[test]
fn p4_crossover_targets_are_exactly_frozen() {
    let expected = [
        (10u8, 4usize, [83usize, 165, 330]),
        (10, 16, [20, 40, 81]),
        (10, 64, [5, 10, 20]),
        (12, 4, [396, 793, 1586]),
        (12, 16, [97, 194, 387]),
        (12, 64, [24, 48, 96]),
        (15, 4, [3964, 7928, 15855]),
        (15, 16, [968, 1935, 3870]),
        (15, 64, [240, 481, 962]),
    ];
    for (n, density, target) in expected {
        assert_eq!(
            [
                rounded_load(n, density, 1, 2),
                rounded_load(n, density, 1, 1),
                rounded_load(n, density, 2, 1),
            ],
            target
        );
        assert!(target.into_iter().all(|load| load <= 1usize << n));
    }
}

#[test]
fn p4_geometry_is_frozen_before_execution() {
    let cells_per_width = 3usize * 3 * 2 * 3;
    assert_eq!(cells_per_width, 54);
    assert_eq!(2 * cells_per_width, 108);
    assert_eq!(cells_per_width, 54);
    assert_eq!(3 * cells_per_width, 162);
}

#[test]
fn p4_lifecycle_decompositions_are_valid() {
    for n in WIDTHS {
        let k = 1usize << n;
        for density in DENSITIES {
            for (num, den) in [(1u128, 2u128), (1, 1), (2, 1)] {
                let load = rounded_load(n, density, num, den);
                let q1 = load;
                let q8 = (load + 4) / 8;
                assert!((1..=k).contains(&q1));
                assert!((1..=k).contains(&q8));
            }
        }
    }
}
