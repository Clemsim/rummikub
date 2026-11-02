use crate::board::Board;
use crate::player::Player;
use crate::tuile::Tile;

pub struct Game {
    pub players: Vec<Player>,
    pub current_player: usize,
    pub board: Board,
    pub pool: Vec<Tile>, // Pioche
    // Utile pour annuler le tour si le joueur fait une manipulation illégale :
    pub board_backup: Option<Board>,
    pub hand_backup: Option<Vec<Tile>>,
}
