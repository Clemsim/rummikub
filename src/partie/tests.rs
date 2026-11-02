use super::*;
use crate::tuile::Color;

fn standard(id: u8, color: Color, value: u8) -> Tuile {
    Tuile::new_standard(id, color, value)
}

fn partie_deux_joueurs() -> Partie {
    Partie::new(vec!["Alice".to_string(), "Bob".to_string()]).unwrap()
}

/// Tour minimal légal : le joueur pioche, ce qui termine son tour
fn tour_pioche(partie: &mut Partie) {
    partie.start_turn();
    assert!(partie.draw_tile().unwrap().is_some());
}

#[test]
fn test_new_game_deals_14_tiles_each() {
    let partie = partie_deux_joueurs();
    assert_eq!(partie.joueurs.len(), 2);
    assert!(partie.joueurs.iter().all(|joueur| joueur.main.len() == 14));
    assert_eq!(partie.pioche.len(), 106 - 2 * 14);
    assert!(partie.plateau.combinaisons.is_empty());
    assert_eq!(partie.index_joueur_courant, 0);
}

#[test]
fn test_new_game_rejects_wrong_player_count() {
    assert!(Partie::new(vec!["Alice".to_string()]).is_err());
    assert!(
        Partie::new(vec![
            "A".to_string(),
            "B".to_string(),
            "C".to_string(),
            "D".to_string(),
            "E".to_string()
        ])
        .is_err()
    );
}

#[test]
fn test_play_melds_initial_meld_below_30_rejected() {
    let mut partie = partie_deux_joueurs();
    partie.joueurs[0].main = vec![
        standard(1, Color::Red, 7),
        standard(2, Color::Blue, 7),
        standard(3, Color::Yellow, 7),
    ]; // 21 points seulement

    partie.start_turn();
    let combinaison = Combinaison::new(partie.joueurs[0].main.clone());
    assert!(partie.play_melds(vec![combinaison]).is_err());

    // Rien n'a été posé, la main est intacte
    assert!(partie.plateau.combinaisons.is_empty());
    assert_eq!(partie.joueurs[0].main.len(), 3);
    assert!(!partie.joueurs[0].a_fait_pose_initiale);
}

#[test]
fn test_play_melds_after_initial_needs_no_30_points() {
    let mut partie = partie_deux_joueurs();
    partie.joueurs[0].main = vec![
        standard(1, Color::Red, 10),
        standard(2, Color::Blue, 10),
        standard(3, Color::Yellow, 10),
        standard(4, Color::Red, 11),
        standard(5, Color::Red, 12),
        standard(6, Color::Red, 13),
    ];

    // Pose initiale : groupe de trois 10 (30 points)
    partie.start_turn();
    let groupe = Combinaison::new(vec![
        standard(1, Color::Red, 10),
        standard(2, Color::Blue, 10),
        standard(3, Color::Yellow, 10),
    ]);
    partie.play_melds(vec![groupe]).unwrap();
    partie.end_turn().unwrap();

    tour_pioche(&mut partie); // Bob pioche et passe

    // Alice rejoue une suite de 36 points : plus d'exigence des 30 points
    partie.start_turn();
    let suite = Combinaison::new(vec![
        standard(4, Color::Red, 11),
        standard(5, Color::Red, 12),
        standard(6, Color::Red, 13),
    ]);
    let points = partie.play_melds(vec![suite]).unwrap();
    assert_eq!(points, 36);
    assert!(partie.joueurs[0].main.is_empty());
    assert_eq!(partie.plateau.combinaisons.len(), 2);

    partie.end_turn().unwrap();
    assert_eq!(partie.winner(), Some(0));
}

