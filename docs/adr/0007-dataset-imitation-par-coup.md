# ADR 0007 : Cibles de supervision au niveau du coup

- Statut : accepté
- Date : 2026-11-03

## Contexte

Pour entraîner un réseau par imitation (behavioural cloning) du professeur
ISMCTS, deux granularités d'étiquetage étaient envisageables :
1. **Au niveau du tour** : prédire l'ensemble des tuiles déposées sur
   l'ensemble du tour (macro-action).
2. **Au niveau du coup** : superviser chaque décision élémentaire prise
   pendant le tour (pose, manipulation, pioche).

## Décision

Les exemples d'apprentissage sont produits **au niveau de chaque coup élémentaire** :

- Chaque transition (état encodé → décision) devient un échantillon.
- **Pose** : étiquetage multi-label des types de tuiles déposées (53 classes).
- **Manipulation** : seules les tuiles **nouvellement apportées depuis la main**
  sont étiquetées (`ids_plateau.contient(...) == false`). Les tuiles déjà présentes
  sur la table ne font pas partie de la décision d'apport.
- **Piocher** : drapeau binaire dédié (`piocher = true`, types à faux).
- **Terminer** : ne produit aucun exemple d'apprentissage. Le réseau n'a pas à
  décider d'appeler `Terminer` : c'est le pilote du bot qui clôt le tour dès
  qu'aucune pose ou manipulation légale proposée par le réseau n'est réalisable.
- Le générateur pilote lui-même la boucle de tour via l'API publique (`Partie`,
  `choisir_coup`, `jouer_coup`), sans instrumenter ni modifier l'ISMCTS.

## Conséquences

- Supervision plus granulaire : le réseau apprend la politique de transition
  état par état, ce qui permet un chaînage naturel de plusieurs manipulations
  ou poses au sein d'un même tour.
- Indépendance stricte : le réseau suggère les types de tuiles à jouer, les règles
  du moteur (`rummikub-core`) valident et exécutent.
