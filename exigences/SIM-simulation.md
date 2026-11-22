# SIM — La simulation du modèle

**SIM-001** — ECS NIDE doit permettre de simuler un projet sans aucun code de jeu.

**SIM-002** — Pendant la simulation, quand une étape s'exécute, ECS NIDE doit appliquer chaque
système de l'étape à toutes les entités qu'il sélectionne.

**SIM-003** — Pendant que la simulation est en pause, quand l'utilisateur demande un pas, ECS NIDE
doit exécuter l'étape suivante du plan puis se remettre en pause.

**SIM-004** — Pendant que la simulation est en pause, quand l'utilisateur demande une boucle,
ECS NIDE doit exécuter les étapes restantes de la boucle en cours puis se remettre en pause au début
de la boucle suivante.

**SIM-005** — Quand l'utilisateur demande un retour en arrière, ECS NIDE doit restaurer exactement
l'état du monde qui précédait la dernière étape exécutée.

**SIM-006** — Pendant la lecture, quand la simulation atteint une étape contenant un système qui
porte un point d'arrêt, ECS NIDE doit se mettre en pause avant d'exécuter cette étape.

**SIM-007** — Quand l'utilisateur désactive ou réactive un système pendant la simulation, ECS NIDE
doit en tenir compte à partir de l'étape suivante, sans modifier le projet.

**SIM-008** — Quand un système traite une entité dont les composants accessibles au système
(déclarés en `read`, `write` ou `optional`), leurs valeurs, l'état de ses composants activables,
les ressources lues et les messages reçus correspondent exactement à l'entrée d'un de ses exemples,
ECS NIDE doit appliquer la sortie de cet exemple (valeurs, activations, désactivations, messages
émis), sans toucher aux autres composants de l'entité.

**SIM-009** — Quand un système traite une entité qui ne correspond à aucun de ses exemples,
ECS NIDE doit appliquer ses changements structurels, activations, désactivations et émissions
certains, signaler ceux qui sont possibles sans les appliquer, et marquer comme indéterminés les
champs des composants qu'il déclare en `write`.

**SIM-010** — Pendant que la simulation est en pause, l'IDE doit permettre d'ajouter ou de retirer
des composants sur une entité, d'activer ou de désactiver ses composants activables, et de modifier
ses valeurs.

**SIM-011** — Quand l'utilisateur peuple la simulation à partir d'un archétype, ECS NIDE doit créer
le nombre d'entités demandé, avec exactement les composants de l'archétype et des valeurs qui
respectent les schémas, de façon reproductible pour une même graine.

**SIM-012** — Quand un système crée une entité d'un archétype, ECS NIDE doit l'ajouter à la fin de
l'étape, avec les valeurs par défaut des composants de l'archétype.

**SIM-013** — Pendant la simulation, ECS NIDE doit exécuter un système de fréquence N uniquement
lors des boucles dont le numéro est un multiple de N, la première boucle portant le numéro 0.

**SIM-014** — Pendant la simulation, ECS NIDE doit rendre chaque message émis pendant une étape
visible aux systèmes qui lisent son canal dans les étapes suivantes de la même boucle, puis vider
tous les canaux à la fin de la boucle ; un message émis sans exemple correspondant a des champs
indéterminés.

**SIM-015** — Pendant la simulation, quand un système active ou désactive un composant activable,
ECS NIDE doit appliquer le changement immédiatement, sans changer l'archétype de l'entité.

**SIM-016** — Pendant la simulation, ECS NIDE doit présenter à un système doté d'un filtre de
changement uniquement les entités dont un composant filtré a été ajouté ou écrit depuis le
précédent passage de ce système, toutes les entités étant présentées lors de son premier passage.
