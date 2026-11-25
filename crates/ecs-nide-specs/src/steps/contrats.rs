//! Étapes de `features/04-contrats-et-tests.spec.md`.

use std::collections::BTreeSet;
use std::fs;

use ecs_nide_core::contract::contract;
use ecs_nide_core::examples::Example;
use ecs_nide_core::generate::generate;
use ecs_nide_core::model::{
    Access, ChannelAccess, Field, Lifecycle, MessageChannel, Resource, ResourceAccess, Toggle,
    Trigger,
};
use ecs_nide_core::properties::{Property, accessible_data, compile, referenced_data};
use ecs_nide_core::reports::{TestKind, TestStatus, add_counterexample, import_ctrf, import_junit};
use ecs_nide_core::{Diagnostic, EngineProfile, Level};
use serde_json::{Value as Json, json};
use serde_yaml::Value;
use spec_markdown::{Registry, Step};

use super::ensure;
use crate::{World, repo};

// --- Aides -----------------------------------------------------------------------------------

fn yaml(text: &str) -> Result<Value, String> {
    serde_yaml::from_str(text).map_err(|e| format!("valeur invalide « {text} » : {e}"))
}

/// `"Health.current = 0, Hit.amount = 1"` → [(Health, current, 0), (Hit, amount, 1)].
fn assignments(text: &str) -> Result<Vec<(String, String, Value)>, String> {
    text.split(',')
        .map(|part| {
            let (path, value) = part
                .split_once('=')
                .ok_or_else(|| format!("affectation invalide : {part}"))?;
            let (element, field) = path
                .trim()
                .split_once('.')
                .ok_or_else(|| format!("chemin invalide : {path}"))?;
            Ok((element.to_string(), field.to_string(), yaml(value.trim())?))
        })
        .collect()
}

/// La valeur par défaut d'un champ : son défaut, sinon sa borne basse, sinon zéro.
fn default_value(field: &Field) -> Value {
    if let Some(default) = &field.default {
        return default.clone();
    }
    match field.ty.as_str() {
        "bool" => Value::Bool(false),
        "string" => Value::from(""),
        "vec2" => yaml("{ x: 0, y: 0 }").unwrap(),
        "vec3" => yaml("{ x: 0, y: 0, z: 0 }").unwrap(),
        _ => field.min.clone().unwrap_or(Value::from(0)),
    }
}

/// Les valeurs par défaut de tous les champs d'un composant.
fn default_component(w: &World, component: &str) -> Value {
    let mut map = serde_yaml::Mapping::new();
    if let Some(c) = w.project.component(component) {
        for (name, field) in &c.fields {
            map.insert(Value::from(name.as_str()), default_value(field));
        }
    }
    Value::Mapping(map)
}

fn set_field(target: &mut Value, field: &str, value: Value) {
    if !matches!(target, Value::Mapping(_)) {
        *target = Value::Mapping(Default::default());
    }
    if let Value::Mapping(map) = target {
        map.insert(Value::from(field), value);
    }
}

fn examples_mut<'a>(w: &'a mut World, actor: &str) -> Result<&'a mut Vec<Value>, String> {
    if w.project.system(actor).is_some() {
        return Ok(&mut w.project.system_mut(actor).unwrap().examples);
    }
    w.project
        .observers
        .iter_mut()
        .find(|o| o.name == actor)
        .map(|o| &mut o.examples)
        .ok_or_else(|| format!("système ou observateur inconnu : {actor}"))
}

fn find_example(w: &World, actor: &str, id: &str) -> Result<Example, String> {
    let list = w
        .project
        .system(actor)
        .map(|s| &s.examples)
        .or_else(|| w.project.observer(actor).map(|o| &o.examples))
        .ok_or_else(|| format!("système ou observateur inconnu : {actor}"))?;
    let value = list
        .iter()
        .find(|v| v["id"].as_str() == Some(id))
        .ok_or_else(|| format!("exemple inconnu : {id}"))?;
    Example::parse(value)
}

fn push_example(w: &mut World, actor: &str, example: Example) -> Result<(), String> {
    let id = example.id.clone();
    examples_mut(w, actor)?.push(example.to_value());
    w.current_actor = Some(actor.to_string());
    w.current_example = Some(id);
    Ok(())
}

/// Remplace l'exemple courant par sa version modifiée par `edit`.
fn edit_current_example(w: &mut World, edit: impl FnOnce(&mut Example)) -> Result<(), String> {
    let actor = w.current_actor.clone().ok_or("aucun système courant")?;
    let id = w.current_example.clone().ok_or("aucun exemple courant")?;
    let mut example = find_example(w, &actor, &id)?;
    edit(&mut example);
    let list = examples_mut(w, &actor)?;
    let slot = list
        .iter_mut()
        .find(|v| v["id"].as_str() == Some(id.as_str()))
        .ok_or("exemple introuvable")?;
    *slot = example.to_value();
    Ok(())
}

