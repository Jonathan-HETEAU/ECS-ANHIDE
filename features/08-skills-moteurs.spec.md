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

  | concept                                      |
  |----------------------------------------------|
  | composant                                    |
  | marqueur                                     |
  | composant activable                          |
  | indication de stockage                       |
  | ressource                                    |
  | canal de messages                            |
  | événement                                    |
  | observateur                                  |
  | relation et accès via                        |
  | accès read, write, optional, with, without   |
  | filtre de changement                         |
  | changements structurels                      |
  | activations et désactivations                |
  | archétype                                    |
  | phases, étapes et contraintes d'ordre        |
  | fréquence                                    |
  | attribut per_entity                          |

* Pour chaque concept que le profil de "bevy" ne prend pas en charge, le skill indique l'équivalent à utiliser

## Un skill explique comment écrire et rapporter les tests
Exigences : SKL-004
* Lire le skill "bevy"
* Le skill explique comment traduire les exemples et les propriétés en tests du moteur
* Le skill explique comment produire le rapport de résultats au format CTRF avec les champs d'extension d'ECS NIDE

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

## Les skills fournissent le profil des capacités du moteur
Exigences : SKL-008
* Choisir le moteur cible "flecs"
* Le fichier "skills/flecs/profile.yaml" indique pour chaque capacité si elle est prise en charge :

  | capacité                          |
  |-----------------------------------|
  | parallélisme entre systèmes       |
  | ensembles disjoints               |
  | composants activables             |
  | filtres de changement             |
  | fréquence des systèmes            |
  | observateurs                      |
  | relations                         |

## Sans moteur cible, le profil générique s'applique
Exigences : SKL-009
* Créer le projet "Asteroides" sans choisir de moteur cible
* Le profil utilisé prend en charge le parallélisme entre systèmes
* Le profil utilisé ne prend pas en charge les ensembles disjoints
* Le profil utilisé prend en charge les composants activables, les filtres de changement, la fréquence, les observateurs et les relations

## Bevy est le premier moteur cible fourni
Exigences : SKL-010
* Choisir le moteur cible "bevy"
* Le dossier "skills/bevy" contient "SKILL.md" et "profile.yaml"
* Le profil de "bevy" prend en charge le parallélisme entre systèmes, les ensembles disjoints, les filtres de changement, la fréquence, les observateurs et les relations
* Le profil de "bevy" ne prend pas en charge les composants activables, et le skill indique leur équivalent
* Le skill de "bevy" explique comment intégrer la sonde de débogage fournie pour Bevy
