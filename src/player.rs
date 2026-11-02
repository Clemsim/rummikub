use crate::meld::Meld;
use crate::tuile::Tuile;
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct Player {
    pub id: usize,
    pub name: String,
    pub hand: Vec<Tuile>,
    pub has_completed_initial_meld: bool,
}

impl Player {
    pub fn new(id: usize, name: String, hand: Vec<Tuile>) -> Self {
        Self {
            id,
            name,
            hand,
            has_completed_initial_meld: false,
        }
    }

    /// Vérifie une proposition de pose sans rien modifier : chaque combinaison
    /// doit être valide, et toutes les tuiles doivent provenir de la main du
    /// joueur, sans réutilisation.
    /// Retourne (points totaux, identifiants des tuiles jouées).
    fn check_proposal(&self, proposed_melds: &[Meld]) -> Result<(u32, HashSet<u8>), String> {
        if proposed_melds.is_empty() {
            return Err("Aucune combinaison proposée.".to_string());
        }

        // 1. Vérifier que chaque combinaison est valide
        for (i, meld) in proposed_melds.iter().enumerate() {
            if !meld.is_valid() {
                return Err(format!("La combinaison n°{} est invalide.", i + 1));
            }
        }

        // 2. Vérifier que toutes les tuiles proviennent de la main du joueur (sans réutilisation)
        let mut played_Tuile_ids = HashSet::new();
        let hand_ids: HashSet<u8> = self.hand.iter().map(|t| t.id).collect();

        for meld in proposed_melds {
            for Tuile in &meld.Tuiles {
                if !hand_ids.contains(&Tuile.id) {
                    return Err(format!(
                        "La tuile ID {} ne provient pas de la main du joueur.",
                        Tuile.id
                    ));
                }
                if !played_Tuile_ids.insert(Tuile.id) {
                    return Err(format!(
                        "La tuile ID {} a été utilisée plusieurs fois.",
                        Tuile.id
                    ));
                }
            }
        }

        // 3. Calculer les points totaux
        let total_points: u32 = proposed_melds.iter().map(|m| m.points()).sum();

        Ok((total_points, played_Tuile_ids))
    }

    /// Valide et applique la pose initiale (>= 30 points, uniquement depuis la main)
    pub fn play_initial_meld(&mut self, proposed_melds: &[Meld]) -> Result<u32, String> {
        if self.has_completed_initial_meld {
            return Err("La pose initiale a déjà été réalisée.".to_string());
        }

        let (total_points, played_Tuile_ids) = self.check_proposal(proposed_melds)?;

        if total_points < 30 {
            return Err(format!(
                "Points insuffisants : {} points obtenus (minimum 30 requis).",
                total_points
            ));
        }

        self.remove_played_tiles(&played_Tuile_ids);
        self.has_completed_initial_meld = true;

        Ok(total_points)
    }

    /// Valide et applique la pose de combinaisons depuis la main
    /// (à utiliser après la pose initiale : pas d'exigence de 30 points ici)
    pub fn play_melds(&mut self, proposed_melds: &[Meld]) -> Result<u32, String> {
        let (total_points, played_Tuile_ids) = self.check_proposal(proposed_melds)?;

        self.remove_played_tiles(&played_Tuile_ids);

        Ok(total_points)
    }

    /// Retire de la main les tuiles identifiées
    fn remove_played_tiles(&mut self, played_Tuile_ids: &HashSet<u8>) {
        self.hand.retain(|t| !played_Tuile_ids.contains(&t.id));
    }
}

#[cfg(test)]
mod tests;
