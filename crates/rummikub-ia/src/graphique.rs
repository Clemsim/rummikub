//! Génération des graphiques du banc d'essai.
//!
//! La librairie utilisée est [`plotters`](https://crates.io/crates/plotters),
//! la référence des graphiques en Rust pur : elle écrit des PNG/SVG sans
//! serveur graphique ni navigateur, donc utilisable dans les tests et les
//! scripts.

use plotters::prelude::*;

/// Trace une courbe (valeur de réglage, taux de victoire) dans un PNG
pub fn courbe(
    points: &[(f64, f64)],
    titre: &str,
    libelle_x: &str,
    chemin: &str,
) -> Result<(), String> {
    let racine = BitMapBackend::new(chemin, (800, 500)).into_drawing_area();
    racine.fill(&WHITE).map_err(|erreur| erreur.to_string())?;

    let mut diagramme = ChartBuilder::on(&racine)
        .caption(titre, ("sans-serif", 24))
        .margin(15)
        .x_label_area_size(45)
        .y_label_area_size(55)
        .build_cartesian_2d(0.0f64..1.0f64, 0.0f64..1.0f64)
        .map_err(|erreur| erreur.to_string())?;

    diagramme
        .configure_mesh()
        .x_desc(libelle_x)
        .y_desc("taux de victoire")
        .disable_x_mesh()
        .disable_y_mesh()
        .draw()
        .map_err(|erreur| erreur.to_string())?;

    diagramme
        .draw_series(LineSeries::new(
            points.iter().map(|(epsilon, taux)| (*epsilon, *taux)),
            &RED,
        ))
        .map_err(|erreur| erreur.to_string())?;
    diagramme
        .draw_series(
            points
                .iter()
                .map(|(epsilon, taux)| Circle::new((*epsilon, *taux), 5, BLUE.filled())),
        )
        .map_err(|erreur| erreur.to_string())?;

    racine.present().map_err(|erreur| erreur.to_string())
}

#[cfg(test)]
mod tests;
