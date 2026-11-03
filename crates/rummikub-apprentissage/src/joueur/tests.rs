use super::*;
use crate::encodeur::{NB_TYPES, TAILLE_VECTEUR, index_type};
use crate::modele::{NB_SORTIES, PolitiqueConfig};
use burn::module::Module;
use rummikub_core::combinaison::Combinaison;
use rummikub_core::tuile::{Couleur, Tuile};

/// Un réseau à poids déterministes (graine fixée) pour les tests
fn reseau_test() -> Politique {
    let device = Device::flex();
    device.seed(42);
    PolitiqueConfig::new(TAILLE_VECTEUR, NB_SORTIES).init(&device)
}

/// Les probabilités de test : tous les types à `base`, un type à `valeur`,
/// le drapeau piocher à `pioche`
fn probabilites_de_test(index: usize, base: f32, valeur: f32, pioche: f32) -> Vec<f32> {
    let mut probabilites = vec![base; NB_SORTIES];
    probabilites[index] = valeur;
    probabilites[NB_TYPES] = pioche;
    probabilites
}

fn poser_un_type(couleur: Couleur, valeur: u8) -> Coup {
    Coup::Poser(vec![Combinaison::nouvelle(vec![Tuile::nouvelle_standard(
        0, couleur, valeur,
    )])])
}

#[test]
fn test_le_score_privilegie_les_types_decides_vrais() {
    // Le rouge-5 est très vraisemblable, le noir-10 presque impossible :
    // la pose du rouge-5 doit score plus haut que celle du noir-10
    let index_rouge5 = index_type(&Tuile::nouvelle_standard(0, Couleur::Rouge, 5));
    let probabilites = probabilites_de_test(index_rouge5, 0.1, 0.95, 0.1);
    let plateau_vide = BitsetTuiles::default();

    let rouge5 = poser_un_type(Couleur::Rouge, 5);
    let noir10 = poser_un_type(Couleur::Noir, 10);

    let score_rouge5 = score_du_coup(&rouge5, &plateau_vide, &probabilites).unwrap();
    let score_noir10 = score_du_coup(&noir10, &plateau_vide, &probabilites).unwrap();
    assert!(score_rouge5 > score_noir10);
}

#[test]
fn test_le_piocher_est_choisi_quand_il_est_vraisemblable() {
    // Tous les types improbables, piocher très probable : entre une pose
    // impossible et la pioche, le réseau doit choisir la pioche
    let probabilites = probabilites_de_test(0, 0.1, 0.1, 0.9);
    let plateau_vide = BitsetTuiles::default();
    let coups = [poser_un_type(Couleur::Rouge, 5), Coup::Piocher];

    let scores: Vec<f64> = coups
        .iter()
        .map(|coup| score_du_coup(coup, &plateau_vide, &probabilites).unwrap())
        .collect();
    assert!(scores[1] > scores[0]);
}

#[test]
fn test_le_score_de_la_pioche_suite_son_drapeau() {
    // Le drapeau piocher pèse dans le score du coup Piocher
    let plateau_vide = BitsetTuiles::default();
    let pioche_basse = probabilites_de_test(0, 0.5, 0.5, 0.2);
    let pioche_haute = probabilites_de_test(0, 0.5, 0.5, 0.8);

    let score_bas = score_du_coup(&Coup::Piocher, &plateau_vide, &pioche_basse).unwrap();
    let score_haut = score_du_coup(&Coup::Piocher, &plateau_vide, &pioche_haute).unwrap();
    assert!(score_haut > score_bas);
}

#[test]
fn test_terminer_n_a_pas_de_score() {
    let probabilites = probabilites_de_test(0, 0.5, 0.5, 0.5);
    let plateau_vide = BitsetTuiles::default();
    assert_eq!(
        score_du_coup(&Coup::Terminer, &plateau_vide, &probabilites),
        None
    );
}

#[test]
fn test_le_choix_est_deterministe_et_legal() {
    let reseau = reseau_test();
    let mut partie = rummikub_core::partie::Partie::nouvelle_avec_graine(
        vec!["A".to_string(), "B".to_string()],
        7,
    )
    .unwrap();
    partie.ouvrir_tour();

    let coup1 = choisir_coup(&reseau, &partie).unwrap().unwrap();
    let coup2 = choisir_coup(&reseau, &partie).unwrap().unwrap();
    assert_eq!(coup1, coup2);

    // Le coup choisi est bien l'un des coups légaux énumérés par les règles
    let coups_legaux = rummikub_core::coups::coups_possibles(&partie).unwrap();
    assert!(coups_legaux.contains(&coup1));
}

