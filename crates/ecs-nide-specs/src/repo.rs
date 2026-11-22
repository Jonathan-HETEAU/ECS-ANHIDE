use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use spec_markdown::Spec;

/// Racine du dépôt.
pub fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("racine du dépôt introuvable")
}

fn sorted_files(dir: &Path, suffix: &str) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("lecture de {} impossible : {e}", dir.display()))
        .flatten()
        .map(|entry| entry.path())
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.ends_with(suffix))
        })
        .collect();
    files.sort();
    files
}

/// Toutes les spécifications de `features/`, par nom de fichier.
///
/// # Panics
/// Si un fichier ne respecte pas le format Spec Markdown.
pub fn features() -> Vec<(String, Spec)> {
    sorted_files(&root().join("features"), ".spec.md")
        .into_iter()
        .map(|path| {
            let name = path.file_name().unwrap().to_string_lossy().into_owned();
            let source = fs::read_to_string(&path).unwrap();
            let spec = spec_markdown::parse(&source)
                .unwrap_or_else(|e| panic!("features/{name} est invalide, {e}"));
            (name, spec)
        })
        .collect()
}

/// Une exigence EARS définie dans `exigences/`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Requirement {
    pub id: String,
    pub file: String,
    pub line: usize,
}

/// Toutes les exigences, dans l'ordre des fichiers puis des lignes.
pub fn requirements() -> Vec<Requirement> {
    let mut result = Vec::new();
    for path in sorted_files(&root().join("exigences"), ".md") {
        let file = path.file_name().unwrap().to_string_lossy().into_owned();
        if file == "README.md" {
            continue;
        }
        let source = fs::read_to_string(&path).unwrap();
        for (index, line) in source.lines().enumerate() {
            if let Some(id) = requirement_id(line) {
                result.push(Requirement {
                    id,
                    file: file.clone(),
                    line: index + 1,
                });
            }
        }
    }
    result
}

/// L'identifiant d'une ligne de définition d'exigence : `**ORD-001** — …`.
fn requirement_id(line: &str) -> Option<String> {
    let rest = line.strip_prefix("**")?;
    let (id, _) = rest.split_once("**")?;
    let (prefix, number) = id.split_once('-')?;
    let valid = prefix.len() == 3
        && prefix.chars().all(|c| c.is_ascii_uppercase())
        && number.len() == 3
        && number.chars().all(|c| c.is_ascii_digit());
    valid.then(|| id.to_string())
}

/// Les scénarios actifs d'un fichier de `features/`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Selection {
    All,
    Titles(Vec<String>),
}

impl Selection {
    pub fn contains(&self, title: &str) -> bool {
        match self {
            Selection::All => true,
            Selection::Titles(titles) => titles.iter().any(|t| t == title),
        }
    }
}

pub const ACTIVE_FILE: &str = "features/actifs.yaml";

/// Contenu de `features/actifs.yaml` : les scénarios qui doivent passer.
pub fn active() -> BTreeMap<String, Selection> {
    let path = root().join(ACTIVE_FILE);
    let source = fs::read_to_string(&path).unwrap_or_default();
    let value: serde_yaml::Value = serde_yaml::from_str(&source)
        .unwrap_or_else(|e| panic!("{ACTIVE_FILE} est invalide : {e}"));
    let mut result = BTreeMap::new();
    let Some(mapping) = value.as_mapping() else {
        return result;
    };
    for (file, selection) in mapping {
        let file = file
            .as_str()
            .unwrap_or_else(|| panic!("{ACTIVE_FILE} : clé non textuelle"))
            .to_string();
        let selection = match selection {
            serde_yaml::Value::String(s) if s == "tous" => Selection::All,
            serde_yaml::Value::Sequence(items) => Selection::Titles(
                items
                    .iter()
                    .map(|t| {
                        t.as_str()
                            .unwrap_or_else(|| {
                                panic!("{ACTIVE_FILE} : titre non textuel dans {file}")
                            })
                            .to_string()
                    })
                    .collect(),
            ),
            other => panic!(
                "{ACTIVE_FILE} : « {file} » doit valoir « tous » ou une liste de titres, pas {other:?}"
            ),
        };
        result.insert(file, selection);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lecture_des_identifiants() {
        assert_eq!(
            requirement_id("**ORD-001** — ECS NIDE doit…"),
            Some("ORD-001".into())
        );
        assert_eq!(requirement_id("**Composant** | …"), None);
        assert_eq!(requirement_id("**ORD-1** — …"), None);
        assert_eq!(requirement_id("ORD-001 — …"), None);
    }
}
