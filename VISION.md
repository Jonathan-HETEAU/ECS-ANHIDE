# ECS NIDE — Vision et attentes

> Ce document décrit **ce qu'on attend** d'ECS NIDE dans les grandes lignes.
> Le détail vit ailleurs :
> - [`features/`](features/) : les spécifications fluides, scénario par scénario (format *Spec Markdown*) ;
> - [`exigences/`](exigences/) : le fonctionnement attendu, phrase par phrase (format EARS) ;
> - [`AGENTS.md`](AGENTS.md) : la méthode de travail (tout part des tests).

## 1. En une phrase

ECS NIDE est un **IDE de bureau, hors ligne**, pour concevoir un jeu vidéo en architecture
**Entité-Composant-Système de manière déclarative**. On y décrit les composants, les systèmes,
leurs contrats, leurs spécifications et leurs tests **sans écrire le code du jeu**. Ce code est
ensuite produit à l'extérieur, par une IA ou un humain, pour **n'importe quel langage et
n'importe quelle bibliothèque ECS**.

## 2. Pourquoi

| Objectif | Ce que ça veut dire |
|---|---|
| **Clarté visuelle** | Voir d'un coup d'œil quelles entités passent par quels systèmes, dans quel ordre, et ce qui tourne en parallèle. |
| **Débogage facile** | Avancer pas à pas, poser des points d'arrêt, activer ou désactiver des systèmes, sur le modèle comme sur le jeu réel. |
| **Travail assisté par IA optimisé** | Chaque système est une unité autonome (contrat + spécification + tests) qu'on peut confier à une IA. |
| **Indépendance** | Le modèle ne dépend d'aucun langage ni d'aucune bibliothèque ECS. Ce qui est propre à un moteur vit dans les *skills*. |
| **Performance extrême** | Pensé pour des jeux de plusieurs millions d'entités. La parallélisation des systèmes est calculée à partir des contrats. |

## 3. La métaphore

| Image | Concept |
|---|---|
| **Le trou noir** | Le projet, le centre autour duquel tout tourne. |
| **Un tour d'orbite** | Une boucle du jeu (une frame). Tous les systèmes s'exécutent pendant ce tour. |
| **Les planètes** | Les systèmes, placés sur l'orbite dans l'ordre d'exécution. |
| **Les orbites concentriques** | Les systèmes d'une même étape, exécutés en parallèle. |
| **Les vaisseaux** | Les entités, reconnaissables aux composants qu'elles transportent. |
| **Les flottes** | Les archétypes (les entités qui ont exactement la même combinaison de composants). À grande échelle, on visualise des flottes, pas des vaisseaux. |

Une flotte qui fait le tour complet s'arrête à chaque planète dont le système la concerne.
Quand elle revient au point de départ, une frame s'est écoulée.

## 4. Principes non négociables

1. **Le dossier projet fait foi.** Un projet est un dossier de fichiers texte, versionnable avec git et
   ouvrable dans VS Code. L'IDE, le serveur MCP, la ligne de commande et les outils externes
   lisent et écrivent les mêmes fichiers.
2. **Déclaratif.** Un système dit *ce qu'il fait* (ses entrées, ses sorties, ses règles), jamais *comment*.
   Le format ne prévoit aucun champ pour du code.
3. **Tout est testé.** Chaque système a une spécification *et* des tests. Un bug est toujours le
   signe d'un test manquant.
4. **Indépendance vis-à-vis du langage.** Les types, les tests et les règles utilisent des notations neutres
   (YAML, CEL, JSON). Tout ce qui est propre à un moteur vit dans les skills.
