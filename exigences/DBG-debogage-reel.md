# DBG — Le débogage du jeu réel

**DBG-001** — ECS NIDE doit pouvoir se connecter à un jeu en cours d'exécution équipé d'une sonde,
via le protocole de débogage d'ECS NIDE : JSON-RPC 2.0 transporté par WebSocket, décrit par un
document OpenRPC publié avec ECS NIDE.

**DBG-002** — Quand la connexion à un jeu est établie, l'IDE doit comparer le plan d'exécution
réel au plan du projet et signaler chaque divergence : système absent, système inconnu, ordre ou
étape différent.

**DBG-003** — Pendant une session de débogage réel, l'IDE doit afficher les entités regroupées en
flottes par archétype, avec l'effectif de chaque flotte à chaque frame.

**DBG-004** — Pendant une session de débogage réel, l'IDE doit afficher le temps d'exécution de
chaque système pour chaque frame reçue.

**DBG-005** — Pendant une session de débogage réel, quand l'utilisateur demande une pause, un pas
ou une boucle, ECS NIDE doit transmettre la commande à la sonde et afficher l'état qu'elle confirme.

**DBG-006** — Pendant une session de débogage réel, quand l'utilisateur active ou désactive un
système, ou pose ou retire un point d'arrêt, ECS NIDE doit transmettre la commande à la sonde.

**DBG-007** — Pendant que le jeu est en pause, quand l'utilisateur demande un échantillon d'une
flotte, l'IDE doit afficher au plus le nombre d'entités demandé, avec les valeurs de leurs
composants.

**DBG-008** — Pendant que le jeu est en pause, quand l'utilisateur modifie une valeur de composant
d'une entité, ECS NIDE doit transmettre la modification à la sonde et afficher la valeur confirmée.

**DBG-009** — Si la connexion au jeu est perdue, alors l'IDE doit indiquer la déconnexion et
conserver l'affichage de la dernière frame reçue.

**DBG-010** — Pendant une session de débogage réel, ECS NIDE doit échanger par frame un volume de
données indépendant du nombre d'entités du jeu, hors échantillons demandés.

**DBG-011** — ECS NIDE doit fournir une sonde de référence qui simule un jeu conforme au protocole,
afin de tester l'IDE sans moteur réel.

**DBG-012** — Si une sonde envoie un message non conforme au document OpenRPC du protocole, alors
l'IDE doit signaler le message non conforme, l'ignorer et poursuivre la session.
