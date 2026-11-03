//! Le point d'entrée de la crate d'apprentissage.
//!
//! Usages :
//! - cargo run -p rummikub-apprentissage donnees [parties] [simulations]

use rummikub_apprentissage::donnees;
use rummikub_ia::ismcts::IsMcts;
use std::time::Instant;

fn main() {
    let mut arguments: Vec<String> = std::env::args().skip(1).collect();

    if arguments.first().map(|argument| argument.as_str()) == Some("donnees") {
        arguments.remove(0);
        if let Err(err) = mode_donnees(&arguments) {
            eprintln!("Erreur pendant la génération du dataset : {err}");
            std::process::exit(1);
        }
        return;
    }

    eprintln!("Usage : cargo run -p rummikub-apprentissage -- donnees [parties] [simulations]");
    std::process::exit(1);
}

/// Génération du dataset d'imitation : ISMCTS contre ISMCTS, exemples
/// écrits en CSV, statistiques ajoutées au CSV du banc.
fn mode_donnees(arguments: &[String]) -> Result<(), String> {
    let nb_parties: u32 = arguments
        .first()
        .and_then(|argument| argument.parse().ok())
        .unwrap_or(10);
    let simulations: usize = arguments
        .get(1)
        .and_then(|argument| argument.parse().ok())
        .unwrap_or(20);

    let ismcts = IsMcts {
        simulations,
        ..Default::default()
    };
    println!(
        "Génération du dataset : {nb_parties} parties ISMCTS contre ISMCTS ({simulations} simulations)..."
    );

    let debut = Instant::now();
    let exemples = donnees::generer_dataset(nb_parties, &ismcts)?;
    let stats = donnees::statistiques(&exemples);

    let chemin = "crates/rummikub-apprentissage/donnees/dataset.csv";
    donnees::ecrire_dataset(chemin, &exemples)?;
    donnees::enregistrer_statistiques(
        "banc/donnees.csv",
        nb_parties,
        simulations,
        &stats,
        debut.elapsed(),
    )?;

    println!(
        "{} exemples ({} poses, {} pioches) écrits dans {chemin} en {:.1}s",
        stats.exemples,
        stats.poses,
        stats.pioches,
        debut.elapsed().as_secs_f64()
    );
    Ok(())
}
