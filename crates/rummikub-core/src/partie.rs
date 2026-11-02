use crate::combinaison::Combinaison;
use crate::joueur::Joueur;
use crate::plateau::Plateau;
use crate::tuile::{Tuile, generer_pioche, melanger_pioche, melanger_pioche_avec_graine};
use std::collections::HashSet;

/// État du tour en cours dans la partie
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Tour {
    /// Aucun tour ouvert : le joueur courant doit ouvrir son tour (start_turn)
    Repos,
    /// Tour ouvert : l'état du plateau et la main du joueur courant au début
    /// du tour sont conservés pour permettre une annulation (cancel_turn)
    Ouvert {
        plateau_debut: Plateau,
        main_debut: Vec<Tuile>,
    },
}

#[derive(Debug, Clone)]
pub struct Partie {
    pub joueurs: Vec<Joueur>,
    pub index_joueur_courant: usize,
    pub plateau: Plateau,
    pub pioche: Vec<Tuile>, // Pioche
    /// État du tour en cours : un tour ouvert conserve le plateau et la main
    /// du début de tour, pour annuler une manipulation illégale (annuler_tour)
    pub tour: Tour,
    /// Nombre de tours consécutifs sans la moindre pose — sert à détecter
    /// la fin de partie par blocage (sac vide)
    pub tours_sans_pose: usize,
}

/// Vue d'une partie telle qu'un joueur donné est autorisé à la voir : le
/// plateau, les comptes et l'état du tour sont publics, seule la main du
/// joueur concerné est visible en détail.
#[derive(Debug, Clone)]
pub struct VuePartie {
    /// Le joueur pour qui la vue est construite
    pub id_joueur: usize,
    pub index_joueur_courant: usize,
    pub plateau: Plateau,
    /// Le contenu de la pioche reste secret, seul son nombre est public
    pub taille_pioche: usize,
    pub gagnant: Option<usize>,
    pub joueurs: Vec<VueJoueur>,
}

/// Un joueur vu depuis une partie : le contenu de sa main n'apparaît que
/// dans la vue du joueur lui-même
#[derive(Debug, Clone)]
pub struct VueJoueur {
    pub id: usize,
    pub nom: String,
    pub a_fait_pose_initiale: bool,
    pub taille_main: usize,
    /// Présente uniquement dans la vue du joueur lui-même
    pub main: Option<Vec<Tuile>>,
}

impl Partie {
    /// Nombre de tuiles distribuées à chaque joueur en début de partie
    pub const TUILES_PAR_JOUEUR: usize = 14;

    /// Prépare une partie : sac mélangé aléatoirement puis 14 tuiles
    /// distribuées à chaque joueur. Le Rummikub se joue de 2 à 4 joueurs.
    /// Pour un mélange déterministe (rejeu à l'identique), voir
    /// nouvelle_avec_graine.
    pub fn nouvelle(noms: Vec<String>) -> Result<Self, String> {
        Self::nouvelle_avec_melange(noms, melanger_pioche)
    }

    /// Comme nouvelle, mais le mélange est déterministe : la même graine donne
    /// toujours la même distribution. Utile pour rejouer une partie à
    /// l'identique (IA, tests, réseau).
    pub fn nouvelle_avec_graine(noms: Vec<String>, graine: u64) -> Result<Self, String> {
        Self::nouvelle_avec_melange(noms, |pioche| melanger_pioche_avec_graine(pioche, graine))
    }

    fn nouvelle_avec_melange(
        noms: Vec<String>,
        melanger: impl FnOnce(&mut [Tuile]),
    ) -> Result<Self, String> {
        if !(2..=4).contains(&noms.len()) {
            return Err(format!(
                "Le jeu se joue de 2 à 4 joueurs (reçu : {}).",
                noms.len()
            ));
        }

        let mut pioche = generer_pioche();
        melanger(&mut pioche);

        let mut joueurs = Vec::with_capacity(noms.len());
        for (id, nom) in noms.into_iter().enumerate() {
            let main: Vec<Tuile> = pioche.drain(..Self::TUILES_PAR_JOUEUR).collect();
            joueurs.push(Joueur::nouveau(id, nom, main));
        }

        Ok(Self {
            joueurs,
            index_joueur_courant: 0,
            plateau: Plateau::default(),
            pioche,
            tour: Tour::Repos,
            tours_sans_pose: 0,
        })
    }

    pub fn joueur_courant(&self) -> &Joueur {
        &self.joueurs[self.index_joueur_courant]
    }

    pub fn joueur_courant_mut(&mut self) -> &mut Joueur {
        &mut self.joueurs[self.index_joueur_courant]
    }

