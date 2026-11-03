use super::*;
use rummikub_core::combinaison::Combinaison;
use rummikub_core::tuile::{Couleur, Tuile};

#[test]
fn test_la_cible_dune_pose_marque_ses_types() {
    let coup = Coup::Poser(vec![Combinaison::nouvelle(vec![
        Tuile::nouvelle_standard(1, Couleur::Rouge, 10),
        Tuile::nouvelle_standard(2, Couleur::Bleu, 10),
        Tuile::nouvelle_standard(3, Couleur::Jaune, 10),
    ])]);

    let (types_jouees, piocher) = cible_du_coup(&coup, &BitsetTuiles::default()).unwrap();
    assert!(!piocher);
    assert!(types_jouees[index_type(&Tuile::nouvelle_standard(0, Couleur::Rouge, 10))]);
    assert!(types_jouees[index_type(&Tuile::nouvelle_standard(0, Couleur::Bleu, 10))]);
    assert!(types_jouees[index_type(&Tuile::nouvelle_standard(0, Couleur::Jaune, 10))]);
    assert!(!types_jouees[index_type(&Tuile::nouvelle_standard(0, Couleur::Noir, 10))]);
}

#[test]
fn test_la_cible_dune_manipulation_ne_marque_que_les_tuiles_nouvelles() {
    // La tuile 1 est déjà sur le plateau : seule la tuile 2 (venue de la
    // main) doit être étiquetée
    let mut ids_plateau = BitsetTuiles::default();
    ids_plateau.inserer(1);

    let coup = Coup::Manipuler(vec![Combinaison::nouvelle(vec![
        Tuile::nouvelle_standard(1, Couleur::Rouge, 10),
        Tuile::nouvelle_standard(2, Couleur::Bleu, 10),
    ])]);

    let (types_jouees, piocher) = cible_du_coup(&coup, &ids_plateau).unwrap();
    assert!(!types_jouees[index_type(&Tuile::nouvelle_standard(0, Couleur::Rouge, 10))]);
    assert!(types_jouees[index_type(&Tuile::nouvelle_standard(0, Couleur::Bleu, 10))]);
    assert!(!piocher);
}

#[test]
fn test_la_pioche_et_terminer() {
    // Piocher : drapeau levé, aucun type
    let (types_jouees, piocher) = cible_du_coup(&Coup::Piocher, &BitsetTuiles::default()).unwrap();
    assert!(piocher);
    assert!(types_jouees.iter().all(|&vrai| !vrai));

    // Terminer : pas de cible du tout
    assert!(cible_du_coup(&Coup::Terminer, &BitsetTuiles::default()).is_none());
}

#[test]
fn test_le_csv_fait_un_aller_retour() {
    let mut types_un = [false; NB_TYPES];
    types_un[9] = true;
    let exemples = vec![
        Exemple {
            id_joueur: 0,
            entree: vec![0.5; TAILLE_VECTEUR],
            types_jouees: types_un,
            piocher: false,
        },
        Exemple {
            id_joueur: 1,
            entree: vec![1.0; TAILLE_VECTEUR],
            types_jouees: [false; NB_TYPES],
            piocher: true,
        },
    ];

    let chemin = std::env::temp_dir().join("dataset-test-rummikub.csv");
    ecrire_dataset(chemin.to_string_lossy().as_ref(), &exemples).unwrap();
    let relus = lire_dataset(chemin.to_string_lossy().as_ref()).unwrap();

    assert_eq!(relus, exemples);
    let _ = std::fs::remove_file(&chemin);
}

#[test]
fn test_les_statistiques_comptent_les_classes() {
    let mut types_un = [false; NB_TYPES];
    types_un[0] = true;
    let exemples = vec![
        Exemple {
            id_joueur: 0,
            entree: vec![0.0; TAILLE_VECTEUR],
            types_jouees: types_un,
            piocher: false,
        },
        Exemple {
            id_joueur: 1,
            entree: vec![0.0; TAILLE_VECTEUR],
            types_jouees: [false; NB_TYPES],
            piocher: true,
        },
    ];

    let stats = statistiques(&exemples);
    assert_eq!(stats.exemples, 2);
    assert_eq!(stats.poses, 1);
    assert_eq!(stats.pioches, 1);
}

#[test]
fn test_la_generation_produit_des_exemples_des_deux_joueurs() {
    let ismcts = IsMcts {
        simulations: 10,
        threads: 1,
        ..Default::default()
    };

    let exemples = generer_dataset(1, &ismcts).unwrap();

    assert!(!exemples.is_empty());
    assert!(
        exemples
            .iter()
            .all(|exemple| exemple.entree.len() == TAILLE_VECTEUR)
    );
    assert!(exemples.iter().any(|exemple| exemple.id_joueur == 0));
    assert!(exemples.iter().any(|exemple| exemple.id_joueur == 1));
    // Chaque exemple porte une décision : pose ou pioche
    assert!(exemples.iter().any(|exemple| exemple.piocher));
    assert!(
        exemples
            .iter()
            .any(|exemple| exemple.types_jouees.iter().any(|&vrai| vrai))
    );
}
