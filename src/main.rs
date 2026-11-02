// Démo du moteur de jeu : la logique vit dans la bibliothèque (src/lib.rs).
use rummikub_core::{
    combinaison::Combinaison,
    joueur::Joueur,
    partie::Partie,
    tuile::{Couleur, Tuile, generer_pioche, melanger_pioche},
};

fn main() {
    // 1. Génération & Mélange de la pioche
    let mut pioche = generer_pioche();
    println!("Sac généré avec {} tuiles.", pioche.len());
    melanger_pioche(&mut pioche);

    // 2. Test d'une suite avec Joker
    // Exemple : Joker, 8 Bleu, 10 Bleu (Le joker prend la place du 9 Bleu -> suite 8, 9, 10)
    let suite = Combinaison::nouvelle(vec![
        Tuile::nouvelle_standard(1, Couleur::Bleu, 8),
        Tuile::nouveau_joker(2),
        Tuile::nouvelle_standard(3, Couleur::Bleu, 10),
    ]);
    println!(
        "La suite [8, Joker, 10] est valide ? {}",
        suite.est_valide()
    );
    println!("Points de cette suite : {}", suite.points()); // 8 + 9 + 10 = 27

    // 3. Test de la pose initiale (30 points)
    let mut joueur = Joueur::nouveau(
        1,
        "Alice".to_string(),
        vec![
            Tuile::nouvelle_standard(1, Couleur::Bleu, 8),
            Tuile::nouveau_joker(2),
            Tuile::nouvelle_standard(3, Couleur::Bleu, 10),
            Tuile::nouvelle_standard(4, Couleur::Rouge, 7),
            Tuile::nouvelle_standard(5, Couleur::Noir, 7),
            Tuile::nouvelle_standard(6, Couleur::Jaune, 7),
        ],
    );

    let groupe = Combinaison::nouvelle(vec![
        Tuile::nouvelle_standard(4, Couleur::Rouge, 7),
        Tuile::nouvelle_standard(5, Couleur::Noir, 7),
        Tuile::nouvelle_standard(6, Couleur::Jaune, 7),
    ]); // 7 * 3 = 21 points

    // Total = 27 + 21 = 48 points (>= 30)
    match joueur.jouer_pose_initiale(&[suite, groupe]) {
        Ok(pts) => println!("Pose initiale validée avec succès avec {} points !", pts),
        Err(err) => println!("Erreur : {}", err),
    }

    println!("Nombre de tuiles restantes en main : {}", joueur.main.len());

    // 4. Partie à 2 joueurs (mains imposées pour une démo déterministe)
    let mut partie = Partie::nouvelle(vec!["Alice".to_string(), "Bob".to_string()])
        .expect("2 joueurs est une configuration valide");

    // Alice reçoit un groupe de 4 dizaines et une tuile supplémentaire
    partie.joueurs[0].main = vec![
        Tuile::nouvelle_standard(200, Couleur::Rouge, 10),
        Tuile::nouvelle_standard(201, Couleur::Bleu, 10),
        Tuile::nouvelle_standard(202, Couleur::Jaune, 10),
        Tuile::nouvelle_standard(203, Couleur::Noir, 10),
        Tuile::nouvelle_standard(204, Couleur::Rouge, 3),
    ];

    partie.ouvrir_tour();
    let groupe = Combinaison::nouvelle(vec![
        Tuile::nouvelle_standard(200, Couleur::Rouge, 10),
        Tuile::nouvelle_standard(201, Couleur::Bleu, 10),
        Tuile::nouvelle_standard(202, Couleur::Jaune, 10),
        Tuile::nouvelle_standard(203, Couleur::Noir, 10),
    ]);
    match partie.poser_combinaisons(vec![groupe]) {
        Ok(points) => println!("Alice : pose initiale validée avec {} points !", points),
        Err(err) => println!("Erreur : {}", err),
    }
    partie.terminer_tour().expect("le plateau reste valide");

    // Tour de Bob : manipulation refusée avant sa pose initiale, il pioche
    partie.ouvrir_tour();
    match partie.soumettre_plateau(partie.plateau.combinaisons.clone()) {
        Ok(()) => println!("Bob : manipulation acceptée"),
        Err(err) => println!("Bob : manipulation refusée ({})", err),
    }
    match partie.piocher() {
        Ok(Some(tuile)) => println!(
            "Bob pioche la tuile ID {} ({} tuiles restent en pioche).",
            tuile.id,
            partie.pioche.len()
        ),
        Ok(None) => println!("Bob passe son tour : la pioche est vide."),
        Err(err) => println!("Erreur : {}", err),
    }

    match partie.gagnant() {
        Some(id) => println!("{} a gagné !", partie.joueurs[id].nom),
        None => println!("Pas encore de gagnant."),
    }
}
