//! ISMCTS : Information Set Monte Carlo Tree Search.
//!
//! Le MCTS classique construit un arbre d'états. Impossible ici : la main
//! des adversaires et le contenu de la pioche sont cachés, et un arbre bâti
//! sur l'état réel « verrait » ce que le joueur ne peut pas voir.
//!
//! L'ISMCTS résout ça en deux temps :
//! 1. **Déterminisation** — chaque simulation part d'un tirage cohérent des
//!    informations cachées : les tuiles inconnues (tout sauf notre main et
//!    le plateau) sont mélangées, chaque adversaire reçoit autant de tuiles
//!    que sa main en contient vraiment, le reste devient la pioche.
//! 2. **Arbre partagé** — les statistiques s'accumulent dans un arbre unique
//!    indexé par la séquence de coups (pas par l'état exact) : les nombreux
//!    tirages votent pour les coups bons « en moyenne », au lieu des coups
//!    bons contre une main exacte devinée.
//!
//! Mécanique d'une simulation : déterminiser → descendre l'arbre en choisissant
//! les enfants prometteurs (UCB1) → développer un nouveau coup → dérouler la
//! partie au hasard (rollout) → propager les récompenses dans les nœuds
//! traversés.
//!
//! Simplifications assumées de cette première version :
//! - une seule déterminisation par simulation, du point de vue du joueur qui
//!   décide ;
//! - rollouts purement aléatoires (une politique heuristique les améliorerait
//!   nettement) ;
//! - récompenses : 1 pour le gagnant, 0 pour les autres, 0 pour tous si la
//!   profondeur maximale de rollout est atteinte sans gagnant.

use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::{RngExt, SeedableRng};
use rummikub_core::coups::{Coup, coups_possibles, jouer_coup};
use rummikub_core::partie::{Partie, Tour};
use rummikub_core::tuile::{Tuile, generer_pioche};
use std::collections::HashSet;

/// Réglages de la recherche. La graine rend la décision reproductible :
/// même état + même graine → même coup choisi.
#[derive(Debug, Clone)]
pub struct IsMcts {
    /// Nombre de simulations : une déterminisation et une descente chacune
    pub simulations: usize,
    /// Profondeur maximale d'un rollout aléatoire
    pub profondeur_rollout: usize,
    /// Constante d'exploration UCB (sqrt(2) est le choix classique)
    pub exploration: f64,
    /// Graine du générateur aléatoire
    pub graine: u64,
}

impl Default for IsMcts {
    fn default() -> Self {
        Self {
            simulations: 300,
            profondeur_rollout: 30,
            exploration: std::f64::consts::SQRT_2,
            graine: 0,
        }
    }
}

impl IsMcts {
    /// Choisit le coup du joueur courant (tour ouvert requis) : l'enfant de la
    /// racine le plus visité après les simulations — le « coup robuste »,
    /// celui qui s'est révélé bon sur le plus de tirages différents.
    pub fn choisir_coup(&self, partie: &Partie) -> Result<Option<Coup>, String> {
        if !matches!(partie.tour, Tour::Ouvert { .. }) {
            return Err(
                "Aucun tour en cours : appelez ouvrir_tour pour ouvrir un tour.".to_string(),
            );
        }

        let mut rng = StdRng::seed_from_u64(self.graine);
        let mut racine = Noeud::default();

        for _ in 0..self.simulations {
            // 1. Déterminisation : une devinette cohérente des infos cachées
            let mut etat = determiniser(partie, &mut rng);
            // 2. Descente + expansion + rollout + propagation, sur cet état-là
            descente(&mut racine, &mut etat, &mut rng, self)?;
        }

        Ok(racine
            .enfants
            .iter()
            .max_by_key(|enfant| enfant.noeud.visites)
            .map(|enfant| enfant.coup.clone()))
    }

    /// Joue le tour complet du joueur courant : la recherche est relancée
    /// après chaque pose ou manipulation (un tour peut comporter plusieurs
    /// coups), et un coup Terminer ou Piocher le clôt.
    pub fn jouer_tour(&self, partie: &mut Partie) -> Result<(), String> {
        partie.ouvrir_tour();
        loop {
            let coup = self
                .choisir_coup(partie)?
                .expect("Piocher est toujours un coup disponible");
            jouer_coup(partie, coup)?;
            if !matches!(partie.tour, Tour::Ouvert { .. }) {
                return Ok(());
            }
        }
    }
}

#[derive(Default)]
struct Noeud {
    /// Nombre de passages dans ce nœud
    visites: u32,
    /// Somme des récompenses du joueur ayant joué le coup menant à ce nœud
    somme: f64,
    enfants: Vec<Enfant>,
}

struct Enfant {
    coup: Coup,
    noeud: Box<Noeud>,
}

impl Noeud {
    /// Le coup a-t-il déjà un enfant dans l'arbre ?
    fn possede(&self, coup: &Coup) -> bool {
        self.enfants.iter().any(|enfant| &enfant.coup == coup)
    }
}

/// Un tour vient-il de se terminer ? On ouvre alors celui du joueur suivant.
fn ouvrir_si_necessaire(etat: &mut Partie) {
    if !matches!(etat.tour, Tour::Ouvert { .. }) {
        etat.ouvrir_tour();
    }
}

