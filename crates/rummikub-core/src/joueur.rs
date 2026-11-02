use crate::combinaison::Combinaison;
use crate::tuile::{BitsetTuiles, Tuile};

#[derive(Debug, Clone)]
pub struct Joueur {
    pub id: usize,
    pub nom: String,
    pub main: Vec<Tuile>,
    pub a_fait_pose_initiale: bool,
}

impl Joueur {
    pub fn nouveau(id: usize, nom: String, main: Vec<Tuile>) -> Self {
        Self {
            id,
            nom,
            main,
            a_fait_pose_initiale: false,
        }
    }

    /// Vérifie une proposition de pose sans rien modifier : chaque combinaison
    /// doit être valide, et toutes les tuiles doivent provenir de la main du
    /// joueur, sans réutilisation.
    /// Retourne (points totaux, identifiants des tuiles jouées).
    fn verifier_proposition(
        &self,
        proposed_melds: &[Combinaison],
    ) -> Result<(u32, BitsetTuiles), String> {
        if proposed_melds.is_empty() {
            return Err("Aucune combinaison proposée.".to_string());
        }

        // 1. Vérifier que chaque combinaison est valide
        for (i, combinaison) in proposed_melds.iter().enumerate() {
            if !combinaison.est_valide() {
                return Err(format!("La combinaison n°{} est invalide.", i + 1));
            }
        }

        // 2. Vérifier que toutes les tuiles proviennent de la main du joueur (sans réutilisation)
        let mut played_Tuile_ids = BitsetTuiles::default();
        let ids_main = BitsetTuiles::depuis_ids(self.main.iter().map(|t| t.id));

        for combinaison in proposed_melds {
            for Tuile in &combinaison.Tuiles {
                if !ids_main.contient(Tuile.id) {
                    return Err(format!(
                        "La tuile ID {} ne provient pas de la main du joueur.",
                        Tuile.id
                    ));
                }
                if !played_Tuile_ids.inserer_si_absent(Tuile.id) {
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
    pub fn jouer_pose_initiale(&mut self, proposed_melds: &[Combinaison]) -> Result<u32, String> {
        if self.a_fait_pose_initiale {
            return Err("La pose initiale a déjà été réalisée.".to_string());
        }

        let (total_points, played_Tuile_ids) = self.verifier_proposition(proposed_melds)?;

        if total_points < 30 {
            return Err(format!(
                "Points insuffisants : {} points obtenus (minimum 30 requis).",
                total_points
            ));
        }

        self.retirer_tuiles_jouees(&played_Tuile_ids);
        self.a_fait_pose_initiale = true;

        Ok(total_points)
    }

    /// Valide et applique la pose de combinaisons depuis la main
    /// (à utiliser après la pose initiale : pas d'exigence de 30 points ici)
    pub fn poser_combinaisons(&mut self, proposed_melds: &[Combinaison]) -> Result<u32, String> {
        let (total_points, played_Tuile_ids) = self.verifier_proposition(proposed_melds)?;

        self.retirer_tuiles_jouees(&played_Tuile_ids);

        Ok(total_points)
    }

    /// Retire de la main les tuiles identifiées
    fn retirer_tuiles_jouees(&mut self, played_Tuile_ids: &BitsetTuiles) {
        self.main.retain(|t| !played_Tuile_ids.contient(t.id));
    }
}

#[cfg(test)]
mod tests;
