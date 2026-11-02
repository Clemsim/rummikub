use crate::combinaison::Combinaison;
use std::collections::HashSet;

/// Plateau de jeu : l'ensemble des combinaisons posées.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Plateau {
    pub combinaisons: Vec<Combinaison>,
}

impl Plateau {
    /// Pose une nouvelle combinaison sur le plateau
    pub fn place_meld(&mut self, combinaison: Combinaison) {
        self.combinaisons.push(combinaison);
    }

    /// Remplace l'état complet du plateau (fin d'une manipulation)
    pub fn set_melds(&mut self, combinaisons: Vec<Combinaison>) {
        self.combinaisons = combinaisons;
    }

    /// Identifiants de toutes les tuiles posées sur le plateau
    pub fn tile_ids(&self) -> HashSet<u8> {
        self.combinaisons
            .iter()
            .flat_map(|combinaison| combinaison.Tuiles.iter().map(|tuile| tuile.id))
            .collect()
    }

    /// Le plateau n'est valide que si toutes ses combinaisons le sont
    pub fn is_valid(&self) -> bool {
        self.combinaisons
            .iter()
            .all(|combinaison| combinaison.is_valid())
    }
}

#[cfg(test)]
mod tests;
