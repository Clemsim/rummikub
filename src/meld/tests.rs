
use super::*;

fn standard(id: u8, color: Color, value: u8) -> Tuile {
    Tuile::new_standard(id, color, value)
}

#[test]
fn test_valid_groups() {
    // Groupe de 3 : même valeur, couleurs différentes
    let g3 = Meld::new(vec![
        standard(1, Color::Red, 7),
        standard(2, Color::Blue, 7),
        standard(3, Color::Yellow, 7),
    ]);
    assert!(g3.is_valid_group());
    assert!(g3.is_valid());
    assert_eq!(g3.points(), 21);

    // Groupe de 4
    let g4 = Meld::new(vec![
        standard(1, Color::Red, 13),
        standard(2, Color::Blue, 13),
        standard(3, Color::Yellow, 13),
        standard(4, Color::Black, 13),
    ]);
    assert!(g4.is_valid_group());
    assert_eq!(g4.points(), 52);
}

#[test]
fn test_invalid_groups() {
    // Deux fois la même couleur
    let dup_color = Meld::new(vec![
        standard(1, Color::Red, 7),
        standard(2, Color::Red, 7),
        standard(3, Color::Blue, 7),
    ]);
    assert!(!dup_color.is_valid_group());

    // Valeurs différentes
    let diff_values = Meld::new(vec![
        standard(1, Color::Red, 7),
        standard(2, Color::Blue, 8),
        standard(3, Color::Yellow, 7),
    ]);
    assert!(!diff_values.is_valid_group());

    // Trop court
    let two_tiles = Meld::new(vec![
        standard(1, Color::Red, 7),
        standard(2, Color::Blue, 7),
    ]);
    assert!(!two_tiles.is_valid_group());
    assert!(!two_tiles.is_valid());
}

#[test]
fn test_valid_group_with_joker() {
    // Le joker complète le groupe et prend la valeur du groupe
    let meld = Meld::new(vec![
        standard(1, Color::Red, 10),
        Tuile::new_joker(2),
        standard(3, Color::Black, 10),
    ]);
    assert!(meld.is_valid_group());
    assert!(meld.is_valid());
    assert_eq!(meld.points(), 3 * 10);
}

#[test]
fn test_invalid_runs() {
    // Couleurs mélangées
    let mixed_colors = Meld::new(vec![
        standard(1, Color::Red, 4),
        standard(2, Color::Blue, 5),
        standard(3, Color::Red, 6),
    ]);
    assert!(!mixed_colors.is_valid_run());
    assert!(!mixed_colors.is_valid());

    // Doublon de valeur (deux 5 rouges)
    let duplicate = Meld::new(vec![
        standard(1, Color::Red, 5),
        standard(2, Color::Red, 5),
        standard(3, Color::Red, 6),
    ]);
    assert!(!duplicate.is_valid_run());

    // Trop long : une suite ne dépasse pas 13 tuiles
    let too_long: Vec<Tuile> = (0u8..14)
        .map(|i| standard(i, Color::Red, i % 13 + 1))
        .collect();
    assert!(!Meld::new(too_long).is_valid_run());
}

#[test]
fn test_run_with_joker_fills_gap() {
    // [8 Bleu, Joker, 10 Bleu] : le joker prend la place du 9
    let meld = Meld::new(vec![
        standard(1, Color::Blue, 8),
        Tuile::new_joker(2),
        standard(3, Color::Blue, 10),
    ]);
    assert!(meld.is_valid_run());
    assert_eq!(meld.points(), 8 + 9 + 10);
}

#[test]
fn test_run_range_bounds() {
    // [Joker, 2 Rouge, 3 Rouge] : démarrage possible à 1 ou 2
    let meld = Meld::new(vec![
        Tuile::new_joker(9),
        standard(1, Color::Red, 2),
        standard(2, Color::Red, 3),
    ]);
    assert_eq!(meld.resolve_run_range(), Some((1, 2)));
}

#[test]
fn test_run_forced_at_extremes() {
    // [Joker, 1 Rouge, 2 Rouge] : seule la suite 1-2-3 contient 1 et 2
    let low = Meld::new(vec![
        Tuile::new_joker(9),
        standard(1, Color::Red, 1),
        standard(2, Color::Red, 2),
    ]);
    assert_eq!(low.resolve_run_range(), Some((1, 1)));
    assert_eq!(low.points(), 1 + 2 + 3);

    // [11 Noir, 12 Noir, 13 Noir, Joker] : le joker ne peut être que 10
    let high = Meld::new(vec![
        standard(1, Color::Black, 11),
        standard(2, Color::Black, 12),
        standard(3, Color::Black, 13),
        Tuile::new_joker(4),
    ]);
    assert_eq!(high.resolve_run_range(), Some((10, 10)));
    assert_eq!(high.points(), 10 + 11 + 12 + 13);
}

#[test]
fn test_run_points_maximize_start() {
    // [Joker, 10 Rouge, 11 Rouge] : le joker vaut 12 (10, 11, 12) plutôt que 9
    let meld = Meld::new(vec![
        Tuile::new_joker(9),
        standard(1, Color::Red, 10),
        standard(2, Color::Red, 11),
    ]);
    assert!(meld.is_valid_run());
    assert_eq!(meld.points(), 10 + 11 + 12);
}

#[test]
fn test_full_run_thirteen_tiles() {
    let tiles: Vec<Tuile> = (1u8..=13).map(|v| standard(v, Color::Blue, v)).collect();
    let meld = Meld::new(tiles);
    assert!(meld.is_valid());
    assert_eq!(meld.points(), (1..=13u8).sum::<u8>() as u32); // 91
}

#[test]
fn test_meld_neither_group_nor_run_is_invalid() {
    let meld = Meld::new(vec![
        standard(1, Color::Red, 4),
        standard(2, Color::Red, 9),
        standard(3, Color::Blue, 4),
    ]);
    assert!(!meld.is_valid());
    assert_eq!(meld.points(), 0);
}