/// Remplace les informations cachées par un tirage cohérent avec ce que le
/// joueur courant connaît : sa main et le plateau sont les siens ; les tuiles
/// inconnues sont mélangées puis distribuées — chaque adversaire reçoit
/// autant de tuiles que sa main en contient, le reste devient la pioche.
fn determiniser(partie: &Partie, rng: &mut StdRng) -> Partie {
    let mut etat = partie.clone();
    let id_vue = etat.index_joueur_courant;

    // Tuiles connues : notre main et le plateau. Le sac est toujours le même
    // et l'identifiant détermine le type d'une tuile : on repart donc du sac
    // complet et on retire les tuiles connues pour obtenir l'ensemble caché.
    let mut connues: HashSet<u8> = etat
        .joueurs
        .iter()
        .find(|joueur| joueur.id == id_vue)
        .expect("le joueur courant existe")
        .main
        .iter()
        .map(|tuile| tuile.id)
        .collect();
    connues.extend(etat.plateau.ids_tuiles());

    let mut cachees: Vec<Tuile> = generer_pioche()
        .into_iter()
        .filter(|tuile| !connues.contains(&tuile.id))
        .collect();
    cachees.shuffle(rng);

    // Distribution : les adversaires d'abord (leurs tailles de main sont
    // connues), le reste constitue la pioche devinée
    for joueur in etat.joueurs.iter_mut() {
        if joueur.id == id_vue {
            continue;
        }
        let taille = joueur.main.len();
        joueur.main = cachees.drain(..taille).collect();
    }
    etat.pioche = std::mem::take(&mut cachees);

    etat
}

/// Descend l'arbre depuis `noeud` (l'état `etat` correspond à ce nœud),
/// développe un coup, déroule la partie, et propage les récompenses en
/// remontant. Retourne les récompenses de la simulation, indexées par
/// identifiant joueur.
fn descente(
    noeud: &mut Noeud,
    etat: &mut Partie,
    rng: &mut StdRng,
    params: &IsMcts,
) -> Result<Vec<f64>, String> {
    noeud.visites += 1;

    // Partie finie : la récompense de chacun est fixée
    let mut recompenses = vec![0.0; etat.joueurs.len()];
    if let Some(id_gagnant) = etat.gagnant() {
        recompenses[id_gagnant] = 1.0;
        return Ok(recompenses);
    }

    // Un tour vient de se terminer : ouvrir celui du joueur suivant
    ouvrir_si_necessaire(etat);
    let coups = coups_possibles(etat)?;

    // Sélection : si tous les coups légaux ont déjà un enfant, on descend
    // par UCB1 parmi les enfants dont le coup est encore légal ici
    if coups.iter().all(|coup| noeud.possede(coup)) {
        let disponibles: Vec<usize> = (0..noeud.enfants.len())
            .filter(|index| coups.contains(&noeud.enfants[*index].coup))
            .collect();

        // Défensif : ne devrait jamais arriver (Piocher est toujours légal)
        if disponibles.is_empty() {
            return deroulement(etat, rng, params.profondeur_rollout);
        }

        let ln_parent = (noeud.visites as f64).ln();
        let valeur_de = |index: &usize| -> f64 {
            let enfant = &noeud.enfants[*index];
            if enfant.noeud.visites == 0 {
                return f64::INFINITY;
            }
            enfant.noeud.somme / enfant.noeud.visites as f64
                + params.exploration * (ln_parent / enfant.noeud.visites as f64).sqrt()
        };
        let meilleur = *disponibles
            .iter()
            .max_by(|a, b| valeur_de(a).partial_cmp(&valeur_de(b)).expect("pas de NaN"))
            .expect("disponibles n'est pas vide");

        let joueur = etat.index_joueur_courant;
        let coup = noeud.enfants[meilleur].coup.clone();
        jouer_coup(etat, coup)?;
        let recompenses = descente(&mut noeud.enfants[meilleur].noeud, etat, rng, params)?;
        noeud.enfants[meilleur].noeud.somme += recompenses[joueur];
        return Ok(recompenses);
    }

    // Expansion : développer un coup légal pas encore présent dans l'arbre
    let non_developpes: Vec<Coup> = coups
        .iter()
        .filter(|coup| !noeud.possede(coup))
        .cloned()
        .collect();
    let coup = non_developpes[rng.random_range(0..non_developpes.len())].clone();

    let joueur = etat.index_joueur_courant;
    jouer_coup(etat, coup.clone())?;
    noeud.enfants.push(Enfant {
        coup,
        noeud: Box::new(Noeud::default()),
    });

    // Simulation : dérouler la partie au hasard depuis le nouvel état
    let recompenses = deroulement(etat, rng, params.profondeur_rollout)?;

    let enfant = noeud
        .enfants
        .last_mut()
        .expect("l'enfant vient d'être ajouté");
    enfant.noeud.visites += 1;
    enfant.noeud.somme += recompenses[joueur];
    Ok(recompenses)
}

/// Déroule la partie au hasard jusqu'à la fin ou la profondeur maximale :
/// chaque coup est pris uniformément parmi les coups légaux.
fn deroulement(
    etat: &mut Partie,
    rng: &mut StdRng,
    profondeur_max: usize,
) -> Result<Vec<f64>, String> {
    for _ in 0..profondeur_max {
        if etat.gagnant().is_some() {
            break;
        }
        ouvrir_si_necessaire(etat);
        let coups = coups_possibles(etat)?;
        let coup = coups[rng.random_range(0..coups.len())].clone();
        jouer_coup(etat, coup)?;
    }

    let mut recompenses = vec![0.0; etat.joueurs.len()];
    if let Some(id_gagnant) = etat.gagnant() {
        recompenses[id_gagnant] = 1.0;
    }
    Ok(recompenses)
}

#[cfg(test)]
mod tests;