#[test]
fn test_manipulation_extends_a_board_meld() {
    let mut partie = partie_deux_joueurs();
    partie.joueurs[0].main = vec![
        standard(1, Color::Red, 10),
        standard(2, Color::Blue, 10),
        standard(3, Color::Yellow, 10),
        standard(4, Color::Black, 10),
    ];

    // Pose initiale : groupe de trois 10, le 10 noir reste en main
    partie.start_turn();
    let groupe_3 = Combinaison::new(vec![
        standard(1, Color::Red, 10),
        standard(2, Color::Blue, 10),
        standard(3, Color::Yellow, 10),
    ]);
    partie.play_melds(vec![groupe_3]).unwrap();
    partie.end_turn().unwrap();

    tour_pioche(&mut partie); // Bob pioche et passe

    // Alice complète son groupe avec le 10 noir resté en main
    partie.start_turn();
    let groupe_4 = Combinaison::new(vec![
        standard(1, Color::Red, 10),
        standard(2, Color::Blue, 10),
        standard(3, Color::Yellow, 10),
        standard(4, Color::Black, 10),
    ]);
    partie.submit_board(vec![groupe_4]).unwrap();
    assert!(partie.joueurs[0].main.is_empty());
    assert_eq!(partie.plateau.combinaisons.len(), 1);
    assert_eq!(partie.plateau.combinaisons[0].Tuiles.len(), 4);

    partie.end_turn().unwrap();
    assert_eq!(partie.winner(), Some(0));
}

#[test]
fn test_manipulation_cannot_remove_board_tiles() {
    let mut partie = partie_deux_joueurs();
    partie.joueurs[0].main = vec![
        standard(1, Color::Red, 10),
        standard(2, Color::Blue, 10),
        standard(3, Color::Yellow, 10),
        standard(4, Color::Black, 10),
        standard(5, Color::Red, 3), // reste en main
    ];

    partie.start_turn();
    let groupe_4 = Combinaison::new(vec![
        standard(1, Color::Red, 10),
        standard(2, Color::Blue, 10),
        standard(3, Color::Yellow, 10),
        standard(4, Color::Black, 10),
    ]); // 40 points
    partie.play_melds(vec![groupe_4]).unwrap();
    partie.end_turn().unwrap();

    tour_pioche(&mut partie); // Bob pioche et passe

    // Alice tente de reprendre le 10 noir du plateau : la tuile disparaîtrait
    partie.start_turn();
    let groupe_3 = Combinaison::new(vec![
        standard(1, Color::Red, 10),
        standard(2, Color::Blue, 10),
        standard(3, Color::Yellow, 10),
    ]);
    assert!(partie.submit_board(vec![groupe_3]).is_err());

    // Rien n'a bougé : plateau intact, main intacte
    assert_eq!(partie.plateau.combinaisons[0].Tuiles.len(), 4);
    assert_eq!(partie.joueurs[0].main.len(), 1);
}

#[test]
fn test_manipulation_rejected_before_initial_meld() {
    let mut partie = partie_deux_joueurs();
    partie.index_joueur_courant = 1; // Bob n'a pas encore fait sa pose initiale

    partie.start_turn();
    assert!(partie.submit_board(vec![]).is_err());
    assert!(partie.plateau.combinaisons.is_empty());
}

#[test]
fn test_manipulation_rejects_invalid_and_foreign_tiles() {
    let mut partie = partie_deux_joueurs();
    partie.joueurs[0].main = vec![
        standard(1, Color::Red, 10),
        standard(2, Color::Blue, 10),
        standard(3, Color::Yellow, 10),
    ];

    partie.start_turn();
    let main = partie.joueurs[0].main.clone();
    partie.play_melds(vec![Combinaison::new(main)]).unwrap(); // pose initiale 30 points
    partie.end_turn().unwrap();
    partie.index_joueur_courant = 0; // Alice, pose initiale déjà faite

    partie.start_turn();

    // Combinaison invalide (2 tuiles)
    let trop_court = Combinaison::new(vec![
        standard(1, Color::Red, 10),
        standard(2, Color::Blue, 10),
    ]);
    assert!(partie.submit_board(vec![trop_court]).is_err());

    // La tuile 99 n'est ni sur le plateau ni dans la main d'Alice (vide)
    let triche = Combinaison::new(vec![
        standard(1, Color::Red, 10),
        standard(2, Color::Blue, 10),
        standard(99, Color::Yellow, 10),
    ]);
    assert!(partie.submit_board(vec![triche]).is_err());

    // Le plateau n'a pas bougé
    assert_eq!(partie.plateau.combinaisons[0].Tuiles.len(), 3);
}

