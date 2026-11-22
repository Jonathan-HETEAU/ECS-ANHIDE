//! Création d'un projet (PRJ-001, PRJ-002, PRJ-010, MCP-008).

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::model::Project;
use crate::save::{SaveError, save_project};

pub const AGENTS_FILE: &str = "AGENTS.md";
pub const VSCODE_MCP_FILE: &str = ".vscode/mcp.json";

/// Langues dans lesquelles ECS NIDE sait rédiger les fichiers qu'il génère (PRJ-010).
pub const LANGUAGES: [&str; 2] = ["en", "fr"];
pub const DEFAULT_LANGUAGE: &str = "en";

const AGENTS_EN: &str = include_str!("../templates/AGENTS.en.md");
const AGENTS_FR: &str = include_str!("../templates/AGENTS.fr.md");

/// Configuration de VS Code qui démarre le serveur MCP sur le projet (MCP-008).
const VSCODE_MCP: &str = r#"{
  "servers": {
    "ecs-nide": {
      "type": "stdio",
      "command": "ecs-nide",
      "args": ["mcp", "--project", "${workspaceFolder}"]
    }
  }
}
"#;

#[derive(Debug, thiserror::Error)]
pub enum CreateError {
    #[error("le dossier {0} n'est pas vide")]
    NotEmpty(PathBuf),
    #[error("langue non prise en charge : {0} (langues disponibles : en, fr)")]
    UnsupportedLanguage(String),
    #[error(transparent)]
    Save(#[from] SaveError),
    #[error("écriture impossible de {path} : {source}")]
    Io { path: PathBuf, source: io::Error },
}

/// Le fichier `AGENTS.md` d'un projet, dans sa langue (PRJ-002, PRJ-010).
pub fn agents_file(project: &Project) -> Result<String, CreateError> {
    let template = match project.manifest.language.as_str() {
        "en" => AGENTS_EN,
        "fr" => AGENTS_FR,
        other => return Err(CreateError::UnsupportedLanguage(other.to_string())),
    };
    Ok(template.replace("{{name}}", &project.manifest.name))
}

/// Crée un projet vide dans `dir`, qui doit être vide ou ne pas exister.
pub fn create_project(
    dir: &Path,
    name: &str,
    language: Option<&str>,
) -> Result<Project, CreateError> {
    let language = language.unwrap_or(DEFAULT_LANGUAGE);
    if !LANGUAGES.contains(&language) {
        return Err(CreateError::UnsupportedLanguage(language.to_string()));
    }
    let not_empty = fs::read_dir(dir).is_ok_and(|mut entries| entries.next().is_some());
    if not_empty {
        return Err(CreateError::NotEmpty(dir.to_path_buf()));
    }
    let mut project = Project::new(name);
    project.manifest.language = language.to_string();
    save_project(&project, dir)?;

    let write = |relative: &str, content: &str| -> Result<(), CreateError> {
        let path = dir.join(relative);
        let io_error = |source| CreateError::Io {
            path: path.clone(),
            source,
        };
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(io_error)?;
        }
        fs::write(&path, content).map_err(io_error)
    };
    write(AGENTS_FILE, &agents_file(&project)?)?;
    write(VSCODE_MCP_FILE, VSCODE_MCP)?;
    Ok(project)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::load::load_project;

    #[test]
    fn prj_001_creer_un_projet() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("asteroides");
        create_project(&dir, "Asteroides", None).unwrap();
        for file in [
            "ecs-nide.yaml",
            "schedule.yaml",
            AGENTS_FILE,
            VSCODE_MCP_FILE,
        ] {
            assert!(dir.join(file).is_file(), "{file} absent");
        }
        let loaded = load_project(&dir).unwrap();
        assert!(loaded.diagnostics.is_empty(), "{:#?}", loaded.diagnostics);
        assert_eq!(loaded.project.manifest.language, "en");
    }

    #[test]
    fn prj_001_un_dossier_non_vide_est_refuse() {
        let tmp = tempfile::tempdir().unwrap();
        fs::write(tmp.path().join("notes.txt"), "").unwrap();
        assert!(matches!(
            create_project(tmp.path(), "x", None),
            Err(CreateError::NotEmpty(_))
        ));
    }

    #[test]
    fn prj_010_une_langue_inconnue_est_refusee() {
        let tmp = tempfile::tempdir().unwrap();
        assert!(matches!(
            create_project(&tmp.path().join("p"), "x", Some("tlh")),
            Err(CreateError::UnsupportedLanguage(_))
        ));
    }

    #[test]
    fn mcp_008_la_configuration_vscode_est_du_json_valide() {
        let value: serde_yaml::Value = serde_yaml::from_str(VSCODE_MCP).unwrap();
        assert_eq!(
            value["servers"]["ecs-nide"]["command"].as_str(),
            Some("ecs-nide")
        );
    }
}
