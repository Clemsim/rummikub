//! Les coups du jeu : ce que le joueur courant peut tenter, et comment le
//! jouer. C'est le socle des IA (bots, MCTS, RL) : elles énumèrent les coups
//! possibles ici, décident, puis appliquent leur choix avec jouer_coup.
//!
//! Deux propriétés sur lesquelles les IA s'appuient :
//! - l'énumération est déterministe (parcours via BTreeMap) : la même main
//!   produit toujours la même liste, dans le même ordre ;
//! - les doublons exacts (deux exemplaires de la même tuile) sont
//!   interchangeables : une seule variante est générée par (couleur, valeur).

use crate::combinaison::Combinaison;
use crate::partie::{Partie, Tour};
use crate::tuile::{Couleur, Tuile, TuileType};
use std::collections::BTreeMap;

/// Un coup que le joueur courant peut tenter, tour ouvert.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Coup {
    /// Poser une ou plusieurs combinaisons construites depuis sa main
    Poser(Vec<Combinaison>),
    /// Piocher une tuile (termine le tour ; passe si la pioche est vide)
    Piocher,
    // La manipulation complète du plateau (soumettre_plateau) sera énumérée
    // plus tard : c'est l'espace de coups le plus coûteux à générer.
}

/// Énumère toutes les combinaisons valides constructibles depuis `main`
/// (groupes et suites, jokers compris).
pub fn combinaisons_possibles(main: &[Tuile]) -> Vec<Combinaison> {
    let mut possibles = groupes_possibles(main);
    possibles.extend(suites_possibles(main));
    possibles
}

/// Trie les tuiles standard de la main par valeur puis couleur, en retenant
/// un exemplaire canonique par couple (couleur, valeur), et les jokers à part.
fn classer(main: &[Tuile]) -> (BTreeMap<u8, BTreeMap<Couleur, Tuile>>, Vec<Tuile>) {
    let mut standard: BTreeMap<u8, BTreeMap<Couleur, Tuile>> = BTreeMap::new();
    let mut jokers: Vec<Tuile> = Vec::new();

    for tuile in main {
        match tuile.Type {
            TuileType::Joker => jokers.push(*tuile),
            TuileType::Standard { couleur, valeur } => {
                standard
                    .entry(valeur)
                    .or_default()
                    .entry(couleur)
                    .or_insert(*tuile); // premier exemplaire vu = canonique
            }
        }
    }

    (standard, jokers)
}

/// Tous les groupes (3 ou 4 tuiles de même valeur et de couleurs distinctes)
/// constructibles depuis la main, jokers compris.
fn groupes_possibles(main: &[Tuile]) -> Vec<Combinaison> {
    let (standard, jokers) = classer(main);
    let nb_jokers = jokers.len();
    let mut groupes = Vec::new();

    for exemplaires in standard.values() {
        // Un exemplaire canonique par couleur disponible, ordre déterministe
        let disponibles: Vec<Tuile> = exemplaires.values().copied().collect();
        let nb_couleurs = disponibles.len();

        // Chaque sous-ensemble non vide de couleurs disponibles définit la
        // partie standard d'un groupe ; le reste (jusqu'à 3 ou 4 tuiles au
        // total) est comblé par des jokers.
        for masque in 1..(1u32 << nb_couleurs) {
            let choisies: Vec<Tuile> = (0..nb_couleurs)
                .filter(|index| masque & (1 << index) != 0)
                .map(|index| disponibles[index])
                .collect();

            for taille in 3..=4usize {
                if taille < choisies.len() {
                    continue;
                }
                let jokers_utilises = taille - choisies.len();
                if jokers_utilises <= nb_jokers {
                    let mut tuiles = choisies.clone();
                    tuiles.extend(jokers.iter().take(jokers_utilises).copied());
                    groupes.push(Combinaison::nouvelle(tuiles));
                }
            }
        }
    }

    groupes
}

