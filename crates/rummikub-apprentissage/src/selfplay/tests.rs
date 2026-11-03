use super::*;
use crate::encodeur::{NB_TYPES, TAILLE_VECTEUR};
use crate::modele::{NB_SORTIES, PolitiqueConfig};

/// Un réseau à poids déterministes (graine fixée) pour les tests
fn reseau_test() -> Politique {
    let device = Device::flex();
    device.seed(42);
    PolitiqueConfig::new(TAILLE_VECTEUR, NB_SORTIES).init(&device)
}

#[test]
fn test_les_auto_parties_sont_deterministes_et_legales() {
    let reseau = reseau_test();
    let tau = 5.0;

    let mut rng1 = StdRng::seed_from_u64(7);
    let partie1 = jouer_partie(&reseau, 1, tau, &mut rng1).unwrap();
    let mut rng2 = StdRng::seed_from_u64(7);
    let partie2 = jouer_partie(&reseau, 1, tau, &mut rng2).unwrap();
    assert_eq!(format!("{:?}", partie1), format!("{:?}", partie2));

    // Une partie joue : il y a des décisions, et une seule se termine par
    // victoire ou blocage
    assert!(!partie1.decisions.is_empty());
}

#[test]
fn test_les_cibles_sont_des_etiquettes_valides() {
    let reseau = reseau_test();
    let mut rng = StdRng::seed_from_u64(7);
    let partie = jouer_partie(&reseau, 1, 5.0, &mut rng).unwrap();

    for decision in &partie.decisions {
        assert_eq!(decision.cible.len(), NB_SORTIES);
        assert!(decision.cible.iter().all(|&v| v == 0.0 || v == 1.0));
        let nb_vrais = decision.cible.iter().filter(|&&v| v == 1.0).count();
        let piocher = decision.cible[NB_TYPES];
        if piocher == 1.0 {
            // Une pioche ne joue aucun type
            assert_eq!(nb_vrais, 1);
        } else {
            // Un coup non-pioche joue au moins un type
            assert!(nb_vrais >= 1);
        }
        // La récompense suit le score officiel : gagnant ≥ 1, perdant < 0,
        // 0 si bloquée
        if decision.recompense == 0.0 {
            continue;
        }
        assert!(decision.recompense >= 1.0 || decision.recompense < 0.0);
    }
}

#[test]
fn test_la_perte_reinforce_vaut_la_somme_des_bce_pondeeree() {
    let device = Device::flex().autodiff();
    // logits [2, -1], cibles [1, 0], avantage 1 :
    // bce = softplus(-2) + softplus(-1) = 0.126928 + 0.313262 = 0.44019
    let logits = Tensor::<1>::from_floats([2.0f32, -1.0], &device).reshape([1, 2]);
    let cibles = Tensor::<1>::from_floats([1.0f32, 0.0], &device).reshape([1, 2]);
    let avantages = Tensor::<1>::from_floats([1.0f32], &device).reshape([1, 1]);
    let perte = perte_reinforce(logits, cibles, avantages).into_scalar::<f32>();
    assert!((perte - 0.44019).abs() < 1e-4);

    // L'avantage change le signe du gradient mais pas celui de la perte si
    // l'avantage est négatif : perte(−1) = −0.44019
    let logits = Tensor::<1>::from_floats([2.0f32, -1.0], &device).reshape([1, 2]);
    let cibles = Tensor::<1>::from_floats([1.0f32, 0.0], &device).reshape([1, 2]);
    let avantages = Tensor::<1>::from_floats([-1.0f32], &device).reshape([1, 1]);
    let perte = perte_reinforce(logits, cibles, avantages).into_scalar::<f32>();
    assert!((perte + 0.44019).abs() < 1e-4);
}

#[test]
fn test_lentrainement_par_renforcement_tourne_sur_petit() {
    let config = ConfigRenforcement {
        cycles: 2,
        parties_par_cycle: 2,
        tau: 5.0,
        lr: 1e-5,
        taille_lot: 64,
        graine: 42,
        evaluations_tous: 0, // aucune évaluation : le réseau final est retenu
        simulations_ismcts: 300,
        graines_eval: 20,
        decroissance_lr: 1.0,
    };
    let resultat = entrainer_par_renforcement(&reseau_test(), &config).unwrap();

    // Deux cycles enregistrés, sans évaluation (taux NaN)
    assert_eq!(resultat.historique.len(), 2);
    assert_eq!(resultat.meilleur_cycle, 0);
    assert!(resultat.meilleur_taux.is_nan());

    // Les pertes sont finies (le gradient n'a pas explosé)
    for (_, perte, _) in &resultat.historique {
        assert!(perte.is_finite());
    }
}
