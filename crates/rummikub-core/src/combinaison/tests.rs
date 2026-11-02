use super::*;

fn standard(id: u8, couleur: Couleur, valeur: u8) -> Tuile {
    Tuile::nouvelle_standard(id, couleur, valeur)
}

#[test]
fn test_valid_groups() {
    // Groupe de 3 : même valeur, couleurs différentes
    let g3 = Combinaison::nouvelle(vec![
        standard(1, Couleur::Rouge, 7),
        standard(2, Couleur::Bleu, 7),
        standard(3, Couleur::Jaune, 7),
    ]);
    assert!(g3.est_groupe_valide());
    assert!(g3.est_valide());
    assert_eq!(g3.points(), 21);

    // Groupe de 4
    let g4 = Combinaison::nouvelle(vec![
        standard(1, Couleur::Rouge, 13),
        standard(2, Couleur::Bleu, 13),
        standard(3, Couleur::Jaune, 13),
        standard(4, Couleur::Noir, 13),
    ]);
    assert!(g4.est_groupe_valide());
    assert_eq!(g4.points(), 52);
}

#[test]
fn test_invalid_groups() {
    // Deux fois la même couleur
    let couleur_dupliquee = Combinaison::nouvelle(vec![
        standard(1, Couleur::Rouge, 7),
        standard(2, Couleur::Rouge, 7),
        standard(3, Couleur::Bleu, 7),
    ]);
    assert!(!couleur_dupliquee.est_groupe_valide());

    // Valeurs différentes
    let valeurs_differentes = Combinaison::nouvelle(vec![
        standard(1, Couleur::Rouge, 7),
        standard(2, Couleur::Bleu, 8),
        standard(3, Couleur::Jaune, 7),
    ]);
    assert!(!valeurs_differentes.est_groupe_valide());

    // Trop court
    let deux_tuiles = Combinaison::nouvelle(vec![
        standard(1, Couleur::Rouge, 7),
        standard(2, Couleur::Bleu, 7),
    ]);
    assert!(!deux_tuiles.est_groupe_valide());
    assert!(!deux_tuiles.est_valide());
}

#[test]
fn test_valid_group_with_joker() {
    // Le joker complète le groupe et prend la valeur du groupe
    let combinaison = Combinaison::nouvelle(vec![
        standard(1, Couleur::Rouge, 10),
        Tuile::nouveau_joker(2),
        standard(3, Couleur::Noir, 10),
    ]);
    assert!(combinaison.est_groupe_valide());
    assert!(combinaison.est_valide());
    assert_eq!(combinaison.points(), 3 * 10);
}

#[test]
fn test_invalid_runs() {
    // Couleurs mélangées
    let couleurs_melangees = Combinaison::nouvelle(vec![
        standard(1, Couleur::Rouge, 4),
        standard(2, Couleur::Bleu, 5),
        standard(3, Couleur::Rouge, 6),
    ]);
    assert!(!couleurs_melangees.est_suite_valide());
    assert!(!couleurs_melangees.est_valide());

    // Doublon de valeur (deux 5 rouges)
    let doublon = Combinaison::nouvelle(vec![
        standard(1, Couleur::Rouge, 5),
        standard(2, Couleur::Rouge, 5),
        standard(3, Couleur::Rouge, 6),
    ]);
    assert!(!doublon.est_suite_valide());

    // Trop long : une suite ne dépasse pas 13 tuiles
    let trop_long: Vec<Tuile> = (0u8..14)
        .map(|i| standard(i, Couleur::Rouge, i % 13 + 1))
        .collect();
    assert!(!Combinaison::nouvelle(trop_long).est_suite_valide());
}

#[test]
fn test_run_with_joker_fills_gap() {
    // [8 Bleu, Joker, 10 Bleu] : le joker prend la place du 9
    let combinaison = Combinaison::nouvelle(vec![
        standard(1, Couleur::Bleu, 8),
        Tuile::nouveau_joker(2),
        standard(3, Couleur::Bleu, 10),
    ]);
    assert!(combinaison.est_suite_valide());
    assert_eq!(combinaison.points(), 8 + 9 + 10);
}

#[test]
fn test_run_range_bounds() {
    // [Joker, 2 Rouge, 3 Rouge] : démarrage possible à 1 ou 2
    let combinaison = Combinaison::nouvelle(vec![
        Tuile::nouveau_joker(9),
        standard(1, Couleur::Rouge, 2),
        standard(2, Couleur::Rouge, 3),
    ]);
    assert_eq!(combinaison.bornes_de_suite(), Some((1, 2)));
}

#[test]
fn test_run_forced_at_extremes() {
    // [Joker, 1 Rouge, 2 Rouge] : seule la suite 1-2-3 contient 1 et 2
    let debut_bas = Combinaison::nouvelle(vec![
        Tuile::nouveau_joker(9),
        standard(1, Couleur::Rouge, 1),
        standard(2, Couleur::Rouge, 2),
    ]);
    assert_eq!(debut_bas.bornes_de_suite(), Some((1, 1)));
    assert_eq!(debut_bas.points(), 1 + 2 + 3);

    // [11 Noir, 12 Noir, 13 Noir, Joker] : le joker ne peut être que 10
    let debut_haut = Combinaison::nouvelle(vec![
        standard(1, Couleur::Noir, 11),
        standard(2, Couleur::Noir, 12),
        standard(3, Couleur::Noir, 13),
        Tuile::nouveau_joker(4),
    ]);
    assert_eq!(debut_haut.bornes_de_suite(), Some((10, 10)));
    assert_eq!(debut_haut.points(), 10 + 11 + 12 + 13);
}

#[test]
fn test_run_points_maximize_start() {
    // [Joker, 10 Rouge, 11 Rouge] : le joker vaut 12 (10, 11, 12) plutôt que 9
    let combinaison = Combinaison::nouvelle(vec![
        Tuile::nouveau_joker(9),
        standard(1, Couleur::Rouge, 10),
        standard(2, Couleur::Rouge, 11),
    ]);
    assert!(combinaison.est_suite_valide());
    assert_eq!(combinaison.points(), 10 + 11 + 12);
}

#[test]
fn test_full_run_thirteen_tiles() {
    let tuiles: Vec<Tuile> = (1u8..=13).map(|v| standard(v, Couleur::Bleu, v)).collect();
    let combinaison = Combinaison::nouvelle(tuiles);
    assert!(combinaison.est_valide());
    assert_eq!(combinaison.points(), (1..=13u8).sum::<u8>() as u32); // 91
}

#[test]
fn test_meld_neither_group_nor_run_is_invalid() {
    let combinaison = Combinaison::nouvelle(vec![
        standard(1, Couleur::Rouge, 4),
        standard(2, Couleur::Rouge, 9),
        standard(3, Couleur::Bleu, 4),
    ]);
    assert!(!combinaison.est_valide());
    assert_eq!(combinaison.points(), 0);
}
