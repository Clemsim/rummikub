use super::*;
use crate::tuile::Color;

fn standard(id: u8, color: Color, value: u8) -> Tuile {
    Tuile::new_standard(id, color, value)
}

#[test]
fn test_initial_meld_success_removes_only_played_tiles() {
    let mut joueur = Joueur::new(
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

    let run = Combinaison::new(vec![
        standard(1, Color::Blue, 8),
        Tuile::new_joker(2),
        standard(3, Color::Blue, 10),
    ]); // 8 + 9 + 10 = 27 points
    let group = Combinaison::new(vec![
        standard(4, Color::Red, 7),
        standard(5, Color::Black, 7),
        standard(6, Color::Yellow, 7),
    ]); // 7 * 3 = 21 points

    let points = joueur.play_initial_meld(&[run, group]).unwrap();
    assert_eq!(points, 48);
    assert!(joueur.a_fait_pose_initiale);
    // Seule la tuile non jouée reste en main
    assert_eq!(joueur.main.len(), 1);
    assert_eq!(joueur.main[0].id, 7);
}

#[test]
fn test_initial_meld_tile_not_in_hand_fails() {
    let mut joueur = Joueur::new(
        1,
        "Bob".to_string(),
        vec![
            standard(1, Color::Red, 7),
            standard(2, Color::Blue, 7),
            standard(3, Color::Yellow, 7),
        ],
    );

    // La tuile ID 99 n'appartient pas à la main
    let combinaison = Combinaison::new(vec![
        standard(1, Color::Red, 7),
        standard(2, Color::Blue, 7),
        standard(99, Color::Black, 7),
    ]);

    assert!(joueur.play_initial_meld(&[combinaison]).is_err());
    assert!(!joueur.a_fait_pose_initiale);
    assert_eq!(joueur.main.len(), 3); // Main inchangée
}

#[test]
fn test_initial_meld_reused_tile_fails() {
    let mut joueur = Joueur::new(
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

    let group = Combinaison::new(vec![
        standard(1, Color::Red, 10),
        standard(2, Color::Blue, 10),
        standard(3, Color::Yellow, 10),
        standard(4, Color::Black, 10),
    ]); // 40 points
    let run = Combinaison::new(vec![
        standard(1, Color::Red, 10), // ID 1 déjà utilisé dans le groupe
        standard(5, Color::Red, 11),
        Tuile::new_joker(7),
    ]); // Suite 10-11-12 = 33 points

    assert!(joueur.play_initial_meld(&[group, run]).is_err());
    assert!(!joueur.a_fait_pose_initiale);
    assert_eq!(joueur.main.len(), 7); // Main inchangée
}

#[test]
fn test_initial_meld_cannot_be_played_twice() {
    let mut joueur = Joueur::new(
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

    let group_7 = Combinaison::new(vec![
        standard(1, Color::Red, 7),
        standard(2, Color::Blue, 7),
        standard(3, Color::Yellow, 7),
    ]);
    let group_10 = Combinaison::new(vec![
        standard(4, Color::Red, 10),
        standard(5, Color::Blue, 10),
        standard(6, Color::Yellow, 10),
    ]);

    assert!(joueur.play_initial_meld(&[group_7, group_10]).is_ok());

    let err = joueur.play_initial_meld(&[]).unwrap_err();
    assert!(err.contains("déjà"));
}

#[test]
fn test_initial_meld_empty_proposal_fails() {
    let mut joueur = Joueur::new(1, "Eve".to_string(), vec![]);
    let err = joueur.play_initial_meld(&[]).unwrap_err();
    assert!(err.contains("Aucune combinaison"));
    assert!(!joueur.a_fait_pose_initiale);
}
