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

    /// Valide et applique la pose initiale (>= 30 points, uniquement depuis la main)
    pub fn play_initial_meld(&mut self, proposed_melds: &[Meld]) -> Result<u32, String> {
        if self.has_completed_initial_meld {
            return Err("La pose initiale a déjà été réalisée.".to_string());
        }

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
        let mut played_tile_ids = HashSet::new();
        let hand_ids: HashSet<u8> = self.hand.iter().map(|t| t.id).collect();

        for meld in proposed_melds {
            for tile in &meld.tiles {
                if !hand_ids.contains(&tile.id) {
                    return Err(format!(
                        "La tuile ID {} ne provient pas de la main du joueur.",
                        tile.id
                    ));
                }
                if !played_tile_ids.insert(tile.id) {
                    return Err(format!(
                        "La tuile ID {} a été utilisée plusieurs fois.",
                        tile.id
                    ));
                }
            }
        }

        // 3. Calculer les points totaux
        let total_points: u32 = proposed_melds.iter().map(|m| m.points()).sum();

        if total_points < 30 {
            return Err(format!(
                "Points insuffisants : {} points obtenus (minimum 30 requis).",
                total_points
            ));
        }

        // Retirer les tuiles jouées de la main du joueur
        self.hand.retain(|t| !played_tile_ids.contains(&t.id));
        self.has_completed_initial_meld = true;

        Ok(total_points)
    }
}
