//! Enregistrement d'un projet dans son dossier (PRJ-003, PRJ-006, PRJ-009).

use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::layout::{self, DIRECTORIES};
use crate::load::{EXAMPLES_FILE, MANIFEST_FILE, PROPERTIES_FILE, SCHEDULE_FILE, SPEC_FILE};
use crate::model::{FORMAT_VERSION, Project};

#[derive(Debug, thiserror::Error)]
pub enum SaveError {
    #[error(
        "le projet utilise le format {0}, plus récent que le format {FORMAT_VERSION} d'ECS NIDE : \
         il n'est jamais réécrit"
    )]
    NewerFormat(u32),
    #[error("« {first} » et « {second} » s'enregistreraient dans le même fichier {path}")]
    PathCollision {
        first: String,
        second: String,
        path: PathBuf,
    },
    #[error("écriture impossible de {path} : {source}")]
    Io { path: PathBuf, source: io::Error },
}

fn yaml<T: Serialize>(value: &T) -> String {
    serde_yaml::to_string(value).expect("le modèle est toujours sérialisable en YAML")
}

/// Le contenu de chaque fichier du projet, par chemin relatif. Le rendu est déterministe : un
/// même projet donne toujours exactement les mêmes fichiers (PRJ-006).
pub fn render(project: &Project) -> Result<BTreeMap<PathBuf, String>, SaveError> {
    let mut files: BTreeMap<PathBuf, String> = BTreeMap::new();
    let mut owners: BTreeMap<PathBuf, String> = BTreeMap::new();
    let mut add = |path: PathBuf, owner: &str, content: String| -> Result<(), SaveError> {
        if let Some(first) = owners.insert(path.clone(), owner.to_string()) {
            return Err(SaveError::PathCollision {
                first,
                second: owner.to_string(),
                path,
            });
        }
        files.insert(path, content);
        Ok(())
    };

    add(MANIFEST_FILE.into(), MANIFEST_FILE, yaml(&project.manifest))?;
    add(SCHEDULE_FILE.into(), SCHEDULE_FILE, yaml(&project.schedule))?;
    for c in &project.components {
        add(layout::component_file(&c.name), &c.name, yaml(c))?;
    }
    for r in &project.resources {
        add(layout::resource_file(&r.name), &r.name, yaml(r))?;
    }
    for m in &project.messages {
        add(layout::message_file(&m.name), &m.name, yaml(m))?;
    }
    for e in &project.events {
        add(layout::event_file(&e.name), &e.name, yaml(e))?;
    }
    for r in &project.relationships {
        add(layout::relationship_file(&r.name), &r.name, yaml(r))?;
    }
    for a in &project.archetypes {
        add(layout::archetype_file(&a.name), &a.name, yaml(a))?;
    }
    for s in &project.systems {
        let dir = layout::system_dir(&s.name);
        add(dir.join("system.yaml"), &s.name, yaml(s))?;
        add(dir.join(SPEC_FILE), &s.name, s.spec.clone())?;
        add(dir.join(EXAMPLES_FILE), &s.name, yaml(&s.examples))?;
        add(dir.join(PROPERTIES_FILE), &s.name, yaml(&s.properties))?;
    }
    for o in &project.observers {
        let dir = layout::observer_dir(&o.name);
        add(dir.join("observer.yaml"), &o.name, yaml(o))?;
        add(dir.join(SPEC_FILE), &o.name, o.spec.clone())?;
        add(dir.join(EXAMPLES_FILE), &o.name, yaml(&o.examples))?;
        add(dir.join(PROPERTIES_FILE), &o.name, yaml(&o.properties))?;
    }
    Ok(files)
}

/// Fichiers qu'un système ou un observateur possède dans son dossier.
const ELEMENT_FILES: [&str; 5] = [
    "system.yaml",
    "observer.yaml",
    SPEC_FILE,
    EXAMPLES_FILE,
    PROPERTIES_FILE,
];

