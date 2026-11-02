use super::*;
use crate::tuile::{Couleur, Tuile};

fn standard(id: u8, couleur: Couleur, valeur: u8) -> Tuile {
    Tuile::nouvelle_standard(id, couleur, valeur)
}

/// Cherche une combinaison dont les tuiles standard sont exactement
/// `attendues` (peu importe l'ordre), avec exactement `nb_jokers` jokers.
fn contient(combinaisons: &[Combinaison], attendues: &[(Couleur, u8)], nb_jokers: usize) -> bool {
    combinaisons.iter().any(|combinaison| {
        let standards: Vec<(Couleur, u8)> = combinaison
            .Tuiles
            .iter()
            .filter_map(|tuile| match tuile.Type {
                TuileType::Standard { couleur, valeur } => Some((couleur, valeur)),
                TuileType::Joker => None,
            })
            .collect();
        let jokers = combinaison
            .Tuiles
            .iter()
            .filter(|tuile| tuile.est_joker())
            .count();

        jokers == nb_jokers
            && standards.len() == attendues.len()
            && standards.iter().all(|paire| attendues.contains(paire))
    })
}

#[test]
fn test_enumere_groupes_et_suites_avec_jokers() {
    let main = vec![
        standard(1, Couleur::Rouge, 10),
        standard(2, Couleur::Bleu, 10),
        standard(3, Couleur::Jaune, 10),
        standard(4, Couleur::Rouge, 11),
        standard(5, Couleur::Rouge, 12),
        standard(6, Couleur::Rouge, 13),
        Tuile::nouveau_joker(7),
    ];

    let possibles = combinaisons_possibles(&main);
    // Déterministe : même main, même liste dans le même ordre
    assert_eq!(possibles, combinaisons_possibles(&main));

    // Le groupe des trois 10, seul puis étendu par le joker
    assert!(contient(
        &possibles,
        &[
            (Couleur::Rouge, 10),
            (Couleur::Bleu, 10),
            (Couleur::Jaune, 10)
        ],
        0
    ));
    assert!(contient(
        &possibles,
        &[
            (Couleur::Rouge, 10),
            (Couleur::Bleu, 10),
            (Couleur::Jaune, 10)
        ],
        1
    ));

    // La suite rouge 10-11-12-13, et le joker en 9 (9-10-11-12)
    assert!(contient(
        &possibles,
        &[
            (Couleur::Rouge, 10),
            (Couleur::Rouge, 11),
            (Couleur::Rouge, 12),
            (Couleur::Rouge, 13)
        ],
        0
    ));
    assert!(contient(
        &possibles,
        &[
            (Couleur::Rouge, 10),
            (Couleur::Rouge, 11),
            (Couleur::Rouge, 12)
        ],
        1
    ));
}

#[test]
fn test_les_doublons_exact_sont_interchangeables() {
    let main = vec![
        standard(1, Couleur::Rouge, 7),
        standard(2, Couleur::Rouge, 7),
        standard(3, Couleur::Bleu, 7),
        standard(4, Couleur::Jaune, 7),
    ];

    let possibles = combinaisons_possibles(&main);

    // Le groupe des 7, en une seule variante (deux 7 rouges identiques)
    assert!(contient(
        &possibles,
        &[(Couleur::Rouge, 7), (Couleur::Bleu, 7), (Couleur::Jaune, 7)],
        0
    ));

    // Aucune combinaison ne contient deux fois le 7 rouge : même couleur
    // interdite dans un groupe, même valeur interdite dans une suite
    for combinaison in &possibles {
        let rouges_7 = combinaison
            .Tuiles
            .iter()
            .filter(|tuile| {
                matches!(
                    tuile.Type,
                    TuileType::Standard {
                        couleur: Couleur::Rouge,
                        valeur: 7
                    }
                )
            })
            .count();
        assert!(rouges_7 <= 1);
    }
}

#[test]
fn test_avant_pose_initiale_seulement_30_points() {
    let mut partie =
        Partie::nouvelle_avec_graine(vec!["A".to_string(), "B".to_string()], 1).unwrap();
    partie.joueurs[0].main = vec![
        standard(1, Couleur::Rouge, 7),
        standard(2, Couleur::Bleu, 7),
        standard(3, Couleur::Jaune, 7),
    ]; // un groupe de 21 points seulement

    partie.ouvrir_tour();
    let coups = coups_possibles(&partie).unwrap();

    // Rien à poser (21 < 30) : il ne reste que piocher
    assert_eq!(coups, vec![Coup::Piocher]);
}

