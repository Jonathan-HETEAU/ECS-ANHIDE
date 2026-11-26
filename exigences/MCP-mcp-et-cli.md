# MCP — Le serveur MCP et la ligne de commande

**MCP-001** — ECS NIDE doit fournir un serveur MCP, démarré par la commande `ecs-nide mcp`, qui
fonctionne sans que l'IDE soit lancé.

**MCP-002** — Le serveur MCP doit lire et écrire exclusivement les fichiers du dossier projet.

**MCP-003** — Quand un client demande le contrat d'un système, le serveur MCP doit renvoyer le
contrat autonome défini par TST-013.

**MCP-004** — Le serveur MCP doit permettre de lire le modèle, et de créer, modifier et supprimer
des composants, ressources, archétypes, systèmes, spécifications, exemples et propriétés.

**MCP-005** — Si une modification demandée au serveur MCP rendrait le projet invalide, alors le
serveur MCP doit la refuser sans modifier aucun fichier et renvoyer les erreurs de validation.

**MCP-006** — Le serveur MCP doit permettre de lancer la validation, le calcul du plan, la
génération de données de test et l'import d'un rapport de résultats de tests.

**MCP-007** — Pendant que l'IDE est ouvert sur un projet, quand le serveur MCP modifie ce projet,
l'IDE doit afficher la modification dans le délai fixé par PRJ-004.

**MCP-008** — Quand l'utilisateur crée un projet, ECS NIDE doit générer `.vscode/mcp.json` pour que
VS Code démarre le serveur MCP sur ce projet.

**MCP-009** — La CLI doit proposer les commandes `validate`, `plan`, `contract`, `gen-tests` et
`import-results`, qui agissent sur le projet du dossier courant ou sur celui indiqué par
`--project`.

**MCP-010** — ECS NIDE doit produire des résultats identiques pour une même opération, qu'elle soit
lancée depuis l'IDE, la CLI ou le serveur MCP.

**MCP-011** — Si la validation d'un projet produit au moins une erreur, alors la CLI doit afficher
les diagnostics et se terminer avec un code de sortie non nul.

**MCP-012** — Quand l'option `--format json` est donnée, la CLI doit écrire le résultat de la commande
en JSON, avec les mêmes données que l'opération équivalente du cœur.

**MCP-013** — Quand l'utilisateur exécute `ecs-nide new <dossier> --name <nom>`, la CLI doit créer le
projet comme le fait l'IDE (PRJ-001, PRJ-002, PRJ-010, MCP-008), dans la langue donnée par
`--language`.
