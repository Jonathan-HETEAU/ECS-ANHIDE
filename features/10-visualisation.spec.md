# La visualisation orbitale

Le projet est un trou noir. Une orbite fait le tour : un tour complet correspond à une boucle.
Les systèmes sont des planètes posées sur l'orbite, les systèmes parallèles occupent des
orbites concentriques, et les entités voyagent en flottes.

## Les étapes du plan sont posées sur l'orbite dans l'ordre d'exécution
Exigences : VIS-001
* Ouvrir le projet d'exemple "space-shooter" dans l'IDE
* L'orbite montre une position par étape du plan
* L'étape "1" est en haut et les étapes suivantes se succèdent dans le sens horaire

## Les systèmes parallèles sont sur des orbites concentriques
Exigences : VIS-002
* Le plan contient une étape avec les systèmes "Input" et "Audio"
* "Input" et "Audio" sont dessinés à la même position angulaire, sur deux orbites différentes

## Les entités voyagent en flottes identifiables par leurs composants
Exigences : VIS-003, VIS-004
* Peupler la simulation avec des entités "Asteroid" et "Player"
* Les flottes "Asteroid" et "Player" sont dessinées avec des signes distinctifs issus de leurs composants
* Pendant une boucle, la flotte "Asteroid" s'arrête sur la planète "Collision" et passe sans s'arrêter devant "PlayerInput"

## Sélectionner une planète affiche le contrat du système
Exigences : VIS-005
* Cliquer sur la planète "Damage"
* Le panneau affiche la spécification, les accès, les exemples, les propriétés et les derniers résultats de tests de "Damage"

## Sélectionner une flotte affiche son parcours
Exigences : VIS-006
* Cliquer sur la flotte "Asteroid"
* Le panneau liste les systèmes qui sélectionnent "Asteroid" dans l'ordre du plan
* Le panneau affiche les entités de la flotte

## Les diagnostics et états sont visibles sur les planètes
Exigences : VIS-009
* Le système "Magic" a une erreur de validation
* Le système "Damage" porte un point d'arrêt
* Le système "EnemyAI" est désactivé dans la simulation
* Chaque planète affiche le signe correspondant à son état

## Une vue matrice résume les accès
Exigences : VIS-008
* Ouvrir la vue matrice
* Chaque case système × composant indique l'accès : "read", "write", "optional", "with", "without", ajout, retrait ou vide

## La vue reste fluide sur un gros projet
Exigences : VIS-007
* Ouvrir un projet généré de "200" systèmes et "1000" archétypes
* Lancer la simulation
* La vue orbitale s'affiche à au moins "60" images par seconde
