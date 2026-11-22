//! Étapes de `features/01-dossier-projet.spec.md`, et étapes communes sur le projet.

use std::fs;

use ecs_nide_core::create::create_project;
use ecs_nide_core::layout::file_stem;
use ecs_nide_core::load::MANIFEST_FILE;
use ecs_nide_core::model::FORMAT_VERSION;
use ecs_nide_core::save::{SaveError, save_project};
use ecs_nide_core::{Level, Project};
use spec_markdown::{Registry, Step};

use super::ensure;
use crate::World;
use crate::world::snapshot;

/// Marqueurs qui identifient la langue d'un `AGENTS.md` généré.
const AGENTS_EN_MARKER: &str = "# AGENTS.md — Working on the";
const AGENTS_FR_MARKER: &str = "# AGENTS.md — Travailler sur le projet";

fn create(w: &mut World, name: &str, language: Option<&str>) -> Result<(), String> {
    let dir = w.new_dir(&file_stem(name));
    w.project = create_project(&dir, name, language).map_err(|e| e.to_string())?;
    w.project_dir = Some(dir);
    w.load_diagnostics.clear();
    Ok(())
}

fn check_language(w: &mut World, s: &Step, marker: &str) -> Result<(), String> {
    let dir = w.dir()?;
    let (manifest, language, agents) = (s.param(0)?, s.param(1)?, s.param(2)?);
    let text = fs::read_to_string(dir.join(manifest)).map_err(|e| format!("{manifest} : {e}"))?;
    let value: serde_yaml::Value = serde_yaml::from_str(&text).map_err(|e| e.to_string())?;
    ensure(value["language"].as_str() == Some(language), || {
        format!("{manifest} indique la langue {:?}", value["language"])
    })?;
    let agents_text =
        fs::read_to_string(dir.join(agents)).map_err(|e| format!("{agents} : {e}"))?;
    ensure(agents_text.starts_with(marker), || {
        format!(
            "{agents} commence par « {} »",
            agents_text.lines().next().unwrap_or("")
        )
    })
}

fn contains_all(w: &mut World, s: &Step, is_dir: bool) -> Result<(), String> {
    let dir = w.dir()?;
    for name in &s.params {
        let path = dir.join(name);
        let ok = if is_dir {
            path.is_dir()
        } else {
            path.is_file()
        };
        ensure(ok, || format!("{name} est absent de {}", dir.display()))?;
    }
    Ok(())
}

