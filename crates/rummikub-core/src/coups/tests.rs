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
        Coup::Piocher => false,
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
