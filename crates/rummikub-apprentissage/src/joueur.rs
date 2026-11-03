//! Le joueur-réseau : faire jouer la politique entraînée comme une IA.
//!
//! C'est la pièce manquante entre l'entraînement et la mesure de force : le
//! réseau produit des probabilités par type de tuile, mais une partie se joue
//! en coups légaux. Le décodeur fait le lien — il regarde les coups que les
//! règles autorisent (`coups_possibles`) et choisit celui dont l'étiquette
//! (types joués + drapeau piocher, au sens de l'ADR 0007) est la plus
//! vraisemblable selon le réseau. Le réseau propose, les règles disposent :
//! il ne peut jamais produire un coup illégal.
//!
//! ## Le décodage, expliqué
//!
//! Le réseau sort 54 probabilités indépendantes (53 types + pioche), et la
//! BCE l'a entraîné à donner à chaque drapeau sa probabilité d'être « vrai ».
//! Le coup le plus cohérent avec ces croyances est celui dont le vecteur
//! d'étiquettes a la plus grande log-vraisemblance :
//!
//! ```text
//! score = Σ_t  log p_t   (types joués, drapeau piocher inclus)
//!       + Σ_t  log(1−p_t) (types non joués)
//! ```
//!
//! Pour les coups non-pioche, les termes des types non joués sont identiques
//! d'un coup à l'autre : le classement se réduit à maximiser la somme des
//! logits des types effectivement joués — un coup gagne en lisant les types
//! « décidés vrais » par le réseau. Une pose est donc exactement le
//! sous-ensemble de types que le réseau croit vrais, sélectionné parmi les
//! poses légales — le chaînage de plusieurs coups dans un même tour reste
//! possible, comme chez l'ISMCTS.
//!
//! Le choix est glouton (argmax), sans échantillonnage : même réseau + même
//! état → même coup, la mesure est reproductible. Les ex æquo (deux poses de
//! types identiques) se départagent dans l'ordre déterministe de
//! `coups_possibles` (points décroissants), comme le veut l'ADR 0001.

use crate::donnees::cible_du_coup;
use crate::encodeur::NB_TYPES;
use crate::modele::Politique;
use burn::module::Module;
use burn::prelude::*;
use burn::store::ModuleRecord;
use rand::RngExt;
use rand::rngs::StdRng;
use rummikub_core::coups::{Coup, coups_possibles, jouer_coup};
use rummikub_core::partie::{Partie, Tour};
use rummikub_core::tuile::BitsetTuiles;
use rummikub_ia::bot_heuristique;
use rummikub_ia::ismcts::IsMcts;
use std::io::Write;
use std::time::{Duration, Instant, SystemTime};

/// Charge les poids entraînés (écrits par la sous-commande entraine) dans un
/// réseau frais de la bonne forme.
pub fn charger_politique(chemin: &str) -> Result<Politique, String> {
    let record = ModuleRecord::load(chemin).map_err(|erreur| erreur.to_string())?;
    let reseau = crate::modele::PolitiqueConfig::new(
        crate::encodeur::TAILLE_VECTEUR,
        crate::modele::NB_SORTIES,
    )
    .init(&Device::flex())
    .load_record(record);
    Ok(reseau)
}

/// Les probabilités (sigmoïde) du réseau pour un état, dans l'ordre des
/// sorties : 53 types puis le drapeau piocher.
pub fn probabilites(reseau: &Politique, entree: &[f32]) -> Result<Vec<f32>, String> {
    let device = Device::flex();
    let lot = Tensor::<1>::from_floats(entree, &device).reshape([1, entree.len()]);
    let logits = reseau.forward(lot);
    let sorties = burn::tensor::activation::sigmoid(logits).into_data();
    Ok(sorties.iter::<f32>().collect())
}

