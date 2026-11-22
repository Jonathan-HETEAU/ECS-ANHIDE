//! Exécute tous les scénarios de `features/`.
//!
//! - Un scénario **actif** (listé dans `features/actifs.yaml`) doit passer : une étape sans
//!   fonction ou un échec fait échouer ce test.
//! - Un scénario **inactif** est exécuté s'il peut l'être, et son état est seulement rapporté.
//!   S'il passe, le rapport demande de l'activer.

use std::collections::BTreeSet;

use ecs_nide_specs::repo::{self, ACTIVE_FILE};
use ecs_nide_specs::{World, steps};
use spec_markdown::Outcome;

#[test]
fn scenarios_actifs() {
    let registry = steps::registry();
    let active = repo::active();
    let features = repo::features();
    let mut problems = Vec::new();
    let mut to_activate = Vec::new();
    let mut lines = Vec::new();
    let mut covered: BTreeSet<String> = BTreeSet::new();
    let (mut total, mut total_active, mut total_passed) = (0, 0, 0);

    for (file, selection) in &active {
        match features.iter().find(|(name, _)| name == file) {
            None => problems.push(format!("{ACTIVE_FILE} cite un fichier inconnu : {file}")),
            Some((_, spec)) => {
                if let repo::Selection::Titles(titles) = selection {
                    for title in titles {
                        if !spec.scenarios.iter().any(|s| &s.title == title) {
                            problems.push(format!(
                                "{ACTIVE_FILE} cite un scénario inconnu de {file} : « {title} »"
                            ));
                        }
                    }
                }
            }
        }
    }

    for (file, spec) in &features {
        let selection = active.get(file);
        let (mut passed, mut failed, mut undefined, mut active_count) = (0, 0, 0, 0);
        for scenario in &spec.scenarios {
            total += 1;
            let is_active = selection.is_some_and(|s| s.contains(&scenario.title));
            let location = format!("features/{file}:{} « {} »", scenario.line, scenario.title);
            let outcome = registry.run(scenario, &mut World::default());
            if is_active {
                active_count += 1;
            }
            match outcome {
                Outcome::Passed => {
                    passed += 1;
                    if is_active {
                        covered.extend(scenario.tag("Exigences").iter().cloned());
                    } else {
                        to_activate.push(location);
                    }
                }
                Outcome::Failed {
                    line,
                    step,
                    message,
                } => {
                    failed += 1;
                    if is_active {
                        problems.push(format!(
                            "{location}\n    ligne {line} : {step}\n    → {message}"
                        ));
                    }
                }
                Outcome::Undefined(steps) => {
                    undefined += 1;
                    if is_active {
                        for step in steps {
                            problems.push(format!(
                                "{location}\n    ligne {} : étape sans fonction « {} »",
                                step.line, step.pattern
                            ));
                        }
                    }
                }
            }
        }
        total_active += active_count;
        total_passed += passed;
        lines.push(format!(
            "  {file:<32} {:>3} scénarios | {active_count:>3} actifs | {passed:>3} passent | {failed:>3} échouent | {undefined:>3} non implémentés",
            spec.scenarios.len()
        ));
    }

    let requirements = repo::requirements().len();
    println!("\nScénarios de features/ :\n{}", lines.join("\n"));
    println!(
        "  Total : {total} scénarios, {total_active} actifs, {total_passed} passent.\n  \
         Exigences couvertes par un scénario actif qui passe : {}/{requirements}",
        covered.len()
    );
    if !to_activate.is_empty() {
        println!("\nScénarios qui passent et peuvent être activés dans {ACTIVE_FILE} :");
        for location in &to_activate {
            println!("  {location}");
        }
    }
    assert!(problems.is_empty(), "\n{}\n", problems.join("\n"));
}