#[test]
fn test_choisir_coup_exige_un_tour_ouvert() {
    let reseau = reseau_test();
    let partie = rummikub_core::partie::Partie::nouvelle_avec_graine(
        vec!["A".to_string(), "B".to_string()],
        7,
    )
    .unwrap();
    assert!(choisir_coup(&reseau, &partie).is_err());
}

#[test]
fn test_le_chargement_des_poids_fait_un_aller_retour() {
    let reseau = reseau_test();
    let chemin = std::env::temp_dir().join("rummikub-test-politique.burnpack");
    reseau
        .clone()
        .into_record()
        .save(&chemin)
        .map_err(|erreur| erreur.to_string())
        .unwrap();

    let recharge = charger_politique(chemin.to_str().unwrap()).unwrap();
    let entree = vec![0.5f32; TAILLE_VECTEUR];
    assert_eq!(
        probabilites(&reseau, &entree).unwrap(),
        probabilites(&recharge, &entree).unwrap()
    );

    std::fs::remove_file(&chemin).ok();
}

#[test]
fn test_le_tournoi_contre_lheuristique_est_legal_et_reproductible() {
    let reseau = reseau_test();

    // Deux tournois identiques : mêmes graines → mêmes vainqueurs
    let premier = tournoi(3, &reseau, &Adversaire::Heuristique).unwrap();
    let second = tournoi(3, &reseau, &Adversaire::Heuristique).unwrap();
    assert_eq!(premier.victoires_reseau, second.victoires_reseau);
    assert_eq!(premier.victoires_adversaire, second.victoires_adversaire);
    assert_eq!(premier.non_terminees, second.non_terminees);

    // Toutes les parties aboutissent à un vainqueur connu
    let total = premier.victoires_reseau + premier.victoires_adversaire + premier.non_terminees;
    assert_eq!(total, 3);
}

#[test]
fn test_le_traceur_lit_les_deux_schemas_du_banc() {
    let png = std::env::temp_dir().join("rummikub-test-joue.png");
    let chemin_png = png.to_str().unwrap();

    // Ancien schéma (8 colonnes, sans professeur) : prof. = 20 par défaut
    let csv_ancien = std::env::temp_dir().join("rummikub-test-joue-ancien.csv");
    std::fs::write(
        &csv_ancien,
        "date,adversaire,simulations,graines,victoires_reseau,victoires_adversaire,non_terminees,duree_s\n1,heuristique,0,20,17,3,0,0.1\n",
    )
    .unwrap();
    tracer_joue(csv_ancien.to_str().unwrap(), chemin_png).unwrap();
    let lignes_anciennes = lire_banc(csv_ancien.to_str().unwrap()).unwrap();
    assert_eq!(lignes_anciennes.len(), 1);
    assert_eq!(lignes_anciennes[0].professeur, 20);
    assert!((lignes_anciennes[0].taux_reseau() - 17.0 / 20.0).abs() < 1e-9);

    // Nouveau schéma (9 colonnes)
    let csv_nouveau = std::env::temp_dir().join("rummikub-test-joue-nouveau.csv");
    std::fs::write(
        &csv_nouveau,
        "date,adversaire,simulations,graines,professeur,victoires_reseau,victoires_adversaire,non_terminees,duree_s\n2,ismcts,300,100,100,35,65,0,1.0\n",
    )
    .unwrap();
    tracer_joue(csv_nouveau.to_str().unwrap(), chemin_png).unwrap();
    let lignes_nouvelles = lire_banc(csv_nouveau.to_str().unwrap()).unwrap();
    assert_eq!(lignes_nouvelles.len(), 1);
    assert_eq!(lignes_nouvelles[0].professeur, 100);
    assert!((lignes_nouvelles[0].taux_reseau() - 35.0 / 100.0).abs() < 1e-9);

    assert!(std::path::Path::new(chemin_png).exists());

    std::fs::remove_file(&csv_ancien).ok();
    std::fs::remove_file(&csv_nouveau).ok();
    std::fs::remove_file(&png).ok();
}

#[test]
fn test_ladversaire_heuristique_joue_son_tour() {
    let mut partie = rummikub_core::partie::Partie::nouvelle_avec_graine(
        vec!["A".to_string(), "B".to_string()],
        7,
    )
    .unwrap();
    partie.joueurs[1].main.truncate(6);
    partie.index_joueur_courant = 1;
    partie.ouvrir_tour();
    Adversaire::Heuristique.jouer_tour(&mut partie).unwrap();
    // Le tour s'est refermé : plus de Tour::Ouvert
    assert!(!matches!(
        partie.tour,
        rummikub_core::partie::Tour::Ouvert { .. }
    ));
    assert_eq!(partie.index_joueur_courant, 0);
}