/// Toutes les suites (3 à 13 tuiles de même couleur et de valeurs
/// consécutives) constructibles depuis la main, les jokers comblant les
/// valeurs manquantes.
fn suites_possibles(main: &[Tuile]) -> Vec<Combinaison> {
    let (standard, jokers) = classer(main);
    let nb_jokers = jokers.len();

    // Transposition : par couleur, les valeurs disponibles (canoniques)
    let mut par_couleur: BTreeMap<Couleur, BTreeMap<u8, Tuile>> = BTreeMap::new();
    for (valeur, exemplaires) in &standard {
        for (couleur, tuile) in exemplaires {
            par_couleur
                .entry(*couleur)
                .or_default()
                .insert(*valeur, *tuile);
        }
    }

    let mut suites = Vec::new();

    for valeurs in par_couleur.values() {
        // Déjà trié : les valeurs disponibles de cette couleur
        let disponibles: Vec<u8> = valeurs.keys().copied().collect();

        // Toutes les fenêtres consécutives [debut ..= fin] dans 1..=13 :
        // une fenêtre est jouable si les valeurs absentes peuvent être
        // couvertes par des jokers.
        for longueur in 3..=13usize {
            for debut in 1u8..=(14 - longueur as u8) {
                let fin = debut + longueur as u8 - 1;
                let presentes: Vec<Tuile> = disponibles
                    .iter()
                    .filter(|valeur| **valeur >= debut && **valeur <= fin)
                    .map(|valeur| valeurs[valeur])
                    .collect();
                let manquantes = longueur - presentes.len();

                if manquantes <= nb_jokers {
                    let mut tuiles = presentes;
                    tuiles.extend(jokers.iter().take(manquantes).copied());
                    suites.push(Combinaison::nouvelle(tuiles));
                }
            }
        }
    }

    suites
}

/// Énumère les coups que le joueur courant peut tenter ; le tour doit être
/// ouvert (ouvrir_tour). Les coups Poser sont triés par points décroissants,
/// et Piocher vient en dernier.
///
/// Simplification assumée de cette première version : chaque pose est un coup
/// à une seule combinaison. Les poses combinant plusieurs combinaisons (pour
/// atteindre les 30 points de la pose initiale) et les manipulations
/// complètes du plateau seront énumérées plus tard.
pub fn coups_possibles(partie: &Partie) -> Result<Vec<Coup>, String> {
    if !matches!(partie.tour, Tour::Ouvert { .. }) {
        return Err("Aucun tour en cours : appelez ouvrir_tour pour ouvrir un tour.".to_string());
    }

    let joueur = partie.joueur_courant();
    let mut coups: Vec<Coup> = combinaisons_possibles(&joueur.main)
        .into_iter()
        .map(|combinaison| Coup::Poser(vec![combinaison]))
        .collect();

    // Avant la pose initiale, seules les poses valant au moins 30 points
    // d'un seul coup sont retenues
    if !joueur.a_fait_pose_initiale {
        coups.retain(|coup| match coup {
            Coup::Poser(combinaisons) => combinaisons.iter().map(|c| c.points()).sum::<u32>() >= 30,
            Coup::Piocher => true,
        });
    }

    coups.sort_by_key(|coup| match coup {
        Coup::Poser(combinaisons) => {
            std::cmp::Reverse(combinaisons.iter().map(|c| c.points()).sum::<u32>())
        }
        Coup::Piocher => std::cmp::Reverse(0),
    });
    coups.push(Coup::Piocher);

    Ok(coups)
}

/// Joue le coup donné pour le joueur courant (tour ouvert requis) : une pose
/// retourne les points gagnés, une pioche retourne 0.
pub fn jouer_coup(partie: &mut Partie, coup: Coup) -> Result<u32, String> {
    match coup {
        Coup::Poser(combinaisons) => partie.poser_combinaisons(combinaisons),
        Coup::Piocher => partie.piocher().map(|_| 0),
    }
}

#[cfg(test)]
mod tests;
