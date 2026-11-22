# ORD — L'ordonnancement

## Définitions

Une **donnée** est un composant, une ressource ou un canal de messages.

Un système **écrit** une donnée s'il déclare :
- `write` sur un composant ou une ressource ;
- l'activation ou la désactivation (certaine ou possible) d'un composant activable ;
- `emit` ou `may_emit` sur un canal de messages.

Un système **lit** une donnée s'il déclare :
- `read` ou `optional` sur un composant ou une ressource, ou un accès `via` sur un composant ;
- `read` sur un canal de messages ;
- `with` ou `without` sur un composant **activable**, car sa sélection dépend alors de l'état
  activé ou désactivé du composant.

Les accès `with` et `without` sur un composant non activable ne sont ni des lectures ni des
écritures.

Deux systèmes sont **en conflit** si l'un écrit une donnée que l'autre lit ou écrit.

Les observateurs ne sont pas placés dans les étapes : ils s'exécutent aux points de
synchronisation, seuls, et n'entrent donc en conflit avec aucun système.

Deux systèmes sont **disjoints** si l'un déclare `with` sur un composant que l'autre déclare
`without` : aucune entité ne peut être sélectionnée par les deux à la fois.

## Exigences

**ORD-001** — ECS NIDE doit calculer un plan d'exécution qui place chaque système activé dans
exactement une étape de la boucle.

**ORD-002** — Sauf dans le cas prévu par ORD-013, ECS NIDE ne doit jamais placer dans la même
étape deux systèmes qui écrivent la même donnée.

**ORD-003** — Sauf dans le cas prévu par ORD-013, ECS NIDE ne doit jamais placer dans la même
étape un système qui écrit une donnée et un système qui la lit.

**ORD-004** — ECS NIDE doit placer chaque système dans la première étape compatible avec ses
conflits, ses contraintes d'ordre et sa phase, en traitant les systèmes dans l'ordre résolu
(contraintes d'ordre explicites et implicites, phases, puis ordre de référence de
`schedule.yaml`).

**ORD-005** — Quand un système déclare devoir s'exécuter après (ou avant) un autre, ECS NIDE doit
le placer dans une étape strictement postérieure (ou antérieure) à celle de cet autre système.

**ORD-006** — Si les contraintes d'ordre, explicites ou implicites, forment un cycle, alors
ECS NIDE doit signaler une erreur qui nomme les systèmes du cycle dans l'ordre, et ne produire
aucun plan.

**ORD-007** — Quand deux systèmes en conflit n'ont entre eux aucune contrainte d'ordre, directe
ou transitive, ECS NIDE doit signaler une ambiguïté et les ordonner selon l'ordre de référence.

**ORD-008** — ECS NIDE doit produire le même plan d'exécution chaque fois qu'il est calculé pour
un même projet.

**ORD-009** — ECS NIDE doit appliquer les changements structurels d'un système (ajouts, retraits,
créations, destructions) à la fin de l'étape dans laquelle il s'exécute.

**ORD-010** — ECS NIDE doit exclure du plan d'exécution les systèmes désactivés dans le projet.

**ORD-011** — Quand ECS NIDE calcule le plan d'exécution, il doit l'écrire dans
`schedule.plan.yaml`, avec ses étapes, leurs systèmes, la phase et la fréquence de chaque système,
le profil de moteur utilisé et les ambiguïtés signalées.

**ORD-012** — Quand des phases sont déclarées, ECS NIDE doit placer chaque système dans une étape
strictement postérieure aux étapes de tous les systèmes des phases précédentes.

**ORD-013** — Là où le profil du moteur cible déclare la prise en charge des ensembles disjoints,
ECS NIDE doit considérer que deux systèmes disjoints ne sont pas en conflit.

**ORD-014** — Là où le profil du moteur cible déclare qu'il n'exécute pas de systèmes en
parallèle, ECS NIDE doit placer chaque système seul dans son étape, dans l'ordre résolu.

**ORD-015** — ECS NIDE doit traiter chaque couple formé d'un système qui émet sur un canal de
messages et d'un système qui lit ce canal comme une contrainte d'ordre implicite : l'émetteur
avant le lecteur.

**ORD-016** — ECS NIDE doit exclure les observateurs des étapes du plan, et indiquer dans
`schedule.plan.yaml`, pour chaque étape, les observateurs que ses systèmes peuvent déclencher,
directement ou en cascade.

**ORD-017** — Si les déclenchements possibles entre observateurs forment un cycle, alors ECS NIDE
doit signaler un avertissement qui nomme les observateurs du cycle dans l'ordre.
