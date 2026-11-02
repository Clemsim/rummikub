# ADR 0006 : Élargissement progressif dans l'ISMCTS

- Statut : accepté
- Date : 2026-11-02

## Contexte

Le facteur de branchement du Rummikub est énorme (100-500 coups légaux par
nœud, cf. ADR 0002) : l'expansion aléatoire laissait l'arbre s'éparpiller —
des centaines d'enfants visités une ou deux fois, estimations bruitées.
Deux variantes étaient en balance (pistes de l'ADR 0003) : la formule
α·N^β et le seuil de points statique. La formule a été retenue : elle dose
dynamiquement — le coup d'investissement hors du top initial peut entrer plus
tard, quand le nœud accumule des visites — là où un seuil statique bannirait
pour toujours. Le seuil avait de plus un piège propre à notre jeu : les
manipulations valent 0 point, un filtre « points ≥ X » les aurait bannies
d'office.

## Décision

- Un nœud visité N fois n'admet que k = α·N^β enfants
  (`alpha_elargissement` = 1.0, `beta_elargissement` = 0.5 par défaut) ;
  k est toujours ≥ 1 : la première visite développe le meilleur coup.
- L'expansion ne tire plus au hasard : elle développe **le premier coup
  légal non développé de la liste triée** (points décroissants) — le
  meilleur candidat restant, choix déterministe.
- Conséquence assumée : le flux aléatoire de la recherche change (l'ordre
  d'expansion a changé) — les scores de tournoi ne sont pas comparables à
  ceux des versions antérieures à graine égale.

## Conséquences

- Le même budget de simulations se concentre sur les meilleurs coups.
- Durée mesurée inchangée (1,0 s contre 0,98 s sur 5 graines × 50
  simulations) ; l'effet sur la force exige des tournois plus larges que
  5 graines — le banc et son CSV sont prêts.
- Le risque documenté (coup génique hors top-k) reste réel aux β faibles :
  la valeur de β se règle au banc, comme epsilon.