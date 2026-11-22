# MOD — La modélisation

**MOD-001** — ECS NIDE doit permettre de déclarer des composants, chacun identifié par un nom
unique dans le projet et doté de champs typés.

**MOD-002** — ECS NIDE doit proposer des types de champs indépendants du langage : `bool`,
`i8`, `i16`, `i32`, `i64`, `u8`, `u16`, `u32`, `u64`, `f32`, `f64`, `string`, `vec2`, `vec3`,
`vec4`, `quat`, `entity`, `enum` et `list<T>`.

**MOD-003** — ECS NIDE doit permettre d'associer à chaque champ une valeur par défaut et, selon
son type, des contraintes : bornes `min` et `max`, liste de valeurs autorisées, longueur maximale.

**MOD-004** — ECS NIDE doit traiter un composant sans champ comme un marqueur.

**MOD-005** — ECS NIDE doit permettre de déclarer des ressources, données globales uniques
identifiées par un nom et dotées de champs typés comme les composants.

**MOD-006** — ECS NIDE doit permettre à un système de déclarer, pour chaque composant qu'il
utilise, exactement un accès parmi : `read`, `write`, `optional`, `with` et `without`.

**MOD-007** — ECS NIDE doit permettre à un système de déclarer, pour chaque ressource qu'il
utilise, un accès `read` ou `write`.

**MOD-008** — ECS NIDE doit permettre à un système de déclarer ses changements structurels
(ajout de composants, retrait de composants, création d'entités d'un archétype, destruction
d'entités), chacun qualifié de certain ou de possible.

**MOD-009** — ECS NIDE doit considérer qu'un système sélectionne une entité si et seulement si
l'entité possède tous les composants que le système déclare en `read`, `write` ou `with`, et
aucun de ceux qu'il déclare en `without`, un composant activable désactivé comptant comme absent.

**MOD-010** — ECS NIDE doit permettre de déclarer des archétypes, chacun identifié par un nom
unique, composé d'une liste de composants et doté d'un volume estimé d'entités.

**MOD-011** — Quand l'utilisateur renomme un composant, une ressource, un canal de messages, un
archétype, une phase ou un système, ECS NIDE doit mettre à jour toutes ses références dans le
projet, y compris dans les exemples, les propriétés et les spécifications.

**MOD-012** — Si l'utilisateur demande la suppression d'un élément référencé ailleurs dans le
projet, alors l'IDE doit lister ces références et demander confirmation avant de supprimer.

**MOD-013** — ECS NIDE doit permettre de déclarer un composant comme activable (`toggleable`).

**MOD-014** — ECS NIDE doit permettre à un système de déclarer qu'il active (`enable`) ou
désactive (`disable`) des composants activables, chaque effet étant qualifié de certain ou de
possible et n'étant pas un changement structurel.

**MOD-015** — ECS NIDE doit permettre à un système de déclarer un filtre de changement
(`changed`) sur des composants qu'il sélectionne, de sorte qu'il ne traite que les entités dont au
moins un de ces composants a été ajouté ou écrit depuis son précédent passage.

**MOD-016** — ECS NIDE doit permettre d'associer à chaque composant une indication de stockage,
`dense` (par défaut) ou `sparse`, reprise dans les contrats des systèmes qui l'utilisent.

**MOD-017** — ECS NIDE doit permettre de déclarer des canaux de messages, chacun identifié par un
nom unique et doté de champs typés comme les composants.

**MOD-018** — ECS NIDE doit permettre à un système de déclarer, pour chaque canal de messages
qu'il utilise, un accès `emit` (émission certaine), `may_emit` (émission possible) ou `read`.

**MOD-019** — ECS NIDE doit permettre à un système de déclarer qu'il traite chaque entité
indépendamment des autres (`per_entity`), à partir de ses seuls composants, des ressources et des
messages qu'il lit, la valeur par défaut étant faux.

**MOD-020** — ECS NIDE doit permettre de déclarer dans `schedule.yaml` une liste ordonnée de
phases, et de rattacher chaque système à l'une d'elles.

**MOD-021** — ECS NIDE doit permettre de déclarer pour chaque système une fréquence `every`,
entier N supérieur ou égal à 1 signifiant « une boucle sur N », la valeur par défaut étant 1.
