//! L'entraînement par renforcement en auto-partie (REINFORCE).
//!
//! ## L'idée, expliquée
//!
//! L'imitation copie le professeur : elle ne peut pas le dépasser. Pour aller
//! au-delà, le réseau joue contre lui-même et apprend de ses résultats :
//! c'est l'auto-apprentissage (self-play). L'algorithme retenu est REINFORCE,
//! le plus simple des gradients de politique :
//!
//! 1. **Collecte** : des auto-parties échantillonnées — à chaque décision, un
//!    coup légal tiré selon la distribution softmax(score/tau) du réseau
//!    (`choisir_coup_echantillonne`). L'échantillonnage est l'exploration ;
//!    le choix glouton de la mesure reste inchangé (ADR 0008).
//! 2. **Récompense** : +1 pour le gagnant, −1 pour le perdant, 0 si la partie
//!    est bloquée (même convention que la limite de tours du banc).
//! 3. **Mise à jour** : REINFORCE — remonter la log-probabilité des coups des
//!    parties gagnées, abaisser celle des parties perdues. La log-prob du
//!    coup choisi est exactement l'opposé de la somme des BCE des 54 sorties
//!    avec l'étiquette du coup : la mise à jour reprend la même loss que
//!    l'imitation, pondérée par l'avantage (récompense − moyenne).
//!
//! Le réseau part des poids d'imitation (`politique.burnpack`) : le gradient
//! de politique affine un comportement déjà bon au lieu de repartir de zéro.
//! Les deux joueurs s'entraînent : chaque décision rapporte la récompense de
//! son propre joueur, l'adversaire apprend en parallèle — jeu à somme nulle.

use crate::donnees::cible_du_coup;
use crate::encodeur::{TAILLE_VECTEUR, encoder};
use crate::modele::{NB_SORTIES, Politique, PolitiqueConfig};
use burn::module::Module;
use burn::optim::{AdamConfig, GradientsParams};
use burn::prelude::*;
use burn::store::ModuleRecord;
use rand::SeedableRng;
use rand::rngs::StdRng;
use rummikub_core::coups::jouer_coup;
use rummikub_core::partie::{Partie, Tour};
use rummikub_ia::ismcts::IsMcts;
use std::time::{Duration, Instant};

/// Une décision d'auto-partie : l'état encodé, l'étiquette du coup choisi
/// (53 types + piocher) et la récompense du joueur qui l'a jouée
#[derive(Debug, Clone)]
struct Decision {
    id_joueur: usize,
    entree: Vec<f32>,
    cible: Vec<f32>,
    recompense: f32,
}

/// Le résultat d'une auto-partie
#[derive(Debug, Clone)]
struct PartieAuto {
    decisions: Vec<Decision>,
    /// Le gagnant (0 ou 1), None si la partie est bloquée
    gagnant: Option<usize>,
}

/// Le résumé d'un cycle d'entraînement
#[derive(Debug, Clone)]
pub struct Resume {
    /// Auto-parties jouées
    pub parties: u32,
    /// Décisions enregistrées
    pub decisions: usize,
    /// Victoires du premier joueur
    pub victoires_joueur1: u32,
    /// Victoires du second joueur
    pub victoires_joueur2: u32,
    /// Parties bloquées dans la limite de tours
    pub non_terminees: u32,
    /// La loss REINFORCE moyenne (pondérée par les avantages)
    pub perte: f32,
    /// Durée du cycle
    pub duree: Duration,
}

