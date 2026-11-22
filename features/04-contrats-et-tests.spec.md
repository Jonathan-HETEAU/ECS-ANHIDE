# Contrats, spécifications et tests

Chaque système est décrit par une spécification fluide (le prompt), des tests par exemples et
des tests par propriétés. Réunis avec les schémas utiles et le skill du moteur, ils forment le
**contrat** qu'une IA reçoit pour implémenter le système.

## Le contrat d'un système contient tout ce qu'une IA doit savoir
Exigences : TST-001, TST-013
* Ouvrir le projet d'exemple "space-shooter"
* Obtenir le contrat du système "Damage"
* Le contrat contient la spécification, les accès, les changements structurels et les contraintes d'ordre
* Le contrat contient la phase, la fréquence et l'attribut "per_entity" de "Damage"
* Le contrat contient l'indication de stockage de chaque composant utilisé
* Le contrat contient les schémas de "Health", "Hit", "Invulnerable" et "Dead"
* Le contrat contient les exemples et les propriétés de "Damage"
* Le contrat contient le profil du moteur cible et référence son skill
* Le contrat ne contient aucun schéma de composant que "Damage" n'utilise pas

## Un scénario de spécification renvoie à des exemples existants
Exigences : TST-001, VAL-001
* Le scénario "Les dégâts réduisent les points de vie" de "Damage" renvoie à l'exemple "inexistant"
* La validation signale un renvoi vers un exemple inconnu dans "systems/damage/spec.md"

## Un exemple décrit l'état avant et après le passage du système
Exigences : TST-002
* L'exemple "basique" de "Damage" donne en entrée "Health.current" à "100" et "Hit.amount" à "30"
* Il attend en sortie "Health.current" à "70" et l'absence de "Hit"
* L'exemple est cohérent avec le contrat de "Damage"

## Un exemple dont l'entrée n'est pas sélectionnée par le système est incohérent
Exigences : TST-004
* Un exemple de "Damage" donne en entrée seulement "Health"
* La validation signale que "Damage" ne sélectionne pas l'entrée de cet exemple, faute de "Hit"

## Un exemple qui modifie un composant non déclaré est incohérent
Exigences : TST-005
* Un exemple de "Damage" attend en sortie une modification de "Position"
* La validation signale que "Damage" n'écrit, n'ajoute ni ne retire "Position"

## Une propriété porte sur l'état avant et après le système
Exigences : TST-003
* La propriété "sante-ne-remonte-pas" de "Damage" impose "after.Health.current <= before.Health.current"
* La règle est une expression CEL valide
* La règle ne référence que des composants accessibles par "Damage"

## Un exemple qui contredit une propriété est signalé
Exigences : TST-006
* L'exemple "soin" de "Damage" fait passer "Health.current" de "50" à "80"
* La validation signale que l'exemple "soin" contredit la propriété "sante-ne-remonte-pas"

## Les données générées respectent les schémas et les domaines
Exigences : TST-007
* Générer "1000" entrées pour la propriété "sante-positive" de "Damage"
* Chaque entrée est sélectionnée par "Damage"
* Chaque "Health.current" est compris entre "0" et "1000"
* Chaque "Hit.amount" est compris entre "0" et "100000"

## Les valeurs limites sont toujours générées
Exigences : TST-009
* Générer "1000" entrées pour la propriété "sante-positive" de "Damage"
* Les valeurs "0" et "1000" de "Health.current" figurent parmi les entrées
* Les valeurs "0" et "100000" de "Hit.amount" figurent parmi les entrées

## La génération est reproductible avec une graine
Exigences : TST-008
* Générer "500" entrées pour "sante-positive" avec la graine "42"
* Générer à nouveau "500" entrées pour "sante-positive" avec la graine "42"
* Les deux jeux de données sont identiques

## Les données générées s'exportent dans un format neutre
Exigences : TST-010
* Générer "100" entrées pour "sante-positive" avec la graine "42"
* Exporter les données en JSON
* Le fichier JSON est valide au regard du schéma publié des données de test

## Les résultats des tests exécutés à l'extérieur remontent dans l'IDE
Exigences : TST-011
* Importer un rapport CTRF où l'exemple "basique" de "Damage" réussit et la propriété "sante-positive" échoue avec le contre-exemple "Health.current = 0, Hit.amount = 1"
* L'IDE affiche "Damage" avec "1" test réussi et "1" test échoué
* Le contre-exemple est affiché avec la propriété "sante-positive"

## Un contre-exemple devient un exemple en un clic
Exigences : TST-012
* Le contre-exemple de "sante-positive" est affiché
* Le convertir en exemple de test
* "Damage" possède un nouvel exemple dont l'entrée est "Health.current = 0, Hit.amount = 1"
* La sortie attendue du nouvel exemple reste à compléter et la validation le signale

## Une entité capturée en débogage devient un exemple
Exigences : TST-012
* Pendant une session de débogage, capturer l'entité "42" avant et après le passage de "Damage"
* "Damage" possède un nouvel exemple dont l'entrée et la sortie sont l'état capturé

## Un exemple peut décrire l'état des composants activables et les messages
Exigences : TST-002
* Le système "StunTimer" écrit "Stunned", peut désactiver "Stunned" et peut émettre sur le canal "StunEnded"
* L'exemple "fin" de "StunTimer" donne en entrée "Stunned" activé avec "Stunned.remaining = 0"
* Il attend en sortie "Stunned" désactivé et un message "StunEnded"
* L'exemple est cohérent avec le contrat de "StunTimer"

## Un exemple qui émet un message non déclaré est incohérent
Exigences : TST-005
* Le système "Sfx" lit le canal "Explosion" et n'émet sur aucun canal
* Un exemple de "Sfx" attend en sortie un message "Explosion"
* La validation signale que "Sfx" n'émet pas sur "Explosion"

## Un rapport JUnit XML est rattaché grâce à la convention de nommage
Exigences : TST-014
* Importer un rapport JUnit XML où le test "ecsnide::Damage::example::basique" réussit et le test "ecsnide::Damage::property::sante-positive" échoue
* L'IDE affiche l'exemple "basique" de "Damage" comme réussi
* L'IDE affiche la propriété "sante-positive" de "Damage" comme échouée

## Un résultat sans correspondance est signalé sans bloquer l'import
Exigences : TST-015
* Importer un rapport contenant les tests "ecsnide::Mana::example::x" et "ecsnide::Damage::example::basique"
* L'import signale le test sans correspondance "ecsnide::Mana::example::x"
* Le résultat de l'exemple "basique" de "Damage" est importé
