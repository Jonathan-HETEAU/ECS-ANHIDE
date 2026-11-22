//! Étapes de `features/02-modelisation.spec.md`.

use std::fs;
use std::path::Path;

use ecs_nide_core::contract::{Contract, contract};
use ecs_nide_core::edit::{
    EditError, Reference, component_references, contains_word, delete_component, rename_component,
};
use ecs_nide_core::model::{
    Access, Archetype, ChannelAccess, Event, Field, Lifecycle, MessageChannel, RelatedSpawn,
    Relationship, Resource, ResourceAccess, Storage, Toggle, Trigger, TriggerAccess, ViaAccess,
};
use serde_yaml::Value;
use spec_markdown::{Registry, Step};

use super::{ensure, list};
use crate::World;

fn yaml_value(text: &str) -> Result<Value, String> {
    serde_yaml::from_str(text).map_err(|e| format!("valeur invalide « {text} » : {e}"))
}

fn field(ty: &str) -> Field {
    Field {
        ty: ty.to_string(),
        min: None,
        max: None,
        values: None,
        max_len: None,
        default: None,
    }
}

fn access(w: &mut World, system: &str, component: &str, access: Access) {
    w.project.ensure_component(component);
    w.project
        .ensure_system(system)
        .components
        .insert(component.to_string(), access);
}

fn contract_of(w: &World, name: &str) -> Result<Contract, String> {
    contract(&w.project, name, &w.profile).ok_or_else(|| format!("aucun contrat pour « {name} »"))
}

/// `Health.current` → le champ `current` du composant `Health`.
fn component_field<'a>(w: &'a World, path: &str) -> Result<&'a Field, String> {
    let (component, field) = path
        .split_once('.')
        .ok_or_else(|| format!("chemin de champ invalide : {path}"))?;
    w.project
        .component(component)
        .ok_or_else(|| format!("composant inconnu : {component}"))?
        .fields
        .get(field)
        .ok_or_else(|| format!("champ inconnu : {path}"))
}

fn ensure_relationship(w: &mut World, name: &str, target: &str) {
    if w.project.relationship(name).is_none() {
        w.project.relationships.push(Relationship {
            name: name.to_string(),
            description: String::new(),
            target_component: target.to_string(),
            linked_spawn: false,
            fields: Default::default(),
        });
    }
}

fn archetype(name: &str, components: Vec<String>) -> Archetype {
    Archetype {
        name: name.to_string(),
        description: String::new(),
        components,
        estimated_count: None,
        related: Vec::new(),
    }
}

/// Une cellule « Position, Stunned (désactivé) » → composants et état activé.
fn entity_cell(cell: &str) -> Vec<(String, bool)> {
    list(cell)
        .into_iter()
        .map(|item| {
            if let Some(name) = item.strip_suffix(" (désactivé)") {
                (name.trim().to_string(), false)
            } else {
                let name = item.strip_suffix(" (activé)").unwrap_or(&item);
                (name.trim().to_string(), true)
            }
        })
        .collect()
}

fn mentions_in_value(value: &Value, word: &str) -> bool {
    match value {
        Value::String(s) => contains_word(s, word),
        Value::Sequence(items) => items.iter().any(|v| mentions_in_value(v, word)),
        Value::Mapping(map) => map
            .iter()
            .any(|(k, v)| mentions_in_value(k, word) || mentions_in_value(v, word)),
        _ => false,
    }
}

fn files_mentioning(dir: &Path, word: &str) -> Vec<String> {
    let mut found = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(current) = stack.pop() {
        for entry in fs::read_dir(&current).into_iter().flatten().flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if matches!(
                path.extension().and_then(|e| e.to_str()),
                Some("yaml" | "md")
            ) && fs::read_to_string(&path).is_ok_and(|t| contains_word(&t, word))
            {
                found.push(path.strip_prefix(dir).unwrap().display().to_string());
            }
        }
    }
    found
}

