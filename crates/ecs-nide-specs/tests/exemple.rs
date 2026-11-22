//! Le projet d'exemple `space-shooter` est exemplaire.
//!
//! Signaler une ambiguïté est le rôle de l'IDE ; la trancher est celui de l'utilisateur. Pour le
//! projet d'exemple, c'est nous l'utilisateur : il ne doit laisser aucune décision en suspens,
//! quel que soit le profil du moteur cible.

use ecs_nide_core::load::load_project;
use ecs_nide_core::schedule::compute_plan;
use ecs_nide_core::{EngineProfile, Project};
use ecs_nide_specs::repo;

fn space_shooter() -> Project {
    let loaded = load_project(&repo::root().join("examples/space-shooter")).unwrap();
    assert!(
        loaded.diagnostics.is_empty(),
        "le projet d'exemple produit des diagnostics :\n{:#?}",
        loaded.diagnostics
    );
    loaded.project
}

fn profiles() -> Vec<EngineProfile> {
    vec![
        EngineProfile::generic(),
        EngineProfile {
            name: "ensembles-disjoints".into(),
            disjoint_sets: true,
            ..EngineProfile::generic()
        },
        EngineProfile {
            name: "sans-parallelisme".into(),
            inter_system_parallelism: false,
            ..EngineProfile::generic()
        },
    ]
}

#[test]
fn le_plan_de_l_exemple_ne_laisse_aucune_ambiguite() {
    let project = space_shooter();
    for profile in profiles() {
        let plan = compute_plan(&project, &profile)
            .unwrap_or_else(|e| panic!("profil {} : {e}", profile.name));
        assert!(
            plan.ambiguities.is_empty(),
            "profil {} : ambiguïtés non tranchées dans l'exemple :\n{:#?}",
            profile.name,
            plan.ambiguities
        );
        assert!(plan.observer_cycles.is_empty(), "profil {}", profile.name);
    }
}

#[test]
fn chaque_systeme_de_l_exemple_appartient_a_une_phase() {
    let project = space_shooter();
    assert!(
        !project.schedule.phases.is_empty(),
        "l'exemple doit déclarer ses phases"
    );
    for system in &project.systems {
        let phase = system
            .phase
            .as_deref()
            .unwrap_or_else(|| panic!("{} n'a pas de phase", system.name));
        assert!(
            project.schedule.phases.iter().any(|p| p == phase),
            "{} appartient à la phase inconnue {phase}",
            system.name
        );
    }
}

#[test]
fn le_rendu_termine_la_boucle_de_l_exemple() {
    let project = space_shooter();
    for profile in profiles() {
        let plan = compute_plan(&project, &profile).unwrap();
        let last = plan.stages.last().expect("le plan a au moins une étape");
        let names: Vec<&str> = last.systems.iter().map(|s| s.name.as_str()).collect();
        assert_eq!(
            names,
            ["Render"],
            "profil {} : dernière étape {names:?}",
            profile.name
        );
    }
}
