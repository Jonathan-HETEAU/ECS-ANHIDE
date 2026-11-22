# SKL — Les skills propres à chaque moteur

**SKL-001** — ECS NIDE doit permettre de choisir le moteur ou la bibliothèque ECS cible d'un
projet, et l'enregistrer dans `ecs-nide.yaml`.

**SKL-002** — Quand l'utilisateur choisit un moteur cible, ECS NIDE doit installer dans `skills/`
les skills de ce moteur, au format Agent Skills (`SKILL.md`).

**SKL-003** — Un skill de moteur doit expliquer comment traduire vers ce moteur chaque concept du
modèle (composant, marqueur, composant activable, indication de stockage, ressource, canal de
messages, événement, observateur, relation, accès `read`, `write`, `optional`, `with`, `without`
et `via`, filtre de changement, changements structurels, activations et désactivations,
archétype, phases, étapes, contraintes d'ordre, fréquence et attribut `per_entity`) et, pour
chaque concept absent du moteur, l'équivalent à utiliser.

**SKL-004** — Un skill de moteur doit expliquer comment traduire les exemples et les propriétés en
tests exécutables dans ce moteur, et comment produire le rapport de résultats au format CTRF avec
les champs d'extension d'ECS NIDE, ou à défaut au format JUnit XML selon la convention de nommage
de TST-014.

**SKL-005** — Là où une sonde de débogage existe pour le moteur, le skill de ce moteur doit
expliquer comment l'intégrer au jeu.

**SKL-006** — Quand l'utilisateur change de moteur cible, ECS NIDE doit remplacer les skills de
l'ancien moteur par ceux du nouveau.

**SKL-007** — Si un skill installé a été modifié par l'utilisateur, alors ECS NIDE doit demander
confirmation avant de le remplacer ou de le supprimer.

**SKL-008** — Les skills d'un moteur doivent fournir un profil de moteur (`profile.yaml`) qui
déclare si le moteur prend en charge : le parallélisme entre systèmes, les ensembles disjoints, les
composants activables, les filtres de changement, la fréquence des systèmes, les observateurs et
les relations.

**SKL-009** — Tant qu'aucun moteur cible n'est choisi, ECS NIDE doit utiliser le profil générique :
parallélisme entre systèmes pris en charge, ensembles disjoints non pris en charge, tous les autres
concepts pris en charge.

**SKL-010** — ECS NIDE doit fournir, pour Bevy, premier moteur cible, ses skills, son profil et sa
sonde de débogage.
