use super::*;
use crate::tuile::Couleur;

fn standard(id: u8, couleur: Couleur, valeur: u8) -> Tuile {
    Tuile::nouvelle_standard(id, couleur, valeur)
}

fn partie_deux_joueurs() -> Partie {
    Partie::nouvelle(vec!["Alice".to_string(), "Bob".to_string()]).unwrap()
}

/// Tour minimal légal : le joueur pioche, ce qui termine son tour
fn tour_pioche(partie: &mut Partie) {
    partie.ouvrir_tour();
    assert!(partie.piocher().unwrap().is_some());
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
    assert!(Partie::nouvelle(vec!["Alice".to_string()]).is_err());
    assert!(
        Partie::nouvelle(vec![
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
        standard(1, Couleur::Rouge, 7),
        standard(2, Couleur::Bleu, 7),
        standard(3, Couleur::Jaune, 7),
    ]; // 21 points seulement

    partie.ouvrir_tour();
    let combinaison = Combinaison::nouvelle(partie.joueurs[0].main.clone());
    assert!(partie.poser_combinaisons(vec![combinaison]).is_err());

    // Rien n'a été posé, la main est intacte
    assert!(partie.plateau.combinaisons.is_empty());
    assert_eq!(partie.joueurs[0].main.len(), 3);
    assert!(!partie.joueurs[0].a_fait_pose_initiale);
}

#[test]
fn test_play_melds_after_initial_needs_no_30_points() {
    let mut partie = partie_deux_joueurs();
    partie.joueurs[0].main = vec![
        standard(1, Couleur::Rouge, 10),
        standard(2, Couleur::Bleu, 10),
        standard(3, Couleur::Jaune, 10),
        standard(4, Couleur::Rouge, 11),
        standard(5, Couleur::Rouge, 12),
        standard(6, Couleur::Rouge, 13),
    ];

    // Pose initiale : groupe de trois 10 (30 points)
    partie.ouvrir_tour();
    let groupe = Combinaison::nouvelle(vec![
        standard(1, Couleur::Rouge, 10),
        standard(2, Couleur::Bleu, 10),
        standard(3, Couleur::Jaune, 10),
    ]);
    partie.poser_combinaisons(vec![groupe]).unwrap();
    partie.terminer_tour().unwrap();

    tour_pioche(&mut partie); // Bob pioche et passe

    // Alice rejoue une suite de 36 points : plus d'exigence des 30 points
    partie.ouvrir_tour();
    let suite = Combinaison::nouvelle(vec![
        standard(4, Couleur::Rouge, 11),
        standard(5, Couleur::Rouge, 12),
        standard(6, Couleur::Rouge, 13),
    ]);
    let points = partie.poser_combinaisons(vec![suite]).unwrap();
    assert_eq!(points, 36);
    assert!(partie.joueurs[0].main.is_empty());
    assert_eq!(partie.plateau.combinaisons.len(), 2);

    partie.terminer_tour().unwrap();
    assert_eq!(partie.gagnant(), Some(0));
}

#[test]
fn test_manipulation_extends_a_board_meld() {
    let mut partie = partie_deux_joueurs();
    partie.joueurs[0].main = vec![
        standard(1, Couleur::Rouge, 10),
        standard(2, Couleur::Bleu, 10),
        standard(3, Couleur::Jaune, 10),
        standard(4, Couleur::Noir, 10),
    ];

    // Pose initiale : groupe de trois 10, le 10 noir reste en main
    partie.ouvrir_tour();
    let groupe_3 = Combinaison::nouvelle(vec![
        standard(1, Couleur::Rouge, 10),
        standard(2, Couleur::Bleu, 10),
        standard(3, Couleur::Jaune, 10),
    ]);
    partie.poser_combinaisons(vec![groupe_3]).unwrap();
    partie.terminer_tour().unwrap();

    tour_pioche(&mut partie); // Bob pioche et passe

    // Alice complète son groupe avec le 10 noir resté en main
    partie.ouvrir_tour();
    let groupe_4 = Combinaison::nouvelle(vec![
        standard(1, Couleur::Rouge, 10),
        standard(2, Couleur::Bleu, 10),
        standard(3, Couleur::Jaune, 10),
        standard(4, Couleur::Noir, 10),
    ]);
    partie.soumettre_plateau(vec![groupe_4]).unwrap();
    assert!(partie.joueurs[0].main.is_empty());
    assert_eq!(partie.plateau.combinaisons.len(), 1);
    assert_eq!(partie.plateau.combinaisons[0].Tuiles.len(), 4);

    partie.terminer_tour().unwrap();
    assert_eq!(partie.gagnant(), Some(0));
}

#[test]
fn test_manipulation_cannot_remove_board_tiles() {
    let mut partie = partie_deux_joueurs();
    partie.joueurs[0].main = vec![
        standard(1, Couleur::Rouge, 10),
        standard(2, Couleur::Bleu, 10),
        standard(3, Couleur::Jaune, 10),
        standard(4, Couleur::Noir, 10),
        standard(5, Couleur::Rouge, 3), // reste en main
    ];

    partie.ouvrir_tour();
    let groupe_4 = Combinaison::nouvelle(vec![
        standard(1, Couleur::Rouge, 10),
        standard(2, Couleur::Bleu, 10),
        standard(3, Couleur::Jaune, 10),
        standard(4, Couleur::Noir, 10),
    ]); // 40 points
    partie.poser_combinaisons(vec![groupe_4]).unwrap();
    partie.terminer_tour().unwrap();

    tour_pioche(&mut partie); // Bob pioche et passe

    // Alice tente de reprendre le 10 noir du plateau : la tuile disparaîtrait
    partie.ouvrir_tour();
    let groupe_3 = Combinaison::nouvelle(vec![
        standard(1, Couleur::Rouge, 10),
        standard(2, Couleur::Bleu, 10),
        standard(3, Couleur::Jaune, 10),
    ]);
    assert!(partie.soumettre_plateau(vec![groupe_3]).is_err());

    // Rien n'a bougé : plateau intact, main intacte
    assert_eq!(partie.plateau.combinaisons[0].Tuiles.len(), 4);
    assert_eq!(partie.joueurs[0].main.len(), 1);
}

#[test]
fn test_manipulation_rejected_before_initial_meld() {
    let mut partie = partie_deux_joueurs();
    partie.index_joueur_courant = 1; // Bob n'a pas encore fait sa pose initiale

    partie.ouvrir_tour();
    assert!(partie.soumettre_plateau(vec![]).is_err());
    assert!(partie.plateau.combinaisons.is_empty());
}

#[test]
fn test_manipulation_rejects_invalid_and_foreign_tiles() {
    let mut partie = partie_deux_joueurs();
    partie.joueurs[0].main = vec![
        standard(1, Couleur::Rouge, 10),
        standard(2, Couleur::Bleu, 10),
        standard(3, Couleur::Jaune, 10),
    ];

    partie.ouvrir_tour();
    let main = partie.joueurs[0].main.clone();
    partie
        .poser_combinaisons(vec![Combinaison::nouvelle(main)])
        .unwrap(); // pose initiale 30 points
    partie.terminer_tour().unwrap();
    partie.index_joueur_courant = 0; // Alice, pose initiale déjà faite

    partie.ouvrir_tour();

    // Combinaison invalide (2 tuiles)
    let trop_court = Combinaison::nouvelle(vec![
        standard(1, Couleur::Rouge, 10),
        standard(2, Couleur::Bleu, 10),
    ]);
    assert!(partie.soumettre_plateau(vec![trop_court]).is_err());

    // La tuile 99 n'est ni sur le plateau ni dans la main d'Alice (vide)
    let triche = Combinaison::nouvelle(vec![
        standard(1, Couleur::Rouge, 10),
        standard(2, Couleur::Bleu, 10),
        standard(99, Couleur::Jaune, 10),
    ]);
    assert!(partie.soumettre_plateau(vec![triche]).is_err());

    // Le plateau n'a pas bougé
    assert_eq!(partie.plateau.combinaisons[0].Tuiles.len(), 3);
}

#[test]
fn test_draw_tile_ends_the_turn() {
    let mut partie = partie_deux_joueurs();
    partie.ouvrir_tour();
    let pioche_avant = partie.pioche.len();
    let main_avant = partie.joueurs[0].main.len();

    let tuile = partie.piocher().unwrap().expect("la pioche n'est pas vide");
    assert_eq!(partie.joueurs[0].main.len(), main_avant + 1);
    assert!(partie.joueurs[0].main.iter().any(|t| t.id == tuile.id));
    assert_eq!(partie.pioche.len(), pioche_avant - 1);

    // Piocher met fin au tour : passage au joueur suivant
    assert_eq!(partie.index_joueur_courant, 1);
    assert_eq!(partie.tour, Tour::Repos);

    // Tour terminé : repiocher est refusé
    assert!(partie.piocher().is_err());
}

#[test]
fn test_draw_tile_passes_when_pool_is_empty() {
    let mut partie = partie_deux_joueurs();
    partie.pioche.clear();
    partie.ouvrir_tour();
    let main_avant = partie.joueurs[0].main.len();

    // Sac vide : le joueur passe son tour sans piocher
    assert!(partie.piocher().unwrap().is_none());
    assert_eq!(partie.joueurs[0].main.len(), main_avant);
    assert_eq!(partie.index_joueur_courant, 1);
}

#[test]
fn test_draw_tile_rejected_after_playing() {
    let mut partie = partie_deux_joueurs();
    partie.joueurs[0].main = vec![
        standard(1, Couleur::Rouge, 10),
        standard(2, Couleur::Bleu, 10),
        standard(3, Couleur::Jaune, 10),
    ];
    partie.ouvrir_tour();
    let main = partie.joueurs[0].main.clone();
    partie
        .poser_combinaisons(vec![Combinaison::nouvelle(main)])
        .unwrap();

    // Le joueur a déjà posé : impossible de piocher
    assert!(partie.piocher().is_err());

    // Le tour peut quand même se terminer normalement
    partie.terminer_tour().unwrap();
    assert_eq!(partie.index_joueur_courant, 1);
}

#[test]
fn test_end_turn_requires_an_open_turn_and_an_action() {
    let mut partie = partie_deux_joueurs();

    // Sans tour ouvert, les actions sont refusées
    assert!(partie.terminer_tour().is_err());
    assert!(partie.piocher().is_err());
    assert!(partie.poser_combinaisons(vec![]).is_err());

    // Un tour sans action est refusé tant que le sac contient des tuiles :
    // le joueur doit jouer ou piocher
    partie.ouvrir_tour();
    assert!(partie.terminer_tour().is_err());

    // Piocher est l'action minimale : elle termine le tour
    partie.ouvrir_tour();
    assert!(partie.piocher().unwrap().is_some());
    assert_eq!(partie.index_joueur_courant, 1);

    // Rotation complète : retour au joueur 0
    partie.ouvrir_tour();
    assert!(partie.piocher().unwrap().is_some());
    assert_eq!(partie.index_joueur_courant, 0);
}

#[test]
fn test_end_turn_allows_passing_when_pool_is_empty() {
    let mut partie = partie_deux_joueurs();
    partie.pioche.clear();

    partie.ouvrir_tour();
    partie.terminer_tour().unwrap(); // passe autorisée à sac vide
    assert_eq!(partie.index_joueur_courant, 1);
}

#[test]
fn test_end_turn_cancels_on_invalid_board() {
    let mut partie = partie_deux_joueurs();
    partie.ouvrir_tour(); // sauvegarde : plateau vide

    // On force un plateau invalide, comme après une manipulation ratée
    partie.plateau = Plateau {
        combinaisons: vec![Combinaison::nouvelle(vec![
            standard(1, Couleur::Rouge, 5),
            standard(2, Couleur::Rouge, 6),
        ])],
    };
    let main_avant = partie.joueurs[0].main.clone();

    assert!(partie.terminer_tour().is_err());

    // Tour annulé : plateau restauré, main intacte, même joueur
    assert!(partie.plateau.combinaisons.is_empty());
    assert_eq!(partie.joueurs[0].main, main_avant);
    assert_eq!(partie.index_joueur_courant, 0);
    assert_eq!(partie.tour, Tour::Repos);
}

#[test]
fn test_winner_when_hand_is_empty() {
    let mut partie = partie_deux_joueurs();
    assert_eq!(partie.gagnant(), None);

    partie.joueurs[0].main.clear();
    assert_eq!(partie.gagnant(), Some(0));
}

#[test]
fn test_une_partie_est_clonable() {
    let mut partie =
        Partie::nouvelle_avec_graine(vec!["Alice".to_string(), "Bob".to_string()], 5).unwrap();
    let copie = partie.clone();

    // La partie originale avance, la copie reste figée à l'instant initial
    partie.ouvrir_tour();
    partie.piocher().unwrap().unwrap();

    assert!(matches!(copie.tour, Tour::Repos));
    assert_eq!(copie.index_joueur_courant, 0);
    assert_eq!(copie.pioche.len(), 106 - 2 * 14);
    assert_eq!(copie.joueurs[0].main.len(), 14);
}

/// Une main sans la moindre combinaison possible : que des paires de deux
/// couleurs par valeur, et des valeurs espacées de 3 pour éviter les suites.
fn main_sans_combinaison(ids: &mut u8, valeurs: [u8; 4]) -> Vec<Tuile> {
    let mut main = Vec::new();
    for valeur in valeurs {
        for couleur in [Couleur::Rouge, Couleur::Bleu] {
            main.push(Tuile::nouvelle_standard(*ids, couleur, valeur));
            *ids += 1;
            main.push(Tuile::nouvelle_standard(*ids, couleur, valeur));
            *ids += 1;
        }
    }
    main
}

#[test]
fn test_la_partie_se_termine_par_blocage() {
    let mut partie =
        Partie::nouvelle_avec_graine(vec!["A".to_string(), "B".to_string()], 1).unwrap();
    partie.pioche.clear(); // sac vide

    let mut ids = 0;
    // Joueur 0 : valeur de main 4 × (1 + 4 + 7 + 10) = 88
    partie.joueurs[0].main = main_sans_combinaison(&mut ids, [1, 4, 7, 10]);
    // Joueur 1 : valeur de main 4 × (2 + 5 + 8 + 11) = 104
    partie.joueurs[1].main = main_sans_combinaison(&mut ids, [2, 5, 8, 11]);

    // Une passe ne suffit pas : la partie n'est pas encore terminée
    partie.ouvrir_tour();
    partie.piocher().unwrap();
    assert!(partie.gagnant().is_none());

    // Après un tour complet sans pose (2 joueurs), le plus faible gagne
    partie.ouvrir_tour();
    partie.piocher().unwrap();
    assert!(partie.est_terminee());
    assert_eq!(partie.gagnant(), Some(0));
}

#[test]
fn test_le_blocage_avec_egalite_donne_une_partie_nulle() {
    let mut partie =
        Partie::nouvelle_avec_graine(vec!["A".to_string(), "B".to_string()], 1).unwrap();
    partie.pioche.clear();

    let mut ids = 0;
    partie.joueurs[0].main = main_sans_combinaison(&mut ids, [1, 4, 7, 10]); // 88 points
    partie.joueurs[1].main = main_sans_combinaison(&mut ids, [1, 4, 7, 10]); // 88 points aussi

    partie.ouvrir_tour();
    partie.piocher().unwrap();
    partie.ouvrir_tour();
    partie.piocher().unwrap();

    assert!(partie.est_terminee());
    assert_eq!(partie.gagnant(), None); // égalité : partie nulle
}

#[test]
fn test_la_main_vide_bat_le_blocage() {
    let mut partie =
        Partie::nouvelle_avec_graine(vec!["A".to_string(), "B".to_string()], 1).unwrap();
    partie.pioche.clear();

    let mut ids = 0;
    partie.joueurs[0].main = main_sans_combinaison(&mut ids, [1, 4, 7, 10]); // 88 points
    partie.joueurs[1].main = vec![]; // main vide : victoire immédiate

    partie.ouvrir_tour();
    partie.piocher().unwrap();
    partie.ouvrir_tour();
    partie.piocher().unwrap();

    // Même en situation de blocage, la main vide reste la victoire prioritaire
    assert_eq!(partie.gagnant(), Some(1));
}

#[test]
fn test_meme_graine_meme_distribution() {
    let noms = vec!["Alice".to_string(), "Bob".to_string()];
    let partie_a = Partie::nouvelle_avec_graine(noms.clone(), 42).unwrap();
    let partie_b = Partie::nouvelle_avec_graine(noms, 42).unwrap();

    // Même graine : les mains distribuées sont identiques
    assert_eq!(partie_a.joueurs[0].main, partie_b.joueurs[0].main);
    assert_eq!(partie_a.joueurs[1].main, partie_b.joueurs[1].main);

    // Graine différente : distribution différente (pratiquement certain)
    let partie_c =
        Partie::nouvelle_avec_graine(vec!["Alice".to_string(), "Bob".to_string()], 43).unwrap();
    assert_ne!(partie_a.joueurs[0].main, partie_c.joueurs[0].main);
}

#[test]
fn test_vue_pour_ne_montrer_que_sa_propre_main() {
    let mut partie = partie_deux_joueurs();
    partie.joueurs[0].main = vec![
        standard(1, Couleur::Rouge, 10),
        standard(2, Couleur::Bleu, 10),
        standard(3, Couleur::Jaune, 10),
    ];

    let vue_alice = partie.vue_pour(0).unwrap();

    // Alice voit sa propre main en détail
    assert_eq!(vue_alice.joueurs[0].main.as_ref().unwrap().len(), 3);
    assert_eq!(vue_alice.joueurs[0].taille_main, 3);

    // Bob n'est visible que par la taille de sa main
    assert!(vue_alice.joueurs[1].main.is_none());
    assert_eq!(vue_alice.joueurs[1].taille_main, 14);

    // Le plateau, la taille de la pioche et l'état du tour sont publics
    assert!(vue_alice.plateau.combinaisons.is_empty());
    assert_eq!(vue_alice.taille_pioche, 106 - 2 * 14);
    assert_eq!(vue_alice.index_joueur_courant, 0);
    assert_eq!(vue_alice.gagnant, None);

    // Identifiant inconnu : erreur
    assert!(partie.vue_pour(9).is_err());
}
