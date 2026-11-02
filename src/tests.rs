use super::*;

#[test]
fn test_valid_runs_with_jokers() {
    // [Joker, 2 Rouge, 3 Rouge] -> valide (1, 2, 3 ou 2, 3, 4)
    let m1 = Combinaison::nouvelle(vec![
        Tuile::nouveau_joker(100),
        Tuile::nouvelle_standard(1, Couleur::Rouge, 2),
        Tuile::nouvelle_standard(2, Couleur::Rouge, 3),
    ]);
    assert!(m1.est_suite_valide());

    // [12 Noir, 13 Noir, Joker] -> le joker ne peut être que 11 (car > 13 interdit)
    let m2 = Combinaison::nouvelle(vec![
        Tuile::nouvelle_standard(3, Couleur::Noir, 12),
        Tuile::nouvelle_standard(4, Couleur::Noir, 13),
        Tuile::nouveau_joker(101),
    ]);
    assert!(m2.est_suite_valide());
    assert_eq!(m2.points(), 11 + 12 + 13);

    // [1 Rouge, 2 Rouge, Joker avec valeur 14 impossible]
    let m3 = Combinaison::nouvelle(vec![
        Tuile::nouvelle_standard(5, Couleur::Rouge, 1),
        Tuile::nouvelle_standard(6, Couleur::Rouge, 13),
        Tuile::nouveau_joker(102),
    ]);
    assert!(!m3.est_suite_valide()); // Écart trop grand pour 1 seul joker
}

#[test]
fn test_initial_meld_under_30_fails() {
    let mut joueur = Joueur::nouveau(
        1,
        "Bob".to_string(),
        vec![
            Tuile::nouvelle_standard(1, Couleur::Rouge, 1),
            Tuile::nouvelle_standard(2, Couleur::Bleu, 1),
            Tuile::nouvelle_standard(3, Couleur::Jaune, 1),
        ],
    );

    let combinaison = Combinaison::nouvelle(joueur.main.clone()); // 3 points seulement
    assert!(joueur.jouer_pose_initiale(&[combinaison]).is_err());
    assert!(!joueur.a_fait_pose_initiale);
}
