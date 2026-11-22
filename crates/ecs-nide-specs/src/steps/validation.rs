//! Étapes de `features/09-validation.spec.md`.

use std::fs;
use std::path::Path;

use ecs_nide_core::layout::system_file;
use ecs_nide_core::model::{
    Access, Archetype, ChannelAccess, Lifecycle, Relationship, Toggle, Trigger, TriggerAccess,
    ViaAccess,
};
use ecs_nide_core::{Diagnostic, Level};
use spec_markdown::Registry;

use super::ensure;
use crate::World;

fn access(w: &mut World, system: &str, component: &str, access: Access) {
    w.project.ensure_component(component);
    w.project
        .ensure_system(system)
        .components
        .insert(component.to_string(), access);
}

fn has(
    code: &'static str,
    level: Level,
    element: Option<String>,
    parts: Vec<String>,
) -> impl Fn(&Diagnostic) -> bool {
    move |d| {
        d.code.as_deref() == Some(code)
            && d.level == level
            && element
                .as_ref()
                .is_none_or(|e| d.element.as_ref() == Some(e))
            && parts.iter().all(|p| d.message.contains(p.as_str()))
    }
}

fn quoted(name: &str) -> String {
    format!("« {name} »")
}

fn nobody_adds(w: &World, component: &str) -> bool {
    !w.project
        .archetypes
        .iter()
        .any(|a| a.components.iter().any(|c| c == component))
        && !w
            .project
            .systems
            .iter()
            .map(|s| s.io())
            .chain(w.project.observers.iter().map(|o| o.io()))
            .any(|io| io.structural.added().any(|c| c == component))
}

