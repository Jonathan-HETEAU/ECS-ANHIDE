# État de l'art des moteurs ECS — réponses aux questions ouvertes

> Étude réalisée en novembre 2026 pour trancher les questions ouvertes de [`VISION.md`](../VISION.md#14-décisions-et-questions-ouvertes).
> Chaque section se termine par une **recommandation**.
>
> **Statut : toutes les recommandations ont été retenues le 22 novembre 2026.** Elles sont
> intégrées dans `VISION.md`, `exigences/` et `features/`.

## Méthode et limites

**Moteurs étudiés**

| Moteur | Langage | Pourquoi |
|---|---|---|
| **Bevy** (0.19) | Rust | L'ECS le plus avancé sur l'ordonnancement automatique et le parallélisme entre systèmes |
| **Flecs** (v4) | C / C++ | L'ECS le plus riche en concepts (relations, observateurs, phases), avec un outil de débogage distant de référence |
| **Unity Entities** (DOTS) | C# | Le plus utilisé en production pour des millions d'entités |
| **Unreal Mass** | C++ | L'approche d'Epic pour les foules et les simulations massives |
| **EnTT** | C++ | La bibliothèque minimaliste de référence, sans ordonnanceur imposé |

**Limites.** Les sites de documentation officiels (flecs.dev, bevy.org, docs.rs,
docs.unity3d.com, dev.epicgames.com) sont bloqués par la politique réseau de la session où cette
étude a été rédigée. Les faits viennent donc de recherches web, croisées avec la connaissance
préalable de ces moteurs. Les points marqués **⚠** restent à vérifier dans la documentation
officielle.

## Vue d'ensemble

| | Bevy | Flecs | Unity Entities | Unreal Mass | EnTT |
|---|---|---|---|---|---|
| **Ordre des systèmes** | Contraintes `before` / `after` / `chain`, ensembles de systèmes, plusieurs *schedules* | Phases intégrées (OnLoad → OnStore) et ordre de déclaration dans chaque phase | Groupes de systèmes, `UpdateBefore` / `UpdateAfter`, `OrderFirst` / `OrderLast` | Phases de traitement, `ExecuteBefore` / `ExecuteAfter` / `ExecuteInGroup` | Aucun ordonnanceur ; l'`organizer` construit un graphe |
| **Parallélisme entre systèmes** | Oui, calculé à partir des accès | Non ⚠ : les systèmes s'exécutent l'un après l'autre | Indirect : ce sont les *jobs* qui se chevauchent, avec des dépendances par type de composant | Oui, à partir d'un graphe de dépendances ⚠ | À la charge de l'utilisateur, via le graphe de l'`organizer` |
| **Parallélisme dans un système** (découpage des entités) | `par_iter` | Option `multi_threaded` : les entités sont réparties entre les threads | `ScheduleParallel` | Traitement parallèle par *chunk* | Manuel |
| **Ensembles disjoints** (`with` / `without`) | **Oui** | Sans objet | Non | Non ⚠ | Non |
| **Changements structurels différés** | `Commands`, avec points de synchronisation **insérés automatiquement** | Mode différé, points de synchronisation déduits des annotations `[in]` / `[out]` | `EntityCommandBuffer`, appliqué à des points de synchronisation | `FMassCommandBuffer`, vidé en fin de phase | Immédiats |
| **Événements** | `Message` (mis en tampon) + `Event` avec observateurs (immédiats) | Observateurs : événements intégrés et personnalisés | Rien d'intégré : entités-événements, tampon singleton, composants activables | *Signals* + *Observers* | `dispatcher` (en file ou immédiat), signaux du registre |
| **Composants activables** | Désactivation au niveau de l'entité (`Disabled`) | Oui (`CanToggle`) | Oui (`IEnableableComponent`) | Non ⚠ | Non |
| **Détection de changements** | `Changed<T>`, `Added<T>` | Requêtes `changed()` | Filtres de changement par *chunk* | Non ⚠ | Via des observateurs |
| **Relations entre entités** | Oui (depuis 0.16 : `ChildOf`, relations personnalisées) | Oui, les plus riches (paires, `ChildOf`, `IsA`, préfabriqués) | Hiérarchie parent-enfant | Limité ⚠ | Non |
| **Débogage à distance** | **BRP** : JSON-RPC 2.0 sur HTTP, abonnements `+watch` | **REST** : HTTP + JSON (port 27750), outil Explorer | Fenêtres de l'éditeur (Systems, Hierarchy, Archetypes, Journaling) | Mass Debugger, Rewind Debugger | Aucun |
| **Pas à pas système par système** | Oui (`Stepping`, depuis 0.13 ⚠) | Pas de pas à pas natif connu ⚠ | Non | Non | Non |

---

## Q1. Événements et messages

### Ce que font les moteurs

On observe trois mécanismes, souvent présents ensemble dans un même moteur :

1. **Les messages mis en tampon.** Un système écrit des messages, d'autres les lisent plus tard
   dans la boucle. C'est le `Message` de Bevy (depuis 0.17, avec `MessageWriter` /
   `MessageReader`), la file du `dispatcher` d'EnTT, et le *pattern* le plus courant chez Unity
   (un `DynamicBuffer` sur une entité singleton, vidé à chaque frame).
