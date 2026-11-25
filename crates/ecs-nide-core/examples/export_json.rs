//! Exporte un projet en JSON pour les prototypes d'interface :
//! `cargo run -p ecs-nide-core --example export_json -- <dossier projet>`.
//!
//! Tout ce que l'interface affiche (plan, contrats, diagnostics) est calculé ici, par le cœur.

use std::path::PathBuf;

use ecs_nide_core::EngineProfile;
use ecs_nide_core::contract::contract;
use ecs_nide_core::load::load_project;
use ecs_nide_core::schedule::compute_plan;
use ecs_nide_core::validate::validate;
use serde_json::{Value, json};

fn main() {
    let dir: PathBuf = std::env::args()
        .nth(1)
        .expect("usage : export_json <dossier projet>")
        .into();
    let loaded = load_project(&dir).unwrap_or_else(|e| panic!("{e}"));
    let p = &loaded.project;
    let profile = EngineProfile::generic();
    let plan = compute_plan(p, &profile).ok();
    let mut diagnostics = loaded.diagnostics.clone();
    diagnostics.extend(validate(p, &profile));

    let to = |v: &dyn erased::Ser| v.value();
    let contracts: Vec<Value> = p
        .systems
        .iter()
        .map(|s| s.name.as_str())
        .chain(p.observers.iter().map(|o| o.name.as_str()))
        .filter_map(|name| contract(p, name, &profile))
        .map(|c| serde_json::to_value(c).unwrap())
        .collect();

    let out = json!({
        "manifest": to(&p.manifest),
        "schedule": to(&p.schedule),
        "components": p.components.iter().map(|c| to(c)).collect::<Vec<_>>(),
        "resources": p.resources.iter().map(|r| to(r)).collect::<Vec<_>>(),
        "archetypes": p.archetypes.iter().map(|a| to(a)).collect::<Vec<_>>(),
        "contracts": contracts,
        "plan": plan.map(|plan| serde_json::to_value(plan).unwrap()),
        "diagnostics": serde_json::to_value(&diagnostics).unwrap(),
    });
    println!("{}", serde_json::to_string_pretty(&out).unwrap());
}

/// Petite aide pour sérialiser des éléments de types différents dans une même fermeture.
mod erased {
    pub trait Ser {
        fn value(&self) -> serde_json::Value;
    }
    impl<T: serde::Serialize> Ser for T {
        fn value(&self) -> serde_json::Value {
            serde_json::to_value(self).unwrap()
        }
    }
}
