# ADR 0009 : Entraînement par renforcement en auto-partie (REINFORCE)

- Statut : accepté
- Date : 2026-11-03

## Contexte

L'imitation (ADR 0007) est plafonnée par le professeur : le réseau entraîné
s'approche de la politique de l'ISMCTS (~63 % de match exact) sans pouvoir la
dépasser. Pour chercher plus de force, il faut que le réseau apprenne de ses
propres résultats : l'auto-apprentissage par renforcement, sur la base de la
politique déjà entraînée.

## Décision

L'entraînement par renforcement est un **REINFORCE en auto-partie** :

1. **Auto-parties échantillonnées** : le réseau joue contre lui-même. À chaque
   décision, un coup légal est tiré selon la distribution
   softmax(score/tau) des scores du décodeur (ADR 0008). `tau` règle
   l'exploration ; le choix glouton de la mesure n'est pas affecté.
2. **Récompenses terminales** : +1 victoire, −1 défaite, 0 si la partie est
   bloquée dans la limite de tours. Les deux joueurs apprennent : chaque
   décision rapporte la récompense de son propre joueur (jeu à somme nulle).
3. **Mise à jour** : REINFORCE — remonter la log-probabilité des coups des
   parties gagnées, abaisser celle des parties perdues, avec pour baseline la
   moyenne des récompenses du cycle. La log-prob du coup choisi est
   exactement l'opposé de la somme des BCE des 54 sorties avec l'étiquette du
   coup (ADR 0007) : la même loss que l'imitation, pondérée par l'avantage.
   La BCE est écrite à la main (identité softplus stable), la sigmoïde
   étant appliquée dans la loss et non dans le réseau.
4. **Sélection du meilleur réseau** : un tournoi de 20 graines contre
   ISMCTS-300 toutes les `evaluations_tous` cycles ; le réseau au taux maximal
   est conservé. Sans évaluation configurée, le réseau final est retenu.
5. **Réglages par défaut** : lr 1e-5 (la loss somme les 54 sorties au lieu
   d'en faire la moyenne — gradients ~54× plus grands que l'imitation),
   lots de 512 décisions, tau 5, graine fixe pour tout ce qui est aléatoire.

## Conséquences

- Le gradient de politique part d'un comportement déjà bon (init par
  imitation) : les premières auto-parties restent du Rummikub jouable.
- L'échantillonnage rend l'entraînement exploratoire ; la mesure reste
  gloutonne et reproductible (ADR 0008).
- Détail d'implémentation : le backend flex de burn porte son RNG dans un
  statique global partagé entre threads. Les exécutions CLI (un seul thread)
  restent déterministes ; les tests parallèles ne doivent pas comparer des
  poids dépendant de ce flux.
- Extension naturelle si REINFORCE stalle : réseau de valeur (actor-critic)
  ou adversaires figés plus forts, mais l'évaluation contre ISMCTS-300
  suffit pour mesurer le palier « > 50 % » visé.