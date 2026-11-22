//! Lecture d'un dossier projet (PRJ-005, PRJ-009, VAL-011).

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use serde::de::DeserializeOwned;
use serde_yaml::Value;

use crate::diagnostic::Diagnostic;
use crate::model::{
    Archetype, Component, Event, FORMAT_VERSION, Manifest, MessageChannel, Observer, Project,
    Relationship, Resource, Schedule, System,
};

pub const MANIFEST_FILE: &str = "ecs-nide.yaml";
pub const SCHEDULE_FILE: &str = "schedule.yaml";
pub const SPEC_FILE: &str = "spec.md";
pub const EXAMPLES_FILE: &str = "examples.yaml";
pub const PROPERTIES_FILE: &str = "properties.yaml";

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
                 il est ouvert en lecture seule et ne sera jamais réécrit",
                manifest.format, FORMAT_VERSION
            ))
            .code("PRJ-009")
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
            let tests = load_tests(dir, &sub, &mut diagnostics);
            system.spec = tests.spec;
            system.examples = tests.examples;
            system.properties = tests.properties;
            project.systems.push(system);
        }
    }
    for sub in subdirectories(&dir.join("observers")) {
        if let Some(mut observer) =
            parse_required::<Observer>(dir, &sub, "observer.yaml", &mut diagnostics)
        {
            let tests = load_tests(dir, &sub, &mut diagnostics);
            observer.spec = tests.spec;
            observer.examples = tests.examples;
            observer.properties = tests.properties;
            project.observers.push(observer);
        }
    }

    Ok(Loaded {
        project,
        diagnostics,
        read_only,
    })
}

struct Tests {
    spec: String,
    examples: Vec<Value>,
    properties: Vec<Value>,
}

/// Lit la spécification, les exemples et les propriétés d'un système ou d'un observateur.
fn load_tests(root: &Path, dir: &Path, diagnostics: &mut Vec<Diagnostic>) -> Tests {
    let list = |file: &str, diagnostics: &mut Vec<Diagnostic>| -> Vec<Value> {
        let path = dir.join(file);
        if path.is_file() {
            parse_file::<Vec<Value>>(root, &path, diagnostics).unwrap_or_default()
        } else {
            Vec::new()
        }
    };
    Tests {
        spec: fs::read_to_string(dir.join(SPEC_FILE)).unwrap_or_default(),
        examples: list(EXAMPLES_FILE, diagnostics),
        properties: list(PROPERTIES_FILE, diagnostics),
    }
}

fn relative(root: &Path, path: &Path) -> PathBuf {
    path.strip_prefix(root).unwrap_or(path).to_path_buf()
}

/// Analyse un texte YAML en `T`. Une clé déclarée deux fois dans un même bloc est une erreur
/// (VAL-011) : sans cette vérification, la dernière déclaration écraserait silencieusement la
/// première.
pub fn parse_yaml<T: DeserializeOwned>(text: &str, file: &Path) -> Result<T, Diagnostic> {
    if let Err(err) = serde_yaml::from_str::<Value>(text) {
        let message = err.to_string();
        if message.contains("duplicate entry") {
            let (key, line) = find_duplicate_key(text)
                .unwrap_or_else(|| (String::new(), err.location().map_or(0, |l| l.line())));
            return Err(Diagnostic::error(format!(
                "la clé « {key} » est déclarée deux fois dans le même bloc"
            ))
            .code("VAL-011")
            .at(file, (line > 0).then_some(line)));
        }
        return Err(syntax_error(&err, file));
    }
    serde_yaml::from_str::<T>(text).map_err(|err| syntax_error(&err, file))
}

fn syntax_error(err: &serde_yaml::Error, file: &Path) -> Diagnostic {
    Diagnostic::error(format!("fichier invalide : {err}"))
        .code("PRJ-005")
        .at(file, err.location().map(|l| l.line()))
}

