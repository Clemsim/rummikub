//! Le réseau de politique : l'approximation de fonction qui regarde l'état
//! encodé de la partie et prédit la probabilité de chaque action possible.
//!
//! ## Les mots, expliqués
//!
//! - **Couche (layer)** : une transformation linéaire suivie d'une
//!   non-linéarité. Une couche regarde l'entrée, la pondère (chaque entrée a
//!   un poids = son importance), ajoute un biais, et passe le résultat dans
//!   une fonction d'activation.
//! - **Poids (weights)** : les coefficients appris. Ils sont stockés dans une
//!   matrice. Si l'entrée a 100 éléments et la couche en sortie 50, la
//!   matrice des poids est 100 x 50 (5000 nombres appris).
//! - **Biais (bias)** : un vecteur ajouté à la sortie de la couche, avant
//!   l'activation. Il permet à la couche de décaler sa réponse : sans biais,
//!   une activation nulle forcerait toujours une sortie nulle.
//! - **Activation (ReLU)** : `max(0, x)`. Elle introduit la non-linéarité :
//!   sans elle, une suite de couches linéaires se réduirait à une seule
//!   transformation linéaire, incapable d'approcher des fonctions complexes
//!   (comme la valeur d'une position de jeu).
//! - **Logits** : les scores bruts de sortie, avant normalisation. Chacun
//!   correspond à une action. On les passe dans une sigmoïde pour obtenir des
//!   probabilités entre 0 et 1 (somme non nécessairement 1, car plusieurs
//!   actions peuvent être bonnes simultanément en multi-label).

use crate::encodeur::NB_TYPES;
use burn::module::Module;
use burn::nn::{Linear, LinearConfig, Relu};
use burn::prelude::*;
use burn::store::ModuleRecord;

/// Le nombre de sorties du réseau : 53 types de tuiles + 1 drapeau piocher.
pub const NB_SORTIES: usize = NB_TYPES + 1;

/// Le réseau de politique.
///
/// Architecture : un perceptron multicouche (MLP) simple.
/// 1. Couche d'entrée : projette l'état encodé dans un espace caché.
/// 2. Couche cachée : affine la représentation (ReLU entre les deux).
/// 3. Couche de sortie : produit un logit par action possible (53).
///
/// Pourquoi un MLP simple ? Les états encodés sont déjà plats (vecteurs de
/// 0/1 et de nombres) : pas besoin de convolutions ou d'attention, une
/// hiérarchie de projections linéaires + ReLU suffit pour approximer une
/// politique sur des états de cette taille. On pourra raffiner plus tard.
#[derive(Module, Debug)]
pub struct Politique {
    /// La couche d'entrée (poids + biais)
    entree: Linear,
    /// La couche cachée
    cachee: Linear,
    /// La non-linéarité entre les deux couches
    activation: Relu,
    /// La couche de sortie (logits)
    sortie: Linear,
}

/// La configuration du réseau : nombre d'entrées, taille de la couche
/// cachée, nombre d'actions de sortie.
#[derive(Config, Debug)]
pub struct PolitiqueConfig {
    /// La taille du vecteur d'entrée (l'état encodé)
    pub taille_entree: usize,
    /// La taille de la couche cachée (compromis : assez grand pour
    /// représenter, assez petit pour rester rapide sur CPU)
    #[config(default = 128)]
    pub taille_cachee: usize,
    /// Le nombre d'actions de sortie (54 dans notre cas)
    pub nb_actions: usize,
}

impl PolitiqueConfig {
    /// Construit le réseau initialisé aléatoirement sur `device`.
    pub fn init(&self, device: &Device) -> Politique {
        Politique {
            entree: LinearConfig::new(self.taille_entree, self.taille_cachee).init(device),
            cachee: LinearConfig::new(self.taille_cachee, self.taille_cachee).init(device),
            activation: Relu::new(),
            sortie: LinearConfig::new(self.taille_cachee, self.nb_actions).init(device),
        }
    }
}

impl Politique {
    /// La passe avant : état encodé -> logits par action.
    ///
    /// # Formes
    /// - entrée : `[taille_du_lot, taille_entree]` (un vecteur par exemple)
    /// - sortie : `[taille_du_lot, nb_actions]` (un logit par action)
    pub fn forward(&self, entree: Tensor<2>) -> Tensor<2> {
        // 1. Projection d'entrée + non-linéarité
        let cachee = self.activation.forward(self.entree.forward(entree));
        // 2. Affinage caché + non-linéarité
        let cachee = self.activation.forward(self.cachee.forward(cachee));
        // 3. Sortie : logits bruts, pas de sigmoïde ici. La sigmoïde sera
        //    appliquée lors du calcul de la loss (pour la stabilité numérique)
        //    et par le décodeur (pour l'interprétation en probabilités).
        self.sortie.forward(cachee)
    }

    /// Expose les poids du réseau pour la sauvegarde/le chargement
    pub fn vers_record(&self) -> ModuleRecord {
        self.clone().into_record()
    }
}

#[cfg(test)]
mod tests;
