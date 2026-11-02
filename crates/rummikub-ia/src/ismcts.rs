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
//! Simplifications assumées de cette version (détails dans l'ADR 0003) :
//! - une seule déterminisation par simulation, du point de vue du joueur qui
//!   décide ;
//! - rollouts ε-greedy : le coup heuristique avec probabilité 1−ε, un coup
//!   au hasard sinon ;
//! - récompenses selon le score officiel en fin de partie (victoire + valeur
//!   des mains adverses), 0 pour tous si la profondeur maximale est atteinte
//!   sans gagnant ;
//! - parallélisation à la racine : un arbre par thread, fusionnés à la fin.

use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::{RngExt, SeedableRng};
use rayon::prelude::*;
use rummikub_core::coups::{Coup, coups_possibles, jouer_coup};
use rummikub_core::partie::{Partie, Tour};
use rummikub_core::tuile::{BitsetTuiles, Tuile, generer_pioche};

/// En dessous de ce nombre de simulations par thread, la coordination de
/// rayon finirait par coûter plus que le travail (ADR 0005). Valeur basse :
/// la mesure a montré que la parallélisation reste rentable même à 6
/// simulations par thread (50 simulations : 1,3 s parallèle contre 6,6 s
/// séquentiel).
const SIMULATIONS_MIN_PAR_THREAD: usize = 4;

/// Réglages de la recherche. La décision est reproductible : même état +
/// même graine + même nombre de threads → même coup choisi.
#[derive(Debug, Clone)]
pub struct IsMcts {
    /// Nombre de simulations : une déterminisation et une descente chacune
    pub simulations: usize,
    /// Profondeur maximale d'un rollout
    pub profondeur_rollout: usize,
    /// Constante d'exploration UCB (sqrt(2) est le choix classique)
    pub exploration: f64,
    /// Pendant un rollout : probabilité de jouer un coup au hasard (le coup
    /// heuristique est joué avec probabilité 1−ε). 1.0 = rollouts aléatoires,
    /// 0.0 = rollouts entièrement heuristiques
    pub epsilon_rollout: f64,
    /// Élargissement progressif : α dans la formule k = α·N^β qui plafonne le
    /// nombre d'enfants d'un nœud visité N fois (1.0 = classique)
    pub alpha_elargissement: f64,
    /// Élargissement progressif : β dans k = α·N^β (0.5 = racine carrée ;
    /// 1.0 annule l'effet de dosage progressif)
    pub beta_elargissement: f64,
    /// Nombre de threads pour la parallélisation à la racine ; 0 = auto
    pub threads: usize,
    /// Graine du générateur aléatoire
    pub graine: u64,
}

