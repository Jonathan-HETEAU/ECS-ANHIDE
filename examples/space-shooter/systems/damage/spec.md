# Damage

Les dégâts reçus pendant la frame sont retirés des points de vie, puis effacés.

## Les dégâts réduisent les points de vie
Exemples : basique
* Une entité à "100" points de vie reçoit un coup de "30"
* Elle termine la frame à "70" points de vie, sans coup en attente

## Les points de vie ne descendent jamais sous zéro
Exemples : plancher-zero
Propriétés : sante-positive
* Une entité à "10" points de vie reçoit un coup de "50"
* Elle termine à "0" point de vie et elle est marquée morte
