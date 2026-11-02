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
mod tests;
