//! Relie les scénarios de `features/` au code d'ECS NIDE.
//!
//! - [`repo`] lit les exigences, les scénarios et la liste des scénarios actifs ;
//! - [`steps`] associe chaque phrase d'étape à une fonction ;
//! - [`World`] est l'état partagé par les étapes d'un scénario.

pub mod repo;
pub mod steps;
mod world;

pub use world::World;
