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
mod tests {
    use super::*;
    use crate::tuile::{Color, Tuile};

    fn standard(id: u8, color: Color, value: u8) -> Tuile {
        Tuile::new_standard(id, color, value)
    }

    #[test]
    fn test_empty_board_is_valid() {
        let board = Board::default();
        assert!(board.is_valid());
        assert!(board.tile_ids().is_empty());
    }

    #[test]
    fn test_board_validity_follows_its_melds() {
        let mut board = Board::default();
        board.place_meld(Meld::new(vec![
            standard(1, Color::Red, 7),
            standard(2, Color::Blue, 7),
            standard(3, Color::Yellow, 7),
        ]));
        assert!(board.is_valid());

        // Une seule combinaison invalide rend le plateau invalide
        board.place_meld(Meld::new(vec![
            standard(4, Color::Red, 5),
            standard(5, Color::Red, 6),
        ]));
        assert!(!board.is_valid());
    }

    #[test]
    fn test_tile_ids_and_replacement() {
        let mut board = Board::default();
        board.place_meld(Meld::new(vec![
            standard(1, Color::Red, 4),
            standard(2, Color::Blue, 4),
            standard(3, Color::Yellow, 4),
        ]));
        board.place_meld(Meld::new(vec![
            standard(4, Color::Red, 8),
            standard(5, Color::Blue, 8),
            standard(6, Color::Yellow, 8),
        ]));

        let ids = board.tile_ids();
        assert_eq!(ids.len(), 6);
        assert!(ids.contains(&1) && ids.contains(&6));

        // set_melds remplace tout l'état du plateau
        board.set_melds(vec![Meld::new(vec![
            standard(7, Color::Black, 2),
            standard(8, Color::Red, 3),
            standard(9, Color::Red, 4),
        ])]);
        assert_eq!(board.tile_ids(), HashSet::from([7, 8, 9]));
    }
}
