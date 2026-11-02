use crate::tuile::{Couleur, Tuile, TuileType};
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Combinaison {
    pub Tuiles: Vec<Tuile>,
}

impl Combinaison {
    pub fn nouvelle(Tuiles: Vec<Tuile>) -> Self {
        Self { Tuiles }
    }

    pub fn est_valide(&self) -> bool {
        if self.Tuiles.len() < 3 {
            return false;
        }
        self.est_groupe_valide() || self.est_suite_valide()
    }

    /// Validation d'un groupe (3 ou 4 tuiles de même valeur et de couleurs différentes)
    pub fn est_groupe_valide(&self) -> bool {
        let longueur = self.Tuiles.len();
        if !(3..=4).contains(&longueur) {
            return false;
        }

        let mut valeur_groupe: Option<u8> = None;
        let mut couleurs_vues = HashSet::new();

        for Tuile in &self.Tuiles {
            match Tuile.Type {
                TuileType::Standard { couleur, valeur } => {
                    // Toutes les tuiles standard doivent avoir la même valeur
                    if let Some(valeur_attendue) = valeur_groupe {
                        if valeur != valeur_attendue {
                            return false;
                        }
                    } else {
                        valeur_groupe = Some(valeur);
                    }

                    // Pas deux fois la même couleur dans un groupe
                    if !couleurs_vues.insert(couleur) {
                        return false;
                    }
                }
                TuileType::Joker => {}
            }
        }

        // Il faut au moins une tuile standard pour définir la valeur du groupe
        valeur_groupe.is_some()
    }

    /// Validation d'une suite (>= 3 tuiles de même couleur, consécutives, avec Jokers)
    pub fn est_suite_valide(&self) -> bool {
        self.bornes_de_suite().is_some()
    }

    /// Analyse la suite et retourne (start_value_min, start_value_max) si la suite est valide.
    /// Une suite de longueur L démarrant à `S` couvrira [S ..= S + L - 1].
    fn bornes_de_suite(&self) -> Option<(u8, u8)> {
        let longueur = self.Tuiles.len();
        // Une suite ne peut pas dépasser 13 tuiles (1 à 13)
        if !(3..=13).contains(&longueur) {
            return None;
        }

        let (jokers, mut valeurs) = self.jokers_et_valeurs_de_suite()?;
        valeurs.sort_unstable();

        if !Self::valeurs_sans_doublon(&valeurs) {
            return None; // Doublon détecté (ex : deux 5 rouges)
        }

        // Jokers restants une fois les trous internes comblés
        // (None s'il n'y en a pas assez)
        let jokers_restants = Self::jokers_restants(&valeurs, jokers)? as u8;
        let v_min = *valeurs.first().unwrap();
        let v_max = *valeurs.last().unwrap();
        let longueur_u8 = longueur as u8;

        // La valeur de début S doit satisfaire :
        // 1. S >= 1
        // 2. S <= v_min (la plus petite valeur standard doit être couverte)
        // 3. S + longueur - 1 <= 13 => S <= 14 - longueur (fin de suite dans les bornes)
        // 4. S + longueur - 1 >= v_max => S >= v_max + 1 - longueur
        //    (la plus grande valeur standard doit être couverte)
        let debut_min = 1u8
            .max(v_min.saturating_sub(jokers_restants))
            .max((v_max + 1).saturating_sub(longueur_u8));
        let debut_max = v_min.min(14u8.saturating_sub(longueur_u8));

        if debut_min <= debut_max {
            Some((debut_min, debut_max))
        } else {
            None
        }
    }

    /// Valeurs des tuiles standard d'une suite, accompagnées du nombre de jokers.
    /// Retourne None si les couleurs ne concordent pas toutes, ou s'il n'y a que
    /// des jokers (impossible : le jeu n'en contient que 2).
    fn jokers_et_valeurs_de_suite(&self) -> Option<(usize, Vec<u8>)> {
        let mut jokers: usize = 0;
        let mut valeurs: Vec<u8> = Vec::new();
        let mut couleur_suite: Option<Couleur> = None;

        for tuile in &self.Tuiles {
            match tuile.Type {
                TuileType::Joker => jokers += 1,
                TuileType::Standard { couleur, valeur } => {
                    if let Some(couleur_attendue) = couleur_suite {
                        if couleur != couleur_attendue {
                            return None; // Couleurs différentes
                        }
                    } else {
                        couleur_suite = Some(couleur);
                    }
                    valeurs.push(valeur);
                }
            }
        }

        if valeurs.is_empty() {
            return None;
        }

        Some((jokers, valeurs))
    }

    /// Les valeurs triées d'une suite ne doivent pas contenir de doublon
    fn valeurs_sans_doublon(valeurs_triees: &[u8]) -> bool {
        valeurs_triees.windows(2).all(|paire| paire[0] != paire[1])
    }

    /// Jokers restants une fois les trous entre valeurs standard comblés.
    /// Retourne None s'il n'y a pas assez de jokers pour les combler.
    fn jokers_restants(valeurs_triees: &[u8], jokers: usize) -> Option<usize> {
        let trous: usize = valeurs_triees
            .windows(2)
            .map(|paire| (paire[1] - paire[0] - 1) as usize)
            .sum();
        jokers.checked_sub(trous)
    }

    /// Calcule la valeur en points de la combinaison.
    /// Si un joker est présent dans une suite, on prend la configuration la plus avantageuse.
    pub fn points(&self) -> u32 {
        if self.est_groupe_valide() {
            // Dans un groupe, chaque joker prend la valeur de la tuile du groupe
            let valeur_standard = self
                .Tuiles
                .iter()
                .find_map(|t| match t.Type {
                    TuileType::Standard { valeur, .. } => Some(valeur),
                    _ => None,
                })
                .unwrap_or(0);
            return (valeur_standard as u32) * (self.Tuiles.len() as u32);
        }

        if let Some((_, debut_max)) = self.bornes_de_suite() {
            // Choix du début de séquence qui maximise les points (le plus grand S possible)
            let longueur = self.Tuiles.len() as u32;
            let debut = debut_max as u32;
            let fin = debut + longueur - 1;
            return longueur * (debut + fin) / 2;
        }

        0
    }
}

#[cfg(test)]
mod tests;
