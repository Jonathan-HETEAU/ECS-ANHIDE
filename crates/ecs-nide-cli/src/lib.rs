//! La ligne de commande `ecs-nide` (MCP-009 à MCP-013).
//!
//! Chaque commande appelle l'opération correspondante de [`ecs_nide_core::ops::Session`], la même
//! que celle de l'IDE et du serveur MCP (MCP-010), puis présente son résultat : en texte (YAML pour
//! un plan ou un contrat) ou en JSON avec `--format json` (MCP-012).
//!
//! Codes de sortie : `0` succès, `1` échec de l'opération (dont une validation en erreur,
//! MCP-011), `2` mauvaise utilisation.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use clap::{Parser, Subcommand, ValueEnum};
use ecs_nide_core::create::create_project;
use ecs_nide_core::diagnostic::Level;
use ecs_nide_core::ops::{DEFAULT_TEST_COUNT, OpError, Session};
use ecs_nide_core::reports::{ImportedResults, TestStatus};
use serde::Serialize;

#[derive(Debug, Parser)]
#[command(
    name = "ecs-nide",
    version,
    about = "ECS NIDE en ligne de commande : valider un projet, calculer son plan, obtenir un contrat, générer des données de test et importer des résultats."
)]
struct Cli {
    /// Dossier du projet (par défaut, le dossier courant).
    #[arg(long, global = true, value_name = "DOSSIER")]
    project: Option<PathBuf>,

    /// Format de sortie : texte lisible (YAML pour un plan ou un contrat) ou JSON.
    #[arg(long, global = true, value_enum, default_value_t = Format::Text)]
    format: Format,

    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum Format {
    Text,
    Json,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Crée un projet dans un dossier vide ou inexistant.
    New {
        /// Dossier du nouveau projet.
        dir: PathBuf,
        /// Nom du projet (par défaut, le nom du dossier).
        #[arg(long)]
        name: Option<String>,
        /// Langue des fichiers générés : en (par défaut) ou fr.
        #[arg(long)]
        language: Option<String>,
    },
    /// Valide le projet ; se termine en erreur s'il reste au moins une erreur.
    Validate,
    /// Calcule le plan d'exécution et l'écrit dans schedule.plan.yaml.
    Plan,
    /// Affiche le contrat d'un système ou d'un observateur.
    Contract {
        /// Nom du système ou de l'observateur.
        name: String,
    },
    /// Génère des données de test pour les propriétés d'un système.
    GenTests {
        /// Nom du système ou de l'observateur.
        system: String,
        /// Une seule propriété (par défaut, toutes).
        #[arg(long)]
        property: Option<String>,
        /// Nombre d'entrées par propriété.
        #[arg(long, default_value_t = DEFAULT_TEST_COUNT)]
        count: usize,
        /// Graine : la même graine donne les mêmes données.
        #[arg(long, default_value_t = 0)]
        seed: u64,
        /// Fichier où écrire les données (par défaut, la sortie standard).
        #[arg(long, value_name = "FICHIER")]
        output: Option<PathBuf>,
    },
    /// Importe un rapport de tests CTRF ou JUnit XML.
    ImportResults {
        /// Fichier du rapport.
        report: PathBuf,
    },
}

/// Exécute la CLI avec ces arguments (le premier est le nom du programme), dans le dossier `cwd`.
/// Renvoie le code de sortie.
pub fn run<I, S>(args: I, cwd: &Path, out: &mut dyn Write, err: &mut dyn Write) -> i32
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let args: Vec<String> = args.into_iter().map(Into::into).collect();
    let cli = match Cli::try_parse_from(&args) {
        Ok(cli) => cli,
        Err(e) => {
            let target: &mut dyn Write = if e.use_stderr() { err } else { out };
            let _ = write!(target, "{}", e.render());
            return e.exit_code();
        }
    };
    match execute(cli, cwd, out) {
        Ok(code) => code,
        Err(message) => {
            let _ = writeln!(err, "erreur : {message}");
            1
        }
    }
}

fn resolve(cwd: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        cwd.join(path)
    }
}

fn json<T: Serialize>(out: &mut dyn Write, value: &T) -> Result<(), String> {
    let text = serde_json::to_string_pretty(value).map_err(|e| e.to_string())?;
    writeln!(out, "{text}").map_err(|e| e.to_string())
}

fn yaml<T: Serialize>(out: &mut dyn Write, value: &T) -> Result<(), String> {
    let text = serde_yaml::to_string(value).map_err(|e| e.to_string())?;
    write!(out, "{text}").map_err(|e| e.to_string())
}

