# ADR 0008 : Décodage de la politique en coups légaux (joueur-réseau)

- Statut : accepté
- Date : 2026-11-03

## Contexte

Le réseau de politique (ADR 0007) sort 54 probabilités indépendantes : les 53
types de tuiles + le drapeau piocher. Une partie se joue en coups légaux
(`Poser`, `Manipuler`, `Piocher`, `Terminer`), pas en drapeaux. Pour mesurer la
force du réseau entraîné, il fallait choisir comment convertir les probabilités
en coups.

Trois décodeurs étaient envisageables :
1. **Argmax sur les types, glouton** : jouer les types dont la probabilité
   dépasse 0,5, en cherchant une pose légale qui les réalise — se raccroche mal
   aux coups réellement légaux (les types choisis ne forment pas toujours une
   pose possible).
2. **Score par coup** : évaluer chaque coup légal et prendre le meilleur.
3. **Échantillonnage** : tirer un coup selon les probabilités — utile pour la
   diversité des données, mais une mesure de force non reproductible.

## Décision

Le joueur-réseau décode **par score de log-vraisemblance** (option 2), glouton :

- À chaque décision, l'état est encodé par le même encodeur que le générateur,
  et `coups_possibles` énumère les coups légaux.
- Chaque coup reçoit le score **log-vraisemblance de son vecteur d'étiquettes**
  (types joués + drapeau piocher, au sens de l'ADR 0007) sous les 54
  probabilités du réseau. C'est la définition que la BCE optimise : le coup
  choisi est celui dont l'étiquette est la plus cohérente avec les croyances
  du réseau.
- Pour les coups non-pioche, les termes des types non joués sont identiques
  d'un coup à l'autre : le classement se réduit à maximiser la somme des logits
  des types effectivement joués — un coup gagne en lisant les types « décidés
  vrais ».
- `Terminer` n'a pas de cible (ADR 0007) : il n'est jamais proposé ; le pilote
  clôt le tour quand le coup choisi est `Piocher` ou que la main est vide.
- Ex æquo (deux poses de types identiques) : le premier de l'ordre
  déterministe de `coups_possibles` gagne (ADR 0001).
- Le choix est reproductible : même réseau + même état → même coup.

## Conséquences

- Le réseau ne produit jamais de coup illégal : les règles filtrent, le réseau
  trie.
- La mesure de force (`joue`) est reproductible : le tournoi réutilise le cadre
  de `rummikub_ia::tournoi` (graines 1..=N, mains à 6 tuiles, limite 300 tours).
- Un étalonnage du décodage (sampling, températures, décodage contraint par les
  valeurs au-delà des types) reste possible plus tard sans remettre en cause
  l'entraînement.