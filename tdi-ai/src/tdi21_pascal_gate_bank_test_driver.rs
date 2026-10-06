mod tdi21_pascal_gate_bank;

use tdi21_pascal_gate_bank::{
    PascalP1Decision, PascalP1Error, PascalP1Schedule, PascalP1Split,
    evaluate_pascal_p1_gates, generate_pascal_p1_bank, generic_p1_materialize,
    pascal_p1_decision, pascal_p1_materialize, pascal_p1_query_addresses,
    run_pascal_p1_cell, run_pascal_p1_matrix, PASCAL_P1_GATE_COUNT,
    PASCAL_P1_SEMANTICS,
};

#[test]
fn frozen_generator_is_unique_and_deterministic() {
    let first = generate_pascal_p1_bank(12, 256).unwrap();
    let second = generate_pascal_p1_bank(12, 256).unwrap();
    assert_eq!(first, second);
    assert_eq!(first.domain_size(), 4096);
    assert_eq!(first.density(), 256);
    assert_eq!(first.constants(), 0b1010_1010);
    for gate in 0..PASCAL_P1_GATE_COUNT {
        let mut terms = first.terms(gate).to_vec();
        terms.sort_unstable();
        terms.dedup();
        assert_eq!(terms.len(), 256);
        assert!(terms.iter().all(|&mask| mask > 0 && mask < 4096));
    }
}

#[test]
fn pascal_matches_independent_generic_materialization() {
    for (width, density) in [(6, 4), (6, 32), (8, 32), (8, 255)] {
        let bank = generate_pascal_p1_bank(width, density).unwrap();
        let (pascal, xor_count) = pascal_p1_materialize(&bank).unwrap();
        let (generic, term_tests) = generic_p1_materialize(&bank).unwrap();
        assert_eq!(pascal, generic);
        assert_eq!(
            xor_count,
            u64::from(width) * bank.domain_size() as u64 / 2
        );
        assert_eq!(
            term_tests,
            bank.domain_size() as u64 * density as u64 * PASCAL_P1_GATE_COUNT as u64
        );
    }
}

#[test]
fn frozen_query_schedules_preserve_declared_extremes() {
    let affine = pascal_p1_query_addresses(8, PascalP1Schedule::Affine, 256).unwrap();
    let mut unique = affine.clone();
    unique.sort_unstable();
    unique.dedup();
    assert_eq!(unique.len(), 256);

    let low = pascal_p1_query_addresses(8, PascalP1Schedule::LowWeightFirst, 16).unwrap();
    let high =
        pascal_p1_query_addresses(8, PascalP1Schedule::HighWeightFirst, 16).unwrap();
    assert_eq!(low[0], 0);
    assert_eq!(high[0], 255);
    assert!(
        low.windows(2)
            .all(|pair| pair[0].count_ones() <= pair[1].count_ones())
    );
    assert!(
        high.windows(2)
            .all(|pair| pair[0].count_ones() >= pair[1].count_ones())
    );
}

#[test]
fn split_boundary_is_fail_closed() {
    assert_eq!(
        run_pascal_p1_cell(
            PascalP1Split::Development,
            10,
            32,
            PascalP1Schedule::Affine,
            16,
            1,
        ),
        Err(PascalP1Error::SplitWidthMismatch)
    );
    assert_eq!(
        run_pascal_p1_cell(
            PascalP1Split::Validation,
            8,
            32,
            PascalP1Schedule::Affine,
            16,
            1,
        ),
        Err(PascalP1Error::SplitWidthMismatch)
    );
}

#[test]
fn full_matrix_retains_positive_and_negative_regions() {
    let cells = run_pascal_p1_matrix().unwrap();
    assert_eq!(cells.len(), 198);
    let gates = evaluate_pascal_p1_gates(&cells);
    assert!(gates.g1_exactness);
    assert!(gates.g2_representation_accounting);
    assert!(gates.g3_tdi21_prohibition);
    assert!(gates.g4_development_structural_utility);
    assert!(gates.g5_validation_generalization);
    assert!(gates.g6_sparse_boundary_retained);
    assert_eq!(
        pascal_p1_decision(gates),
        PascalP1Decision::RetainNonfinalPascalGateBankUtility
    );
    assert!(cells.iter().any(|cell| {
        cell.split == PascalP1Split::Validation
            && cell.density == 4
            && cell.query_load == 16
            && cell.reuse == 1
            && cell.direct_term_tests < cell.pascal_zeta_xors
    }));
}

#[test]
fn emit_deterministic_p1_summary() {
    let cells = run_pascal_p1_matrix().unwrap();
    let gates = evaluate_pascal_p1_gates(&cells);
    let decision = pascal_p1_decision(gates);
    let sparse_boundary_cells = cells
        .iter()
        .filter(|cell| cell.direct_term_tests < cell.pascal_zeta_xors)
        .count();
    println!(
        "PASCAL_P1_SUMMARY,semantics={},cells={},g1={},g2={},g3={},g4={},g5={},g6={},decision={:?},sparse_boundary_cells={}",
        PASCAL_P1_SEMANTICS,
        cells.len(),
        u8::from(gates.g1_exactness),
        u8::from(gates.g2_representation_accounting),
        u8::from(gates.g3_tdi21_prohibition),
        u8::from(gates.g4_development_structural_utility),
        u8::from(gates.g5_validation_generalization),
        u8::from(gates.g6_sparse_boundary_retained),
        decision,
        sparse_boundary_cells,
    );
    for cell in cells.iter().filter(|cell| {
        cell.schedule == PascalP1Schedule::Affine
            && cell.query_load == cell.domain_size
            && cell.reuse == 1
            && cell.density >= 32
    }) {
        println!(
            "PASCAL_P1_FULL,split={:?},n={},k={},density={},pascal_xors={},direct_terms={},generic_terms={},checksum={:016x}",
            cell.split,
            cell.variable_count,
            cell.domain_size,
            cell.density,
            cell.pascal_zeta_xors,
            cell.direct_term_tests,
            cell.generic_term_tests,
            cell.pascal_checksum,
        );
    }
    for cell in cells.iter().filter(|cell| {
        cell.schedule == PascalP1Schedule::Affine
            && cell.density == 4
            && cell.query_load == 16
            && cell.reuse == 1
    }) {
        println!(
            "PASCAL_P1_SPARSE,split={:?},n={},k={},pascal_xors={},direct_terms={},generic_terms={},direct_lower={}",
            cell.split,
            cell.variable_count,
            cell.domain_size,
            cell.pascal_zeta_xors,
            cell.direct_term_tests,
            cell.generic_term_tests,
            u8::from(cell.direct_term_tests < cell.pascal_zeta_xors),
        );
    }
}
