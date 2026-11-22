//! Lecture d'un dossier projet (PRJ-005, PRJ-009).

use std::fs;
use std::path::{Path, PathBuf};

use serde::de::DeserializeOwned;

use crate::diagnostic::Diagnostic;
use crate::model::{
    Archetype, Component, Event, FORMAT_VERSION, Manifest, MessageChannel, Observer, Project,
    Relationship, Resource, Schedule, System,
};

pub const MANIFEST_FILE: &str = "ecs-nide.yaml";
pub const SCHEDULE_FILE: &str = "schedule.yaml";

#[derive(Debug, thiserror::Error)]
pub enum LoadError {
    #[error("dossier projet introuvable : {0}")]
    NotADirectory(PathBuf),
    #[error("le fichier {MANIFEST_FILE} est absent de {0}")]
    MissingManifest(PathBuf),
    #[error("{0}")]
    InvalidManifest(Diagnostic),
}

/// Un projet chargé, avec les diagnostics de chargement.
#[derive(Debug, Clone)]
pub struct Loaded {
    pub project: Project,
    pub diagnostics: Vec<Diagnostic>,
    /// Vrai si le format du projet est plus récent que celui que gère ECS NIDE (PRJ-009).
    pub read_only: bool,
}

/// Charge le projet contenu dans `dir`.
///
/// Un fichier invalide produit un diagnostic (fichier, ligne, cause) sans empêcher le chargement
/// des autres fichiers (PRJ-005). Seul un `ecs-nide.yaml` absent ou invalide est bloquant.
pub fn load_project(dir: &Path) -> Result<Loaded, LoadError> {
    if !dir.is_dir() {
        return Err(LoadError::NotADirectory(dir.to_path_buf()));
    }
    let manifest_path = dir.join(MANIFEST_FILE);
    if !manifest_path.is_file() {
        return Err(LoadError::MissingManifest(dir.to_path_buf()));
    }
    let mut diagnostics = Vec::new();
    let manifest: Manifest = parse_file(dir, &manifest_path, &mut diagnostics)
        .ok_or_else(|| LoadError::InvalidManifest(diagnostics.remove(0)))?;

    let read_only = manifest.format > FORMAT_VERSION;
    if read_only {
        diagnostics.push(
            Diagnostic::warning(format!(
                "le projet utilise le format {} alors qu'ECS NIDE gère le format {} : \
                 il est ouvert en lecture seule",
                manifest.format, FORMAT_VERSION
            ))
            .at(MANIFEST_FILE, None),
        );
    }

    let schedule_path = dir.join(SCHEDULE_FILE);
    let schedule = if schedule_path.is_file() {
        parse_file::<Schedule>(dir, &schedule_path, &mut diagnostics).unwrap_or_default()
    } else {
        diagnostics.push(
            Diagnostic::warning("le fichier schedule.yaml est absent").at(SCHEDULE_FILE, None),
        );
        Schedule::default()
    };

    let mut project = Project::new(manifest.name.clone());
    project.manifest = manifest;
    project.schedule = schedule;
    project.components = load_all::<Component>(dir, "components", &mut diagnostics);
    project.resources = load_all::<Resource>(dir, "resources", &mut diagnostics);
    project.messages = load_all::<MessageChannel>(dir, "messages", &mut diagnostics);
    project.events = load_all::<Event>(dir, "events", &mut diagnostics);
    project.relationships = load_all::<Relationship>(dir, "relationships", &mut diagnostics);
    project.archetypes = load_all::<Archetype>(dir, "archetypes", &mut diagnostics);

    for sub in subdirectories(&dir.join("systems")) {
        if let Some(mut system) =
            parse_required::<System>(dir, &sub, "system.yaml", &mut diagnostics)
        {
            system.spec = fs::read_to_string(sub.join("spec.md")).unwrap_or_default();
            project.systems.push(system);
        }
    }
    for sub in subdirectories(&dir.join("observers")) {
        if let Some(mut observer) =
            parse_required::<Observer>(dir, &sub, "observer.yaml", &mut diagnostics)
        {
            observer.spec = fs::read_to_string(sub.join("spec.md")).unwrap_or_default();
            project.observers.push(observer);
        }
    }

    Ok(Loaded {
        project,
        diagnostics,
        read_only,
    })
}

fn relative(root: &Path, path: &Path) -> PathBuf {
    path.strip_prefix(root).unwrap_or(path).to_path_buf()
}

fn parse_file<T: DeserializeOwned>(
    root: &Path,
    path: &Path,
    diagnostics: &mut Vec<Diagnostic>,
) -> Option<T> {
    let rel = relative(root, path);
    let text = match fs::read_to_string(path) {
        Ok(text) => text,
        Err(err) => {
            diagnostics
                .push(Diagnostic::error(format!("lecture impossible : {err}")).at(rel, None));
            return None;
        }
    };
    match serde_yaml::from_str::<T>(&text) {
        Ok(value) => Some(value),
        Err(err) => {
            let line = err.location().map(|l| l.line());
            diagnostics.push(Diagnostic::error(format!("fichier invalide : {err}")).at(rel, line));
            None
        }
    }
}

