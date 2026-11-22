//! Étapes de `features/03-ordonnancement.spec.md`.

use std::fs;

use ecs_nide_core::model::{
    Access, ChannelAccess, Lifecycle, Relationship, ResourceAccess, Toggle, Trigger, TriggerAccess,
    ViaAccess,
};
use ecs_nide_core::schedule::{AccessSet, PLAN_FILE, PlanError};
use spec_markdown::Registry;

use super::{ensure, list};
use crate::World;

fn access(w: &mut World, system: &str, component: &str, access: Access) {
    w.project.ensure_component(component);
    w.project
        .ensure_system(system)
        .components
        .insert(component.to_string(), access);
}

fn different_stages(w: &mut World, a: &str, b: &str) -> Result<(), String> {
    let (sa, sb) = (w.stage_of(a)?, w.stage_of(b)?);
    ensure(sa != sb, || {
        format!("« {a} » et « {b} » sont tous deux dans l'étape {sa}")
    })
}

fn same_stage(w: &mut World, a: &str, b: &str) -> Result<(), String> {
    let (sa, sb) = (w.stage_of(a)?, w.stage_of(b)?);
    ensure(sa == sb, || {
        format!("« {a} » est dans l'étape {sa} et « {b} » dans l'étape {sb}")
    })
}

fn strictly_after(w: &mut World, later: &str, earlier: &str) -> Result<(), String> {
    let (sl, se) = (w.stage_of(later)?, w.stage_of(earlier)?);
    ensure(sl > se, || {
        format!("« {later} » est dans l'étape {sl}, « {earlier} » dans l'étape {se}")
    })
}

fn in_stage(w: &mut World, system: &str, stage: &str) -> Result<(), String> {
    let expected: usize = stage
        .parse()
        .map_err(|_| format!("numéro d'étape invalide : {stage}"))?;
    let actual = w.stage_of(system)?;
    ensure(actual == expected, || {
        format!("« {system} » est dans l'étape {actual}, pas {expected}")
    })
}

fn set_reference_order(w: &mut World, names: &[String]) {
    for name in names {
        w.project.ensure_system(name);
    }
    let mut order: Vec<String> = names.to_vec();
    order.extend(
        w.project
            .schedule
            .order
            .iter()
            .filter(|n| !names.contains(n))
            .cloned(),
    );
    w.project.schedule.order = order;
}

fn ensure_relationship(w: &mut World, name: &str) {
    if w.project.relationship(name).is_none() {
        w.project.relationships.push(Relationship {
            name: name.to_string(),
            description: String::new(),
            target_component: format!("{name}Targets"),
            linked_spawn: false,
            fields: Default::default(),
        });
    }
}

