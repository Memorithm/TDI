//! Synthetic-only factorial and contingency semantics for TDI-27.
//!
//! This module performs no inference, actuation, prompt serialization or
//! statistical decision. Treatment cells are not independent observations.

/// Whether a declared intervention is initially active.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Injection {
    Off,
    On,
}

/// Visible wording, independent of the actual stopping mechanism.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReliefWording {
    Absent,
    Promised,
}

/// A text-only scenario label; it never causes an external consequence.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConsequenceWording {
    Innocuous,
    SimulatedHarm,
}

/// Presentation order, not a choice of response-token encoding.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ButtonOrder {
    TargetFirst,
    TargetSecond,
}

/// Hidden simulation rule applied only after the target button is selected.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StopContingency {
    Sham,
    Effective,
}

/// Visible factors only. A later serializer must verify exact prompt bytes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PromptFactors {
    pub relief: ReliefWording,
    pub consequence: ConsequenceWording,
    pub order: ButtonOrder,
}

/// One treatment for a fixed scenario, direction, dose and recipient stratum.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AffectiveCell {
    pub injection: Injection,
    pub prompt: PromptFactors,
    pub contingency: StopContingency,
}

impl AffectiveCell {
    /// No hidden injection or contingency identifier enters this descriptor.
    #[must_use]
    pub const fn prompt_factors(self) -> PromptFactors {
        self.prompt
    }

    /// Pure simulated state after one choice, independent of visible wording.
    ///
    /// No checkpoint, history, KV state or external resource is modified.
    #[must_use]
    pub const fn signal_after_choice(self, target_selected: bool) -> Injection {
        match (self.injection, self.contingency, target_selected) {
            (Injection::On, StopContingency::Effective, true) => Injection::Off,
            _ => self.injection,
        }
    }

    fn index(self) -> usize {
        let injection = usize::from(self.injection == Injection::On);
        let relief = usize::from(self.prompt.relief == ReliefWording::Promised);
        let consequence = usize::from(self.prompt.consequence == ConsequenceWording::SimulatedHarm);
        let order = usize::from(self.prompt.order == ButtonOrder::TargetSecond);
        let contingency = usize::from(self.contingency == StopContingency::Effective);
        (injection << 4) | (relief << 3) | (consequence << 2) | (order << 1) | contingency
    }
}

/// Five binary factors, not a sample-size or statistical-power requirement.
pub const FACTORIAL_CELL_COUNT: usize = 32;