/// Le système qui porte la propriété `id`.
fn owner_of_property(w: &World, id: &str) -> Result<String, String> {
    w.project
        .systems
        .iter()
        .find(|s| s.properties.iter().any(|p| p["id"].as_str() == Some(id)))
        .map(|s| s.name.clone())
        .ok_or_else(|| format!("propriété inconnue : {id}"))
}

fn find_property(w: &World, actor: &str, id: &str) -> Result<Property, String> {
    let system = w.project.system(actor).ok_or("système inconnu")?;
    let value = system
        .properties
        .iter()
        .find(|p| p["id"].as_str() == Some(id))
        .ok_or_else(|| format!("propriété inconnue : {id}"))?;
    Property::parse(value)
}

fn has_diagnostic(
    w: &World,
    what: &str,
    code: &str,
    element: &str,
    parts: &[String],
) -> Result<Diagnostic, String> {
    w.find_diagnostic(what, |d| {
        d.code.as_deref() == Some(code)
            && d.element.as_deref() == Some(element)
            && parts.iter().all(|p| d.message.contains(p.as_str()))
    })
}

fn quoted(name: &str) -> String {
    format!("« {name} »")
}

fn ctrf(tests: Vec<Json>) -> String {
    json!({ "results": { "tool": { "name": "scenario" }, "tests": tests } }).to_string()
}

fn ctrf_test(
    system: &str,
    kind: &str,
    id: &str,
    status: &str,
    counterexample: Option<Json>,
) -> Json {
    let mut extra = json!({ "system": system, "kind": kind, "id": id });
    if let Some(c) = counterexample {
        extra["counterexample"] = c;
    }
    json!({ "name": format!("ecsnide::{system}::{kind}::{id}"), "status": status, "duration": 1, "extra": { "ecsnide": extra } })
}

fn counterexample_json(text: &str) -> Result<Json, String> {
    let mut map = serde_json::Map::new();
    for (element, field, value) in assignments(text)? {
        map.insert(
            format!("{element}.{field}"),
            serde_json::to_value(value).map_err(|e| e.to_string())?,
        );
    }
    Ok(Json::Object(map))
}

