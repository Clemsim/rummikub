use super::*;
use crate::tuile::{Couleur, Tuile};

fn standard(id: u8, couleur: Couleur, valeur: u8) -> Tuile {
    Tuile::nouvelle_standard(id, couleur, valeur)
}

#[test]
fn test_empty_board_is_valid() {
    let plateau = Plateau::default();
    assert!(plateau.est_valide());
    assert!(plateau.ids_tuiles().est_vide());
}

#[test]
fn test_board_validity_follows_its_melds() {
    let mut plateau = Plateau::default();
    plateau.placer_combinaison(Combinaison::nouvelle(vec![
        standard(1, Couleur::Rouge, 7),
        standard(2, Couleur::Bleu, 7),
        standard(3, Couleur::Jaune, 7),
    ]));
    assert!(plateau.est_valide());

    // Une seule combinaison invalide rend le plateau invalide
    plateau.placer_combinaison(Combinaison::nouvelle(vec![
        standard(4, Couleur::Rouge, 5),
        standard(5, Couleur::Rouge, 6),
    ]));
    assert!(!plateau.est_valide());
}

#[test]
fn test_tile_ids_and_replacement() {
    let mut plateau = Plateau::default();
    plateau.placer_combinaison(Combinaison::nouvelle(vec![
        standard(1, Couleur::Rouge, 4),
        standard(2, Couleur::Bleu, 4),
        standard(3, Couleur::Jaune, 4),
    ]));
    plateau.placer_combinaison(Combinaison::nouvelle(vec![
        standard(4, Couleur::Rouge, 8),
        standard(5, Couleur::Bleu, 8),
        standard(6, Couleur::Jaune, 8),
    ]));

    let ids = plateau.ids_tuiles();
    assert_eq!(ids.ids().count(), 6);
    assert!(ids.contient(1) && ids.contient(6));

    // remplacer_combinaisons remplace tout l'état du plateau
    plateau.remplacer_combinaisons(vec![Combinaison::nouvelle(vec![
        standard(7, Couleur::Noir, 2),
        standard(8, Couleur::Rouge, 3),
        standard(9, Couleur::Rouge, 4),
    ])]);
    assert_eq!(
        plateau.ids_tuiles().ids().collect::<Vec<u8>>(),
        vec![7, 8, 9]
    );
}
