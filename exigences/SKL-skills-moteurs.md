# SKL — Les skills propres à chaque moteur

**SKL-001** — ECS NIDE doit permettre de choisir le moteur ou la bibliothèque ECS cible d'un
projet, et l'enregistrer dans `ecs-nide.yaml`.

**SKL-002** — Quand l'utilisateur choisit un moteur cible, ECS NIDE doit installer dans `skills/`
les skills de ce moteur, au format Agent Skills (`SKILL.md`).

**SKL-003** — Un skill de moteur doit expliquer comment traduire vers ce moteur chaque concept du
modèle : composant, marqueur, ressource, accès `read`, `write`, `optional`, `with` et `without`,
changements structurels, archétype, étapes et contraintes d'ordre.

**SKL-004** — Un skill de moteur doit expliquer comment traduire les exemples et les propriétés en
tests exécutables dans ce moteur, et comment produire le rapport de résultats au format neutre.

**SKL-005** — Là où une sonde de débogage existe pour le moteur, le skill de ce moteur doit
expliquer comment l'intégrer au jeu.

**SKL-006** — Quand l'utilisateur change de moteur cible, ECS NIDE doit remplacer les skills de
l'ancien moteur par ceux du nouveau.

**SKL-007** — Si un skill installé a été modifié par l'utilisateur, alors ECS NIDE doit demander
confirmation avant de le remplacer ou de le supprimer.
