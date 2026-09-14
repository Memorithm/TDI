//! Fail-closed TDI-21 freeze-template scaffolding.
//!
//! No scientific values are pinned here. Missing fields remain explicit until a
//! later preregistration chooses them under the TDI evidence discipline.

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FreezeTemplate {
    pub sequence_lengths: Option<Vec<usize>>,
    pub state_width_bits: Option<Vec<u8>>,
    pub memory_slots: Option<Vec<usize>>,
    pub task_count: Option<usize>,
    pub development_seeds: Option<Vec<u64>>,
    pub acceptance_rule_id: Option<String>,
    pub development_split_id: Option<String>,
    pub future_holdout_rule_id: Option<String>,
    pub confirmatory_execution_authorized: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FreezeTemplateError {
    UnresolvedField(&'static str),
    EmptyField(&'static str),
    ZeroValue(&'static str),
    StateWidthExceedsBootstrapLimit,
    ConfirmatoryExecutionPremature,
}

impl FreezeTemplate {
    /// Validate that a prospective TDI-21.1 configuration has resolved all
    /// required fields. Passing this method does not freeze or authorize an
    /// experiment; it only proves that no required bootstrap field is absent.
    pub fn validate_resolved(&self) -> Result<(), FreezeTemplateError> {
        if self.confirmatory_execution_authorized {
            return Err(FreezeTemplateError::ConfirmatoryExecutionPremature);
        }

        let sequence_lengths = self
            .sequence_lengths
            .as_ref()
            .ok_or(FreezeTemplateError::UnresolvedField("sequence_lengths"))?;
        let state_width_bits = self
            .state_width_bits
            .as_ref()
            .ok_or(FreezeTemplateError::UnresolvedField("state_width_bits"))?;
        let memory_slots = self
            .memory_slots
            .as_ref()
            .ok_or(FreezeTemplateError::UnresolvedField("memory_slots"))?;
        let task_count = self
            .task_count
            .ok_or(FreezeTemplateError::UnresolvedField("task_count"))?;
        let development_seeds = self
            .development_seeds
            .as_ref()
            .ok_or(FreezeTemplateError::UnresolvedField("development_seeds"))?;
        let acceptance_rule_id = self
            .acceptance_rule_id
            .as_ref()
            .ok_or(FreezeTemplateError::UnresolvedField("acceptance_rule_id"))?;
        let development_split_id = self
            .development_split_id
            .as_ref()
            .ok_or(FreezeTemplateError::UnresolvedField("development_split_id"))?;
        let future_holdout_rule_id =
            self.future_holdout_rule_id
                .as_ref()
                .ok_or(FreezeTemplateError::UnresolvedField(
                    "future_holdout_rule_id",
                ))?;

        if sequence_lengths.is_empty() {
            return Err(FreezeTemplateError::EmptyField("sequence_lengths"));
        }
        if state_width_bits.is_empty() {
            return Err(FreezeTemplateError::EmptyField("state_width_bits"));
        }
        if memory_slots.is_empty() {
            return Err(FreezeTemplateError::EmptyField("memory_slots"));
        }
        if development_seeds.is_empty() {
            return Err(FreezeTemplateError::EmptyField("development_seeds"));
        }
        if acceptance_rule_id.is_empty() {
            return Err(FreezeTemplateError::EmptyField("acceptance_rule_id"));
        }
        if development_split_id.is_empty() {
            return Err(FreezeTemplateError::EmptyField("development_split_id"));
        }
        if future_holdout_rule_id.is_empty() {
            return Err(FreezeTemplateError::EmptyField("future_holdout_rule_id"));
        }

        if sequence_lengths.contains(&0) {
            return Err(FreezeTemplateError::ZeroValue("sequence_lengths"));
        }
        if memory_slots.contains(&0) {
            return Err(FreezeTemplateError::ZeroValue("memory_slots"));
        }
        if task_count == 0 {
            return Err(FreezeTemplateError::ZeroValue("task_count"));
        }
        if state_width_bits.contains(&0) {
            return Err(FreezeTemplateError::ZeroValue("state_width_bits"));
        }
        if state_width_bits.iter().any(|&width| width > 64) {
            return Err(FreezeTemplateError::StateWidthExceedsBootstrapLimit);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn synthetic_resolved_template() -> FreezeTemplate {
        FreezeTemplate {
            // Synthetic unit-test values only; these are not TDI-21 scientific
            // pins and are deliberately confined to the test fixture.
            sequence_lengths: Some(vec![1]),
            state_width_bits: Some(vec![1]),
            memory_slots: Some(vec![1]),
            task_count: Some(1),
            development_seeds: Some(vec![0]),
            acceptance_rule_id: Some("synthetic-test-rule".to_owned()),
            development_split_id: Some("synthetic-test-split".to_owned()),
            future_holdout_rule_id: Some("synthetic-test-holdout-rule".to_owned()),
            confirmatory_execution_authorized: false,
        }
    }

    #[test]
    fn default_template_is_unresolved_and_fails_closed() {
        assert_eq!(
            FreezeTemplate::default().validate_resolved(),
            Err(FreezeTemplateError::UnresolvedField("sequence_lengths"))
        );
    }

    #[test]
    fn fully_resolved_synthetic_template_passes_structure_validation() {
        assert_eq!(synthetic_resolved_template().validate_resolved(), Ok(()));
    }

    #[test]
    fn premature_confirmatory_authorization_is_rejected() {
        let mut template = synthetic_resolved_template();
        template.confirmatory_execution_authorized = true;
        assert_eq!(
            template.validate_resolved(),
            Err(FreezeTemplateError::ConfirmatoryExecutionPremature)
        );
    }

    #[test]
    fn invalid_bootstrap_state_width_is_rejected() {
        let mut template = synthetic_resolved_template();
        template.state_width_bits = Some(vec![65]);
        assert_eq!(
            template.validate_resolved(),
            Err(FreezeTemplateError::StateWidthExceedsBootstrapLimit)
        );
    }
}