    /// Le joueur courant ne pose rien : il pioche une tuile et son tour se
    /// termine immédiatement (règle officielle). Si le sac est vide, il passe
    /// simplement son tour et la méthode retourne None.
    ///
    /// Erreur si aucun tour n'est ouvert (ouvrir_tour) ou si le joueur a déjà
    /// posé ou manipulé pendant ce tour.
    pub fn piocher(&mut self) -> Result<Option<Tuile>, String> {
        if !self.tour_ouvert() {
            return Err(
                "Aucun tour en cours : appelez ouvrir_tour pour ouvrir un tour.".to_string(),
            );
        }
        if self.a_deja_joue_ce_tour() {
            return Err(
                "Le joueur a déjà joué pendant ce tour : impossible de piocher.".to_string(),
            );
        }

        let tuile = self.pioche.pop();
        if let Some(tuile) = tuile {
            self.joueur_courant_mut().main.push(tuile);
        }
        // Piocher (ou passer à sac vide) met fin au tour
        self.fin_de_tour();
        Ok(tuile)
    }

    /// Ouvre le tour du joueur courant : le plateau et sa main sont sauvegardés
    /// pour permettre d'annuler le tour (annuler_tour).
    pub fn ouvrir_tour(&mut self) {
        self.tour = Tour::Ouvert {
            plateau_debut: self.plateau.clone(),
            main_debut: self.joueur_courant().main.clone(),
        };
    }

    /// Annule le tour en cours : restaure le plateau et la main du début de
    /// tour. Sans tour ouvert, l'appel ne fait rien.
    pub fn annuler_tour(&mut self) {
        if let Tour::Ouvert {
            plateau_debut,
            main_debut,
        } = std::mem::replace(&mut self.tour, Tour::Repos)
        {
            self.plateau = plateau_debut;
            self.joueur_courant_mut().main = main_debut;
        }
    }

    /// Le joueur courant pose des combinaisons construites uniquement à partir de
    /// sa main. Avant la pose initiale, l'exigence des 30 points s'applique
    /// automatiquement.
    pub fn poser_combinaisons(&mut self, proposed_melds: Vec<Combinaison>) -> Result<u32, String> {
        if !self.tour_ouvert() {
            return Err(
                "Aucun tour en cours : appelez ouvrir_tour pour ouvrir un tour.".to_string(),
            );
        }

        let joueur = self.joueur_courant_mut();
        let points = if joueur.a_fait_pose_initiale {
            joueur.poser_combinaisons(&proposed_melds)?
        } else {
            joueur.jouer_pose_initiale(&proposed_melds)?
        };

        for combinaison in proposed_melds {
            self.plateau.placer_combinaison(combinaison);
        }
        Ok(points)
    }

    /// Le joueur courant soumet le nouvel état complet du plateau (manipulation
    /// des combinaisons déjà posées et/ou ajout depuis la main).
    ///
    /// Règles vérifiées :
    /// - interdit avant la pose initiale du joueur ;
    /// - chaque combinaison du nouvel état est valide ;
    /// - aucune tuile ne disparaît du plateau ;
    /// - chaque nouvelle tuile provient de la main du joueur ;
    /// - aucune tuile n'est utilisée deux fois.
    pub fn soumettre_plateau(&mut self, combinaisons: Vec<Combinaison>) -> Result<(), String> {
        if !self.tour_ouvert() {
            return Err(
                "Aucun tour en cours : appelez ouvrir_tour pour ouvrir un tour.".to_string(),
            );
        }
        if !self.joueur_courant().a_fait_pose_initiale {
            return Err("Impossible de manipuler le plateau avant la pose initiale.".to_string());
        }

        for (i, combinaison) in combinaisons.iter().enumerate() {
            if !combinaison.est_valide() {
                return Err(format!("La combinaison n°{} est invalide.", i + 1));
            }
        }

        let mut ids_plateau = self.plateau.ids_tuiles();
        let mut ids_main: HashSet<u8> = self
            .joueur_courant()
            .main
            .iter()
            .map(|tuile| tuile.id)
            .collect();
        let mut ids_deja_vus: HashSet<u8> = HashSet::new();

        for combinaison in &combinaisons {
            for tuile in &combinaison.Tuiles {
                if !ids_deja_vus.insert(tuile.id) {
                    return Err(format!(
                        "La tuile ID {} apparaît deux fois sur le plateau.",
                        tuile.id
                    ));
                }
                // La tuile était-elle déjà posée sur le plateau ?
                if ids_plateau.remove(&tuile.id) {
                    continue;
                }
                // Sinon, elle doit venir de la main du joueur
                if !ids_main.remove(&tuile.id) {
                    return Err(format!(
                        "La tuile ID {} ne provient ni du plateau ni de la main du joueur.",
                        tuile.id
                    ));
                }
            }
        }

        if !ids_plateau.is_empty() {
            return Err("Des tuiles du plateau ont disparu : manipulation interdite.".to_string());
        }

        // Application : nouveau plateau, puis retrait de la main des tuiles
        // nouvellement posées (ids_main ne contient plus que les tuiles non utilisées)
        self.plateau.remplacer_combinaisons(combinaisons);
        self.joueur_courant_mut()
            .main
            .retain(|tuile| ids_main.contains(&tuile.id));
        Ok(())
    }

