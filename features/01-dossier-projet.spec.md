# Le dossier projet

Un projet ECS NIDE est un dossier de fichiers texte. C'est la seule source de vérité :
l'IDE, le serveur MCP, la CLI et les outils externes lisent et écrivent les mêmes fichiers.

## Créer un projet produit un dossier complet et valide
Exigences : PRJ-001, PRJ-002, MCP-008
* Créer le projet "Asteroides" dans un dossier vide
* Le dossier contient les fichiers "ecs-nide.yaml", "schedule.yaml", "AGENTS.md" et ".vscode/mcp.json"
* Le dossier contient les répertoires "components", "resources", "archetypes", "systems" et "skills"
* La validation du projet ne signale aucune erreur

## Un projet rouvert est identique au projet enregistré
Exigences : PRJ-003
* Ouvrir le projet d'exemple "space-shooter"
* Enregistrer le projet dans un nouveau dossier
* Rouvrir le projet depuis ce nouveau dossier
* Les deux modèles sont identiques

## Enregistrer sans modification ne change aucun fichier
Exigences : PRJ-006
* Ouvrir le projet d'exemple "space-shooter"
* Enregistrer le projet sans rien modifier
* Aucun fichier du dossier n'a changé

## Une modification faite hors de l'IDE apparaît dans l'IDE
Exigences : PRJ-004
* Ouvrir le projet d'exemple "space-shooter" dans l'IDE
* Depuis l'extérieur, modifier "systems/movement/system.yaml" pour que "Movement" écrive aussi "Rotation"
* En moins d'une seconde, l'IDE affiche que "Movement" écrit "Rotation"

## Une modification simultanée dans l'IDE et à l'extérieur demande un arbitrage
Exigences : PRJ-008
* Ouvrir le projet d'exemple "space-shooter" dans l'IDE
* Modifier la description de "Movement" dans l'IDE sans enregistrer
* Depuis l'extérieur, modifier la description de "Movement" dans "systems/movement/system.yaml"
* L'IDE propose de garder sa version ou la version extérieure

## Un fichier invalide est signalé sans bloquer le reste du projet
Exigences : PRJ-005
* Ouvrir le projet d'exemple "space-shooter"
* Remplacer le contenu de "components/health.yaml" par un YAML mal formé à la ligne "3"
* La validation signale une erreur dans "components/health.yaml" à la ligne "3"
* Tous les autres composants sont chargés

## Tout fonctionne sans connexion réseau
Exigences : PRJ-007
* Couper tout accès réseau
* Créer un projet, choisir le moteur cible "bevy", valider, calculer le plan, simuler une boucle et démarrer "ecs-nide mcp"
* Toutes les opérations réussissent

## Un format plus récent que l'IDE n'est jamais écrasé
Exigences : PRJ-009
* Ouvrir un projet dont "ecs-nide.yaml" déclare une version de format supérieure à celle de l'IDE
* L'IDE ouvre le projet en lecture seule et explique pourquoi