5. **L'IDE ne génère pas de code.** Il produit des contrats ; le code est écrit à l'extérieur.
6. **Bureau et hors ligne.** Aucune fonctionnalité ne dépend d'un service en ligne.
7. **Prévu pour l'échelle.** Le débogage réel et la visualisation fonctionnent avec des millions d'entités.
8. **L'IDE signale, l'utilisateur décide.** L'IDE détecte, calcule et signale : conflits,
   ambiguïtés, cycles, incohérences. Les choix de conception du jeu (ordre des systèmes, phases,
   contraintes…) appartiennent à l'utilisateur, humain ou IA. Quand l'IDE doit quand même produire
   un résultat, par exemple ordonner deux systèmes ambigus, il applique une règle déterministe et
   documentée, et il le signale.

## 5. Les concepts du modèle

| Concept | Définition |
|---|---|
| **Composant** | Une donnée typée attachée à une entité. Sans champ, c'est un **marqueur** (un tag). Un composant **activable** s'active ou se désactive sans changement structurel. |
| **Ressource** | Une donnée globale unique (le temps, les entrées clavier, la configuration…). |
| **Canal de messages** | Un flux de données typées, émises par des systèmes et lues par d'autres pendant la même boucle, puis effacées à la fin de la boucle. |
| **Événement** | Un fait ponctuel, éventuellement attaché à une entité, déclenché par un système ou un observateur. Un événement d'entité peut se propager le long d'une relation. |
| **Relation** | Un lien entre entités, à la façon de Bevy : un composant source (`ChildOf`) désigne une entité cible, dont le composant cible (`Children`) liste les sources, tenu à jour automatiquement. |
| **Système** | Une règle qui s'applique à toutes les entités qu'il sélectionne, à chaque boucle. |
| **Observateur** | Une réaction à un déclencheur : un événement, ou l'ajout, l'insertion, le remplacement, le retrait ou la destruction d'un composant. Il s'exécute au point de synchronisation de fin d'étape, donc toujours dans la boucle. |
| **Archétype** | Une combinaison de composants, accompagnée d'un volume estimé d'entités. |
| **Phase** | Une grande section facultative de la boucle (entrées, logique, rendu…), dessinée comme un secteur de l'orbite. |
| **Étape** | Un groupe de systèmes sans conflit entre eux, exécutés en parallèle. |
| **Plan d'exécution** | La suite des étapes d'une boucle, calculée par ECS NIDE. |
| **Contrat** | Tout ce qu'une IA doit savoir pour implémenter un système, réuni en un seul bloc. |
| **Profil de moteur** | Les capacités du moteur cible (parallélisme entre systèmes, ensembles disjoints, composants activables…), fournies avec ses skills. Le plan d'exécution s'y adapte. |

## 6. Anatomie d'un système

Un système est un dossier composé de quatre fichiers :

```
systems/damage/
├── system.yaml       # le contrat : accès, changements structurels, ordre
├── spec.md           # la spécification fluide, c'est-à-dire le prompt de l'IA
├── examples.yaml     # les tests par exemples
└── properties.yaml   # les tests par propriétés
```

### 6.1 Le contrat : `system.yaml`

```yaml
name: Damage
description: Applique aux points de vie les dégâts reçus pendant la frame.
phase: Resolution
per_entity: true         # chaque entité est traitée à partir de ses seules données
every: 1                 # fréquence : une boucle sur N (1 par défaut)
components:
  Health: write          # lecture et écriture
  Hit: read              # lecture seule
  Invulnerable: without  # filtre : l'entité ne doit pas l'avoir
changed: []              # filtre de changement (facultatif)
messages:
  DamageTaken: may_emit  # émission possible sur un canal de messages
structural:
  remove: [Hit]          # certain : retiré à chaque passage
  may_add: [Dead]        # possible : ajouté seulement dans certains cas
toggles: {}              # activations / désactivations de composants activables
after: [Collision]       # contrainte d'ordre
```

- **Composants** : `read`, `write`, `optional` (lecture si présent), `with` (présence requise,
  sans accès aux données) et `without` (absence requise). Un composant activable désactivé
  compte comme absent.