2. **Les observateurs réactifs.** Une réaction se déclenche *immédiatement* quand un événement
   survient : ajout ou retrait de composant, destruction, événement personnalisé. On les trouve
   dans Bevy (`Event` + `On<E>`, propagation le long des relations avec `EntityEvent`), Flecs
   (`OnAdd`, `OnRemove`, `OnSet` et événements personnalisés), Unreal Mass (*Observers* sur ajout
   ou retrait de fragment) et EnTT (`on_construct`, `on_update`, `on_destroy`).
3. **Les composants utilisés comme événements.** Un marqueur `Hit` ajouté puis retiré, ou une
   entité-événement créée puis détruite. Notre modèle sait déjà l'exprimer, mais **à l'échelle de
   millions d'entités, c'est coûteux** : chaque ajout ou retrait déplace l'entité d'archétype.
   Unity le dit explicitement et recommande alors les composants activables.

### Ce qu'on en retient

- Les **messages mis en tampon** entrent parfaitement dans notre modèle « tout dans une boucle ».
  Écrire un message est un accès, lire un message est un accès, et on applique les mêmes règles
  de conflit que pour une ressource. L'ordonnanceur et le simulateur les gèrent sans concept
  nouveau.
- Les **observateurs** contredisent le principe « tout se passe dans la boucle » : ils s'exécutent
  en dehors du plan, au moment où l'événement survient. Bevy et Unreal Mass montrent d'ailleurs que
  leur moment d'exécution est délicat (à la vidange des commandes, et pas avant la première phase
  pour une entité qui vient d'être créée).

### Recommandation

- **Ajouter les messages comme concept à part entière**, sous le nom de *canaux de messages* :
  un type déclaré dans `messages/`, avec des systèmes qui déclarent `emit` ou `read`.
  - Un message émis pendant une étape est visible par les lecteurs des étapes suivantes de la même
    boucle, puis il est effacé à la fin de la boucle.
  - Les conflits suivent la règle des ressources (`emit` = écriture, `read` = lecture).
  - Dans la vue orbitale, les messages peuvent apparaître comme des flux entre planètes.
- **Reporter les observateurs.** S'ils deviennent nécessaires, on les modélisera comme des
  *réactions* attachées à un changement structurel et exécutées au point de synchronisation de fin
  d'étape. Ils resteraient ainsi dans la boucle, et donc visibles et déboguables.

---

## Q2. Parallélisation par ensembles disjoints

### Ce que font les moteurs

- **Bevy est le seul à le faire.** Son ordonnanceur décide du parallélisme à partir de la seule
  signature des systèmes, sans regarder les entités présentes à l'exécution. Ajouter un filtre
  `Without<Player>` d'un côté et `With<Player>` de l'autre prouve que les deux requêtes sont
  disjointes, et autorise leur exécution en parallèle (voir l'erreur B0001 et le guide de
  migration).
