//! Le banc d'essai : des tournois à graines fixes pour mesurer les IA.
//! Chaque amélioration de l'ISMCTS doit se justifier ici, victoires à
//! l'appui.

use crate::bot_heuristique;
use crate::ismcts::IsMcts;
use rummikub_core::partie::Partie;
use std::fs::OpenOptions;
use std::io::Write;
use std::time::{Duration, Instant, SystemTime};

/// Résumé d'un tournoi
#[derive(Debug, Clone, PartialEq)]
pub struct Resultat {
    /// Victoires du joueur ISMCTS (joueur 0)
    pub victoires_ismcts: u32,
    /// Victoires du joueur heuristique (joueur 1)
    pub victoires_heuristique: u32,
    /// Parties non terminées dans la limite de tours
    pub non_terminees: u32,
    /// Durée totale du tournoi
    pub duree: Duration,
}

impl Resultat {
    /// Taux de victoire de l'ISMCTS sur les parties terminées
    pub fn taux_ismcts(&self) -> f64 {
        let terminees = (self.victoires_ismcts + self.victoires_heuristique) as f64;
        if terminees == 0.0 {
            return 0.0;
        }
        self.victoires_ismcts as f64 / terminees
    }
}

/// Fait s'affronter l'ISMCTS (joueur 0) contre le bot heuristique (joueur 1)
/// sur les graines `1..=nb_graines`. Même graine → mêmes donnes pour les deux
/// joueurs : la comparaison est équitable. Les mains sont raccourcies à 6
/// tuiles pour garder chaque partie rapide ; ce cadre de mesure doit rester
/// stable pour comparer les versions entre elles.
pub fn tournoi(nb_graines: u32, ismcts: &IsMcts) -> Result<Resultat, String> {
    let debut = Instant::now();
    let mut resultat = Resultat {
        victoires_ismcts: 0,
        victoires_heuristique: 0,
        non_terminees: 0,
        duree: Duration::ZERO,
    };

    for graine in 1..=nb_graines {
        let mut partie = Partie::nouvelle_avec_graine(
            vec!["ISMCTS".to_string(), "Heuristique".to_string()],
            graine as u64,
        )?;
        partie.joueurs[0].main.truncate(6);
        partie.joueurs[1].main.truncate(6);

        let mut tours = 0;
        while partie.gagnant().is_none() && tours < 300 {
            if partie.index_joueur_courant == 0 {
                ismcts.jouer_tour(&mut partie)?;
            } else {
                bot_heuristique::jouer_tour(&mut partie)?;
            }
            tours += 1;
        }

        match partie.gagnant() {
            Some(0) => resultat.victoires_ismcts += 1,
            Some(1) => resultat.victoires_heuristique += 1,
            _ => resultat.non_terminees += 1,
        }
    }

    resultat.duree = debut.elapsed();
    Ok(resultat)
}

/// Balaye des valeurs de réglage : pour chaque valeur, l'ISMCTS construit
/// par `fabriquer` joue le même tournoi (mêmes graines — comparaison
/// équitable) et la fonction retourne les points (valeur, taux de victoire
/// de l'ISMCTS).
pub fn balayage(
    valeurs: &[f64],
    nb_graines: u32,
    fabriquer: impl Fn(f64) -> IsMcts,
) -> Result<Vec<(f64, f64)>, String> {
    valeurs
        .iter()
        .map(|&valeur| {
            let resultat = tournoi(nb_graines, &fabriquer(valeur))?;
            Ok((valeur, resultat.taux_ismcts()))
        })
        .collect()
}

/// Enregistre une ligne de résultats dans le CSV du banc (créé avec son
/// en-tête au besoin), pour suivre l'évolution des IA au fil des versions.
/// L'horodatage est en secondes depuis l'epoch (format CSV, sans dépendance).
pub fn enregistrer_resultat(
    chemin: &str,
    epsilon: f64,
    nb_graines: u32,
    simulations: usize,
    resultat: &Resultat,
) -> Result<(), String> {
    if let Some(dossier) = std::path::Path::new(chemin).parent()
        && !dossier.as_os_str().is_empty()
    {
        std::fs::create_dir_all(dossier).map_err(|erreur| erreur.to_string())?;
    }

    let nouveau_fichier = !std::path::Path::new(chemin).exists();
    let mut fichier = OpenOptions::new()
        .create(true)
        .append(true)
        .open(chemin)
        .map_err(|erreur| erreur.to_string())?;

    if nouveau_fichier {
        writeln!(
            fichier,
            "date,epsilon,graines,simulations,victoires_ismcts,victoires_heuristique,non_terminees,duree_s"
        )
        .map_err(|erreur| erreur.to_string())?;
    }

    let horodatage = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map_err(|erreur| erreur.to_string())?
        .as_secs();

    writeln!(
        fichier,
        "{horodatage},{epsilon},{nb_graines},{simulations},{},{},{},{:.2}",
        resultat.victoires_ismcts,
        resultat.victoires_heuristique,
        resultat.non_terminees,
        resultat.duree.as_secs_f64()
    )
    .map_err(|erreur| erreur.to_string())
}

#[cfg(test)]
mod tests;
