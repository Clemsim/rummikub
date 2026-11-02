// Naming français délibéré (Tuile, Tuiles, Type...) : le lint non_snake_case
// est désactivé pour tout le projet, ne pas renommer en anglais.
#![allow(non_snake_case)]

mod meld;
mod player;
mod tuile;

use meld::Meld;
use player::Player;
use tuile::{Color, Tuile, generate_pool, shuffle_pool};

fn main() {
    // 1. Génération & Mélange du pool
    let mut pool = generate_pool();
    println!("Sac généré avec {} tuiles.", pool.len());
    shuffle_pool(&mut pool);

    // 2. Test d'une suite avec Joker
    // Exemple : Joker, 8 Bleu, 10 Bleu (Le joker prend la place du 9 Bleu -> suite 8, 9, 10)
    let run_meld = Meld::new(vec![
        Tuile::new_standard(1, Color::Blue, 8),
        Tuile::new_joker(2),
        Tuile::new_standard(3, Color::Blue, 10),
    ]);
    println!(
        "La suite [8, Joker, 10] est valide ? {}",
        run_meld.is_valid()
    );
    println!("Points de cette suite : {}", run_meld.points()); // 8 + 9 + 10 = 27

    // 3. Test de la pose initiale (30 points)
    let mut player = Player::new(
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

    let group_meld = Meld::new(vec![
        Tuile::new_standard(4, Color::Red, 7),
        Tuile::new_standard(5, Color::Black, 7),
        Tuile::new_standard(6, Color::Yellow, 7),
    ]); // 7 * 3 = 21 points

    // Total = 27 + 21 = 48 points (>= 30)
    match player.play_initial_meld(&[run_meld, group_meld]) {
        Ok(pts) => println!("Pose initiale validée avec succès avec {} points !", pts),
        Err(err) => println!("Erreur : {}", err),
    }

    println!("Nombre de tuiles restantes en main : {}", player.hand.len());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_runs_with_jokers() {
        // [Joker, 2 Rouge, 3 Rouge] -> valide (1, 2, 3 ou 2, 3, 4)
        let m1 = Meld::new(vec![
            Tuile::new_joker(100),
            Tuile::new_standard(1, Color::Red, 2),
            Tuile::new_standard(2, Color::Red, 3),
        ]);
        assert!(m1.is_valid_run());

        // [12 Noir, 13 Noir, Joker] -> le joker ne peut être que 11 (car > 13 interdit)
        let m2 = Meld::new(vec![
            Tuile::new_standard(3, Color::Black, 12),
            Tuile::new_standard(4, Color::Black, 13),
            Tuile::new_joker(101),
        ]);
        assert!(m2.is_valid_run());
        assert_eq!(m2.points(), 11 + 12 + 13);

        // [1 Rouge, 2 Rouge, Joker avec valeur 14 impossible]
        let m3 = Meld::new(vec![
            Tuile::new_standard(5, Color::Red, 1),
            Tuile::new_standard(6, Color::Red, 13),
            Tuile::new_joker(102),
        ]);
        assert!(!m3.is_valid_run()); // Écart trop grand pour 1 seul joker
    }

    #[test]
    fn test_initial_meld_under_30_fails() {
        let mut player = Player::new(
            1,
            "Bob".to_string(),
            vec![
                Tuile::new_standard(1, Color::Red, 1),
                Tuile::new_standard(2, Color::Blue, 1),
                Tuile::new_standard(3, Color::Yellow, 1),
            ],
        );

        let meld = Meld::new(player.hand.clone()); // 3 points seulement
        assert!(player.play_initial_meld(&[meld]).is_err());
        assert!(!player.has_completed_initial_meld);
    }
}
