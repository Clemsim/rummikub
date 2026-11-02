# ADR 0001 : Énumération des coups déterministe

- Statut : accepté
- Date : 2026-11-02

## Contexte

Les IA (bot heuristique, ISMCTS) choisissent un coup en parcourant la liste
produite par `coups_possibles`. En cas d'égalité de points entre deux coups,
l'ordre de la liste départage le choix. Par ailleurs, toute la chaîne
« graine → distribution → énumération → décision » doit être reproductible
pour que les simulations bot contre bot (et plus tard IA contre IA) soient
comparables d'une exécution à l'autre.

Or `HashMap` parcourt ses clés dans un ordre aléatoire à chaque exécution
(randomisation volontaire de la bibliothèque standard) : un parcours d'ordre
indéterminé ferait varier le choix du bot à résultat de main égal, et casserait
les tests de reproductibilité.

## Décision

- L'énumération classe la main exclusivement avec des structures triées
  (`BTreeMap`) : par valeur puis couleur pour les groupes, transposé par
  couleur puis valeur pour les suites.
- Les doublons exacts (deux exemplaires de la même tuile) sont
  interchangeables : un seul exemplaire canonique par couple
  (couleur, valeur) est énuméré.
- Les coups sont triés par points décroissants ; les égalités tombent donc
  dans un ordre stable et défini.

## Conséquences

- Même main → même liste, dans le même ordre, à chaque exécution.
- Coût marginal : O(log n) contre O(1) par accès, négligeable à cette échelle.
- Ne pas introduire de `HashMap` dans le chemin d'énumération sans réviser
  cet ADR.