/// Le score (log-vraisemblance de l'étiquette) d'un coup, en s'appuyant sur
/// `cible_du_coup` — la même définition de « types joués » que le générateur.
/// None pour `Terminer` : le réseau n'a pas de cible pour ce coup (ADR 0007).
fn score_du_coup(coup: &Coup, ids_plateau: &BitsetTuiles, probabilites: &[f32]) -> Option<f64> {
    let (types_jouees, piocher) = cible_du_coup(coup, ids_plateau)?;

    // Bornes anti-log(0) : la sigmoïde peut s'arrondir à 0 ou 1 en f32.
    let p = |index: usize| -> f64 { (probabilites[index] as f64).clamp(1e-7, 1.0 - 1e-7) };
    let mut score = 0.0f64;
    for (index, &vrai) in types_jouees.iter().enumerate() {
        score += if vrai {
            p(index).ln()
        } else {
            (1.0 - p(index)).ln()
        };
    }
    let p_pioche = p(NB_TYPES);
    score += if piocher {
        p_pioche.ln()
    } else {
        (1.0 - p_pioche).ln()
    };
    Some(score)
}

/// Choisit le coup du joueur courant : le coup légal le plus vraisemblable
/// selon le réseau. None si la partie est déjà gagnée (plus rien à décider).
pub fn choisir_coup(reseau: &Politique, partie: &Partie) -> Result<Option<Coup>, String> {
    if partie.gagnant().is_some() {
        return Ok(None);
    }
    if !matches!(partie.tour, Tour::Ouvert { .. }) {
        return Err("Aucun tour en cours : appelez ouvrir_tour pour ouvrir un tour.".to_string());
    }

    let coups = coups_possibles(partie)?;
    let entree = crate::encodeur::encoder(partie, partie.index_joueur_courant)?;
    let probabilites = probabilites(reseau, &entree)?;
    let ids_plateau = partie.plateau.ids_tuiles();

    // argmax avec départage stable : le premier de l'ordre déterministe de
    // coups_possibles (points décroissants) gagne les ex æquo.
    let mut meilleur: Option<(f64, &Coup)> = None;
    for coup in &coups {
        let Some(score) = score_du_coup(coup, &ids_plateau, &probabilites) else {
            continue; // Terminer : jamais proposé
        };
        if meilleur.is_none_or(|(meilleur_score, _)| score > meilleur_score) {
            meilleur = Some((score, coup));
        }
    }

    // Défensif : si piocher existe toujours, ce cas est vide ; sinon il ne
    // reste que Terminer (le réseau n'a pas de cible pour ce coup, le pilote
    // le clôt à sa place).
    match meilleur {
        Some((_, coup)) => Ok(Some(coup.clone())),
        None => Ok(coups
            .iter()
            .find(|coup| matches!(coup, Coup::Terminer))
            .cloned()),
    }
}

/// Choisit un coup en échantillonnant la distribution softmax(score / tau)
/// sur les coups légaux — le mode exploratoire de l'entraînement par
/// renforcement (`selfplay`). Plus `tau` est grand, plus l'exploration est
/// large ; tau → 0 redonne le choix glouton. Reproductible via la graine.
pub fn choisir_coup_echantillonne(
    reseau: &Politique,
    partie: &Partie,
    rng: &mut StdRng,
    tau: f64,
) -> Result<Option<Coup>, String> {
    if partie.gagnant().is_some() {
        return Ok(None);
    }
    if !matches!(partie.tour, Tour::Ouvert { .. }) {
        return Err("Aucun tour en cours : appelez ouvrir_tour pour ouvrir un tour.".to_string());
    }

    let coups = coups_possibles(partie)?;
    let entree = crate::encodeur::encoder(partie, partie.index_joueur_courant)?;
    let probabilites = probabilites(reseau, &entree)?;
    let ids_plateau = partie.plateau.ids_tuiles();

    let scores: Vec<f64> = coups
        .iter()
        .filter_map(|coup| score_du_coup(coup, &ids_plateau, &probabilites))
        .collect();

    // Défensif : aucun coup scorable (liste réduite à Terminer) → le pilote clôt
    if scores.is_empty() {
        return Ok(coups
            .iter()
            .find(|coup| matches!(coup, Coup::Terminer))
            .cloned());
    }

    // softmax(score / tau) : la soustraction du max évite l'exp overflow
    let max_score = scores.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let poids: Vec<f64> = scores
        .iter()
        .map(|&score| ((score - max_score) / tau).exp())
        .collect();
    let total: f64 = poids.iter().sum();

    let tirage = rng.random::<f64>() * total;
    let mut cumul = 0.0;
    for (index, &poids) in poids.iter().enumerate() {
        cumul += poids;
        if tirage <= cumul {
            return Ok(Some(coups[index].clone()));
        }
    }
    Ok(Some(coups[coups.len() - 1].clone()))
}

