# TDI-22 — Budget de parcours hiérarchique et repli exact V1

Statut : **module de recherche Rust expérimental, non confirmatoire**.
Ne touche ni aux fichiers gelés de TDI-22.0, ni aux protocoles de
confirmation, ni aux routes de production FLAT-ATTENTION.

## Problème visé

Un arbre binaire hiérarchique de torsions peut agréger une grande quantité
de clés lorsque les scores sont proches. Sur des clés difficiles à
agréger, la traversée complète atteint au pire un ordre O(N) et peut
coûter davantage que le streaming exact. La qualification V1 sur un
panneau lisse donnait une hiérarchie exacte moins rapide que le dense ;
ce constat particulier ne permet pas d'affirmer que toute hiérarchie
est plus lente.

## Nouveau contrat logiciel

Identité du contrat :
`tdi22-visit-limited-exact-fallback-v1`.

`HierarchicalTorsorBank::row_with_visit_budget` reçoit un nombre
maximal de nœuds visités, y compris ceux exclus par masque causal.
La traversée ne produit aucune sortie partielle si elle ne peut pas
être achevée avant la limite. Elle renvoie l'erreur typée
`HierarchyError::VisitBudgetExceeded`. Les validations de l'échelle,
de la tolérance et de l'arithmétique non finie restent indépendantes du
budget : une entrée invalide ne doit jamais être transformée en
résultat accepté.

`GuardedTorsorBank` prépare à partir des **mêmes clés ordonnées**
deux formes internes : arbre pour la tentative hiérarchique, clés
origin-réduites pour le streaming de référence.
`GuardedTorsorBank::row` tente la hiérarchie avec le budget déclaré,
puis, uniquement après `VisitBudgetExceeded`, recalcule
la ligne **entière** avec `stream_torsor_row`. Les sommes partielles
de l'arbre ne sont pas réutilisées. Le résultat signale explicitement
l'itinéraire via `GuardedPath`, la limite demandée, ainsi que les
statistiques et bornes de la hiérarchie uniquement lorsqu'elle a terminé.

Le cas budget 0 force volontairement la voie exacte. À budget
suffisant, la sortie et la borne de la hiérarchie restent inchangées.

## Coût et limites

Cette réalisation optimise d'abord la sûreté de décision et la
lisibilité des chemins. La préparation de l'arbre et des clés de
streaming **est actuellement répétée**, avec stockage simultané de
deux représentations. Un échec du budget signifie que quelques nœuds
ont déjà été parcourus avant de payer le coût O(N) du streaming exact.
Il n'existe donc pas de démonstration de gain de temps universel :
le budget limite uniquement le travail *hiérarchique*, pas le coût
total d'une ligne avec repli.

Les garanties d'erreur sur les blocs acceptés sont celles de
TDI-22-HIERARCHICAL-ENVELOPE-V1.md, sous arithmétique réelle.
Le streaming de repli conserve l'algèbre exacte du softmax et le
masque causal ; il peut présenter des différences d'arrondi binaire64
face à un calcul direct Varignon. Aucune revendication sur la
qualité d'un modèle réel, les GPU, la bande passante ou le trafic DRAM
n'est autorisée.

## Qualification unitaire et CI

- budget nul : identité bit à bit avec le streaming factorisé existant,
  pour des masques Full et Causal ;
- budget large : identité avec la hiérarchie originale, sortie, bornes
  et compteurs ;
- clés identiques : un seul nœud accepté pour une ligne Full ;
- clés opposées alternées : refus de l'agrégation et repli exact avec
  un budget de 8 visites ;
- échelle/tolérance invalides : erreur explicite, jamais repli validé.

Commandes exécutables dans le dépôt :

    cargo test -p tdi-ai --features experimental tdi22_guarded_attention
    cargo test -p tdi-ai --features experimental tdi22_hierarchical_attention
    cargo fmt --all -- --check
    bash scripts/check-tdi22-bootstrap.sh
    bash scripts/check-tdi22-freeze.sh

La progression vers une politique de sélection automatique demanderait
une calibration non confirmatoire des coûts de visite, des tailles,
des distributions, des points d'évaluation et de la mémoire. Le
budget actuel est un **paramètre explicite de l'appelant**, jamais une
auto-tuning silencieuse sur des résultats confirmatoires.
