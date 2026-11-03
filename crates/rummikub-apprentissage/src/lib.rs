//! Apprentissage pour le Rummikub : encodage des positions, génération de
//! données par self-play, réseaux (burn) et décodeurs. S'appuie sur l'API
//! publique de `rummikub_core` (les règles) et de `rummikub_ia` (le
//! professeur ISMCTS) — le réseau propose, les règles disposent.

// Naming français délibéré : le lint non_snake_case reste désactivé ici aussi.
#![allow(non_snake_case)]

pub mod donnees;
pub mod encodeur;
pub mod entrainement;
pub mod modele;
