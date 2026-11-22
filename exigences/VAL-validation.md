# VAL — La validation du modèle

Définition : les **archétypes atteignables** sont les archétypes déclarés, plus toutes les
combinaisons de composants obtenues en leur appliquant, de proche en proche, les ajouts et retraits
(certains ou possibles) des systèmes qui les sélectionnent.

**VAL-001** — ECS NIDE doit signaler comme erreur tout nom en double et toute référence à un
composant, une ressource, un canal de messages, un événement, une relation, un archétype, une
phase, un système, un observateur, un exemple ou une propriété inexistant.

**VAL-002** — ECS NIDE doit signaler un avertissement pour tout composant déclaré en `read`,
`write` ou `with` par un système, qui n'appartient à aucun archétype et n'est ajouté par aucun
système.

**VAL-003** — ECS NIDE doit signaler un avertissement pour tout système qui ne sélectionne aucun
archétype atteignable.

**VAL-004** — ECS NIDE doit signaler un avertissement pour tout composant, ressource, canal de
messages, événement ou relation déclaré mais utilisé par aucun système, aucun observateur ni aucun
archétype.

**VAL-005** — ECS NIDE doit signaler un avertissement pour tout système dont la spécification est
vide, et pour tout système qui n'a aucun exemple ni aucune propriété.

**VAL-006** — ECS NIDE doit associer à chaque diagnostic un niveau (erreur ou avertissement), un
message, l'élément du modèle concerné, ainsi que le fichier et la ligne en cause.

**VAL-007** — Pendant qu'un projet est ouvert dans l'IDE, quand le modèle change, l'IDE doit mettre
à jour les diagnostics en moins d'une seconde.

**VAL-008** — Si des phases sont déclarées et qu'un système n'est rattaché à aucune phase, alors
ECS NIDE doit signaler une erreur.

**VAL-009** — Si un système déclare l'activation ou la désactivation d'un composant qui n'est pas
activable, alors ECS NIDE doit signaler une erreur.

**VAL-010** — Si un système déclare un filtre de changement sur un composant qu'il ne déclare pas
en `read`, `write` ou `with`, alors ECS NIDE doit signaler une erreur.

**VAL-011** — Si un système déclare à la fois l'émission et la lecture d'un même canal de messages,
alors ECS NIDE doit signaler une erreur.

**VAL-012** — ECS NIDE doit signaler un avertissement pour tout canal de messages lu par au moins un
système mais sur lequel aucun système n'émet, et pour tout canal sur lequel un système émet mais
qu'aucun système ne lit.

**VAL-013** — Si le projet utilise un concept que le profil du moteur cible ne prend pas en charge,
alors ECS NIDE doit signaler un avertissement qui nomme le concept et renvoie au skill du moteur.

**VAL-014** — ECS NIDE doit signaler un avertissement pour tout observateur que rien ne peut
déclencher : aucun système ni observateur ne déclenche son événement, ou aucun ne peut produire
l'événement de cycle de vie qu'il observe.

**VAL-015** — ECS NIDE doit signaler un avertissement pour tout événement déclenché par un système
ou un observateur mais observé par aucun observateur.

**VAL-016** — Si un système ou un observateur déclare un accès `via` sur une relation dont il ne
sélectionne pas le composant source, alors ECS NIDE doit signaler une erreur.
