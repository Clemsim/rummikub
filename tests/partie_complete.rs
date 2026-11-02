//! Test d'intégration : une partie complète jouée uniquement via l'API
//! publique de la bibliothèque, pour vérifier qu'elle est bien utilisable
//! depuis l'extérieur du crate.

use rummikub_core::combinaison::Combinaison;
use rummikub_core::partie::{Partie, Tour};
use rummikub_core::tuile::{Couleur, Tuile};

#[test]
fn test_full_game_to_victory() {
    let mut partie = Partie::nouvelle(vec!["Alice".to_string(), "Bob".to_string()])
        .expect("2 joueurs est une configuration valide");

    // Alice reçoit un groupe de 4 dizaines
    partie.joueurs[0].main = vec![
        Tuile::nouvelle_standard(200, Couleur::Rouge, 10),
        Tuile::nouvelle_standard(201, Couleur::Bleu, 10),
        Tuile::nouvelle_standard(202, Couleur::Jaune, 10),
        Tuile::nouvelle_standard(203, Couleur::Noir, 10),
    ];

    // Alice ouvre son tour et fait sa pose initiale (40 points)
    partie.ouvrir_tour();
    let groupe = Combinaison::nouvelle(partie.joueurs[0].main.clone());
    partie.poser_combinaisons(vec![groupe]).unwrap();
    partie.terminer_tour().unwrap();

    // Bob pioche, ce qui termine son tour et rend la main à Alice
    partie.ouvrir_tour();
    assert!(partie.piocher().unwrap().is_some());
    assert_eq!(partie.index_joueur_courant, 0);

    // Alice n'a plus de tuiles : elle a gagné, le tour est fermé
    assert_eq!(partie.gagnant(), Some(0));
    assert_eq!(partie.tour, Tour::Repos);
}
