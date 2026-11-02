use crate::tuile::{Color, Tuile, TuileType};
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Meld {
    pub Tuiles: Vec<Tuile>,
}

impl Meld {
    pub fn new(Tuiles: Vec<Tuile>) -> Self {
        Self { Tuiles }
    }

    pub fn is_valid(&self) -> bool {
        if self.Tuiles.len() < 3 {
            return false;
        }
        self.is_valid_group() || self.is_valid_run()
    }

    /// Validation d'un groupe (3 ou 4 tuiles de même valeur et de couleurs différentes)
    pub fn is_valid_group(&self) -> bool {
        let len = self.Tuiles.len();
        if !(3..=4).contains(&len) {
            return false;
        }

        let mut group_val: Option<u8> = None;
        let mut colors_seen = HashSet::new();

        for Tuile in &self.Tuiles {
            match Tuile.Type {
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
        let len = self.Tuiles.len();
        // Une suite ne peut pas dépasser 13 tuiles (1 à 13)
        if !(3..=13).contains(&len) {
            return None;
        }

        let mut jokers_count: usize = 0;
        let mut standard_values = Vec::new();
        let mut run_color: Option<Color> = None;

        for Tuile in &self.Tuiles {
            match Tuile.Type {
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
                .Tuiles
                .iter()
                .find_map(|t| match t.Type {
                    TuileType::Standard { value, .. } => Some(value),
                    _ => None,
                })
                .unwrap_or(0);
            return (standard_val as u32) * (self.Tuiles.len() as u32);
        }

        if let Some((_, max_s)) = self.resolve_run_range() {
            // Choix du début de séquence qui maximise les points (le plus grand S possible)
            let len = self.Tuiles.len() as u32;
            let start = max_s as u32;
            let end = start + len - 1;
            return len * (start + end) / 2;
        }

        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn standard(id: u8, color: Color, value: u8) -> Tuile {
        Tuile::new_standard(id, color, value)
    }

    #[test]
    fn test_valid_groups() {
        // Groupe de 3 : même valeur, couleurs différentes
        let g3 = Meld::new(vec![
            standard(1, Color::Red, 7),
            standard(2, Color::Blue, 7),
            standard(3, Color::Yellow, 7),
        ]);
        assert!(g3.is_valid_group());
        assert!(g3.is_valid());
        assert_eq!(g3.points(), 21);

        // Groupe de 4
        let g4 = Meld::new(vec![
            standard(1, Color::Red, 13),
            standard(2, Color::Blue, 13),
            standard(3, Color::Yellow, 13),
            standard(4, Color::Black, 13),
        ]);
        assert!(g4.is_valid_group());
        assert_eq!(g4.points(), 52);
    }

    #[test]
    fn test_invalid_groups() {
        // Deux fois la même couleur
        let dup_color = Meld::new(vec![
            standard(1, Color::Red, 7),
            standard(2, Color::Red, 7),
            standard(3, Color::Blue, 7),
        ]);
        assert!(!dup_color.is_valid_group());

        // Valeurs différentes
        let diff_values = Meld::new(vec![
            standard(1, Color::Red, 7),
            standard(2, Color::Blue, 8),
            standard(3, Color::Yellow, 7),
        ]);
        assert!(!diff_values.is_valid_group());

        // Trop court
        let two_tiles = Meld::new(vec![
            standard(1, Color::Red, 7),
            standard(2, Color::Blue, 7),
        ]);
        assert!(!two_tiles.is_valid_group());
        assert!(!two_tiles.is_valid());
    }

    #[test]
    fn test_valid_group_with_joker() {
        // Le joker complète le groupe et prend la valeur du groupe
        let meld = Meld::new(vec![
            standard(1, Color::Red, 10),
            Tuile::new_joker(2),
            standard(3, Color::Black, 10),
        ]);
        assert!(meld.is_valid_group());
        assert!(meld.is_valid());
        assert_eq!(meld.points(), 3 * 10);
    }

    #[test]
    fn test_invalid_runs() {
        // Couleurs mélangées
        let mixed_colors = Meld::new(vec![
            standard(1, Color::Red, 4),
            standard(2, Color::Blue, 5),
            standard(3, Color::Red, 6),
        ]);
        assert!(!mixed_colors.is_valid_run());
        assert!(!mixed_colors.is_valid());

        // Doublon de valeur (deux 5 rouges)
        let duplicate = Meld::new(vec![
            standard(1, Color::Red, 5),
            standard(2, Color::Red, 5),
            standard(3, Color::Red, 6),
        ]);
        assert!(!duplicate.is_valid_run());

        // Trop long : une suite ne dépasse pas 13 tuiles
        let too_long: Vec<Tuile> = (0u8..14)
            .map(|i| standard(i, Color::Red, i % 13 + 1))
            .collect();
        assert!(!Meld::new(too_long).is_valid_run());
    }

    #[test]
    fn test_run_with_joker_fills_gap() {
        // [8 Bleu, Joker, 10 Bleu] : le joker prend la place du 9
        let meld = Meld::new(vec![
            standard(1, Color::Blue, 8),
            Tuile::new_joker(2),
            standard(3, Color::Blue, 10),
        ]);
        assert!(meld.is_valid_run());
        assert_eq!(meld.points(), 8 + 9 + 10);
    }

    #[test]
    fn test_run_range_bounds() {
        // [Joker, 2 Rouge, 3 Rouge] : démarrage possible à 1 ou 2
        let meld = Meld::new(vec![
            Tuile::new_joker(9),
            standard(1, Color::Red, 2),
            standard(2, Color::Red, 3),
        ]);
        assert_eq!(meld.resolve_run_range(), Some((1, 2)));
    }

    #[test]
    fn test_run_forced_at_extremes() {
        // [Joker, 1 Rouge, 2 Rouge] : seule la suite 1-2-3 contient 1 et 2
        let low = Meld::new(vec![
            Tuile::new_joker(9),
            standard(1, Color::Red, 1),
            standard(2, Color::Red, 2),
        ]);
        assert_eq!(low.resolve_run_range(), Some((1, 1)));
        assert_eq!(low.points(), 1 + 2 + 3);

        // [11 Noir, 12 Noir, 13 Noir, Joker] : le joker ne peut être que 10
        let high = Meld::new(vec![
            standard(1, Color::Black, 11),
            standard(2, Color::Black, 12),
            standard(3, Color::Black, 13),
            Tuile::new_joker(4),
        ]);
        assert_eq!(high.resolve_run_range(), Some((10, 10)));
        assert_eq!(high.points(), 10 + 11 + 12 + 13);
    }

    #[test]
    fn test_run_points_maximize_start() {
        // [Joker, 10 Rouge, 11 Rouge] : le joker vaut 12 (10, 11, 12) plutôt que 9
        let meld = Meld::new(vec![
            Tuile::new_joker(9),
            standard(1, Color::Red, 10),
            standard(2, Color::Red, 11),
        ]);
        assert!(meld.is_valid_run());
        assert_eq!(meld.points(), 10 + 11 + 12);
    }

    #[test]
    fn test_full_run_thirteen_tiles() {
        let tiles: Vec<Tuile> = (1u8..=13).map(|v| standard(v, Color::Blue, v)).collect();
        let meld = Meld::new(tiles);
        assert!(meld.is_valid());
        assert_eq!(meld.points(), (1..=13u8).sum::<u8>() as u32); // 91
    }

    #[test]
    fn test_meld_neither_group_nor_run_is_invalid() {
        let meld = Meld::new(vec![
            standard(1, Color::Red, 4),
            standard(2, Color::Red, 9),
            standard(3, Color::Blue, 4),
        ]);
        assert!(!meld.is_valid());
        assert_eq!(meld.points(), 0);
    }
}
