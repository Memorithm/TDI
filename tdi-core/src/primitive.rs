use crate::{
    Action, ExploreError, SignatureError, State, StateError, TdiSignature, TransitionSystem,
    explore,
};

/// Erreurs produites par l'API de primitive prospective TDI.
#[derive(Debug)]
pub enum PrimitiveError<E> {
    /// L'exploration prospective du système a échoué.
    Exploration(ExploreError<E>),
    /// Le rapport d'exploration ne permet pas de construire une signature exacte.
    Signature(SignatureError),
    /// L'intervention demandée ne peut pas être appliquée à l'état initial.
    Intervention(StateError),
}

impl<E> From<ExploreError<E>> for PrimitiveError<E> {
    fn from(error: ExploreError<E>) -> Self {
        Self::Exploration(error)
    }
}

impl<E> From<SignatureError> for PrimitiveError<E> {
    fn from(error: SignatureError) -> Self {
        Self::Signature(error)
    }
}

/// Comparaison exacte entre la signature prospective de référence et celle
/// obtenue après une intervention ponctuelle sur l'état initial.
///
/// Les deux signatures sont évaluées avec la même séquence d'actions futures.
/// Aucune métrique de distance, calibration statistique ou prédiction n'est
/// imposée ici : cette structure constitue la frontière mathématique stable
/// sur laquelle les couches expérimentales peuvent bâtir leurs propres tests.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InterventionComparison {
    reference_initial: State,
    intervened_initial: State,
    reference_signature: TdiSignature,
    intervened_signature: TdiSignature,
}

impl InterventionComparison {
    /// État initial non perturbé.
    #[must_use]
    pub const fn reference_initial(&self) -> State {
        self.reference_initial
    }

    /// État initial obtenu après application de l'intervention.
    #[must_use]
    pub const fn intervened_initial(&self) -> State {
        self.intervened_initial
    }

    /// Signature prospective du système sans intervention.
    #[must_use]
    pub const fn reference_signature(&self) -> &TdiSignature {
        &self.reference_signature
    }

    /// Signature prospective du système après intervention.
    #[must_use]
    pub const fn intervened_signature(&self) -> &TdiSignature {
        &self.intervened_signature
    }
}

/// Calcule la primitive prospective canonique de TDI.
///
/// Pour chaque profondeur `d`, la signature conserve exactement :
///
/// - le nombre d'états distincts accessibles `N_d` ;
/// - le nombre total de chemins prospectifs `P_d` ;
/// - le rapport rationnel exact `rho_d = C_d / P_d`, où `C_d` est le nombre
///   de chemins revenant à l'état initial de l'exploration.
///
/// Mathématiquement, la valeur retournée représente
/// `[(N_d, P_d, rho_d)]_{d=1..H}` pour la séquence d'actions fournie.
pub fn prospective_signature<S>(
    system: &S,
    initial: State,
    actions: &[Action],
) -> Result<TdiSignature, PrimitiveError<S::Error>>
where
    S: TransitionSystem,
{
    let report = explore(system, initial, actions).map_err(PrimitiveError::Exploration)?;
    TdiSignature::from_report(&report).map_err(PrimitiveError::Signature)
}

/// Compare la primitive prospective de référence à celle obtenue après une
/// intervention ponctuelle sur l'état initial.
///
/// L'intervention est appliquée une seule fois, avant l'exploration. Les deux
/// branches utilisent ensuite exactement la même séquence `actions`. Le rapport
/// de retour de chaque signature reste relatif à son propre état initial :
/// `initial` pour la référence et `intervention(initial)` pour la branche
/// interventionnelle.
pub fn compare_intervention<S>(
    system: &S,
    initial: State,
    intervention: Action,
    actions: &[Action],
) -> Result<InterventionComparison, PrimitiveError<S::Error>>
where
    S: TransitionSystem,
{
    let intervened_initial = intervention
        .apply(initial)
        .map_err(PrimitiveError::Intervention)?;

    let reference_signature = prospective_signature(system, initial, actions)?;
    let intervened_signature = prospective_signature(system, intervened_initial, actions)?;

    Ok(InterventionComparison {
        reference_initial: initial,
        intervened_initial,
        reference_signature,
        intervened_signature,
    })
}

#[cfg(test)]
mod tests {
    use crate::{
        Action, ExactRatio, PrimitiveError, State, TableSystem, compare_intervention,
        prospective_signature,
    };

    fn branching_system() -> TableSystem {
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

        system
    }

    #[test]
    fn exposes_the_canonical_prospective_signature() {
        let zero = State::new(0b00, 2).expect("valid state");
        let signature =
            prospective_signature(&branching_system(), zero, &[Action::Noop, Action::Noop])
                .expect("signature succeeds");

        assert_eq!(signature.reachable_profile(), &[2, 1]);
        assert_eq!(signature.path_profile(), &[2, 2]);
        assert_eq!(
            signature.return_profile(),
            &[
                ExactRatio::new(0, 2).expect("valid ratio"),
                ExactRatio::new(2, 2).expect("valid ratio"),
            ]
        );
    }

    #[test]
    fn compares_reference_and_intervened_signatures() {
        let zero = State::new(0b00, 2).expect("valid state");
        let one = State::new(0b01, 2).expect("valid state");

        let comparison = compare_intervention(
            &branching_system(),
            zero,
            Action::Flip { node: 0 },
            &[Action::Noop, Action::Noop],
        )
        .expect("comparison succeeds");

        assert_eq!(comparison.reference_initial(), zero);
        assert_eq!(comparison.intervened_initial(), one);
        assert_eq!(
            comparison.reference_signature().reachable_profile(),
            &[2, 1]
        );
        assert_eq!(
            comparison.intervened_signature().reachable_profile(),
            &[1, 2]
        );
        assert_eq!(
            comparison.intervened_signature().return_profile()[1],
            ExactRatio::new(1, 2).expect("valid ratio")
        );
    }

    #[test]
    fn rejects_an_invalid_intervention_before_exploration() {
        let zero = State::new(0b00, 2).expect("valid state");

        let error = compare_intervention(
            &branching_system(),
            zero,
            Action::Flip { node: 2 },
            &[Action::Noop],
        )
        .expect_err("invalid intervention must fail");

        assert!(matches!(error, PrimitiveError::Intervention(_)));
    }
}
