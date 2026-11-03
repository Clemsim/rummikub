//! La génération et le stockage du dataset d'imitation.
//!
//! Principe : des parties ISMCTS contre ISMCTS sur graines fixes. À chaque
//! décision (chaque coup du tour), on enregistre **l'état encodé** et **le
//! coup joué** — la cible est au niveau du coup, pas du tour :
//! - une pose étiquette les types de tuiles qu'elle dépose ;
//! - une manipulation n'étiquette que les tuiles qu'elle apporte depuis la
//!   main (celles déjà sur le plateau ne comptent pas) ;
//! - une pioche pose le drapeau piocher (y compris une passe à sac vide) ;
//! - `Terminer` ne produit pas d'exemple : le réseau ne décide pas de
//!   terminer, c'est l'affaire du pilote quand plus rien n'est jouable.
//!
//! Le générateur pilote la boucle de tour avec l'API publique du core
//! (`choisir_coup` + `jouer_coup`) : la supervision ne peut jamais tricher.

use crate::encodeur::{NB_TYPES, TAILLE_VECTEUR, encoder, index_type};
use rummikub_core::coups::{Coup, jouer_coup};
use rummikub_core::partie::{Partie, Tour};
use rummikub_core::tuile::BitsetTuiles;
use rummikub_ia::ismcts::IsMcts;
use std::fs::OpenOptions;
use std::io::Write;
use std::time::{Duration, SystemTime};

/// Un exemple d'apprentissage : une décision de l'ISMCTS
#[derive(Debug, Clone, PartialEq)]
pub struct Exemple {
    /// Le joueur qui a pris la décision
    pub id_joueur: usize,
    /// L'état encodé (TAILLE_VECTEUR valeurs)
    pub entree: Vec<f32>,
    /// Les types de tuiles jouées par ce coup
    pub types_jouees: [bool; NB_TYPES],
    /// Le coup était-il une pioche (y compris une passe à sac vide) ?
    pub piocher: bool,
}

/// La cible d'apprentissage d'un coup : les types de tuiles jouées depuis la
/// main, et le drapeau piocher. `Terminer` retourne None (pas de classe).
fn cible_du_coup(coup: &Coup, ids_plateau: &BitsetTuiles) -> Option<([bool; NB_TYPES], bool)> {
    let mut types_jouees = [false; NB_TYPES];

    match coup {
        Coup::Poser(combinaisons) => {
            for combinaison in combinaisons {
                for tuile in &combinaison.Tuiles {
                    types_jouees[index_type(tuile)] = true;
                }
            }
        }
        Coup::Manipuler(combinaisons) => {
            for combinaison in combinaisons {
                for tuile in &combinaison.Tuiles {
                    // Seules les tuiles venues de la main sont « jouées »
                    if !ids_plateau.contient(tuile.id) {
                        types_jouees[index_type(tuile)] = true;
                    }
                }
            }
        }
        Coup::Piocher => return Some((types_jouees, true)),
        Coup::Terminer => return None,
    }

    Some((types_jouees, false))
}

/// Génère le dataset : `nb_parties` d'ISMCTS contre ISMCTS sur les graines
/// `1..=nb_parties`. Un exemple par décision.
pub fn generer_dataset(nb_parties: u32, ismcts: &IsMcts) -> Result<Vec<Exemple>, String> {
    let mut exemples = Vec::new();

    for graine in 1..=nb_parties {
        let mut partie = Partie::nouvelle_avec_graine(
            vec!["ISMCTS-1".to_string(), "ISMCTS-2".to_string()],
            graine as u64,
        )?;

        let mut tours = 0;
        while partie.gagnant().is_none() && tours < 300 {
            partie.ouvrir_tour();
            let joueur = partie.index_joueur_courant;

            // Le même protocole que jouer_tour, mais en enregistrant chaque
            // décision
            loop {
                let entree = encoder(&partie, joueur)?;
                let coup = ismcts
                    .choisir_coup(&partie)?
                    .expect("Piocher est toujours un coup disponible");
                let ids_plateau = partie.plateau.ids_tuiles();

                if let Some((types_jouees, piocher)) = cible_du_coup(&coup, &ids_plateau) {
                    exemples.push(Exemple {
                        id_joueur: joueur,
                        entree,
                        types_jouees,
                        piocher,
                    });
                }

                jouer_coup(&mut partie, coup)?;
                // Main vide en cours de tour = partie gagnée : plus rien à décider
                if partie.gagnant().is_some() || !matches!(partie.tour, Tour::Ouvert { .. }) {
                    break;
                }
            }
            tours += 1;
        }
    }

    Ok(exemples)
}

