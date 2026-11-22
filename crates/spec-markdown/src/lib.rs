//! Spec Markdown : des scénarios lisibles par tous et exécutables comme des tests.
//!
//! Le format est décrit dans `features/README.md`. Ce crate fournit :
//! - [`parse`] : la lecture d'un fichier `*.spec.md` en [`Spec`] ;
//! - [`Registry`] : l'association des phrases d'étapes à des fonctions, et l'exécution des scénarios.

mod parser;
mod runner;

pub use parser::{ParseError, Scenario, Spec, Step, Table, parse};
pub use runner::{Outcome, Registry, StepFn, UndefinedStep};