- **Ressources** : `read` ou `write`.
- **Canaux de messages** : `emit` (émission certaine), `may_emit` (émission possible) ou `read`.
- **Filtre de changement** (`changed`) : le système ne traite que les entités dont l'un des
  composants filtrés a été ajouté ou écrit depuis son précédent passage.
- **Changements structurels** : ajout ou retrait de composants, création d'entités d'un archétype,
  destruction d'entités. Chacun est **certain** ou **possible**.
- **Activations et désactivations** (`toggles`) de composants activables : `enable`, `disable`,
  `may_enable` ou `may_disable`. Ce ne sont **pas** des changements structurels.
- **`per_entity`** : vrai si le système traite chaque entité indépendamment des autres. L'IA sait
  alors qu'elle peut répartir ses entités entre tous les cœurs. Faux par défaut.
- **`phase`** et **`every`** : la phase du système et sa fréquence.

Un **observateur** se décrit exactement de la même façon, dans `observers/<nom>/`, avec un
`observer.yaml` qui ajoute son déclencheur :

```yaml
name: ScoreOnDeath
on: { lifecycle: add, component: Dead }   # ou { event: EnemyKilled }
components:
  Enemy: with
resources:
  GameState: write
triggers:
  EnemyKilled: may_trigger
```

### 6.2 La spécification : `spec.md`

Elle est rédigée dans le même format fluide que nos propres `features/` (voir
[`features/README.md`](features/README.md)). C'est **le prompt** qui exprime l'intention.
Chaque scénario renvoie aux exemples qui l'illustrent.

```markdown
# Damage

Les dégâts reçus pendant la frame sont retirés des points de vie, puis effacés.

## Les dégâts réduisent les points de vie
Exemples : basique
* Une entité à "100" points de vie reçoit un coup de "30"
* Elle termine la frame à "70" points de vie, sans coup en attente

## Les points de vie ne descendent jamais sous zéro
Exemples : plancher-zero
Propriétés : sante-positive
* Une entité à "10" points de vie reçoit un coup de "50"
* Elle termine à "0" point de vie et elle est marquée morte
```

### 6.3 Les tests par exemples : `examples.yaml`

```yaml
- id: basique
  given:
    Health: { current: 100, max: 100 }
    Hit: { amount: 30 }
  expect:
    Health: { current: 70, max: 100 }
  expect_absent: [Hit]
  expect_messages:
    DamageTaken: [{ amount: 30 }]

- id: plancher-zero
  given:
    Health: { current: 10, max: 100 }
    Hit: { amount: 50 }
  expect:
    Health: { current: 0, max: 100 }
    Dead: {}
  expect_absent: [Hit]
```

Un exemple peut aussi décrire :

| Clé | Sens |
|---|---|
| `given_resources` / `expect_resources` | les ressources lues avant, et écrites après |
| `given_messages` / `expect_messages` | les messages reçus avant, et émis après |
| `disabled` / `expect_disabled` / `expect_enabled` | l'état des composants activables |
| `expect_absent` | les composants retirés |
| `expect_spawned` | les archétypes des entités créées |
| `expect_despawned` | `true` si l'entité est détruite |

### 6.4 Les tests par propriétés : `properties.yaml`

Une propriété est une règle qui doit rester vraie pour **toutes** les données d'un domaine.
Les données sont générées à partir des schémas des composants, de leurs contraintes et du
domaine de la propriété. Les règles sont écrites en **CEL** (*Common Expression Language*),
un langage d'expressions neutre disponible en Rust, Go, C++, Java, C# et JavaScript.

```yaml
- id: sante-positive
  description: Les points de vie ne deviennent jamais négatifs.
  domain:
    Hit.amount: { min: 0, max: 100000 }
  ensure: after.Health.current >= 0.0

- id: sante-ne-remonte-pas
  ensure: after.Health.current <= before.Health.current
```

## 7. Le dossier projet

