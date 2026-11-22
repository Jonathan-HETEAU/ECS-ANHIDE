//! Cœur d'ECS NIDE.
//!
//! Toute la logique métier vit ici et sert à l'identique l'IDE, la CLI et le serveur MCP :
//! - [`model`] : les concepts du modèle déclaratif ;
//! - [`layout`], [`load`], [`save`], [`create`] : le dossier projet ;
//! - [`edit`] : renommage et suppression ;
//! - [`profile`] : les capacités du moteur cible ;
//! - [`schedule`] : le calcul du plan d'exécution ;
//! - [`contract`] : le contrat d'un système ou d'un observateur ;
//! - [`validate`] : la validation du modèle.

pub mod contract;
pub mod create;
pub mod diagnostic;
pub mod edit;
pub mod layout;
pub mod load;
pub mod model;
pub mod profile;
pub mod save;
pub mod schedule;
pub mod validate;

pub use diagnostic::{Diagnostic, Level};
pub use model::Project;
pub use profile::EngineProfile;
