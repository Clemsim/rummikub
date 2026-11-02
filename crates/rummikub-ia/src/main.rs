//! Le banc d'essai en ligne de commande : fait s'affronter l'ISMCTS et le
//! bot heuristique sur des graines fixes, pour suivre l'évolution des IA.
//!
//! Usage : cargo run -p rummikub-ia [nb_graines] [simulations] [epsilon]

use rummikub_ia::ismcts::IsMcts;
use rummikub_ia::tournoi;

fn main() {
    let mut arguments = std::env::args().skip(1);
    let graines: u32 = arguments
        .next()
        .and_then(|argument| argument.parse().ok())
        .unwrap_or(10);
    let simulations: usize = arguments
        .next()
        .and_then(|argument| argument.parse().ok())
        .unwrap_or(100);
    let epsilon: f64 = arguments
        .next()
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

    match tournoi::tournoi(graines, &ismcts) {
        Ok(resultat) => println!(
            "ISMCTS {} — {} heuristique ({} non terminées) en {:.1}s — taux ISMCTS : {:.0}%",
            resultat.victoires_ismcts,
            resultat.victoires_heuristique,
            resultat.non_terminees,
            resultat.duree.as_secs_f64(),
            resultat.taux_ismcts() * 100.0
        ),
        Err(err) => {
            eprintln!("Erreur pendant le tournoi : {err}");
            std::process::exit(1);
        }
    }
}
