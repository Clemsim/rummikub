use super::*;

#[test]
fn test_le_tournoi_compte_toutes_les_parties() {
    let ismcts = IsMcts {
        simulations: 20,
        profondeur_rollout: 10,
        threads: 1,
        ..Default::default()
    };

    let resultat = tournoi(3, &ismcts).unwrap();

    let jouees =
        resultat.victoires_ismcts + resultat.victoires_heuristique + resultat.non_terminees;
    assert_eq!(jouees, 3);
}
