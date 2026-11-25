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

impl EngineProfile {
    /// Le profil intégré d'un moteur cible (SKL-008), s'il est connu. C'est lui qu'installent
    /// les skills du moteur, dans `skills/<moteur>/profile.yaml`.
    pub fn for_engine(engine: &str) -> Option<Self> {
        match engine {
            // Bevy (SKL-010) : parallélisme entre systèmes et ensembles disjoints (With/Without),
            // filtres Changed/Added, fréquence par conditions d'exécution, observateurs et
            // relations. Pas de composant activable : seule l'entité entière se désactive.
            "bevy" => Some(Self {
                name: "bevy".to_string(),
                inter_system_parallelism: true,
                disjoint_sets: true,
                toggleable_components: false,
                change_filters: true,
                system_frequency: true,
                observers: true,
                relationships: true,
            }),
            _ => None,
        }
    }

    /// Le profil du moteur cible d'un projet, ou le profil générique (SKL-009).
    pub fn of_project(project: &crate::Project) -> Self {
        project
            .manifest
            .engine
            .as_deref()
            .and_then(Self::for_engine)
            .unwrap_or_else(Self::generic)
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
