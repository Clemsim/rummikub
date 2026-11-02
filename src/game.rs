use crate::board::Board;
use crate::meld::Meld;
use crate::player::Player;
use crate::tuile::{Tuile, generate_pool, shuffle_pool};
use std::collections::HashSet;

pub struct Game {
    pub players: Vec<Player>,
    pub current_player: usize,
    pub board: Board,
    pub pool: Vec<Tuile>, // Pioche
    // Utile pour annuler le tour si le joueur fait une manipulation illégale :
    pub board_backup: Option<Board>,
    pub hand_backup: Option<Vec<Tuile>>,
}

impl Game {
    /// Nombre de tuiles distribuées à chaque joueur en début de partie
    pub const TUILES_PAR_JOUEUR: usize = 14;

    /// Prépare une partie : sac mélangé puis 14 tuiles distribuées à chaque joueur.
    /// Le Rummikub se joue de 2 à 4 joueurs.
    pub fn new(noms: Vec<String>) -> Result<Self, String> {
        if !(2..=4).contains(&noms.len()) {
            return Err(format!(
                "Le jeu se joue de 2 à 4 joueurs (reçu : {}).",
                noms.len()
            ));
        }

        let mut pool = generate_pool();
        shuffle_pool(&mut pool);

        let mut players = Vec::with_capacity(noms.len());
        for (id, nom) in noms.into_iter().enumerate() {
            let main: Vec<Tuile> = pool.drain(..Self::TUILES_PAR_JOUEUR).collect();
            players.push(Player::new(id, nom, main));
        }

        Ok(Self {
            players,
            current_player: 0,
            board: Board::default(),
            pool,
            board_backup: None,
            hand_backup: None,
        })
    }

    pub fn current_player(&self) -> &Player {
        &self.players[self.current_player]
    }

    pub fn current_player_mut(&mut self) -> &mut Player {
        &mut self.players[self.current_player]
    }

    /// Le joueur courant ne pose rien : il pioche une tuile et son tour se
    /// termine immédiatement (règle officielle). Si le sac est vide, il passe
    /// simplement son tour et la méthode retourne None.
    ///
    /// Erreur si aucun tour n'est ouvert (start_turn) ou si le joueur a déjà
    /// posé ou manipulé pendant ce tour.
    pub fn draw_tile(&mut self) -> Result<Option<Tuile>, String> {
        if self.board_backup.is_none() {
            return Err(
                "Aucun tour en cours : appelez start_turn pour ouvrir un tour.".to_string(),
            );
        }
        if self.a_deja_joue_ce_tour() {
            return Err(
                "Le joueur a déjà joué pendant ce tour : impossible de piocher.".to_string(),
            );
        }

        let tuile = self.pool.pop();
        if let Some(tuile) = tuile {
            self.current_player_mut().hand.push(tuile);
        }
        // Piocher (ou passer à sac vide) met fin au tour
        self.fin_de_tour();
        Ok(tuile)
    }

    /// Ouvre le tour du joueur courant : le plateau et sa main sont sauvegardés
    /// pour permettre d'annuler le tour (cancel_turn).
    pub fn start_turn(&mut self) {
        self.board_backup = Some(self.board.clone());
        self.hand_backup = Some(self.current_player().hand.clone());
    }

    /// Annule le tour en cours : restaure le plateau et la main du début de tour.
    pub fn cancel_turn(&mut self) {
        if let Some(board) = self.board_backup.take() {
            self.board = board;
        }
        if let Some(main) = self.hand_backup.take() {
            self.current_player_mut().hand = main;
        }
    }

