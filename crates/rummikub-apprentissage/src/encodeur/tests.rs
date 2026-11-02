use super::*;
use rummikub_core::combinaison::Combinaison;
use rummikub_core::tuile::Couleur;

fn partie_deux_joueurs() -> Partie {
    Partie::nouvelle_avec_graine(vec!["A".to_string(), "B".to_string()], 1).unwrap()
}

fn index_standard(couleur: Couleur, valeur: u8) -> usize {
    couleur as usize * 13 + valeur as usize - 1
}

#[test]
fn test_le_vecteur_a_une_taille_fixe_et_est_deterministe() {
    let mut partie = partie_deux_joueurs();
    let premier = encoder(&partie, 0).unwrap();
    assert_eq!(premier.len(), TAILLE_VECTEUR);
    assert_eq!(premier, encoder(&partie, 0).unwrap());

    // Même taille quelle que soit la situation
    partie.ouvrir_tour();
    partie.piocher().unwrap().unwrap();
    assert_eq!(encoder(&partie, 1).unwrap().len(), TAILLE_VECTEUR);
}

#[test]
fn test_la_main_est_encodee_par_type() {
    let mut partie = partie_deux_joueurs();
    partie.joueurs[0].main = vec![
        Tuile::nouvelle_standard(1, Couleur::Rouge, 10),
        Tuile::nouvelle_standard(2, Couleur::Bleu, 10),
        Tuile::nouveau_joker(3),
    ];

    let vecteur = encoder(&partie, 0).unwrap();

    // Comptage /2 : un exemplaire vaut 0.5 ; le joker vit à l'index 52
    assert_eq!(vecteur[index_standard(Couleur::Rouge, 10)], 0.5);
    assert_eq!(vecteur[index_standard(Couleur::Bleu, 10)], 0.5);
    assert_eq!(vecteur[NB_TYPES - 1], 0.5); // le joker
    assert_eq!(vecteur[index_standard(Couleur::Rouge, 1)], 0.0);
}

#[test]
fn test_deux_exemplaires_remplissent_la_case() {
    let mut partie = partie_deux_joueurs();
    partie.joueurs[0].main = vec![
        Tuile::nouvelle_standard(1, Couleur::Rouge, 7),
        Tuile::nouvelle_standard(2, Couleur::Rouge, 7),
    ];

    let vecteur = encoder(&partie, 0).unwrap();
    assert_eq!(vecteur[index_standard(Couleur::Rouge, 7)], 1.0);
}

#[test]
fn test_le_plateau_est_encode_a_sa_place() {
    let mut partie = partie_deux_joueurs();
    partie.joueurs[0].main = vec![
        Tuile::nouvelle_standard(1, Couleur::Rouge, 10),
        Tuile::nouvelle_standard(2, Couleur::Bleu, 10),
        Tuile::nouvelle_standard(3, Couleur::Jaune, 10),
    ];

    // Pose initiale : le groupe des trois 10 (30 points)
    partie.ouvrir_tour();
    partie
        .poser_combinaisons(vec![Combinaison::nouvelle(partie.joueurs[0].main.clone())])
        .unwrap();

    let vecteur = encoder(&partie, 0).unwrap();
    assert_eq!(vecteur[NB_TYPES + index_standard(Couleur::Rouge, 10)], 0.5);
    assert_eq!(vecteur[NB_TYPES + index_standard(Couleur::Bleu, 10)], 0.5);
}

#[test]
fn test_la_main_des_adversaires_ne_fuit_pas() {
    let mut partie = partie_deux_joueurs();
    let vecteur_avant = encoder(&partie, 0).unwrap();

    // On remplace le CONTENU de la main adverse en gardant sa taille : la
    // vue du joueur 0 ne doit rien révéler de ce contenu
    partie.joueurs[1].main = (0..14)
        .map(|decalage| Tuile::nouveau_joker(90 + decalage))
        .collect();

    assert_eq!(vecteur_avant, encoder(&partie, 0).unwrap());
}

#[test]
fn test_les_adversaires_occupent_leurs_emplacements() {
    let mut partie = partie_deux_joueurs();

    // Le joueur 0 pioche : sa main passe à 15 (0.75 en normalisé /20)
    partie.ouvrir_tour();
    partie.piocher().unwrap().unwrap();

    let vecteur = encoder(&partie, 1).unwrap();
    assert_eq!(vecteur[2 * NB_TYPES + 1], 15.0 / 20.0); // taille de l'adversaire
    assert_eq!(vecteur[2 * NB_TYPES + 2], 0.0); // pas encore de pose initiale

    // À 2 joueurs, les emplacements d'adversaires inutilisés valent 0
    assert_eq!(vecteur[2 * NB_TYPES + 3], 0.0);
    assert_eq!(vecteur[2 * NB_TYPES + 4], 0.0);
}
