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
use std::collections::{BTreeMap, HashSet};

/// Un coup que le joueur courant peut tenter, tour ouvert.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Coup {
    /// Poser une ou plusieurs combinaisons construites depuis sa main
    Poser(Vec<Combinaison>),
    /// Remanier le plateau : soumettre un nouvel état complet où rien ne
    /// disparaît et où chaque nouvelle tuile vient de la main
    Manipuler(Vec<Combinaison>),
    /// Terminer son tour (légal après avoir joué, ou à pioche vide pour passer)
    Terminer,
    /// Piocher une tuile (termine le tour ; passe si la pioche est vide)
    Piocher,
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
/// ouvert (ouvrir_tour). Ordre du résultat : les poses par points
/// décroissants, puis les manipulations, puis Terminer (si disponible) et
/// Piocher en dernier.
///
/// Simplifications assumées de cette version :
/// - avant la pose initiale, les poses combinées couvrent les paires de
///   combinaisons disjointes atteignant 30 points ; trois petites
///   combinaisons (ex. 10 + 10 + 10) ne sont pas énumérées ;
/// - les manipulations couvrent l'ajout d'une tuile de la main à une
///   combinaison du plateau, et la libération du joker d'un groupe (la vraie
///   tuile prend sa place, le joker rejoint une autre combinaison) ; libérer
///   le joker d'une suite viendra ensuite.
pub fn coups_possibles(partie: &Partie) -> Result<Vec<Coup>, String> {
    if !matches!(partie.tour, Tour::Ouvert { .. }) {
        return Err("Aucun tour en cours : appelez ouvrir_tour pour ouvrir un tour.".to_string());
    }

    let joueur = partie.joueur_courant();
    let candidates = combinaisons_possibles(&joueur.main);

    let mut coups: Vec<Coup> = if joueur.a_fait_pose_initiale {
        // Chaque combinaison candidate est une pose possible à elle seule ;
        // poser plusieurs combinaisons se fait en enchaînant plusieurs poses
        // dans le même tour.
        candidates
            .into_iter()
            .map(|combinaison| Coup::Poser(vec![combinaison]))
            .collect()
    } else {
        poses_initiales(&candidates)
    };

    // Les manipulations du plateau ne sont autorisées qu'après la pose initiale
    if joueur.a_fait_pose_initiale {
        coups.extend(
            manipulations_possibles(partie)
                .into_iter()
                .map(Coup::Manipuler),
        );
    }

    coups.sort_by_key(|coup| match coup {
        Coup::Poser(combinaisons) => {
            std::cmp::Reverse(combinaisons.iter().map(|c| c.points()).sum::<u32>())
        }
        // les manipulations ne rapportent pas de points : après toutes les poses
        Coup::Manipuler(_) => std::cmp::Reverse(0),
        Coup::Terminer => std::cmp::Reverse(0),
        Coup::Piocher => std::cmp::Reverse(0),
    });
    // Terminer son tour est légal après avoir joué quelque chose, ou à
    // pioche vide (la passe est alors la seule action possible)
    if partie.a_deja_joue_ce_tour() || partie.pioche.is_empty() {
        coups.push(Coup::Terminer);
    }
    coups.push(Coup::Piocher);

    Ok(coups)
}

/// Les poses légales avant la pose initiale : les combinaisons valant au
/// moins 30 points à elles seules, et les paires de combinaisons disjointes
/// dont la somme atteint 30 (aucune des deux ne valant 30 seule — sinon la
/// pose simple existe déjà, et une pose en deux temps suffit).
fn poses_initiales(candidates: &[Combinaison]) -> Vec<Coup> {
    let mut coups: Vec<Coup> = candidates
        .iter()
        .filter(|combinaison| combinaison.points() >= 30)
        .map(|combinaison| Coup::Poser(vec![combinaison.clone()]))
        .collect();

    let petites: Vec<(&Combinaison, u32, HashSet<u8>)> = candidates
        .iter()
        .filter(|combinaison| combinaison.points() < 30)
        .map(|combinaison| {
            (
                combinaison,
                combinaison.points(),
                combinaison.Tuiles.iter().map(|t| t.id).collect(),
            )
        })
        .collect();

    for i in 0..petites.len() {
        for j in (i + 1)..petites.len() {
            let (a, points_a, ids_a) = &petites[i];
            let (b, points_b, ids_b) = &petites[j];
            if points_a + points_b >= 30 && ids_a.is_disjoint(ids_b) {
                coups.push(Coup::Poser(vec![(*a).clone(), (*b).clone()]));
            }
        }
    }

    coups
}