/// Joue le tour complet du joueur courant avec le réseau : relance le choix
/// après chaque pose ou manipulation (un tour peut comporter plusieurs coups),
/// et s'arrête dès que le coup choisi clôt le tour (Piocher, Terminer) ou que
/// la main est vide (partie gagnée).
pub fn jouer_tour(reseau: &Politique, partie: &mut Partie) -> Result<(), String> {
    partie.ouvrir_tour();
    loop {
        let Some(coup) = choisir_coup(reseau, partie)? else {
            return Ok(()); // partie gagnée en cours de tour
        };
        jouer_coup(partie, coup)?;
        if partie.gagnant().is_some() || !matches!(partie.tour, Tour::Ouvert { .. }) {
            return Ok(());
        }
    }
}

/// L'adversaire du réseau dans le tournoi de mesure.
#[derive(Debug, Clone)]
pub enum Adversaire {
    /// Le bot de baseline : meilleure pose possible, sinon piocher
    Heuristique,
    /// Le professeur lui-même (ISMCTS), avec ses réglages
    Isme(IsMcts),
}

impl Adversaire {
    /// Joue le tour de la partie pour le joueur courant
    fn jouer_tour(&self, partie: &mut Partie) -> Result<(), String> {
        match self {
            Adversaire::Heuristique => bot_heuristique::jouer_tour(partie),
            Adversaire::Isme(ismcts) => ismcts.jouer_tour(partie),
        }
    }

    /// Le libellé du CSV du banc (public : la sous-commande joue l'affiche)
    pub fn libelle(&self) -> &str {
        match self {
            Adversaire::Heuristique => "heuristique",
            Adversaire::Isme(_) => "ismcts",
        }
    }

    /// Le nombre de simulations de l'adversaire (0 pour l'heuristique)
    fn simulations(&self) -> usize {
        match self {
            Adversaire::Heuristique => 0,
            Adversaire::Isme(ismcts) => ismcts.simulations,
        }
    }
}

/// Résumé d'un tournoi du réseau
#[derive(Debug, Clone, PartialEq)]
pub struct ResultatJoue {
    /// Victoires du joueur-réseau (joueur 0)
    pub victoires_reseau: u32,
    /// Victoires de l'adversaire (joueur 1)
    pub victoires_adversaire: u32,
    /// Parties non terminées dans la limite de tours
    pub non_terminees: u32,
    /// Durée totale du tournoi
    pub duree: Duration,
}

impl ResultatJoue {
    /// Taux de victoire du réseau sur les parties terminées
    pub fn taux_reseau(&self) -> f64 {
        let terminees = (self.victoires_reseau + self.victoires_adversaire) as f64;
        if terminees == 0.0 {
            return 0.0;
        }
        self.victoires_reseau as f64 / terminees
    }
}

