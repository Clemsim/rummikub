//! L'entraînement du réseau de politique par imitation : descente de
//! gradient classique, supervisée par les décisions de l'ISMCTS.
//!
//! ## Les mots, expliqués
//!
//! - **Époque (epoch)** : un passage complet sur les données d'entraînement.
//! - **Lot (batch)** : au lieu d'apprendre exemple par exemple (bruit) ou
//!   tous à la fois (lent), on découpe en tranches de `taille_lot` et on
//!   moyenne le gradient dessus — bon compromis vitesse/stabilité.
//! - **Entropie croisée binaire (BCE)** : la « mesure de l'erreur » du
//!   multi-label. Pour chaque sortie : petit logarithme de pénalité quand le
//!   réseau dit « 0 » alors que la vérité est « 1 » (et inversement). La
//!   moyenne sur les 54 sorties donne la loss du lot.
//! - **Adam** : l'optimiseur. Il met à jour les poids dans la direction qui
//!   fait baisser la loss, avec un pas adapté coordonnée par coordonnée
//!   (mémoires de moyenne et de variance des gradients). `lr` (learning
//!   rate) borne l'ampleur de chaque pas.
//! - **Surapprentissage** : quand le réseau mémorise le jeu d'entraînement
//!   au lieu d'apprendre la règle générale. Visible aux courbes : la loss
//!   d'entraînement continue de baisser pendant que celle de validation
//!   remonte. On garde donc le modèle de l'époque où la validation est la
//!   meilleure.

use crate::donnees::Exemple;
use crate::encodeur::{NB_TYPES, TAILLE_VECTEUR};
use crate::modele::{NB_SORTIES, Politique, PolitiqueConfig};
use burn::module::Module;
use burn::optim::{AdamConfig, GradientsParams};
use burn::prelude::*;
use burn::store::ModuleRecord;
use rand::SeedableRng;
use rand::seq::SliceRandom;

/// Les courbes d'apprentissage : une valeur par époque
#[derive(Debug, Clone)]
pub struct Courbes {
    /// Loss moyenne sur les lots d'entraînement, par époque
    pub train: Vec<f32>,
    /// Loss moyenne sur l'ensemble de validation, par époque
    pub validation: Vec<f32>,
}

/// Le résultat d'un entraînement
pub struct Resultat {
    pub courbes: Courbes,
    /// Le réseau réinitialisé avec les poids de la meilleure époque (loss de
    /// validation minimale), sans autodiff
    pub meilleur: Politique,
    /// L'index (0-based) de la meilleure époque
    pub meilleure_epoque: usize,
}

/// Découpe les exemples en (entraînement, validation) par une proportion de
/// validation et une graine. Déterministe : même graine, même split — les
/// courbes se lisent d'une exécution à l'autre.
pub fn separer_train_validation(
    exemples: &[Exemple],
    proportion_validation: f32,
    graine: u64,
) -> (Vec<Exemple>, Vec<Exemple>) {
    let mut indices: Vec<usize> = (0..exemples.len()).collect();
    let mut rng = rand::rngs::StdRng::seed_from_u64(graine);
    indices.shuffle(&mut rng);
    let nb_validation = (exemples.len() as f32 * proportion_validation) as usize;
    let decoupe = nb_validation.min(exemples.len());
    let validation = indices[..decoupe]
        .iter()
        .map(|&index| exemples[index].clone())
        .collect();
    let entrainement = indices[decoupe..]
        .iter()
        .map(|&index| exemples[index].clone())
        .collect();
    (entrainement, validation)
}