```
asteroides/
├── ecs-nide.yaml          # nom, version du format, moteur cible, langue des fichiers générés
├── schedule.yaml          # phases et ordre de référence des systèmes
├── schedule.plan.yaml     # plan d'exécution calculé (généré)
├── components/            # un fichier YAML par composant
├── resources/             # un fichier YAML par ressource
├── messages/              # un fichier YAML par canal de messages
├── events/                # un fichier YAML par événement
├── relationships/         # un fichier YAML par relation
├── archetypes/            # un fichier YAML par archétype
├── systems/               # un dossier par système (voir § 6)
├── observers/             # un dossier par observateur (voir § 6)
├── skills/<moteur>/       # skills du moteur cible (SKILL.md) et son profil (profile.yaml)
├── AGENTS.md              # instructions pour les IA (généré)
└── .vscode/mcp.json       # démarre le serveur MCP depuis VS Code (généré)
```

Un composant :

```yaml
name: Health
description: Points de vie.
fields:
  current: { type: f32, min: 0, max: 1000, default: 100 }
  max:     { type: f32, min: 1, max: 1000, default: 100 }
```

Un composant activable, stocké de façon dispersée :

```yaml
name: Stunned
description: L'entité est étourdie et ne peut pas agir.
toggleable: true
storage: sparse          # dense (par défaut) ou sparse
fields:
  remaining: { type: f32, min: 0, default: 0 }
```

Une relation, à la façon de Bevy :

```yaml
name: ChildOf
target_component: Children   # liste des entités sources, tenue à jour automatiquement
linked_spawn: true           # détruire la cible détruit ses sources
fields: {}                   # la relation peut porter des données
```

Un système lit les composants de la cible d'une relation avec un accès `via` :

```yaml
components:
  ChildOf: read
  GlobalTransform: write
via:
  ChildOf: { GlobalTransform: read }
```

L'ordre de référence et les phases :

```yaml
phases: [Entrees, Logique, Physique, Resolution, Rendu]
order: [PlayerInput, EnemyAI, Weapon, Movement, Collision, Damage, Lifetime, Death, Render]
```

Un archétype :

```yaml
name: Asteroid
components: [Position, Velocity, Health, Collider, Enemy]
estimated_count: 50000
```

Un archétype peut créer des entités liées, comme `children!` dans Bevy :

```yaml
name: Car
components: [Position, Velocity]
related:
  - { relationship: ChildOf, archetype: Wheel, count: 4 }
```

## 8. L'ordonnancement

- **Une seule boucle.** Tous les systèmes activés s'exécutent pendant la boucle, une fois par tour
  ou une boucle sur N selon leur fréquence.
- **Ordre de référence.** `schedule.yaml` donne l'ordre de déclaration. Les contraintes `before`
  et `after` de chaque système le précisent.
- **Phases.** Si des phases sont déclarées, chaque système appartient à une phase, et une phase ne
  commence que lorsque la précédente est terminée.
- **Lire et écrire.** Écrire une donnée, c'est déclarer `write` sur un composant ou une ressource,
  activer ou désactiver un composant activable, ou émettre sur un canal. Lire, c'est déclarer
  `read` ou `optional`, ou `with` / `without` sur un composant activable (la sélection dépend alors
  de son état).
- **Conflits.** Deux systèmes sont en conflit si l'un écrit une donnée que l'autre lit ou écrit.
  Deux systèmes en conflit ne sont jamais dans la même étape.
- **Messages.** Un système qui émet sur un canal passe toujours avant ceux qui le lisent. C'est une
  contrainte d'ordre implicite, qui évite les ambiguïtés.
- **Parallélisation calculée.** Chaque système est placé dans la première étape compatible avec
  ses conflits, ses contraintes d'ordre et sa phase. Les systèmes d'une même étape se retrouvent
  sur des orbites concentriques.
