use rand::SeedableRng;
use rand::rngs::StdRng;
use rand::seq::SliceRandom;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Couleur {
    Rouge,
    Bleu,
    Jaune,
    Noir,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TuileType {
    Standard { couleur: Couleur, valeur: u8 }, // valeur: 1..=13
    Joker,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Tuile {
    pub id: u8, // Identifiant unique (0..106)
    pub Type: TuileType,
}

impl Tuile {
    pub fn nouvelle_standard(id: u8, couleur: Couleur, valeur: u8) -> Self {
        assert!(
            (1..=13).contains(&valeur),
            "La valeur d'une tuile doit être entre 1 et 13"
        );
        Self {
            id,
            Type: TuileType::Standard { couleur, valeur },
        }
    }

    pub fn nouveau_joker(id: u8) -> Self {
        Self {
            id,
            Type: TuileType::Joker,
        }
    }

    pub fn est_joker(&self) -> bool {
        matches!(self.Type, TuileType::Joker)
    }

    /// Valeur de la tuile pour le décompte final : sa valeur, ou 30 pour un
    /// joker resté en main (règle officielle)
    pub fn valeur_points(&self) -> u8 {
        match self.Type {
            TuileType::Standard { valeur, .. } => valeur,
            TuileType::Joker => 30,
        }
    }
}

/// Génère les 106 tuiles du jeu (4 couleurs * 13 valeurs * 2 exemplaires + 2 jokers)
pub fn generer_pioche() -> Vec<Tuile> {
    let mut pioche = Vec::with_capacity(106);
    let mut current_id: u8 = 0;

    let colors = [Couleur::Rouge, Couleur::Bleu, Couleur::Jaune, Couleur::Noir];

    // 2 exemplaires de chaque tuile
    for _ in 0..2 {
        for &couleur in &colors {
            for valeur in 1..=13 {
                pioche.push(Tuile::nouvelle_standard(current_id, couleur, valeur));
                current_id += 1;
            }
        }
    }

    // 2 Jokers
    pioche.push(Tuile::nouveau_joker(current_id));
    current_id += 1;
    pioche.push(Tuile::nouveau_joker(current_id));

    pioche
}

/// Mélange le sac de tuiles aléatoirement
pub fn melanger_pioche(pioche: &mut [Tuile]) {
    let mut rng = rand::rng();
    pioche.shuffle(&mut rng);
}

/// Mélange la pioche de façon déterministe : la même graine produit toujours
/// le même ordre de tirage (rejeu à l'identique pour l'IA, les tests, le réseau)
pub fn melanger_pioche_avec_graine(pioche: &mut [Tuile], graine: u64) {
    let mut rng = StdRng::seed_from_u64(graine);
    pioche.shuffle(&mut rng);
}

#[cfg(test)]
mod tests;
