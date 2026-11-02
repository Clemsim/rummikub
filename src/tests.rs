use super::*;

#[test]
fn test_valid_runs_with_jokers() {
    // [Joker, 2 Rouge, 3 Rouge] -> valide (1, 2, 3 ou 2, 3, 4)
    let m1 = Combinaison::new(vec![
        Tuile::new_joker(100),
        Tuile::new_standard(1, Color::Red, 2),
        Tuile::new_standard(2, Color::Red, 3),
    ]);
    assert!(m1.is_valid_run());

    // [12 Noir, 13 Noir, Joker] -> le joker ne peut être que 11 (car > 13 interdit)
    let m2 = Combinaison::new(vec![
        Tuile::new_standard(3, Color::Black, 12),
        Tuile::new_standard(4, Color::Black, 13),
        Tuile::new_joker(101),
    ]);
    assert!(m2.is_valid_run());
    assert_eq!(m2.points(), 11 + 12 + 13);

    // [1 Rouge, 2 Rouge, Joker avec valeur 14 impossible]
    let m3 = Combinaison::new(vec![
        Tuile::new_standard(5, Color::Red, 1),
        Tuile::new_standard(6, Color::Red, 13),
        Tuile::new_joker(102),
    ]);
    assert!(!m3.is_valid_run()); // Écart trop grand pour 1 seul joker
}

#[test]
fn test_initial_meld_under_30_fails() {
    let mut joueur = Joueur::new(
        1,
        "Bob".to_string(),
        vec![
            Tuile::new_standard(1, Color::Red, 1),
            Tuile::new_standard(2, Color::Blue, 1),
            Tuile::new_standard(3, Color::Yellow, 1),
        ],
    );

    let combinaison = Combinaison::new(joueur.main.clone()); // 3 points seulement
    assert!(joueur.play_initial_meld(&[combinaison]).is_err());
    assert!(!joueur.a_fait_pose_initiale);
}