- **Profil de moteur.** Le plan s'adapte aux capacités du moteur cible :
  - si le moteur gère les **ensembles disjoints**, deux systèmes dont l'un exige `with: X` et
    l'autre `without: X` ne sont pas en conflit (seul Bevy le fait aujourd'hui) ;
  - si le moteur **n'exécute pas de systèmes en parallèle** (cas de Flecs), chaque système a sa
    propre étape : une seule orbite ;
  - sans moteur choisi, le **profil générique** s'applique : parallélisme entre systèmes, mais pas
    d'ensembles disjoints.
- **Ambiguïtés.** Deux systèmes en conflit sans contrainte d'ordre entre eux sont signalés.
  ECS NIDE les ordonne alors selon l'ordre de déclaration.
- **Changements structurels différés.** Les ajouts, retraits, créations et destructions sont
  appliqués à la fin de l'étape. Les activations et désactivations, elles, sont immédiates.
- **Observateurs au point de synchronisation.** Les observateurs n'occupent aucune étape. Ils
  s'exécutent au point de synchronisation de fin d'étape, dans un ordre déterministe, en cascade
  jusqu'à stabilité. Le plan indique, pour chaque étape, les observateurs qu'elle peut déclencher,
  et une cascade circulaire possible est signalée.
- Le plan est **déterministe** et exporté dans `schedule.plan.yaml` pour les outils externes.

## 9. Simulation et débogage

### 9.1 Mode modèle : simuler sans code

- On peuple le monde à partir des archétypes, avec des valeurs tirées des exemples ou générées
  depuis les schémas.
- Les flottes parcourent l'orbite étape par étape.
- Quand l'état d'une entité correspond exactement à l'entrée d'un exemple, on applique la sortie
  de cet exemple.
- Sinon, on applique les changements structurels, activations, désactivations et émissions
  **certains**, on signale ceux qui sont **possibles** et on marque comme **indéterminés** les
  champs écrits.
- Les messages émis pendant une boucle sont lus par les étapes suivantes, puis effacés à la fin de
  la boucle.
- Un système de fréquence N ne s'exécute que lors des boucles 0, N, 2N…
- Un système doté d'un filtre de changement ne voit que les entités modifiées depuis son précédent
  passage.
- À chaque fin d'étape, les observateurs réagissent en cascade aux changements et aux événements.
  Une cascade de plus de 64 générations est interrompue et signalée.
- Les relations se tiennent à jour toutes seules, la destruction d'une cible détruit ses sources
  quand la relation le demande, et un événement d'entité peut remonter le long d'une relation.
- On dispose du pas à pas (une étape ou une boucle), du retour en arrière, des points d'arrêt,
  de l'activation et de la désactivation de systèmes, et de la modification d'une entité pendant
  la pause.

### 9.2 Mode réel : se brancher sur le jeu

- Le jeu embarque une **sonde** propre à sa bibliothèque ECS et dialogue avec l'IDE via le
  **protocole de débogage d'ECS NIDE** :
  - **JSON-RPC 2.0**, la même base que MCP et que le protocole distant de Bevy ;
  - transporté par **WebSocket** en local, pour pousser une frame à chaque boucle et recevoir les
    commandes de l'IDE ;
  - décrit par un document **OpenRPC** publié, qui sert à valider chaque message ;
  - un encodage binaire (MessagePack) pourra être négocié plus tard si les mesures le justifient.
- Les sondes s'appuient sur l'existant de chaque moteur, par exemple `bevy_remote` et `Stepping`
  pour Bevy, ou le module REST pour Flecs.
- L'IDE compare le plan réel au plan déclaré et signale les divergences.
- La vue montre les flottes (une par archétype) avec leurs effectifs et le temps passé dans
  chaque système.
- Les commandes sont transmises à la sonde : pause, pas à pas, points d'arrêt, activation et
  désactivation de systèmes, inspection d'un échantillon d'entités, modification de valeurs.
- Le volume de données échangé par frame ne dépend pas du nombre d'entités.
- Une entité capturée avant et après un système devient un **nouvel exemple de test** en un clic :
  un bug observé se transforme directement en test.

