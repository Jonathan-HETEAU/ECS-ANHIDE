# ECS NIDE

**Un IDE de bureau pour concevoir des jeux en Entité-Composant-Système de manière déclarative.**

Le projet est un trou noir. Les systèmes sont des planètes en orbite, et les entités sont des
vaisseaux qui font le tour : un tour correspond à une frame. On décrit chaque système par son
contrat, sa spécification et ses tests, sans écrire de code. Une IA, guidée par les skills du
moteur choisi, l'implémente ensuite dans n'importe quel langage et n'importe quelle
bibliothèque ECS.

## Structure du dépôt

| Chemin | Contenu |
|---|---|
| [`VISION.md`](VISION.md) | Ce qu'on attend du projet |
| [`exigences/`](exigences/) | Le fonctionnement attendu, phrase par phrase (EARS) |
| [`features/`](features/) | Les scénarios exécutables (Spec Markdown) |
| [`AGENTS.md`](AGENTS.md) | Les règles de travail : tout part des tests |
| [`recherche/`](recherche/) | Les études qui éclairent les décisions |
| [`prototypes/`](prototypes/) | Des prototypes jetables qui évaluent une technologie (hors produit) |

## Stack

Tauri 2 · Rust · Svelte 5 + TypeScript · CEL · YAML

## Développer

La ligne de commande `ecs-nide` (`cargo install --path crates/ecs-nide-cli`, ou
`cargo run -p ecs-nide-cli --`) :

```sh
ecs-nide new mon-jeu --name "Mon jeu" --language fr   # crée un projet
ecs-nide validate                                     # diagnostics ; code 1 s'il reste une erreur
ecs-nide plan                                         # plan d'exécution, écrit dans schedule.plan.yaml
ecs-nide contract Damage                              # contrat d'un système, pour une IA
ecs-nide gen-tests Damage --seed 42                   # données de test des propriétés (JSON)
ecs-nide import-results rapport.json                  # résultats CTRF ou JUnit XML
```

Toutes les commandes acceptent `--project <dossier>` et `--format json`.

Pour développer ECS NIDE :

```sh
cargo test --workspace                                       # tous les tests, scénarios compris
cargo test -p ecs-nide-specs --test scenarios -- --nocapture # rapport d'avancement des scénarios
```

## État

Phase 0 (spécifications) terminée. Phase 1 (cœur Rust) terminée pour tout ce qui ne dépend ni de
l'IDE ni de la simulation : dossier projet, modélisation, ordonnancement, validation, contrats, tests
par exemples et par propriétés (CEL), génération de données de test, import des rapports. Suite :
la CLI (phase 2), elle aussi terminée. Suite : le serveur MCP. Voir la feuille de route dans
[`VISION.md`](VISION.md#13-feuille-de-route-proposée).
