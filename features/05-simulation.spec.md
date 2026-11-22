# La simulation du modèle

Sans aucune ligne de code de jeu, on peut faire tourner le modèle. Les entités parcourent
l'orbite, et chaque système applique ce que son contrat et ses exemples permettent de savoir.
Ce qui ne peut pas être connu est affiché comme tel.

## Une boucle fait passer chaque entité par les systèmes qui la sélectionnent
Exigences : SIM-001, SIM-002
* Ouvrir le projet d'exemple "space-shooter"
* Peupler la simulation avec une entité de l'archétype "Asteroid"
* Exécuter une boucle
* L'entité est passée par "Movement", "Collision" et "Render"
* L'entité n'est pas passée par "PlayerInput"

## Un pas exécute exactement l'étape suivante
Exigences : SIM-003
* Ouvrir le projet d'exemple "space-shooter" et peupler la simulation
* La simulation est en pause au début de la boucle
* Avancer d'un pas
* Seuls les systèmes de l'étape "1" ont été exécutés
* La simulation est en pause avant l'étape "2"

## Une boucle exécute toutes les étapes restantes
Exigences : SIM-004
* La simulation est en pause avant l'étape "2"
* Avancer d'une boucle
* Toutes les étapes à partir de l'étape "2" ont été exécutées
* La simulation est en pause au début de la boucle suivante

## Revenir en arrière restaure exactement l'état précédent
Exigences : SIM-005
* Enregistrer l'état du monde
* Avancer d'un pas
* Revenir en arrière d'un pas
* L'état du monde est identique à l'état enregistré

## Un point d'arrêt met en pause à l'arrivée sur le système
Exigences : SIM-006
* Poser un point d'arrêt sur "Damage"
* Lancer la lecture
* La simulation est en pause avant l'étape qui contient "Damage"

## Désactiver un système pendant la simulation ne modifie pas le projet
Exigences : SIM-007
* Lancer la simulation
* Désactiver "EnemyAI" dans la simulation
* "EnemyAI" n'est plus exécuté à partir de l'étape suivante
* Le fichier "systems/enemy-ai/system.yaml" n'a pas changé

## Un exemple correspondant fournit les valeurs de sortie
Exigences : SIM-008
* Une entité possède "Health.current = 100", "Health.max = 100" et "Hit.amount = 30"
* "Damage" traite cette entité
* L'entité a "Health.current = 70" et ne possède plus "Hit"

## Sans exemple correspondant, on applique ce qui est certain et on signale le reste
Exigences : SIM-009
* Une entité possède "Health.current = 55", "Health.max = 100" et "Hit.amount = 12"
* "Damage" traite cette entité
* L'entité ne possède plus "Hit"
* "Health.current" et "Health.max" sont indéterminés
* L'ajout possible de "Dead" est signalé sur l'entité sans être appliqué

## Une valeur indéterminée ne correspond à aucun exemple
Exigences : SIM-008, SIM-009
* Une entité possède "Health.current" indéterminé et "Hit.amount = 30"
* "Damage" traite cette entité
* Aucun exemple n'est appliqué

## Pendant la pause, on peut modifier une entité
Exigences : SIM-010
* La simulation est en pause
* Ajouter "Dead" à l'entité "1" et fixer "Health.current" à "0"
* Avancer d'une boucle
* L'entité "1" est passée par "Death" et a été détruite

## Peupler la simulation à partir des archétypes
Exigences : SIM-011
* Ouvrir le projet d'exemple "space-shooter"
* Peupler la simulation avec "3" entités de l'archétype "Asteroid" et la graine "7"
* Il existe "3" entités possédant exactement les composants de "Asteroid"
* Leurs valeurs respectent les contraintes des schémas
* Peupler à nouveau avec la graine "7" donne exactement les mêmes entités

## Une entité créée par un système reçoit les valeurs par défaut
Exigences : SIM-012
* Le système "Weapon" crée toujours une entité de l'archétype "Bullet"
* Une entité est traitée par "Weapon"
* À la fin de l'étape, une nouvelle entité possède les composants de "Bullet" avec leurs valeurs par défaut
