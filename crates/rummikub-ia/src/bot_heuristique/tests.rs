use super::*;
use rummikub_core::tuile::{Couleur, Tuile};

fn standard(id: u8, couleur: Couleur, valeur: u8) -> Tuile {
    Tuile::nouvelle_standard(id, couleur, valeur)
}

fn partie_deux_joueurs() -> Partie {
    Partie::nouvelle_avec_graine(vec!["Alice".to_string(), "Bob".to_string()], 7).unwrap()
}

#[test]
fn test_choisit_la_pose_la_plus_avantageuse() {
    let mut partie = partie_deux_joueurs();
    partie.joueurs[0].main = vec![
        standard(1, Couleur::Rouge, 10),
        standard(2, Couleur::Bleu, 10),
        standard(3, Couleur::Jaune, 10), // groupe de 10 : 30 points
        standard(4, Couleur::Rouge, 11),
        standard(5, Couleur::Rouge, 12),
        standard(6, Couleur::Rouge, 13), // suite 10-11-12-13 : 46 points
    ];

    partie.ouvrir_tour();
    let coup = choisir_coup(&partie).unwrap().expect("une pose existe");

    let Coup::Poser(combinaisons) = &coup else {
        panic!("le bot devrait proposer une pose");
    };
    let points: u32 = combinaisons.iter().map(|c| c.points()).sum();
    assert_eq!(points, 46); // la suite de 4 bat le groupe de 3
}

#[test]
fn test_pioche_quand_aucune_pose_n_est_possible() {
    let mut partie = partie_deux_joueurs();
    // Avant la pose initiale, rien ne vaut 30 points dans cette main
    partie.joueurs[0].main = vec![
        standard(1, Couleur::Rouge, 7),
        standard(2, Couleur::Bleu, 7),
        standard(3, Couleur::Jaune, 7),
    ];

    let pioche_avant = partie.pioche.len();
    jouer_tour(&mut partie).unwrap();

    // Le bot a pioché, ce qui a terminé son tour
    assert_eq!(partie.joueurs[0].main.len(), 4);
    assert_eq!(partie.pioche.len(), pioche_avant - 1);
    assert_eq!(partie.index_joueur_courant, 1);
}
