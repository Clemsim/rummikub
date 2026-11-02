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

/// Un ensemble d'identifiants de tuiles, stocké en bitmap : tout identifiant
/// u8 tient dans 4 × u64, soit 32 octets sur la pile — aucun hachage, aucune
/// allocation (voir ADR 0005).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct BitsetTuiles {
    mots: [u64; 4],
}

impl BitsetTuiles {
    /// Insère l'identifiant
    pub fn inserer(&mut self, id: u8) {
        self.mots[id as usize / 64] |= 1 << (id as usize % 64);
    }

    /// Insère l'identifiant et retourne false s'il était déjà présent
    pub fn inserer_si_absent(&mut self, id: u8) -> bool {
        if self.contient(id) {
            return false;
        }
        self.inserer(id);
        true
    }

    /// Retire l'identifiant
    pub fn retirer(&mut self, id: u8) {
        self.mots[id as usize / 64] &= !(1 << (id as usize % 64));
    }

    /// Retire l'identifiant et retourne true s'il était présent
    pub fn retirer_si_present(&mut self, id: u8) -> bool {
        if !self.contient(id) {
            return false;
        }
        self.retirer(id);
        true
    }

    /// L'identifiant est-il présent ?
    pub fn contient(&self, id: u8) -> bool {
        self.mots[id as usize / 64] & (1 << (id as usize % 64)) != 0
    }

    /// L'ensemble est-il vide ?
    pub fn est_vide(&self) -> bool {
        self.mots == [0; 4]
    }

    /// Aucun identifiant en commun avec l'autre ensemble ?
    pub fn disjoint(&self, autre: &Self) -> bool {
        self.mots
            .iter()
            .zip(autre.mots)
            .all(|(mot, mot_autre)| mot & mot_autre == 0)
    }

    /// Insère tous les identifiants de l'autre ensemble
    pub fn inserer_tout(&mut self, autre: &Self) {
        for (mot, mot_autre) in self.mots.iter_mut().zip(autre.mots) {
            *mot |= mot_autre;
        }
    }

    /// Itère sur les identifiants présents, dans l'ordre croissant
    pub fn ids(&self) -> impl Iterator<Item = u8> + '_ {
        (0..=u8::MAX).filter(|id| self.contient(*id))
    }

    /// Construit l'ensemble depuis une suite d'identifiants
    pub fn depuis_ids(ids: impl IntoIterator<Item = u8>) -> Self {
        ids.into_iter().collect()
    }
}

impl FromIterator<u8> for BitsetTuiles {
    fn from_iter<I: IntoIterator<Item = u8>>(ids: I) -> Self {
        let mut ensemble = Self::default();
        for id in ids {
            ensemble.inserer(id);
        }
        ensemble
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
