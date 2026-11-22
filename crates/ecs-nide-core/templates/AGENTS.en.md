# AGENTS.md — Working on the {{name}} ECS NIDE project

This folder is an **ECS NIDE** project: a declarative description of a game built on the
Entity-Component-System architecture. It contains no game code. The game is implemented
elsewhere, in the target engine, from the **contracts** described here.

## Folder layout

| Path | Content |
|---|---|
| `ecs-nide.yaml` | Project name, file format version, target engine, language |
| `schedule.yaml` | Phases and reference order of the systems |
| `schedule.plan.yaml` | Execution plan computed by ECS NIDE — never edit it by hand |
| `components/` | One YAML file per component: typed fields and constraints |
| `resources/` | One YAML file per resource (unique global data) |
| `messages/` | One YAML file per message channel |
| `events/` | One YAML file per event |
| `relationships/` | One YAML file per relationship between entities |
| `archetypes/` | One YAML file per archetype (combination of components) |
| `systems/<name>/` | `system.yaml` (contract), `spec.md` (specification), `examples.yaml`, `properties.yaml` |
| `observers/<name>/` | Same files as a system, with `observer.yaml` and its trigger |
| `skills/` | Skills of the target engine: how to translate this model into that engine |

## How to work

1. **Get the contract** of the system you implement: `ecs-nide contract <System>`, or the
   `contract` tool of the ECS NIDE MCP server. It contains everything you need: specification,
   accesses, schemas, examples, properties and target engine.
2. **Follow the skill** of the target engine in `skills/` to translate each concept.
3. **Tests first.** Turn every example and every property into a test in the engine, see it
   fail, then implement the system.
4. **Report the results** as a CTRF report and import it: `ecs-nide import-results <report>`.
5. **Every bug is a missing test.** When a behaviour is wrong, first add the example that
   reproduces it to `examples.yaml`, then fix the code.

## Rules

- A system describes **what** it does, never **how**: no code in the YAML files.
- Run `ecs-nide validate` after every change to the model: it must report no error.
- An ambiguity reported by ECS NIDE is a design decision to make: add an ordering constraint or
  a phase, do not leave it unresolved.