pub fn register(r: &mut Registry<World>) {
    // --- Construction du projet -------------------------------------------------------------
    r.step("Le système {} écrit {}", |w, s| {
        access(w, s.param(0)?, s.param(1)?, Access::Write);
        Ok(())
    });
    r.step("Le système {} lit {}", |w, s| {
        access(w, s.param(0)?, s.param(1)?, Access::Read);
        Ok(())
    });
    r.step("Le système {} écrit la ressource {}", |w, s| {
        w.project
            .ensure_system(s.param(0)?)
            .resources
            .insert(s.param(1)?.into(), ResourceAccess::Write);
        Ok(())
    });
    r.step("Le système {} lit la ressource {}", |w, s| {
        w.project
            .ensure_system(s.param(0)?)
            .resources
            .insert(s.param(1)?.into(), ResourceAccess::Read);
        Ok(())
    });
    fn write_with(w: &mut World, s: &spec_markdown::Step) -> Result<(), String> {
        access(w, s.param(0)?, s.param(1)?, Access::Write);
        access(w, s.param(0)?, s.param(2)?, Access::With);
        Ok(())
    }
    fn write_without(w: &mut World, s: &spec_markdown::Step) -> Result<(), String> {
        access(w, s.param(0)?, s.param(1)?, Access::Write);
        access(w, s.param(0)?, s.param(2)?, Access::Without);
        Ok(())
    }
    r.step(
        "Le système {} écrit {} et exige la présence de {}",
        write_with,
    );
    r.step("{} écrit {} et exige la présence de {}", write_with);
    r.step(
        "Le système {} écrit {} et exige l'absence de {}",
        write_without,
    );
    r.step("{} écrit {} et exige l'absence de {}", write_without);
    r.step("L'ordre de référence est {…}", |w, s| {
        set_reference_order(w, &s.params);
        Ok(())
    });
    r.step("Les accès des systèmes sont :", |w, s| {
        let table = s.table()?.clone();
        for row in 0..table.rows.len() {
            let system = table
                .cell(row, "système")
                .ok_or("colonne « système » absente")?
                .to_string();
            w.project.ensure_system(&system);
            for component in list(table.cell(row, "lit").unwrap_or("")) {
                access(w, &system, &component, Access::Read);
            }
            for component in list(table.cell(row, "écrit").unwrap_or("")) {
                access(w, &system, &component, Access::Write);
            }
        }
        Ok(())
    });
    r.step(
        "Le système {} écrit {} et doit s'exécuter après {}",
        |w, s| {
            access(w, s.param(0)?, s.param(1)?, Access::Write);
            w.project.ensure_system(s.param(2)?);
            w.project
                .ensure_system(s.param(0)?)
                .after
                .push(s.param(2)?.into());
            Ok(())
        },
    );
    r.step("{} doit s'exécuter avant {}", |w, s| {
        // Les systèmes sont créés dans l'ordre où la phrase les cite.
        w.project.ensure_system(s.param(0)?);
        w.project.ensure_system(s.param(1)?);
        w.project
            .ensure_system(s.param(0)?)
            .before
            .push(s.param(1)?.into());
        Ok(())
    });
    r.step(
        "Les systèmes {} et {} écrivent {}, sans contrainte d'ordre entre eux",
        |w, s| {
            access(w, s.param(0)?, s.param(2)?, Access::Write);
            access(w, s.param(1)?, s.param(2)?, Access::Write);
            Ok(())
        },
    );
    r.step("Désactiver le système {} dans le projet", |w, s| {
        let name = s.param(0)?;
        w.project
            .system_mut(name)
            .ok_or_else(|| format!("système inconnu : {name}"))?
            .enabled = false;
        Ok(())
    });
    r.step("Le composant {} est activable", |w, s| {
        w.project.ensure_component(s.param(0)?).toggleable = true;
        Ok(())
    });
    r.step("Le système {} écrit {} et peut désactiver {}", |w, s| {
        access(w, s.param(0)?, s.param(1)?, Access::Write);
        w.project
            .ensure_system(s.param(0)?)
            .toggles
            .insert(s.param(2)?.into(), Toggle::MayDisable);
        Ok(())
    });
    r.step("{} émet sur le canal {} et {} lit {}", |w, s| {
        w.project
            .ensure_system(s.param(0)?)
            .messages
            .insert(s.param(1)?.into(), ChannelAccess::Emit);
        w.project
            .ensure_system(s.param(2)?)
            .messages
            .insert(s.param(3)?.into(), ChannelAccess::Read);
        Ok(())
    });
    r.step("{} émet sur le canal {}, que lit {}", |w, s| {
        w.project
            .ensure_system(s.param(0)?)
            .messages
            .insert(s.param(1)?.into(), ChannelAccess::Emit);
        w.project
            .ensure_system(s.param(2)?)
            .messages
            .insert(s.param(1)?.into(), ChannelAccess::Read);
        Ok(())
    });
    r.step("Les phases sont {} puis {}", |w, s| {
        w.project.schedule.phases = s.params.clone();
        Ok(())
    });
    r.step("{} écrit {} et appartient à {}", |w, s| {
        access(w, s.param(0)?, s.param(1)?, Access::Write);
        w.project.ensure_system(s.param(0)?).phase = Some(s.param(2)?.into());
        Ok(())
    });
    r.step(
        "Le profil du moteur cible prend en charge les ensembles disjoints",
        |w, _| {
            w.profile.disjoint_sets = true;
            Ok(())
        },
    );
    r.step(
        "Le profil du moteur cible ne prend pas en charge les ensembles disjoints",
        |w, _| {
            w.profile.disjoint_sets = false;
            Ok(())
        },
    );
    r.step(
        "Le profil du moteur cible n'exécute pas de systèmes en parallèle",
        |w, _| {
            w.profile.inter_system_parallelism = false;
            Ok(())
        },
    );
    r.step("Le système {} peut ajouter {}", |w, s| {
        w.project.ensure_component(s.param(1)?);
        w.project
            .ensure_system(s.param(0)?)
            .structural
            .may_add
            .push(s.param(1)?.into());
        Ok(())
    });
    r.step(
        "L'observateur {} réagit à l'ajout de {} et peut déclencher l'événement {}",
        |w, s| {
            let on = Trigger::Lifecycle {
                lifecycle: Lifecycle::Add,
                component: s.param(1)?.into(),
            };
            w.project
                .ensure_observer(s.param(0)?, on)
                .triggers
                .insert(s.param(2)?.into(), TriggerAccess::MayTrigger);
            Ok(())
        },
    );
    r.step("L'observateur {} réagit à l'événement {}", |w, s| {
        w.project.ensure_observer(
            s.param(0)?,
            Trigger::Event {
                event: s.param(1)?.into(),
            },
        );
        Ok(())
    });
    r.step(
        "L'observateur {} réagit à l'événement {} et peut déclencher {}",
        |w, s| {
            let on = Trigger::Event {
                event: s.param(1)?.into(),
            };
            w.project
                .ensure_observer(s.param(0)?, on)
                .triggers
                .insert(s.param(2)?.into(), TriggerAccess::MayTrigger);
            Ok(())
        },
    );
    r.step("Le système {} lit {} via {} et écrit {}", |w, s| {
        let (system, component, relationship, written) =
            (s.param(0)?, s.param(1)?, s.param(2)?, s.param(3)?);
        ensure_relationship(w, relationship);
        access(w, system, relationship, Access::Read);
        access(w, system, written, Access::Write);
        w.project
            .ensure_system(system)
            .via
            .entry(relationship.to_string())
            .or_default()
            .insert(component.to_string(), ViaAccess::Read);
        Ok(())
    });

    // --- Calcul ------------------------------------------------------------------------------
    r.step("Calculer le plan d'exécution", |w, _| w.compute_plan());
    r.step("Calculer le plan d'exécution {} fois", |w, s| {
        let count: usize = s
            .param(0)?
            .parse()
            .map_err(|_| "nombre invalide".to_string())?;
        for _ in 0..count {
            w.compute_plan()?;
            let plan = w.plan()?.clone();
            w.plans.push(plan);
        }
        Ok(())
    });

    // --- Vérifications -----------------------------------------------------------------------
    r.step(
        "Chaque système activé figure dans exactement une étape",
        |w, _| {
            let plan = w.plan()?;
            for system in w.project.systems.iter().filter(|s| s.enabled) {
                let count = plan
                    .stages
                    .iter()
                    .flat_map(|stage| &stage.systems)
                    .filter(|p| p.name == system.name)
                    .count();
                ensure(count == 1, || {
                    format!("« {} » figure dans {count} étapes", system.name)
                })?;
            }
            Ok(())
        },
    );
    r.step("{} et {} sont dans des étapes différentes", |w, s| {
        different_stages(w, s.param(0)?, s.param(1)?)
    });
    r.step("{} et {} sont dans la même étape", |w, s| {
        same_stage(w, s.param(0)?, s.param(1)?)
    });
    r.step("Le plan est :", |w, s| {
        let table = s.table()?.clone();
        let plan = w.plan()?;
        ensure(plan.stages.len() == table.rows.len(), || {
            format!(
                "le plan a {} étapes au lieu de {}",
                plan.stages.len(),
                table.rows.len()
            )
        })?;
        for row in 0..table.rows.len() {
            let number: usize = table
                .cell(row, "étape")
                .unwrap_or("")
                .parse()
                .map_err(|_| "numéro d'étape invalide")?;
            let expected = list(table.cell(row, "systèmes").unwrap_or(""));
            let stage = &plan.stages[number - 1];
            let actual: Vec<String> = stage.systems.iter().map(|p| p.name.clone()).collect();
            ensure(actual == expected, || {
                format!("étape {number} : {actual:?} au lieu de {expected:?}")
            })?;
        }
        Ok(())
    });
    r.step(
        "{} est dans une étape strictement postérieure à celle de {}",
        |w, s| strictly_after(w, s.param(0)?, s.param(1)?),
    );
    r.step(
        "{} est dans une étape strictement antérieure à celle de {}",
        |w, s| strictly_after(w, s.param(1)?, s.param(0)?),
    );
    r.step("Aucun plan n'est produit", |w, _| {
        ensure(matches!(w.plan, Some(Err(_))), || {
            "un plan a été produit".to_string()
        })
    });
    r.step("L'erreur nomme le cycle {}", |w, s| {
        let expected = s.param(0)?;
        match &w.plan {
            Some(Err(PlanError::OrderCycle(names))) => {
                let actual = names.join(" → ");
                ensure(actual == expected, || {
                    format!("cycle « {actual} » au lieu de « {expected} »")
                })
            }
            _ => Err("aucune erreur de cycle".to_string()),
        }
    });
    r.step("Une ambiguïté est signalée entre {} et {}", |w, s| {
        let plan = w.plan()?;
        ensure(plan.has_ambiguity(s.param(0)?, s.param(1)?), || {
            format!("ambiguïtés : {:?}", plan.ambiguities)
        })
    });
    r.step(
        "Aucune ambiguïté n'est signalée entre {} et {}",
        |w, s| {
            let plan = w.plan()?;
            ensure(!plan.has_ambiguity(s.param(0)?, s.param(1)?), || {
                format!("ambiguïtés : {:?}", plan.ambiguities)
            })
        },
    );
    r.step("{} est placé avant {}", |w, s| {
        let (a, b) = (s.param(0)?, s.param(1)?);
        let order = &w.plan()?.resolved_order;
        let pa = order
            .iter()
            .position(|n| n == a)
            .ok_or_else(|| format!("{a} absent"))?;
        let pb = order
            .iter()
            .position(|n| n == b)
            .ok_or_else(|| format!("{b} absent"))?;
        ensure(pa < pb, || format!("ordre résolu : {order:?}"))?;
        strictly_after(w, b, a)
    });
    r.step("Les {} plans sont identiques", |w, s| {
        let count: usize = s
            .param(0)?
            .parse()
            .map_err(|_| "nombre invalide".to_string())?;
        ensure(w.plans.len() == count, || {
            format!("{} plans calculés", w.plans.len())
        })?;
        ensure(w.plans.windows(2).all(|p| p[0] == p[1]), || {
            "les plans diffèrent".to_string()
        })
    });
    r.step("{} ne figure dans aucune étape", |w, s| {
        let name = s.param(0)?;
        ensure(w.plan()?.stage_of(name).is_none(), || {
            format!("« {name} » figure dans le plan")
        })
    });
    r.step(
        "Le fichier {} contient les étapes, leurs systèmes, la phase et la fréquence de chaque système, le profil de moteur utilisé et les ambiguïtés signalées",
        |w, s| {
            let file = s.param(0)?;
            ensure(file == PLAN_FILE, || format!("fichier inattendu : {file}"))?;
            let dir = w.project_dir.as_ref().ok_or("aucun dossier projet ouvert")?;
            let text = fs::read_to_string(dir.path().join(file)).map_err(|e| format!("{file} : {e}"))?;
            let yaml: serde_yaml::Value = serde_yaml::from_str(&text).map_err(|e| e.to_string())?;
            ensure(yaml["profile"]["name"].as_str().is_some(), || "profil absent".into())?;
            ensure(yaml["ambiguities"].is_sequence(), || "ambiguïtés absentes".into())?;
            let stages = yaml["stages"].as_sequence().ok_or("étapes absentes")?;
            ensure(!stages.is_empty(), || "aucune étape".into())?;
            let mut count = 0;
            for stage in stages {
                for system in stage["systems"].as_sequence().ok_or("systèmes absents")? {
                    let map = system.as_mapping().ok_or("système mal formé")?;
                    for key in ["name", "phase", "every"] {
                        ensure(map.contains_key(key), || format!("clé « {key} » absente d'un système"))?;
                    }
                    count += 1;
                }
            }
            let enabled = w.project.systems.iter().filter(|s| s.enabled).count();
            ensure(count == enabled, || format!("{count} systèmes exportés sur {enabled}"))
        },
    );
    r.step("{} et {} sont dans l'étape {}", |w, s| {
        in_stage(w, s.param(0)?, s.param(2)?)?;
        in_stage(w, s.param(1)?, s.param(2)?)
    });
    r.step(
        "{} est dans l'étape {}, bien qu'il ne soit en conflit avec aucun autre système",
        |w, s| {
            let name = s.param(0)?;
            let system = w
                .project
                .system(name)
                .ok_or_else(|| format!("système inconnu : {name}"))?;
            let own = AccessSet::of(&w.project, system.io());
            for other in w.project.systems.iter().filter(|o| o.name != name) {
                let conflict = own.conflicts_with(&AccessSet::of(&w.project, other.io()));
                ensure(conflict.is_empty(), || {
                    format!("« {name} » est en conflit avec « {} »", other.name)
                })?;
            }
            in_stage(w, name, s.param(1)?)
        },
    );
    r.step("Chaque étape contient exactement un système", |w, _| {
        let plan = w.plan()?;
        ensure(
            plan.stages.iter().all(|stage| stage.systems.len() == 1),
            || format!("{:?}", plan.stages),
        )
    });
    r.step(
        "Les systèmes se succèdent dans l'ordre résolu",
        |w, _| {
            let plan = w.plan()?;
            let flattened: Vec<&String> = plan
                .stages
                .iter()
                .flat_map(|st| &st.systems)
                .map(|p| &p.name)
                .collect();
            let resolved: Vec<&String> = plan.resolved_order.iter().collect();
            ensure(flattened == resolved, || {
                format!("{flattened:?} au lieu de {resolved:?}")
            })
        },
    );
    r.step("Aucune étape ne contient {} ni {}", |w, s| {
        let plan = w.plan()?;
        for name in [s.param(0)?, s.param(1)?] {
            ensure(plan.stage_of(name).is_none(), || {
                format!("« {name} » occupe une étape")
            })?;
        }
        Ok(())
    });
    r.step(
        "Le plan indique que l'étape de {} peut déclencher {} puis {}",
        |w, s| {
            let (system, first, second) = (s.param(0)?, s.param(1)?, s.param(2)?);
            let number = w.stage_of(system)?;
            let observers = &w.plan()?.stages[number - 1].observers;
            let pf = observers.iter().position(|o| o == first);
            let ps = observers.iter().position(|o| o == second);
            ensure(matches!((pf, ps), (Some(a), Some(b)) if a < b), || {
                format!("observateurs de l'étape {number} : {observers:?}")
            })
        },
    );
    r.step(
        "Un avertissement nomme le cycle d'observateurs {}",
        |w, s| {
            let expected = s.param(0)?;
            let cycles: Vec<String> = w
                .plan()?
                .observer_cycles
                .iter()
                .map(|c| c.join(" → "))
                .collect();
            ensure(cycles.iter().any(|c| c == expected), || {
                format!("cycles signalés : {cycles:?}")
            })
        },
    );
}
