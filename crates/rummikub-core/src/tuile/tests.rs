use super::*;
use std::collections::{HashMap, HashSet};

#[test]
fn test_generate_pool_contains_106_unique_tiles() {
    let pioche = generer_pioche();
    assert_eq!(pioche.len(), 106);

    // Identifiants uniques de 0 à 105
    let ids: HashSet<u8> = pioche.iter().map(|t| t.id).collect();
    assert_eq!(ids.len(), 106);
    assert_eq!(*ids.iter().max().unwrap(), 105);

    // Exactement 2 jokers
    assert_eq!(pioche.iter().filter(|t| t.est_joker()).count(), 2);

    // Chaque couple (couleur, valeur) présent exactement 2 fois
    let mut counts: HashMap<(Couleur, u8), usize> = HashMap::new();
    for tuile in &pioche {
        if let TuileType::Standard { couleur, valeur } = tuile.Type {
            *counts.entry((couleur, valeur)).or_insert(0) += 1;
        }
    }
    assert_eq!(counts.len(), 4 * 13);
    assert!(counts.values().all(|&count| count == 2));
}

#[test]
#[should_panic(expected = "La valeur d'une tuile doit être entre 1 et 13")]
fn test_new_standard_rejects_value_zero() {
    Tuile::nouvelle_standard(0, Couleur::Rouge, 0);
}

#[test]
#[should_panic(expected = "La valeur d'une tuile doit être entre 1 et 13")]
fn test_new_standard_rejects_value_above_13() {
    Tuile::nouvelle_standard(0, Couleur::Rouge, 14);
}

#[test]
fn test_shuffle_pool_keeps_all_tiles() {
    let mut pioche = generer_pioche();
    let mut before: Vec<u8> = pioche.iter().map(|t| t.id).collect();
    before.sort_unstable();

    melanger_pioche(&mut pioche);

    let mut after: Vec<u8> = pioche.iter().map(|t| t.id).collect();
    after.sort_unstable();
    assert_eq!(before, after);
}