/// Trouve la première clé déclarée deux fois dans un même bloc YAML, avec la ligne (à partir de
/// 1) de sa seconde déclaration. Seuls les blocs en style « bloc » sont examinés.
pub fn find_duplicate_key(text: &str) -> Option<(String, usize)> {
    // Pile des blocs ouverts : (indentation, clés déjà vues).
    let mut stack: Vec<(usize, HashSet<String>)> = Vec::new();
    for (index, raw) in text.lines().enumerate() {
        let trimmed = raw.trim_start();
        if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with("---") {
            continue;
        }
        let mut indent = raw.len() - trimmed.len();
        let mut content = trimmed;
        if let Some(rest) =
            content
                .strip_prefix("- ")
                .or(if content == "-" { Some("") } else { None })
        {
            // Un élément de liste ouvre un nouveau bloc, plus indenté de deux colonnes.
            while stack.last().is_some_and(|(i, _)| *i > indent) {
                stack.pop();
            }
            indent += 2;
            content = rest.trim_start();
            if let Some(key) = mapping_key(content) {
                stack.push((indent, HashSet::from([key])));
            }
            continue;
        }
        let Some(key) = mapping_key(content) else {
            continue;
        };
        while stack.last().is_some_and(|(i, _)| *i > indent) {
            stack.pop();
        }
        match stack.last_mut() {
            Some((i, keys)) if *i == indent => {
                if !keys.insert(key.clone()) {
                    return Some((key, index + 1));
                }
            }
            _ => stack.push((indent, HashSet::from([key]))),
        }
    }
    None
}

fn mapping_key(content: &str) -> Option<String> {
    if content.starts_with(['{', '[', '"', '\'', '|', '>']) {
        return None;
    }
    let end = content
        .find(": ")
        .or_else(|| content.strip_suffix(':').map(str::len))?;
    let key = content[..end].trim();
    (!key.is_empty()).then(|| key.to_string())
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
    match parse_yaml(&text, &rel) {
        Ok(value) => Some(value),
        Err(diagnostic) => {
            diagnostics.push(diagnostic);
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
    fn val_011_detection_des_cles_en_double() {
        assert_eq!(
            find_duplicate_key("a: 1\nb: 2\na: 3\n"),
            Some(("a".into(), 3))
        );
        assert_eq!(find_duplicate_key("x:\n  a: 1\ny:\n  a: 2\n"), None);
        assert_eq!(
            find_duplicate_key("- id: a\n  v: 1\n- id: b\n  v: 2\n"),
            None
        );
        assert_eq!(
            find_duplicate_key("- id: a\n  v: 1\n  v: 2\n"),
            Some(("v".into(), 3))
        );
        assert_eq!(
            find_duplicate_key(
                "components:\n  Position: read\nresources:\n  Time: read\n  Time: write\n"
            ),
            Some(("Time".into(), 5))
        );
        assert_eq!(find_duplicate_key("a:\n- x\n- y\nb: 1\n"), None);
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
    fn val_011_une_cle_en_double_est_signalee_et_non_ecrasee() {
        let tmp = tempfile::tempdir().unwrap();
        copy_dir(&space_shooter(), tmp.path());
        fs::write(
            tmp.path().join("systems/movement/system.yaml"),
            "name: Movement\ncomponents:\n  Position: read\n  Velocity: read\n  Position: write\n",
        )
        .unwrap();
        let loaded = load_project(tmp.path()).unwrap();
        let duplicate = loaded
            .diagnostics
            .iter()
            .find(|d| d.code.as_deref() == Some("VAL-011"))
            .unwrap_or_else(|| panic!("aucune erreur VAL-011 : {:#?}", loaded.diagnostics));
        assert_eq!(duplicate.level, Level::Error);
        assert!(
            duplicate.message.contains("Position"),
            "{}",
            duplicate.message
        );
        assert_eq!(
            duplicate.file.as_deref(),
            Some(Path::new("systems/movement/system.yaml"))
        );
        assert_eq!(duplicate.line, Some(5));
        assert!(
            loaded.project.system("Movement").is_none(),
            "le fichier invalide ne doit pas être retenu"
        );
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