/// Les réglages de l'entraînement par renforcement
#[derive(Debug, Clone)]
pub struct ConfigRenforcement {
    /// Le nombre de cycles (collecte + mise à jour + éval périodique)
    pub cycles: usize,
    /// Les auto-parties par cycle
    pub parties_par_cycle: u32,
    /// La température d'exploration du décodeur échantillonné
    pub tau: f64,
    /// Le learning rate de REINFORCE (plus petit que l'imitation : la loss
    /// somme les 54 sorties au lieu d'en faire la moyenne)
    pub lr: f64,
    /// La taille des lots de mise à jour
    pub taille_lot: usize,
    /// La graine de tout ce qui est aléatoire (collecte, split, initialisation)
    pub graine: u64,
    /// La fréquence d'évaluation contre ISMCTS-300 (0 = jamais) : un tournoi
    /// lent, ne vaut la peine que de temps en temps
    pub evaluations_tous: usize,
    /// Les simulations de l'ISMCTS évaluateur
    pub simulations_ismcts: usize,
    /// Les graines du tournoi d'évaluation (plus nombreuses = moins de bruit
    /// dans la sélection du meilleur réseau)
    pub graines_eval: u32,
    /// La décroissance géométrique du lr par cycle (1.0 = constant) : tenir
    /// le pic de qualité au lieu de dériver ensuite
    pub decroissance_lr: f64,
}

/// Le résultat de l'entraînement : le meilleur réseau (taux d'évaluation
/// maximal observé) et l'historique (cycle, perte, taux d'évaluation)
pub struct ResultatRenforcement {
    pub meilleur: Politique,
    pub meilleur_taux: f64,
    pub meilleur_cycle: usize,
    pub historique: Vec<(usize, f32, f64)>,
}

/// Joue une auto-partie échantillonnée (le réseau contre lui-même).
/// Déterministe pour une graine donnée.
fn jouer_partie(
    reseau: &Politique,
    graine: u64,
    tau: f64,
    rng: &mut StdRng,
) -> Result<PartieAuto, String> {
    let mut partie =
        Partie::nouvelle_avec_graine(vec!["Reseau-1".to_string(), "Reseau-2".to_string()], graine)?;
    partie.joueurs[0].main.truncate(6);
    partie.joueurs[1].main.truncate(6);

    let mut decisions: Vec<Decision> = Vec::new();
    let mut tours = 0;
    while partie.gagnant().is_none() && tours < 300 {
        partie.ouvrir_tour();
        loop {
            let entree = encoder(&partie, partie.index_joueur_courant)?;
            let Some(coup) = crate::joueur::choisir_coup_echantillonne(reseau, &partie, rng, tau)?
            else {
                break; // partie gagnée en cours de tour
            };

            // L'étiquette du coup choisi, capturée avant qu'il ne modifie le
            // plateau — la même définition que le générateur (ADR 0007)
            let Some((types_jouees, piocher)) = cible_du_coup(&coup, &partie.plateau.ids_tuiles())
            else {
                // Défensif : Terminer n'est jamais échantillonné ; s'il est
                // renvoyé (aucun coup scorable), on clôt et le tour est fini
                jouer_coup(&mut partie, coup)?;
                break;
            };
            let mut cible = Vec::with_capacity(NB_SORTIES);
            cible.extend(
                types_jouees
                    .iter()
                    .map(|&vrai| if vrai { 1.0 } else { 0.0 }),
            );
            cible.push(if piocher { 1.0 } else { 0.0 });
            decisions.push(Decision {
                id_joueur: partie.index_joueur_courant,
                entree,
                cible,
                recompense: 0.0, // rempli à la fin de partie
            });

            jouer_coup(&mut partie, coup)?;
            if partie.gagnant().is_some() || !matches!(partie.tour, Tour::Ouvert { .. }) {
                break;
            }
        }
        tours += 1;
    }

    // Récompenses selon le score officiel du Rummikub (la même convention que
    // l'ISMCTS) : le gagnant marque 1 + la valeur de la main adverse (/100),
    // le perdant perd la valeur de sa propre main ; 0 pour tous si bloquée —
    // une marge de points, plus fine que ±1
    let gagnant = partie.gagnant();
    for decision in &mut decisions {
        decision.recompense = match gagnant {
            Some(id) => {
                let perdant = partie
                    .joueurs
                    .iter()
                    .find(|joueur| joueur.id != id)
                    .expect("la partie a deux joueurs");
                let valeur_perdante = perdant
                    .main
                    .iter()
                    .map(|t| t.valeur_points() as f64)
                    .sum::<f64>()
                    / 100.0;
                if decision.id_joueur == id {
                    (1.0 + valeur_perdante) as f32
                } else {
                    -valeur_perdante as f32
                }
            }
            None => 0.0,
        };
    }

    Ok(PartieAuto { decisions, gagnant })
}

