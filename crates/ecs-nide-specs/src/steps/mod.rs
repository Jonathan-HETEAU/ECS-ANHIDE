//! Les fonctions des étapes, regroupées par domaine.

mod cli;
mod contrats;
mod modelisation;
mod ordonnancement;
mod projet;
mod validation;

use spec_markdown::Registry;

use crate::World;

/// Le registre de toutes les étapes connues.
pub fn registry() -> Registry<World> {
    let mut registry = Registry::new();
    projet::register(&mut registry);
    contrats::register(&mut registry);
    cli::register(&mut registry);
    modelisation::register(&mut registry);
    ordonnancement::register(&mut registry);
    validation::register(&mut registry);
    registry
}

/// Découpe une liste séparée par des virgules (cellule de tableau).
pub(crate) fn list(cell: &str) -> Vec<String> {
    cell.split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(String::from)
        .collect()
}

/// Échoue avec `message` si `condition` est fausse.
pub(crate) fn ensure(condition: bool, message: impl FnOnce() -> String) -> Result<(), String> {
    if condition { Ok(()) } else { Err(message()) }
}
