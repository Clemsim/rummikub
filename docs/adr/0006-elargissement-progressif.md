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

## Mesures au banc (2026-11-02, 20 graines × 50 simulations, mains tronquées à 6)

Balayage de β (ε au défaut de 0.2) — `banc/balayage-beta.csv`,
`banc/graphique-beta.png` :

| β | 0.1 | 0.2 | 0.3 | 0.5 | 0.7 | 1.0 |
|---|---|---|---|---|---|---|
| Victoires | 95% | 95% | 95% | 55% | 15% | **0%** |

- **Plateau net à β ≤ 0.3, puis effondrement** : le défaut initial (0.5)
  était dans la pente, il est corrigé à **0.2** (milieu du plateau).
- **β = 1.0 (aucun élargissement) perd 20 parties sur 20** : à 50
  simulations, l'arbre non dosé s'éparpille sur des centaines d'enfants et
  les décisions deviennent quasi aléatoires. L'élargissement n'est pas une
  raffinement ici, c'est ce qui rend la recherche jouable.
- Balayage d'epsilon **re-fait au nouveau β** (`banc/balayage-epsilon.csv`) :
  courbe plate à 95% sur tout le segment ε ∈ [0, 1] — une fois l'arbre bien
  concentré, la qualité des rollouts devient secondaire. ε garde son défaut
  de 0.2 (réglage devenu secondaire).

## Conséquences

- Le même budget de simulations se concentre sur les meilleurs coups — et
  la mesure montre que c'est le facteur dominant, devant ε et les rollouts.
- Les valeurs mesurées dépendent du budget (50 simulations) et du cadre du
  banc (mains tronquées) : à budget plus large, un β plus grand pourrait
  redevenir meilleur — re-balayer si le budget change.
- Le risque documenté (coup génique hors top-k) reste réel aux β faibles :
  le plateau observé (0.1 aussi bon que 0.3) suggère une marge, mais la
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