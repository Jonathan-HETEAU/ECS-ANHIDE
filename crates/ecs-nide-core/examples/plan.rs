//! Affiche le plan d'exécution d'un dossier projet : `cargo run -p ecs-nide-core --example plan -- <dossier>`.
use std::path::PathBuf;

use ecs_nide_core::EngineProfile;
use ecs_nide_core::load::load_project;
use ecs_nide_core::schedule::compute_plan;

fn main() {
    let dir: PathBuf = std::env::args()
        .nth(1)
        .expect("usage : plan <dossier projet>")
        .into();
    let loaded = load_project(&dir).unwrap_or_else(|e| panic!("{e}"));
    for diagnostic in &loaded.diagnostics {
        eprintln!("{diagnostic}");
    }
    match compute_plan(&loaded.project, &EngineProfile::generic()) {
        Ok(plan) => print!("{}", plan.to_yaml()),
        Err(err) => {
            eprintln!("{err}");
            std::process::exit(1);
        }
    }
}
