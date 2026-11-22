//! Modifications du modèle qui touchent plusieurs éléments : renommage (MOD-011) et suppression
//! (MOD-012).

use indexmap::IndexMap;
use serde_yaml::Value;

use crate::model::{Project, Trigger};

/// Un élément du projet qui en référence un autre.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reference {
    /// `système`, `observateur`, `archétype` ou `relation`.
    pub kind: &'static str,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum EditError {
    #[error("composant inconnu : {0}")]
    UnknownComponent(String),
    #[error("le nom {0} est déjà pris")]
    NameTaken(String),
    #[error("{name} est référencé par {} élément(s) : confirmation nécessaire", references.len())]
    NeedsConfirmation {
        name: String,
        references: Vec<Reference>,
    },
}

/// Les éléments qui référencent le composant `name`.
pub fn component_references(project: &Project, name: &str) -> Vec<Reference> {
    let mut refs = Vec::new();
    for system in &project.systems {
        if system
            .io()
            .mentioned_components()
            .iter()
            .any(|c| *c == name)
        {
            refs.push(Reference {
                kind: "système",
                name: system.name.clone(),
            });
        }
    }
    for observer in &project.observers {
        let in_trigger =
            matches!(&observer.on, Trigger::Lifecycle { component, .. } if component == name);
        if in_trigger
            || observer
                .io()
                .mentioned_components()
                .iter()
                .any(|c| *c == name)
        {
            refs.push(Reference {
                kind: "observateur",
                name: observer.name.clone(),
            });
        }
    }
    for archetype in &project.archetypes {
        if archetype.components.iter().any(|c| c == name) {
            refs.push(Reference {
                kind: "archétype",
                name: archetype.name.clone(),
            });
        }
    }
    for relationship in &project.relationships {
        if relationship.target_component == name {
            refs.push(Reference {
                kind: "relation",
                name: relationship.name.clone(),
            });
        }
    }
    refs
}

/// Supprime le composant `name`. S'il est référencé, la suppression n'a lieu que si elle est
/// confirmée ; sinon, les références sont renvoyées pour que l'utilisateur décide (MOD-012).
pub fn delete_component(
    project: &mut Project,
    name: &str,
    confirmed: bool,
) -> Result<(), EditError> {
    if project.component(name).is_none() {
        return Err(EditError::UnknownComponent(name.to_string()));
    }
    let references = component_references(project, name);
    if !references.is_empty() && !confirmed {
        return Err(EditError::NeedsConfirmation {
            name: name.to_string(),
            references,
        });
    }
    project.components.retain(|c| c.name != name);
    Ok(())
}

/// Renomme le composant `old` en `new` et met à jour toutes ses références, y compris dans les
/// exemples, les propriétés et les spécifications (MOD-011).
pub fn rename_component(project: &mut Project, old: &str, new: &str) -> Result<(), EditError> {
    if project.component(new).is_some() {
        return Err(EditError::NameTaken(new.to_string()));
    }
    let component = project
        .components
        .iter_mut()
        .find(|c| c.name == old)
        .ok_or_else(|| EditError::UnknownComponent(old.to_string()))?;
    component.name = new.to_string();

    let rename_list = |list: &mut Vec<String>| {
        for item in list.iter_mut().filter(|i| *i == old) {
            *item = new.to_string();
        }
    };

    macro_rules! rename_io {
        ($element:expr) => {{
            let e = $element;
            rename_key(&mut e.components, old, new);
            rename_list(&mut e.changed);
            rename_key(&mut e.via, old, new);
            for inner in e.via.values_mut() {
                rename_key(inner, old, new);
            }
            rename_list(&mut e.structural.add);
            rename_list(&mut e.structural.may_add);
            rename_list(&mut e.structural.remove);
            rename_list(&mut e.structural.may_remove);
            rename_key(&mut e.toggles, old, new);
            for value in e.examples.iter_mut().chain(e.properties.iter_mut()) {
                rename_in_value(value, old, new);
            }
            e.spec = replace_word(&e.spec, old, new);
        }};
    }
    for system in &mut project.systems {
        rename_io!(system);
    }
    for observer in &mut project.observers {
        if let Trigger::Lifecycle { component, .. } = &mut observer.on
            && component == old
        {
            *component = new.to_string();
        }
        rename_io!(observer);
    }
    for archetype in &mut project.archetypes {
        rename_list(&mut archetype.components);
    }
    for relationship in &mut project.relationships {
        if relationship.target_component == old {
            relationship.target_component = new.to_string();
        }
    }
    Ok(())
}

fn rename_key<V>(map: &mut IndexMap<String, V>, old: &str, new: &str) {
    if map.contains_key(old) {
        *map = std::mem::take(map)
            .into_iter()
            .map(|(k, v)| (if k == old { new.to_string() } else { k }, v))
            .collect();
    }
}

/// Renomme dans une valeur YAML (exemple ou propriété) : les clés égales à `old`, et le mot `old`
/// dans les chaînes (`Health.current`, règles CEL, listes de composants).
fn rename_in_value(value: &mut Value, old: &str, new: &str) {
    match value {
        Value::String(s) => *s = replace_word(s, old, new),
        Value::Sequence(items) => items.iter_mut().for_each(|v| rename_in_value(v, old, new)),
        Value::Mapping(map) => {
            let entries: Vec<(Value, Value)> = std::mem::take(map).into_iter().collect();
            for (mut key, mut v) in entries {
                if let Value::String(k) = &mut key {
                    *k = replace_word(k, old, new);
                }
                rename_in_value(&mut v, old, new);
                map.insert(key, v);
            }
        }
        Value::Tagged(tagged) => rename_in_value(&mut tagged.value, old, new),
        _ => {}
    }
}

fn is_word_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// Remplace le mot entier `old` par `new` dans `text`.
pub fn replace_word(text: &str, old: &str, new: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(pos) = rest.find(old) {
        let before = rest[..pos].chars().next_back();
        let after = rest[pos + old.len()..].chars().next();
        out.push_str(&rest[..pos]);
        if before.is_some_and(is_word_char) || after.is_some_and(is_word_char) {
            out.push_str(old);
        } else {
            out.push_str(new);
        }
        rest = &rest[pos + old.len()..];
    }
    out.push_str(rest);
    out
}

/// Vrai si `text` contient le mot entier `word`.
pub fn contains_word(text: &str, word: &str) -> bool {
    replace_word(text, word, "\u{0}") != text
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn remplacement_de_mots_entiers() {
        assert_eq!(
            replace_word("after.Health.current <= Healthy", "Health", "HP"),
            "after.HP.current <= Healthy"
        );
        assert_eq!(replace_word("Health", "Health", "HP"), "HP");
        assert!(contains_word("a Health b", "Health"));
        assert!(!contains_word("HealthBar", "Health"));
    }
}