- **Unity** suit les dépendances **par type de composant**, sans tenir compte des filtres. Deux
  jobs qui écrivent le même type sont sérialisés.
- **EnTT** (`organizer`) et **Unreal Mass** raisonnent aussi par type de donnée.
- **Flecs** ne parallélise pas les systèmes entre eux ⚠ : il parallélise *l'intérieur* de chaque
  système, en répartissant ses entités entre les threads.

### Un point plus important que la question posée

Avec des millions d'entités, le gain principal ne vient pas de l'exécution de deux systèmes en
parallèle. Il vient du **parallélisme de données à l'intérieur d'un système**, c'est-à-dire de la
répartition de ses entités entre les cœurs. Les cinq moteurs le proposent (`par_iter`,
`multi_threaded`, `ScheduleParallel`…). La condition est qu'un système ne traite chaque entité
qu'**à partir de ses propres composants**. Un système de collision (*broadphase*), qui compare les
entités entre elles, ne remplit pas cette condition.

C'est une information **déclarative**, exactement du type que notre modèle doit porter : l'IA en a
besoin pour choisir la bonne implémentation.

### Recommandation

- **Calculer la disjonction dans le cœur**, puisque c'est un fait du modèle : deux systèmes dont
  l'un exige `with: X` et l'autre `without: X` sont disjoints. Mais **ne l'utiliser pour le plan que
  si le moteur cible le permet.**
- Introduire pour cela un **profil de moteur**, fourni avec les skills, qui déclare les capacités du
  moteur : parallélisme entre systèmes (oui / non), ensembles disjoints (oui / non), composants
  activables, relations… Le plan s'adapte au profil. Pour Flecs, par exemple, on n'aurait qu'une
  seule orbite.
- **Ajouter à chaque système un attribut `per_entity`** (vrai ou faux) : le système traite-t-il
  chaque entité indépendamment des autres ? S'il est vrai, l'IA sait que le système peut être
  parallélisé par entité.

---

## Q3. Protocole de débogage réel

### Ce que font les moteurs

| | Transport | Encodage | Ce qu'on peut faire |
|---|---|---|---|
| **Bevy Remote Protocol** | HTTP (port 15702 par défaut ⚠) | **JSON-RPC 2.0** | Interroger et modifier le monde (`world.query`, `world.get_components`…), s'abonner aux changements (`+watch`), découvrir les méthodes disponibles (`rpc.discover`), ajouter ses propres méthodes |
| **Flecs REST** | HTTP, port 27750 | JSON | Interroger les entités et les requêtes, statistiques par système et par pipeline, outil web Explorer |
| **Unity / Unreal** | Protocoles internes de l'éditeur et du profileur | Propriétaires | Fenêtres intégrées à l'éditeur |
| **Tracy** (profileur utilisé par Bevy et beaucoup de moteurs C++) | TCP | Binaire | Chronologie fine des temps d'exécution |

À noter : la crate `bevy_brp_mcp` expose déjà le protocole de Bevy sous forme de **serveur MCP**,
pour qu'une IA pilote un jeu en cours d'exécution. Ça confirme la pertinence de notre approche.

### Recommandation

- **JSON-RPC 2.0 sur WebSocket en local.**
  - JSON-RPC parce que c'est le choix de Bevy et que c'est **la même base que MCP** : un seul style
    de protocole dans tout ECS NIDE. Il est simple à implémenter dans n'importe quel langage.
  - WebSocket, plutôt qu'HTTP seul, parce qu'il faut un flux bidirectionnel : une frame poussée
    par le jeu à chaque boucle, et des commandes envoyées par l'IDE.
- **Le volume par frame est déjà borné par l'exigence DBG-010** : effectifs des flottes et temps par
  système, soit quelques kilo-octets. Du JSON à 60 frames par seconde suffit. Un encodage
  **MessagePack** pourra être négocié plus tard, si la mesure le justifie.
