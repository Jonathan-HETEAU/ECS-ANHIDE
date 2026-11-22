//! Organisation des fichiers d'un dossier projet : un nom d'élément donne toujours le même chemin.

use std::path::PathBuf;

/// Les répertoires d'un projet (PRJ-001).
pub const DIRECTORIES: [&str; 9] = [
    "components",
    "resources",
    "messages",
    "events",
    "relationships",
    "archetypes",
    "systems",
    "observers",
    "skills",
];

/// Nom de fichier d'un élément : son nom en minuscules, mots séparés par des tirets.
///
/// `Health` → `health`, `InputState` → `input-state`, `EnemyAI` → `enemy-ai`.
pub fn file_stem(name: &str) -> String {
    let chars: Vec<char> = name.chars().collect();
    let mut out = String::with_capacity(name.len() + 4);
    for (i, &c) in chars.iter().enumerate() {
        if !c.is_alphanumeric() {
            if !out.ends_with('-') && !out.is_empty() {
                out.push('-');
            }
            continue;
        }
        if c.is_uppercase() && i > 0 {
            let previous = chars[i - 1];
            let next_is_lower = chars.get(i + 1).is_some_and(|n| n.is_lowercase());
            let boundary = previous.is_lowercase()
                || previous.is_ascii_digit()
                || (previous.is_uppercase() && next_is_lower);
            if boundary && !out.ends_with('-') {
                out.push('-');
            }
        }
        out.extend(c.to_lowercase());
    }
    out.trim_end_matches('-').to_string()
}

pub fn component_file(name: &str) -> PathBuf {
    PathBuf::from(format!("components/{}.yaml", file_stem(name)))
}

pub fn resource_file(name: &str) -> PathBuf {
    PathBuf::from(format!("resources/{}.yaml", file_stem(name)))
}

pub fn message_file(name: &str) -> PathBuf {
    PathBuf::from(format!("messages/{}.yaml", file_stem(name)))
}

pub fn event_file(name: &str) -> PathBuf {
    PathBuf::from(format!("events/{}.yaml", file_stem(name)))
}

pub fn relationship_file(name: &str) -> PathBuf {
    PathBuf::from(format!("relationships/{}.yaml", file_stem(name)))
}

pub fn archetype_file(name: &str) -> PathBuf {
    PathBuf::from(format!("archetypes/{}.yaml", file_stem(name)))
}

pub fn system_dir(name: &str) -> PathBuf {
    PathBuf::from(format!("systems/{}", file_stem(name)))
}

pub fn system_file(name: &str) -> PathBuf {
    system_dir(name).join("system.yaml")
}

pub fn observer_dir(name: &str) -> PathBuf {
    PathBuf::from(format!("observers/{}", file_stem(name)))
}

pub fn observer_file(name: &str) -> PathBuf {
    observer_dir(name).join("observer.yaml")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn noms_de_fichiers() {
        assert_eq!(file_stem("Health"), "health");
        assert_eq!(file_stem("InputState"), "input-state");
        assert_eq!(file_stem("EnemyAI"), "enemy-ai");
        assert_eq!(file_stem("HTTPServer"), "http-server");
        assert_eq!(file_stem("Vec2Grid"), "vec2-grid");
        assert_eq!(file_stem("ScoreOnDeath"), "score-on-death");
        assert_eq!(file_stem("Asteroides-fr"), "asteroides-fr");
        assert_eq!(file_stem("ChildOf"), "child-of");
    }
}
