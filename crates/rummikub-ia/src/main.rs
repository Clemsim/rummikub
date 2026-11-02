//! Le banc d'essai en ligne de commande : fait s'affronter l'ISMCTS et le
//! bot heuristique sur des graines fixes, pour suivre l'évolution des IA,
//! et trace la courbe de réglage d'epsilon.
//!
//! Usages :
//! - cargo run -p rummikub-ia [graines] [simulations] [epsilon]
//! - cargo run -p rummikub-ia graphique [graines] [simulations]
//! - cargo run -p rummikub-ia graphique-2d [graines] [simulations]

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

    if arguments.first().map(|argument| argument.as_str()) == Some("graphique-2d") {
        arguments.remove(0);
        if let Err(err) = mode_cartographie(&arguments) {
            eprintln!("Erreur pendant la cartographie 2D : {err}");
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

/// Balayages d'epsilon et de beta : courbes (PNG) et tableaux CSV.
/// L'epsilon est balayé au réglage d'élargissement par défaut, et beta est
/// balayé à epsilon par défaut — les deux sur les mêmes graines.
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
    let points_epsilon = tournoi::balayage(&epsilons, graines, |epsilon| IsMcts {
        epsilon_rollout: epsilon,
        simulations,
        ..Default::default()
    })?;
    afficher_points("epsilon", &points_epsilon);
    ecrire_csv("banc/balayage-epsilon.csv", "epsilon", &points_epsilon)?;
    graphique::courbe(
        &points_epsilon,
        "Taux de victoire de l'ISMCTS selon epsilon",
        "epsilon (probabilité de coup aléatoire en rollout)",
        "banc/graphique-epsilon.png",
    )?;
    println!("Graphique écrit dans banc/graphique-epsilon.png");

    let betas = [0.1, 0.2, 0.3, 0.5, 0.7, 1.0];
    println!(
        "Balayage de beta (élargissement progressif, epsilon au défaut) : {graines} graines..."
    );
    let points_beta = tournoi::balayage(&betas, graines, |beta| IsMcts {
        beta_elargissement: beta,
        simulations,
        ..Default::default()
    })?;
    afficher_points("beta", &points_beta);
    ecrire_csv("banc/balayage-beta.csv", "beta", &points_beta)?;
    graphique::courbe(
        &points_beta,
        "Taux de victoire de l'ISMCTS selon beta",
        "beta (élargissement progressif k = α·N^β)",
        "banc/graphique-beta.png",
    )?;
    println!("Graphique écrit dans banc/graphique-beta.png");
    Ok(())
}

/// Affiche les points d'un balayage et la meilleure valeur
fn afficher_points(nom: &str, points: &[(f64, f64)]) {
    for (valeur, taux) in points {
        println!("{nom} {valeur} → {:.0}% de victoires", taux * 100.0);
    }
    let meilleur = points
        .iter()
        .copied()
        .max_by(|a, b| a.1.total_cmp(&b.1))
        .expect("au moins un point de balayage");
    println!(
        "Meilleur {nom} : {:.2} ({:.0}% de victoires)",
        meilleur.0,
        meilleur.1 * 100.0
    );
}

/// Écrit un CSV à deux colonnes (valeur, taux) pour un balayage
fn ecrire_csv(chemin: &str, entete: &str, points: &[(f64, f64)]) -> Result<(), String> {
    let mut csv = format!("{entete},taux_ismcts\n");
    for (valeur, taux) in points {
        csv.push_str(&format!("{valeur},{taux}\n"));
    }
    std::fs::create_dir_all("banc").map_err(|erreur| erreur.to_string())?;
    std::fs::write(chemin, csv).map_err(|erreur| erreur.to_string())
}

/// Cartographie 2D : les couples (epsilon, beta) croisés, affichés en
/// matrice, archivés en CSV et tracés en courbes familles (PNG)
fn mode_cartographie(arguments: &[String]) -> Result<(), String> {
    let graines: u32 = arguments
        .first()
        .and_then(|argument| argument.parse().ok())
        .unwrap_or(20);
    let simulations: usize = arguments
        .get(1)
        .and_then(|argument| argument.parse().ok())
        .unwrap_or(50);

    let epsilons = [0.0, 0.1, 0.2, 0.3, 0.5, 0.7, 1.0];
    let betas = [0.1, 0.2, 0.3, 0.5, 0.7, 1.0];
    println!(
        "Cartographie 2D : {} epsilons × {} betas, {graines} graines, {simulations} simulations...",
        epsilons.len(),
        betas.len()
    );

    let grille = tournoi::balayage_2d(&epsilons, &betas, graines, |epsilon, beta| IsMcts {
        epsilon_rollout: epsilon,
        beta_elargissement: beta,
        simulations,
        ..Default::default()
    })?;

    // Matrice lisible : une ligne par beta, une colonne par epsilon
    println!(
        "{: >8} | {}",
        "",
        epsilons
            .iter()
            .map(|epsilon| format!("ε={epsilon:<4}"))
            .collect::<Vec<_>>()
            .join(" ")
    );
    for beta in &betas {
        let ligne: Vec<String> = epsilons
            .iter()
            .map(|epsilon| {
                let (_, _, taux) = grille
                    .iter()
                    .find(|(e, b, _)| *e == *epsilon && *b == *beta)
                    .expect("chaque couple est joué");
                format!("{:.0}%     ", taux * 100.0)
            })
            .collect();
        println!("{: >8} | {}", format!("β={beta}"), ligne.join(" "));
    }

    let meilleure = grille
        .iter()
        .copied()
        .max_by(|a, b| a.2.total_cmp(&b.2))
        .expect("la grille n'est pas vide");
    println!(
        "Meilleur couple : epsilon {:.1}, beta {:.1} ({:.0}% de victoires)",
        meilleure.0,
        meilleure.1,
        meilleure.2 * 100.0
    );

    // CSV à plat : epsilon,beta,taux
    let mut csv = String::from("epsilon,beta,taux_ismcts\n");
    for (epsilon, beta, taux) in &grille {
        csv.push_str(&format!("{epsilon},{beta},{taux}\n"));
    }
    std::fs::create_dir_all("banc").map_err(|erreur| erreur.to_string())?;
    std::fs::write("banc/cartographie-epsilon-beta.csv", csv)
        .map_err(|erreur| erreur.to_string())?;

    graphique::courbes_familles(
        &epsilons,
        &betas,
        &grille,
        "Taux de victoire de l'ISMCTS : une courbe par beta",
        "epsilon (probabilité de coup aléatoire en rollout)",
        "banc/cartographie-epsilon-beta.png",
    )?;
    println!("Graphique écrit dans banc/cartographie-epsilon-beta.png");
    Ok(())
}