pub fn register(r: &mut Registry<World>) {
    // --- Composants et ressources --------------------------------------------------------------
    r.step("Déclarer le composant {} avec les champs :", |w, s| {
        let table = s.table()?.clone();
        let name = s.param(0)?.to_string();
        let mut fields = Vec::new();
        for row in 0..table.rows.len() {
            let cell = |h: &str| table.cell(row, h).filter(|c| !c.is_empty());
            let mut f = field(cell("type").ok_or("colonne « type » absente")?);
            f.min = cell("min").map(yaml_value).transpose()?;
            f.max = cell("max").map(yaml_value).transpose()?;
            f.default = cell("défaut").map(yaml_value).transpose()?;
            fields.push((
                cell("champ")
                    .ok_or("colonne « champ » absente")?
                    .to_string(),
                f,
            ));
        }
        w.project.ensure_component(&name).fields.extend(fields);
        Ok(())
    });
    r.step("Le composant {} possède {} champs", |w, s| {
        let (name, count) = (s.param(0)?, s.param(1)?);
        let actual = w
            .project
            .component(name)
            .ok_or("composant inconnu")?
            .fields
            .len();
        ensure(actual.to_string() == count, || format!("{actual} champs"))
    });
    r.step("La valeur par défaut de {} est {}", |w, s| {
        let f = component_field(w, s.param(0)?)?;
        let expected = yaml_value(s.param(1)?)?;
        ensure(f.default.as_ref() == Some(&expected), || {
            format!("défaut : {:?}", f.default)
        })
    });
    r.step("La valeur {} est hors contraintes pour {}", |w, s| {
        let value = yaml_value(s.param(0)?)?;
        let f = component_field(w, s.param(1)?)?;
        ensure(f.check(&value).is_err(), || "la valeur est acceptée".into())
    });
    r.step("Déclarer le composant {} sans champ", |w, s| {
        w.project.ensure_component(s.param(0)?);
        Ok(())
    });
    r.step("{} est un marqueur", |w, s| {
        let c = w
            .project
            .component(s.param(0)?)
            .ok_or("composant inconnu")?;
        ensure(c.is_marker(), || "ce composant a des champs".into())
    });
    r.step("Déclarer le composant {}", |w, s| {
        w.project.ensure_component(s.param(0)?);
        Ok(())
    });
    r.step("Déclarer à nouveau un composant {}", |w, s| {
        w.project
            .components
            .push(ecs_nide_core::model::Component::new(s.param(0)?));
        Ok(())
    });
    r.step(
        "La validation signale un nom de composant en double : {}",
        |w, s| {
            let expected = format!("nom de composant en double : {}", s.param(0)?);
            w.find_diagnostic("VAL-001", |d| d.message == expected)
                .map(|_| ())
        },
    );
    r.step(
        "Déclarer la ressource {} avec le champ {} de type {}",
        |w, s| {
            let mut fields = ecs_nide_core::model::Fields::new();
            fields.insert(s.param(1)?.to_string(), field(s.param(2)?));
            w.project.resources.push(Resource {
                name: s.param(0)?.to_string(),
                description: String::new(),
                fields,
            });
            Ok(())
        },
    );
    r.step(
        "Déclarer le système {} qui lit la ressource {}",
        |w, s| {
            w.project
                .ensure_system(s.param(0)?)
                .resources
                .insert(s.param(1)?.into(), ResourceAccess::Read);
            Ok(())
        },
    );
    r.step(
        "Le contrat de {} contient la ressource {} en lecture",
        |w, s| {
            let c = contract_of(w, s.param(0)?)?;
            let name = s.param(1)?;
            ensure(c.resources.get(name) == Some(&ResourceAccess::Read), || {
                format!("ressources : {:?}", c.resources)
            })?;
            ensure(c.schemas.resources.iter().any(|r| r.name == name), || {
                "schéma de la ressource absent".into()
            })
        },
    );

    // --- Sélection -----------------------------------------------------------------------------
    r.step(
        "Déclarer le système {} qui lit {}, écrit {}, lit {} en optionnel, exige la présence de {} et l'absence de {}",
        |w, s| {
            let system = s.param(0)?;
            access(w, system, s.param(1)?, Access::Read);
            access(w, system, s.param(2)?, Access::Write);
            access(w, system, s.param(3)?, Access::Optional);
            access(w, system, s.param(4)?, Access::With);
            access(w, system, s.param(5)?, Access::Without);
            Ok(())
        },
    );
    r.step("La sélection de {} donne :", |w, s| {
        let name = s.param(0)?;
        let system = w.project.system(name).ok_or("système inconnu")?;
        let table = s.table()?;
        for row in 0..table.rows.len() {
            let entity = entity_cell(
                table
                    .cell(row, "composants de l'entité")
                    .ok_or("colonne absente")?,
            );
            let expected = table.cell(row, "sélectionnée").ok_or("colonne absente")? == "oui";
            let actual = system.io().selects(&entity);
            ensure(actual == expected, || {
                format!("ligne {} : sélection {actual} pour {entity:?}", row + 1)
            })?;
        }
        Ok(())
    });

    // --- Changements structurels ---------------------------------------------------------------
    r.step(
        "Déclarer le système {} qui retire toujours {} et peut ajouter {}",
        |w, s| {
            let system = w.project.ensure_system(s.param(0)?);
            system.structural.remove.push(s.param(1)?.into());
            system.structural.may_add.push(s.param(2)?.into());
            Ok(())
        },
    );
    r.step(
        "Déclarer le système {} qui peut créer des entités de l'archétype {}",
        |w, s| {
            w.project
                .ensure_system(s.param(0)?)
                .structural
                .may_spawn
                .push(s.param(1)?.into());
            Ok(())
        },
    );
    r.step(
        "Déclarer le système {} qui exige {} et détruit toujours l'entité",
        |w, s| {
            access(w, s.param(0)?, s.param(1)?, Access::With);
            w.project.ensure_system(s.param(0)?).structural.despawn = true;
            Ok(())
        },
    );
    r.step(
        "Le contrat de {} indique le retrait certain de {} et l'ajout possible de {}",
        |w, s| {
            let c = contract_of(w, s.param(0)?)?;
            ensure(
                c.structural.remove.iter().any(|x| x == s.param(1).unwrap())
                    && c.structural
                        .may_add
                        .iter()
                        .any(|x| x == s.param(2).unwrap()),
                || format!("changements structurels : {:?}", c.structural),
            )
        },
    );
    r.step(
        "Le contrat de {} indique la création possible de {}",
        |w, s| {
            let c = contract_of(w, s.param(0)?)?;
            let name = s.param(1)?;
            ensure(c.structural.may_spawn.iter().any(|x| x == name), || {
                format!("changements structurels : {:?}", c.structural)
            })
        },
    );
    r.step(
        "Le contrat de {} indique la destruction certaine",
        |w, s| {
            let c = contract_of(w, s.param(0)?)?;
            ensure(c.structural.despawn, || {
                format!("changements structurels : {:?}", c.structural)
            })
        },
    );

    // --- Archétypes ----------------------------------------------------------------------------
    r.step(
        "Déclarer l'archétype {} composé de {…} et {}, avec un volume estimé de {}",
        |w, s| {
            let n = s.params.len();
            ensure(n >= 4, || "paramètres insuffisants".into())?;
            let mut a = archetype(&s.params[0], s.params[1..n - 1].to_vec());
            a.estimated_count = Some(s.params[n - 1].parse().map_err(|_| "volume invalide")?);
            for c in &a.components {
                w.project.ensure_component(c);
            }
            w.project.archetypes.push(a);
            Ok(())
        },
    );
    r.step(
        "L'archétype {} est sélectionné par tout système qui lit {} et {}",
        |w, s| {
            let a = w
                .project
                .archetypes
                .iter()
                .find(|a| a.name == s.param(0).unwrap())
                .ok_or("archétype inconnu")?;
            let mut system = ecs_nide_core::model::System::new("Lecteur");
            system.components.insert(s.param(1)?.into(), Access::Read);
            system.components.insert(s.param(2)?.into(), Access::Read);
            ensure(system.io().selects_archetype(a), || {
                "non sélectionné".into()
            })
        },
    );
    r.step("Déclarer l'archétype {} composé de {} et {}", |w, s| {
        w.project.archetypes.push(archetype(
            s.param(0)?,
            vec![s.param(1)?.into(), s.param(2)?.into()],
        ));
        Ok(())
    });
    r.step(
        "Déclarer l'archétype {} composé de {} et {}, qui crée {} entités {} reliées par {}",
        |w, s| {
            let mut a = archetype(s.param(0)?, vec![s.param(1)?.into(), s.param(2)?.into()]);
            a.related.push(RelatedSpawn {
                relationship: s.param(5)?.into(),
                archetype: s.param(4)?.into(),
                count: s.param(3)?.parse().map_err(|_| "nombre invalide")?,
            });
            w.project.archetypes.push(a);
            Ok(())
        },
    );
    r.step(
        "L'archétype {} déclare {} entités liées de l'archétype {} par la relation {}",
        |w, s| {
            let a = w
                .project
                .archetypes
                .iter()
                .find(|a| a.name == s.param(0).unwrap())
                .ok_or("archétype inconnu")?;
            let expected = RelatedSpawn {
                relationship: s.param(3)?.into(),
                archetype: s.param(2)?.into(),
                count: s.param(1)?.parse().map_err(|_| "nombre invalide")?,
            };
            ensure(a.related.contains(&expected), || {
                format!("entités liées : {:?}", a.related)
            })
        },
    );

    // --- Renommage et suppression --------------------------------------------------------------
    r.step("Renommer le composant {} en {}", |w, s| {
        rename_component(&mut w.project, s.param(0)?, s.param(1)?).map_err(|e| e.to_string())?;
        w.persist()?;
        Ok(())
    });
    r.step(
        "Aucun fichier du projet ne mentionne plus {} comme composant",
        |w, s| {
            let found = files_mentioning(w.dir()?, s.param(0)?);
            ensure(found.is_empty(), || {
                format!("encore mentionné dans : {found:?}")
            })
        },
    );
    r.step(
        "Les systèmes, archétypes, exemples et propriétés qui utilisaient {} utilisent {}",
        |w, s| {
            let (old, new) = (s.param(0)?, s.param(1)?);
            let p = &w.project;
            let systems = p.systems.iter().any(|x| x.io().mentioned_components().iter().any(|c| *c == new));
            let archetypes = p.archetypes.iter().any(|a| a.components.iter().any(|c| c == new));
            let examples = p.systems.iter().flat_map(|x| &x.examples).any(|v| mentions_in_value(v, new));
            let properties = p.systems.iter().flat_map(|x| &x.properties).any(|v| mentions_in_value(v, new));
            ensure(systems && archetypes && examples && properties, || {
                format!("systèmes {systems}, archétypes {archetypes}, exemples {examples}, propriétés {properties}")
            })?;
            let stale = p.systems.iter().any(|x| {
                x.io().mentioned_components().iter().any(|c| *c == old)
                    || x.examples.iter().chain(&x.properties).any(|v| mentions_in_value(v, old))
            }) || p.archetypes.iter().any(|a| a.components.iter().any(|c| c == old));
            ensure(!stale, || format!("{old} est encore référencé"))
        },
    );
    r.step("La validation ne signale aucune nouvelle erreur", |w, _| {
        let new: Vec<String> = w
            .errors()
            .into_iter()
            .filter(|e| !w.baseline_errors.contains(e))
            .collect();
        ensure(new.is_empty(), || new.join("\n"))
    });
    r.step("Demander la suppression du composant {}", |w, s| {
        w.edit_result = Some(delete_component(&mut w.project, s.param(0)?, false));
        Ok(())
    });
    r.step(
        "L'IDE liste les systèmes et archétypes qui référencent {} et demande confirmation",
        |w, s| {
            let name = s.param(0)?;
            let expected: Vec<Reference> = component_references(&w.project, name);
            ensure(w.project.component(name).is_some(), || {
                "le composant a été supprimé sans confirmation".into()
            })?;
            match &w.edit_result {
                Some(Err(EditError::NeedsConfirmation { references, .. })) => {
                    let kinds: Vec<&str> = references.iter().map(|r| r.kind).collect();
                    ensure(
                        kinds.contains(&"système") && kinds.contains(&"archétype"),
                        || format!("références : {references:?}"),
                    )?;
                    ensure(*references == expected, || {
                        format!("références : {references:?}")
                    })
                }
                other => Err(format!("résultat inattendu : {other:?}")),
            }
        },
    );

    // --- Composants activables, filtres et stockage --------------------------------------------
    r.step("Déclarer le composant activable {}", |w, s| {
        w.project.ensure_component(s.param(0)?).toggleable = true;
        Ok(())
    });
    r.step(
        "Déclarer le système {} qui écrit {} et exige l'absence de {}",
        |w, s| {
            access(w, s.param(0)?, s.param(1)?, Access::Write);
            access(w, s.param(0)?, s.param(2)?, Access::Without);
            Ok(())
        },
    );
    r.step(
        "Déclarer le système {} qui écrit {} et peut désactiver {}",
        |w, s| {
            access(w, s.param(0)?, s.param(1)?, Access::Write);
            w.project
                .ensure_system(s.param(0)?)
                .toggles
                .insert(s.param(2)?.into(), Toggle::MayDisable);
            Ok(())
        },
    );
    r.step(
        "Le contrat de {} indique la désactivation possible de {}",
        |w, s| {
            let c = contract_of(w, s.param(0)?)?;
            ensure(
                c.toggles.get(s.param(1)?) == Some(&Toggle::MayDisable),
                || format!("activations : {:?}", c.toggles),
            )
        },
    );
    r.step(
        "Le contrat de {} ne déclare aucun changement structurel",
        |w, s| {
            let c = contract_of(w, s.param(0)?)?;
            ensure(c.structural.is_empty(), || format!("{:?}", c.structural))
        },
    );
    r.step(
        "Déclarer le système {} qui lit {}, écrit {} et filtre les changements de {}",
        |w, s| {
            access(w, s.param(0)?, s.param(1)?, Access::Read);
            access(w, s.param(0)?, s.param(2)?, Access::Write);
            w.project
                .ensure_system(s.param(0)?)
                .changed
                .push(s.param(3)?.into());
            Ok(())
        },
    );
    r.step(
        "Le contrat de {} indique le filtre de changement sur {}",
        |w, s| {
            let c = contract_of(w, s.param(0)?)?;
            ensure(c.changed.iter().any(|x| x == s.param(1).unwrap()), || {
                format!("filtres : {:?}", c.changed)
            })
        },
    );
    r.step("Déclarer le composant {} avec le stockage {}", |w, s| {
        let storage: Storage = serde_yaml::from_str(s.param(1)?).map_err(|e| e.to_string())?;
        w.project.ensure_component(s.param(0)?).storage = storage;
        Ok(())
    });
    r.step(
        "Déclarer le composant {} sans indication de stockage",
        |w, s| {
            w.project.ensure_component(s.param(0)?);
            Ok(())
        },
    );
    r.step("Le stockage de {} est {} et celui de {} est {}", |w, s| {
        for (name, expected) in [(s.param(0)?, s.param(1)?), (s.param(2)?, s.param(3)?)] {
            let c = w.project.component(name).ok_or("composant inconnu")?;
            let actual = serde_yaml::to_string(&c.storage).unwrap();
            ensure(actual.trim() == expected, || format!("{name} : {actual}"))?;
        }
        Ok(())
    });
    r.step(
        "Déclarer le système {} qui lit le composant {}",
        |w, s| {
            access(w, s.param(0)?, s.param(1)?, Access::Read);
            Ok(())
        },
    );
    r.step("Le contrat de {} indique le stockage {} pour {}", |w, s| {
        let c = contract_of(w, s.param(0)?)?;
        let schema = c
            .component_schema(s.param(2)?)
            .ok_or("schéma absent du contrat")?;
        let actual = serde_yaml::to_string(&schema.storage).unwrap();
        ensure(actual.trim() == s.param(1)?, || {
            format!("stockage : {actual}")
        })
    });

    // --- Canaux de messages, per_entity, phases, fréquence -------------------------------------
    r.step(
        "Déclarer le canal de messages {} avec les champs {} de type {} et {} de type {}",
        |w, s| {
            let mut fields = ecs_nide_core::model::Fields::new();
            fields.insert(s.param(1)?.into(), field(s.param(2)?));
            fields.insert(s.param(3)?.into(), field(s.param(4)?));
            w.project.messages.push(MessageChannel {
                name: s.param(0)?.into(),
                description: String::new(),
                fields,
            });
            Ok(())
        },
    );
    r.step(
        "Déclarer le système {} qui peut émettre sur {}",
        |w, s| {
            w.project
                .ensure_system(s.param(0)?)
                .messages
                .insert(s.param(1)?.into(), ChannelAccess::MayEmit);
            Ok(())
        },
    );
    r.step("Déclarer le système {} qui lit le canal {}", |w, s| {
        w.project
            .ensure_system(s.param(0)?)
            .messages
            .insert(s.param(1)?.into(), ChannelAccess::Read);
        Ok(())
    });
    r.step(
        "Le contrat de {} indique l'émission possible sur {}",
        |w, s| {
            let c = contract_of(w, s.param(0)?)?;
            ensure(
                c.messages.get(s.param(1)?) == Some(&ChannelAccess::MayEmit),
                || format!("canaux : {:?}", c.messages),
            )
        },
    );
    r.step(
        "Le contrat de {} indique la lecture de {} et contient le schéma de {}",
        |w, s| {
            let c = contract_of(w, s.param(0)?)?;
            ensure(
                c.messages.get(s.param(1)?) == Some(&ChannelAccess::Read),
                || format!("canaux : {:?}", c.messages),
            )?;
            let channel = s.param(2)?;
            ensure(c.schemas.messages.iter().any(|m| m.name == channel), || {
                "schéma du canal absent".into()
            })
        },
    );
    r.step("Déclarer le système {} avec {} à vrai", |w, s| {
        ensure(s.param(1)? == "per_entity", || "attribut inattendu".into())?;
        w.project.ensure_system(s.param(0)?).per_entity = true;
        Ok(())
    });
    r.step("Déclarer le système {} sans préciser {}", |w, s| {
        w.project.ensure_system(s.param(0)?);
        Ok(())
    });
    r.step(
        "Le contrat de {} indique un traitement par entité",
        |w, s| {
            ensure(contract_of(w, s.param(0)?)?.per_entity, || {
                "per_entity est faux".into()
            })
        },
    );
    r.step(
        "Le contrat de {} indique un traitement qui n'est pas par entité",
        |w, s| {
            ensure(!contract_of(w, s.param(0)?)?.per_entity, || {
                "per_entity est vrai".into()
            })
        },
    );
    r.step(
        "Déclarer les phases {…} et {}, dans cet ordre",
        |w, s| {
            w.project.schedule.phases = s.params.clone();
            Ok(())
        },
    );
    r.step("Rattacher {} à {} et {} à {}", |w, s| {
        for (system, phase) in [(s.param(0)?, s.param(1)?), (s.param(2)?, s.param(3)?)] {
            w.project.ensure_system(system).phase = Some(phase.into());
        }
        Ok(())
    });
    r.step("La phase de {} est {} et celle de {} est {}", |w, s| {
        for (system, phase) in [(s.param(0)?, s.param(1)?), (s.param(2)?, s.param(3)?)] {
            let actual = contract_of(w, system)?.phase;
            ensure(actual.as_deref() == Some(phase), || {
                format!("{system} : {actual:?}")
            })?;
        }
        Ok(())
    });
    r.step("Déclarer le système {} avec la fréquence {}", |w, s| {
        w.project.ensure_system(s.param(0)?).every =
            s.param(1)?.parse().map_err(|_| "fréquence invalide")?;
        Ok(())
    });
    r.step("Déclarer le système {} sans fréquence", |w, s| {
        w.project.ensure_system(s.param(0)?);
        Ok(())
    });
    r.step(
        "La fréquence de {} est {} et celle de {} est {}",
        |w, s| {
            for (system, every) in [(s.param(0)?, s.param(1)?), (s.param(2)?, s.param(3)?)] {
                let actual = contract_of(w, system)?.every;
                ensure(actual.to_string() == every, || {
                    format!("{system} : {actual}")
                })?;
            }
            Ok(())
        },
    );

    // --- Événements, observateurs, relations ---------------------------------------------------
    r.step(
        "Déclarer l'événement d'entité {} avec le champ {} de type {}",
        |w, s| {
            let mut fields = ecs_nide_core::model::Fields::new();
            fields.insert(s.param(1)?.into(), field(s.param(2)?));
            w.project.events.push(Event {
                name: s.param(0)?.into(),
                description: String::new(),
                entity: true,
                propagate: None,
                fields,
            });
            Ok(())
        },
    );
    r.step(
        "Déclarer le système {} qui peut déclencher {}",
        |w, s| {
            w.project
                .ensure_system(s.param(0)?)
                .triggers
                .insert(s.param(1)?.into(), TriggerAccess::MayTrigger);
            Ok(())
        },
    );
    r.step("{} est un événement d'entité", |w, s| {
        let name = s.param(0)?;
        let e = w
            .project
            .events
            .iter()
            .find(|e| e.name == name)
            .ok_or("événement inconnu")?;
        ensure(e.entity, || "ce n'est pas un événement d'entité".into())
    });
    r.step(
        "Le contrat de {} indique le déclenchement possible de {}",
        |w, s| {
            let c = contract_of(w, s.param(0)?)?;
            ensure(
                c.triggers.get(s.param(1)?) == Some(&TriggerAccess::MayTrigger),
                || format!("déclenchements : {:?}", c.triggers),
            )
        },
    );
    r.step(
        "Déclarer l'observateur {} qui réagit à l'ajout de {}, exige {} et écrit la ressource {}",
        |w, s| {
            let on = Trigger::Lifecycle {
                lifecycle: Lifecycle::Add,
                component: s.param(1)?.into(),
            };
            let o = w.project.ensure_observer(s.param(0)?, on);
            o.components.insert(s.param(2)?.into(), Access::With);
            o.resources
                .insert(s.param(3)?.into(), ResourceAccess::Write);
            Ok(())
        },
    );
    r.step(
        "Déclarer l'observateur {} qui réagit à l'événement {} et peut émettre sur le canal {}",
        |w, s| {
            let on = Trigger::Event {
                event: s.param(1)?.into(),
            };
            w.project
                .ensure_observer(s.param(0)?, on)
                .messages
                .insert(s.param(2)?.into(), ChannelAccess::MayEmit);
            Ok(())
        },
    );
    r.step("Le dossier {} contient {…} et {}", |w, s| {
        let dir = w.persist()?;
        let sub = dir.join(s.param(0)?);
        for file in &s.params[1..] {
            ensure(sub.join(file).is_file(), || {
                format!("{file} absent de {}", sub.display())
            })?;
        }
        Ok(())
    });
    r.step("Le déclencheur de {} est l'ajout de {}", |w, s| {
        let c = contract_of(w, s.param(0)?)?;
        let expected = Trigger::Lifecycle {
            lifecycle: Lifecycle::Add,
            component: s.param(1)?.into(),
        };
        ensure(c.trigger.as_ref() == Some(&expected), || {
            format!("déclencheur : {:?}", c.trigger)
        })
    });
    r.step(
        "Déclarer le système {} qui lit {}, écrit {} et lit {} via {}",
        |w, s| {
            let (system, relationship, written, component) =
                (s.param(0)?, s.param(1)?, s.param(2)?, s.param(3)?);
            ensure(s.param(4)? == relationship, || {
                "relation incohérente".into()
            })?;
            ensure_relationship(w, relationship, "Children");
            w.project.ensure_component(written);
            let sys = w.project.ensure_system(system);
            sys.components.insert(relationship.into(), Access::Read);
            sys.components.insert(written.into(), Access::Write);
            sys.via
                .entry(relationship.to_string())
                .or_default()
                .insert(component.into(), ViaAccess::Read);
            Ok(())
        },
    );
    r.step(
        "Le contrat de {} indique la lecture de {} sur la cible de {}",
        |w, s| {
            let c = contract_of(w, s.param(0)?)?;
            let read = c
                .via
                .get(s.param(2)?)
                .and_then(|inner| inner.get(s.param(1).unwrap()));
            ensure(read == Some(&ViaAccess::Read), || {
                format!("via : {:?}", c.via)
            })?;
            ensure(!c.schemas.relationships.is_empty(), || {
                "schéma de la relation absent".into()
            })
        },
    );
    r.step(
        "Déclarer l'événement d'entité {} qui se propage le long de {}",
        |w, s| {
            w.project.events.push(Event {
                name: s.param(0)?.into(),
                description: String::new(),
                entity: true,
                propagate: Some(s.param(1)?.into()),
                fields: Default::default(),
            });
            Ok(())
        },
    );
    r.step(
        "{} se propage de l'entité source vers sa cible par {}",
        |w, s: &Step| {
            let name = s.param(0)?;
            let e = w
                .project
                .events
                .iter()
                .find(|e| e.name == name)
                .ok_or("événement inconnu")?;
            ensure(
                e.entity && e.propagate.as_deref() == Some(s.param(1)?),
                || format!("propagation : {:?}", e.propagate),
            )
        },
    );
}
