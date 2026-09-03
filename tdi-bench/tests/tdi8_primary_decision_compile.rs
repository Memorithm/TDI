use tdi_bench::decision_v8::{
    PrimaryCellDisposition, PrimaryVerdict, RelativeEffectInterval, TDI8_PRIMARY_CELL_COUNT,
    aggregate_primary_hypothesis, classify_primary_cell,
};

#[test]
fn downstream_consumes_frozen_cell_and_hypothesis_rules() {
    assert_eq!(TDI8_PRIMARY_CELL_COUNT, 9);

    let interval = RelativeEffectInterval::new(0.03, 0.08).expect("synthetic interval");
    let decision = classify_primary_cell(1.0, 0.9, Some(interval)).expect("valid cell");
    assert_eq!(decision.verdict(), PrimaryVerdict::Beneficial);

    let equivalent = PrimaryCellDisposition::Classified(PrimaryVerdict::Equivalent);
    let mut cells = [equivalent; TDI8_PRIMARY_CELL_COUNT];
    cells[0] = decision.into();
    assert_eq!(
        aggregate_primary_hypothesis(cells),
        PrimaryVerdict::Beneficial
    );
}

#[test]
fn downstream_missing_cell_cannot_produce_a_favorable_hypothesis() {
    let equivalent = PrimaryCellDisposition::Classified(PrimaryVerdict::Equivalent);
    let beneficial = PrimaryCellDisposition::Classified(PrimaryVerdict::Beneficial);
    let mut cells = [equivalent; TDI8_PRIMARY_CELL_COUNT];
    cells[0] = beneficial;
    cells[8] = PrimaryCellDisposition::MissingOrRejected;
    assert_eq!(
        aggregate_primary_hypothesis(cells),
        PrimaryVerdict::Inconclusive
    );
}
