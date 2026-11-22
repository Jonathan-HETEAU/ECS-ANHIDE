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
