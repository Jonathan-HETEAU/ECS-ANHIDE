//! Traçabilité : chaque exigence EARS est couverte par au moins un scénario, et chaque scénario
//! ne cite que des exigences existantes (règle de `AGENTS.md`).

use std::collections::{BTreeMap, BTreeSet};

use ecs_nide_specs::repo;

#[test]
fn les_identifiants_d_exigences_sont_uniques_et_bien_ranges() {
    let mut seen: BTreeMap<String, String> = BTreeMap::new();
    let mut problems = Vec::new();
    for req in repo::requirements() {
        let prefix = &req.id[..3];
        if !req.file.starts_with(&format!("{prefix}-")) {
            problems.push(format!(
                "{} est défini dans {} au lieu d'un fichier {prefix}-…",
                req.id, req.file
            ));
        }
        if let Some(previous) = seen.insert(req.id.clone(), format!("{}:{}", req.file, req.line)) {
            problems.push(format!(
                "{} est défini deux fois ({previous} et {}:{})",
                req.id, req.file, req.line
            ));
        }
    }
    assert!(!seen.is_empty(), "aucune exigence trouvée");
    assert!(problems.is_empty(), "{}", problems.join("\n"));
}

#[test]
fn chaque_scenario_cite_au_moins_une_exigence_existante() {
    let defined: BTreeSet<String> = repo::requirements().into_iter().map(|r| r.id).collect();
    let mut problems = Vec::new();
    for (file, spec) in repo::features() {
        for scenario in &spec.scenarios {
            let cited = scenario.tag("Exigences");
            if cited.is_empty() {
                problems.push(format!(
                    "features/{file}:{} « {} » ne cite aucune exigence",
                    scenario.line, scenario.title
                ));
            }
            for id in cited {
                if !defined.contains(id) {
                    problems.push(format!(
                        "features/{file}:{} cite l'exigence inconnue {id}",
                        scenario.line
                    ));
                }
            }
        }
    }
    assert!(problems.is_empty(), "{}", problems.join("\n"));
}

#[test]
fn chaque_exigence_est_couverte_par_au_moins_un_scenario() {
    let cited: BTreeSet<String> = repo::features()
        .iter()
        .flat_map(|(_, spec)| spec.scenarios.iter())
        .flat_map(|scenario| scenario.tag("Exigences").to_vec())
        .collect();
    let missing: Vec<String> = repo::requirements()
        .into_iter()
        .filter(|r| !cited.contains(&r.id))
        .map(|r| format!("{} ({}:{})", r.id, r.file, r.line))
        .collect();
    assert!(
        missing.is_empty(),
        "exigences sans scénario :\n{}",
        missing.join("\n")
    );
}

#[test]
fn les_titres_de_scenarios_sont_uniques_dans_chaque_fichier() {
    let mut problems = Vec::new();
    for (file, spec) in repo::features() {
        let mut titles = BTreeSet::new();
        for scenario in &spec.scenarios {
            if !titles.insert(scenario.title.clone()) {
                problems.push(format!(
                    "features/{file} : titre en double « {} »",
                    scenario.title
                ));
            }
        }
    }
    assert!(problems.is_empty(), "{}", problems.join("\n"));
}
