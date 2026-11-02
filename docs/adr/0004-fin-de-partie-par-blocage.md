# ADR 0004 : Fin de partie par blocage

- Statut : accepté
- Date : 2026-11-02

## Contexte

Découvert par le banc d'essai : une partie peut atteindre un état où le sac
est vide et où plus personne ne tient la moindre combinaison jouable (vérifié
sur une main réelle de 19 tuiles). Sans règle de fin, les deux joueurs passent
indéfiniment : les parties ne se terminent jamais, et les rollouts de
l'ISMCTS errent jusqu'à leur profondeur maximale.

## Décision

Règle officielle du Rummikub : la partie se termine quand le sac est vide et
qu'un tour complet s'écoule sans la moindre pose. Le joueur avec la valeur de
main la plus faible (joker resté en main = 30) gagne ; en cas d'égalité, la
partie est nulle (`gagnant()` retourne None).

- `Partie.tours_sans_pose` compte les tours consécutifs sans pose (remis à
  zéro dès qu'un joueur pose ou manipule).
- `Partie.est_terminee()` : sac vide + compteur ≥ nombre de joueurs.
- `gagnant()` : victoire par main vide d'abord, puis blocage, sinon None.

## Conséquences

- Toutes les parties se terminent : le banc d'essai produit des scores
  exploitables, et les rollouts de la recherche résolvent les blocages en
  une étape (gain de vitesse appréciable).
- Une partie nulle reste possible en cas d'égalité parfaite des valeurs de
  main.
