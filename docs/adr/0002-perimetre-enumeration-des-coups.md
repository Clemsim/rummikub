# ADR 0002 : Périmètre de l'énumération des coups

- Statut : accepté
- Date : 2026-11-02

## Contexte

L'espace de coups complet du Rummikub est immense : poses de plusieurs
combinaisons à la fois, manipulations arbitraires du plateau (découper,
recoller, déplacer des jokers...). L'énumération est appelée des milliers de
fois par les IA de recherche : elle doit rester rapide tout en couvrant les
coups qui comptent, et chaque coup énuméré doit être accepté par les règles
(`jouer_coup` passe par `poser_combinaisons`, `soumettre_plateau`,
`terminer_tour`, `piocher`).

## Décision

- **Pose initiale** : combinaisons valant 30 points ou plus à elles seules,
  plus les paires de combinaisons disjointes dont la somme atteint 30. Les
  paires où une combinaison vaut déjà 30+ sont exclues : la pose simple
  existe, et poser en deux temps dans le même tour est équivalent.
- **Après la pose initiale** : chaque combinaison candidate est une pose ;
  les poses multi-combinaisons se font en enchaînant plusieurs coups dans le
  même tour (l'IA peut re-décider entre chaque pose).
- **Manipulations en une étape** : ajout d'une tuile de la main à une
  combinaison du plateau ; libération du joker d'un groupe (remplacé par la
  vraie tuile venue de la main, le joker rejoint une autre combinaison). Les
  chaînes de manipulations se font en enchaînant plusieurs coups.
- **Fin de tour** : `Coup::Terminer` est légal après avoir joué quelque
  chose, ou à pioche vide ; `Coup::Piocher` est toujours disponible.

## Limites connues (à lever si le besoin s'en fait sentir)

- Trois petites combinaisons pour la pose initiale (ex. 10 + 10 + 10) ne
  sont pas énumérées.
- Libérer le joker d'une suite n'est pas énuméré : sa valeur y est une
  plage, pas un point précis.
- Les manipulations combinant deux changements hors libération de joker ne
  sont pas énumérées d'un seul coup ; elles passent par des chaînes de
  plusieurs coups.

## Conséquences

- Énumération en fractions de milliseconde, exploitable par la recherche.
- Quelques coups rares restent hors de portée des IA sans réviser cet ADR ;
  les humains, eux, passent toujours par l'API complète du core.
