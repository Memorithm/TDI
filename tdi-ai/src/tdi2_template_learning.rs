//! Evidence-gated template learning primitives for TDI-2.2.
//!
//! This module operates only on bounded structural terms derived from declared
//! observations. Expected labels, evaluator annotations, final data and latency
//! are not induction inputs.

use super::tdi2_incremental_anti_unification::{
    IncrementalAntiUnificationError, incremental_anti_unify,
};
use super::tdi2_structural_terms::StructuralTerm;

/// Versioned identity for a positive-only induced template candidate.
pub const POSITIVE_TEMPLATE_SCHEMA: &str = "tdi2.2-positive-template-v1";

/// A candidate generalized from positive structural examples only.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PositiveTemplateCandidate {
    generalization: StructuralTerm,
    support: usize,
    source_records: Vec<String>,
}

impl PositiveTemplateCandidate {
    /// Generalized structure induced from the positive examples.
    #[must_use]
    pub const fn generalization(&self) -> &StructuralTerm {
        &self.generalization
    }

    /// Number of retained positive evidence rows, including duplicates.
    #[must_use]
    pub const fn support(&self) -> usize {
        self.support
    }

    /// Canonically ordered source structures retained for replay/provenance.
    #[must_use]
    pub fn source_records(&self) -> &[String] {
        &self.source_records
    }

    /// Canonical non-final evidence record.
    #[must_use]
    pub fn canonical_record(&self) -> String {
        use core::fmt::Write as _;
        let generalization = self.generalization.canonical_record();
        let mut output = format!(
            "{POSITIVE_TEMPLATE_SCHEMA};support={};generalization={}:{};sources=[",
            self.support,
            generalization.len(),
            generalization
        );
        for record in &self.source_records {
            write!(output, "{}:{};", record.len(), record).expect("write to string");
        }
        output.push_str("];\n");
        output
    }
}

/// Induce one positive-only template candidate from bounded structural examples.
pub fn induce_positive_template(
    positives: &[StructuralTerm],
) -> Result<PositiveTemplateCandidate, IncrementalAntiUnificationError> {
    let result = incremental_anti_unify(positives)?;
    let mut source_records = positives
        .iter()
        .map(StructuralTerm::canonical_record)
        .collect::<Vec<_>>();
    source_records.sort_unstable();
    Ok(PositiveTemplateCandidate {
        generalization: result.generalization().clone(),
        support: positives.len(),
        source_records,
    })
}

#[cfg(test)]
mod positive_tests {
    use super::*;
    use crate::experimental::tdi2_structural_terms::{StructuralSymbol, StructuralTerm};

    fn atom(id: u32) -> StructuralTerm {
        StructuralTerm::atom(StructuralSymbol::constructor(10_000 + id))
    }

    fn app(id: u32, arguments: Vec<StructuralTerm>) -> StructuralTerm {
        StructuralTerm::application(StructuralSymbol::constructor(id), arguments).expect("term")
    }

    #[test]
    fn positive_candidate_generalizes_surface_difference_and_keeps_support() {
        let positives = [
            app(1, vec![atom(10), atom(20)]),
            app(1, vec![atom(11), atom(20)]),
            app(1, vec![atom(12), atom(20)]),
        ];
        let candidate = induce_positive_template(&positives).expect("candidate");
        assert_eq!(candidate.support(), 3);
        assert!(
            candidate
                .generalization()
                .canonical_record()
                .contains("v0;")
        );
        assert_eq!(candidate.source_records().len(), 3);
    }

    #[test]
    fn empty_positive_set_fails_closed() {
        assert_eq!(
            induce_positive_template(&[]),
            Err(IncrementalAntiUnificationError::EmptyInput)
        );
    }
}