fn junit(cases: &[(&str, bool)]) -> String {
    let body: String = cases
        .iter()
        .map(|(name, ok)| {
            if *ok {
                format!(r#"<testcase classname="jeu" name="{name}"/>"#)
            } else {
                format!(r#"<testcase classname="jeu" name="{name}"><failure message="échec"/></testcase>"#)
            }
        })
        .collect();
    format!(
        r#"<?xml version="1.0"?><testsuites><testsuite name="jeu">{body}</testsuite></testsuites>"#
    )
}

fn number_at(input: &ecs_nide_core::generate::Input, path: &str) -> Result<f64, String> {
    let (element, field) = path.split_once('.').ok_or("chemin invalide")?;
    input
        .components
        .get(element)
        .or_else(|| input.resources.get(element))
        .and_then(|e| e.get(field))
        .and_then(Json::as_f64)
        .ok_or_else(|| format!("{path} absent d'une entrée"))
}

fn generate_into(
    w: &mut World,
    s: &Step,
    count: &str,
    property: &str,
    actor: Option<&str>,
    seed: u64,
) -> Result<(), String> {
    let count: usize = count.parse().map_err(|_| "nombre invalide")?;
    let actor = match actor {
        Some(a) => a.to_string(),
        None => owner_of_property(w, property)?,
    };
    let data = generate(&w.project, &actor, property, count, seed).map_err(|e| e.to_string())?;
    ensure(data.inputs.len() == count, || {
        format!(
            "{} entrées générées pour l'étape « {} »",
            data.inputs.len(),
            s.text
        )
    })?;
    w.datasets.push(data);
    Ok(())
}

pub fn register(r: &mut Registry<World>) {
    // --- Contrat (TST-013, TST-016) ------------------------------------------------------------
    r.step("Choisir le moteur cible {}", |w, s| {
        let engine = s.param(0)?;
        w.profile = EngineProfile::for_engine(engine)
            .ok_or_else(|| format!("moteur inconnu : {engine}"))?;
        w.project.manifest.engine = Some(engine.to_string());
        Ok(())
    });
    r.step("Obtenir le contrat du système {}", |w, s| {
        let name = s.param(0)?;
        w.contract = Some(contract(&w.project, name, &w.profile).ok_or("aucun contrat")?);
        w.current_actor = Some(name.to_string());
        Ok(())
    });
    r.step(
        "Le contrat contient la spécification, les accès, les changements structurels et les contraintes d'ordre",
        |w, _| {
            let c = w.contract.as_ref().ok_or("aucun contrat")?;
            let system = w.project.system(&c.name).ok_or("système inconnu")?;
            ensure(!c.spec.is_empty() && c.spec == system.spec, || "spécification absente".into())?;
            ensure(c.components == system.components, || "accès différents".into())?;
            ensure(c.structural == system.structural && !c.structural.is_empty(), || "changements structurels absents".into())?;
            ensure(c.after == system.after && c.before == system.before, || "contraintes d'ordre absentes".into())
        },
    );
    r.step(
        "Le contrat contient la phase, la fréquence et l'attribut {} de {}",
        |w, s| {
            ensure(s.param(0)? == "per_entity", || "attribut inattendu".into())?;
            let c = w.contract.as_ref().ok_or("aucun contrat")?;
            let system = w.project.system(s.param(1)?).ok_or("système inconnu")?;
            ensure(c.phase.is_some() && c.phase == system.phase, || {
                format!("phase : {:?}", c.phase)
            })?;
            ensure(
                c.every == system.every && c.per_entity == system.per_entity,
                || "fréquence ou per_entity différents".into(),
            )
        },
    );
    r.step(
        "Le contrat contient l'indication de stockage de chaque composant utilisé",
        |w, _| {
            let c = w.contract.as_ref().ok_or("aucun contrat")?;
            for name in c
                .components
                .keys()
                .filter(|n| w.project.component(n).is_some())
            {
                let schema = c
                    .component_schema(name)
                    .ok_or_else(|| format!("schéma de {name} absent"))?;
                let expected = w.project.component(name).unwrap().storage;
                ensure(schema.storage == expected, || {
                    format!("stockage de {name} différent")
                })?;
            }
            Ok(())
        },
    );
    r.step("Le contrat contient les schémas de {…} et {}", |w, s| {
        let c = w.contract.as_ref().ok_or("aucun contrat")?;
        for name in &s.params {
            ensure(c.component_schema(name).is_some(), || {
                format!("schéma de {name} absent")
            })?;
        }
        Ok(())
    });
    r.step(
        "Le contrat contient les exemples et les propriétés de {}",
        |w, s| {
            let c = w.contract.as_ref().ok_or("aucun contrat")?;
            let system = w.project.system(s.param(0)?).ok_or("système inconnu")?;
            ensure(
                !c.examples.is_empty() && c.examples == system.examples,
                || "exemples différents".into(),
            )?;
            ensure(
                !c.properties.is_empty() && c.properties == system.properties,
                || "propriétés différentes".into(),
            )
        },
    );
    r.step(
        "Le contrat contient le profil du moteur cible et référence son skill",
        |w, _| {
            let c = w.contract.as_ref().ok_or("aucun contrat")?;
            let engine = w
                .project
                .manifest
                .engine
                .as_deref()
                .ok_or("aucun moteur cible")?;
            ensure(c.engine.profile.name == engine, || {
                format!("profil : {}", c.engine.profile.name)
            })?;
            let expected = format!("skills/{engine}/SKILL.md");
            ensure(c.engine.skill.as_deref() == Some(expected.as_str()), || {
                format!("skill : {:?}", c.engine.skill)
            })
        },
    );
    r.step(
        "Le contrat ne contient aucun schéma de composant que {} n'utilise pas",
        |w, s| {
            let c = w.contract.as_ref().ok_or("aucun contrat")?;
            let system = w.project.system(s.param(0)?).ok_or("système inconnu")?;
            let used: BTreeSet<String> = system
                .io()
                .mentioned_components()
                .into_iter()
                .cloned()
                .collect();
            for schema in &c.schemas.components {
                ensure(used.contains(&schema.name), || {
                    format!("schéma superflu : {}", schema.name)
                })?;
            }
            Ok(())
        },
    );

    // --- Spécification (TST-001) ---------------------------------------------------------------
    r.step("Le scénario {} de {} renvoie à l'exemple {}", |w, s| {
        let (title, actor, id) = (s.param(0)?, s.param(1)?, s.param(2)?);
        let system = w.project.system_mut(actor).ok_or("système inconnu")?;
        let mut lines: Vec<String> = system.spec.lines().map(String::from).collect();
        let at = lines
            .iter()
            .position(|l| l.trim() == format!("## {title}"))
            .ok_or_else(|| format!("scénario introuvable : {title}"))?;
        let tag = format!("Exemples : {id}");
        if lines
            .get(at + 1)
            .is_some_and(|l| l.starts_with("Exemples :"))
        {
            lines[at + 1] = tag;
        } else {
            lines.insert(at + 1, tag);
        }
        system.spec = lines.join("\n") + "\n";
        Ok(())
    });
    r.step(
        "La validation signale un renvoi vers un exemple inconnu dans {}",
        |w, s| {
            let file = s.param(0)?;
            w.find_diagnostic("VAL-001", |d| {
                d.code.as_deref() == Some("VAL-001")
                    && d.message.starts_with("exemple inconnu")
                    && d.file.as_deref() == Some(std::path::Path::new(file))
            })
            .map(|_| ())
        },
    );

    // --- Exemples (TST-002, TST-004, TST-005) --------------------------------------------------
    r.step(
        "L'exemple {} de {} donne en entrée {} à {} et {} à {}",
        |w, s| {
            let (id, actor) = (s.param(0)?, s.param(1)?);
            let example = find_example(w, actor, id)?;
            for (path, value) in [(s.param(2)?, s.param(3)?), (s.param(4)?, s.param(5)?)] {
                let (element, field) = path.split_once('.').ok_or("chemin invalide")?;
                let actual = &example
                    .given
                    .get(element)
                    .ok_or_else(|| format!("{element} absent"))?[field];
                ensure(*actual == yaml(value)?, || {
                    format!("{path} vaut {actual:?}")
                })?;
            }
            w.current_actor = Some(actor.into());
            w.current_example = Some(id.into());
            Ok(())
        },
    );
    r.step("Il attend en sortie {} à {} et l'absence de {}", |w, s| {
        let example = find_example(
            w,
            w.current_actor.as_deref().ok_or("aucun système")?,
            w.current_example.as_deref().ok_or("aucun exemple")?,
        )?;
        let (path, value, absent) = (s.param(0)?, s.param(1)?, s.param(2)?);
        let (element, field) = path.split_once('.').ok_or("chemin invalide")?;
        let actual = &example
            .expect
            .get(element)
            .ok_or_else(|| format!("{element} absent de la sortie"))?[field];
        ensure(*actual == yaml(value)?, || {
            format!("{path} vaut {actual:?}")
        })?;
        ensure(example.expect_absent.iter().any(|c| c == absent), || {
            format!("{absent} n'est pas attendu absent")
        })
    });
    r.step("L'exemple est cohérent avec le contrat de {}", |w, s| {
        let actor = s.param(0)?;
        let problems: Vec<String> = w
            .diagnostics()
            .into_iter()
            .filter(|d| {
                d.element.as_deref() == Some(actor)
                    && d.code.as_deref().is_some_and(|c| c.starts_with("TST-"))
            })
            .map(|d| d.to_string())
            .collect();
        ensure(problems.is_empty(), || problems.join("\n"))
    });
    r.step("Un exemple de {} donne en entrée seulement {}", |w, s| {
        let (actor, component) = (s.param(0)?, s.param(1)?);
        let mut example = Example {
            id: "entree-partielle".into(),
            ..Example::default()
        };
        example
            .given
            .insert(component.into(), default_component(w, component));
        push_example(w, actor, example)
    });
    r.step(
        "La validation signale que {} ne sélectionne pas l'entrée de cet exemple, faute de {}",
        |w, s| {
            has_diagnostic(
                w,
                "TST-004",
                "TST-004",
                s.param(0)?,
                &[format!("faute de {}", quoted(s.param(1)?))],
            )
            .map(|_| ())
        },
    );
    r.step(
        "Un exemple de {} attend en sortie une modification de {}",
        |w, s| {
            let (actor, component) = (s.param(0)?, s.param(1)?);
            let system = w.project.system(actor).ok_or("système inconnu")?;
            let required: Vec<String> = system.io().required().cloned().collect();
            let mut example = Example {
                id: format!("modifie-{}", component.to_lowercase()),
                ..Example::default()
            };
            for c in &required {
                example.given.insert(c.clone(), default_component(w, c));
            }
            example
                .given
                .insert(component.into(), yaml("{ value: { x: 0, y: 0 } }")?);
            example
                .expect
                .insert(component.into(), yaml("{ value: { x: 1, y: 1 } }")?);
            push_example(w, actor, example)
        },
    );
    r.step(
        "La validation signale que {} n'écrit, n'ajoute ni ne retire {}",
        |w, s| {
            has_diagnostic(
                w,
                "TST-005",
                "TST-005",
                s.param(0)?,
                &[format!(
                    "n'écrit, n'ajoute ni ne retire {}",
                    quoted(s.param(1)?)
                )],
            )
            .map(|_| ())
        },
    );
    r.step(
        "Le système {} écrit {}, peut désactiver {} et peut émettre sur le canal {}",
        |w, s| {
            let (actor, component, toggled, channel) =
                (s.param(0)?, s.param(1)?, s.param(2)?, s.param(3)?);
            let c = w.project.ensure_component(component);
            c.toggleable = true;
            c.fields
                .entry("remaining".into())
                .or_insert(serde_yaml::from_str("{ type: f32, min: 0 }").unwrap());
            if !w.project.messages.iter().any(|m| m.name == channel) {
                w.project.messages.push(MessageChannel {
                    name: channel.into(),
                    description: String::new(),
                    fields: Default::default(),
                });
            }
            let system = w.project.ensure_system(actor);
            system.components.insert(component.into(), Access::Write);
            system.toggles.insert(toggled.into(), Toggle::MayDisable);
            system
                .messages
                .insert(channel.into(), ChannelAccess::MayEmit);
            Ok(())
        },
    );
    r.step(
        "L'exemple {} de {} donne en entrée {} activé avec {}",
        |w, s| {
            let (id, actor, component) = (s.param(0)?, s.param(1)?, s.param(2)?);
            let mut example = Example {
                id: id.into(),
                ..Example::default()
            };
            let mut value = Value::Mapping(Default::default());
            for (element, field, v) in assignments(s.param(3)?)? {
                ensure(element == component, || {
                    "affectation d'un autre composant".into()
                })?;
                set_field(&mut value, &field, v);
            }
            example.given.insert(component.into(), value);
            push_example(w, actor, example)
        },
    );
    r.step(
        "Il attend en sortie {} désactivé et un message {}",
        |w, s| {
            let (component, channel) = (s.param(0)?.to_string(), s.param(1)?.to_string());
            edit_current_example(w, |e| {
                e.expect_disabled.push(component);
                e.expect_messages
                    .insert(channel, vec![Value::Mapping(Default::default())]);
            })
        },
    );
    r.step(
        "Le système {} lit le canal {} et n'émet sur aucun canal",
        |w, s| {
            let (actor, channel) = (s.param(0)?, s.param(1)?);
            if !w.project.messages.iter().any(|m| m.name == channel) {
                w.project.messages.push(MessageChannel {
                    name: channel.into(),
                    description: String::new(),
                    fields: Default::default(),
                });
            }
            w.project
                .ensure_system(actor)
                .messages
                .insert(channel.into(), ChannelAccess::Read);
            Ok(())
        },
    );
    r.step("Un exemple de {} attend en sortie un message {}", |w, s| {
        let (actor, channel) = (s.param(0)?, s.param(1)?);
        let mut example = Example {
            id: "message".into(),
            ..Example::default()
        };
        example
            .expect_messages
            .insert(channel.into(), vec![Value::Mapping(Default::default())]);
        push_example(w, actor, example)
    });
    r.step("La validation signale que {} n'émet pas sur {}", |w, s| {
        has_diagnostic(
            w,
            "TST-005",
            "TST-005",
            s.param(0)?,
            &[format!("n'émet pas sur {}", quoted(s.param(1)?))],
        )
        .map(|_| ())
    });

    // --- Propriétés (TST-003, TST-006) ---------------------------------------------------------
    r.step("La propriété {} de {} impose {}", |w, s| {
        let (id, actor, rule) = (s.param(0)?, s.param(1)?, s.param(2)?);
        let property = find_property(w, actor, id)?;
        ensure(property.ensure == rule, || {
            format!("règle : {}", property.ensure)
        })?;
        w.current_property = Some((actor.into(), id.into()));
        Ok(())
    });
    r.step("La règle est une expression CEL valide", |w, _| {
        let (actor, id) = w
            .current_property
            .clone()
            .ok_or("aucune propriété courante")?;
        compile(&find_property(w, &actor, &id)?.ensure).map(|_| ())
    });
    r.step(
        "La règle ne référence que des composants accessibles par {}",
        |w, s| {
            let (actor, id) = w
                .current_property
                .clone()
                .ok_or("aucune propriété courante")?;
            let property = find_property(w, &actor, &id)?;
            let system = w.project.system(s.param(0)?).ok_or("système inconnu")?;
            let accessible = accessible_data(system.io());
            let referenced = referenced_data(&property.ensure);
            ensure(
                !referenced.is_empty() && referenced.is_subset(&accessible),
                || format!("{referenced:?} hors de {accessible:?}"),
            )
        },
    );
    r.step("L'exemple {} de {} fait passer {} de {} à {}", |w, s| {
        let (id, actor, path) = (s.param(0)?, s.param(1)?, s.param(2)?);
        let (element, field) = path.split_once('.').ok_or("chemin invalide")?;
        let system = w.project.system(actor).ok_or("système inconnu")?;
        let required: Vec<String> = system.io().required().cloned().collect();
        let mut example = Example {
            id: id.into(),
            ..Example::default()
        };
        for c in &required {
            example.given.insert(c.clone(), default_component(w, c));
        }
        let mut after = example
            .given
            .get(element)
            .cloned()
            .ok_or_else(|| format!("{element} n'est pas lu par {actor}"))?;
        set_field(
            example.given.get_mut(element).unwrap(),
            field,
            yaml(s.param(3)?)?,
        );
        set_field(&mut after, field, yaml(s.param(4)?)?);
        example.expect.insert(element.into(), after);
        let removed: Vec<String> = system.structural.remove.clone();
        example.expect_absent = removed;
        push_example(w, actor, example)
    });
    r.step(
        "La validation signale que l'exemple {} contredit la propriété {}",
        |w, s| {
            let (example, property) = (s.param(0)?, s.param(1)?);
            let actor = w.current_actor.clone().ok_or("aucun système courant")?;
            has_diagnostic(
                w,
                "TST-006",
                "TST-006",
                &actor,
                &[quoted(example), quoted(property), "contredit".into()],
            )
            .map(|_| ())
        },
    );

    // --- Génération de données (TST-007 à TST-010) ---------------------------------------------
    r.step(
        "Générer {} entrées pour la propriété {} de {}",
        |w, s| {
            let (count, property, actor) = (
                s.param(0)?.to_string(),
                s.param(1)?.to_string(),
                s.param(2)?.to_string(),
            );
            generate_into(w, s, &count, &property, Some(&actor), 0)
        },
    );
    r.step("Générer {} entrées pour {} avec la graine {}", |w, s| {
        let (count, property) = (s.param(0)?.to_string(), s.param(1)?.to_string());
        let seed = s.param(2)?.parse().map_err(|_| "graine invalide")?;
        generate_into(w, s, &count, &property, None, seed)
    });
    r.step(
        "Générer à nouveau {} entrées pour {} avec la graine {}",
        |w, s| {
            let (count, property) = (s.param(0)?.to_string(), s.param(1)?.to_string());
            let seed = s.param(2)?.parse().map_err(|_| "graine invalide")?;
            generate_into(w, s, &count, &property, None, seed)
        },
    );
    r.step("Chaque entrée est sélectionnée par {}", |w, s| {
        let data = w.datasets.last().ok_or("aucune donnée générée")?;
        let system = w.project.system(s.param(0)?).ok_or("système inconnu")?;
        for input in &data.inputs {
            let entity: Vec<(String, bool)> =
                input.components.keys().map(|c| (c.clone(), true)).collect();
            ensure(system.io().selects(&entity), || {
                format!("entrée non sélectionnée : {entity:?}")
            })?;
        }
        Ok(())
    });
    r.step("Chaque {} est compris entre {} et {}", |w, s| {
        let data = w.datasets.last().ok_or("aucune donnée générée")?;
        let (path, min, max) = (s.param(0)?, s.param(1)?, s.param(2)?);
        let (min, max): (f64, f64) = (
            min.parse().map_err(|_| "borne invalide")?,
            max.parse().map_err(|_| "borne invalide")?,
        );
        for input in &data.inputs {
            let n = number_at(input, path)?;
            ensure((min..=max).contains(&n), || format!("{path} vaut {n}"))?;
        }
        Ok(())
    });
    r.step(
        "Les valeurs {} et {} de {} figurent parmi les entrées",
        |w, s| {
            let data = w.datasets.last().ok_or("aucune donnée générée")?;
            let path = s.param(2)?;
            for wanted in [s.param(0)?, s.param(1)?] {
                let wanted: f64 = wanted.parse().map_err(|_| "valeur invalide")?;
                let found = data
                    .inputs
                    .iter()
                    .any(|i| number_at(i, path).is_ok_and(|n| n == wanted));
                ensure(found, || format!("{path} = {wanted} n'est jamais généré"))?;
            }
            Ok(())
        },
    );
    r.step("Les deux jeux de données sont identiques", |w, _| {
        let n = w.datasets.len();
        ensure(n >= 2, || "moins de deux jeux de données".into())?;
        ensure(w.datasets[n - 1] == w.datasets[n - 2], || {
            "les jeux de données diffèrent".into()
        })
    });
    r.step("Exporter les données en JSON", |w, _| {
        let data = w.datasets.last().ok_or("aucune donnée générée")?;
        let path = w.new_dir("donnees").with_extension("json");
        fs::write(&path, data.to_json()).map_err(|e| e.to_string())?;
        w.exported = Some(path);
        Ok(())
    });
    r.step(
        "Le fichier JSON est valide au regard du schéma publié des données de test",
        |w, _| {
            let path = w.exported.as_ref().ok_or("aucun export")?;
            let instance: Json =
                serde_json::from_str(&fs::read_to_string(path).map_err(|e| e.to_string())?)
                    .map_err(|e| e.to_string())?;
            let schema_path =
                repo::root().join("crates/ecs-nide-core/schemas/test-data.schema.json");
            let schema: Json =
                serde_json::from_str(&fs::read_to_string(schema_path).map_err(|e| e.to_string())?)
                    .map_err(|e| e.to_string())?;
            let validator = jsonschema::validator_for(&schema).map_err(|e| e.to_string())?;
            let errors: Vec<String> = validator
                .iter_errors(&instance)
                .map(|e| e.to_string())
                .collect();
            ensure(errors.is_empty(), || errors.join("\n"))?;
            ensure(
                instance["inputs"].as_array().is_some_and(|a| !a.is_empty()),
                || "aucune entrée exportée".into(),
            )
        },
    );

    // --- Rapports de tests (TST-011, TST-012, TST-014, TST-015) --------------------------------
    r.step(
        "Importer un rapport CTRF où l'exemple {} de {} réussit et la propriété {} échoue avec le contre-exemple {}",
        |w, s| {
            let (example, actor, property) = (s.param(0)?, s.param(1)?, s.param(2)?);
            let report = ctrf(vec![
                ctrf_test(actor, "example", example, "passed", None),
                ctrf_test(actor, "property", property, "failed", Some(counterexample_json(s.param(3)?)?)),
            ]);
            w.imported = Some(import_ctrf(&w.project, &report).map_err(|e| e.to_string())?);
            Ok(())
        },
    );
    r.step(
        "Les résultats importés de {} comptent {} test réussi et {} test échoué",
        |w, s| {
            let imported = w.imported.as_ref().ok_or("aucun rapport importé")?;
            let expected = (
                s.param(1)?
                    .parse::<usize>()
                    .map_err(|_| "nombre invalide")?,
                s.param(2)?
                    .parse::<usize>()
                    .map_err(|_| "nombre invalide")?,
            );
            let actual = imported.summary(s.param(0)?);
            ensure(actual == expected, || {
                format!("{actual:?} au lieu de {expected:?}")
            })
        },
    );
    r.step(
        "Le contre-exemple {} est rattaché à la propriété {}",
        |w, s| {
            let imported = w.imported.as_ref().ok_or("aucun rapport importé")?;
            let expected = counterexample_json(s.param(0)?)?;
            let found = imported.results.iter().any(|r| {
                r.kind == TestKind::Property
                    && r.id == s.param(1).unwrap()
                    && r.counterexample.as_ref() == Some(&expected)
            });
            ensure(found, || format!("résultats : {:?}", imported.results))
        },
    );
    r.step(
        "Un rapport importé contient le contre-exemple {} pour la propriété {}",
        |w, s| {
            let property = s.param(1)?;
            let actor = owner_of_property(w, property)?;
            let report = ctrf(vec![ctrf_test(
                &actor,
                "property",
                property,
                "failed",
                Some(counterexample_json(s.param(0)?)?),
            )]);
            w.imported = Some(import_ctrf(&w.project, &report).map_err(|e| e.to_string())?);
            Ok(())
        },
    );
    r.step("Convertir ce contre-exemple en exemple de test", |w, _| {
        let imported = w.imported.clone().ok_or("aucun rapport importé")?;
        let result = imported
            .results
            .iter()
            .find(|r| r.counterexample.is_some())
            .ok_or("aucun contre-exemple")?;
        let id = add_counterexample(&mut w.project, result)?;
        w.converted = Some((result.system.clone(), id));
        Ok(())
    });
    r.step(
        "{} possède un nouvel exemple dont l'entrée est {}",
        |w, s| {
            let (actor, id) = w.converted.clone().ok_or("aucun exemple converti")?;
            ensure(actor == s.param(0)?, || format!("exemple ajouté à {actor}"))?;
            let example = find_example(w, &actor, &id)?;
            for (element, field, value) in assignments(s.param(1)?)? {
                let actual = example
                    .given
                    .get(&element)
                    .map(|e| e[field.as_str()].clone());
                ensure(actual.as_ref() == Some(&value), || {
                    format!("{element}.{field} vaut {actual:?}")
                })?;
            }
            Ok(())
        },
    );
    r.step(
        "La sortie attendue du nouvel exemple reste à compléter et la validation le signale",
        |w, _| {
            let (actor, id) = w.converted.clone().ok_or("aucun exemple converti")?;
            let example = find_example(w, &actor, &id)?;
            ensure(example.incomplete && example.expect.is_empty(), || {
                "l'exemple n'est pas marqué incomplet".into()
            })?;
            let d = has_diagnostic(
                w,
                "TST-012",
                "TST-012",
                &actor,
                &[quoted(&id), "à compléter".into()],
            )?;
            ensure(d.level == Level::Warning, || {
                "le signalement devrait être un avertissement".into()
            })
        },
    );
    r.step(
        "Importer un rapport JUnit XML où le test {} réussit et le test {} échoue",
        |w, s| {
            let xml = junit(&[(s.param(0)?, true), (s.param(1)?, false)]);
            w.imported = Some(import_junit(&w.project, &xml).map_err(|e| e.to_string())?);
            Ok(())
        },
    );
    r.step(
        "L'exemple {} de {} est rattaché à un résultat réussi",
        |w, s| {
            let imported = w.imported.as_ref().ok_or("aucun rapport importé")?;
            let result = imported
                .find(s.param(1)?, TestKind::Example, s.param(0)?)
                .ok_or("aucun résultat")?;
            ensure(result.status == TestStatus::Passed, || {
                format!("statut : {:?}", result.status)
            })
        },
    );
    r.step(
        "La propriété {} de {} est rattachée à un résultat échoué",
        |w, s| {
            let imported = w.imported.as_ref().ok_or("aucun rapport importé")?;
            let result = imported
                .find(s.param(1)?, TestKind::Property, s.param(0)?)
                .ok_or("aucun résultat")?;
            ensure(result.status == TestStatus::Failed, || {
                format!("statut : {:?}", result.status)
            })
        },
    );
    r.step(
        "Importer un rapport contenant les tests {} et {}",
        |w, s| {
            let xml = junit(&[(s.param(0)?, true), (s.param(1)?, true)]);
            w.imported = Some(import_junit(&w.project, &xml).map_err(|e| e.to_string())?);
            Ok(())
        },
    );
    r.step("L'import signale le test sans correspondance {}", |w, s| {
        let imported = w.imported.as_ref().ok_or("aucun rapport importé")?;
        let name = s.param(0)?;
        ensure(imported.unmatched.iter().any(|n| n == name), || {
            format!("sans correspondance : {:?}", imported.unmatched)
        })?;
        ensure(
            imported
                .diagnostics
                .iter()
                .any(|d| d.code.as_deref() == Some("TST-015") && d.message.contains(name)),
            || "aucun diagnostic TST-015".into(),
        )
    });
    r.step("Le résultat de l'exemple {} de {} est importé", |w, s| {
        let imported = w.imported.as_ref().ok_or("aucun rapport importé")?;
        imported
            .find(s.param(1)?, TestKind::Example, s.param(0)?)
            .map(|_| ())
            .ok_or_else(|| "résultat absent".into())
    });

    // --- Observateurs (TST-016) ----------------------------------------------------------------
    r.step(
        "L'observateur {} réagit à l'ajout de {}, exige {} et écrit la ressource {}",
        |w, s| {
            let (name, trigger, required, resource) =
                (s.param(0)?, s.param(1)?, s.param(2)?, s.param(3)?);
            w.project.ensure_component(trigger);
            w.project.ensure_component(required);
            if !w.project.resources.iter().any(|r| r.name == resource) {
                let mut fields = ecs_nide_core::model::Fields::new();
                fields.insert(
                    "score".into(),
                    serde_yaml::from_str("{ type: u32 }").unwrap(),
                );
                w.project.resources.push(Resource {
                    name: resource.into(),
                    description: String::new(),
                    fields,
                });
            }
            let on = Trigger::Lifecycle {
                lifecycle: Lifecycle::Add,
                component: trigger.into(),
            };
            let observer = w.project.ensure_observer(name, on);
            observer.components.insert(required.into(), Access::With);
            observer
                .resources
                .insert(resource.into(), ResourceAccess::Write);
            w.current_actor = Some(name.into());
            Ok(())
        },
    );
    r.step(
        "L'exemple {} donne en entrée l'ajout de {} sur une entité {} et {}",
        |w, s| {
            let actor = w.current_actor.clone().ok_or("aucun observateur courant")?;
            let (id, added, tag) = (s.param(0)?, s.param(1)?, s.param(2)?);
            let mut example = Example {
                id: id.into(),
                ..Example::default()
            };
            example
                .given
                .insert(tag.into(), Value::Mapping(Default::default()));
            example
                .given
                .insert(added.into(), Value::Mapping(Default::default()));
            for (element, field, value) in assignments(s.param(3)?)? {
                let slot = example
                    .given_resources
                    .entry(element)
                    .or_insert(Value::Mapping(Default::default()));
                set_field(slot, &field, value);
            }
            push_example(w, &actor, example)
        },
    );
    r.step("Il attend en sortie {}", |w, s| {
        let values = assignments(s.param(0)?)?;
        let resources: Vec<String> = w.project.resources.iter().map(|r| r.name.clone()).collect();
        edit_current_example(w, |e| {
            for (element, field, value) in values {
                let target = if resources.contains(&element) {
                    &mut e.expect_resources
                } else {
                    &mut e.expect
                };
                let slot = target
                    .entry(element)
                    .or_insert(Value::Mapping(Default::default()));
                set_field(slot, &field, value);
            }
        })
    });
    r.step("Le contrat de {} contient son déclencheur, sa spécification, ses exemples et ses propriétés", |w, s| {
        let name = s.param(0)?;
        let observer = w.project.observer(name).ok_or("observateur inconnu")?;
        let c = contract(&w.project, name, &w.profile).ok_or("aucun contrat")?;
        ensure(c.trigger.as_ref() == Some(&observer.on), || format!("déclencheur : {:?}", c.trigger))?;
        ensure(c.spec == observer.spec, || "spécification différente".into())?;
        ensure(!c.examples.is_empty() && c.examples == observer.examples, || "exemples différents".into())?;
        ensure(c.properties == observer.properties, || "propriétés différentes".into())
    });
}
