// Naming français délibéré (Tuile, Tuiles, Type...) : le lint non_snake_case
// est désactivé pour tout le projet, ne pas renommer en anglais.
#![allow(non_snake_case)]

mod combinaison;
mod joueur;
mod partie;
mod plateau;
mod tuile;

use combinaison::Combinaison;
use joueur::Joueur;
use partie::Partie;
use tuile::{Color, Tuile, generate_pool, shuffle_pool};

fn main() {
    // 1. Génération & Mélange de la pioche
    let mut pioche = generate_pool();
    println!("Sac généré avec {} tuiles.", pioche.len());
    shuffle_pool(&mut pioche);

    // 2. Test d'une suite avec Joker
    // Exemple : Joker, 8 Bleu, 10 Bleu (Le joker prend la place du 9 Bleu -> suite 8, 9, 10)
    let suite = Combinaison::new(vec![
        Tuile::new_standard(1, Color::Blue, 8),
        Tuile::new_joker(2),
        Tuile::new_standard(3, Color::Blue, 10),
    ]);
    println!("La suite [8, Joker, 10] est valide ? {}", suite.is_valid());
    println!("Points de cette suite : {}", suite.points()); // 8 + 9 + 10 = 27

    // 3. Test de la pose initiale (30 points)
    let mut joueur = Joueur::new(
        1,
        "Alice".to_string(),
        vec![
            Tuile::new_standard(1, Color::Blue, 8),
            Tuile::new_joker(2),
            Tuile::new_standard(3, Color::Blue, 10),
            Tuile::new_standard(4, Color::Red, 7),
            Tuile::new_standard(5, Color::Black, 7),
            Tuile::new_standard(6, Color::Yellow, 7),
        ],
    );

    let groupe = Combinaison::new(vec![
        Tuile::new_standard(4, Color::Red, 7),
        Tuile::new_standard(5, Color::Black, 7),
        Tuile::new_standard(6, Color::Yellow, 7),
    ]); // 7 * 3 = 21 points

    // Total = 27 + 21 = 48 points (>= 30)
    match joueur.play_initial_meld(&[suite, groupe]) {
        Ok(pts) => println!("Pose initiale validée avec succès avec {} points !", pts),
        Err(err) => println!("Erreur : {}", err),
    }

    println!("Nombre de tuiles restantes en main : {}", joueur.main.len());

    // 4. Partie à 2 joueurs (mains imposées pour une démo déterministe)
    let mut partie = Partie::new(vec!["Alice".to_string(), "Bob".to_string()])
        .expect("2 joueurs est une configuration valide");

    // Alice reçoit un groupe de 4 dizaines et une tuile supplémentaire
    partie.joueurs[0].main = vec![
        Tuile::new_standard(200, Color::Red, 10),
        Tuile::new_standard(201, Color::Blue, 10),
        Tuile::new_standard(202, Color::Yellow, 10),
        Tuile::new_standard(203, Color::Black, 10),
        Tuile::new_standard(204, Color::Red, 3),
    ];

    partie.start_turn();
    let groupe = Combinaison::new(vec![
        Tuile::new_standard(200, Color::Red, 10),
        Tuile::new_standard(201, Color::Blue, 10),
        Tuile::new_standard(202, Color::Yellow, 10),
        Tuile::new_standard(203, Color::Black, 10),
    ]);
    match partie.play_melds(vec![groupe]) {
        Ok(points) => println!("Alice : pose initiale validée avec {} points !", points),
        Err(err) => println!("Erreur : {}", err),
    }
    partie.end_turn().expect("le plateau reste valide");

    // Tour de Bob : manipulation refusée avant sa pose initiale, il pioche
    partie.start_turn();
    match partie.submit_board(partie.plateau.combinaisons.clone()) {
        Ok(()) => println!("Bob : manipulation acceptée"),
        Err(err) => println!("Bob : manipulation refusée ({})", err),
    }
    match partie.draw_tile() {
        Ok(Some(tuile)) => println!(
            "Bob pioche la tuile ID {} ({} tuiles restent en pioche).",
            tuile.id,
            partie.pioche.len()
        ),
        Ok(None) => println!("Bob passe son tour : la pioche est vide."),
        Err(err) => println!("Erreur : {}", err),
    }

    match partie.winner() {
        Some(id) => println!("{} a gagné !", partie.joueurs[id].nom),
        None => println!("Pas encore de gagnant."),
    }
}

#[cfg(test)]
mod tests;