/// Fait s'affronter le joueur-réseau (joueur 0) contre `adversaire` (joueur 1)
/// sur les graines `1..=nb_graines`. Même graine → mêmes donnes, mains
/// raccourcies à 6 tuiles, limite de 300 tours : le même cadre que le
/// tournoi de `rummikub_ia`, pour des chiffres comparables.
pub fn tournoi(
    nb_graines: u32,
    reseau: &Politique,
    adversaire: &Adversaire,
) -> Result<ResultatJoue, String> {
    let debut = Instant::now();
    let mut resultat = ResultatJoue {
        victoires_reseau: 0,
        victoires_adversaire: 0,
        non_terminees: 0,
        duree: Duration::ZERO,
    };

    for graine in 1..=nb_graines {
        let mut partie = Partie::nouvelle_avec_graine(
            vec!["Reseau".to_string(), adversaire.libelle().to_string()],
            graine as u64,
        )?;
        partie.joueurs[0].main.truncate(6);
        partie.joueurs[1].main.truncate(6);

        let mut tours = 0;
        while partie.gagnant().is_none() && tours < 300 {
            if partie.index_joueur_courant == 0 {
                jouer_tour(reseau, &mut partie)?;
            } else {
                adversaire.jouer_tour(&mut partie)?;
            }
            tours += 1;
        }

        match partie.gagnant() {
            Some(0) => resultat.victoires_reseau += 1,
            Some(1) => resultat.victoires_adversaire += 1,
            _ => resultat.non_terminees += 1,
        }
    }

    resultat.duree = debut.elapsed();
    Ok(resultat)
}

/// Ajoute une ligne de résultats du joueur-réseau au CSV du banc (créé avec
/// son en-tête au besoin), dans l'esprit de `rummikub_ia::tournoi`. Les
/// simulations du professeur (celui du dataset d'entraînement) sont
/// enregistrées : le graphique du banc distingue les générations de réseaux.
pub fn enregistrer_resultat(
    chemin: &str,
    adversaire: &Adversaire,
    nb_graines: u32,
    professeur: u32,
    resultat: &ResultatJoue,
) -> Result<(), String> {
    if let Some(dossier) = std::path::Path::new(chemin).parent()
        && !dossier.as_os_str().is_empty()
    {
        std::fs::create_dir_all(dossier).map_err(|erreur| erreur.to_string())?;
    }

    let nouveau_fichier = !std::path::Path::new(chemin).exists();
    let mut fichier = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(chemin)
        .map_err(|erreur| erreur.to_string())?;

    if nouveau_fichier {
        writeln!(
            fichier,
            "date,adversaire,simulations,graines,professeur,victoires_reseau,victoires_adversaire,non_terminees,duree_s"
        )
        .map_err(|erreur| erreur.to_string())?;
    }

    let horodatage = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map_err(|erreur| erreur.to_string())?
        .as_secs();

    writeln!(
        fichier,
        "{horodatage},{},{},{},{},{},{},{},{:.2}",
        adversaire.libelle(),
        adversaire.simulations(),
        nb_graines,
        professeur,
        resultat.victoires_reseau,
        resultat.victoires_adversaire,
        resultat.non_terminees,
        resultat.duree.as_secs_f64()
    )
    .map_err(|erreur| erreur.to_string())
}

/// Une ligne du CSV du banc des tournois du réseau
#[derive(Debug, Clone, PartialEq)]
struct LigneBanc {
    adversaire: String,
    simulations_adversaire: usize,
    professeur: u32,
    nb_graines: u32,
    victoires_reseau: u32,
    victoires_adversaire: u32,
}

impl LigneBanc {
    /// Le taux de victoire du réseau sur les parties terminées
    fn taux_reseau(&self) -> f64 {
        let terminees = (self.victoires_reseau + self.victoires_adversaire) as f64;
        if terminees == 0.0 {
            return 0.0;
        }
        self.victoires_reseau as f64 / terminees
    }

    /// L'étiquette de la barre du graphique
    fn etiquette(&self) -> String {
        format!(
            "{} ×{} (prof. {})",
            self.adversaire, self.nb_graines, self.professeur
        )
    }
}

