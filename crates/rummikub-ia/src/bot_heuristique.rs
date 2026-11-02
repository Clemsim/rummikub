//! Le bot heuristique : la baseline du projet. Sa stratégie tient en une
//! phrase — poser la combinaison qui rapporte le plus de points ce tour-ci,
//! sinon piocher. Les IA plus ambitieuses (MCTS, RL) devront battre CELA pour
//! justifier leur complexité.

use rummikub_core::coups::{Coup, coups_possibles, jouer_coup};
use rummikub_core::partie::Partie;

/// Choisit le coup du joueur courant : la pose la plus avantageuse s'il en
/// existe une (coups_possibles trie les poses par points décroissants),
/// sinon None — il faudra piocher.
pub fn choisir_coup(partie: &Partie) -> Result<Option<Coup>, String> {
    let coups = coups_possibles(partie)?;
    Ok(coups
        .into_iter()
        .find(|coup| matches!(coup, Coup::Poser(_))))
}

/// Joue le tour complet du joueur courant : la meilleure pose possible puis
/// fin de tour, ou pioche — ce qui termine le tour, même à pioche vide.
pub fn jouer_tour(partie: &mut Partie) -> Result<(), String> {
    partie.ouvrir_tour();

    match choisir_coup(partie)? {
        Some(coup) => {
            jouer_coup(partie, coup)?;
            partie.terminer_tour()?;
        }
        None => {
            partie.piocher()?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests;
