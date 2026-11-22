//! Cœur d'ECS NIDE.
//!
//! Toute la logique métier vit ici et sert à l'identique l'IDE, la CLI et le serveur MCP :
//! - [`model`] : les concepts du modèle déclaratif ;
//! - [`load`] : la lecture d'un dossier projet ;
//! - [`profile`] : les capacités du moteur cible ;
//! - [`schedule`] : le calcul du plan d'exécution.

pub mod diagnostic;
pub mod load;
pub mod model;
pub mod profile;
pub mod schedule;

pub use diagnostic::{Diagnostic, Level};
pub use model::Project;
pub use profile::EngineProfile;
