# Les skills propres à chaque moteur

Le modèle ne dépend d'aucun langage. Pour qu'une IA le traduise correctement dans un moteur
donné, ECS NIDE installe dans le projet des **skills** (format Agent Skills, fichiers `SKILL.md`)
propres au moteur cible.

## Choisir un moteur installe ses skills dans le projet
Exigences : SKL-001, SKL-002
* Créer le projet "Asteroides"
* Choisir le moteur cible "bevy"
* Le dossier "skills/bevy" contient un fichier "SKILL.md"
* "ecs-nide.yaml" indique le moteur cible "bevy"

## Un skill explique la traduction de chaque concept du modèle
Exigences : SKL-003
* Lire le skill "bevy"
* Le skill explique la traduction des concepts :

  | concept                    |
  |----------------------------|
  | composant                  |
  | marqueur                   |
  | ressource                  |
  | accès read, write, optional, with, without |
  | changements structurels    |
  | archétype                  |
  | étapes et contraintes d'ordre |

## Un skill explique comment écrire et rapporter les tests
Exigences : SKL-004
* Lire le skill "bevy"
* Le skill explique comment traduire les exemples et les propriétés en tests du moteur
* Le skill explique comment produire le rapport de résultats au format neutre

## Un skill explique comment intégrer la sonde de débogage
Exigences : SKL-005
* Une sonde de débogage existe pour le moteur "bevy"
* Le skill "bevy" explique comment l'intégrer au jeu

## Changer de moteur remplace les skills
Exigences : SKL-006
* Le projet a le moteur cible "bevy"
* Choisir le moteur cible "flecs"
* Le dossier "skills/flecs" existe et le dossier "skills/bevy" n'existe plus

## Un skill modifié n'est jamais écrasé sans confirmation
Exigences : SKL-007
* Le projet a le moteur cible "bevy"
* Modifier le fichier "skills/bevy/SKILL.md"
* Choisir le moteur cible "flecs"
* L'IDE demande confirmation avant de supprimer le skill "bevy" modifié
