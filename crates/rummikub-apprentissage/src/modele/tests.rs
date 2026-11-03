use super::{NB_SORTIES, PolitiqueConfig};
use crate::encodeur::TAILLE_VECTEUR;
use burn::module::Module;
use burn::prelude::*;
use burn::store::ModuleRecord;

#[test]
fn test_la_forme_de_sortie_est_un_logits_par_type_et_pioche() {
    let device = Device::flex();
    let modele = PolitiqueConfig::new(TAILLE_VECTEUR, NB_SORTIES).init(&device);

    let entree = Tensor::<2>::zeros([4, TAILLE_VECTEUR], &device);
    let logits = modele.forward(entree);

    assert_eq!(logits.dims(), [4, NB_SORTIES]);
}

#[test]
fn test_les_logits_initialises_sont_finis_et_proches_de_zero() {
    // À l'initialisation, les sorties doivent être de petits nombres finis :
    // si elles explosent dès le départ, l'entraînement part en vrille.
    let device = Device::flex();
    let modele = PolitiqueConfig::new(TAILLE_VECTEUR, NB_SORTIES).init(&device);

    let entree = Tensor::<2>::zeros([8, TAILLE_VECTEUR], &device);
    let logits = modele.forward(entree);
    let valeurs = logits.into_data().iter::<f32>().collect::<Vec<_>>();

    assert!(valeurs.iter().all(|v| v.is_finite()));
    assert!(valeurs.iter().all(|v| v.abs() < 10.0));
}

#[test]
fn test_la_taille_cachee_est_parametrable() {
    let device = Device::flex();
    let modele = PolitiqueConfig::new(TAILLE_VECTEUR, NB_SORTIES)
        .with_taille_cachee(32)
        .init(&device);

    let entree = Tensor::<2>::zeros([1, TAILLE_VECTEUR], &device);
    assert_eq!(modele.forward(entree).dims(), [1, NB_SORTIES]);
}

#[test]
fn test_le_jeu_daller_retour_du_record_conserve_les_logits() {
    let device = Device::flex();
    let modele = PolitiqueConfig::new(TAILLE_VECTEUR, NB_SORTIES).init(&device);

    let entree = Tensor::<2>::zeros([2, TAILLE_VECTEUR], &device);
    let attendus = modele
        .forward(entree.clone())
        .into_data()
        .iter::<f32>()
        .collect::<Vec<_>>();

    // Sauvegarde + rechargement via le format natif burnpack : les poids
    // doivent produire exactement les mêmes logits après relecture.
    let chemin = std::env::temp_dir().join("politique-test-rummikub.burnpack");
    let _ = std::fs::remove_file(&chemin);
    modele
        .clone()
        .into_record()
        .save(&chemin)
        .expect("sauvegarde du record");

    let record = ModuleRecord::load(&chemin).expect("relecture du record");
    let relu = PolitiqueConfig::new(TAILLE_VECTEUR, NB_SORTIES)
        .init(&device)
        .load_record(record);
    let relus = relu
        .forward(entree)
        .into_data()
        .iter::<f32>()
        .collect::<Vec<_>>();

    assert_eq!(attendus.len(), relus.len());
    for (attendu, relu) in attendus.iter().zip(relus.iter()) {
        assert!((attendu - relu).abs() < 1e-5, "{attendu} vs {relu}");
    }
    let _ = std::fs::remove_file(&chemin);
}