/// Enregistre le projet dans `dir`.
///
/// - Un fichier n'est réécrit que si son contenu change (PRJ-006).
/// - Les fichiers d'éléments qui n'existent plus dans le modèle sont retirés ; les autres fichiers
///   du dossier (skills, `.gitkeep`, notes…) ne sont jamais touchés.
/// - Un projet dont le format est plus récent que celui d'ECS NIDE n'est jamais réécrit (PRJ-009).
pub fn save_project(project: &Project, dir: &Path) -> Result<(), SaveError> {
    if project.manifest.format > FORMAT_VERSION {
        return Err(SaveError::NewerFormat(project.manifest.format));
    }
    let files = render(project)?;
    let io_error = |path: &Path| {
        let path = path.to_path_buf();
        move |source| SaveError::Io { path, source }
    };

    for sub in DIRECTORIES {
        let path = dir.join(sub);
        fs::create_dir_all(&path).map_err(io_error(&path))?;
    }
    for (relative, content) in &files {
        let path = dir.join(relative);
        if fs::read_to_string(&path).is_ok_and(|current| &current == content) {
            continue;
        }
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(io_error(parent))?;
        }
        fs::write(&path, content).map_err(io_error(&path))?;
    }

    // Retrait des fichiers d'éléments disparus du modèle.
    for sub in [
        "components",
        "resources",
        "messages",
        "events",
        "relationships",
        "archetypes",
    ] {
        for entry in read_dir(&dir.join(sub)) {
            let is_yaml = matches!(
                entry.extension().and_then(|e| e.to_str()),
                Some("yaml" | "yml")
            );
            let relative = PathBuf::from(sub).join(entry.file_name().unwrap());
            if entry.is_file() && is_yaml && !files.contains_key(&relative) {
                fs::remove_file(&entry).map_err(io_error(&entry))?;
            }
        }
    }
    for sub in ["systems", "observers"] {
        for entry in read_dir(&dir.join(sub)).into_iter().filter(|p| p.is_dir()) {
            let relative = PathBuf::from(sub).join(entry.file_name().unwrap());
            if files.keys().any(|f| f.starts_with(&relative)) {
                continue;
            }
            for file in ELEMENT_FILES {
                let path = entry.join(file);
                if path.is_file() {
                    fs::remove_file(&path).map_err(io_error(&path))?;
                }
            }
            if read_dir(&entry).is_empty() {
                fs::remove_dir(&entry).map_err(io_error(&entry))?;
            }
        }
    }
    Ok(())
}

fn read_dir(dir: &Path) -> Vec<PathBuf> {
    let mut entries: Vec<PathBuf> = fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| e.path())
        .collect();
    entries.sort();
    entries
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::load::load_project;
    use crate::model::{Access, System};

    #[test]
    fn prj_003_enregistrer_puis_recharger_donne_le_meme_projet() {
        let mut project = Project::new("t");
        let s = project.ensure_system("Movement");
        s.components.insert("Position".into(), Access::Write);
        s.spec = "# Movement\n".into();
        project.ensure_component("Position");
        let tmp = tempfile::tempdir().unwrap();
        save_project(&project, tmp.path()).unwrap();
        let loaded = load_project(tmp.path()).unwrap();
        assert!(loaded.diagnostics.is_empty(), "{:#?}", loaded.diagnostics);
        assert_eq!(loaded.project, project);
    }

    #[test]
    fn un_element_retire_du_modele_perd_ses_fichiers() {
        let mut project = Project::new("t");
        project.ensure_system("Old");
        project.ensure_component("Legacy");
        let tmp = tempfile::tempdir().unwrap();
        save_project(&project, tmp.path()).unwrap();
        fs::write(tmp.path().join("systems/old/notes.txt"), "à garder").unwrap();
        project.systems.clear();
        project.schedule.order.clear();
        project.components.clear();
        save_project(&project, tmp.path()).unwrap();
        assert!(!tmp.path().join("components/legacy.yaml").exists());
        assert!(!tmp.path().join("systems/old/system.yaml").exists());
        assert!(
            tmp.path().join("systems/old/notes.txt").exists(),
            "un fichier inconnu n'est jamais supprimé"
        );
    }

    #[test]
    fn deux_elements_ne_partagent_jamais_un_fichier() {
        let mut project = Project::new("t");
        project.systems.push(System::new("EnemyAI"));
        project.systems.push(System::new("Enemy-AI"));
        assert!(matches!(
            render(&project),
            Err(SaveError::PathCollision { .. })
        ));
    }

    #[test]
    fn prj_009_un_format_plus_recent_n_est_jamais_reecrit() {
        let mut project = Project::new("t");
        project.manifest.format = FORMAT_VERSION + 1;
        let tmp = tempfile::tempdir().unwrap();
        assert!(matches!(
            save_project(&project, tmp.path()),
            Err(SaveError::NewerFormat(_))
        ));
        assert!(fs::read_dir(tmp.path()).unwrap().next().is_none());
    }
}