- **Les sondes s'appuient sur l'existant.** La sonde Bevy peut reposer sur `bevy_remote` et sur
  `Stepping` pour le pas à pas ⚠. La sonde Flecs peut reposer sur son module REST.
- Publier le protocole sous forme de **schéma OpenRPC**, que Bevy utilise déjà pour décrire ses
  méthodes.

---

## Q4. Format des rapports de tests

### Ce qui existe

- **JUnit XML** : le format de fait, lu nativement par GitLab, Jenkins et GitHub Actions, et
  produit par cargo-nextest, GoogleTest, Catch2, et par Unity sous forme de NUnit XML convertible.
  Mais **il n'a pas de spécification officielle**, ses variantes divergent, et il ne sait pas porter
  de données riches comme un contre-exemple.
- **CTRF** (*Common Test Report Format*) : un format JSON avec un **schéma JSON normatif**, des
  générateurs de rapports pour pytest, Playwright, Jest, .NET…, et un convertisseur depuis JUnit.
  Il prévoit des champs d'extension, ce qui permet d'y loger nos identifiants et nos contre-exemples.

### Recommandation

- **Adopter CTRF comme format neutre** d'ECS NIDE, en utilisant ses champs d'extension pour le
  système, l'identifiant d'exemple ou de propriété, le contre-exemple et la graine.
- **Accepter aussi JUnit XML** à l'import, pour les moteurs sans générateur CTRF. Il faut alors une
  **convention de nommage** des tests (par exemple `ecsnide::Damage::example::basique` et
  `ecsnide::Damage::property::sante-positive`) pour retrouver le système et le test. Les skills
  imposent cette convention.

---

## Q5. Rendu de la vue orbitale

### Contraintes de Tauri

Tauri utilise le moteur web du système d'exploitation : WebView2 (Chromium) sous Windows,
WKWebView sous macOS et WebKitGTK sous Linux.

- **WebGPU n'est pas disponible partout.** Sous macOS, il faut macOS 26 ou plus. Sous Linux, son
  support dans WebKitGTK n'est pas confirmé (la question est encore ouverte dans la communauté), et
  la documentation de Tauri signale des problèmes graphiques avec certains pilotes, NVIDIA en
  particulier. Sous Windows, WebView2 suit Chromium, mais c'est à vérifier ⚠.
- **WebGL 2 est disponible sur les trois plateformes.**

### Recommandation

- Utiliser **PixiJS v8**. Il choisit automatiquement WebGPU quand il est disponible et se replie sur
  WebGL 2 sinon. Il regroupe les dessins de milliers d'objets 2D en peu d'appels au GPU, ce qui
  laisse de la marge pour l'exigence VIS-007 (60 images par seconde avec 200 systèmes et
  1 000 flottes).
- Le Canvas 2D seul risque de manquer de marge. Un rendu natif en Rust (wgpu) dans Tauri serait bien
  plus complexe, sans bénéfice visible pour une vue 2D.
- Valider ce choix par un **prototype mesuré** dès la phase 5, sur les trois systèmes
  d'exploitation.

---

## Q6. Langue de l'interface

Cette question ne dépend pas des moteurs ECS. Une remarque tout de même : l'écosystème (moteurs,
documentation, skills, IA) est très majoritairement anglophone.

### Recommandation

- **Internationaliser dès le départ, avec le français comme langue de référence.** Avec Svelte, une
  bibliothèque compilée et typée comme Paraglide JS coûte très peu au début et évite une reprise
  pénible plus tard.
- Pour les **fichiers générés dans les projets** (`AGENTS.md`, skills) : proposer une langue par
  projet, en anglais par défaut, puisque ces fichiers s'adressent surtout aux IA et aux équipes
  internationales.

---

## Autres enseignements : concepts absents de notre modèle

Ce que les moteurs font et que notre modèle ne sait pas encore exprimer :

