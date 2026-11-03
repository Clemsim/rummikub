//! L'encodeur : transformer une vue de partie en vecteur de nombres de
//! taille fixe — l'entrée du futur réseau.
//!
//! Le vecteur est indexé par **type de tuile** (52 couples couleur × valeur +
//! le joker), en application du principe d'exemplaires interchangeables
//! (ADR 0001) : une combinaison ne peut pas contenir deux exemplaires
//! identiques, donc une pose est exactement un sous-ensemble de types.
//!
//! Composition du vecteur (116 valeurs, dans l'ordre) :
//! - [0 .. 53]   : ma main, comptage par type (0, 0.5, 1 — normalisé /2) ;
//! - [53 .. 106] : le plateau, comptage par type (0, 0.5, 1) ;
//! - [106]       : taille de la pioche / 106 ;
//! - [107 .. 113]: adversaires — 3 emplacements × (taille de main / 20,
//!   pose initiale faite), remplis à 0 au-delà du nombre d'adversaires ;
//! - [113]       : ma pose initiale (0/1) ;
//! - [114]       : victoire à moi (0/1) ;
//! - [115]       : tours sans pose / nombre de joueurs (proximité du blocage).

use rummikub_core::partie::{Partie, VuePartie};
use rummikub_core::tuile::{Tuile, TuileType};

/// Types de tuiles canoniques : 52 couples couleur × valeur + le joker
pub const NB_TYPES: usize = 53;

/// Taille du vecteur encodé
pub const TAILLE_VECTEUR: usize = 2 * NB_TYPES + 10;

/// L'index canonique d'un type de tuile dans le vecteur (publique : le
/// décodeur du bot-réseau en aura besoin)
pub fn index_type(tuile: &Tuile) -> usize {
    match tuile.Type {
        TuileType::Standard { couleur, valeur } => couleur as usize * 13 + valeur as usize - 1,
        TuileType::Joker => NB_TYPES - 1,
    }
}

/// Encode la partie du point de vue du joueur `id_joueur`
pub fn encoder(partie: &Partie, id_joueur: usize) -> Result<Vec<f32>, String> {
    let vue = partie.vue_pour(id_joueur)?;
    Ok(encoder_vue(&vue))
}

/// Encode une vue de partie — le même objet que verrait un client réseau
pub fn encoder_vue(vue: &VuePartie) -> Vec<f32> {
    let mut vecteur = vec![0.0; TAILLE_VECTEUR];

    // Ma main, comptée par type (deux exemplaires remplissent la case)
    let moi = vue
        .joueurs
        .iter()
        .find(|joueur| joueur.id == vue.id_joueur)
        .expect("la vue contient le joueur demandé");
    if let Some(main) = &moi.main {
        for tuile in main {
            vecteur[index_type(tuile)] += 0.5;
        }
    }

    // Le plateau, compté par type
    for combinaison in &vue.plateau.combinaisons {
        for tuile in &combinaison.Tuiles {
            vecteur[NB_TYPES + index_type(tuile)] += 0.5;
        }
    }

    // La pioche
    vecteur[2 * NB_TYPES] = vue.taille_pioche as f32 / 106.0;

    // Les adversaires : au plus 3 (le jeu se joue de 2 à 4)
    for (slot, adversaire) in vue
        .joueurs
        .iter()
        .filter(|joueur| joueur.id != vue.id_joueur)
        .take(3)
        .enumerate()
    {
        vecteur[2 * NB_TYPES + 1 + slot * 2] = adversaire.taille_main as f32 / 20.0;
        vecteur[2 * NB_TYPES + 2 + slot * 2] = if adversaire.a_fait_pose_initiale {
            1.0
        } else {
            0.0
        };
    }

    // Ma pose initiale, la victoire, la proximité du blocage
    let index_pose_moi = 2 * NB_TYPES + 7;
    vecteur[index_pose_moi] = if moi.a_fait_pose_initiale { 1.0 } else { 0.0 };
    vecteur[index_pose_moi + 1] = if vue.gagnant == Some(vue.id_joueur) {
        1.0
    } else {
        0.0
    };
    vecteur[index_pose_moi + 2] = vue.tours_sans_pose as f32 / vue.joueurs.len() as f32;

    vecteur
}

#[cfg(test)]
mod tests;
