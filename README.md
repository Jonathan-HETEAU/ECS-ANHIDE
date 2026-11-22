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

## Stack

Tauri 2 · Rust · Svelte 5 + TypeScript · CEL · YAML

## Développer

```sh
cargo test --workspace                                       # tous les tests, scénarios compris
cargo test -p ecs-nide-specs --test scenarios -- --nocapture # rapport d'avancement des scénarios
cargo run -p ecs-nide-core --example plan -- examples/space-shooter   # plan d'exécution d'un projet
```

## État

Phase 0 (spécifications) terminée. Phase 1 (cœur Rust) en cours : dossier projet, modélisation,
ordonnancement, validation et contrats sont implémentés, et leurs scénarios passent. Voir la feuille de route dans
[`VISION.md`](VISION.md#13-feuille-de-route-proposée).
