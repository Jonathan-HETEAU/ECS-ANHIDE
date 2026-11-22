# exigences/ — Le fonctionnement attendu, phrase par phrase (format EARS)

Chaque exigence est **une seule phrase**, construite sur un modèle **EARS** (*Easy Approach to
Requirements Syntax*). Elle porte un identifiant stable (`ORD-002`) que reprennent les scénarios
de [`features/`](../features/) et les tests.

## Les modèles EARS

| Modèle | Forme | Exemple |
|---|---|---|
| **Permanent** | `<acteur> doit <réponse>.` | ECS NIDE doit produire le même plan pour le même projet. |
| **Événement** | `Quand <déclencheur>, <acteur> doit <réponse>.` | Quand l'utilisateur crée un projet, l'IDE doit… |
| **État** | `Pendant que <état>, <acteur> doit <réponse>.` | Pendant que la simulation est en pause, l'IDE doit… |
| **Option** | `Là où <fonctionnalité présente>, <acteur> doit <réponse>.` | Là où une sonde existe pour le moteur, le skill doit… |
| **Indésirable** | `Si <situation indésirable>, alors <acteur> doit <réponse>.` | Si la connexion est perdue, alors l'IDE doit… |
| **Complexe** | une combinaison des modèles ci-dessus | Pendant que…, quand…, l'IDE doit… |

## Les acteurs

| Acteur | Ce qu'il désigne |
|---|---|
| **ECS NIDE** | L'ensemble : le cœur Rust, utilisé à l'identique par l'IDE, la CLI et le serveur MCP. |
| **l'IDE** | L'application de bureau Tauri + Svelte. |
| **le serveur MCP** | `ecs-nide mcp`. |
| **la CLI** | Les autres commandes du binaire `ecs-nide`. |
| **un skill** | Un skill de moteur installé dans un projet. |

## Règles

- `doit` exprime une obligation. Une exigence facultative n'a rien à faire ici : elle va dans les
  questions ouvertes de [`VISION.md`](../VISION.md).
- Une exigence doit être **vérifiable** : chacune est couverte par au moins un scénario de
  `features/` et par au moins un test.
- Un identifiant n'est **jamais réutilisé**. Une exigence abandonnée est barrée et marquée
  *(abandonnée)*.

## Index

| Fichier | Préfixe |
|---|---|
| [PRJ-dossier-projet.md](PRJ-dossier-projet.md) | PRJ |
| [MOD-modelisation.md](MOD-modelisation.md) | MOD |
| [ORD-ordonnancement.md](ORD-ordonnancement.md) | ORD |
| [TST-contrats-et-tests.md](TST-contrats-et-tests.md) | TST |
| [SIM-simulation.md](SIM-simulation.md) | SIM |
| [DBG-debogage-reel.md](DBG-debogage-reel.md) | DBG |
| [MCP-mcp-et-cli.md](MCP-mcp-et-cli.md) | MCP |
| [SKL-skills-moteurs.md](SKL-skills-moteurs.md) | SKL |
| [VAL-validation.md](VAL-validation.md) | VAL |
| [VIS-visualisation.md](VIS-visualisation.md) | VIS |
