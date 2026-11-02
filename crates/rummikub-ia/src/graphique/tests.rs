use super::*;

#[test]
fn test_la_courbe_epsilon_est_ecrite_en_png() {
    let chemin = std::env::temp_dir().join("courbe-test-rummikub.png");
    courbe(
        &[(0.0, 0.5), (0.5, 0.8), (1.0, 0.6)],
        "Taux de victoire de l'ISMCTS selon epsilon",
        "epsilon (probabilité de coup aléatoire en rollout)",
        chemin.to_string_lossy().as_ref(),
    )
    .unwrap();

    // Un PNG réel fait largement plus de 1 Ko
    let taille = std::fs::metadata(&chemin).unwrap().len();
    assert!(taille > 1000);
    let _ = std::fs::remove_file(&chemin);
}
