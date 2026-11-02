//! Bibliothèque de logique Rummikub : tuiles, combinaisons, plateau, joueurs
//! et déroulement d'une partie. Sans I/O, réutilisable depuis un binaire, un
//! serveur, une page web (WASM) ou un bot.

// Naming français délibéré (Tuile, Couleur::Rouge, piocher...) : le lint
// non_snake_case est désactivé pour toute la bibliothèque, ne pas renommer
// en anglais.
#![allow(non_snake_case)]

pub mod combinaison;
pub mod coups;
pub mod joueur;
pub mod partie;
pub mod plateau;
pub mod tuile;

#[cfg(test)]
mod tests;
