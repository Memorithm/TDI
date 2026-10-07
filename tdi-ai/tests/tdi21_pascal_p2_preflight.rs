//! TDI-21 Pascal P2 preregistration feasibility preflight.
//!
//! This is not a confirmatory/final experiment. It inspects only the frozen
//! Development/Validation geometry and the P1 generator contract that P2 says
//! it inherits unchanged. It deliberately preserves blockers instead of
//! silently changing the preregistered generator.

use std::collections::BTreeSet;

const GATE_COUNT: usize = 8;
const CELLS_PER_WIDTH: usize = 54;
const DEVELOPMENT_CELL_COUNT: usize = 108;
const FULL_NONFINAL_CELL_COUNT: usize = 216;

fn inherited_p1_masks(variable_count: u8, density: usize, gate: usize) -> Vec<usize> {
    let domain_size = 1usize << variable_count;
    (0..density)
        .map(|term| 1 + ((257usize * gate + 73usize * term) % (domain_size - 1)))
        .collect()
}

fn generator_has_unique_terms(variable_count: u8, density: usize) -> bool {
    (0..GATE_COUNT).all(|gate| {
        let masks = inherited_p1_masks(variable_count, density, gate);
        masks.iter().copied().collect::<BTreeSet<_>>().len() == density
    })
}

fn cells_for_density() -> usize {
    // 3 schedules x 3 query loads x 2 reuse values.
    3 * 3 * 2
}

#[test]
fn p2_frozen_geometry_is_exactly_216_nonfinal_cells() {
    let development_widths = [9u8, 11u8];
    let validation_widths = [13u8, 14u8];
    let densities = [4usize, 32usize, 256usize];

    let development_cells = development_widths.len() * densities.len() * cells_for_density();
    let validation_cells = validation_widths.len() * densities.len() * cells_for_density();

    assert_eq!(development_cells, DEVELOPMENT_CELL_COUNT);
    assert_eq!(
        development_cells + validation_cells,
        FULL_NONFINAL_CELL_COUNT
    );
    assert_eq!(CELLS_PER_WIDTH, densities.len() * cells_for_density());
}

#[test]
fn p2_development_preserves_inherited_generator_collision_as_blocker() {
    let mut eligible_cells = 0usize;
    let mut blocked_cells = 0usize;

    for variable_count in [9u8, 11u8] {
        for density in [4usize, 32usize, 256usize] {
            let cell_count = cells_for_density();
            if generator_has_unique_terms(variable_count, density) {
                eligible_cells += cell_count;
            } else {
                blocked_cells += cell_count;
            }
        }
    }

    // n=9 has generator period 7 because gcd(73, 2^9-1)=73.
    // Density 32 and 256 therefore collide. Each density spans 18 cells.
    assert_eq!(blocked_cells, 36);
    assert_eq!(eligible_cells, 72);
    assert_eq!(eligible_cells + blocked_cells, DEVELOPMENT_CELL_COUNT);

    assert!(generator_has_unique_terms(9, 4));
    assert!(!generator_has_unique_terms(9, 32));
    assert!(!generator_has_unique_terms(9, 256));
    assert!(generator_has_unique_terms(11, 4));
    assert!(generator_has_unique_terms(11, 32));
    assert!(generator_has_unique_terms(11, 256));

    println!(
        "PASCAL_P2_DEV_PREFLIGHT,total={},eligible={},blocked={},reason=inherited_generator_duplicate_masks",
        DEVELOPMENT_CELL_COUNT, eligible_cells, blocked_cells
    );
}

#[test]
fn p2_validation_geometry_is_not_executed_by_this_preflight() {
    // The Development preflight above already exposes a preregistered
    // implementation/protocol blocker. Validation scientific cells are
    // intentionally not executed here. This assertion only freezes their count.
    let validation_cells = 2 * 3 * cells_for_density();
    assert_eq!(validation_cells, 108);
}
