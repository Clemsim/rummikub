use super::*;
use crate::tuile::{Color, Tuile};

fn standard(id: u8, color: Color, value: u8) -> Tuile {
    Tuile::new_standard(id, color, value)
}

#[test]
fn test_empty_board_is_valid() {
    let plateau = Plateau::default();
    assert!(plateau.is_valid());
    assert!(plateau.tile_ids().is_empty());
}

#[test]
fn test_board_validity_follows_its_melds() {
    let mut plateau = Plateau::default();
    plateau.place_meld(Combinaison::new(vec![
        standard(1, Color::Red, 7),
        standard(2, Color::Blue, 7),
        standard(3, Color::Yellow, 7),
    ]));
    assert!(plateau.is_valid());

    // Une seule combinaison invalide rend le plateau invalide
    plateau.place_meld(Combinaison::new(vec![
        standard(4, Color::Red, 5),
        standard(5, Color::Red, 6),
    ]));
    assert!(!plateau.is_valid());
}

#[test]
fn test_tile_ids_and_replacement() {
    let mut plateau = Plateau::default();
    plateau.place_meld(Combinaison::new(vec![
        standard(1, Color::Red, 4),
        standard(2, Color::Blue, 4),
        standard(3, Color::Yellow, 4),
    ]));
    plateau.place_meld(Combinaison::new(vec![
        standard(4, Color::Red, 8),
        standard(5, Color::Blue, 8),
        standard(6, Color::Yellow, 8),
    ]));

    let ids = plateau.tile_ids();
    assert_eq!(ids.len(), 6);
    assert!(ids.contains(&1) && ids.contains(&6));

    // set_melds remplace tout l'état du plateau
    plateau.set_melds(vec![Combinaison::new(vec![
        standard(7, Color::Black, 2),
        standard(8, Color::Red, 3),
        standard(9, Color::Red, 4),
    ])]);
    assert_eq!(plateau.tile_ids(), HashSet::from([7, 8, 9]));
}
