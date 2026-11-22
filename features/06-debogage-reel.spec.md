# Le débogage du jeu réel

Le jeu embarque une sonde propre à sa bibliothèque ECS. L'IDE s'y connecte par un protocole
documenté et indépendant du langage. À l'échelle de millions d'entités, on observe des flottes
(une par archétype) et des échantillons, jamais l'intégralité du monde.

Les scénarios utilisent une **sonde de référence** fournie avec ECS NIDE, qui simule un jeu
conforme au protocole.

## L'IDE se connecte à un jeu équipé d'une sonde
Exigences : DBG-001, DBG-011
* Démarrer la sonde de référence avec le projet d'exemple "space-shooter"
* Se connecter depuis l'IDE
* L'IDE indique la connexion et affiche la frame en cours

## Une différence entre le plan réel et le plan déclaré est signalée
Exigences : DBG-002
* Démarrer la sonde de référence avec un plan où "Damage" s'exécute avant "Collision"
* Se connecter depuis l'IDE
* L'IDE signale que "Damage" et "Collision" sont dans un ordre différent du plan déclaré

## Les entités sont regroupées en flottes par archétype
Exigences : DBG-003
* La sonde de référence contient "1000000" entités "Particle" et "500" entités "Asteroid"
* Se connecter depuis l'IDE
* L'IDE affiche une flotte "Particle" de "1000000" entités et une flotte "Asteroid" de "500" entités

## Le temps passé dans chaque système est affiché
Exigences : DBG-004
* Se connecter à la sonde de référence
* Pour la dernière frame, l'IDE affiche le temps d'exécution de chaque système du plan

## Pause et pas à pas pilotent le jeu
Exigences : DBG-005
* Se connecter à la sonde de référence
* Mettre le jeu en pause
* Avancer d'un pas
* La sonde a exécuté exactement une étape et s'est remise en pause

## Points d'arrêt et désactivation sont transmis au jeu
Exigences : DBG-006
* Se connecter à la sonde de référence
* Poser un point d'arrêt sur "Damage" et désactiver "EnemyAI"
* La sonde se met en pause avant l'étape contenant "Damage"
* La sonde n'exécute plus "EnemyAI"

## Pendant la pause, on inspecte un échantillon d'entités
Exigences : DBG-007
* Le jeu est en pause
* Sélectionner la flotte "Asteroid" et demander un échantillon de "20" entités
* L'IDE affiche "20" entités "Asteroid" avec les valeurs de leurs composants

## Pendant la pause, on modifie une entité réelle
Exigences : DBG-008
* Le jeu est en pause
* Fixer "Health.current" à "0" pour l'entité "42"
* La sonde confirme que "Health.current" vaut "0" pour l'entité "42"

## La perte de connexion conserve la dernière frame
Exigences : DBG-009
* Se connecter à la sonde de référence
* Arrêter brutalement la sonde
* L'IDE indique la déconnexion et affiche toujours la dernière frame reçue

## Le trafic par frame ne dépend pas du nombre d'entités
Exigences : DBG-010
* Mesurer le volume échangé par frame avec "1000" entités
* Mesurer le volume échangé par frame avec "10000000" entités
* Hors échantillons demandés, les deux volumes sont identiques à une constante près