/// Entraîne le réseau par imitation sur `exemples`, split 80/20 par graine.
///
/// ` Politique ` sort des logits ; la BCE est configurée avec `logits = true`
/// pour appliquer la sigmoïde en interne (plus stable numériquement).
pub fn entrainer(
    exemples: &[Exemple],
    epoques: usize,
    taille_lot: usize,
    lr: f64,
    graine: u64,
) -> Result<Resultat, String> {
    if exemples.len() < 100 {
        return Err(format!(
            "dataset trop petit ({} exemples) : générez-en d'abord avec la sous-commande donnees",
            exemples.len()
        ));
    }

    let device = Device::flex().autodiff();
    device.seed(graine);

    // Split 80/20 mélangé une fois pour toutes : le même split pour toutes
    // les époques, donc les courbes se lisent d'une époque à l'autre.
    let (entrainement, validation) = separer_train_validation(exemples, 0.2, graine);
    let mut rng = rand::rngs::StdRng::seed_from_u64(graine);
    let mut indices_train: Vec<usize> = (0..entrainement.len()).collect();
    let indices_validation: Vec<usize> = (0..validation.len()).collect();

    let mut modele = PolitiqueConfig::new(TAILLE_VECTEUR, NB_SORTIES).init(&device);
    let mut optimiseur = AdamConfig::new().init();
    let loss_fn = burn::nn::loss::BinaryCrossEntropyLossConfig::new()
        .with_logits(true)
        .init(&device);

    let mut courbes = Courbes {
        train: Vec::new(),
        validation: Vec::new(),
    };
    let mut meilleure_loss = f32::INFINITY;
    let mut meilleur_record: Option<ModuleRecord> = None;
    let mut meilleure_epoque = 0;

    for _epoque in 0..epoques {
        // --- Entraînement : lots mélangés à chaque époque
        indices_train.shuffle(&mut rng);
        let mut perte_totale = 0.0f32;
        let mut nb_lots = 0usize;
        for lot in indices_train.chunks(taille_lot) {
            let entree = lot_d_entree(&entrainement, lot, &device);
            let cibles = lot_de_cibles(&entrainement, lot, &device);
            let logits = modele.forward(entree);
            let loss = loss_fn.forward(logits, cibles);
            perte_totale += loss.clone().into_scalar::<f32>();
            nb_lots += 1;

            let grads = loss.backward();
            let grads = GradientsParams::from_grads(grads, &modele);
            modele = optimiseur.step(lr, modele, grads);
        }
        courbes.train.push(perte_totale / nb_lots as f32);

        // --- Validation : pas de mise à jour, réseau sans autodiff
        let modele_validation = modele.valid();
        let mut perte_validation = 0.0f32;
        let mut nb_lots_validation = 0usize;
        for lot in indices_validation.chunks(taille_lot) {
            let entree = lot_d_entree(&validation, lot, &device);
            let cibles = lot_de_cibles(&validation, lot, &device);
            let logits = modele_validation.forward(entree);
            let loss = loss_fn.forward(logits, cibles);
            perte_validation += loss.into_scalar::<f32>();
            nb_lots_validation += 1;
        }
        let moyenne = perte_validation / nb_lots_validation as f32;
        courbes.validation.push(moyenne);

        if moyenne < meilleure_loss {
            meilleure_loss = moyenne;
            meilleure_epoque = courbes.validation.len() - 1;
            // Les poids actuels, figés hors autodiff pour l'inférence
            meilleur_record = Some(modele.valid().into_record());
        }
    }

    let meilleur_record =
        meilleur_record.ok_or_else(|| "aucune époque d'entraînement".to_string())?;
    let meilleur = PolitiqueConfig::new(TAILLE_VECTEUR, NB_SORTIES)
        .init(&Device::flex())
        .load_record(meilleur_record);

    Ok(Resultat {
        courbes,
        meilleur,
        meilleure_epoque,
    })
}

/// Le lot d'entrées, forme [taille du lot, TAILLE_VECTEUR]
fn lot_d_entree(exemples: &[Exemple], indices: &[usize], device: &Device) -> Tensor<2> {
    let mut valeurs = Vec::with_capacity(indices.len() * TAILLE_VECTEUR);
    for &index in indices {
        valeurs.extend_from_slice(&exemples[index].entree);
    }
    Tensor::<1>::from_floats(valeurs.as_slice(), device).reshape([indices.len(), TAILLE_VECTEUR])
}

/// Le lot de cibles, forme [taille du lot, NB_SORTIES] : les 53 drapeaux de
/// types puis le drapeau piocher en dernière colonne, comme entiers 0/1
/// (la BCE de burn attend des cibles entières).
fn lot_de_cibles(exemples: &[Exemple], indices: &[usize], device: &Device) -> Tensor<2, Int> {
    let mut valeurs = Vec::with_capacity(indices.len() * NB_SORTIES);
    for &index in indices {
        let exemple = &exemples[index];
        for &vrai in &exemple.types_jouees {
            valeurs.push(if vrai { 1.0 } else { 0.0 });
        }
        valeurs.push(if exemple.piocher { 1.0 } else { 0.0 });
    }
    Tensor::<1>::from_floats(valeurs.as_slice(), device)
        .int()
        .reshape([indices.len(), NB_SORTIES])
}

