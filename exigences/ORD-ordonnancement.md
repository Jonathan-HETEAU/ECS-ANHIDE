# ORD — L'ordonnancement

Définition : deux systèmes sont **en conflit** si l'un déclare en `write` un composant ou une
ressource que l'autre déclare en `read`, `write` ou `optional`. Les accès `with` et `without` ne
créent pas de conflit.

**ORD-001** — ECS NIDE doit calculer un plan d'exécution qui place chaque système activé dans
exactement une étape de la boucle.

**ORD-002** — ECS NIDE ne doit jamais placer dans la même étape deux systèmes qui déclarent en
`write` le même composant ou la même ressource.

**ORD-003** — ECS NIDE ne doit jamais placer dans la même étape un système qui déclare en `write`
un composant ou une ressource et un système qui le déclare en `read` ou en `optional`.

**ORD-004** — ECS NIDE doit placer chaque système dans la première étape compatible avec ses
conflits et ses contraintes d'ordre, en traitant les systèmes dans l'ordre résolu (contraintes
d'ordre, puis ordre de référence de `schedule.yaml`).

**ORD-005** — Quand un système déclare devoir s'exécuter après (ou avant) un autre, ECS NIDE doit
le placer dans une étape strictement postérieure (ou antérieure) à celle de cet autre système.

**ORD-006** — Si les contraintes d'ordre forment un cycle, alors ECS NIDE doit signaler une erreur
qui nomme les systèmes du cycle dans l'ordre, et ne produire aucun plan.

**ORD-007** — Quand deux systèmes en conflit n'ont entre eux aucune contrainte d'ordre, directe
ou transitive, ECS NIDE doit signaler une ambiguïté et les ordonner selon l'ordre de référence.

**ORD-008** — ECS NIDE doit produire le même plan d'exécution chaque fois qu'il est calculé pour
un même projet.

**ORD-009** — ECS NIDE doit appliquer les changements structurels d'un système à la fin de l'étape
dans laquelle il s'exécute.

**ORD-010** — ECS NIDE doit exclure du plan d'exécution les systèmes désactivés dans le projet.

**ORD-011** — Quand ECS NIDE calcule le plan d'exécution, il doit l'écrire dans
`schedule.plan.yaml`, avec ses étapes, leurs systèmes et les ambiguïtés signalées.