| Concept | Où on le trouve | Pourquoi c'est important pour nous | Proposition |
|---|---|---|---|
| **Composants activables** | Unity, Flecs, Bevy (au niveau de l'entité) | Activer ou désactiver un composant **sans changement structurel**, donc sans déplacer l'entité d'archétype. C'est indispensable à l'échelle de millions d'entités (`Hit`, `Stunned`…). | Ajouter à chaque composant un trait `toggleable` et deux effets, `enable` et `disable`, qui comptent comme des écritures et non comme des changements structurels |
| **Détection de changements** | Bevy, Unity (par *chunk*), Flecs | Ne traiter que ce qui a changé. C'est un gain majeur à grande échelle. | Ajouter un filtre d'accès `changed` |
| **Phases** | Les cinq moteurs | Regrouper les systèmes en grandes sections (entrées, logique, physique, rendu) pour la lisibilité et pour un ordre grossier | Des phases facultatives, dessinées comme des **secteurs de l'orbite**, en restant dans la même boucle |
| **Fréquence et pas fixe** | Bevy (`FixedUpdate`), Unity (groupe à pas fixe), Flecs (intervalle, fréquence, sources de tick) | La physique tourne souvent à fréquence fixe | Un attribut de fréquence par système (toutes les N boucles), qui reste dans la boucle unique |
| **Relations entre entités** | Flecs, Bevy, Unity (hiérarchie) | Hiérarchies, inventaires, liens entre entités | Le type de champ `entity` suffit au départ ; les relations sont à reconsidérer plus tard |
| **Type de stockage** | Bevy (table ou ensemble clairsemé), EnTT, Flecs | Un composant ajouté et retiré souvent se stocke mieux de façon clairsemée | Un indice facultatif `storage: dense | sparse` sur chaque composant, transmis aux skills |
| **Ressources stockées comme des entités** | Bevy 0.19, Flecs (singletons), Unity (composants singletons) | Les moteurs convergent vers ce modèle | Garder le concept de ressource pour la clarté ; les skills le traduisent |

---

## Décisions à prendre

| # | Question | Recommandation |
|---|---|---|
| 1 | Événements | Ajouter des **canaux de messages** (`emit` / `read`, effacés à chaque boucle) ; reporter les observateurs |
| 2 | Ensembles disjoints | Les calculer dans le cœur, les utiliser selon un **profil de moteur** ; ajouter l'attribut **`per_entity`** |
| 3 | Protocole de débogage | **JSON-RPC 2.0 sur WebSocket**, décrit en OpenRPC ; MessagePack plus tard si nécessaire |
| 4 | Rapports de tests | **CTRF** comme format neutre, JUnit XML accepté avec une convention de nommage |
| 5 | Rendu | **PixiJS v8** (WebGPU si disponible, sinon WebGL 2), validé par un prototype mesuré |
| 6 | Langue | **Interface internationalisée dès le départ**, français en langue de référence ; langue des fichiers générés choisie par projet |
| 7 | Nouveaux concepts | Composants **activables**, filtre **`changed`**, **phases**, **fréquence**, indice de **stockage** : lesquels retenir ? |

## Sources

- Bevy : [notes de version 0.17](https://bevy.org/news/bevy-0-17/), [guide de migration 0.16 → 0.17](https://bevy.org/learn/migration-guides/0-16-to-0-17/), [séparation Message / Event](https://hackmd.io/@alice-i-cecile/SyuIWJPLxg), [refonte des observateurs](https://cocalc.com/github/bevyengine/bevy/blob/main/release-content/release-notes/observer_overhaul.md), [erreur B0001 (requêtes disjointes)](https://bevy.org/learn/errors/b0001/), [changements de la stratégie de parallélisme](https://cocalc.com/github/bevyengine/bevy/blob/main/release-content/migration-guides/parallelism_strategy_changes.md), [Bevy 0.13 (points de synchronisation automatiques)](https://bevy.org/news/bevy-0-13/), [PR #16782](https://github.com/bevyengine/bevy/pull/16782), [notes de version 0.18](https://bevy.org/news/bevy-0-18/), [guide de migration 0.18 → 0.19](https://bevy.org/learn/migration-guides/0-18-to-0-19/), [Bevy 0.19](https://bevy.org/news/bevy-0-19/), [bevy_remote](https://docs.rs/bevy_remote), [bevy_brp_mcp](https://docs.rs/bevy_brp_mcp)
- Flecs : [Explorer](https://github.com/flecs-hub/explorer), [wiki de l'Explorer](https://github.com/flecs-hub/explorer/wiki), [API distante (exemples Rust)](https://docs.rs/crate/flecs_ecs/0.2.1/source/tests/docs/flecsremoteapi.rs), [statistiques](https://docs.rs/crate/flecs_ecs/0.2.2/source/src/addons/stats/mod.rs), [exemple de point de synchronisation](https://rust-digger.code-maven.com/src/flecs_ecs-0.1.3/examples/flecs/systems/system_sync_point.rs), [discussion sur `multi_threaded`](https://github.junw03049948.workers.dev/SanderMertens/flecs/discussions/1136), [manuel (miroir)](https://github.com/NrdyBhu1/flecs/blob/master/docs/Manual.md), [relations (miroir jecs)](https://ukendio.github.io/jecs/learn/concepts/relationships.html)
- Unity Entities : [composants activables et changements structurels](https://docs.unity3d.com/Packages/com.unity.entities@1.3/manual/structural-changes-enableable-components.html), [optimiser les changements structurels](https://docs.unity3d.com/Packages/com.unity.entities@1.4/manual/optimize-structural-changes.html), [dépendances des jobs](https://docs.unity3d.com/Packages/com.unity.entities@1.4/manual/scheduling-jobs-dependencies.html), [points de synchronisation](https://docs.unity3d.com/Packages/com.unity.entities@6.4/manual/performance-sync-points.html), [discussion sur les événements](https://discussions.unity.com/t/designing-an-event-system-for-ecs/824936), [Entities Events](https://openupm.com/packages/com.annulusgames.entities-events/)
- Unreal Mass : [FMassProcessorExecutionOrder](https://dev.epicgames.com/documentation/unreal-engine/API/Runtime/MassEntity/FMassProcessorExecutionOrder), [FMassProcessorDependencySolver](https://dev.epicgames.com/documentation/unreal-engine/API/Runtime/MassEntity/FMassProcessorDependencySolver?lang=en-US), [UMassObserverProcessor](https://dev.epicgames.com/documentation/unreal-engine/API/Runtime/MassEntity/UMassObserverProcessor?lang=en-US), [UMassSignalProcessorBase](https://dev.epicgames.com/documentation/unreal-engine/API/Runtime/MassSignals/UMassSignalProcessorBase), [signaux et observateurs (forum)](https://forums.unrealengine.com/t/mass-signal-vs-observer-processor-vs-onworldbeginplay/2702448), [parallélisme (forum)](https://forums.unrealengine.com/t/mass-entity-processors-not-processing-in-parallel/1297162)
- EnTT : [basic_organizer](https://skypjack.github.io/entt/classentt_1_1basic__organizer.html), [Crash Course ECS](https://github.com/skypjack/entt/wiki/Crash-Course:-entity-component-system)
- Rapports de tests : [CTRF](https://github.com/ctrf-io), [JUnit XML (testmo)](https://github.com/testmoapp/junitxml), [formats de rapports de tests](https://github.com/ligurio/sqa-wiki/wiki/test-report-formats)
- Tauri et WebGPU : [issue Tauri #6381](https://github.com/tauri-apps/tauri/issues/6381), [WebGPU dans WebKitGTK (spike)](https://github.com/777555777/pano-retouch/issues/5), [versions des webviews](https://v2.tauri.app/reference/webview-versions/), [problèmes graphiques sous Linux](https://v2.tauri.app/develop/debug/linux-graphics/), [WebGPU (Wikipedia)](https://en.wikipedia.org/wiki/WebGPU)
