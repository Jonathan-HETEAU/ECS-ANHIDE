# features/ — Spécifications fluides (format *Spec Markdown*)

Ce dossier décrit le comportement d'ECS NIDE par des **scénarios lisibles par tous et
exécutables comme des tests**. Le format s'inspire de [Gauge](https://gauge.org) : c'est du
Markdown ordinaire, plus fluide que Gherkin, sans `Given/When/Then` imposé.

Le même format sert aux `spec.md` des systèmes dans les projets ECS NIDE.

## Règles du format

| Élément | Syntaxe | Rôle |
|---|---|---|
| Spécification | `# Titre` (un seul par fichier) | La fonctionnalité décrite. |
| Contexte | paragraphe libre | Explication pour le lecteur, non exécutée. |
| Scénario | `## Phrase qui énonce la règle` | Un cas de comportement. Le titre est la règle elle-même. |
| Traçabilité | `Exigences : ORD-002, ORD-003` | Les exigences EARS couvertes par le scénario. |
| Étape | `* Phrase naturelle` | Une action ou une vérification exécutable. |
| Paramètre | `"valeur"` entre guillemets | Une valeur transmise à l'étape. |
| Données | tableau Markdown placé juste sous une étape | Données d'exemple transmises à l'étape. |

Dans les `spec.md` des systèmes, la ligne de traçabilité devient `Exemples : …` et/ou
`Propriétés : …`, qui renvoient aux identifiants de `examples.yaml` et `properties.yaml`.

## Exécution

Chaque phrase d'étape est reliée à une fonction de test : en Rust pour le cœur, la CLI et le
serveur MCP, en TypeScript pour l'interface. Une étape sans fonction associée fait **échouer**
le scénario. Une spécification n'est donc jamais « décorative ».

## Exemple

```markdown
# Ordonnancement

## Deux systèmes qui écrivent le même composant ne sont jamais dans la même étape
Exigences : ORD-002
* Le système "Movement" écrit "Position"
* Le système "Knockback" écrit "Position"
* Calculer le plan d'exécution
* "Movement" et "Knockback" sont dans des étapes différentes
```

## Le projet d'exemple `space-shooter`

Beaucoup de scénarios s'appuient sur un même projet de référence, à créer dans
`examples/space-shooter/` au début de la phase 1. Sa composition est figée ici : changer ce projet
revient à changer les scénarios.

**Composants** : `Position` (vec2), `Velocity` (vec2), `Rotation` (f32), `Sprite`, `Collider` (radius),
`Health` (current, max), `Hit` (amount), `Lifetime` (remaining), `Target` (entity), et les marqueurs
`Player`, `Enemy`, `Dead`, `Invulnerable`.

**Ressources** : `Time` (delta), `InputState`, `GameState` (score).

**Archétypes** :

| archétype | composants |
|---|---|
| Player   | Position, Velocity, Sprite, Collider, Health, Player |
| Asteroid | Position, Velocity, Sprite, Collider, Health, Enemy |
| Bullet   | Position, Velocity, Sprite, Collider, Lifetime |
| Particle | Position, Velocity, Lifetime |

**Systèmes**, dans l'ordre de référence :

| système | accès | changements structurels |
|---|---|---|
| PlayerInput | read `InputState`ᴿ, write Velocity, with Player | |
| EnemyAI | read Position, write Velocity, optional Target, with Enemy, without Dead | |
| Weapon | read Position, read `InputState`ᴿ, with Player | peut créer Bullet |
| Movement | read Velocity, read `Time`ᴿ, write Position | |
| Lifetime | write Lifetime, read `Time`ᴿ | peut détruire |
| Collision | read Position, read Collider | peut ajouter Hit |
| Damage | write Health, read Hit, without Invulnerable — après Collision | retire Hit ; peut ajouter Dead |
| Death | with Dead, write `GameState`ᴿ | détruit |
| Render | read Position, read Sprite | |

ᴿ = ressource.

## Index

| Fichier | Domaine | Préfixe des exigences |
|---|---|---|
| [01-dossier-projet.spec.md](01-dossier-projet.spec.md) | Le dossier projet, source de vérité | PRJ |
| [02-modelisation.spec.md](02-modelisation.spec.md) | Composants, ressources, systèmes, archétypes | MOD |
| [03-ordonnancement.spec.md](03-ordonnancement.spec.md) | Plan d'exécution et parallélisation | ORD |
| [04-contrats-et-tests.spec.md](04-contrats-et-tests.spec.md) | Spécifications, exemples, propriétés, contrats | TST |
| [05-simulation.spec.md](05-simulation.spec.md) | Simulation du modèle sans code | SIM |
| [06-debogage-reel.spec.md](06-debogage-reel.spec.md) | Connexion au jeu réel | DBG |
| [07-mcp-et-cli.spec.md](07-mcp-et-cli.spec.md) | Serveur MCP et ligne de commande | MCP |
| [08-skills-moteurs.spec.md](08-skills-moteurs.spec.md) | Skills propres à chaque moteur | SKL |
| [09-validation.spec.md](09-validation.spec.md) | Diagnostics du modèle | VAL |
| [10-visualisation.spec.md](10-visualisation.spec.md) | Trou noir, orbites, planètes, flottes | VIS |
