# PRJ — Le dossier projet

**PRJ-001** — Quand l'utilisateur crée un projet, ECS NIDE doit créer un dossier contenant
`ecs-nide.yaml`, `schedule.yaml` et les répertoires `components/`, `resources/`, `messages/`,
`archetypes/`, `systems/` et `skills/`.

**PRJ-002** — Quand l'utilisateur crée un projet, ECS NIDE doit générer un fichier `AGENTS.md` qui
explique aux IA la structure du dossier, le format des fichiers et la méthode de travail fondée
sur les tests.

**PRJ-003** — ECS NIDE doit pouvoir enregistrer puis rouvrir un projet sans perte ni altération
d'information.

**PRJ-004** — Pendant qu'un projet est ouvert dans l'IDE, quand un fichier du projet est modifié
de l'extérieur, l'IDE doit recharger le modèle et l'afficher en moins d'une seconde.

**PRJ-005** — Si un fichier du projet est invalide, alors ECS NIDE doit signaler le fichier, la
ligne et la cause de l'erreur, et continuer à charger les autres fichiers.

**PRJ-006** — ECS NIDE doit écrire les fichiers du projet de façon déterministe, afin qu'un
enregistrement sans modification ne change aucun fichier.

**PRJ-007** — ECS NIDE doit fournir toutes ses fonctionnalités sans connexion réseau.

**PRJ-008** — Pendant qu'un élément a des modifications non enregistrées dans l'IDE, quand son
fichier est modifié de l'extérieur, l'IDE doit proposer de conserver soit sa version, soit la
version extérieure.

**PRJ-009** — Si `ecs-nide.yaml` déclare une version de format supérieure à celle que gère
ECS NIDE, alors ECS NIDE doit ouvrir le projet en lecture seule et en expliquer la raison.

**PRJ-010** — ECS NIDE doit rédiger les fichiers qu'il génère dans un projet (`AGENTS.md`, et les
skills lorsqu'ils existent dans cette langue) dans la langue choisie pour le projet et enregistrée
dans `ecs-nide.yaml`, l'anglais étant la langue par défaut.
