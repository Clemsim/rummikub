use super::*;
use crate::tuile::Couleur;

fn standard(id: u8, couleur: Couleur, valeur: u8) -> Tuile {
    Tuile::nouvelle_standard(id, couleur, valeur)
}

#[test]
fn test_initial_meld_success_removes_only_played_tiles() {
    let mut joueur = Joueur::nouveau(
        1,
        "Alice".to_string(),
        vec![
            standard(1, Couleur::Bleu, 8),
            Tuile::nouveau_joker(2),
            standard(3, Couleur::Bleu, 10),
            standard(4, Couleur::Rouge, 7),
            standard(5, Couleur::Noir, 7),
            standard(6, Couleur::Jaune, 7),
            standard(7, Couleur::Rouge, 3), // Reste en main
        ],
    );

    let suite = Combinaison::nouvelle(vec![
        standard(1, Couleur::Bleu, 8),
        Tuile::nouveau_joker(2),
        standard(3, Couleur::Bleu, 10),
    ]); // 8 + 9 + 10 = 27 points
    let groupe = Combinaison::nouvelle(vec![
        standard(4, Couleur::Rouge, 7),
        standard(5, Couleur::Noir, 7),
        standard(6, Couleur::Jaune, 7),
    ]); // 7 * 3 = 21 points

    let points = joueur.jouer_pose_initiale(&[suite, groupe]).unwrap();
    assert_eq!(points, 48);
    assert!(joueur.a_fait_pose_initiale);
    // Seule la tuile non jouée reste en main
    assert_eq!(joueur.main.len(), 1);
    assert_eq!(joueur.main[0].id, 7);
}

#[test]
fn test_initial_meld_tile_not_in_hand_fails() {
    let mut joueur = Joueur::nouveau(
        1,
        "Bob".to_string(),
        vec![
            standard(1, Couleur::Rouge, 7),
            standard(2, Couleur::Bleu, 7),
            standard(3, Couleur::Jaune, 7),
        ],
    );

    // La tuile ID 99 n'appartient pas à la main
    let combinaison = Combinaison::nouvelle(vec![
        standard(1, Couleur::Rouge, 7),
        standard(2, Couleur::Bleu, 7),
        standard(99, Couleur::Noir, 7),
    ]);

    assert!(joueur.jouer_pose_initiale(&[combinaison]).is_err());
    assert!(!joueur.a_fait_pose_initiale);
    assert_eq!(joueur.main.len(), 3); // Main inchangée
}

#[test]
fn test_initial_meld_reused_tile_fails() {
    let mut joueur = Joueur::nouveau(
        1,
        "Carol".to_string(),
        vec![
            standard(1, Couleur::Rouge, 10),
            standard(2, Couleur::Bleu, 10),
            standard(3, Couleur::Jaune, 10),
            standard(4, Couleur::Noir, 10),
            standard(5, Couleur::Rouge, 11),
            standard(6, Couleur::Rouge, 12),
            Tuile::nouveau_joker(7),
        ],
    );

    let groupe = Combinaison::nouvelle(vec![
        standard(1, Couleur::Rouge, 10),
        standard(2, Couleur::Bleu, 10),
        standard(3, Couleur::Jaune, 10),
        standard(4, Couleur::Noir, 10),
    ]); // 40 points
    let suite = Combinaison::nouvelle(vec![
        standard(1, Couleur::Rouge, 10), // ID 1 déjà utilisé dans le groupe
        standard(5, Couleur::Rouge, 11),
        Tuile::nouveau_joker(7),
    ]); // Suite 10-11-12 = 33 points

    assert!(joueur.jouer_pose_initiale(&[groupe, suite]).is_err());
    assert!(!joueur.a_fait_pose_initiale);
    assert_eq!(joueur.main.len(), 7); // Main inchangée
}

#[test]
fn test_initial_meld_cannot_be_played_twice() {
    let mut joueur = Joueur::nouveau(
        1,
        "Dave".to_string(),
        vec![
            standard(1, Couleur::Rouge, 7),
            standard(2, Couleur::Bleu, 7),
            standard(3, Couleur::Jaune, 7),
            standard(4, Couleur::Rouge, 10),
            standard(5, Couleur::Bleu, 10),
            standard(6, Couleur::Jaune, 10),
        ],
    );

    let groupe_7 = Combinaison::nouvelle(vec![
        standard(1, Couleur::Rouge, 7),
        standard(2, Couleur::Bleu, 7),
        standard(3, Couleur::Jaune, 7),
    ]);
    let groupe_10 = Combinaison::nouvelle(vec![
        standard(4, Couleur::Rouge, 10),
        standard(5, Couleur::Bleu, 10),
        standard(6, Couleur::Jaune, 10),
    ]);

    assert!(joueur.jouer_pose_initiale(&[groupe_7, groupe_10]).is_ok());

    let err = joueur.jouer_pose_initiale(&[]).unwrap_err();
    assert!(err.contains("déjà"));
}

#[test]
fn test_initial_meld_empty_proposal_fails() {
    let mut joueur = Joueur::nouveau(1, "Eve".to_string(), vec![]);
    let err = joueur.jouer_pose_initiale(&[]).unwrap_err();
    assert!(err.contains("Aucune combinaison"));
    assert!(!joueur.a_fait_pose_initiale);
}
