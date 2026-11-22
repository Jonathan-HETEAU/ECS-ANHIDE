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

## Choix techniques

| Partie | Technologie |
|---|---|
| Application de bureau | Tauri 2 |
| Cœur, CLI, serveur MCP | Rust (le cœur est une bibliothèque partagée) |
| Interface | Svelte 5 + TypeScript |
| Règles des propriétés | CEL |
| Fichiers des projets | YAML + schémas JSON |

Toute logique métier (modèle, validation, ordonnancement, simulation, génération de données) vit
dans le **cœur Rust**. L'interface ne fait qu'afficher et transmettre les actions.

## Langue

- La documentation, les exigences, les scénarios et l'interface sont en **français**.
- Le code, les identifiants et les clés des fichiers YAML sont en **anglais**.

## État actuel

Phase 0 : les spécifications sont en cours de validation. Aucun code n'est écrit tant qu'elles ne
sont pas validées.
