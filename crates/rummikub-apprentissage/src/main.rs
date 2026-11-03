//! Le point d'entrée de la crate d'apprentissage.
//!
//! Usages :
//! - cargo run -p rummikub-apprentissage donnees [parties] [simulations]
//! - cargo run -p rummikub-apprentissage entraine [epoques] [lr]
//! - cargo run -p rummikub-apprentissage joue [parties] [adversaire]
//!
//! `joue` mesure la force du réseau entraîné : il affronte un adversaire
//! ("heuristique", défaut, ou "ismcts" — 300 simulations) sur des graines
//! fixes, dans le même cadre que le tournoi de `rummikub_ia`.

use burn::module::Module;
use rummikub_apprentissage::donnees;
use rummikub_apprentissage::entrainement;
use rummikub_apprentissage::joueur::{self, Adversaire};
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

    if arguments.first().map(|argument| argument.as_str()) == Some("entraine") {
        arguments.remove(0);
        if let Err(err) = mode_entrainement(&arguments) {
            eprintln!("Erreur pendant l'entraînement : {err}");
            std::process::exit(1);
        }
        return;
    }

    if arguments.first().map(|argument| argument.as_str()) == Some("joue") {
        arguments.remove(0);
        if let Err(err) = mode_joue(&arguments) {
            eprintln!("Erreur pendant le tournoi du réseau : {err}");
            std::process::exit(1);
        }
        return;
    }

    eprintln!("Usage : cargo run -p rummikub-apprentissage -- donnees [parties] [simulations]");
    eprintln!("        cargo run -p rummikub-apprentissage -- entraine [epoques] [lr]");
    eprintln!("        cargo run -p rummikub-apprentissage -- joue [parties] [heuristique|ismcts]");
    std::process::exit(1);
}

/// Tournoi de mesure du réseau entraîné : joueur-réseau contre adversaire sur
/// des graines fixes, résultat ajouté au CSV du banc.
fn mode_joue(arguments: &[String]) -> Result<(), String> {
    let nb_parties: u32 = arguments
        .first()
        .and_then(|argument| argument.parse().ok())
        .unwrap_or(20);
    let adversaire = match arguments.get(1).map(|argument| argument.as_str()) {
        None | Some("heuristique") => Adversaire::Heuristique,
        Some("ismcts") => Adversaire::Isme(IsMcts::default()),
        Some(autre) => {
            return Err(format!(
                "adversaire inconnu : {autre} (choix : heuristique, ismcts)"
            ));
        }
    };
    // Les simulations du professeur qui a produit le dataset d'entraînement :
    // étiquette de la génération de réseau dans le banc
    let professeur: u32 = arguments
        .get(2)
        .and_then(|argument| argument.parse().ok())
        .unwrap_or(20);

    let poids = "crates/rummikub-apprentissage/donnees/politique.burnpack";
    if !std::path::Path::new(poids).exists() {
        return Err(format!(
            "poids introuvables ({poids}) : entraínez d'abord avec la sous-commande entraine"
        ));
    }
    let reseau = joueur::charger_politique(poids)?;
    println!(
        "Tournoi : réseau contre {} sur {nb_parties} graines...",
        adversaire.libelle()
    );

    let debut = Instant::now();
    let resultat = joueur::tournoi(nb_parties, &reseau, &adversaire)?;
    println!(
        "Réseau {:.1}% ({} V), adversaire {:.1}% ({} V), {} non terminées, en {:.1}s.",
        resultat.taux_reseau() * 100.0,
        resultat.victoires_reseau,
        (1.0 - resultat.taux_reseau()) * 100.0,
        resultat.victoires_adversaire,
        resultat.non_terminees,
        debut.elapsed().as_secs_f64(),
    );

    joueur::enregistrer_resultat(
        "banc/joue.csv",
        &adversaire,
        nb_parties,
        professeur,
        &resultat,
    )?;
    joueur::tracer_joue("banc/joue.csv", "banc/joue.png")?;
    println!("Résultat ajouté à banc/joue.csv, graphique dans banc/joue.png.");
    Ok(())
}

/// Entraînement du réseau de politique sur le dataset CSV : courbes de loss,
/// PNG de suivi dans banc/, poids de la meilleure époque sauvegardés.
fn mode_entrainement(arguments: &[String]) -> Result<(), String> {
    let epoques: usize = arguments
        .first()
        .and_then(|argument| argument.parse().ok())
        .unwrap_or(30);
    let lr: f64 = arguments
        .get(1)
        .and_then(|argument| argument.parse().ok())
        .unwrap_or(0.001);

    let chemin = "crates/rummikub-apprentissage/donnees/dataset.csv";
    let exemples = donnees::lire_dataset(chemin)?;
    println!(
        "Entraînement : {} exemples, {epoques} époques, lr {lr}",
        exemples.len()
    );

    let debut = Instant::now();
    let resultat = entrainement::entrainer(&exemples, epoques, 256, lr, 42)?;
    println!(
        "Entraîné en {:.1}s. Meilleure époque {} (loss validation {:.4}).",
        debut.elapsed().as_secs_f64(),
        resultat.meilleure_epoque + 1,
        resultat.courbes.validation[resultat.meilleure_epoque],
    );

    let (accord_types, accord_pioche) =
        entrainement::taux_accord(&resultat.meilleur, &exemples, 256)?;
    println!(
        "Accord avec les cibles sur le dataset complet : types {:.1}%, pioche {:.1}%.",
        accord_types * 100.0,
        accord_pioche * 100.0,
    );

    let repertoire_banc = "banc";
    std::fs::create_dir_all(repertoire_banc).map_err(|err| err.to_string())?;
    let png = "banc/entrainement.png";
    entrainement::tracer_courbes(&resultat.courbes, png)?;
    println!("Courbes tracées dans {png}.");

    let poids = "crates/rummikub-apprentissage/donnees/politique.burnpack";
    std::fs::create_dir_all("crates/rummikub-apprentissage/donnees")
        .map_err(|err| err.to_string())?;
    resultat
        .meilleur
        .clone()
        .into_record()
        .save(poids)
        .map_err(|err| err.to_string())?;
    println!("Poids de la meilleure époque écrits dans {poids}.");
    Ok(())
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