#[test]
fn test_apres_pose_initiale_les_poses_libres_sont_enumerees() {
    let mut partie =
        Partie::nouvelle_avec_graine(vec!["A".to_string(), "B".to_string()], 1).unwrap();
    partie.joueurs[0].main = vec![
        standard(1, Couleur::Rouge, 10),
        standard(2, Couleur::Bleu, 10),
        standard(3, Couleur::Jaune, 10),
        standard(4, Couleur::Rouge, 7),
        standard(5, Couleur::Bleu, 7),
        standard(6, Couleur::Jaune, 7),
    ];

    // Pose initiale : le groupe des 10 (30 points)
    partie.ouvrir_tour();
    let groupe_10 = Combinaison::nouvelle(vec![
        standard(1, Couleur::Rouge, 10),
        standard(2, Couleur::Bleu, 10),
        standard(3, Couleur::Jaune, 10),
    ]);
    jouer_coup(&mut partie, Coup::Poser(vec![groupe_10])).unwrap();
    partie.terminer_tour().unwrap();

    // Le joueur 1 pioche entre-temps, puis retour au joueur 0
    partie.ouvrir_tour();
    partie.piocher().unwrap().unwrap();
    partie.ouvrir_tour();

    let coups = coups_possibles(&partie).unwrap();

    // Le groupe des 7 (21 points) est maintenant un coup légal...
    assert!(coups.iter().any(|coup| match coup {
        Coup::Poser(combinaisons) => contient(
            combinaisons,
            &[(Couleur::Rouge, 7), (Couleur::Bleu, 7), (Couleur::Jaune, 7)],
            0
        ),
        Coup::Manipuler(_) | Coup::Terminer | Coup::Piocher => false,
    }));
    // ...et piocher termine la liste
    assert_eq!(coups.last(), Some(&Coup::Piocher));
}

#[test]
fn test_coups_possibles_exige_un_tour_ouvert() {
    let partie = Partie::nouvelle_avec_graine(vec!["A".to_string(), "B".to_string()], 1).unwrap();
    assert!(coups_possibles(&partie).is_err());
}

#[test]
fn test_jouer_coup_pose_et_pioche() {
    let mut partie =
        Partie::nouvelle_avec_graine(vec!["A".to_string(), "B".to_string()], 1).unwrap();
    partie.joueurs[0].main = vec![
        standard(1, Couleur::Rouge, 10),
        standard(2, Couleur::Bleu, 10),
        standard(3, Couleur::Jaune, 10),
    ];

    partie.ouvrir_tour();
    let groupe = Combinaison::nouvelle(partie.joueurs[0].main.clone());
    let points = jouer_coup(&mut partie, Coup::Poser(vec![groupe])).unwrap();
    assert_eq!(points, 30);
    assert!(partie.joueurs[0].main.is_empty());
    assert_eq!(partie.plateau.combinaisons.len(), 1);
    partie.terminer_tour().unwrap();

    // Piocher termine le tour : la pioche va au joueur 1, puis rotation
    // vers le joueur 0
    partie.ouvrir_tour();
    let points = jouer_coup(&mut partie, Coup::Piocher).unwrap();
    assert_eq!(points, 0);
    assert_eq!(partie.joueurs[1].main.len(), 15);
    assert_eq!(partie.index_joueur_courant, 0);
}

#[test]
fn test_poses_combinees_pour_la_pose_initiale() {
    let mut partie =
        Partie::nouvelle_avec_graine(vec!["A".to_string(), "B".to_string()], 1).unwrap();
    partie.joueurs[0].main = vec![
        standard(1, Couleur::Rouge, 7),
        standard(2, Couleur::Bleu, 7),
        standard(3, Couleur::Jaune, 7), // groupe des 7 : 21 points
        standard(4, Couleur::Rouge, 2),
        standard(5, Couleur::Rouge, 3),
        standard(6, Couleur::Rouge, 4), // suite 2-3-4 : 9 points
    ];

    partie.ouvrir_tour();
    let coups = coups_possibles(&partie).unwrap();

    // Aucune pose seule n'atteint 30, mais 21 + 9 = 30 : la pose combinée existe
    assert!(
        !coups
            .iter()
            .any(|coup| matches!(coup, Coup::Poser(un) if un.len() == 1))
    );
    let combinees: Vec<&Vec<Combinaison>> = coups
        .iter()
        .filter_map(|coup| match coup {
            Coup::Poser(combinaisons) if combinaisons.len() == 2 => Some(combinaisons),
            _ => None,
        })
        .collect();
    assert_eq!(combinees.len(), 1);

    // Elle passe les vraies règles et marque la pose initiale
    let pose = combinees[0].clone();
    let points = jouer_coup(&mut partie, Coup::Poser(pose)).unwrap();
    assert_eq!(points, 30);
    assert!(partie.joueur_courant().a_fait_pose_initiale);
    assert!(partie.joueurs[0].main.is_empty());
}

