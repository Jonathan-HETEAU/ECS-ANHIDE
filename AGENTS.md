# AGENTS.md — Règles de travail sur ECS NIDE

Ce fichier s'adresse à toute personne ou IA qui travaille sur ce dépôt.

## À lire avant tout

1. [`VISION.md`](VISION.md) : ce qu'on attend du projet.
2. [`exigences/`](exigences/) : le fonctionnement attendu, phrase par phrase (EARS).
3. [`features/`](features/) : les scénarios exécutables (Spec Markdown).

## La règle d'or : tout part des tests

- **Pas de comportement sans exigence.** Tout nouveau comportement commence par une exigence EARS
  dans `exigences/` et au moins un scénario dans `features/`.
- **Le test d'abord.** On écrit le test (ou on relie les étapes du scénario), on le voit échouer,
  puis on écrit le code qui le fait passer.
- **Tout bug est un test manquant.** Avant de corriger un bug :
  1. écrire le test qui le reproduit et le voir échouer ;
  2. si aucune exigence ni aucun scénario ne couvrait ce cas, les ajouter ;
  3. corriger, puis vérifier que le test passe.

  Le commit de correction contient toujours le test.
- **Traçabilité.** Chaque test cite l'identifiant de l'exigence qu'il vérifie (par exemple,
  `ord_002_same_write_never_same_stage`). Chaque exigence est couverte par au moins un scénario et
  au moins un test.
- **Une exigence qui change entraîne la mise à jour de ses scénarios et de ses tests**, dans le même commit.
- **Ne jamais désactiver, ignorer ou affaiblir un test pour obtenir une exécution au vert.**

## Ce qui relève de l'IDE, et ce qui relève de l'utilisateur

- **L'IDE signale, l'utilisateur décide** (principe 8 de `VISION.md`). Le code d'ECS NIDE détecte
  et signale les problèmes d'un projet ; il ne prend jamais de décision de conception du jeu à la
  place de l'utilisateur.
- Le **projet d'exemple** `examples/space-shooter` joue le rôle d'un utilisateur : quand on y fait
  un choix de conception, on fait le plus sensé. Il doit rester exemplaire (aucun diagnostic,
  aucune ambiguïté), ce que vérifie `crates/ecs-nide-specs/tests/exemple.rs`.

## Choix techniques

| Partie | Technologie |
|---|---|
| Application de bureau | Tauri 2 |
| Cœur, CLI, serveur MCP | Rust (le cœur est une bibliothèque partagée) |
| Interface | Svelte 5 + TypeScript |
| Vue orbitale | PixiJS v8 (WebGPU si disponible, sinon WebGL 2) |
| Traductions de l'interface | Paraglide JS (français en référence) |
| Règles des propriétés | CEL |
| Fichiers des projets | YAML + schémas JSON |
| Protocole de débogage | JSON-RPC 2.0 sur WebSocket, décrit en OpenRPC |
| Rapports de tests | CTRF (JUnit XML accepté) |

Toute logique métier (modèle, validation, ordonnancement, simulation, génération de données) vit
dans le **cœur Rust**. L'interface ne fait qu'afficher et transmettre les actions.

## Langue

- La documentation, les exigences, les scénarios et l'interface sont en **français**.
- Le code, les identifiants et les clés des fichiers YAML sont en **anglais**.

## Scénarios actifs

- `cargo test` exécute **tous** les scénarios de `features/` (test `scenarios_actifs`).
- Les scénarios listés dans [`features/actifs.yaml`](features/actifs.yaml) sont **actifs** : ils
  doivent passer. Une étape sans fonction ou un échec fait échouer `cargo test`.
- Les autres sont seulement rapportés (passent, échouent, non implémentés). Quand un scénario
  inactif passe, le rapport demande de l'activer : on l'ajoute alors à `actifs.yaml`.
- **Un scénario actif ne redevient jamais inactif.** Le retirer de `actifs.yaml` revient à
  désactiver un test.
- Le test `traceabilite` vérifie que chaque exigence est citée par au moins un scénario et que
  chaque scénario ne cite que des exigences existantes.

## Commandes

```sh
cargo test --workspace                                       # tous les tests, scénarios compris
cargo test -p ecs-nide-specs --test scenarios -- --nocapture # rapport d'avancement des scénarios
cargo clippy --workspace --all-targets -- -D warnings        # aucun avertissement toléré
cargo fmt --all
```

## Organisation du code

| Crate | Rôle |
|---|---|
| `crates/spec-markdown` | Parseur et exécuteur du format Spec Markdown |
| `crates/ecs-nide-core` | Le cœur : modèle, dossier projet (chargement, enregistrement, création), renommage et suppression, ordonnancement, contrats, validation |
| `crates/ecs-nide-specs` | Les fonctions des étapes de `features/` et les tests de traçabilité |
| `examples/space-shooter` | Le projet d'exemple décrit dans `features/README.md` |

## État actuel

Phase 0 (spécifications) : **terminée**. Phase 1 (cœur Rust) : **en cours**. Faits : dossier projet,
modélisation, ordonnancement, validation, contrats. Suite : vérification des exemples et des
propriétés (CEL), puis la CLI (phase 2).
