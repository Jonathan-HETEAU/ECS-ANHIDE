use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use ecs_nide_core::contract::Contract;
use ecs_nide_core::edit::EditError;
use ecs_nide_core::generate::Dataset;
use ecs_nide_core::load::load_project;
use ecs_nide_core::reports::ImportedResults;
use ecs_nide_core::save::save_project;
use ecs_nide_core::schedule::{Plan, PlanError, compute_plan};
use ecs_nide_core::validate::validate;
use ecs_nide_core::{Diagnostic, EngineProfile, Level, Project};

use crate::repo;

/// L'état partagé par les étapes d'un scénario. Chaque scénario part d'un monde neuf.
pub struct World {
    pub project: Project,
    pub profile: EngineProfile,
    /// Dossier temporaire où le scénario crée et enregistre ses projets.
    workspace: tempfile::TempDir,
    /// Dossier du projet courant, quand il a été ouvert, créé ou enregistré.
    pub project_dir: Option<PathBuf>,
    /// Diagnostics du dernier chargement du projet courant.
    pub load_diagnostics: Vec<Diagnostic>,
    pub read_only: bool,
    /// Contenu des fichiers du projet au moment de son ouverture.
    pub snapshot: BTreeMap<PathBuf, Vec<u8>>,
    /// Erreurs de validation au moment de l'ouverture du projet.
    pub baseline_errors: Vec<String>,
    /// Dernier dossier dans lequel le projet a été enregistré.
    pub saved_dir: Option<PathBuf>,
    pub reopened: Option<Project>,
    pub edit_result: Option<Result<(), EditError>>,
    pub plan: Option<Result<Plan, PlanError>>,
    pub plans: Vec<Plan>,
    /// Le système ou l'observateur dont parlent les étapes suivantes.
    pub current_actor: Option<String>,
    /// L'exemple dont parlent les étapes suivantes.
    pub current_example: Option<String>,
    /// La propriété dont parlent les étapes suivantes : (système, identifiant).
    pub current_property: Option<(String, String)>,
    pub contract: Option<Contract>,
    pub datasets: Vec<Dataset>,
    pub exported: Option<PathBuf>,
    pub imported: Option<ImportedResults>,
    /// L'exemple créé à partir d'un contre-exemple : (système, identifiant).
    pub converted: Option<(String, String)>,
    /// Les exécutions de la CLI, dans l'ordre.
    pub cli_runs: Vec<CliRun>,
}

/// Une exécution de la CLI : arguments, dossier courant, code de sortie et sorties.
#[derive(Debug, Clone)]
pub struct CliRun {
    pub args: Vec<String>,
    pub cwd: PathBuf,
    pub code: i32,
    pub stdout: String,
    pub stderr: String,
}

impl Default for World {
    fn default() -> Self {
        Self {
            project: Project::new("scenario"),
            profile: EngineProfile::generic(),
            workspace: tempfile::tempdir().expect("dossier temporaire"),
            project_dir: None,
            load_diagnostics: Vec::new(),
            read_only: false,
            snapshot: BTreeMap::new(),
            baseline_errors: Vec::new(),
            saved_dir: None,
            reopened: None,
            edit_result: None,
            plan: None,
            plans: Vec::new(),
            current_actor: None,
            current_example: None,
            current_property: None,
            contract: None,
            datasets: Vec::new(),
            exported: None,
            imported: None,
            converted: None,
            cli_runs: Vec::new(),
        }
    }
}

impl World {
    /// Exécute une ligne de commande `ecs-nide …` dans le dossier `cwd`.
    pub fn run_cli(&mut self, line: &str, cwd: &Path) -> Result<CliRun, String> {
        let args: Vec<String> = line.split_whitespace().map(String::from).collect();
        if args.first().map(String::as_str) != Some("ecs-nide") {
            return Err(format!("commande inattendue : {line}"));
        }
        let (mut out, mut err) = (Vec::new(), Vec::new());
        let code = ecs_nide_cli::run(args.clone(), cwd, &mut out, &mut err);
        let run = CliRun {
            args,
            cwd: cwd.to_path_buf(),
            code,
            stdout: String::from_utf8_lossy(&out).into_owned(),
            stderr: String::from_utf8_lossy(&err).into_owned(),
        };
        self.cli_runs.push(run.clone());
        Ok(run)
    }

