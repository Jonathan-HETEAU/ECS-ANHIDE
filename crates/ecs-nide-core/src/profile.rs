use serde::{Deserialize, Serialize};

/// Les capacités du moteur cible (SKL-008). Le plan d'exécution s'y adapte.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EngineProfile {
    pub name: String,
    pub inter_system_parallelism: bool,
    pub disjoint_sets: bool,
    pub toggleable_components: bool,
    pub change_filters: bool,
    pub system_frequency: bool,
    pub observers: bool,
    pub relationships: bool,
}

impl EngineProfile {
    /// Le profil générique, utilisé tant qu'aucun moteur cible n'est choisi (SKL-009).
    pub fn generic() -> Self {
        Self {
            name: "generic".to_string(),
            inter_system_parallelism: true,
            disjoint_sets: false,
            toggleable_components: true,
            change_filters: true,
            system_frequency: true,
            observers: true,
            relationships: true,
        }
    }
}

impl Default for EngineProfile {
    fn default() -> Self {
        Self::generic()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skl_009_profil_generique() {
        let p = EngineProfile::generic();
        assert!(p.inter_system_parallelism);
        assert!(!p.disjoint_sets);
        assert!(p.toggleable_components && p.change_filters && p.system_frequency);
        assert!(p.observers && p.relationships);
    }
}