/// Relit le CSV du banc des tournois. Tolérant aux lignes de l'ancien
/// schéma (8 colonnes, sans la colonne professeur) : elles datent des
/// réseaux entraînés sur le professeur à 20 simulations.
fn lire_banc(chemin_csv: &str) -> Result<Vec<LigneBanc>, String> {
    let contenu = std::fs::read_to_string(chemin_csv).map_err(|erreur| erreur.to_string())?;
    let mut lignes = Vec::new();
    for ligne in contenu.lines().skip(1) {
        let champs: Vec<&str> = ligne.split(',').collect();
        if champs.len() < 8 {
            return Err(format!("ligne du banc mal formée : {ligne}"));
        }
        // Schéma 9 colonnes : professeur en 4e ; schéma 8 : prof. = 20.
        // Dans les deux cas : adversaire 1, simulations 2, graines 3.
        let professeur = if champs.len() >= 9 {
            champs[4]
                .parse()
                .map_err(|erreur: std::num::ParseIntError| erreur.to_string())?
        } else {
            20
        };
        let (index_vr, index_va) = if champs.len() >= 9 { (5, 6) } else { (4, 5) };
        lignes.push(LigneBanc {
            adversaire: champs[1].to_string(),
            simulations_adversaire: champs[2]
                .parse()
                .map_err(|erreur: std::num::ParseIntError| erreur.to_string())?,
            professeur,
            nb_graines: champs[3]
                .parse()
                .map_err(|erreur: std::num::ParseIntError| erreur.to_string())?,
            victoires_reseau: champs[index_vr]
                .parse()
                .map_err(|erreur: std::num::ParseIntError| erreur.to_string())?,
            victoires_adversaire: champs[index_va]
                .parse()
                .map_err(|erreur: std::num::ParseIntError| erreur.to_string())?,
        });
    }
    Ok(lignes)
}

/// Trace le taux de victoire du réseau pour chaque ligne du CSV du banc
/// (`banc/joue.csv`) : une barre par tournoi enregistré, étiquetée
/// adversaire, nombre de graines et professeur. La lecture se fait à chaque
/// mesure — le graphique se met à jour tout seul.
pub fn tracer_joue(chemin_csv: &str, chemin_png: &str) -> Result<(), String> {
    use plotters::prelude::*;

    let lignes = lire_banc(chemin_csv)?;
    if lignes.is_empty() {
        return Err(format!("{chemin_csv} ne contient aucun tournoi"));
    }

    let n = lignes.len();
    let racine = BitMapBackend::new(chemin_png, (900, 500)).into_drawing_area();
    racine.fill(&WHITE).map_err(|erreur| erreur.to_string())?;
    let mut diagramme = ChartBuilder::on(&racine)
        .caption(
            "Taux de victoire du joueur-réseau (parties terminées)",
            ("sans-serif", 22),
        )
        .margin(15)
        .x_label_area_size(45)
        .y_label_area_size(60)
        .build_cartesian_2d(0.0f64..n as f64, 0.0f64..1.1)
        .map_err(|erreur| erreur.to_string())?;
    diagramme
        .configure_mesh()
        .x_desc("tournois enregistrés (ordre du CSV)")
        .y_desc("taux de victoire")
        .disable_x_mesh()
        .disable_y_mesh()
        .draw()
        .map_err(|erreur| erreur.to_string())?;

    for (index, ligne) in lignes.iter().enumerate() {
        let couleur = if ligne.adversaire == "ismcts" {
            &BLUE
        } else {
            &RED
        };
        let centre = index as f64 + 0.5;
        let barre = diagramme
            .draw_series(std::iter::once(Rectangle::new(
                [(centre - 0.4, 0.0), (centre + 0.4, ligne.taux_reseau())],
                couleur.mix(0.6).filled(),
            )))
            .map_err(|erreur| erreur.to_string())?;
        barre
            .label(ligne.etiquette())
            .legend(move |(x, y)| Rectangle::new([(x, y - 5), (x + 15, y + 5)], couleur.mix(0.6)));
    }
    diagramme
        .configure_series_labels()
        .background_style(WHITE.mix(0.8))
        .border_style(BLACK)
        .draw()
        .map_err(|erreur| erreur.to_string())?;
    racine.present().map_err(|erreur| erreur.to_string())
}

#[cfg(test)]
mod tests;
