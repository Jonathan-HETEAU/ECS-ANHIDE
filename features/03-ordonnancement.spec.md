# L'ordonnancement

Tous les systèmes s'exécutent dans une même boucle. ECS NIDE calcule un plan d'exécution
découpé en étapes. Les systèmes d'une même étape tournent en parallèle, chacun sur son orbite.
Deux systèmes sont en conflit si l'un écrit un composant ou une ressource que l'autre lit ou écrit.

## Chaque système activé apparaît exactement une fois dans le plan
Exigences : ORD-001
* Ouvrir le projet d'exemple "space-shooter"
* Calculer le plan d'exécution
* Chaque système activé figure dans exactement une étape

## Deux systèmes qui écrivent le même composant ne sont jamais dans la même étape
Exigences : ORD-002
* Le système "Movement" écrit "Position"
* Le système "Knockback" écrit "Position"
* Calculer le plan d'exécution
* "Movement" et "Knockback" sont dans des étapes différentes

## Un système qui lit ce qu'un autre écrit n'est pas dans la même étape
Exigences : ORD-003
* Le système "Movement" écrit "Position"
* Le système "Render" lit "Position"
* Calculer le plan d'exécution
* "Movement" et "Render" sont dans des étapes différentes

## Les ressources suivent les mêmes règles de conflit
Exigences : ORD-002, ORD-003
* Le système "Score" écrit la ressource "GameState"
* Le système "Hud" lit la ressource "GameState"
* Calculer le plan d'exécution
* "Score" et "Hud" sont dans des étapes différentes

## Un filtre de présence ou d'absence ne crée pas de conflit
Exigences : ORD-003
* Le système "ShieldedMove" écrit "Position" et exige la présence de "Shield"
* Le système "ShieldDecay" écrit "Shield"
* Calculer le plan d'exécution
* "ShieldedMove" et "ShieldDecay" sont dans la même étape

## Chaque système rejoint la première étape compatible
Exigences : ORD-004
* L'ordre de référence est "Input", "Movement", "Audio", "Render"
* Les accès des systèmes sont :

  | système  | lit                | écrit    |
  |----------|--------------------|----------|
  | Input    |                    | Velocity |
  | Movement | Velocity           | Position |
  | Audio    |                    | Sound    |
  | Render   | Position, Sprite   |          |

* Calculer le plan d'exécution
* Le plan est :

  | étape | systèmes      |
  |-------|---------------|
  | 1     | Input, Audio  |
  | 2     | Movement      |
  | 3     | Render        |

## Une contrainte d'ordre est toujours respectée
Exigences : ORD-005
* Le système "Physics" écrit "Position"
* Le système "Audio" écrit "Sound" et doit s'exécuter après "Physics"
* Calculer le plan d'exécution
* "Audio" est dans une étape strictement postérieure à celle de "Physics"

## Une contrainte d'ordre circulaire est une erreur
Exigences : ORD-006
* "A" doit s'exécuter avant "B"
* "B" doit s'exécuter avant "C"
* "C" doit s'exécuter avant "A"
* Calculer le plan d'exécution
* Aucun plan n'est produit
* L'erreur nomme le cycle "A → B → C → A"

## Deux systèmes en conflit sans contrainte d'ordre forment une ambiguïté
Exigences : ORD-007
* L'ordre de référence est "Knockback", "Movement"
* Les systèmes "Movement" et "Knockback" écrivent "Position", sans contrainte d'ordre entre eux
* Calculer le plan d'exécution
* Une ambiguïté est signalée entre "Knockback" et "Movement"
* "Knockback" est placé avant "Movement"

## Le plan est déterministe
Exigences : ORD-008
* Ouvrir le projet d'exemple "space-shooter"
* Calculer le plan d'exécution "100" fois
* Les "100" plans sont identiques

## Un système désactivé ne figure pas dans le plan
Exigences : ORD-010
* Ouvrir le projet d'exemple "space-shooter"
* Désactiver le système "EnemyAI" dans le projet
* Calculer le plan d'exécution
* "EnemyAI" ne figure dans aucune étape

## Les changements structurels sont appliqués à la fin de l'étape
Exigences : ORD-009
* Le système "Damage" retire toujours "Hit"
* Le système "Flash" lit "Hit" et se trouve dans la même étape que "Damage"
* Simuler une entité qui possède "Hit" et "Health"
* "Flash" voit encore "Hit" pendant l'étape
* L'entité ne possède plus "Hit" à l'étape suivante

## Le plan calculé est exporté pour les outils externes
Exigences : ORD-011
* Ouvrir le projet d'exemple "space-shooter"
* Calculer le plan d'exécution
* Le fichier "schedule.plan.yaml" contient les étapes, leurs systèmes et les ambiguïtés signalées
