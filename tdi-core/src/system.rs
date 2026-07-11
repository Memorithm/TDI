use crate::{Action, State};

/// Contrat minimal d'un système de transition fini.
///
/// Plusieurs successeurs permettent de représenter une dynamique ramifiée.
pub trait TransitionSystem {
    type Error;

    /// Nombre de constituants booléens du système.
    fn width(&self) -> u8;

    /// Calcule tous les états suivants admissibles.
    fn successors(&self, state: State, action: Action) -> Result<Vec<State>, Self::Error>;
}
