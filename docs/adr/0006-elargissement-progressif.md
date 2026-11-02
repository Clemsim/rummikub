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

## Mesures au banc (2026-11-02, 20 graines, mains tronquées à 6)

> Note d'outillage : le premier balayage de β portait mal le budget — il
> tournait à 300 simulations (défaut) au lieu des 50 annoncées, et ses
> chiffres étaient incohérents avec la cartographie 2D. Le bug (les
> balayages 1D ignoraient l'argument `simulations`) est corrigé ; les
> chiffres ci-dessous sont les mesures corrigées.

**β à 50 simulations** (ε au défaut) — `banc/balayage-beta.csv` :

| β | 0.1 | 0.2 | 0.3 | 0.5 | 0.7 | 1.0 |
|---|---|---|---|---|---|---|
| Victoires | 95% | 95% | 95% | 95% | 60% | 30% |

**Cartographie 2D ε × β** (42 couples) — `banc/cartographie-epsilon-beta.csv`,
`banc/cartographie-epsilon-beta.png` : pour β ≤ 0.5, la ligne est **plate à
95% sur tout le segment ε ∈ [0, 1]** ; β = 0.7 oscille entre 47% et 65% sans
tendance nette en ε ; β = 1.0 est plat à 30%. La cartographie PNG trace une
courbe par β.

Conclusions :

- **β est le facteur dominant, ε est secondaire** à ce budget : la carte
  est plate en ε partout, et ne varie qu'en β.
- **Le plateau s'étend jusqu'à β = 0.5** à 50 simulations, avec l'effondrement
  attendu au-delà (β = 1.0, l'arbre non dosé : 30%). Le défaut est fixé à
  **0.2**, au milieu du plateau [0.1 ; 0.5].
- Le plafond à ~95% face à l'heuristique est probablement la limite du cadre
  (mains tronquées : certaines parties sont perdantes quelle que soit la
  recherche).
- À budget plus large (plus de simulations), la frontière du plateau se
  déplacera vraisemblablement vers le haut — re-balayer si le budget change.

## Conséquences

- Le même budget de simulations se concentre sur les meilleurs coups — et
  la mesure montre que c'est le facteur dominant, devant ε et les rollouts.
- Le risque documenté (coup génique hors top-k) reste réel aux β faibles :
  le plateau large (0.1 aussi bon que 0.5) suggère une marge, mais la
  frontière exacte se re-valide à chaque changement de budget.

## Le parti pris de l'ordre d'admission

L'élargissement admet les enfants **dans l'ordre du tri par points
immédiats** (`coups_possibles` trie par points décroissants). C'est un parti
pris assumé : il favorise les coups marquants sur les coups d'investissement.
Une manipulation à 0 point n'entre que lorsque k dépasse son rang — tard, et
uniquement sur les nœuds très visités (donc proches de la racine). Le remède
structurel serait un prior différent — donner une valeur proxy aux
manipulations, ou plus tard un réseau de politique — pas un réglage de α/β :
ceux-ci ne changent que la vitesse d'ouverture du nœud, pas l'ordre dans
lequel il s'ouvre.