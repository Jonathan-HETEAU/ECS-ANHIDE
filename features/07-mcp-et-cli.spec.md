# Le serveur MCP et la ligne de commande

Le projet vit dans un dossier externe, qu'on peut ouvrir dans VS Code. Le binaire `ecs-nide`
fournit un serveur MCP autonome (`ecs-nide mcp`) et une CLI. Les deux passent par le même cœur
que l'IDE.

## Le serveur MCP fonctionne sans l'IDE
Exigences : MCP-001
* L'IDE n'est pas lancé
* Démarrer "ecs-nide mcp" sur le projet d'exemple "space-shooter"
* Un client MCP obtient la liste des systèmes du projet

## Une IA obtient le contrat d'un système
Exigences : MCP-003, TST-013
* Démarrer "ecs-nide mcp" sur le projet d'exemple "space-shooter"
* Un client MCP demande le contrat de "Damage"
* Le contrat reçu est identique à celui que produit "ecs-nide contract Damage"

## Une IA crée un système et le dossier est mis à jour
Exigences : MCP-002, MCP-004
* Démarrer "ecs-nide mcp" sur le projet d'exemple "space-shooter"
* Un client MCP crée le système "Regeneration" qui écrit "Health" et exige "Regenerating"
* Le dossier "systems/regeneration" contient "system.yaml", "spec.md", "examples.yaml" et "properties.yaml"

## Une modification invalide est refusée avec une explication
Exigences : MCP-005
* Démarrer "ecs-nide mcp" sur le projet d'exemple "space-shooter"
* Un client MCP crée un système qui lit le composant inexistant "Mana"
* La demande est refusée avec l'erreur de validation "composant inconnu : Mana"
* Aucun fichier du projet n'a changé

## Une IA lance validation, plan, génération et import
Exigences : MCP-006
* Démarrer "ecs-nide mcp" sur le projet d'exemple "space-shooter"
* Un client MCP lance successivement la validation, le calcul du plan, la génération de données pour "Damage" et l'import d'un rapport de tests
* Chaque opération renvoie le même résultat que la commande CLI correspondante

## Les modifications MCP apparaissent dans l'IDE ouvert
Exigences : MCP-007, PRJ-004
* Ouvrir le projet d'exemple "space-shooter" dans l'IDE
* Démarrer "ecs-nide mcp" sur le même projet
* Un client MCP ajoute un exemple à "Damage"
* En moins d'une seconde, l'IDE affiche le nouvel exemple

## La CLI propose les mêmes opérations que le cœur
Exigences : MCP-009, MCP-010
* Exécuter "ecs-nide validate", "ecs-nide plan", "ecs-nide contract Damage", "ecs-nide gen-tests Damage --seed 42" et "ecs-nide import-results rapport.json" sur le projet d'exemple
* Chaque commande réussit et produit les mêmes données que l'opération équivalente du cœur, que partagent l'IDE et le serveur MCP

## La CLI échoue quand le projet est invalide
Exigences : MCP-011
* Introduire une référence au composant inexistant "Mana" dans le projet d'exemple
* Exécuter "ecs-nide validate"
* La commande se termine avec un code de sortie non nul et affiche le diagnostic

## La CLI écrit ses résultats en JSON pour les outils
Exigences : MCP-012
* Introduire une référence au composant inexistant "Mana" dans le projet d'exemple
* Exécuter "ecs-nide validate --format json"
* La sortie est une liste JSON de diagnostics, dont l'erreur "composant inconnu : Mana" avec son code, son fichier et sa ligne

## La CLI crée un projet
Exigences : MCP-013, PRJ-001, PRJ-010
* Exécuter "ecs-nide new asteroides --name Asteroides --language fr" dans un dossier vide
* La commande réussit
* Le dossier "asteroides" contient un projet valide, avec un "AGENTS.md" rédigé en français
