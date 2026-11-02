use crate::combinaison::Combinaison;
use crate::joueur::Joueur;
use crate::plateau::Plateau;
use crate::tuile::{Tuile, generer_pioche, melanger_pioche, melanger_pioche_avec_graine};
use std::collections::HashSet;

/// État du tour en cours dans la partie
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Tour {
    /// Aucun tour ouvert : le joueur courant doit ouvrir son tour (ouvrir_tour)
    Repos,
    /// Tour ouvert : l'état du plateau et la main du joueur courant au début
    /// du tour sont conservés pour permettre une annulation (annuler_tour)
    Ouvert {
        plateau_debut: Plateau,
        main_debut: Vec<Tuile>,
    },
}

pub struct Partie {
    pub joueurs: Vec<Joueur>,
    pub index_joueur_courant: usize,
    pub plateau: Plateau,
    pub pioche: Vec<Tuile>, // Pioche
    /// État du tour en cours : un tour ouvert conserve le plateau et la main
    /// du début de tour, pour annuler une manipulation illégale (annuler_tour)
    pub tour: Tour,
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
    fn a_deja_joue_ce_tour(&self) -> bool {
        match &self.tour {
            Tour::Ouvert { plateau_debut, .. } => *plateau_debut != self.plateau,
            Tour::Repos => false,
        }
    }

    /// Ferme le tour (Tour::Repos) et passe au joueur suivant
    fn fin_de_tour(&mut self) {
        self.tour = Tour::Repos;
        self.index_joueur_courant = (self.index_joueur_courant + 1) % self.joueurs.len();
    }

    /// Identifiant du joueur gagnant (main vide), s'il y en a un.
    pub fn gagnant(&self) -> Option<usize> {
        self.joueurs
            .iter()
            .find(|joueur| joueur.main.is_empty())
            .map(|joueur| joueur.id)
    }
}

#[cfg(test)]
mod tests;