/// La loss REINFORCE : moyenne des (avantage × −log π du coup), où −log π
/// est la somme des BCE des 54 sorties avec l'étiquette du coup. Les logits
/// sont en entrée ; la sigmoïde est appliquée via l'identité
/// softplus(x) = max(x,0) + log1p(exp(−|x|)), stable aux grands |x|.
fn perte_reinforce(logits: Tensor<2>, cibles: Tensor<2>, avantages: Tensor<2>) -> Tensor<1> {
    let absolu = logits.clone().abs();
    let queue = (-absolu.clone()).exp().log1p();
    let softplus_neg = (logits.clone().neg() + absolu.clone()) * 0.5 + queue.clone();
    let softplus_pos = (logits.clone() + absolu.clone()) * 0.5 + queue;
    let bce = cibles.clone() * softplus_neg + (cibles.clone().neg() + 1.0) * softplus_pos;
    (bce.sum_dim(1) * avantages).mean()
}

/// Un cycle : les auto-parties sont collectées (échantillonnées), la moyenne
/// des récompenses servira de baseline pour la mise à jour
fn collecter_cycle(
    reseau: &Politique,
    config: &ConfigRenforcement,
    index_cycle: usize,
) -> Result<(Vec<Decision>, Resume), String> {
    let mut rng = StdRng::seed_from_u64(config.graine + index_cycle as u64);
    let mut decisions = Vec::new();
    let mut resume = Resume {
        parties: 0,
        decisions: 0,
        victoires_joueur1: 0,
        victoires_joueur2: 0,
        non_terminees: 0,
        perte: 0.0,
        duree: Duration::ZERO,
    };
    for index in 0..config.parties_par_cycle {
        let partie = jouer_partie(reseau, index as u64 + 1, config.tau, &mut rng)?;
        match partie.gagnant {
            Some(0) => resume.victoires_joueur1 += 1,
            Some(_) => resume.victoires_joueur2 += 1,
            None => resume.non_terminees += 1,
        }
        resume.parties += 1;
        resume.decisions += partie.decisions.len();
        decisions.extend(partie.decisions);
    }
    Ok((decisions, resume))
}

/// Le lot d'entrées, forme [taille du lot, TAILLE_VECTEUR]
fn lot_entree(decisions: &[Decision], indices: &[usize], device: &Device) -> Tensor<2> {
    let mut valeurs = Vec::with_capacity(indices.len() * TAILLE_VECTEUR);
    for &index in indices {
        valeurs.extend_from_slice(&decisions[index].entree);
    }
    Tensor::<1>::from_floats(valeurs.as_slice(), device).reshape([indices.len(), TAILLE_VECTEUR])
}

/// Le lot de cibles (réelles, pas d'entiers : la BCE est écrite à la main)
fn lot_cibles(decisions: &[Decision], indices: &[usize], device: &Device) -> Tensor<2> {
    let mut valeurs = Vec::with_capacity(indices.len() * NB_SORTIES);
    for &index in indices {
        valeurs.extend_from_slice(&decisions[index].cible);
    }
    Tensor::<1>::from_floats(valeurs.as_slice(), device).reshape([indices.len(), NB_SORTIES])
}

/// Le lot d'avantages (récompense − baseline), forme [taille du lot, 1]
fn lot_avantages(
    decisions: &[Decision],
    indices: &[usize],
    baseline: f32,
    device: &Device,
) -> Tensor<2> {
    let valeurs: Vec<f32> = indices
        .iter()
        .map(|&index| decisions[index].recompense - baseline)
        .collect();
    Tensor::<1>::from_floats(valeurs.as_slice(), device).reshape([indices.len(), 1])
}

