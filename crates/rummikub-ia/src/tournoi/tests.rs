use super::*;
use std::time::Duration;

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

#[test]
fn test_les_resultats_sont_enregistres_en_csv() {
    let chemin = std::env::temp_dir().join("banc-test-rummikub.csv");
    let _ = std::fs::remove_file(&chemin);

    let resultat = Resultat {
        victoires_ismcts: 2,
        victoires_heuristique: 1,
        non_terminees: 0,
        duree: Duration::from_secs(3),
    };
    enregistrer_resultat(chemin.to_string_lossy().as_ref(), 0.2, 3, 50, &resultat).unwrap();
    enregistrer_resultat(chemin.to_string_lossy().as_ref(), 0.4, 3, 50, &resultat).unwrap();

    let contenu = std::fs::read_to_string(&chemin).unwrap();
    let lignes: Vec<&str> = contenu.lines().collect();
    assert_eq!(lignes.len(), 3); // en-tête + 2 lignes
    assert!(lignes[0].starts_with("date,epsilon"));
    assert!(lignes[1].contains("0.2"));
    assert!(lignes[1].contains(",2,1,0,"));
    assert!(lignes[2].contains("0.4"));
    let _ = std::fs::remove_file(&chemin);
}

#[test]
fn test_le_balayage_d_epsilon_renvoie_tous_les_points() {
    let points = balayage_epsilon(&[1.0, 0.0], 1, 10).unwrap();
    assert_eq!(points.len(), 2);
    assert_eq!(points[0].0, 1.0);
    assert_eq!(points[1].0, 0.0);
    assert!((0.0..=1.0).contains(&points[0].1));
}