#[test]
fn test_poses_combinees_exigent_des_tuiles_disjointes() {
    let mut partie =
        Partie::nouvelle_avec_graine(vec!["A".to_string(), "B".to_string()], 1).unwrap();
    // Deux 7 rouges, un bleu, un jaune, un 5 et un 6 rouges : le groupe des 7
    // (21 points) et la suite 5-6-7 rouge (18 points) se disputent le 7 rouge
    partie.joueurs[0].main = vec![
        standard(1, Couleur::Rouge, 7),
        standard(2, Couleur::Rouge, 7),
        standard(3, Couleur::Bleu, 7),
        standard(4, Couleur::Jaune, 7),
        standard(5, Couleur::Rouge, 5),
        standard(6, Couleur::Rouge, 6),
    ];

    partie.ouvrir_tour();
    let coups = coups_possibles(&partie).unwrap();

    // 21 + 18 = 39 >= 30, mais les deux combinaisons partagent le 7 rouge
    // (exemplaire canonique) : aucune pose combinée n'est légale
    assert!(
        !coups
            .iter()
            .any(|coup| matches!(coup, Coup::Poser(c) if c.len() == 2))
    );
    assert_eq!(coups, vec![Coup::Piocher]);
}

#[test]
fn test_manipulation_ajout_dune_tuile_au_plateau() {
    let mut partie =
        Partie::nouvelle_avec_graine(vec!["A".to_string(), "B".to_string()], 1).unwrap();
    partie.joueurs[0].main = vec![
        standard(1, Couleur::Rouge, 10),
        standard(2, Couleur::Bleu, 10),
        standard(3, Couleur::Jaune, 10),
        standard(4, Couleur::Noir, 10),
    ];

    // Pose initiale : groupe de trois 10 (30 points)
    partie.ouvrir_tour();
    jouer_coup(
        &mut partie,
        Coup::Poser(vec![Combinaison::nouvelle(vec![
            standard(1, Couleur::Rouge, 10),
            standard(2, Couleur::Bleu, 10),
            standard(3, Couleur::Jaune, 10),
        ])]),
    )
    .unwrap();
    partie.terminer_tour().unwrap();
    partie.ouvrir_tour();
    partie.piocher().unwrap().unwrap();
    partie.ouvrir_tour();

    let coups = coups_possibles(&partie).unwrap();
    let manips: Vec<&Vec<Combinaison>> = coups
        .iter()
        .filter_map(|coup| match coup {
            Coup::Manipuler(etat) => Some(etat),
            _ => None,
        })
        .collect();

    // Une seule manipulation : compléter le groupe avec le 10 noir
    assert_eq!(manips.len(), 1);
    assert!(manips[0].iter().any(|c| c.points() == 40));

    // Elle passe les vraies règles du jeu
    jouer_coup(&mut partie, Coup::Manipuler(manips[0].clone())).unwrap();
    assert!(partie.joueurs[0].main.is_empty());
    assert_eq!(partie.plateau.combinaisons[0].Tuiles.len(), 4);
}

/// Une partie où le joueur 0 a posé [8R, 8B, joker] + [11R, 11B, 11Y]
/// (57 points) et où c'est à nouveau son tour, avec le 8 jaune en main
fn partie_avec_joker_liberable() -> Partie {
    let mut partie =
        Partie::nouvelle_avec_graine(vec!["A".to_string(), "B".to_string()], 1).unwrap();
    partie.joueurs[0].main = vec![
        standard(1, Couleur::Rouge, 8),
        standard(2, Couleur::Bleu, 8),
        Tuile::nouveau_joker(100),
        standard(3, Couleur::Rouge, 11),
        standard(4, Couleur::Bleu, 11),
        standard(5, Couleur::Jaune, 11),
        standard(6, Couleur::Jaune, 8), // la vraie tuile du joker
        standard(7, Couleur::Rouge, 3), // ne sert à aucune manipulation
    ];

    // Pose initiale : 24 + 33 = 57 points
    partie.ouvrir_tour();
    let pose_initiale = vec![
        Combinaison::nouvelle(vec![
            standard(1, Couleur::Rouge, 8),
            standard(2, Couleur::Bleu, 8),
            Tuile::nouveau_joker(100),
        ]),
        Combinaison::nouvelle(vec![
            standard(3, Couleur::Rouge, 11),
            standard(4, Couleur::Bleu, 11),
            standard(5, Couleur::Jaune, 11),
        ]),
    ];
    jouer_coup(&mut partie, Coup::Poser(pose_initiale)).unwrap();
    partie.terminer_tour().unwrap();

    // Le joueur 1 pioche (tour terminé), puis retour au joueur 0
    partie.ouvrir_tour();
    partie.piocher().unwrap().unwrap();
    partie.ouvrir_tour();
    partie
}

