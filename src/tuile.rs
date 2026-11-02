use rand::seq::SliceRandom;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Color {
    Red,
    Blue,
    Yellow,
    Black,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TuileType {
    Standard { color: Color, value: u8 }, // value: 1..=13
    Joker,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Tuile {
    pub id: u8, // Identifiant unique (0..106)
    pub Type: TuileType,
}

impl Tuile {
    pub fn new_standard(id: u8, color: Color, value: u8) -> Self {
        assert!(
            (1..=13).contains(&value),
            "La valeur d'une tuile doit être entre 1 et 13"
        );
        Self {
            id,
            Type: TuileType::Standard { color, value },
        }
    }

    pub fn new_joker(id: u8) -> Self {
        Self {
            id,
            Type: TuileType::Joker,
        }
    }

    pub fn is_joker(&self) -> bool {
        matches!(self.Type, TuileType::Joker)
    }
}

/// Génère les 106 tuiles du jeu (4 couleurs * 13 valeurs * 2 exemplaires + 2 jokers)
pub fn generate_pool() -> Vec<Tuile> {
    let mut pool = Vec::with_capacity(106);
    let mut current_id: u8 = 0;

    let colors = [Color::Red, Color::Blue, Color::Yellow, Color::Black];

    // 2 exemplaires de chaque tuile
    for _ in 0..2 {
        for &color in &colors {
            for value in 1..=13 {
                pool.push(Tuile::new_standard(current_id, color, value));
                current_id += 1;
            }
        }
    }

    // 2 Jokers
    pool.push(Tuile::new_joker(current_id));
    current_id += 1;
    pool.push(Tuile::new_joker(current_id));

    pool
}

/// Mélange le sac de tuiles aléatoirement
pub fn shuffle_pool(pool: &mut [Tuile]) {
    let mut rng = rand::rng();
    pool.shuffle(&mut rng);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{HashMap, HashSet};

    #[test]
    fn test_generate_pool_contains_106_unique_tiles() {
        let pool = generate_pool();
        assert_eq!(pool.len(), 106);

        // Identifiants uniques de 0 à 105
        let ids: HashSet<u8> = pool.iter().map(|t| t.id).collect();
        assert_eq!(ids.len(), 106);
        assert_eq!(*ids.iter().max().unwrap(), 105);

        // Exactement 2 jokers
        assert_eq!(pool.iter().filter(|t| t.is_joker()).count(), 2);

        // Chaque couple (couleur, valeur) présent exactement 2 fois
        let mut counts: HashMap<(Color, u8), usize> = HashMap::new();
        for tuile in &pool {
            if let TuileType::Standard { color, value } = tuile.Type {
                *counts.entry((color, value)).or_insert(0) += 1;
            }
        }
        assert_eq!(counts.len(), 4 * 13);
        assert!(counts.values().all(|&count| count == 2));
    }

    #[test]
    #[should_panic(expected = "La valeur d'une tuile doit être entre 1 et 13")]
    fn test_new_standard_rejects_value_zero() {
        Tuile::new_standard(0, Color::Red, 0);
    }

    #[test]
    #[should_panic(expected = "La valeur d'une tuile doit être entre 1 et 13")]
    fn test_new_standard_rejects_value_above_13() {
        Tuile::new_standard(0, Color::Red, 14);
    }

    #[test]
    fn test_shuffle_pool_keeps_all_tiles() {
        let mut pool = generate_pool();
        let mut before: Vec<u8> = pool.iter().map(|t| t.id).collect();
        before.sort_unstable();

        shuffle_pool(&mut pool);

        let mut after: Vec<u8> = pool.iter().map(|t| t.id).collect();
        after.sort_unstable();
        assert_eq!(before, after);
    }
}