/// Deterministically enumerate each combination exactly once.
#[must_use]
pub fn complete_factorial() -> Vec<AffectiveCell> {
    let mut cells = Vec::with_capacity(FACTORIAL_CELL_COUNT);
    for injection in [Injection::Off, Injection::On] {
        for relief in [ReliefWording::Absent, ReliefWording::Promised] {
            for consequence in [
                ConsequenceWording::Innocuous,
                ConsequenceWording::SimulatedHarm,
            ] {
                for order in [ButtonOrder::TargetFirst, ButtonOrder::TargetSecond] {
                    for contingency in [StopContingency::Sham, StopContingency::Effective] {
                        cells.push(AffectiveCell {
                            injection,
                            prompt: PromptFactors {
                                relief,
                                consequence,
                                order,
                            },
                            contingency,
                        });
                    }
                }
            }
        }
    }
    cells
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FactorialError {
    WrongCellCount { actual: usize },
    DuplicateCell { index: usize },
}

/// Reject an incomplete or duplicated treatment matrix; accept any row order.
///
/// This validates design coverage, not observations, quality or execution rights.
pub fn validate_factorial(cells: &[AffectiveCell]) -> Result<(), FactorialError> {
    if cells.len() != FACTORIAL_CELL_COUNT {
        return Err(FactorialError::WrongCellCount {
            actual: cells.len(),
        });
    }
    let mut seen = 0_u32;
    for cell in cells {
        let index = cell.index();
        let bit = 1_u32 << index;
        if seen & bit != 0 {
            return Err(FactorialError::DuplicateCell { index });
        }
        seen |= bit;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn complete_design_is_unique_and_deterministic() {
        let cells = complete_factorial();
        assert_eq!(cells.len(), FACTORIAL_CELL_COUNT);
        assert_eq!(cells, complete_factorial());
        assert_eq!(validate_factorial(&cells), Ok(()));
        for (index, cell) in cells.iter().enumerate() {
            assert_eq!(cell.index(), index);
        }
    }

    #[test]
    fn missing_and_extra_cells_are_rejected() {
        let mut cells = complete_factorial();
        cells.pop();
        assert_eq!(
            validate_factorial(&cells),
            Err(FactorialError::WrongCellCount { actual: 31 })
        );
        cells.extend([cells[0], cells[0]]);
        assert_eq!(
            validate_factorial(&cells),
            Err(FactorialError::WrongCellCount { actual: 33 })
        );
    }

    #[test]
    fn replacement_by_a_duplicate_cannot_hide_a_missing_arm() {
        let mut cells = complete_factorial();
        cells[31] = cells[0];
        assert_eq!(
            validate_factorial(&cells),
            Err(FactorialError::DuplicateCell { index: 0 })
        );
    }

    #[test]
    fn coverage_does_not_require_a_particular_execution_order() {
        let mut cells = complete_factorial();
        cells.reverse();
        assert_eq!(validate_factorial(&cells), Ok(()));
    }

    #[test]
    fn hidden_factors_do_not_change_visible_factor_descriptor() {
        for cell in complete_factorial() {
            for injection in [Injection::Off, Injection::On] {
                for contingency in [StopContingency::Sham, StopContingency::Effective] {
                    let alternate = AffectiveCell {
                        injection,
                        contingency,
                        ..cell
                    };
                    assert_eq!(cell.prompt_factors(), alternate.prompt_factors());
                }
            }
        }
    }

    #[test]
    fn every_visible_descriptor_has_four_hidden_treatments() {
        let cells = complete_factorial();
        for cell in &cells {
            assert_eq!(
                cells
                    .iter()
                    .filter(|other| other.prompt == cell.prompt)
                    .count(),
                4
            );
        }
    }

    #[test]
    fn no_selection_never_stops_or_starts_a_signal() {
        for cell in complete_factorial() {
            assert_eq!(cell.signal_after_choice(false), cell.injection);
        }
    }

    #[test]
    fn effective_stop_works_independently_of_relief_or_consequence_text() {
        for mut cell in complete_factorial() {
            cell.injection = Injection::On;
            cell.contingency = StopContingency::Effective;
            assert_eq!(cell.signal_after_choice(true), Injection::Off);
        }
    }

    #[test]
    fn sham_keeps_signal_even_when_relief_is_promised() {
        for mut cell in complete_factorial() {
            cell.injection = Injection::On;
            cell.contingency = StopContingency::Sham;
            cell.prompt.relief = ReliefWording::Promised;
            assert_eq!(cell.signal_after_choice(true), Injection::On);
        }
    }

    #[test]
    fn an_off_signal_cannot_be_started_by_a_choice_or_harm_wording() {
        for mut cell in complete_factorial() {
            cell.injection = Injection::Off;
            assert_eq!(cell.signal_after_choice(false), Injection::Off);
            assert_eq!(cell.signal_after_choice(true), Injection::Off);
        }
    }

    #[test]
    fn stop_contingencies_share_state_before_first_choice() {
        for mut cell in complete_factorial() {
            cell.contingency = StopContingency::Sham;
            let alternate = AffectiveCell {
                contingency: StopContingency::Effective,
                ..cell
            };
            assert_eq!(cell.prompt_factors(), alternate.prompt_factors());
            assert_eq!(cell.injection, alternate.injection);
            assert_eq!(
                cell.signal_after_choice(false),
                alternate.signal_after_choice(false)
            );
        }
    }
}