## 10. L'IA : dossier externe, serveur MCP et skills

- **Dossier externe.** Le projet s'ouvre dans VS Code ou n'importe quel éditeur. `AGENTS.md`
  explique aux IA la structure du dossier et la méthode de travail.
- **Serveur MCP autonome.** La commande `ecs-nide mcp` démarre le serveur, sans que l'IDE soit
  ouvert. Il permet de lire le modèle, d'obtenir le contrat d'un système, de modifier le projet
  (modifications validées, et refusées si elles rendraient le projet invalide), de valider,
  de calculer le plan, de générer des données de test et d'importer des résultats de tests.
- **La CLI** `ecs-nide` propose les mêmes opérations en ligne de commande, pour l'intégration
  continue.
- **Skills par moteur.** Quand on choisit un moteur cible (Bevy, Flecs, EnTT, Unity DOTS…), l'IDE
  installe ses skills dans `skills/`. Ils expliquent comment traduire chaque concept du modèle
  vers ce moteur (et quel équivalent utiliser quand le moteur ne le connaît pas), comment
  transformer exemples et propriétés en tests (par exemple avec `proptest` pour Bevy), comment
  produire le rapport de résultats et comment intégrer la sonde de débogage. Ils fournissent aussi
  le **profil du moteur** (`profile.yaml`).
- **Bevy est le premier moteur cible.** Son profil prend en charge le parallélisme entre systèmes,
  les ensembles disjoints, les filtres de changement, la fréquence, les observateurs et les
  relations. Il ne prend pas en charge les composants activables, et le skill Bevy indique
  l'équivalent à utiliser.
- **Rapports de tests.** Le format neutre est **CTRF** (JSON avec un schéma normatif), avec des
  champs d'extension d'ECS NIDE : système, type de test, identifiant, contre-exemple, graine
  (schéma publié : `crates/ecs-nide-core/schemas/ctrf-extension.schema.json`). Les données de test
  générées suivent elles aussi un schéma publié (`schemas/test-data.schema.json`).
  **JUnit XML** est accepté aussi, grâce à la convention de nommage
  `ecsnide::<Système>::example::<id>` ou `ecsnide::<Système>::property::<id>`.
- **Langue des fichiers générés.** `AGENTS.md` et les skills sont rédigés dans la langue du projet,
  l'anglais par défaut.
- **Boucle de travail type** : l'humain décrit un système → l'IA récupère son contrat via MCP →
  l'IA implémente le système et ses tests dans le moteur, en suivant le skill → les résultats
  remontent dans l'IDE → un échec, ou un bug observé, devient un nouveau test.

## 11. Architecture technique

| Partie | Technologie |
|---|---|
| Application de bureau | **Tauri 2** |
| Cœur : modèle, validation, ordonnancement, simulation, générateurs, protocole | **Rust** (bibliothèque partagée) |
| Serveur MCP et CLI | **Rust**, un seul binaire `ecs-nide`, réutilisant le cœur |
| Interface | **Svelte 5 + TypeScript** |
| Composants d'interface | **UI SDK** de Vercel Labs (`@ui-sdk/svelte`), version exacte figée tant qu'il est en bêta ; style par **variables CSS**, sans Tailwind |
| Vue orbitale | **PixiJS v8** : WebGPU quand il est disponible, sinon WebGL 2 |
| Traductions de l'interface | **Paraglide JS**, le français étant la langue de référence |
| Règles des propriétés | **CEL** |
| Format des fichiers | **YAML**, accompagné de schémas JSON |
| Protocole de débogage | **JSON-RPC 2.0 sur WebSocket**, décrit en **OpenRPC** |
| Rapports de tests | **CTRF**, et JUnit XML accepté |

L'IDE, la CLI et le serveur MCP passent tous par le **même cœur Rust**. Une opération donne donc
le même résultat partout.

## 12. Méthode de développement d'ECS NIDE lui-même

On applique à notre projet ce qu'ECS NIDE demande à ses utilisateurs :