fn execute(cli: Cli, cwd: &Path, out: &mut dyn Write) -> Result<i32, String> {
    let open = || -> Result<Session, String> {
        let dir = resolve(cwd, cli.project.as_deref().unwrap_or(Path::new(".")));
        Session::open(&dir).map_err(|e| e.to_string())
    };
    let w = |r: std::io::Result<()>| r.map_err(|e| e.to_string());
    match cli.command {
        Command::New {
            dir,
            name,
            language,
        } => {
            let dir = resolve(cwd, &dir);
            let name = name.unwrap_or_else(|| {
                dir.file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_else(|| "projet".into())
            });
            let project =
                create_project(&dir, &name, language.as_deref()).map_err(|e| e.to_string())?;
            match cli.format {
                Format::Json => json(
                    out,
                    &serde_json::json!({ "name": project.manifest.name, "path": dir }),
                )?,
                Format::Text => w(writeln!(
                    out,
                    "Projet « {} » créé dans {}",
                    project.manifest.name,
                    dir.display()
                ))?,
            }
            Ok(0)
        }
        Command::Validate => {
            let session = open()?;
            let diagnostics = session.diagnostics();
            match cli.format {
                Format::Json => json(out, &diagnostics)?,
                Format::Text => {
                    for d in &diagnostics {
                        w(writeln!(out, "{d}"))?;
                    }
                    let errors = diagnostics
                        .iter()
                        .filter(|d| d.level == Level::Error)
                        .count();
                    let warnings = diagnostics.len() - errors;
                    if diagnostics.is_empty() {
                        w(writeln!(out, "Aucun diagnostic."))?;
                    } else {
                        w(writeln!(
                            out,
                            "{errors} erreur(s), {warnings} avertissement(s)."
                        ))?;
                    }
                }
            }
            Ok(if Session::has_errors(&diagnostics) {
                1
            } else {
                0
            })
        }
        Command::Plan => {
            let plan = open()?.plan().map_err(|e| e.to_string())?;
            match cli.format {
                Format::Json => json(out, &plan)?,
                Format::Text => yaml(out, &plan)?,
            }
            Ok(0)
        }
        Command::Contract { name } => {
            let contract = open()?.contract(&name).map_err(|e| e.to_string())?;
            match cli.format {
                Format::Json => json(out, &contract)?,
                Format::Text => yaml(out, &contract)?,
            }
            Ok(0)
        }
        Command::GenTests {
            system,
            property,
            count,
            seed,
            output,
        } => {
            let datasets = open()?
                .gen_tests(&system, property.as_deref(), count, seed)
                .map_err(|e: OpError| e.to_string())?;
            match output {
                Some(file) => {
                    let file = resolve(cwd, &file);
                    let text =
                        serde_json::to_string_pretty(&datasets).map_err(|e| e.to_string())?;
                    fs::write(&file, text + "\n")
                        .map_err(|e| format!("écriture impossible de {} : {e}", file.display()))?;
                    if cli.format == Format::Text {
                        w(writeln!(
                            out,
                            "{} jeu(x) de données écrit(s) dans {}",
                            datasets.len(),
                            file.display()
                        ))?;
                    }
                }
                None => json(out, &datasets)?,
            }
            Ok(0)
        }
        Command::ImportResults { report } => {
            let path = resolve(cwd, &report);
            let text = fs::read_to_string(&path)
                .map_err(|e| format!("lecture impossible de {} : {e}", path.display()))?;
            let imported = open()?.import_results(&text).map_err(|e| e.to_string())?;
            match cli.format {
                Format::Json => json(out, &imported)?,
                Format::Text => print_results(out, &imported)?,
            }
            Ok(0)
        }
    }
}

fn print_results(out: &mut dyn Write, imported: &ImportedResults) -> Result<(), String> {
    let w = |r: std::io::Result<()>| r.map_err(|e| e.to_string());
    let mut systems: Vec<&str> = imported.results.iter().map(|r| r.system.as_str()).collect();
    systems.dedup();
    systems.sort_unstable();
    systems.dedup();
    for system in systems {
        let (passed, failed) = imported.summary(system);
        w(writeln!(
            out,
            "{system} : {passed} réussi(s), {failed} échoué(s)"
        ))?;
        for result in imported
            .results
            .iter()
            .filter(|r| r.system == system && r.status == TestStatus::Failed)
        {
            w(writeln!(out, "  échec : {}", result.name))?;
            if let Some(counterexample) = &result.counterexample {
                w(writeln!(out, "    contre-exemple : {counterexample}"))?;
            }
        }
    }
    for name in &imported.unmatched {
        w(writeln!(out, "sans correspondance : {name}"))?;
    }
    Ok(())
}
