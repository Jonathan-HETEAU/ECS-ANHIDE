use std::fs;
use std::path::Path;

use ecs_nide_core::load::load_project;
use ecs_nide_core::schedule::{Plan, PlanError, compute_plan};
use ecs_nide_core::{EngineProfile, Level, Project};

use crate::repo;

/// L'état partagé par les étapes d'un scénario. Chaque scénario part d'un monde neuf.
pub struct World {
    pub project: Project,
    pub profile: EngineProfile,
    /// Copie temporaire du dossier projet, quand le scénario ouvre un projet d'exemple.
    pub project_dir: Option<tempfile::TempDir>,
    pub plan: Option<Result<Plan, PlanError>>,
    pub plans: Vec<Plan>,
}

impl Default for World {
    fn default() -> Self {
        Self {
            project: Project::new("scenario"),
            profile: EngineProfile::generic(),
            project_dir: None,
            plan: None,
            plans: Vec::new(),
        }
    }
}

impl World {
    /// Copie le projet d'exemple dans un dossier temporaire et le charge.
    pub fn open_example(&mut self, name: &str) -> Result<(), String> {
        let source = repo::root().join("examples").join(name);
        let dir = tempfile::tempdir().map_err(|e| e.to_string())?;
        copy_dir(&source, dir.path()).map_err(|e| format!("copie de {name} impossible : {e}"))?;
        let loaded = load_project(dir.path()).map_err(|e| e.to_string())?;
        let errors: Vec<String> = loaded
            .diagnostics
            .iter()
            .filter(|d| d.level == Level::Error)
            .map(ToString::to_string)
            .collect();
        if !errors.is_empty() {
            return Err(format!(
                "le projet {name} contient des erreurs :\n{}",
                errors.join("\n")
            ));
        }
        self.project = loaded.project;
        self.project_dir = Some(dir);
        Ok(())
    }

    /// Calcule le plan ; pour un projet ouvert depuis un dossier, l'écrit aussi (ORD-011).
    pub fn compute_plan(&mut self) -> Result<(), String> {
        let result = compute_plan(&self.project, &self.profile);
        if let (Ok(plan), Some(dir)) = (&result, &self.project_dir) {
            plan.write(dir.path()).map_err(|e| e.to_string())?;
        }
        self.plan = Some(result);
        Ok(())
    }

    pub fn plan(&self) -> Result<&Plan, String> {
        match &self.plan {
            Some(Ok(plan)) => Ok(plan),
            Some(Err(err)) => Err(format!("aucun plan n'a été produit : {err}")),
            None => Err("le plan n'a pas été calculé".to_string()),
        }
    }

    pub fn stage_of(&self, system: &str) -> Result<usize, String> {
        self.plan()?
            .stage_of(system)
            .ok_or_else(|| format!("« {system} » ne figure dans aucune étape"))
    }
}

fn copy_dir(from: &Path, to: &Path) -> std::io::Result<()> {
    fs::create_dir_all(to)?;
    for entry in fs::read_dir(from)? {
        let entry = entry?;
        let target = to.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir(&entry.path(), &target)?;
        } else {
            fs::copy(entry.path(), target)?;
        }
    }
    Ok(())
}
