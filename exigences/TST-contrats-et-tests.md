# TST — Contrats, spécifications et tests

**TST-001** — ECS NIDE doit associer à chaque système une spécification au format *Spec Markdown*,
dont chaque scénario peut renvoyer à des exemples et à des propriétés du système.

**TST-002** — ECS NIDE doit associer à chaque système des tests par exemples, chacun décrivant
l'entité avant le passage du système (composants et leurs valeurs, état des composants activables,
ressources lues, messages reçus) et ce qu'on attend après (composants et leurs valeurs, état des
composants activables, ressources écrites, messages émis).

**TST-003** — ECS NIDE doit associer à chaque système des tests par propriétés, chacun composé d'un
domaine de données facultatif et d'une règle CEL portant sur l'état avant (`before`) et après
(`after`) le passage du système.

**TST-004** — Si un exemple fournit en entrée une entité que le système ne sélectionne pas, alors
ECS NIDE doit signaler l'exemple comme incohérent et nommer les composants manquants ou interdits.

**TST-005** — Si un exemple attend en sortie la modification, l'ajout, le retrait, l'activation ou
la désactivation d'un composant, ou l'émission d'un message, que le système ne déclare pas, alors
ECS NIDE doit signaler l'exemple comme incohérent.

**TST-006** — Si un exemple ne respecte pas une propriété du même système, alors ECS NIDE doit
signaler la contradiction en nommant l'exemple et la propriété.

**TST-007** — Quand ECS NIDE génère des données de test pour une propriété, il doit produire des
entrées sélectionnées par le système, dont les valeurs respectent les types, les contraintes des
schémas et le domaine de la propriété.

**TST-008** — Quand une graine est fournie, ECS NIDE doit générer exactement les mêmes données de
test à chaque génération.

**TST-009** — Quand ECS NIDE génère des données de test, il doit inclure les valeurs limites de
chaque champ contraint : bornes, zéro lorsqu'il est autorisé, liste vide lorsqu'elle est autorisée.

**TST-010** — ECS NIDE doit exporter les données de test générées en JSON, conformément à un schéma
publié.

**TST-011** — Quand un rapport de résultats de tests au format CTRF est importé, ECS NIDE doit
associer chaque résultat à son système et à son exemple ou sa propriété grâce aux champs
d'extension qu'il définit et publie (système, type de test, identifiant, contre-exemple, graine),
et l'IDE doit afficher les tests réussis, les tests échoués et les contre-exemples.

**TST-012** — Quand l'utilisateur convertit un contre-exemple ou une entité capturée en débogage,
ECS NIDE doit créer pour le système concerné un nouvel exemple pré-rempli avec l'état connu, et
signaler comme incomplètes les parties restant à compléter.

**TST-013** — ECS NIDE doit produire pour chaque système un contrat autonome qui regroupe : sa
spécification ; ses accès aux composants, ressources et canaux de messages ; ses filtres de
changement ; ses changements structurels ; ses activations et désactivations ; ses contraintes
d'ordre ; sa phase ; sa fréquence ; son attribut `per_entity` ; les schémas et indications de
stockage des seuls composants, ressources et canaux qu'il utilise ; ses exemples ; ses propriétés ;
le profil et la référence au skill du moteur cible.

**TST-014** — Quand un rapport de résultats de tests au format JUnit XML est importé, ECS NIDE doit
associer chaque test à son système et à son exemple ou sa propriété d'après la convention de
nommage `ecsnide::<Système>::example::<id>` ou `ecsnide::<Système>::property::<id>`.

**TST-015** — Si un résultat d'un rapport importé ne correspond à aucun système, exemple ou
propriété du projet, alors ECS NIDE doit le signaler et poursuivre l'import des autres résultats.
