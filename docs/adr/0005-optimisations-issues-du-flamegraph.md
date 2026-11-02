# ADR 0005 : Optimisations issues du flamegraph

- Statut : accepté
- Date : 2026-11-02

## Contexte

Le flamegraph du tournoi (constats dans l'ADR 0003) désignait trois postes :
l'allocation mémoire (~5%), le hachage des identifiants de tuiles dans des
`HashSet<u8>` (~2-3%), et des `sched_yield` attribués à la surcharge rayon à
petit budget de simulations.

## Décisions

1. **`BitsetTuiles`** (dans `tuile.rs`) : un bitmap de 4 × u64 (32 octets sur
   la pile, tout identifiant u8) remplace les `HashSet<u8>` d'identifiants
   dans `verifier_proposition`, `soumettre_plateau`, `plateau.ids_tuiles`,
   `poses_initiales` et la déterminisation de l'ISMCTS. Opérations
   ensemblistes bit-à-bit (`contient`, `inserer_si_absent`,
   `retirer_si_present`, `disjoint`, `inserer_tout`, `ids`), zéro hachage,
   zéro allocation.
2. **Seuil de parallélisation** : rayon n'est lancé que si chaque thread
   aurait au moins `SIMULATIONS_MIN_PAR_THREAD` simulations. Voir la saga de
   mesure ci-dessous.
3. **Clones évitables** : dans les rollouts et l'expansion, les coups sont
   consommés (`swap_remove`) au lieu d'être clonés ; `poses_initiales`
   partitionne d'abord et consomme les poses simples de 30+ points. Les
   clones du plateau dans les manipulations restent inhérents à la
   conception (chaque candidat est un état complet possédé) — la vraie
   suite serait `SmallVec` pour les tuiles de combinaison, ou
   l'élargissement progressif (ADR 0003) pour réduire le nombre de
   candidats.

## La saga de mesure (leçon centrale)

Référence : 5 graines × 50 simulations, release (`banc/resultats.csv`) :

| Réglage | Durée | Verdict |
|---|---|---|
| Référence (HashSet, threads auto) | 1,32 s | — |
| Seuil 32 sims/thread (→ séquentiel) | 6,6 s | **réfuté** : le parallélisme valait le coup |
| Seuil 8 sims/thread (→ 6 threads) | 1,67 s | trop haut : bride la machine |
| Seuil 4 sims/thread (→ ~12 threads) | 0,98 s | **validé**, ~26% plus rapide que la référence |

Deux leçons :
- la surcharge de coordination rayon, visible dans le flamegraph, ne coûte
  rien de significatif à cette échelle — plus de threads = mieux, même à
  quelques simulations par thread ;
- toute intuition d'optimisation se mesure avec le banc avant d'être
  conservée.

## Conséquences

- Le flux aléatoire de la recherche dépend du découpage des simulations
  entre threads : la décision est reproductible à (graine, threads,
  simulations) fixés ; les tests épinent `threads: 1`. Changer le seuil
  change le flux (et donc le style de jeu d'un bout à l'autre du tournoi).
- Les identifiants de démonstration au-delà de 105 (mains imposées) restent
  valables : le bitset couvre tout u8.
- La mesure initiale de l'ISMCTS contre l'heuristique (100% à 5 graines)
  reflétait un flux aléatoire particulier ; les flux suivants donnent 60-80%.
  Les comparaisons de force exigent plus de graines que ça.