#[test]
fn test_manipulation_libere_le_joker_dun_groupe() {
    let partie = partie_avec_joker_liberable();

    let coups = coups_possibles(&partie).unwrap();
    let manips: Vec<Vec<Combinaison>> = coups
        .iter()
        .filter_map(|coup| match coup {
            Coup::Manipuler(etat) => Some(etat.clone()),
            _ => None,
        })
        .collect();

    // Deux manipulations : l'ajout simple du 8 jaune au groupe (4 tuiles,
    // joker toujours dedans), et la libération du joker (groupe des 8 sans
    // joker + joker rejoint le groupe des 11)
    assert_eq!(manips.len(), 2);
    assert!(manips.iter().any(|etat| {
        etat.iter()
            .any(|c| c.points() == 32 && c.Tuiles.iter().filter(|t| t.est_joker()).count() == 1)
    }));
    assert!(manips.iter().any(|etat| {
        etat.iter()
            .any(|c| c.points() == 24 && c.Tuiles.iter().all(|t| !t.est_joker()))
            && etat.iter().any(|c| c.points() == 44)
    }));

    // Chaque manipulation est acceptée par les vraies règles du jeu
    for etat in manips {
        let mut partie = partie_avec_joker_liberable();
        jouer_coup(&mut partie, Coup::Manipuler(etat)).unwrap();
        // le 8 jaune est posé, seule la tuile inutile reste en main
        assert_eq!(partie.joueur_courant().main.len(), 1);
    }
}

#[test]
fn test_pas_de_manipulation_avant_sa_propre_pose_initiale() {
    let mut partie =
        Partie::nouvelle_avec_graine(vec!["A".to_string(), "B".to_string()], 1).unwrap();

    // Le joueur 0 pose un groupe de 10 (30 points)
    partie.joueurs[0].main = vec![
        standard(1, Couleur::Rouge, 10),
        standard(2, Couleur::Bleu, 10),
        standard(3, Couleur::Jaune, 10),
    ];
    partie.ouvrir_tour();
    let pose = Coup::Poser(vec![Combinaison::nouvelle(partie.joueurs[0].main.clone())]);
    jouer_coup(&mut partie, pose).unwrap();
    partie.terminer_tour().unwrap();

    // Le joueur 1 n'a pas encore fait sa pose initiale : impossible de
    // manipuler, même si son 10 noir compléterait le groupe des 10
    partie.joueurs[1].main = vec![standard(4, Couleur::Noir, 10)];
    partie.ouvrir_tour();
    let coups = coups_possibles(&partie).unwrap();
    assert!(coups.iter().all(|coup| !matches!(coup, Coup::Manipuler(_))));
    assert_eq!(coups, vec![Coup::Piocher]);
}

#[test]
fn test_terminer_le_tour_est_un_coup() {
    let mut partie =
        Partie::nouvelle_avec_graine(vec!["A".to_string(), "B".to_string()], 1).unwrap();
    partie.joueurs[0].main = vec![
        standard(1, Couleur::Rouge, 10),
        standard(2, Couleur::Bleu, 10),
        standard(3, Couleur::Jaune, 10),
    ];

    // Avant d'avoir joué, avec une pioche non vide : pas de Terminer
    partie.ouvrir_tour();
    let coups = coups_possibles(&partie).unwrap();
    assert!(!coups.iter().any(|coup| matches!(coup, Coup::Terminer)));

    // Après une pose : Terminer devient disponible
    let groupe = Combinaison::nouvelle(partie.joueurs[0].main.clone());
    jouer_coup(&mut partie, Coup::Poser(vec![groupe])).unwrap();
    let coups = coups_possibles(&partie).unwrap();
    assert!(coups.iter().any(|coup| matches!(coup, Coup::Terminer)));

    // Et il fait ce qu'on attend : fin du tour, joueur suivant
    let points = jouer_coup(&mut partie, Coup::Terminer).unwrap();
    assert_eq!(points, 0);
    assert_eq!(partie.index_joueur_courant, 1);
}
