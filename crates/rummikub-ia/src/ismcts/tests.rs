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
        graine: 5,
        ..Default::default()
    };
    assert_eq!(
        bot.choisir_coup(&partie).unwrap(),
        bot.choisir_coup(&partie).unwrap()
    );
}