pub fn register(r: &mut Registry<World>) {
    r.step("Ouvrir le projet d'exemple {}", |w, s| {
        w.open_example(s.param(0)?)
    });
    r.step("Créer le projet {} dans un dossier vide", |w, s| {
        create(w, s.param(0)?, None)
    });
    r.step("Créer le projet {} sans préciser de langue", |w, s| {
        create(w, s.param(0)?, None)
    });
    r.step("Créer le projet {} avec la langue {}", |w, s| {
        create(w, s.param(0)?, Some(s.param(1)?))
    });
    r.step("Le dossier contient les fichiers {…} et {}", |w, s| {
        contains_all(w, s, false)
    });
    r.step(
        "Le dossier contient les répertoires {…} et {}",
        |w, s| contains_all(w, s, true),
    );
    r.step(
        "La validation du projet ne signale aucune erreur",
        |w, _| {
            w.reload()?;
            let errors = w.errors();
            ensure(errors.is_empty(), || errors.join("\n"))
        },
    );
    r.step(
        "{} indique la langue {} et {} est rédigé en anglais",
        |w, s| check_language(w, s, AGENTS_EN_MARKER),
    );
    r.step(
        "{} indique la langue {} et {} est rédigé en français",
        |w, s| check_language(w, s, AGENTS_FR_MARKER),
    );
    r.step("Enregistrer le projet dans un nouveau dossier", |w, _| {
        let dir = w.new_dir("copie");
        save_project(&w.project, &dir).map_err(|e| e.to_string())?;
        w.saved_dir = Some(dir);
        Ok(())
    });
    r.step("Rouvrir le projet depuis ce nouveau dossier", |w, _| {
        let dir = w
            .saved_dir
            .clone()
            .ok_or("le projet n'a pas été enregistré")?;
        let loaded = ecs_nide_core::load::load_project(&dir).map_err(|e| e.to_string())?;
        ensure(loaded.diagnostics.is_empty(), || {
            format!("diagnostics au rechargement : {:#?}", loaded.diagnostics)
        })?;
        w.reopened = Some(loaded.project);
        Ok(())
    });
    r.step("Les deux modèles sont identiques", |w, _| {
        let reopened: &Project = w.reopened.as_ref().ok_or("aucun projet rouvert")?;
        ensure(*reopened == w.project, || {
            format!(
                "les modèles diffèrent :\n{:#?}\n---\n{:#?}",
                w.project, reopened
            )
        })
    });
    r.step("Enregistrer le projet sans rien modifier", |w, _| {
        save_project(&w.project, w.dir()?).map_err(|e| e.to_string())
    });
    r.step("Aucun fichier du dossier n'a changé", |w, _| {
        let now = snapshot(w.dir()?);
        let mut changes = Vec::new();
        for (path, bytes) in &now {
            match w.snapshot.get(path) {
                None => changes.push(format!("créé : {}", path.display())),
                Some(before) if before != bytes => {
                    changes.push(format!("modifié : {}", path.display()))
                }
                _ => {}
            }
        }
        for path in w.snapshot.keys().filter(|p| !now.contains_key(*p)) {
            changes.push(format!("supprimé : {}", path.display()));
        }
        ensure(changes.is_empty(), || changes.join("\n"))
    });
    r.step(
        "Remplacer le contenu de {} par un YAML mal formé à la ligne {}",
        |w, s| {
            let (file, line) = (s.param(0)?, s.param(1)?);
            let line: usize = line.parse().map_err(|_| "numéro de ligne invalide")?;
            let mut text = String::new();
            for i in 1..line {
                text.push_str(&format!("key{i}: value\n"));
            }
            // Une tabulation ne peut pas commencer une ligne YAML.
            text.push_str("\tbroken: value\n");
            fs::write(w.dir()?.join(file), text).map_err(|e| e.to_string())?;
            w.reload()
        },
    );
    r.step(
        "La validation signale une erreur dans {} à la ligne {}",
        |w, s| {
            let (file, line) = (s.param(0)?, s.param(1)?);
            let line: usize = line.parse().map_err(|_| "numéro de ligne invalide")?;
            w.find_diagnostic("à cet emplacement", |d| {
                d.level == Level::Error
                    && d.file.as_deref() == Some(std::path::Path::new(file))
                    && d.line == Some(line)
            })
            .map(|_| ())
        },
    );
    r.step("Tous les autres composants sont chargés", |w, _| {
        let files = fs::read_dir(w.dir()?.join("components"))
            .map_err(|e| e.to_string())?
            .count();
        let loaded = w.project.components.len();
        ensure(loaded == files - 1, || {
            format!("{loaded} composants chargés sur {files} fichiers")
        })
    });
    r.step(
        "Ouvrir un projet dont {} déclare une version de format supérieure à celle de l'IDE",
        |w, s| {
            ensure(s.param(0)? == MANIFEST_FILE, || "fichier inattendu".into())?;
            w.open_example("space-shooter")?;
            let path = w.dir()?.join(MANIFEST_FILE);
            fs::write(
                &path,
                format!("format: {}\nname: futur\n", FORMAT_VERSION + 1),
            )
            .map_err(|e| e.to_string())?;
            w.snapshot = snapshot(w.dir()?);
            w.reload()
        },
    );
    r.step(
        "L'IDE ouvre le projet en lecture seule et explique pourquoi",
        |w, _| {
            ensure(w.read_only, || {
                "le projet n'est pas en lecture seule".into()
            })?;
            w.find_diagnostic("PRJ-009", |d| {
                d.code.as_deref() == Some("PRJ-009") && d.message.contains("lecture seule")
            })?;
            let refused = matches!(
                save_project(&w.project, w.dir()?),
                Err(SaveError::NewerFormat(_))
            );
            ensure(refused, || "l'enregistrement n'a pas été refusé".into())?;
            ensure(snapshot(w.dir()?) == w.snapshot, || {
                "des fichiers ont été modifiés".into()
            })
        },
    );
}
