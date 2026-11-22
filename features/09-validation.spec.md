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
* La validation ne signale rien pour "Freeze"

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
