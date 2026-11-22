# La validation du modèle

ECS NIDE analyse le modèle en continu et produit des diagnostics : des erreurs (le projet est
incohérent) et des avertissements (le projet est probablement incomplet).

## Une référence inconnue est une erreur
Exigences : VAL-001
* Le système "Magic" lit le composant inexistant "Mana"
* La validation signale l'erreur "composant inconnu : Mana" dans "systems/magic/system.yaml"

## Un composant que personne ne fournit est signalé
Exigences : VAL-002
* Le système "Magic" lit le composant "Mana"
* Aucun archétype ne contient "Mana" et aucun système ne l'ajoute
* La validation signale l'avertissement « "Mana" est lu par "Magic" mais n'est jamais fourni »

## Un système qui ne sélectionne aucune entité atteignable est signalé
Exigences : VAL-003
* Le système "Freeze" exige "Frozen" et "Velocity"
* Aucun archétype déclaré ne contient "Frozen"
* Aucun système n'ajoute "Frozen"
* La validation signale que "Freeze" ne sélectionne aucun archétype atteignable

## Un archétype obtenu par ajout de composant est atteignable
Exigences : VAL-003
* L'archétype "Asteroid" ne contient pas "Frozen"
* Le système "IceRay" peut ajouter "Frozen" aux entités qui possèdent "Velocity"
* Le système "Freeze" exige "Frozen" et "Velocity"
* La validation ne signale pas que "Freeze" ne sélectionne aucun archétype atteignable

## Un composant inutilisé est signalé
Exigences : VAL-004
* Le composant "Legacy" n'est utilisé par aucun système ni aucun archétype
* La validation signale que "Legacy" est inutilisé

## Un système sans spécification ou sans test est signalé
Exigences : VAL-005
* Le système "Audio" a un fichier "spec.md" vide et aucun exemple ni propriété
* La validation signale que "Audio" n'a pas de spécification
* La validation signale que "Audio" n'a aucun test

## Chaque diagnostic indique son niveau, son message et son emplacement
Exigences : VAL-006
* Ouvrir un projet qui contient des erreurs et des avertissements
* Chaque diagnostic indique un niveau, un message, un élément du modèle, un fichier et une ligne

## Les diagnostics suivent les modifications en temps réel
Exigences : VAL-007
* Ouvrir le projet d'exemple "space-shooter" dans l'IDE
* Ajouter au système "Movement" la lecture du composant inexistant "Mana"
* En moins d'une seconde, le diagnostic "composant inconnu : Mana" apparaît

## Avec des phases déclarées, chaque système doit avoir sa phase
Exigences : VAL-008
* Les phases sont "Logique" puis "Rendu"
* Le système "Audio" n'est rattaché à aucune phase
* La validation signale l'erreur « "Audio" n'est rattaché à aucune phase »

## On ne peut activer ou désactiver qu'un composant activable
Exigences : VAL-009
* Le composant "Health" n'est pas activable
* Le système "Heal" déclare l'activation de "Health"
* La validation signale que "Health" n'est pas activable

## Un filtre de changement porte sur un composant sélectionné
Exigences : VAL-010
* Le système "RebuildBounds" lit "Collider" et filtre les changements de "Velocity"
* La validation signale que "RebuildBounds" filtre "Velocity" sans le sélectionner

## Une clé déclarée deux fois est une erreur
Exigences : VAL-011
* Le fichier du système "Echo" déclare le canal "Ping" deux fois, en émission puis en lecture
* La validation signale la clé en double "Ping" dans "systems/echo/system.yaml"

## Un canal sans émetteur ou sans lecteur est signalé
Exigences : VAL-012
* Le système "Sfx" lit le canal "Explosion" et aucun système n'émet sur "Explosion"
* Le système "Score" émet sur le canal "Points" et aucun système ne lit "Points"
* La validation signale que "Explosion" est lu mais jamais émis
* La validation signale que "Points" est émis mais jamais lu

## Un concept non pris en charge par le moteur est signalé
Exigences : VAL-013
* Le profil du moteur cible ne prend pas en charge les composants activables
* Le projet déclare le composant activable "Stunned"
* La validation signale que les composants activables ne sont pas pris en charge par le moteur cible et renvoie à son skill

## Un observateur que rien ne déclenche est signalé
Exigences : VAL-014
* L'observateur "OnFreeze" réagit à l'ajout de "Frozen"
* Aucun système ni observateur ne peut ajouter "Frozen", et aucun archétype ne le contient
* La validation signale que "OnFreeze" ne peut jamais être déclenché

## Un événement que personne n'observe est signalé
Exigences : VAL-015
* Le système "Collision" peut déclencher l'événement "Damaged"
* Aucun observateur ne réagit à "Damaged"
* La validation signale que "Damaged" est déclenché mais jamais observé

## Un accès via une relation exige de sélectionner sa source
Exigences : VAL-016
* Le système "Follow" lit "GlobalTransform" via "ChildOf" sans sélectionner "ChildOf"
* La validation signale que "Follow" doit sélectionner "ChildOf" pour lire via cette relation
