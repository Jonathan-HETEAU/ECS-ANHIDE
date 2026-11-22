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
* Le fichier "schedule.plan.yaml" contient les étapes, leurs systèmes, la phase et la fréquence de chaque système, le profil de moteur utilisé et les ambiguïtés signalées

## Activer ou désactiver un composant compte comme une écriture
Exigences : ORD-002, ORD-003
* Le composant "Stunned" est activable
* Le système "StunTimer" écrit "Stunned" et peut désactiver "Stunned"
* Le système "Act" écrit "Velocity" et exige l'absence de "Stunned"
* Calculer le plan d'exécution
* "StunTimer" et "Act" sont dans des étapes différentes

## Un émetteur s'exécute toujours avant les lecteurs de son canal
Exigences : ORD-015
* L'ordre de référence est "Sfx", "Collision"
* "Collision" émet sur le canal "Explosion" et "Sfx" lit "Explosion"
* Calculer le plan d'exécution
* "Collision" est dans une étape strictement antérieure à celle de "Sfx"
* Aucune ambiguïté n'est signalée entre "Collision" et "Sfx"

## Des messages qui se répondent en boucle forment un cycle
Exigences : ORD-006, ORD-015
* "A" émet sur le canal "Ping", que lit "B"
* "B" émet sur le canal "Pong", que lit "A"
* Calculer le plan d'exécution
* Aucun plan n'est produit
* L'erreur nomme le cycle "A → B → A"

## Les phases ne se mélangent jamais
Exigences : ORD-012
* Les phases sont "Logique" puis "Rendu"
* "Movement" écrit "Position" et appartient à "Logique"
* "Ai" écrit "Velocity" et appartient à "Logique"
* "Audio" écrit "Sound" et appartient à "Rendu"
* Calculer le plan d'exécution
* "Movement" et "Ai" sont dans l'étape "1"
* "Audio" est dans l'étape "2", bien qu'il ne soit en conflit avec aucun autre système

## Deux systèmes disjoints partagent une étape si le moteur le permet
Exigences : ORD-013
* Le profil du moteur cible prend en charge les ensembles disjoints
* "PlayerMove" écrit "Position" et exige la présence de "Player"
* "EnemyMove" écrit "Position" et exige l'absence de "Player"
* Calculer le plan d'exécution
* "PlayerMove" et "EnemyMove" sont dans la même étape

## Sans prise en charge par le moteur, deux systèmes disjoints restent séparés
Exigences : ORD-013, ORD-002
* Le profil du moteur cible ne prend pas en charge les ensembles disjoints
* "PlayerMove" écrit "Position" et exige la présence de "Player"
* "EnemyMove" écrit "Position" et exige l'absence de "Player"
* Calculer le plan d'exécution
* "PlayerMove" et "EnemyMove" sont dans des étapes différentes

## Un moteur sans parallélisme entre systèmes reçoit une seule orbite
Exigences : ORD-014
* Le profil du moteur cible n'exécute pas de systèmes en parallèle
* Ouvrir le projet d'exemple "space-shooter"
* Calculer le plan d'exécution
* Chaque étape contient exactement un système
* Les systèmes se succèdent dans l'ordre résolu

## Les observateurs n'occupent aucune étape
Exigences : ORD-016
* Le système "Damage" peut ajouter "Dead"
* L'observateur "ScoreOnDeath" réagit à l'ajout de "Dead" et peut déclencher l'événement "EnemyKilled"
* L'observateur "Achievements" réagit à l'événement "EnemyKilled"
* Calculer le plan d'exécution
* Aucune étape ne contient "ScoreOnDeath" ni "Achievements"
* Le plan indique que l'étape de "Damage" peut déclencher "ScoreOnDeath" puis "Achievements"

## Une cascade circulaire d'observateurs est signalée
Exigences : ORD-017
* L'observateur "A" réagit à l'événement "Ping" et peut déclencher "Pong"
* L'observateur "B" réagit à l'événement "Pong" et peut déclencher "Ping"
* Calculer le plan d'exécution
* Un avertissement nomme le cycle d'observateurs "A → B → A"

## Une lecture via une relation compte comme une lecture
Exigences : ORD-003
* Le système "PropagateTransform" lit "GlobalTransform" via "ChildOf" et écrit "Transform"
* Le système "Physics" écrit "GlobalTransform"
* Calculer le plan d'exécution
* "PropagateTransform" et "Physics" sont dans des étapes différentes
