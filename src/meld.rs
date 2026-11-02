use crate::tuile::{Color, Tuile, TuileType};
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Meld {
    pub tiles: Vec<Tuile>,
}

impl Meld {
    pub fn new(tiles: Vec<Tuile>) -> Self {
        Self { tiles }
    }

    pub fn is_valid(&self) -> bool {
        if self.tiles.len() < 3 {
            return false;
        }
        self.is_valid_group() || self.is_valid_run()
    }

    /// Validation d'un groupe (3 ou 4 tuiles de même valeur et de couleurs différentes)
    pub fn is_valid_group(&self) -> bool {
        let len = self.tiles.len();
        if !(3..=4).contains(&len) {
            return false;
        }

        let mut group_val: Option<u8> = None;
        let mut colors_seen = HashSet::new();

        for tile in &self.tiles {
            match tile.kind {
                TuileType::Standard { color, value } => {
                    // Toutes les tuiles standard doivent avoir la même valeur
                    if let Some(expected_val) = group_val {
                        if value != expected_val {
                            return false;
                        }
                    } else {
                        group_val = Some(value);
                    }

                    // Pas deux fois la même couleur dans un groupe
                    if !colors_seen.insert(color) {
                        return false;
                    }
                }
                TuileType::Joker => {}
            }
        }

        // Il faut au moins une tuile standard pour définir la valeur du groupe
        group_val.is_some()
    }

    /// Validation d'une suite (>= 3 tuiles de même couleur, consécutives, avec Jokers)
    pub fn is_valid_run(&self) -> bool {
        self.resolve_run_range().is_some()
    }

    /// Analyse la suite et retourne (start_value_min, start_value_max) si la suite est valide.
    /// Une suite de longueur L démarrant à `S` couvrira [S ..= S + L - 1].
    fn resolve_run_range(&self) -> Option<(u8, u8)> {
        let len = self.tiles.len();
        // Une suite ne peut pas dépasser 13 tuiles (1 à 13)
        if !(3..=13).contains(&len) {
            return false.then_some((0, 0))?;
        }

        let mut jokers_count: usize = 0;
        let mut standard_values = Vec::new();
        let mut run_color: Option<Color> = None;

        for tile in &self.tiles {
            match tile.kind {
                TuileType::Joker => jokers_count += 1,
                TuileType::Standard { color, value } => {
                    if let Some(expected_color) = run_color {
                        if color != expected_color {
                            return None; // Couleurs différentes
                        }
                    } else {
                        run_color = Some(color);
                    }
                    standard_values.push(value);
                }
            }
        }

        // Cas extrême : que des jokers (impossible car le jeu n'a que 2 jokers et len >= 3)
        if standard_values.is_empty() {
            return None;
        }

        // Trier les valeurs standard
        standard_values.sort_unstable();

        // 1. Vérifier qu'il n'y a pas de doublons
        for pair in standard_values.windows(2) {
            if pair[0] == pair[1] {
                return None; // Doublon détecté (ex: deux 5 rouges)
            }
        }

        // 2. Calculer le nombre de jokers nécessaires pour combler les trous internes
        let mut internal_jokers_needed: usize = 0;
        for pair in standard_values.windows(2) {
            let gap = (pair[1] - pair[0] - 1) as usize;
            internal_jokers_needed += gap;
        }

        if internal_jokers_needed > jokers_count {
            return None; // Pas assez de jokers pour combler les trous
        }

        let remaining_jokers = (jokers_count - internal_jokers_needed) as u8;
        let v_min = *standard_values.first().unwrap();
        let v_max = *standard_values.last().unwrap();
        let total_len = len as u8;

        // La valeur de début S doit satisfaire :
        // 1. S >= 1
        // 2. S + total_len - 1 <= 13 => S <= 14 - total_len
        // 3. S doit être dans l'intervalle permis par les jokers restants :
        //    v_min - remaining_jokers <= S <= v_min
        let min_s = 1.max(v_min.saturating_sub(remaining_jokers));
        let max_s = v_min.min(14_u8.saturating_sub(total_len));

        // De plus, la tuile max ne doit pas dépasser la fin :
        // S + total_len - 1 >= v_max => S >= v_max + 1 - total_len
        let lower_bound_from_vmax = (v_max + 1).saturating_sub(total_len);
        let final_min_s = min_s.max(lower_bound_from_vmax);

        if final_min_s <= max_s {
            Some((final_min_s, max_s))
        } else {
            None
        }
    }

    /// Calcule la valeur en points de la combinaison.
    /// Si un joker est présent dans une suite, on prend la configuration la plus avantageuse.
    pub fn points(&self) -> u32 {
        if self.is_valid_group() {
            // Dans un groupe, chaque joker prend la valeur de la tuile du groupe
            let standard_val = self
                .tiles
                .iter()
                .find_map(|t| match t.kind {
                    TuileType::Standard { value, .. } => Some(value),
                    _ => None,
                })
                .unwrap_or(0);
            return (standard_val as u32) * (self.tiles.len() as u32);
        }

        if let Some((_, max_s)) = self.resolve_run_range() {
            // Choix du début de séquence qui maximise les points (le plus grand S possible)
            let len = self.tiles.len() as u32;
            let start = max_s as u32;
            let end = start + len - 1;
            return len * (start + end) / 2;
        }

        0
    }
}
