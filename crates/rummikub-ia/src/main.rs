//! Le banc d'essai en ligne de commande : fait s'affronter l'ISMCTS et le
//! bot heuristique sur des graines fixes, pour suivre l'évolution des IA,
//! et trace la courbe de réglage d'epsilon.
//!
//! Usages :
//! - cargo run -p rummikub-ia [graines] [simulations] [epsilon]
//! - cargo run -p rummikub-ia graphique [graines] [simulations]

use rummikub_ia::graphique;
use rummikub_ia::ismcts::IsMcts;
use rummikub_ia::tournoi;

const CHEMIN_CSV: &str = "banc/resultats.csv";

fn main() {
    let mut arguments: Vec<String> = std::env::args().skip(1).collect();

    if arguments.first().map(|argument| argument.as_str()) == Some("graphique") {
        arguments.remove(0);
        if let Err(err) = mode_graphique(&arguments) {
            eprintln!("Erreur pendant le graphique : {err}");
            std::process::exit(1);
        }
        return;
    }

    if let Err(err) = mode_tournoi(&arguments) {
        eprintln!("Erreur pendant le tournoi : {err}");
        std::process::exit(1);
    }
}

/// Tournoi simple : ISMCTS contre le bot heuristique, résultat affiché
/// puis ajouté au CSV du banc.
fn mode_tournoi(arguments: &[String]) -> Result<(), String> {
    let graines: u32 = arguments
        .first()
        .and_then(|argument| argument.parse().ok())
        .unwrap_or(10);
    let simulations: usize = arguments
        .get(1)
        .and_then(|argument| argument.parse().ok())
        .unwrap_or(100);
    let epsilon: f64 = arguments
        .get(2)
        .and_then(|argument| argument.parse().ok())
        .unwrap_or(0.2);

    let ismcts = IsMcts {
        simulations,
        epsilon_rollout: epsilon,
        ..Default::default()
    };

    println!(
        "Tournoi ISMCTS contre heuristique : {graines} graines, {simulations} simulations, epsilon {epsilon}"
    );

    let resultat = tournoi::tournoi(graines, &ismcts)?;
    tournoi::enregistrer_resultat(CHEMIN_CSV, epsilon, graines, simulations, &resultat)?;

    println!(
        "ISMCTS {} — {} heuristique ({} non terminées) en {:.1}s — taux ISMCTS : {:.0}%",
        resultat.victoires_ismcts,
        resultat.victoires_heuristique,
        resultat.non_terminees,
        resultat.duree.as_secs_f64(),
        resultat.taux_ismcts() * 100.0
    );
    println!("Résultat ajouté à {CHEMIN_CSV}");
    Ok(())
}

/// Balayage d'epsilon : courbe du taux de victoire (PNG) et tableau CSV
fn mode_graphique(arguments: &[String]) -> Result<(), String> {
    let graines: u32 = arguments
        .first()
        .and_then(|argument| argument.parse().ok())
        .unwrap_or(5);
    let simulations: usize = arguments
        .get(1)
        .and_then(|argument| argument.parse().ok())
        .unwrap_or(30);

    let epsilons = [0.0, 0.1, 0.2, 0.3, 0.5, 0.7, 1.0];
    println!("Balayage d'epsilon : {graines} graines, {simulations} simulations par décision...");

    let points = tournoi::balayage_epsilon(&epsilons, graines, simulations)?;
    for (epsilon, taux) in &points {
        println!("epsilon {epsilon:.1} → {:.0}% de victoires", taux * 100.0);
    }
    let meilleur = points
        .iter()
        .copied()
        .max_by(|a, b| a.1.total_cmp(&b.1))
        .expect("au moins un point de balayage");
    println!(
        "Meilleur epsilon : {:.1} ({:.0}% de victoires)",
        meilleur.0,
        meilleur.1 * 100.0
    );

    // CSV du balayage
    let mut csv = String::from("epsilon,taux_ismcts\n");
    for (epsilon, taux) in &points {
        csv.push_str(&format!("{epsilon},{taux}\n"));
    }
    std::fs::create_dir_all("banc").map_err(|erreur| erreur.to_string())?;
    std::fs::write("banc/balayage-epsilon.csv", csv).map_err(|erreur| erreur.to_string())?;

    // Courbe PNG
    graphique::courbe_epsilon(&points, "banc/graphique-epsilon.png")?;
    println!("Graphique écrit dans banc/graphique-epsilon.png");
    Ok(())
}
