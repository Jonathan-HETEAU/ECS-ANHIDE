//! Étapes de la CLI dans `features/07-mcp-et-cli.spec.md`.

use std::fs;

use ecs_nide_core::model::Access;
use ecs_nide_core::ops::{DEFAULT_TEST_COUNT, Session};
use serde_json::{Value as Json, json};
use spec_markdown::Registry;

use super::ensure;
use crate::{CliRun, World};

const AGENTS_FR_MARKER: &str = "# AGENTS.md — Travailler sur le projet";

/// Un rapport CTRF minimal : l'exemple « basique » de « Damage » réussit.
fn sample_report() -> String {
    json!({ "results": { "tool": { "name": "scenario" }, "tests": [
        { "name": "ecsnide::Damage::example::basique", "status": "passed", "duration": 1,
          "extra": { "ecsnide": { "system": "Damage", "kind": "example", "id": "basique" } } }
    ] } })
    .to_string()
}

fn last(w: &World) -> Result<&CliRun, String> {
    w.cli_runs
        .last()
        .ok_or_else(|| "aucune commande exécutée".to_string())
}

fn to_json<T: serde::Serialize>(value: &T) -> Json {
    serde_json::to_value(value).expect("sérialisable")
}

/// Les données que produit le cœur pour la même opération que cette commande (MCP-010).
fn core_result(session: &Session, args: &[String], cwd: &std::path::Path) -> Result<Json, String> {
    let arg = |i: usize| {
        args.get(i)
            .cloned()
            .ok_or_else(|| format!("argument manquant dans {args:?}"))
    };
    let option = |name: &str| {
        args.iter()
            .position(|a| a == name)
            .and_then(|i| args.get(i + 1))
            .cloned()
    };
    match arg(1)?.as_str() {
        "validate" => Ok(to_json(&session.diagnostics())),
        "plan" => Ok(to_json(&session.plan().map_err(|e| e.to_string())?)),
        "contract" => Ok(to_json(
            &session.contract(&arg(2)?).map_err(|e| e.to_string())?,
        )),
        "gen-tests" => {
            let seed = option("--seed")
                .map(|s| s.parse::<u64>())
                .transpose()
                .map_err(|e| e.to_string())?
                .unwrap_or(0);
            let count = option("--count")
                .map(|s| s.parse::<usize>())
                .transpose()
                .map_err(|e| e.to_string())?
                .unwrap_or(DEFAULT_TEST_COUNT);
            let data = session
                .gen_tests(&arg(2)?, option("--property").as_deref(), count, seed)
                .map_err(|e| e.to_string())?;
            Ok(to_json(&data))
        }
        "import-results" => {
            let report = fs::read_to_string(cwd.join(arg(2)?)).map_err(|e| e.to_string())?;
            Ok(to_json(
                &session.import_results(&report).map_err(|e| e.to_string())?,
            ))
        }
        other => Err(format!("commande sans équivalent connu : {other}")),
    }
}

pub fn register(r: &mut Registry<World>) {
    r.step("Exécuter {…} et {} sur le projet d'exemple", |w, s| {
        w.open_example("space-shooter")?;
        let dir = w.dir()?.to_path_buf();
        fs::write(dir.join("rapport.json"), sample_report()).map_err(|e| e.to_string())?;
        for line in &s.params {
            w.run_cli(line, &dir)?;
        }
        Ok(())
    });
    r.step(
        "Chaque commande réussit et produit les mêmes données que l'opération équivalente du cœur, que partagent l'IDE et le serveur MCP",
        |w, _| {
            let runs = w.cli_runs.clone();
            ensure(!runs.is_empty(), || "aucune commande exécutée".into())?;
            for run in runs {
                ensure(run.code == 0, || format!("{} : code {}\n{}", run.args.join(" "), run.code, run.stderr))?;
                let line = format!("{} --format json", run.args.join(" "));
                let json_run = w.run_cli(&line, &run.cwd)?;
                ensure(json_run.code == 0, || format!("{line} : code {}\n{}", json_run.code, json_run.stderr))?;
                let actual: Json = serde_json::from_str(&json_run.stdout).map_err(|e| format!("{line} : sortie non JSON : {e}"))?;
                let session = Session::open(&run.cwd).map_err(|e| e.to_string())?;
                let expected = core_result(&session, &run.args, &run.cwd)?;
                ensure(actual == expected, || format!("{line} : la sortie diffère du résultat du cœur"))?;
            }
            Ok(())
        },
    );
    r.step(
        "Introduire une référence au composant inexistant {} dans le projet d'exemple",
        |w, s| {
            w.open_example("space-shooter")?;
            let movement = w
                .project
                .system_mut("Movement")
                .ok_or("système Movement absent")?;
            movement.components.insert(s.param(0)?.into(), Access::Read);
            w.persist()?;
            Ok(())
        },
    );
    r.step("Exécuter {}", |w, s| {
        let cwd = match &w.project_dir {
            Some(dir) => dir.clone(),
            None => w.new_dir("cli"),
        };
        fs::create_dir_all(&cwd).map_err(|e| e.to_string())?;
        w.run_cli(s.param(0)?, &cwd).map(|_| ())
    });
    r.step("Exécuter {} dans un dossier vide", |w, s| {
        let cwd = w.new_dir("vide");
        fs::create_dir_all(&cwd).map_err(|e| e.to_string())?;
        w.run_cli(s.param(0)?, &cwd).map(|_| ())
    });
    r.step("La commande réussit", |w, _| {
        let run = last(w)?;
        ensure(run.code == 0, || {
            format!("code {}\n{}{}", run.code, run.stdout, run.stderr)
        })
    });
    r.step(
        "La commande se termine avec un code de sortie non nul et affiche le diagnostic",
        |w, _| {
            let run = last(w)?;
            ensure(run.code != 0, || "code de sortie nul".into())?;
            let output = format!("{}{}", run.stdout, run.stderr);
            ensure(
                output.contains("composant inconnu : Mana") && output.contains("VAL-001"),
                || format!("sortie :\n{output}"),
            )
        },
    );
    r.step(
        "La sortie est une liste JSON de diagnostics, dont l'erreur {} avec son code, son fichier et sa ligne",
        |w, s| {
            let run = last(w)?;
            let list: Json = serde_json::from_str(&run.stdout).map_err(|e| format!("sortie non JSON : {e}\n{}", run.stdout))?;
            let items = list.as_array().ok_or("la sortie n'est pas une liste")?;
            let message = s.param(0)?;
            let found = items.iter().any(|d| {
                d["message"] == message
                    && d["level"] == "error"
                    && d["code"].is_string()
                    && d["file"].is_string()
                    && d["line"].is_u64()
            });
            ensure(found, || format!("diagnostic absent de :\n{}", run.stdout))
        },
    );
    r.step(
        "Le dossier {} contient un projet valide, avec un {} rédigé en français",
        |w, s| {
            let run = last(w)?;
            let dir = run.cwd.join(s.param(0)?);
            let session = Session::open(&dir).map_err(|e| e.to_string())?;
            let diagnostics = session.diagnostics();
            ensure(!Session::has_errors(&diagnostics), || {
                format!("{diagnostics:#?}")
            })?;
            let agents = fs::read_to_string(dir.join(s.param(1)?)).map_err(|e| e.to_string())?;
            ensure(agents.starts_with(AGENTS_FR_MARKER), || {
                "AGENTS.md n'est pas en français".into()
            })
        },
    );
}
