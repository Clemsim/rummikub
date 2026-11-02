use super::*;
use rummikub_core::tuile::{Couleur, Tuile};

fn standard(id: u8, couleur: Couleur, valeur: u8) -> Tuile {
    Tuile::nouvelle_standard(id, couleur, valeur)
}

#[test]
fn test_la_determinisation_est_coherente_et_reproductible() {
    let partie = Partie::nouvelle_avec_graine(vec!["A".to_string(), "B".to_string()], 3).unwrap();
    let vue = determiniser(&partie, &mut StdRng::seed_from_u64(42));

    // La main du joueur qui regarde est intacte, les tailles sont préservées
    assert_eq!(vue.joueurs[0].main, partie.joueurs[0].main);
    assert_eq!(vue.joueurs[1].main.len(), partie.joueurs[1].main.len());

    // Aucune tuile n'apparaît deux fois dans la partie devinée
    let mut ids: Vec<u8> = vue
        .joueurs
        .iter()
        .flat_map(|joueur| joueur.main.iter().map(|t| t.id))
        .collect();
    ids.extend(vue.pioche.iter().map(|t| t.id));
    ids.extend(vue.plateau.ids_tuiles());
    assert_eq!(ids.len(), 106);

    // Même graine → même devinette
    let autre = determiniser(&partie, &mut StdRng::seed_from_u64(42));
    assert_eq!(vue.joueurs[1].main, autre.joueurs[1].main);
    assert_eq!(vue.pioche, autre.pioche);
}

#[test]
fn test_ismcts_pose_pour_gagner() {
    let mut partie =
        Partie::nouvelle_avec_graine(vec!["A".to_string(), "B".to_string()], 1).unwrap();
    // La main forme exactement un groupe de 10 (30 points) : le poser vide
    // la main et gagne immédiatement, alors que piocher retarde tout
    partie.joueurs[0].main = vec![
        standard(1, Couleur::Rouge, 10),
        standard(2, Couleur::Bleu, 10),
        standard(3, Couleur::Jaune, 10),
    ];

    partie.ouvrir_tour();
    let bot = IsMcts {
        simulations: 300,
        profondeur_rollout: 30,
        threads: 1,
        ..Default::default()
    };
    let coup = bot
        .choisir_coup(&partie)
        .unwrap()
        .expect("des coups existent");

    let Coup::Poser(combinaisons) = &coup else {
        panic!("l'ISMCTS devrait poser le groupe gagnant, pas {coup:?}");
    };
    assert_eq!(combinaisons[0].points(), 30);
}

#[test]
fn test_ismcts_est_reproductible() {
    let mut partie =
        Partie::nouvelle_avec_graine(vec!["A".to_string(), "B".to_string()], 1).unwrap();
    partie.joueurs[0].main = vec![
        standard(1, Couleur::Rouge, 10),
        standard(2, Couleur::Bleu, 10),
        standard(3, Couleur::Jaune, 10),
    ];
    partie.ouvrir_tour();

    let bot = IsMcts {
        simulations: 100,
        threads: 1,
        graine: 5,
        ..Default::default()
    };
    assert_eq!(
        bot.choisir_coup(&partie).unwrap(),
        bot.choisir_coup(&partie).unwrap()
    );
}

#[test]
fn test_les_recompenses_suivent_le_score_officiel() {
    let mut partie =
        Partie::nouvelle_avec_graine(vec!["A".to_string(), "B".to_string()], 1).unwrap();
    // Le joueur 0 a la main vide (gagnant), le joueur 1 un 7 et un joker :
    // 7 + 30 = 37 points de pénalité
    partie.joueurs[0].main.clear();
    partie.joueurs[1].main = vec![standard(1, Couleur::Rouge, 7), Tuile::nouveau_joker(99)];

    let recompenses = recompenses_finales(&partie);
    assert!((recompenses[0] - 1.37).abs() < 1e-9);
    assert!((recompenses[1] + 0.37).abs() < 1e-9);

    // Sans gagnant (profondeur max atteinte) : 0 pour tous
    partie.joueurs[0].main = vec![standard(2, Couleur::Bleu, 3)];
    assert_eq!(recompenses_finales(&partie), vec![0.0, 0.0]);
}
