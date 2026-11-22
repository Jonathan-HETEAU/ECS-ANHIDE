# VIS — La visualisation et l'interface

**VIS-001** — L'IDE doit représenter le projet par un trou noir central entouré d'une orbite, sur
laquelle chaque étape du plan occupe une position angulaire, la première en haut et les suivantes
dans le sens horaire.

**VIS-002** — L'IDE doit représenter chaque système par une planète, les systèmes d'une même étape
étant placés à la même position angulaire, sur des orbites concentriques distinctes.

**VIS-003** — L'IDE doit représenter les entités par des vaisseaux regroupés en flottes par
archétype, chaque flotte portant des signes distinctifs dérivés de ses composants.

**VIS-004** — Pendant une simulation ou une session de débogage réel, l'IDE doit animer les flottes
le long de l'orbite et montrer leur arrêt sur chaque planète dont le système les sélectionne.

**VIS-005** — Quand l'utilisateur sélectionne une planète, l'IDE doit afficher le contrat du système
et ses derniers résultats de tests.

**VIS-006** — Quand l'utilisateur sélectionne une flotte, l'IDE doit lister, dans l'ordre du plan,
les systèmes qui la sélectionnent, et afficher ses entités.

**VIS-007** — L'IDE doit afficher la vue orbitale à au moins 60 images par seconde pour un projet
de 200 systèmes et 1000 archétypes.

**VIS-008** — L'IDE doit proposer une vue matrice où chaque ligne est un système, chaque colonne un
composant, une ressource ou un canal de messages, et chaque case l'accès déclaré, le filtre de
changement, les changements structurels et les activations ou désactivations.

**VIS-009** — L'IDE doit indiquer sur chaque planète les diagnostics du système, la présence d'un
point d'arrêt, son état désactivé et sa fréquence lorsqu'elle est différente de 1.

**VIS-010** — Quand des phases sont déclarées, l'IDE doit découper l'orbite en secteurs nommés,
un par phase, dans l'ordre des phases, chacun couvrant les étapes des systèmes de sa phase.

**VIS-011** — L'IDE doit représenter chaque canal de messages par un flux allant de chaque planète
qui émet sur ce canal vers chaque planète qui le lit.

**VIS-012** — L'IDE doit tirer tous les textes de son interface de catalogues de traduction, le
catalogue français étant la référence et devant être complet.

**VIS-013** — L'IDE doit représenter chaque observateur par une lune en orbite autour de chaque
planète dont le système peut le déclencher, directement ou en cascade.

**VIS-014** — L'IDE doit représenter les relations entre archétypes par des liens entre leurs
flottes, et afficher les entités liées dans le panneau d'une flotte.
