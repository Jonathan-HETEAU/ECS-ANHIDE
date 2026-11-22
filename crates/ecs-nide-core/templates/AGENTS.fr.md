# AGENTS.md — Travailler sur le projet ECS NIDE {{name}}

Ce dossier est un projet **ECS NIDE** : la description déclarative d'un jeu construit sur
l'architecture Entité-Composant-Système. Il ne contient aucun code de jeu. Le jeu est implémenté
ailleurs, dans le moteur cible, à partir des **contrats** décrits ici.

## Organisation du dossier

| Chemin | Contenu |
|---|---|
| `ecs-nide.yaml` | Nom du projet, version du format, moteur cible, langue |
| `schedule.yaml` | Phases et ordre de référence des systèmes |
| `schedule.plan.yaml` | Plan d'exécution calculé par ECS NIDE — ne jamais le modifier à la main |
| `components/` | Un fichier YAML par composant : champs typés et contraintes |
| `resources/` | Un fichier YAML par ressource (donnée globale unique) |
| `messages/` | Un fichier YAML par canal de messages |
| `events/` | Un fichier YAML par événement |
| `relationships/` | Un fichier YAML par relation entre entités |
| `archetypes/` | Un fichier YAML par archétype (combinaison de composants) |
| `systems/<nom>/` | `system.yaml` (contrat), `spec.md` (spécification), `examples.yaml`, `properties.yaml` |
| `observers/<nom>/` | Les mêmes fichiers qu'un système, avec `observer.yaml` et son déclencheur |
| `skills/` | Les skills du moteur cible : comment traduire ce modèle dans ce moteur |

## Méthode de travail

1. **Récupérer le contrat** du système à implémenter : `ecs-nide contract <Système>`, ou l'outil
   `contract` du serveur MCP d'ECS NIDE. Il contient tout le nécessaire : spécification, accès,
   schémas, exemples, propriétés et moteur cible.
2. **Suivre le skill** du moteur cible dans `skills/` pour traduire chaque concept.
3. **Les tests d'abord.** Traduire chaque exemple et chaque propriété en test dans le moteur, le
   voir échouer, puis implémenter le système.
4. **Rapporter les résultats** dans un rapport CTRF et l'importer :
   `ecs-nide import-results <rapport>`.
5. **Tout bug est un test manquant.** Quand un comportement est faux, ajouter d'abord l'exemple
   qui le reproduit dans `examples.yaml`, puis corriger le code.

## Règles

- Un système décrit **ce qu'il fait**, jamais **comment** : aucun code dans les fichiers YAML.
- Lancer `ecs-nide validate` après chaque modification du modèle : aucune erreur ne doit rester.
- Une ambiguïté signalée par ECS NIDE est une décision de conception à prendre : ajouter une
  contrainte d'ordre ou une phase, ne pas la laisser en suspens.
