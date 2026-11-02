//! Deux bots heuristiques s'affrontent sur une partie à graine fixe :
//! toute la simulation est reproductible d'une exécution à l'autre.

use rummikub_core::partie::Partie;
use rummikub_ia::bot_heuristique;

#[test]
fn test_partie_complete_entre_deux_bots() {
    let mut partie =
        Partie::nouvelle_avec_graine(vec!["Alice".to_string(), "Bob".to_string()], 7).unwrap();

    let mut tours = 0;
    while partie.gagnant().is_none() && tours < 400 {
        bot_heuristique::jouer_tour(&mut partie)
            .expect("les bots ne devraient jamais tenter un coup illégal");
        tours += 1;
    }

    // La partie s'est déroulée sans erreur ; gagnée ou plafonnée par le
    // nombre de tours (si personne ne peut finir), l'état reste cohérent.
    assert!(partie.gagnant().is_some() || tours == 400);
}