#[test]
fn test_draw_tile_ends_the_turn() {
    let mut partie = partie_deux_joueurs();
    partie.start_turn();
    let pioche_avant = partie.pioche.len();
    let main_avant = partie.joueurs[0].main.len();

    let tuile = partie
        .draw_tile()
        .unwrap()
        .expect("la pioche n'est pas vide");
    assert_eq!(partie.joueurs[0].main.len(), main_avant + 1);
    assert!(partie.joueurs[0].main.iter().any(|t| t.id == tuile.id));
    assert_eq!(partie.pioche.len(), pioche_avant - 1);

    // Piocher met fin au tour : passage au joueur suivant
    assert_eq!(partie.index_joueur_courant, 1);
    assert!(partie.plateau_sauvegarde.is_none());
    assert!(partie.main_sauvegarde.is_none());

    // Tour terminé : repiocher est refusé
    assert!(partie.draw_tile().is_err());
}

#[test]
fn test_draw_tile_passes_when_pool_is_empty() {
    let mut partie = partie_deux_joueurs();
    partie.pioche.clear();
    partie.start_turn();
    let main_avant = partie.joueurs[0].main.len();

    // Sac vide : le joueur passe son tour sans piocher
    assert!(partie.draw_tile().unwrap().is_none());
    assert_eq!(partie.joueurs[0].main.len(), main_avant);
    assert_eq!(partie.index_joueur_courant, 1);
}

#[test]
fn test_draw_tile_rejected_after_playing() {
    let mut partie = partie_deux_joueurs();
    partie.joueurs[0].main = vec![
        standard(1, Color::Red, 10),
        standard(2, Color::Blue, 10),
        standard(3, Color::Yellow, 10),
    ];
    partie.start_turn();
    let main = partie.joueurs[0].main.clone();
    partie.play_melds(vec![Combinaison::new(main)]).unwrap();

    // Le joueur a déjà posé : impossible de piocher
    assert!(partie.draw_tile().is_err());

    // Le tour peut quand même se terminer normalement
    partie.end_turn().unwrap();
    assert_eq!(partie.index_joueur_courant, 1);
}

#[test]
fn test_end_turn_requires_an_open_turn_and_an_action() {
    let mut partie = partie_deux_joueurs();

    // Sans tour ouvert, les actions sont refusées
    assert!(partie.end_turn().is_err());
    assert!(partie.draw_tile().is_err());
    assert!(partie.play_melds(vec![]).is_err());

    // Un tour sans action est refusé tant que le sac contient des tuiles :
    // le joueur doit jouer ou piocher
    partie.start_turn();
    assert!(partie.end_turn().is_err());

    // Piocher est l'action minimale : elle termine le tour
    partie.start_turn();
    assert!(partie.draw_tile().unwrap().is_some());
    assert_eq!(partie.index_joueur_courant, 1);

    // Rotation complète : retour au joueur 0
    partie.start_turn();
    assert!(partie.draw_tile().unwrap().is_some());
    assert_eq!(partie.index_joueur_courant, 0);
}

#[test]
fn test_end_turn_allows_passing_when_pool_is_empty() {
    let mut partie = partie_deux_joueurs();
    partie.pioche.clear();

    partie.start_turn();
    partie.end_turn().unwrap(); // passe autorisée à sac vide
    assert_eq!(partie.index_joueur_courant, 1);
}

#[test]
fn test_end_turn_cancels_on_invalid_board() {
    let mut partie = partie_deux_joueurs();
    partie.start_turn(); // sauvegarde : plateau vide

    // On force un plateau invalide, comme après une manipulation ratée
    partie.plateau = Plateau {
        combinaisons: vec![Combinaison::new(vec![
            standard(1, Color::Red, 5),
            standard(2, Color::Red, 6),
        ])],
    };
    let main_avant = partie.joueurs[0].main.clone();

    assert!(partie.end_turn().is_err());

    // Tour annulé : plateau restauré, main intacte, même joueur
    assert!(partie.plateau.combinaisons.is_empty());
    assert_eq!(partie.joueurs[0].main, main_avant);
    assert_eq!(partie.index_joueur_courant, 0);
    assert!(partie.plateau_sauvegarde.is_none());
    assert!(partie.main_sauvegarde.is_none());
}

#[test]
fn test_winner_when_hand_is_empty() {
    let mut partie = partie_deux_joueurs();
    assert_eq!(partie.winner(), None);

    partie.joueurs[0].main.clear();
    assert_eq!(partie.winner(), Some(0));
}
