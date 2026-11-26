//! Les opérations sur un projet, offertes à l'identique à l'IDE, à la CLI et au serveur MCP
//! (MCP-010). Chacun ouvre une [`Session`] et en présente les résultats à sa façon.

use std::path::{Path, PathBuf};

use crate::contract::{Contract, contract};
use crate::diagnostic::{Diagnostic, Level};
use crate::generate::{Dataset, GenerateError, generate};
use crate::load::{LoadError, Loaded, load_project};
use crate::model::Project;
use crate::profile::EngineProfile;
use crate::reports::{ImportError, ImportedResults, import_report};
use crate::schedule::{Plan, PlanError, compute_plan};
use crate::validate::validate;

/// Nombre d'entrées générées par défaut pour une propriété.
pub const DEFAULT_TEST_COUNT: usize = 100;

#[derive(Debug, thiserror::Error)]
pub enum OpError {
    #[error(transparent)]
    Load(#[from] LoadError),
    #[error("système ou observateur inconnu : {0}")]
    UnknownActor(String),
    #[error(transparent)]
    Plan(#[from] PlanError),
    #[error(transparent)]
    Generate(#[from] GenerateError),
    #[error(transparent)]
    Import(#[from] ImportError),
    #[error("écriture impossible de {path} : {source}")]
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
}

/// Un projet ouvert depuis son dossier, avec le profil de son moteur cible.
#[derive(Debug, Clone)]
pub struct Session {
    pub dir: PathBuf,
    pub loaded: Loaded,
    pub profile: EngineProfile,
}

impl Session {
    pub fn open(dir: &Path) -> Result<Self, OpError> {
        let loaded = load_project(dir)?;
        let profile = EngineProfile::of_project(&loaded.project);
        Ok(Self {
            dir: dir.to_path_buf(),
            loaded,
            profile,
        })
    }

    pub fn project(&self) -> &Project {
        &self.loaded.project
    }

    /// Les diagnostics du chargement, puis de la validation (exigences VAL, PRJ-005, PRJ-009).
    pub fn diagnostics(&self) -> Vec<Diagnostic> {
        let mut all = self.loaded.diagnostics.clone();
        all.extend(validate(self.project(), &self.profile));
        all
    }

    pub fn has_errors(diagnostics: &[Diagnostic]) -> bool {
        diagnostics.iter().any(|d| d.level == Level::Error)
    }

    /// Calcule le plan et l'écrit dans `schedule.plan.yaml` (ORD-011), sauf pour un projet ouvert
    /// en lecture seule (PRJ-009).
    pub fn plan(&self) -> Result<Plan, OpError> {
        let plan = compute_plan(self.project(), &self.profile)?;
        if !self.loaded.read_only {
            plan.write(&self.dir).map_err(|source| OpError::Io {
                path: self.dir.join(crate::schedule::PLAN_FILE),
                source,
            })?;
        }
        Ok(plan)
    }

    /// Le contrat d'un système ou d'un observateur (TST-013, TST-016).
    pub fn contract(&self, name: &str) -> Result<Contract, OpError> {
        contract(self.project(), name, &self.profile)
            .ok_or_else(|| OpError::UnknownActor(name.to_string()))
    }

    /// Génère des données de test pour une propriété, ou pour toutes les propriétés du système
    /// (TST-007 à TST-010).
    pub fn gen_tests(
        &self,
        actor: &str,
        property: Option<&str>,
        count: usize,
        seed: u64,
    ) -> Result<Vec<Dataset>, OpError> {
        let project = self.project();
        let properties = project
            .system(actor)
            .map(|s| &s.properties)
            .or_else(|| project.observer(actor).map(|o| &o.properties))
            .ok_or_else(|| OpError::UnknownActor(actor.to_string()))?;
        let ids: Vec<String> = match property {
            Some(id) => vec![id.to_string()],
            None => properties
                .iter()
                .filter_map(|p| p["id"].as_str().map(String::from))
                .collect(),
        };
        ids.iter()
            .map(|id| generate(project, actor, id, count, seed).map_err(OpError::from))
            .collect()
    }

    /// Importe un rapport de tests CTRF ou JUnit XML (TST-011, TST-014, TST-015).
    pub fn import_results(&self, report: &str) -> Result<ImportedResults, OpError> {
        Ok(import_report(self.project(), report)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn example() -> Session {
        let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/space-shooter");
        let tmp = tempfile::tempdir().unwrap().keep();
        for entry in walk(&source) {
            let target = tmp.join(entry.strip_prefix(&source).unwrap());
            std::fs::create_dir_all(target.parent().unwrap()).unwrap();
            std::fs::copy(&entry, target).unwrap();
        }
        Session::open(&tmp).unwrap()
    }

    fn walk(dir: &Path) -> Vec<PathBuf> {
        let mut files = Vec::new();
        for entry in std::fs::read_dir(dir).unwrap().flatten() {
            let path = entry.path();
            if path.is_dir() {
                files.extend(walk(&path));
            } else {
                files.push(path);
            }
        }
        files
    }

    #[test]
    fn mcp_010_session_sur_le_projet_d_exemple() {
        let session = example();
        assert!(session.diagnostics().is_empty());
        let plan = session.plan().unwrap();
        assert!(
            session.dir.join(crate::schedule::PLAN_FILE).is_file(),
            "ORD-011"
        );
        assert_eq!(plan.stages.len(), 7);
        assert_eq!(session.contract("Damage").unwrap().name, "Damage");
        assert!(matches!(
            session.contract("Nope"),
            Err(OpError::UnknownActor(_))
        ));
        let datasets = session.gen_tests("Damage", None, 10, 42).unwrap();
        assert_eq!(
            datasets.len(),
            2,
            "une jeu de données par propriété de Damage"
        );
        assert_eq!(
            session
                .gen_tests("Damage", Some("sante-positive"), 10, 42)
                .unwrap()
                .len(),
            1
        );
    }
}
