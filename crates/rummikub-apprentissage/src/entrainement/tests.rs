use crate::donnees::{Exemple, ecrire_dataset, lire_dataset};
use crate::encodeur::{NB_TYPES, TAILLE_VECTEUR};
use crate::entrainement::{entrainer, separer_train_validation};

fn exemples_fictifs() -> Vec<Exemple> {
    let mut ex = Exemple {
        id_joueur: 0,
        entree: vec![0.0; TAILLE_VECTEUR],
        types_jouees: [false; NB_TYPES],
        piocher: false,
    };
    ex.entree[0] = 1.0;
    ex.types_jouees[3] = true;
    let mut ex2 = ex.clone();
    ex2.id_joueur = 1;
    ex2.entree[53] = 1.0;
    ex2.piocher = true;
    vec![ex, ex2]
}

#[test]
fn test_le_csv_fait_un_aller_retour_relie_a_donnees() {
    // On re-vérifie ici le format vu du côté entraînement : lire le CSV et
    // construire des lots doit rester stable (le split, les lots, tout
    // consomme ce format).
    let chemin = std::env::temp_dir().join("dataset-test-entrainement.csv");
    let exemples = exemples_fictifs();
    ecrire_dataset(chemin.to_string_lossy().as_ref(), &exemples).unwrap();
    let relus = lire_dataset(chemin.to_string_lossy().as_ref()).unwrap();
    assert_eq!(relus, exemples);
    let _ = std::fs::remove_file(&chemin);
}

#[test]
fn test_le_split_est_deterministe_et_couvant() {
    let exemples = exemples_fictifs();
    let (train, validation) = separer_train_validation(&exemples, 0.5, 7);
    assert_eq!(train.len() + validation.len(), 2);
    assert_eq!(validation.len(), 1);
    let (train_2, validation_2) = separer_train_validation(&exemples, 0.5, 7);
    assert_eq!(train, train_2);
    assert_eq!(validation, validation_2);
}

#[test]
fn test_lentraînement_baisse_la_loss_sur_un_dataset_trivial() {
    // Dataset synthétique : la cible = le premier bit de l'entrée. Un réseau
    // doit pouvoir apprendre ça sans effort : la loss doit baisser nettement.
    let mut exemples = Vec::new();
    for index in 0..120 {
        let mut entree = vec![0.0; TAILLE_VECTEUR];
        entree[0] = if index % 2 == 0 { 1.0 } else { 0.0 };
        let mut types_jouees = [false; NB_TYPES];
        types_jouees[0] = index % 2 == 0;
        exemples.push(Exemple {
            id_joueur: index % 2,
            entree,
            types_jouees,
            piocher: index % 2 != 0,
        });
    }

    let resultat = entrainer(&exemples, 3, 32, 0.01, 42).unwrap();
    assert_eq!(resultat.courbes.train.len(), 3);
    assert_eq!(resultat.courbes.validation.len(), 3);
    assert!(
        resultat.courbes.train[2] < resultat.courbes.train[0],
        "la loss d'entraînement doit baisser : {:?}",
        resultat.courbes
    );
}
