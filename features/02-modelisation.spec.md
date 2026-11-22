# La modélisation

On décrit le jeu avec quatre concepts : les composants (données d'une entité), les ressources
(données globales), les systèmes (règles) et les archétypes (combinaisons de composants).
Un système déclare ses accès et ses effets, jamais son implémentation.

## Un composant déclare des champs typés, contraints et initialisés
Exigences : MOD-001, MOD-002, MOD-003
* Déclarer le composant "Health" avec les champs :

  | champ   | type | min | max  | défaut |
  |---------|------|-----|------|--------|
  | current | f32  | 0   | 1000 | 100    |
  | max     | f32  | 1   | 1000 | 100    |

* Le composant "Health" possède "2" champs
* La valeur par défaut de "Health.current" est "100"
* La valeur "1500" est hors contraintes pour "Health.current"

## Un composant sans champ est un marqueur
Exigences : MOD-004
* Déclarer le composant "Enemy" sans champ
* "Enemy" est un marqueur

## Deux composants ne peuvent pas porter le même nom
Exigences : MOD-001, VAL-001
* Déclarer le composant "Health"
* Déclarer à nouveau un composant "Health"
* La validation signale un nom de composant en double : "Health"

## Une ressource est une donnée globale unique
Exigences : MOD-005, MOD-007
* Déclarer la ressource "Time" avec le champ "delta" de type "f32"
* Déclarer le système "Movement" qui lit la ressource "Time"
* Le contrat de "Movement" contient la ressource "Time" en lecture

## Un système sélectionne les entités d'après ses accès
Exigences : MOD-006, MOD-009
* Déclarer le système "EnemyAI" qui lit "Position", écrit "Velocity", lit "Target" en optionnel, exige la présence de "Enemy" et l'absence de "Dead"
* La sélection de "EnemyAI" donne :

  | composants de l'entité              | sélectionnée |
  |-------------------------------------|--------------|
  | Position, Velocity, Enemy           | oui          |
  | Position, Velocity, Enemy, Target   | oui          |
  | Position, Velocity, Enemy, Dead     | non          |
  | Position, Velocity                  | non          |
  | Velocity, Enemy                     | non          |

## Un système déclare ses changements structurels, certains ou possibles
Exigences : MOD-008
* Déclarer le système "Damage" qui retire toujours "Hit" et peut ajouter "Dead"
* Déclarer le système "Weapon" qui peut créer des entités de l'archétype "Bullet"
* Déclarer le système "Death" qui exige "Dead" et détruit toujours l'entité
* Le contrat de "Damage" indique le retrait certain de "Hit" et l'ajout possible de "Dead"
* Le contrat de "Weapon" indique la création possible de "Bullet"
* Le contrat de "Death" indique la destruction certaine

## Un archétype décrit une composition et un volume estimé
Exigences : MOD-010
* Déclarer l'archétype "Particle" composé de "Position", "Velocity" et "Lifetime", avec un volume estimé de "2000000"
* L'archétype "Particle" est sélectionné par tout système qui lit "Position" et "Velocity"

## Renommer un composant met à jour toutes ses références
Exigences : MOD-011
* Ouvrir le projet d'exemple "space-shooter"
* Renommer le composant "Health" en "HitPoints"
* Aucun fichier du projet ne mentionne plus "Health" comme composant
* Les systèmes, archétypes, exemples et propriétés qui utilisaient "Health" utilisent "HitPoints"
* La validation ne signale aucune nouvelle erreur

## Supprimer un élément référencé demande une confirmation
Exigences : MOD-012
* Ouvrir le projet d'exemple "space-shooter"
* Demander la suppression du composant "Velocity"
* L'IDE liste les systèmes et archétypes qui référencent "Velocity" et demande confirmation

## Un composant activable désactivé compte comme absent
Exigences : MOD-013, MOD-009
* Déclarer le composant activable "Stunned"
* Déclarer le système "Act" qui écrit "Velocity" et exige l'absence de "Stunned"
* La sélection de "Act" donne :

  | composants de l'entité          | sélectionnée |
  |---------------------------------|--------------|
  | Velocity                        | oui          |
  | Velocity, Stunned (activé)      | non          |
  | Velocity, Stunned (désactivé)   | oui          |

## Activer ou désactiver un composant n'est pas un changement structurel
Exigences : MOD-014
* Déclarer le composant activable "Stunned"
* Déclarer le système "StunTimer" qui écrit "Stunned" et peut désactiver "Stunned"
* Le contrat de "StunTimer" indique la désactivation possible de "Stunned"
* Le contrat de "StunTimer" ne déclare aucun changement structurel

## Un filtre de changement restreint le système aux entités modifiées
Exigences : MOD-015
* Déclarer le système "RebuildBounds" qui lit "Collider", écrit "Bounds" et filtre les changements de "Collider"
* Le contrat de "RebuildBounds" indique le filtre de changement sur "Collider"

## Un composant porte une indication de stockage
Exigences : MOD-016
* Déclarer le composant "Hit" avec le stockage "sparse"
* Déclarer le composant "Position" sans indication de stockage
* Le stockage de "Hit" est "sparse" et celui de "Position" est "dense"
* Le contrat de "Damage" indique le stockage "sparse" pour "Hit"

## Un canal de messages transporte des données typées
Exigences : MOD-017, MOD-018
* Déclarer le canal de messages "Explosion" avec les champs "position" de type "vec2" et "radius" de type "f32"
* Déclarer le système "Collision" qui peut émettre sur "Explosion"
* Déclarer le système "Sfx" qui lit "Explosion"
* Le contrat de "Collision" indique l'émission possible sur "Explosion"
* Le contrat de "Sfx" indique la lecture de "Explosion" et contient le schéma de "Explosion"

## Un système déclare s'il traite chaque entité indépendamment
Exigences : MOD-019
* Déclarer le système "Movement" avec "per_entity" à vrai
* Déclarer le système "Broadphase" sans préciser "per_entity"
* Le contrat de "Movement" indique un traitement par entité
* Le contrat de "Broadphase" indique un traitement qui n'est pas par entité

## Les systèmes se rattachent à des phases
Exigences : MOD-020
* Déclarer les phases "Entrees", "Logique" et "Rendu", dans cet ordre
* Rattacher "PlayerInput" à "Entrees" et "Render" à "Rendu"
* La phase de "PlayerInput" est "Entrees" et celle de "Render" est "Rendu"

## Un système peut ne s'exécuter qu'une boucle sur N
Exigences : MOD-021
* Déclarer le système "Pathfinding" avec la fréquence "4"
* Déclarer le système "Movement" sans fréquence
* La fréquence de "Pathfinding" est "4" et celle de "Movement" est "1"

## Un événement porte des données et peut viser une entité
Exigences : MOD-022, MOD-023
* Déclarer l'événement d'entité "Damaged" avec le champ "amount" de type "f32"
* Déclarer le système "Collision" qui peut déclencher "Damaged"
* "Damaged" est un événement d'entité
* Le contrat de "Collision" indique le déclenchement possible de "Damaged"

## Un observateur réagit à un seul déclencheur
Exigences : MOD-024, MOD-025
* Déclarer l'observateur "ScoreOnDeath" qui réagit à l'ajout de "Dead", exige "Enemy" et écrit la ressource "GameState"
* Déclarer l'observateur "ShowDamage" qui réagit à l'événement "Damaged" et peut émettre sur le canal "Hud"
* Le dossier "observers/score-on-death" contient "observer.yaml", "spec.md", "examples.yaml" et "properties.yaml"
* Le déclencheur de "ScoreOnDeath" est l'ajout de "Dead"
* Le contrat de "ShowDamage" indique l'émission possible sur "Hud"

## Une relation relie des entités sources à une entité cible
Exigences : MOD-026, MOD-027
* Déclarer la relation "ChildOf" dont le composant cible est "Children", avec destruction liée
* "ChildOf" désigne une entité cible
* "Children" liste les entités sources et n'est jamais écrit directement par un système
* Détruire une entité cible de "ChildOf" détruit ses entités sources

## Un système lit les composants de la cible d'une relation
Exigences : MOD-028
* Déclarer le système "PropagateTransform" qui lit "ChildOf", écrit "GlobalTransform" et lit "GlobalTransform" via "ChildOf"
* Le contrat de "PropagateTransform" indique la lecture de "GlobalTransform" sur la cible de "ChildOf"

## Un archétype crée ses entités liées
Exigences : MOD-029
* Déclarer l'archétype "Wheel" composé de "Position" et "Rotation"
* Déclarer l'archétype "Car" composé de "Position" et "Velocity", qui crée "4" entités "Wheel" reliées par "ChildOf"
* L'archétype "Car" déclare "4" entités liées de l'archétype "Wheel" par la relation "ChildOf"

## Un événement d'entité peut se propager le long d'une relation
Exigences : MOD-030
* Déclarer l'événement d'entité "Clicked" qui se propage le long de "ChildOf"
* "Clicked" se propage de l'entité source vers sa cible par "ChildOf"