impl Default for IsMcts {
    fn default() -> Self {
        Self {
            simulations: 300,
            profondeur_rollout: 30,
            exploration: std::f64::consts::SQRT_2,
            epsilon_rollout: 0.2,
            alpha_elargissement: 1.0,
            beta_elargissement: 0.5,
            threads: 0,
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

        // Parallélisation à la racine : chaque thread construit son propre
        // arbre avec sa graine dérivée, puis les statistiques sont fusionnées
        let threads = if self.threads == 0 {
            // Ne paralléliser que si chaque thread aurait assez à faire
            let coeurs = std::thread::available_parallelism()
                .map(|n| n.get())
                .unwrap_or(1);
            (coeurs.min(self.simulations / SIMULATIONS_MIN_PAR_THREAD)).max(1)
        } else {
            self.threads
        };
        let par_thread = self.simulations / threads;
        let reste = self.simulations % threads;

        let branches: Vec<Noeud> = (0..threads)
            .into_par_iter()
            .map(|index| {
                let simulations = par_thread + usize::from(index < reste);
                let mut rng = StdRng::seed_from_u64(self.graine.wrapping_add(index as u64));
                let mut racine = Noeud::default();
                for _ in 0..simulations {
                    // 1. Déterminisation : une devinette cohérente des infos cachées
                    let mut etat = determiniser(partie, &mut rng);
                    // 2. Descente + expansion + rollout + propagation, sur cet état-là
                    descente(&mut racine, &mut etat, &mut rng, self)?;
                }
                Ok(racine)
            })
            .collect::<Result<Vec<Noeud>, String>>()?;

        let mut racine = Noeud::default();
        for branche in branches {
            racine.fusionner(branche);
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

    /// Fusionne les statistiques d'un autre arbre (parallélisation à la racine)
    fn fusionner(&mut self, autre: Noeud) {
        self.visites += autre.visites;
        self.somme += autre.somme;
        for enfant in autre.enfants {
            match self.enfants.iter_mut().find(|e| e.coup == enfant.coup) {
                Some(existant) => existant.noeud.fusionner(*enfant.noeud),
                None => self.enfants.push(enfant),
            }
        }
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
    let mut connues = BitsetTuiles::depuis_ids(
        etat.joueurs
            .iter()
            .find(|joueur| joueur.id == id_vue)
            .expect("le joueur courant existe")
            .main
            .iter()
            .map(|tuile| tuile.id),
    );
    connues.inserer_tout(&etat.plateau.ids_tuiles());

    let mut cachees: Vec<Tuile> = generer_pioche()
        .into_iter()
        .filter(|tuile| !connues.contient(tuile.id))
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

    // Partie finie : récompenses selon le score officiel
    if etat.gagnant().is_some() {
        return Ok(recompenses_finales(etat));
    }

    // Un tour vient de se terminer : ouvrir celui du joueur suivant
    ouvrir_si_necessaire(etat);
    let coups = coups_possibles(etat)?;

    // Élargissement progressif (ADR 0006) : le nœud visité N fois n'admet
    // que k = α·N^β enfants — peu au début (les premiers du tri par points),
    // davantage si les visites s'accumulent.
    let plafond = plafond_enfants(noeud.visites, params);
    let non_developpes: Vec<&Coup> = coups.iter().filter(|coup| !noeud.possede(coup)).collect();

    // Sélection : tout est déjà développé, ou le plafond d'enfants est
    // atteint — on descend par UCB1 parmi les enfants dont le coup est
    // encore légal ici
    if non_developpes.is_empty() || noeud.enfants.len() >= plafond {
        let disponibles: Vec<usize> = (0..noeud.enfants.len())
            .filter(|index| coups.contains(&noeud.enfants[*index].coup))
            .collect();

        // Défensif : ne devrait jamais arriver (Piocher est toujours légal)
        if disponibles.is_empty() {
            return deroulement(etat, rng, params);
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

    // Expansion : développer le premier coup légal pas encore dans l'arbre.
    // La liste est triée par points décroissants : ce coup est donc le
    // meilleur candidat restant, et le choix est déterministe.
    let coup = (*non_developpes[0]).clone();

    let joueur = etat.index_joueur_courant;
    jouer_coup(etat, coup.clone())?;
    noeud.enfants.push(Enfant {
        coup,
        noeud: Box::new(Noeud::default()),
    });

    // Simulation : dérouler la partie depuis le nouvel état
    let recompenses = deroulement(etat, rng, params)?;

    let enfant = noeud
        .enfants
        .last_mut()
        .expect("l'enfant vient d'être ajouté");
    enfant.noeud.visites += 1;
    enfant.noeud.somme += recompenses[joueur];
    Ok(recompenses)
}

/// Déroule la partie jusqu'à la fin ou la profondeur maximale, avec une
/// politique ε-greedy : le coup heuristique avec probabilité 1−ε, un coup
/// au hasard sinon.
fn deroulement(etat: &mut Partie, rng: &mut StdRng, params: &IsMcts) -> Result<Vec<f64>, String> {
    for _ in 0..params.profondeur_rollout {
        if etat.gagnant().is_some() {
            break;
        }
        ouvrir_si_necessaire(etat);
        let mut coups = coups_possibles(etat)?;

        let coup = if rng.random::<f64>() < params.epsilon_rollout {
            // swap_remove : la liste est jetable, on consomme le coup au lieu
            // de le cloner
            coups.swap_remove(rng.random_range(0..coups.len()))
        } else {
            coup_heuristique(&coups)
        };
        jouer_coup(etat, coup)?;
    }

    Ok(recompenses_finales(etat))
}

/// Nombre maximal d'enfants d'un nœud visité N fois : α × N^β (élargissement
/// progressif). Au moins 1, pour que la première visite développe le meilleur
/// coup.
fn plafond_enfants(visites: u32, params: &IsMcts) -> usize {
    (params.alpha_elargissement * (visites as f64).powf(params.beta_elargissement))
        .floor()
        .max(1.0) as usize
}

/// Le coup qu'un bot heuristique jouerait parmi les coups légaux : la
/// meilleure pose (ils sont triés par points décroissants), sinon piocher,
/// sinon — si piocher est interdit — terminer.
fn coup_heuristique(coups: &[Coup]) -> Coup {
    coups
        .iter()
        .find(|coup| matches!(coup, Coup::Poser(_)))
        .or_else(|| coups.iter().find(|coup| matches!(coup, Coup::Piocher)))
        .unwrap_or(&coups[0])
        .clone()
}

/// Récompenses de fin de partie, selon le score officiel du Rummikub : le
/// gagnant marque 1 (la victoire) plus la valeur des mains adverses ; chaque
/// perdant perd la valeur de sa propre main. Le joker resté en main vaut 30
/// points. Le tout normalisé (/100) pour rester comparable à la prime de
/// victoire. Sans gagnant (profondeur max atteinte) : 0 pour tous.
fn recompenses_finales(etat: &Partie) -> Vec<f64> {
    let mut recompenses = vec![0.0; etat.joueurs.len()];
    let Some(id_gagnant) = etat.gagnant() else {
        return recompenses;
    };

    recompenses[id_gagnant] = 1.0;
    for joueur in &etat.joueurs {
        if joueur.id == id_gagnant {
            continue;
        }
        let valeur_main: u32 = joueur.main.iter().map(|t| t.valeur_points() as u32).sum();
        let normalisee = valeur_main as f64 / 100.0;
        recompenses[id_gagnant] += normalisee;
        recompenses[joueur.id] -= normalisee;
    }

    recompenses
}

#[cfg(test)]
mod tests;