fn parse_required<T: DeserializeOwned>(
    root: &Path,
    dir: &Path,
    file: &str,
    diagnostics: &mut Vec<Diagnostic>,
) -> Option<T> {
    let path = dir.join(file);
    if !path.is_file() {
        diagnostics.push(
            Diagnostic::error(format!("le fichier {file} est absent"))
                .at(relative(root, dir), None),
        );
        return None;
    }
    parse_file(root, &path, diagnostics)
}

fn load_all<T: DeserializeOwned>(
    root: &Path,
    sub: &str,
    diagnostics: &mut Vec<Diagnostic>,
) -> Vec<T> {
    yaml_files(&root.join(sub))
        .iter()
        .filter_map(|path| parse_file(root, path, diagnostics))
        .collect()
}

fn yaml_files(dir: &Path) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .map(|entry| entry.path())
        .filter(|p| {
            p.is_file() && matches!(p.extension().and_then(|e| e.to_str()), Some("yaml" | "yml"))
        })
        .collect();
    files.sort();
    files
}

fn subdirectories(dir: &Path) -> Vec<PathBuf> {
    let mut dirs: Vec<PathBuf> = fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .map(|entry| entry.path())
        .filter(|p| p.is_dir())
        .collect();
    dirs.sort();
    dirs
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Level;

    fn space_shooter() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/space-shooter")
    }

    fn copy_dir(from: &Path, to: &Path) {
        fs::create_dir_all(to).unwrap();
        for entry in fs::read_dir(from).unwrap().flatten() {
            let target = to.join(entry.file_name());
            if entry.path().is_dir() {
                copy_dir(&entry.path(), &target);
            } else {
                fs::copy(entry.path(), target).unwrap();
            }
        }
    }

    #[test]
    fn le_projet_d_exemple_se_charge_sans_diagnostic() {
        let loaded = load_project(&space_shooter()).unwrap();
        assert!(loaded.diagnostics.is_empty(), "{:#?}", loaded.diagnostics);
        let p = &loaded.project;
        assert_eq!(p.manifest.name, "space-shooter");
        assert_eq!(p.components.len(), 13);
        assert_eq!(p.resources.len(), 3);
        assert_eq!(p.archetypes.len(), 4);
        assert_eq!(p.systems.len(), 9);
        assert!(p.system("Damage").unwrap().spec.contains("# Damage"));
        assert!(!loaded.read_only);
    }

    #[test]
    fn prj_005_un_fichier_invalide_est_signale_sans_bloquer_les_autres() {
        let tmp = tempfile::tempdir().unwrap();
        copy_dir(&space_shooter(), tmp.path());
        fs::write(
            tmp.path().join("components/health.yaml"),
            "name: Health\nfields:\n  current: [\n",
        )
        .unwrap();

        let loaded = load_project(tmp.path()).unwrap();
        let errors: Vec<_> = loaded
            .diagnostics
            .iter()
            .filter(|d| d.level == Level::Error)
            .collect();
        assert_eq!(errors.len(), 1, "{errors:#?}");
        assert_eq!(
            errors[0].file.as_deref(),
            Some(Path::new("components/health.yaml"))
        );
        assert!(errors[0].line.is_some());
        assert_eq!(loaded.project.components.len(), 12);
    }

    #[test]
    fn prj_005_un_champ_inconnu_est_une_erreur() {
        let tmp = tempfile::tempdir().unwrap();
        copy_dir(&space_shooter(), tmp.path());
        fs::write(
            tmp.path().join("components/health.yaml"),
            "name: Health\ncolour: red\n",
        )
        .unwrap();
        let loaded = load_project(tmp.path()).unwrap();
        assert_eq!(loaded.diagnostics.len(), 1);
        assert_eq!(loaded.diagnostics[0].line, Some(2));
    }

    #[test]
    fn prj_009_un_format_plus_recent_ouvre_en_lecture_seule() {
        let tmp = tempfile::tempdir().unwrap();
        copy_dir(&space_shooter(), tmp.path());
        fs::write(tmp.path().join(MANIFEST_FILE), "format: 99\nname: futur\n").unwrap();
        let loaded = load_project(tmp.path()).unwrap();
        assert!(loaded.read_only);
        assert!(
            loaded
                .diagnostics
                .iter()
                .any(|d| d.message.contains("lecture seule"))
        );
    }

    #[test]
    fn un_dossier_sans_manifeste_est_refuse() {
        let tmp = tempfile::tempdir().unwrap();
        assert!(matches!(
            load_project(tmp.path()),
            Err(LoadError::MissingManifest(_))
        ));
    }
}