/// Le taux d'accord du réseau (seuil 0.5 sur les sigmoïdes) avec les cibles
/// du split de validation — la métrique réelle d'imitation, lisible sans
/// comprendre les logits.
///
/// Retourne (accord_types, accord_pioche) : la proportion d'exemples où les
/// 53 drapeaux prédits égalent exactement les cibles, et la proportion où le
/// drapeau piocher prédit égalent la cible.
pub fn taux_accord(
    modele: &crate::modele::Politique,
    exemples: &[Exemple],
    taille_lot: usize,
) -> Result<(f32, f32), String> {
    if exemples.is_empty() {
        return Err("aucun exemple à évaluer".to_string());
    }
    let device = Device::flex();
    let mut accords_types = 0usize;
    let mut accords_pioche = 0usize;

    for lot in (0..exemples.len())
        .collect::<Vec<_>>()
        .chunks(taille_lot.max(1))
    {
        let entree = lot_d_entree(exemples, lot, &device);
        let probabilites = burn::tensor::activation::sigmoid(modele.forward(entree)).into_data();
        let valeurs = probabilites.iter::<f32>().collect::<Vec<_>>();

        for (position_dans_le_lot, &index) in lot.iter().enumerate() {
            let exemple = &exemples[index];
            let ligne = &valeurs
                [position_dans_le_lot * NB_SORTIES..(position_dans_le_lot + 1) * NB_SORTIES];
            let types_predits: Vec<bool> = ligne[..NB_TYPES].iter().map(|&p| p > 0.5).collect();
            let types_reels: Vec<bool> = exemple.types_jouees.to_vec();
            if types_predits == types_reels {
                accords_types += 1;
            }
            if (ligne[NB_TYPES] > 0.5) == exemple.piocher {
                accords_pioche += 1;
            }
        }
    }

    let total = exemples.len() as f32;
    Ok((accords_types as f32 / total, accords_pioche as f32 / total))
}

/// Trace les courbes train/validation dans un PNG (le surapprentissage se
/// voit : train baisse, validation remonte).
pub fn tracer_courbes(courbes: &Courbes, chemin: &str) -> Result<(), String> {
    use plotters::prelude::*;

    let epoques = courbes.train.len().max(2);
    let max_loss = courbes
        .train
        .iter()
        .chain(&courbes.validation)
        .copied()
        .fold(0.0f32, f32::max)
        * 1.1
        + 1e-6;

    let racine = BitMapBackend::new(chemin, (800, 500)).into_drawing_area();
    racine.fill(&WHITE).map_err(|erreur| erreur.to_string())?;
    let mut diagramme = ChartBuilder::on(&racine)
        .caption("Courbes d'apprentissage (BCE)", ("sans-serif", 24))
        .margin(15)
        .x_label_area_size(45)
        .y_label_area_size(55)
        .build_cartesian_2d(0..epoques, 0.0f32..max_loss)
        .map_err(|erreur| erreur.to_string())?;
    diagramme
        .configure_mesh()
        .x_desc("époque")
        .y_desc("loss")
        .disable_x_mesh()
        .disable_y_mesh()
        .draw()
        .map_err(|erreur| erreur.to_string())?;

    let serie_train = diagramme
        .draw_series(LineSeries::new(
            courbes.train.iter().enumerate().map(|(i, &l)| (i + 1, l)),
            &RED,
        ))
        .map_err(|erreur| erreur.to_string())?;
    serie_train
        .label("train")
        .legend(move |(x, y)| Rectangle::new([(x - 10, y - 3), (x + 10, y + 3)], RED));
    let serie_validation = diagramme
        .draw_series(LineSeries::new(
            courbes
                .validation
                .iter()
                .enumerate()
                .map(|(i, &l)| (i + 1, l)),
            &BLUE,
        ))
        .map_err(|erreur| erreur.to_string())?;
    serie_validation
        .label("validation")
        .legend(move |(x, y)| Rectangle::new([(x - 10, y - 3), (x + 10, y + 3)], BLUE));
    diagramme
        .configure_series_labels()
        .background_style(WHITE.mix(0.8))
        .border_style(BLACK)
        .draw()
        .map_err(|erreur| erreur.to_string())?;

    racine.present().map_err(|erreur| erreur.to_string())
}

#[cfg(test)]
mod tests;