pub fn register(r: &mut Registry<World>) {
    r.step("Le système {} lit le composant inexistant {}", |w, s| {
        w.project
            .ensure_system(s.param(0)?)
            .components
            .insert(s.param(1)?.into(), Access::Read);
        ensure(w.project.component(s.param(1)?).is_none(), || {
            "le composant existe".into()
        })
    });
    r.step("La validation signale l'erreur {} dans {}", |w, s| {
        let (message, file) = (s.param(0)?, s.param(1)?);
        w.find_diagnostic("correspondant", |d| {
            d.level == Level::Error
                && d.message == message
                && d.file.as_deref() == Some(Path::new(file))
        })
        .map(|_| ())
    });
    r.step("Le système {} lit le composant {}", |w, s| {
        access(w, s.param(0)?, s.param(1)?, Access::Read);
        Ok(())
    });
    r.step(
        "Aucun archétype ne contient {} et aucun système ne l'ajoute",
        |w, s| {
            ensure(nobody_adds(w, s.param(0)?), || {
                "le composant est fourni".into()
            })
        },
    );
    r.step(
        "La validation signale l'avertissement « {} est lu par {} mais n'est jamais fourni »",
        |w, s| {
            let (component, system) = (s.param(0)?, s.param(1)?);
            w.find_diagnostic(
                "VAL-002",
                has(
                    "VAL-002",
                    Level::Warning,
                    Some(system.into()),
                    vec![quoted(component), "jamais fourni".into()],
                ),
            )
            .map(|_| ())
        },
    );
    r.step("Le système {} exige {} et {}", |w, s| {
        access(w, s.param(0)?, s.param(1)?, Access::With);
        access(w, s.param(0)?, s.param(2)?, Access::With);
        Ok(())
    });
    r.step("Aucun archétype déclaré ne contient {}", |w, s| {
        let name = s.param(0)?;
        ensure(
            !w.project
                .archetypes
                .iter()
                .any(|a| a.components.iter().any(|c| c == name)),
            || "un archétype le contient".into(),
        )
    });
    r.step("Aucun système n'ajoute {}", |w, s| {
        let name = s.param(0)?;
        ensure(
            !w.project
                .systems
                .iter()
                .any(|x| x.io().structural.added().any(|c| c == name)),
            || "un système l'ajoute".into(),
        )
    });
    r.step(
        "La validation signale que {} ne sélectionne aucun archétype atteignable",
        |w, s| {
            w.find_diagnostic(
                "VAL-003",
                has("VAL-003", Level::Warning, Some(s.param(0)?.into()), vec![]),
            )
            .map(|_| ())
        },
    );
    r.step(
        "La validation ne signale pas que {} ne sélectionne aucun archétype atteignable",
        |w, s| {
            let found = w.diagnostics().into_iter().find(has(
                "VAL-003",
                Level::Warning,
                Some(s.param(0)?.into()),
                vec![],
            ));
            ensure(found.is_none(), || {
                format!("diagnostic inattendu : {found:?}")
            })
        },
    );
    r.step("L'archétype {} ne contient pas {}", |w, s| {
        let (name, component) = (s.param(0)?, s.param(1)?);
        if !w.project.archetypes.iter().any(|a| a.name == name) {
            for c in ["Position", "Velocity", "Enemy"] {
                w.project.ensure_component(c);
            }
            w.project.archetypes.push(Archetype {
                name: name.into(),
                description: String::new(),
                components: vec!["Position".into(), "Velocity".into(), "Enemy".into()],
                estimated_count: None,
                related: Vec::new(),
            });
        }
        let a = w
            .project
            .archetypes
            .iter()
            .find(|a| a.name == name)
            .unwrap();
        ensure(!a.components.iter().any(|c| c == component), || {
            "l'archétype le contient".into()
        })
    });
    r.step(
        "Le système {} peut ajouter {} aux entités qui possèdent {}",
        |w, s| {
            access(w, s.param(0)?, s.param(2)?, Access::With);
            w.project.ensure_component(s.param(1)?);
            w.project
                .ensure_system(s.param(0)?)
                .structural
                .may_add
                .push(s.param(1)?.into());
            Ok(())
        },
    );
    r.step(
        "Le composant {} n'est utilisé par aucun système ni aucun archétype",
        |w, s| {
            let name = s.param(0)?;
            w.project.ensure_component(name);
            let used = w
                .project
                .systems
                .iter()
                .any(|x| x.io().mentioned_components().iter().any(|c| *c == name))
                || w.project
                    .archetypes
                    .iter()
                    .any(|a| a.components.iter().any(|c| c == name));
            ensure(!used, || "le composant est utilisé".into())
        },
    );
    r.step("La validation signale que {} est inutilisé", |w, s| {
        w.find_diagnostic(
            "VAL-004",
            has(
                "VAL-004",
                Level::Warning,
                Some(s.param(0)?.into()),
                vec!["inutilisé".into()],
            ),
        )
        .map(|_| ())
    });
    r.step(
        "Le système {} a un fichier {} vide et aucun exemple ni propriété",
        |w, s| {
            let system = w.project.ensure_system(s.param(0)?);
            system.spec.clear();
            system.examples.clear();
            system.properties.clear();
            Ok(())
        },
    );
    r.step(
        "La validation signale que {} n'a pas de spécification",
        |w, s| {
            w.find_diagnostic(
                "VAL-005",
                has(
                    "VAL-005",
                    Level::Warning,
                    Some(s.param(0)?.into()),
                    vec!["pas de spécification".into()],
                ),
            )
            .map(|_| ())
        },
    );
    r.step("La validation signale que {} n'a aucun test", |w, s| {
        w.find_diagnostic(
            "VAL-005",
            has(
                "VAL-005",
                Level::Warning,
                Some(s.param(0)?.into()),
                vec!["aucun test".into()],
            ),
        )
        .map(|_| ())
    });
    r.step(
        "Ouvrir un projet qui contient des erreurs et des avertissements",
        |w, _| {
            w.open_example("space-shooter")?;
            access(w, "Magic", "Mana", Access::Read);
            w.project.components.retain(|c| c.name != "Mana");
            w.project.ensure_component("Legacy");
            w.persist()?;
            w.reload()
        },
    );
    r.step(
        "Chaque diagnostic indique un niveau, un message, un élément du modèle, un fichier et une ligne",
        |w, _| {
            let all = w.diagnostics();
            ensure(all.iter().any(|d| d.level == Level::Error), || "aucune erreur".into())?;
            ensure(all.iter().any(|d| d.level == Level::Warning), || "aucun avertissement".into())?;
            for d in &all {
                ensure(
                    !d.message.is_empty() && d.element.is_some() && d.file.is_some() && d.line.is_some(),
                    || format!("diagnostic incomplet : {d:?}"),
                )?;
            }
            Ok(())
        },
    );
    r.step("Le système {} n'est rattaché à aucune phase", |w, s| {
        w.project.ensure_system(s.param(0)?).phase = None;
        Ok(())
    });
    r.step(
        "La validation signale l'erreur « {} n'est rattaché à aucune phase »",
        |w, s| {
            let system = s.param(0)?;
            w.find_diagnostic(
                "VAL-008",
                has(
                    "VAL-008",
                    Level::Error,
                    Some(system.into()),
                    vec![format!("{} n'est rattaché à aucune phase", quoted(system))],
                ),
            )
            .map(|_| ())
        },
    );
    r.step("Le composant {} n'est pas activable", |w, s| {
        w.project.ensure_component(s.param(0)?).toggleable = false;
        Ok(())
    });
    r.step("Le système {} déclare l'activation de {}", |w, s| {
        w.project
            .ensure_system(s.param(0)?)
            .toggles
            .insert(s.param(1)?.into(), Toggle::Enable);
        Ok(())
    });
    r.step(
        "La validation signale que {} n'est pas activable",
        |w, s| {
            let component = s.param(0)?;
            w.find_diagnostic(
                "VAL-009",
                has(
                    "VAL-009",
                    Level::Error,
                    None,
                    vec![format!("{} n'est pas activable", quoted(component))],
                ),
            )
            .map(|_| ())
        },
    );
    r.step(
        "Le système {} lit {} et filtre les changements de {}",
        |w, s| {
            access(w, s.param(0)?, s.param(1)?, Access::Read);
            w.project.ensure_component(s.param(2)?);
            w.project
                .ensure_system(s.param(0)?)
                .changed
                .push(s.param(2)?.into());
            Ok(())
        },
    );
    r.step(
        "La validation signale que {} filtre {} sans le sélectionner",
        |w, s| {
            let (system, component) = (s.param(0)?, s.param(1)?);
            w.find_diagnostic(
                "VAL-010",
                has(
                    "VAL-010",
                    Level::Error,
                    Some(system.into()),
                    vec![quoted(component), "sans le sélectionner".into()],
                ),
            )
            .map(|_| ())
        },
    );
    r.step(
        "Le fichier du système {} déclare le canal {} deux fois, en émission puis en lecture",
        |w, s| {
            let (system, channel) = (s.param(0)?, s.param(1)?);
            w.project.ensure_system(system);
            let dir = w.persist()?;
            let text = format!("name: {system}\nmessages:\n  {channel}: emit\n  {channel}: read\n");
            fs::write(dir.join(system_file(system)), text).map_err(|e| e.to_string())?;
            w.reload()
        },
    );
    r.step(
        "La validation signale la clé en double {} dans {}",
        |w, s| {
            let (key, file) = (s.param(0)?, s.param(1)?);
            let d = w.find_diagnostic(
                "VAL-011",
                has("VAL-011", Level::Error, None, vec![quoted(key)]),
            )?;
            ensure(d.file.as_deref() == Some(Path::new(file)), || {
                format!("fichier : {:?}", d.file)
            })
        },
    );
    r.step(
        "Le système {} lit le canal {} et aucun système n'émet sur {}",
        |w, s| {
            w.project
                .ensure_system(s.param(0)?)
                .messages
                .insert(s.param(1)?.into(), ChannelAccess::Read);
            let channel = s.param(2)?;
            ensure(
                !w.project
                    .systems
                    .iter()
                    .any(|x| x.messages.get(channel).is_some_and(|a| a.emits())),
                || "un système émet sur ce canal".into(),
            )
        },
    );
    r.step(
        "Le système {} émet sur le canal {} et aucun système ne lit {}",
        |w, s| {
            w.project
                .ensure_system(s.param(0)?)
                .messages
                .insert(s.param(1)?.into(), ChannelAccess::Emit);
            let channel = s.param(2)?;
            ensure(
                !w.project
                    .systems
                    .iter()
                    .any(|x| x.messages.get(channel) == Some(&ChannelAccess::Read)),
                || "un système lit ce canal".into(),
            )
        },
    );
    r.step(
        "La validation signale que {} est lu mais jamais émis",
        |w, s| {
            w.find_diagnostic(
                "VAL-012",
                has(
                    "VAL-012",
                    Level::Warning,
                    None,
                    vec![quoted(s.param(0)?), "lu mais jamais émis".into()],
                ),
            )
            .map(|_| ())
        },
    );
    r.step(
        "La validation signale que {} est émis mais jamais lu",
        |w, s| {
            w.find_diagnostic(
                "VAL-012",
                has(
                    "VAL-012",
                    Level::Warning,
                    None,
                    vec![quoted(s.param(0)?), "émis mais jamais lu".into()],
                ),
            )
            .map(|_| ())
        },
    );
    r.step(
        "Le profil du moteur cible ne prend pas en charge les composants activables",
        |w, _| {
            w.profile.toggleable_components = false;
            Ok(())
        },
    );
    r.step("Le projet déclare le composant activable {}", |w, s| {
        w.project.ensure_component(s.param(0)?).toggleable = true;
        Ok(())
    });
    r.step(
        "La validation signale que les composants activables ne sont pas pris en charge par le moteur cible et renvoie à son skill",
        |w, _| {
            w.find_diagnostic(
                "VAL-013",
                has("VAL-013", Level::Warning, None, vec!["composants activables".into(), "SKILL.md".into()]),
            )
            .map(|_| ())
        },
    );
    r.step("L'observateur {} réagit à l'ajout de {}", |w, s| {
        w.project.ensure_component(s.param(1)?);
        let on = Trigger::Lifecycle {
            lifecycle: Lifecycle::Add,
            component: s.param(1)?.into(),
        };
        w.project.ensure_observer(s.param(0)?, on);
        Ok(())
    });
    r.step(
        "Aucun système ni observateur ne peut ajouter {}, et aucun archétype ne le contient",
        |w, s| {
            ensure(nobody_adds(w, s.param(0)?), || {
                "le composant est fourni".into()
            })
        },
    );
    r.step(
        "La validation signale que {} ne peut jamais être déclenché",
        |w, s| {
            w.find_diagnostic(
                "VAL-014",
                has("VAL-014", Level::Warning, Some(s.param(0)?.into()), vec![]),
            )
            .map(|_| ())
        },
    );
    r.step(
        "Le système {} peut déclencher l'événement {}",
        |w, s| {
            w.project
                .ensure_system(s.param(0)?)
                .triggers
                .insert(s.param(1)?.into(), TriggerAccess::MayTrigger);
            Ok(())
        },
    );
    r.step("Aucun observateur ne réagit à {}", |w, s| {
        let event = s.param(0)?;
        ensure(
            !w.project
                .observers
                .iter()
                .any(|o| matches!(&o.on, Trigger::Event { event: e } if e == event)),
            || "un observateur réagit à cet événement".into(),
        )
    });
    r.step(
        "La validation signale que {} est déclenché mais jamais observé",
        |w, s| {
            w.find_diagnostic(
                "VAL-015",
                has("VAL-015", Level::Warning, None, vec![quoted(s.param(0)?)]),
            )
            .map(|_| ())
        },
    );
    r.step(
        "Le système {} lit {} via {} sans sélectionner {}",
        |w, s| {
            let (system, component, relationship) = (s.param(0)?, s.param(1)?, s.param(2)?);
            ensure(s.param(3)? == relationship, || {
                "relation incohérente".into()
            })?;
            if w.project.relationship(relationship).is_none() {
                w.project.relationships.push(Relationship {
                    name: relationship.into(),
                    description: String::new(),
                    target_component: "Children".into(),
                    linked_spawn: false,
                    fields: Default::default(),
                });
            }
            w.project.ensure_component(component);
            w.project
                .ensure_system(system)
                .via
                .entry(relationship.to_string())
                .or_default()
                .insert(component.into(), ViaAccess::Read);
            Ok(())
        },
    );
    r.step(
        "La validation signale que {} doit sélectionner {} pour lire via cette relation",
        |w, s| {
            w.find_diagnostic(
                "VAL-016",
                has(
                    "VAL-016",
                    Level::Error,
                    Some(s.param(0)?.into()),
                    vec![quoted(s.param(1)?)],
                ),
            )
            .map(|_| ())
        },
    );
}