/// L'entraînement complet : part des poids chargés (imitation), enchaîne les
/// cycles, évalue périodiquement contre ISMCTS et garde le meilleur réseau.
pub fn entrainer_par_renforcement(
    reseau_initial: &Politique,
    config: &ConfigRenforcement,
) -> Result<ResultatRenforcement, String> {
    let device = Device::flex().autodiff();
    let record = reseau_initial.clone().into_record();
    let mut reseau = PolitiqueConfig::new(TAILLE_VECTEUR, NB_SORTIES)
        .init(&device)
        .load_record(record);
    let mut optimiseur = AdamConfig::new().init();

    let ismcts_eval = IsMcts {
        simulations: config.simulations_ismcts,
        ..Default::default()
    };

    let mut historique = Vec::new();
    let mut meilleur_taux = f64::NEG_INFINITY;
    let mut meilleur_record: Option<ModuleRecord> = None;
    let mut meilleur_cycle = 0usize;

    // Aucune évaluation configurée : le taux est indéfini (NaN), le réseau
    // final sera retenu
    let mut une_evaluation = false;

    for index_cycle in 0..config.cycles {
        let debut = Instant::now();
        let lr_cycle = config.lr * config.decroissance_lr.powi(index_cycle as i32);
        let (decisions, mut resume) = collecter_cycle(&reseau, config, index_cycle)?;

        // --- Mise à jour REINFORCE par lots : la moyenne des récompenses du
        // cycle sert de baseline
        let device = Device::flex().autodiff();
        let baseline = decisions.iter().map(|d| d.recompense).sum::<f32>() / decisions.len() as f32;
        let indices: Vec<usize> = (0..decisions.len()).collect();
        let mut perte_totale = 0.0f32;
        let mut nb_lots = 0usize;
        for lot in indices.chunks(config.taille_lot) {
            let entree = lot_entree(&decisions, lot, &device);
            let cibles = lot_cibles(&decisions, lot, &device);
            let avantages = lot_avantages(&decisions, lot, baseline, &device);
            let logits = reseau.forward(entree);
            let perte = perte_reinforce(logits, cibles, avantages);
            perte_totale += perte.clone().into_scalar::<f32>();
            nb_lots += 1;
            let grads = perte.backward();
            let grads = GradientsParams::from_grads(grads, &reseau);
            reseau = optimiseur.step(lr_cycle, reseau, grads);
        }
        resume.perte = if nb_lots == 0 {
            0.0
        } else {
            perte_totale / nb_lots as f32
        };
        resume.duree = debut.elapsed();

        println!(
            "cycle {:3} : {} parties, {} décisions, perte {:.3}, V1-V2 {}/{} ({} bloq.) en {:.1}s",
            index_cycle + 1,
            resume.parties,
            resume.decisions,
            resume.perte,
            resume.victoires_joueur1,
            resume.victoires_joueur2,
            resume.non_terminees,
            resume.duree.as_secs_f64(),
        );

        let taux = if config.evaluations_tous > 0
            && ((index_cycle + 1) % config.evaluations_tous == 0
                || index_cycle + 1 == config.cycles)
        {
            let resultat = crate::joueur::tournoi(
                config.graines_eval,
                &reseau.valid(),
                &crate::joueur::Adversaire::Isme(ismcts_eval.clone()),
            )?;
            println!(
                "          éval : {:.0}% contre ISMCTS-{} sur {} graines",
                resultat.taux_reseau() * 100.0,
                config.simulations_ismcts,
                config.graines_eval
            );
            Some(resultat.taux_reseau())
        } else {
            None
        };
        historique.push((index_cycle + 1, resume.perte, taux.unwrap_or(f64::NAN)));

        if let Some(taux) = taux {
            une_evaluation = true;
            if taux > meilleur_taux {
                meilleur_taux = taux;
                meilleur_cycle = index_cycle + 1;
                meilleur_record = Some(reseau.valid().into_record());
            }
        }
    }

    if !une_evaluation {
        meilleur_taux = f64::NAN;
    }

    let meilleur_record = match meilleur_record {
        Some(record) => record,
        // Aucune évaluation configurée : le réseau final est le résultat
        None => reseau.valid().into_record(),
    };
    let meilleur = PolitiqueConfig::new(TAILLE_VECTEUR, NB_SORTIES)
        .init(&Device::flex())
        .load_record(meilleur_record);

    Ok(ResultatRenforcement {
        meilleur,
        meilleur_taux,
        meilleur_cycle,
        historique,
    })
}

#[cfg(test)]
mod tests;
