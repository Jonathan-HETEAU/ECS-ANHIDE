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
aucun de ceux qu'il déclare en `without`.

**MOD-010** — ECS NIDE doit permettre de déclarer des archétypes, chacun identifié par un nom
unique, composé d'une liste de composants et doté d'un volume estimé d'entités.

**MOD-011** — Quand l'utilisateur renomme un composant, une ressource, un archétype ou un
système, ECS NIDE doit mettre à jour toutes ses références dans le projet, y compris dans les
exemples, les propriétés et les spécifications.

**MOD-012** — Si l'utilisateur demande la suppression d'un élément référencé ailleurs dans le
projet, alors l'IDE doit lister ces références et demander confirmation avant de supprimer.
