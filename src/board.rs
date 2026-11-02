use crate::meld::Meld;
use std::collections::HashSet;

/// Plateau de jeu : l'ensemble des combinaisons posées.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Board {
    pub melds: Vec<Meld>,
}

impl Board {
    /// Pose une nouvelle combinaison sur le plateau
    pub fn place_meld(&mut self, meld: Meld) {
        self.melds.push(meld);
    }

    /// Remplace l'état complet du plateau (fin d'une manipulation)
    pub fn set_melds(&mut self, melds: Vec<Meld>) {
        self.melds = melds;
    }

    /// Identifiants de toutes les tuiles posées sur le plateau
    pub fn tile_ids(&self) -> HashSet<u8> {
        self.melds
            .iter()
            .flat_map(|meld| meld.Tuiles.iter().map(|tuile| tuile.id))
            .collect()
    }

    /// Le plateau n'est valide que si toutes ses combinaisons le sont
    pub fn is_valid(&self) -> bool {
        self.melds.iter().all(|meld| meld.is_valid())
    }
}

#[cfg(test)]
mod tests;