/// Écrit le dataset en CSV : `joueur`, les 53 cibles de types, le drapeau
/// piocher, puis les 116 entrées. Retourne le nombre d'exemples écrits.
pub fn ecrire_dataset(chemin: &str, exemples: &[Exemple]) -> Result<usize, String> {
    if let Some(dossier) = std::path::Path::new(chemin).parent()
        && !dossier.as_os_str().is_empty()
    {
        std::fs::create_dir_all(dossier).map_err(|erreur| erreur.to_string())?;
    }

    let mut entetes: Vec<String> = (0..NB_TYPES).map(|index| format!("t{index}")).collect();
    entetes.push("p".to_string());
    entetes.extend((0..TAILLE_VECTEUR).map(|index| format!("e{index}")));
    entetes.insert(0, "joueur".to_string());

    let mut contenu = entetes.join(",");
    contenu.push('\n');
    for exemple in exemples {
        let mut ligne = vec![exemple.id_joueur.to_string()];
        ligne.extend(
            exemple
                .types_jouees
                .iter()
                .map(|&vrai| if vrai { "1" } else { "0" }.to_string()),
        );
        ligne.push(if exemple.piocher { "1" } else { "0" }.to_string());
        ligne.extend(exemple.entree.iter().map(|valeur| format!("{valeur}")));
        contenu.push_str(&ligne.join(","));
        contenu.push('\n');
    }

    std::fs::write(chemin, contenu).map_err(|erreur| erreur.to_string())?;
    Ok(exemples.len())
}

/// Relit un dataset écrit par [`ecrire_dataset`]
pub fn lire_dataset(chemin: &str) -> Result<Vec<Exemple>, String> {
    let contenu = std::fs::read_to_string(chemin).map_err(|erreur| erreur.to_string())?;
    let mut lignes = contenu.lines();
    let entete = lignes.next().ok_or("dataset vide")?;
    let taille_attendue = 2 + NB_TYPES + TAILLE_VECTEUR;
    if entete.split(',').count() != taille_attendue {
        return Err(format!(
            "en-tête inattendu (attendu {taille_attendue} colonnes)"
        ));
    }

    let mut exemples = Vec::new();
    for ligne in lignes {
        let champs: Vec<&str> = ligne.split(',').collect();
        if champs.len() != taille_attendue {
            return Err(format!(
                "ligne de dataset mal formée ({} champs au lieu de {taille_attendue})",
                champs.len()
            ));
        }

        let id_joueur: usize = champs[0]
            .parse::<usize>()
            .map_err(|erreur: std::num::ParseIntError| erreur.to_string())?;
        let mut curseur = 1;

        let mut types_jouees = [false; NB_TYPES];
        for type_flag in &mut types_jouees {
            *type_flag = champs[curseur] == "1";
            curseur += 1;
        }
        let piocher = champs[curseur] == "1";
        curseur += 1;

        let entree: Vec<f32> = champs[curseur..]
            .iter()
            .map(|champ| champ.parse::<f32>().map_err(|erreur| erreur.to_string()))
            .collect::<Result<Vec<_>, _>>()?;

        exemples.push(Exemple {
            id_joueur,
            entree,
            types_jouees,
            piocher,
        });
    }

    Ok(exemples)
}

/// Quelques compteurs sur un dataset
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Statistiques {
    /// Nombre total d'exemples
    pub exemples: usize,
    /// Exemples où au moins une tuile a été jouée (poses ou manipulations)
    pub poses: usize,
    /// Exemples de pioche (y compris les passes à sac vide)
    pub pioches: usize,
}

pub fn statistiques(exemples: &[Exemple]) -> Statistiques {
    let mut stats = Statistiques::default();
    for exemple in exemples {
        stats.exemples += 1;
        if exemple.types_jouees.iter().any(|&vrai| vrai) {
            stats.poses += 1;
        } else if exemple.piocher {
            stats.pioches += 1;
        }
    }
    stats
}

/// Ajoute une ligne de statistiques de génération au CSV du banc
pub fn enregistrer_statistiques(
    chemin: &str,
    nb_parties: u32,
    simulations: usize,
    stats: &Statistiques,
    duree: Duration,
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
            "date,parties,simulations,exemples,poses,pioches,duree_s"
        )
        .map_err(|erreur| erreur.to_string())?;
    }

    let horodatage = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map_err(|erreur| erreur.to_string())?
        .as_secs();

    writeln!(
        fichier,
        "{horodatage},{nb_parties},{simulations},{},{},{},{:.2}",
        stats.exemples,
        stats.poses,
        stats.pioches,
        duree.as_secs_f64()
    )
    .map_err(|erreur| erreur.to_string())
}

#[cfg(test)]
mod tests;
