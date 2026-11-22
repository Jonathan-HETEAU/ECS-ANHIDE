# VAL — La validation du modèle

Définition : les **archétypes atteignables** sont les archétypes déclarés, plus toutes les
combinaisons de composants obtenues en leur appliquant, de proche en proche, les ajouts et retraits
(certains ou possibles) des systèmes qui les sélectionnent.

**VAL-001** — ECS NIDE doit signaler comme erreur tout nom en double et toute référence à un
composant, une ressource, un archétype, un système, un exemple ou une propriété inexistant.

**VAL-002** — ECS NIDE doit signaler un avertissement pour tout composant déclaré en `read`,
`write` ou `with` par un système, qui n'appartient à aucun archétype et n'est ajouté par aucun
système.

**VAL-003** — ECS NIDE doit signaler un avertissement pour tout système qui ne sélectionne aucun
archétype atteignable.

**VAL-004** — ECS NIDE doit signaler un avertissement pour tout composant ou ressource déclaré mais
utilisé par aucun système ni aucun archétype.

**VAL-005** — ECS NIDE doit signaler un avertissement pour tout système dont la spécification est
vide, et pour tout système qui n'a aucun exemple ni aucune propriété.

**VAL-006** — ECS NIDE doit associer à chaque diagnostic un niveau (erreur ou avertissement), un
message, l'élément du modèle concerné, ainsi que le fichier et la ligne en cause.

**VAL-007** — Pendant qu'un projet est ouvert dans l'IDE, quand le modèle change, l'IDE doit mettre
à jour les diagnostics en moins d'une seconde.