    /// Le joueur courant pose des combinaisons construites uniquement à partir de
    /// sa main. Avant la pose initiale, l'exigence des 30 points s'applique
    /// automatiquement.
    pub fn play_melds(&mut self, proposed_melds: Vec<Meld>) -> Result<u32, String> {
        if self.board_backup.is_none() {
            return Err(
                "Aucun tour en cours : appelez start_turn pour ouvrir un tour.".to_string(),
            );
        }

        let joueur = self.current_player_mut();
        let points = if joueur.has_completed_initial_meld {
            joueur.play_melds(&proposed_melds)?
        } else {
            joueur.play_initial_meld(&proposed_melds)?
        };

        for meld in proposed_melds {
            self.board.place_meld(meld);
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
    pub fn submit_board(&mut self, melds: Vec<Meld>) -> Result<(), String> {
        if self.board_backup.is_none() {
            return Err(
                "Aucun tour en cours : appelez start_turn pour ouvrir un tour.".to_string(),
            );
        }
        if !self.current_player().has_completed_initial_meld {
            return Err("Impossible de manipuler le plateau avant la pose initiale.".to_string());
        }

        for (i, meld) in melds.iter().enumerate() {
            if !meld.is_valid() {
                return Err(format!("La combinaison n°{} est invalide.", i + 1));
            }
        }

        let mut ids_plateau = self.board.tile_ids();
        let mut ids_main: HashSet<u8> = self
            .current_player()
            .hand
            .iter()
            .map(|tuile| tuile.id)
            .collect();
        let mut ids_deja_vus: HashSet<u8> = HashSet::new();

        for meld in &melds {
            for tuile in &meld.Tuiles {
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
        self.board.set_melds(melds);
        self.current_player_mut()
            .hand
            .retain(|tuile| ids_main.contains(&tuile.id));
        Ok(())
    }

    /// Clôture le tour du joueur courant et passe au suivant.
    ///
    /// Erreurs :
    /// - aucun tour ouvert (start_turn) ;
    /// - plateau invalide : le tour est alors annulé (cancel_turn) ;
    /// - le joueur n'a rien joué alors que le sac contient encore des tuiles :
    ///   il doit jouer ou piocher, la passe n'étant permise qu'à sac vide.
    pub fn end_turn(&mut self) -> Result<(), String> {
        if self.board_backup.is_none() {
            return Err(
                "Aucun tour en cours : appelez start_turn pour ouvrir un tour.".to_string(),
            );
        }
        if !self.board.is_valid() {
            self.cancel_turn();
            return Err("Plateau invalide en fin de tour : tour annulé.".to_string());
        }
        if !self.a_deja_joue_ce_tour() && !self.pool.is_empty() {
            return Err("Le joueur doit poser des tuiles ou piocher : tour refusé.".to_string());
        }

        self.fin_de_tour();
        Ok(())
    }

    /// Le joueur a-t-il déjà posé ou manipulé pendant le tour en cours ?
    fn a_deja_joue_ce_tour(&self) -> bool {
        match &self.board_backup {
            Some(debut_de_tour) => *debut_de_tour != self.board,
            None => false,
        }
    }

    /// Nettoie les sauvegardes et passe au joueur suivant
    fn fin_de_tour(&mut self) {
        self.board_backup = None;
        self.hand_backup = None;
        self.current_player = (self.current_player + 1) % self.players.len();
    }

    /// Identifiant du joueur gagnant (main vide), s'il y en a un.
    pub fn winner(&self) -> Option<usize> {
        self.players
            .iter()
            .find(|joueur| joueur.hand.is_empty())
            .map(|joueur| joueur.id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tuile::Color;

    fn standard(id: u8, color: Color, value: u8) -> Tuile {
        Tuile::new_standard(id, color, value)
    }

    fn game_deux_joueurs() -> Game {
        Game::new(vec!["Alice".to_string(), "Bob".to_string()]).unwrap()
    }

    /// Tour minimal légal : le joueur pioche, ce qui termine son tour
    fn tour_pioche(game: &mut Game) {
        game.start_turn();
        assert!(game.draw_tile().unwrap().is_some());
    }

    #[test]
    fn test_new_game_deals_14_tiles_each() {
        let game = game_deux_joueurs();
        assert_eq!(game.players.len(), 2);
        assert!(game.players.iter().all(|joueur| joueur.hand.len() == 14));
        assert_eq!(game.pool.len(), 106 - 2 * 14);
        assert!(game.board.melds.is_empty());
        assert_eq!(game.current_player, 0);
    }

    #[test]
    fn test_new_game_rejects_wrong_player_count() {
        assert!(Game::new(vec!["Alice".to_string()]).is_err());
        assert!(
            Game::new(vec![
                "A".to_string(),
                "B".to_string(),
                "C".to_string(),
                "D".to_string(),
                "E".to_string()
            ])
            .is_err()
        );
    }

    #[test]
    fn test_play_melds_initial_meld_below_30_rejected() {
        let mut game = game_deux_joueurs();
        game.players[0].hand = vec![
            standard(1, Color::Red, 7),
            standard(2, Color::Blue, 7),
            standard(3, Color::Yellow, 7),
        ]; // 21 points seulement

        game.start_turn();
        let meld = Meld::new(game.players[0].hand.clone());
        assert!(game.play_melds(vec![meld]).is_err());

        // Rien n'a été posé, la main est intacte
        assert!(game.board.melds.is_empty());
        assert_eq!(game.players[0].hand.len(), 3);
        assert!(!game.players[0].has_completed_initial_meld);
    }

    #[test]
    fn test_play_melds_after_initial_needs_no_30_points() {
        let mut game = game_deux_joueurs();
        game.players[0].hand = vec![
            standard(1, Color::Red, 10),
            standard(2, Color::Blue, 10),
            standard(3, Color::Yellow, 10),
            standard(4, Color::Red, 11),
            standard(5, Color::Red, 12),
            standard(6, Color::Red, 13),
        ];

        // Pose initiale : groupe de trois 10 (30 points)
        game.start_turn();
        let groupe = Meld::new(vec![
            standard(1, Color::Red, 10),
            standard(2, Color::Blue, 10),
            standard(3, Color::Yellow, 10),
        ]);
        game.play_melds(vec![groupe]).unwrap();
        game.end_turn().unwrap();

        tour_pioche(&mut game); // Bob pioche et passe

        // Alice rejoue une suite de 36 points : plus d'exigence des 30 points
        game.start_turn();
        let suite = Meld::new(vec![
            standard(4, Color::Red, 11),
            standard(5, Color::Red, 12),
            standard(6, Color::Red, 13),
        ]);
        let points = game.play_melds(vec![suite]).unwrap();
        assert_eq!(points, 36);
        assert!(game.players[0].hand.is_empty());
        assert_eq!(game.board.melds.len(), 2);

        game.end_turn().unwrap();
        assert_eq!(game.winner(), Some(0));
    }

    #[test]
    fn test_manipulation_extends_a_board_meld() {
        let mut game = game_deux_joueurs();
        game.players[0].hand = vec![
            standard(1, Color::Red, 10),
            standard(2, Color::Blue, 10),
            standard(3, Color::Yellow, 10),
            standard(4, Color::Black, 10),
        ];

        // Pose initiale : groupe de trois 10, le 10 noir reste en main
        game.start_turn();
        let groupe_3 = Meld::new(vec![
            standard(1, Color::Red, 10),
            standard(2, Color::Blue, 10),
            standard(3, Color::Yellow, 10),
        ]);
        game.play_melds(vec![groupe_3]).unwrap();
        game.end_turn().unwrap();

        tour_pioche(&mut game); // Bob pioche et passe

        // Alice complète son groupe avec le 10 noir resté en main
        game.start_turn();
        let groupe_4 = Meld::new(vec![
            standard(1, Color::Red, 10),
            standard(2, Color::Blue, 10),
            standard(3, Color::Yellow, 10),
            standard(4, Color::Black, 10),
        ]);
        game.submit_board(vec![groupe_4]).unwrap();
        assert!(game.players[0].hand.is_empty());
        assert_eq!(game.board.melds.len(), 1);
        assert_eq!(game.board.melds[0].Tuiles.len(), 4);

        game.end_turn().unwrap();
        assert_eq!(game.winner(), Some(0));
    }

    #[test]
    fn test_manipulation_cannot_remove_board_tiles() {
        let mut game = game_deux_joueurs();
        game.players[0].hand = vec![
            standard(1, Color::Red, 10),
            standard(2, Color::Blue, 10),
            standard(3, Color::Yellow, 10),
            standard(4, Color::Black, 10),
            standard(5, Color::Red, 3), // reste en main
        ];

        game.start_turn();
        let groupe_4 = Meld::new(vec![
            standard(1, Color::Red, 10),
            standard(2, Color::Blue, 10),
            standard(3, Color::Yellow, 10),
            standard(4, Color::Black, 10),
        ]); // 40 points
        game.play_melds(vec![groupe_4]).unwrap();
        game.end_turn().unwrap();

        tour_pioche(&mut game); // Bob pioche et passe

        // Alice tente de reprendre le 10 noir du plateau : la tuile disparaîtrait
        game.start_turn();
        let groupe_3 = Meld::new(vec![
            standard(1, Color::Red, 10),
            standard(2, Color::Blue, 10),
            standard(3, Color::Yellow, 10),
        ]);
        assert!(game.submit_board(vec![groupe_3]).is_err());

        // Rien n'a bougé : plateau intact, main intacte
        assert_eq!(game.board.melds[0].Tuiles.len(), 4);
        assert_eq!(game.players[0].hand.len(), 1);
    }

    #[test]
    fn test_manipulation_rejected_before_initial_meld() {
        let mut game = game_deux_joueurs();
        game.current_player = 1; // Bob n'a pas encore fait sa pose initiale

        game.start_turn();
        assert!(game.submit_board(vec![]).is_err());
        assert!(game.board.melds.is_empty());
    }

    #[test]
    fn test_manipulation_rejects_invalid_and_foreign_tiles() {
        let mut game = game_deux_joueurs();
        game.players[0].hand = vec![
            standard(1, Color::Red, 10),
            standard(2, Color::Blue, 10),
            standard(3, Color::Yellow, 10),
        ];

        game.start_turn();
        let main = game.players[0].hand.clone();
        game.play_melds(vec![Meld::new(main)]).unwrap(); // pose initiale 30 points
        game.end_turn().unwrap();
        game.current_player = 0; // Alice, pose initiale déjà faite

        game.start_turn();

        // Combinaison invalide (2 tuiles)
        let trop_court = Meld::new(vec![
            standard(1, Color::Red, 10),
            standard(2, Color::Blue, 10),
        ]);
        assert!(game.submit_board(vec![trop_court]).is_err());

        // La tuile 99 n'est ni sur le plateau ni dans la main d'Alice (vide)
        let triche = Meld::new(vec![
            standard(1, Color::Red, 10),
            standard(2, Color::Blue, 10),
            standard(99, Color::Yellow, 10),
        ]);
        assert!(game.submit_board(vec![triche]).is_err());

        // Le plateau n'a pas bougé
        assert_eq!(game.board.melds[0].Tuiles.len(), 3);
    }

    #[test]
    fn test_draw_tile_ends_the_turn() {
        let mut game = game_deux_joueurs();
        game.start_turn();
        let pool_avant = game.pool.len();
        let main_avant = game.players[0].hand.len();

        let tuile = game.draw_tile().unwrap().expect("la pioche n'est pas vide");
        assert_eq!(game.players[0].hand.len(), main_avant + 1);
        assert!(game.players[0].hand.iter().any(|t| t.id == tuile.id));
        assert_eq!(game.pool.len(), pool_avant - 1);

        // Piocher met fin au tour : passage au joueur suivant
        assert_eq!(game.current_player, 1);
        assert!(game.board_backup.is_none());
        assert!(game.hand_backup.is_none());

        // Tour terminé : repiocher est refusé
        assert!(game.draw_tile().is_err());
    }

    #[test]
    fn test_draw_tile_passes_when_pool_is_empty() {
        let mut game = game_deux_joueurs();
        game.pool.clear();
        game.start_turn();
        let main_avant = game.players[0].hand.len();

        // Sac vide : le joueur passe son tour sans piocher
        assert!(game.draw_tile().unwrap().is_none());
        assert_eq!(game.players[0].hand.len(), main_avant);
        assert_eq!(game.current_player, 1);
    }

    #[test]
    fn test_draw_tile_rejected_after_playing() {
        let mut game = game_deux_joueurs();
        game.players[0].hand = vec![
            standard(1, Color::Red, 10),
            standard(2, Color::Blue, 10),
            standard(3, Color::Yellow, 10),
        ];
        game.start_turn();
        let main = game.players[0].hand.clone();
        game.play_melds(vec![Meld::new(main)]).unwrap();

        // Le joueur a déjà posé : impossible de piocher
        assert!(game.draw_tile().is_err());

        // Le tour peut quand même se terminer normalement
        game.end_turn().unwrap();
        assert_eq!(game.current_player, 1);
    }

    #[test]
    fn test_end_turn_requires_an_open_turn_and_an_action() {
        let mut game = game_deux_joueurs();

        // Sans tour ouvert, les actions sont refusées
        assert!(game.end_turn().is_err());
        assert!(game.draw_tile().is_err());
        assert!(game.play_melds(vec![]).is_err());

        // Un tour sans action est refusé tant que le sac contient des tuiles :
        // le joueur doit jouer ou piocher
        game.start_turn();
        assert!(game.end_turn().is_err());

        // Piocher est l'action minimale : elle termine le tour
        game.start_turn();
        assert!(game.draw_tile().unwrap().is_some());
        assert_eq!(game.current_player, 1);

        // Rotation complète : retour au joueur 0
        game.start_turn();
        assert!(game.draw_tile().unwrap().is_some());
        assert_eq!(game.current_player, 0);
    }

    #[test]
    fn test_end_turn_allows_passing_when_pool_is_empty() {
        let mut game = game_deux_joueurs();
        game.pool.clear();

        game.start_turn();
        game.end_turn().unwrap(); // passe autorisée à sac vide
        assert_eq!(game.current_player, 1);
    }

    #[test]
    fn test_end_turn_cancels_on_invalid_board() {
        let mut game = game_deux_joueurs();
        game.start_turn(); // sauvegarde : plateau vide

        // On force un plateau invalide, comme après une manipulation ratée
        game.board = Board {
            melds: vec![Meld::new(vec![
                standard(1, Color::Red, 5),
                standard(2, Color::Red, 6),
            ])],
        };
        let main_avant = game.players[0].hand.clone();

        assert!(game.end_turn().is_err());

        // Tour annulé : plateau restauré, main intacte, même joueur
        assert!(game.board.melds.is_empty());
        assert_eq!(game.players[0].hand, main_avant);
        assert_eq!(game.current_player, 0);
        assert!(game.board_backup.is_none());
        assert!(game.hand_backup.is_none());
    }

    #[test]
    fn test_winner_when_hand_is_empty() {
        let mut game = game_deux_joueurs();
        assert_eq!(game.winner(), None);

        game.players[0].hand.clear();
        assert_eq!(game.winner(), Some(0));
    }
}