    /// Clôture le tour du joueur courant et passe au suivant.
    ///
    /// Erreurs :
    /// - aucun tour ouvert (ouvrir_tour) ;
    /// - plateau invalide : le tour est alors annulé (annuler_tour) ;
    /// - le joueur n'a rien joué alors que le sac contient encore des tuiles :
    ///   il doit jouer ou piocher, la passe n'étant permise qu'à sac vide.
    pub fn terminer_tour(&mut self) -> Result<(), String> {
        if !self.tour_ouvert() {
            return Err(
                "Aucun tour en cours : appelez ouvrir_tour pour ouvrir un tour.".to_string(),
            );
        }
        if !self.plateau.est_valide() {
            self.annuler_tour();
            return Err("Plateau invalide en fin de tour : tour annulé.".to_string());
        }
        if !self.a_deja_joue_ce_tour() && !self.pioche.is_empty() {
            return Err("Le joueur doit poser des tuiles ou piocher : tour refusé.".to_string());
        }

        self.fin_de_tour();
        Ok(())
    }

    /// Un tour est-il actuellement ouvert ?
    fn tour_ouvert(&self) -> bool {
        matches!(self.tour, Tour::Ouvert { .. })
    }

    /// Le joueur a-t-il déjà posé ou manipulé pendant le tour en cours ?
    /// (public : les IA en ont besoin pour savoir si terminer le tour est
    /// une option légale)
    pub fn a_deja_joue_ce_tour(&self) -> bool {
        match &self.tour {
            Tour::Ouvert { plateau_debut, .. } => *plateau_debut != self.plateau,
            Tour::Repos => false,
        }
    }

    /// Ferme le tour (Tour::Repos) et passe au joueur suivant
    fn fin_de_tour(&mut self) {
        // Comptabilise les tours sans pose : la détection du blocage (sac
        // vide) en dépend
        if !self.a_deja_joue_ce_tour() {
            self.tours_sans_pose += 1;
        } else {
            self.tours_sans_pose = 0;
        }

        self.tour = Tour::Repos;
        self.index_joueur_courant = (self.index_joueur_courant + 1) % self.joueurs.len();
    }

    /// Identifiant du joueur gagnant, s'il y en a un :
    /// - main vide : victoire immédiate ;
    /// - partie bloquée (sac vide et un tour complet sans pose) : la valeur
    ///   de main la plus faible l'emporte, égalité = partie nulle ;
    /// - sinon : la partie continue.
    pub fn gagnant(&self) -> Option<usize> {
        if let Some(joueur) = self.joueurs.iter().find(|joueur| joueur.main.is_empty()) {
            return Some(joueur.id);
        }
        if !self.est_terminee() {
            return None;
        }

        let valeurs: Vec<(usize, u32)> = self
            .joueurs
            .iter()
            .map(|joueur| {
                (
                    joueur.id,
                    joueur
                        .main
                        .iter()
                        .map(|t| t.valeur_points() as u32)
                        .sum::<u32>(),
                )
            })
            .collect();
        let plus_faible = valeurs.iter().map(|(_, valeur)| *valeur).min()?;
        let gagnants: Vec<usize> = valeurs
            .iter()
            .filter(|(_, valeur)| *valeur == plus_faible)
            .map(|(id, _)| *id)
            .collect();
        if gagnants.len() == 1 {
            gagnants.into_iter().next()
        } else {
            None
        }
    }

    /// La partie est-elle bloquée : sac vide et un tour complet sans la
    /// moindre pose ? (règle officielle de fin de partie)
    pub fn est_terminee(&self) -> bool {
        self.pioche.is_empty() && self.tours_sans_pose >= self.joueurs.len()
    }

    /// Construit la vue de la partie pour le joueur identifié par `id_joueur`.
    /// Fonction pure : la partie n'est pas modifiée, et les mains des autres
    /// joueurs sont remplacées par leur seule taille.
    pub fn vue_pour(&self, id_joueur: usize) -> Result<VuePartie, String> {
        let id_vue = self
            .joueurs
            .iter()
            .find(|joueur| joueur.id == id_joueur)
            .ok_or_else(|| format!("Aucun joueur avec l'identifiant {}.", id_joueur))?
            .id;

        let joueurs = self
            .joueurs
            .iter()
            .map(|joueur| VueJoueur {
                id: joueur.id,
                nom: joueur.nom.clone(),
                a_fait_pose_initiale: joueur.a_fait_pose_initiale,
                taille_main: joueur.main.len(),
                main: (joueur.id == id_vue).then(|| joueur.main.clone()),
            })
            .collect();

        Ok(VuePartie {
            id_joueur: id_vue,
            index_joueur_courant: self.index_joueur_courant,
            plateau: self.plateau.clone(),
            taille_pioche: self.pioche.len(),
            gagnant: self.gagnant(),
            joueurs,
        })
    }
}

#[cfg(test)]
mod tests;
