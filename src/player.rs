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

        if total_points < 30 {
            return Err(format!(
                "Points insuffisants : {} points obtenus (minimum 30 requis).",
                total_points
            ));
        }

        // Retirer les tuiles jouées de la main du joueur
        self.hand.retain(|t| !played_Tuile_ids.contains(&t.id));
        self.has_completed_initial_meld = true;

        Ok(total_points)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tuile::Color;

    fn standard(id: u8, color: Color, value: u8) -> Tuile {
        Tuile::new_standard(id, color, value)
    }

    #[test]
    fn test_initial_meld_success_removes_only_played_tiles() {
        let mut player = Player::new(
            1,
            "Alice".to_string(),
            vec![
                standard(1, Color::Blue, 8),
                Tuile::new_joker(2),
                standard(3, Color::Blue, 10),
                standard(4, Color::Red, 7),
                standard(5, Color::Black, 7),
                standard(6, Color::Yellow, 7),
                standard(7, Color::Red, 3), // Reste en main
            ],
        );

        let run = Meld::new(vec![
            standard(1, Color::Blue, 8),
            Tuile::new_joker(2),
            standard(3, Color::Blue, 10),
        ]); // 8 + 9 + 10 = 27 points
        let group = Meld::new(vec![
            standard(4, Color::Red, 7),
            standard(5, Color::Black, 7),
            standard(6, Color::Yellow, 7),
        ]); // 7 * 3 = 21 points

        let points = player.play_initial_meld(&[run, group]).unwrap();
        assert_eq!(points, 48);
        assert!(player.has_completed_initial_meld);
        // Seule la tuile non jouée reste en main
        assert_eq!(player.hand.len(), 1);
        assert_eq!(player.hand[0].id, 7);
    }

    #[test]
    fn test_initial_meld_tile_not_in_hand_fails() {
        let mut player = Player::new(
            1,
            "Bob".to_string(),
            vec![
                standard(1, Color::Red, 7),
                standard(2, Color::Blue, 7),
                standard(3, Color::Yellow, 7),
            ],
        );

        // La tuile ID 99 n'appartient pas à la main
        let meld = Meld::new(vec![
            standard(1, Color::Red, 7),
            standard(2, Color::Blue, 7),
            standard(99, Color::Black, 7),
        ]);

        assert!(player.play_initial_meld(&[meld]).is_err());
        assert!(!player.has_completed_initial_meld);
        assert_eq!(player.hand.len(), 3); // Main inchangée
    }

    #[test]
    fn test_initial_meld_reused_tile_fails() {
        let mut player = Player::new(
            1,
            "Carol".to_string(),
            vec![
                standard(1, Color::Red, 10),
                standard(2, Color::Blue, 10),
                standard(3, Color::Yellow, 10),
                standard(4, Color::Black, 10),
                standard(5, Color::Red, 11),
                standard(6, Color::Red, 12),
                Tuile::new_joker(7),
            ],
        );

        let group = Meld::new(vec![
            standard(1, Color::Red, 10),
            standard(2, Color::Blue, 10),
            standard(3, Color::Yellow, 10),
            standard(4, Color::Black, 10),
        ]); // 40 points
        let run = Meld::new(vec![
            standard(1, Color::Red, 10), // ID 1 déjà utilisé dans le groupe
            standard(5, Color::Red, 11),
            Tuile::new_joker(7),
        ]); // Suite 10-11-12 = 33 points

        assert!(player.play_initial_meld(&[group, run]).is_err());
        assert!(!player.has_completed_initial_meld);
        assert_eq!(player.hand.len(), 7); // Main inchangée
    }

    #[test]
    fn test_initial_meld_cannot_be_played_twice() {
        let mut player = Player::new(
            1,
            "Dave".to_string(),
            vec![
                standard(1, Color::Red, 7),
                standard(2, Color::Blue, 7),
                standard(3, Color::Yellow, 7),
                standard(4, Color::Red, 10),
                standard(5, Color::Blue, 10),
                standard(6, Color::Yellow, 10),
            ],
        );

        let group_7 = Meld::new(vec![
            standard(1, Color::Red, 7),
            standard(2, Color::Blue, 7),
            standard(3, Color::Yellow, 7),
        ]);
        let group_10 = Meld::new(vec![
            standard(4, Color::Red, 10),
            standard(5, Color::Blue, 10),
            standard(6, Color::Yellow, 10),
        ]);

        assert!(player.play_initial_meld(&[group_7, group_10]).is_ok());

        let err = player.play_initial_meld(&[]).unwrap_err();
        assert!(err.contains("déjà"));
    }

    #[test]
    fn test_initial_meld_empty_proposal_fails() {
        let mut player = Player::new(1, "Eve".to_string(), vec![]);
        let err = player.play_initial_meld(&[]).unwrap_err();
        assert!(err.contains("Aucune combinaison"));
        assert!(!player.has_completed_initial_meld);
    }
}
