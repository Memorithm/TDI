use crate::ReachabilityReport;

/// Rapport rationnel exact, sans conversion flottante.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ExactRatio {
    numerator: u128,
    denominator: u128,
}

/// Erreurs de construction d'une signature TDI.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SignatureError {
    EmptyReport,
    MissingLayer { depth: usize },
    ZeroPathCount { depth: usize },
}

/// Signature prospective minimale de TDI-1.
///
/// Elle conserve séparément :
/// - le nombre d'états accessibles à chaque profondeur ;
/// - le nombre de chemins admissibles ;
/// - la proportion exacte des chemins revenant à l'état initial.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TdiSignature {
    reachable_profile: Vec<usize>,
    path_profile: Vec<u128>,
    return_profile: Vec<ExactRatio>,
}

impl ExactRatio {
    /// Construit et réduit une fraction exacte.
    #[must_use]
    pub fn new(numerator: u128, denominator: u128) -> Option<Self> {
        if denominator == 0 {
            return None;
        }

        let divisor = greatest_common_divisor(numerator, denominator);

        Some(Self {
            numerator: numerator / divisor,
            denominator: denominator / divisor,
        })
    }

    #[must_use]
    pub const fn numerator(self) -> u128 {
        self.numerator
    }

    #[must_use]
    pub const fn denominator(self) -> u128 {
        self.denominator
    }

    /// Conversion destinée uniquement à l'affichage et aux modèles statistiques.
    #[must_use]
    pub fn as_f64(self) -> f64 {
        self.numerator as f64 / self.denominator as f64
    }
}

impl TdiSignature {
    /// Extrait une signature exacte depuis un rapport d'exploration.
    pub fn from_report(report: &ReachabilityReport) -> Result<Self, SignatureError> {
        if report.horizon() == 0 {
            return Err(SignatureError::EmptyReport);
        }

        let mut reachable_profile = Vec::with_capacity(report.horizon());
        let mut path_profile = Vec::with_capacity(report.horizon());
        let mut return_profile = Vec::with_capacity(report.horizon());

        for depth in 1..=report.horizon() {
            let reachable = report
                .reachable_count(depth)
                .ok_or(SignatureError::MissingLayer { depth })?;

            let total_paths = report
                .path_count(depth)
                .ok_or(SignatureError::MissingLayer { depth })?;

            if total_paths == 0 {
                return Err(SignatureError::ZeroPathCount { depth });
            }

            let returned_paths = report
                .return_path_count(depth)
                .ok_or(SignatureError::MissingLayer { depth })?;

            let return_ratio = ExactRatio::new(returned_paths, total_paths)
                .ok_or(SignatureError::ZeroPathCount { depth })?;

            reachable_profile.push(reachable);
            path_profile.push(total_paths);
            return_profile.push(return_ratio);
        }

        Ok(Self {
            reachable_profile,
            path_profile,
            return_profile,
        })
    }

    #[must_use]
    pub fn horizon(&self) -> usize {
        self.reachable_profile.len()
    }

    #[must_use]
    pub fn reachable_profile(&self) -> &[usize] {
        &self.reachable_profile
    }

    #[must_use]
    pub fn path_profile(&self) -> &[u128] {
        &self.path_profile
    }

    #[must_use]
    pub fn return_profile(&self) -> &[ExactRatio] {
        &self.return_profile
    }
}

const fn greatest_common_divisor(mut left: u128, mut right: u128) -> u128 {
    while right != 0 {
        let remainder = left % right;
        left = right;
        right = remainder;
    }

    if left == 0 { 1 } else { left }
}

#[cfg(test)]
mod tests {
    use crate::{Action, ExactRatio, SignatureError, State, TableSystem, TdiSignature, explore};

    #[test]
    fn reduces_exact_ratios() {
        let ratio = ExactRatio::new(6, 8).expect("non-zero denominator");

        assert_eq!(ratio.numerator(), 3);
        assert_eq!(ratio.denominator(), 4);
        assert_eq!(ratio.as_f64(), 0.75);
    }

    #[test]
    fn rejects_zero_denominator() {
        assert_eq!(ExactRatio::new(1, 0), None);
    }

    #[test]
    fn extracts_branching_and_return_profiles() {
        let zero = State::new(0b00, 2).expect("valid state");
        let one = State::new(0b01, 2).expect("valid state");
        let two = State::new(0b10, 2).expect("valid state");

        let mut system = TableSystem::new(2).expect("valid system");
        system
            .insert(zero, Action::Noop, vec![one, two])
            .expect("valid transition");
        system
            .insert(one, Action::Noop, vec![zero])
            .expect("valid transition");
        system
            .insert(two, Action::Noop, vec![zero])
            .expect("valid transition");

        let report =
            explore(&system, zero, &[Action::Noop, Action::Noop]).expect("exploration succeeds");

        let signature = TdiSignature::from_report(&report).expect("valid signature");

        assert_eq!(signature.horizon(), 2);
        assert_eq!(signature.reachable_profile(), &[2, 1]);
        assert_eq!(signature.path_profile(), &[2, 2]);
        assert_eq!(
            signature.return_profile(),
            &[
                ExactRatio::new(0, 2).expect("valid ratio"),
                ExactRatio::new(2, 2).expect("valid ratio")
            ]
        );
    }

    #[test]
    fn rejects_an_empty_report() {
        let state = State::new(0, 1).expect("valid state");
        let system = TableSystem::new(1).expect("valid system");

        let report = explore(&system, state, &[]).expect("zero-horizon exploration succeeds");

        assert_eq!(
            TdiSignature::from_report(&report),
            Err(SignatureError::EmptyReport)
        );
    }
}
