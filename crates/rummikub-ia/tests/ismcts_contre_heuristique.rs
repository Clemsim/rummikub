//! Une partie complète : l'ISMCTS contre le bot heuristique, à graine fixe.
//! Les mains sont raccourcies pour que la recherche reste rapide dans les
//! tests — tout le reste se joue par les règles du core.

use rummikub_core::partie::Partie;
use rummikub_ia::{bot_heuristique, ismcts::IsMcts};

#[test]
fn test_ismcts_contre_bot_heuristique() {
    let mut partie =
        Partie::nouvelle_avec_graine(vec!["ISMCTS".to_string(), "Heuristique".to_string()], 11)
            .unwrap();

    // Mains raccourcies : moins de coups à énumérer, partie plus courte
    partie.joueurs[0].main.truncate(6);
    partie.joueurs[1].main.truncate(6);

    let ismcts = IsMcts {
        simulations: 40,
        profondeur_rollout: 12,
        threads: 1,
        ..Default::default()
    };

    let mut tours = 0;
    while partie.gagnant().is_none() && tours < 100 {
        if partie.index_joueur_courant == 0 {
            ismcts
                .jouer_tour(&mut partie)
                .expect("l'ISMCTS ne doit jamais tenter un coup illégal");
        } else {
            bot_heuristique::jouer_tour(&mut partie).unwrap();
        }
        tours += 1;
    }

    // Gagnée ou plafonnée, la partie se déroule sans coup illégal
    assert!(partie.gagnant().is_some() || tours == 100);
}