    /// Un sous-dossier neuf de l'espace de travail du scénario.
    pub fn new_dir(&self, name: &str) -> PathBuf {
        let mut path = self.workspace.path().join(name);
        let mut n = 1;
        while path.exists() {
            n += 1;
            path = self.workspace.path().join(format!("{name}-{n}"));
        }
        path
    }

    pub fn dir(&self) -> Result<&Path, String> {
        self.project_dir
            .as_deref()
            .ok_or_else(|| "aucun dossier projet".to_string())
    }

    /// Copie le projet d'exemple dans l'espace de travail et l'ouvre.
    pub fn open_example(&mut self, name: &str) -> Result<(), String> {
        let source = repo::root().join("examples").join(name);
        let dir = self.new_dir(name);
        copy_dir(&source, &dir).map_err(|e| format!("copie de {name} impossible : {e}"))?;
        self.load(&dir)?;
        if self.read_only {
            return Ok(());
        }
        let errors = self.errors();
        if !errors.is_empty() {
            return Err(format!(
                "le projet {name} contient des erreurs :\n{}",
                errors.join("\n")
            ));
        }
        self.snapshot = snapshot(&dir);
        self.baseline_errors = errors;
        Ok(())
    }

    /// Charge le projet contenu dans `dir`, qui devient le projet courant.
    pub fn load(&mut self, dir: &Path) -> Result<(), String> {
        let loaded = load_project(dir).map_err(|e| e.to_string())?;
        self.project = loaded.project;
        self.load_diagnostics = loaded.diagnostics;
        self.read_only = loaded.read_only;
        self.project_dir = Some(dir.to_path_buf());
        Ok(())
    }

    /// Recharge le projet courant depuis son dossier.
    pub fn reload(&mut self) -> Result<(), String> {
        let dir = self.dir()?.to_path_buf();
        self.load(&dir)
    }

    /// Enregistre le projet courant dans son dossier, en lui en créant un s'il n'en a pas.
    pub fn persist(&mut self) -> Result<PathBuf, String> {
        let dir = match &self.project_dir {
            Some(dir) => dir.clone(),
            None => self.new_dir("projet"),
        };
        save_project(&self.project, &dir).map_err(|e| e.to_string())?;
        self.project_dir = Some(dir.clone());
        Ok(dir)
    }

    /// Diagnostics du dernier chargement, puis de la validation du modèle courant.
    pub fn diagnostics(&self) -> Vec<Diagnostic> {
        let mut all = self.load_diagnostics.clone();
        all.extend(validate(&self.project, &self.profile));
        all
    }

    pub fn errors(&self) -> Vec<String> {
        self.diagnostics()
            .into_iter()
            .filter(|d| d.level == Level::Error)
            .map(|d| d.to_string())
            .collect()
    }

    /// Le premier diagnostic qui vérifie `predicate`, ou une erreur qui liste tous les diagnostics.
    pub fn find_diagnostic(
        &self,
        what: &str,
        predicate: impl Fn(&Diagnostic) -> bool,
    ) -> Result<Diagnostic, String> {
        let all = self.diagnostics();
        all.iter().find(|d| predicate(d)).cloned().ok_or_else(|| {
            let list: Vec<String> = all.iter().map(|d| format!("  {d}")).collect();
            format!("aucun diagnostic {what} parmi :\n{}", list.join("\n"))
        })
    }

    /// Calcule le plan ; pour un projet qui a un dossier, l'écrit aussi (ORD-011).
    pub fn compute_plan(&mut self) -> Result<(), String> {
        let result = compute_plan(&self.project, &self.profile);
        if let (Ok(plan), Some(dir)) = (&result, &self.project_dir) {
            plan.write(dir).map_err(|e| e.to_string())?;
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

/// Contenu de tous les fichiers d'un dossier, par chemin relatif.
pub fn snapshot(dir: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    fn walk(root: &Path, dir: &Path, out: &mut BTreeMap<PathBuf, Vec<u8>>) {
        for entry in fs::read_dir(dir).into_iter().flatten().flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk(root, &path, out);
            } else if let Ok(bytes) = fs::read(&path) {
                out.insert(path.strip_prefix(root).unwrap().to_path_buf(), bytes);
            }
        }
    }
    let mut out = BTreeMap::new();
    walk(dir, dir, &mut out);
    out
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