/// Énumère les manipulations du plateau réalisables en une seule étape avec
/// la main du joueur courant. Chaque résultat est un état complet du plateau,
/// prêt pour soumettre_plateau ; les chaînes de manipulations se font en
/// enchaînant plusieurs coups dans le même tour.
fn manipulations_possibles(partie: &Partie) -> Vec<Vec<Combinaison>> {
    let main = &partie.joueur_courant().main;
    let mut etats: Vec<Vec<Combinaison>> = Vec::new();

    for (index_meld, meld) in partie.plateau.combinaisons.iter().enumerate() {
        // 1. Ajouter une tuile de la main à cette combinaison du plateau
        for tuile in main {
            let mut etendue = meld.clone();
            etendue.Tuiles.push(*tuile);
            if etendue.est_valide() {
                etats.push(etat_avec(partie, &[(index_meld, etendue)]));
            }
        }

        // 2. Libérer le joker d'un groupe : la vraie tuile (depuis la main)
        //    prend sa place, et le joker rejoint une autre combinaison.
        if meld.est_groupe_valide()
            && let Some(position_joker) = meld.Tuiles.iter().position(|t| t.est_joker())
            && let Some(valeur) = valeur_du_groupe(meld)
        {
            let couleurs_groupe: HashSet<Couleur> = meld
                .Tuiles
                .iter()
                .filter_map(|t| match t.Type {
                    TuileType::Standard { couleur, .. } => Some(couleur),
                    _ => None,
                })
                .collect();
            let joker_libre = meld.Tuiles[position_joker];

            for tuile in main {
                let TuileType::Standard { couleur, valeur: v } = tuile.Type else {
                    continue;
                };
                if v != valeur || couleurs_groupe.contains(&couleur) {
                    continue;
                }

                let mut groupe_reconstitue = meld.clone();
                groupe_reconstitue.Tuiles[position_joker] = *tuile;
                if !groupe_reconstitue.est_valide() {
                    continue;
                }

                for (autre_index, autre) in partie.plateau.combinaisons.iter().enumerate() {
                    if autre_index == index_meld {
                        continue;
                    }
                    let mut etendue = autre.clone();
                    etendue.Tuiles.push(joker_libre);
                    if etendue.est_valide() {
                        etats.push(etat_avec(
                            partie,
                            &[
                                (index_meld, groupe_reconstitue.clone()),
                                (autre_index, etendue),
                            ],
                        ));
                    }
                }
            }
        }
    }

    etats
}

/// L'état du plateau où les combinaisons listées dans `changements` sont
/// remplacées (les autres restent inchangées)
fn etat_avec(partie: &Partie, changements: &[(usize, Combinaison)]) -> Vec<Combinaison> {
    partie
        .plateau
        .combinaisons
        .iter()
        .enumerate()
        .filter(|(index, _)| !changements.iter().any(|(i, _)| i == index))
        .map(|(_, meld)| meld.clone())
        .chain(changements.iter().map(|(_, meld)| meld.clone()))
        .collect()
}

/// La valeur définie par une combinaison de type groupe, s'il y en a une
fn valeur_du_groupe(meld: &Combinaison) -> Option<u8> {
    meld.Tuiles.iter().find_map(|t| match t.Type {
        TuileType::Standard { valeur, .. } => Some(valeur),
        _ => None,
    })
}

/// Joue le coup donné pour le joueur courant (tour ouvert requis) : une pose
/// retourne les points gagnés, une manipulation ou une pioche retourne 0.
pub fn jouer_coup(partie: &mut Partie, coup: Coup) -> Result<u32, String> {
    match coup {
        Coup::Poser(combinaisons) => partie.poser_combinaisons(combinaisons),
        Coup::Manipuler(combinaisons) => {
            partie.soumettre_plateau(combinaisons)?;
            Ok(0)
        }
        Coup::Terminer => {
            partie.terminer_tour()?;
            Ok(0)
        }
        Coup::Piocher => partie.piocher().map(|_| 0),
    }
}

#[cfg(test)]
mod tests;
