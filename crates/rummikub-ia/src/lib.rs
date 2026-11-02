//! Joueurs artificiels pour le Rummikub : bots, recherche, apprentissage.
//! S'appuie uniquement sur l'API publique de `rummikub_core` : les IA passent
//! par les mêmes coups que les humains, elles ne contournent jamais les
//! règles (voir coups_possibles / jouer_coup dans le core).

// Naming français délibéré : le lint non_snake_case reste désactivé ici aussi.
#![allow(non_snake_case)]

pub mod bot_heuristique;