1. **Ce document** décrit les attentes générales.
2. **`exigences/`** (EARS) énonce chaque comportement attendu, en une phrase avec un identifiant
   (`ORD-002`…).
3. **`features/`** (Spec Markdown) illustre les exigences par des scénarios exécutables.
4. **Les tests** sont écrits avant le code et référencent les identifiants d'exigences.
5. **Tout bug est un test manquant.** On ajoute d'abord le test qui reproduit le bug (et, si
   besoin, l'exigence et le scénario qui manquaient), puis on corrige.

Le parseur de *Spec Markdown* écrit en Rust sert deux fois : il exécute nos `features/`, et il lit
les `spec.md` des systèmes dans l'IDE.

## 13. Feuille de route proposée

| Phase | Contenu |
|---|---|
| 0 | Spécifications (ce document, `features/`, `exigences/`) **validées** — terminée |
| 1 | Cœur Rust : format du dossier projet, modèle, validation, ordonnancement, avec leurs tests — terminée (hors scénarios qui dépendent de l'IDE ou de la simulation) |
| 2 | CLI `ecs-nide` : `new`, `validate`, `plan`, `contract`, `gen-tests`, `import-results` — terminée |
| 3 | Tests : exemples, propriétés CEL, générateurs, cohérence entre exemples et propriétés — faite avec la phase 1 |
| 4 | Serveur MCP + génération de `AGENTS.md` et `.vscode/mcp.json` |
| 5 | IDE Tauri/Svelte : vue orbitale (prototype de rendu PixiJS mesuré d'abord sur Windows, macOS et Linux), éditeurs, diagnostics |
| 6 | Simulation en mode modèle dans l'IDE |
| 7 | Skills du premier moteur (Bevy proposé) |
| 8 | Protocole de débogage réel + sonde du premier moteur |

## 14. Décisions et questions ouvertes

### Décisions du 22 novembre 2026

Elles s'appuient sur l'étude des principaux moteurs ECS :
[`recherche/etat-de-l-art-ecs.md`](recherche/etat-de-l-art-ecs.md).

| Sujet | Décision |
|---|---|
| Événements | **Canaux de messages** (`emit` / `may_emit` / `read`), effacés à chaque boucle, l'émetteur passant toujours avant les lecteurs. Les observateurs sont reportés. |
| Ensembles disjoints | Calculés dans le cœur, utilisés seulement si le **profil du moteur** le permet |
| Parallélisme de données | Attribut **`per_entity`** sur chaque système |
| Protocole de débogage | **JSON-RPC 2.0 sur WebSocket**, décrit en **OpenRPC** ; MessagePack plus tard si les mesures le justifient |
| Rapports de tests | **CTRF**, et JUnit XML avec une convention de nommage |
| Rendu | **PixiJS v8** (WebGPU si disponible, sinon WebGL 2), validé par un prototype mesuré |
| Langue | Interface **internationalisée** (Paraglide JS), français en référence ; fichiers générés dans la langue du projet, l'anglais par défaut |
| Composants d'interface (25 novembre 2026) | **UI SDK** de Vercel Labs, évalué dans `prototypes/ui-sdk-svelte` : composants sans style imposé, accessibles, pont pour agents ; version figée tant qu'il est en bêta, choix confirmé au début de la phase 5 ; style par variables CSS, sans Tailwind |
| Nouveaux concepts | Composants **activables**, filtre **`changed`**, **phases**, **fréquence** (`every`), indication de **stockage** |
| Observateurs et relations | **Comme Bevy** : événements, observateurs (exécutés au point de synchronisation de fin d'étape), relations avec composant cible tenu à jour, destruction liée, accès `via`, entités liées des archétypes |
| Premier moteur cible | **Bevy** : skills, profil et sonde de débogage |

### Questions encore ouvertes

- **Points à vérifier** dans les documentations officielles : ceux marqués ⚠ dans l'étude.
