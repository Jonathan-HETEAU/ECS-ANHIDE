//! Les tests par exemples : leur format, leur cohérence avec le contrat (TST-002, TST-004,
//! TST-005, TST-012, TST-016) et l'état avant / après qu'ils décrivent.

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_json::Value as Json;
use serde_yaml::Value;

use crate::model::{Access, Io, Lifecycle, Project, ResourceAccess, Toggle, Trigger};

fn is_false(b: &bool) -> bool {
    !*b
}

/// Un test par exemple, tel qu'il s'écrit dans `examples.yaml` (TST-002).
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Example {
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Composants de l'entité avant le passage, avec leurs valeurs.
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub given: IndexMap<String, Value>,
    /// Composants activables désactivés avant le passage.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub disabled: Vec<String>,
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub given_resources: IndexMap<String, Value>,
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub given_messages: IndexMap<String, Vec<Value>>,
    /// Événement qui déclenche un observateur (TST-016).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event: Option<Value>,
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub expect: IndexMap<String, Value>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub expect_absent: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub expect_disabled: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub expect_enabled: Vec<String>,
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub expect_resources: IndexMap<String, Value>,
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub expect_messages: IndexMap<String, Vec<Value>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expect_spawned: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub expect_despawned: bool,
    /// Vrai tant que la sortie attendue reste à compléter (TST-012).
    #[serde(default, skip_serializing_if = "is_false")]
    pub incomplete: bool,
}

impl Example {
    /// Lit un exemple depuis sa valeur YAML.
    pub fn parse(value: &Value) -> Result<Example, String> {
        serde_yaml::from_value(value.clone()).map_err(|e| e.to_string())
    }

    pub fn to_value(&self) -> Value {
        serde_yaml::to_value(self).expect("un exemple est toujours sérialisable")
    }

    /// Les composants de l'entité en entrée, avec leur état activé (`false` si désactivé).
    pub fn entity(&self) -> Vec<(String, bool)> {
        self.given
            .keys()
            .map(|c| (c.clone(), !self.disabled.contains(c)))
            .collect()
    }
}

/// Un défaut de cohérence d'un exemple : code de l'exigence, élément cité, message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    pub code: &'static str,
    pub error: bool,
    /// Le nom à repérer dans le fichier pour situer le défaut (l'identifiant de l'exemple).
    pub needle: String,
    pub message: String,
}

impl Finding {
    fn error(code: &'static str, needle: &str, message: String) -> Self {
        Self {
            code,
            error: true,
            needle: needle.to_string(),
            message,
        }
    }

    fn warning(code: &'static str, needle: &str, message: String) -> Self {
        Self {
            error: false,
            ..Self::error(code, needle, message)
        }
    }
}

/// Convertit une valeur YAML en JSON typé selon le schéma des champs : un champ `f32` ou `f64`
/// devient toujours un décimal, pour que les règles CEL comparent et calculent sans surprise.
pub fn typed(project: &Project, element: &str, value: &Value) -> Json {
    let mut json = serde_json::to_value(value).unwrap_or(Json::Null);
    let fields = project.component(element).map(|c| &c.fields).or_else(|| {
        project
            .resources
            .iter()
            .find(|r| r.name == element)
            .map(|r| &r.fields)
    });
    if let (Some(fields), Some(map)) = (fields, json.as_object_mut()) {
        for (name, field) in fields {
            if matches!(field.ty.as_str(), "f32" | "f64")
                && let Some(n) = map.get(name).and_then(Json::as_f64)
            {
                map.insert(name.clone(), Json::from(n));
            }
        }
    }
    if json.is_null() {
        json = Json::Object(Default::default());
    }
    json
}

/// L'état décrit par un exemple, avant et après le passage, sous la forme attendue par les
/// règles CEL : `{ "Health": { "current": 100.0 }, "GameState": { "score": 10 } }`.
pub fn states(project: &Project, example: &Example) -> (Json, Json) {
    let mut before = serde_json::Map::new();
    for (name, value) in example.given.iter().chain(&example.given_resources) {
        before.insert(name.clone(), typed(project, name, value));
    }
    let mut after = before.clone();
    if example.expect_despawned {
        let resources: Vec<&String> = example.given_resources.keys().collect();
        after.retain(|k, _| resources.contains(&k));
    }
    for (name, value) in example.expect.iter().chain(&example.expect_resources) {
        after.insert(name.clone(), typed(project, name, value));
    }
    for name in &example.expect_absent {
        after.remove(name);
    }
    (Json::Object(before), Json::Object(after))
}

fn quoted(name: &str) -> String {
    format!("« {name} »")
}

/// Vérifie les valeurs d'un composant ou d'une ressource au regard de son schéma (TST-002).
fn check_values(
    project: &Project,
    example: &Example,
    element: &str,
    value: &Value,
    resource: bool,
    out: &mut Vec<Finding>,
) {
    let fields = if resource {
        project
            .resources
            .iter()
            .find(|r| r.name == element)
            .map(|r| &r.fields)
    } else {
        project.component(element).map(|c| &c.fields)
    };
    let Some(fields) = fields else {
        // Une relation se manipule comme un composant sans schéma déclaré.
        if resource || project.relationship(element).is_none() {
            let kind = if resource {
                "ressource inconnue"
            } else {
                "composant inconnu"
            };
            out.push(Finding::error(
                "TST-002",
                &example.id,
                format!("exemple « {} » : {kind} : {element}", example.id),
            ));
        }
        return;
    };
    let map = match value {
        Value::Mapping(map) => map.clone(),
        Value::Null => Default::default(),
        _ => {
            out.push(Finding::error(
                "TST-002",
                &example.id,
                format!(
                    "exemple « {} » : la valeur de {element} doit être une table de champs",
                    example.id
                ),
            ));
            return;
        }
    };
    for (key, v) in &map {
        let key = key.as_str().unwrap_or_default();
        match fields.get(key) {
            None => out.push(Finding::error(
                "TST-002",
                &example.id,
                format!("exemple « {} » : champ inconnu {element}.{key}", example.id),
            )),
            Some(field) => {
                if let Err(reason) = field.check(v) {
                    out.push(Finding::error(
                        "TST-002",
                        &example.id,
                        format!(
                            "exemple « {} » : valeur invalide pour {element}.{key} : {reason}",
                            example.id
                        ),
                    ));
                }
            }
        }
    }
}

/// Vérifie les exemples d'un système ou d'un observateur au regard de son contrat.
pub fn check_examples(
    project: &Project,
    name: &str,
    io: Io<'_>,
    trigger: Option<&Trigger>,
    examples: &[Value],
) -> Vec<Finding> {
    let mut out = Vec::new();
    let mut ids = std::collections::BTreeSet::new();
    for value in examples {
        let example = match Example::parse(value) {
            Ok(example) => example,
            Err(err) => {
                let id = value["id"].as_str().unwrap_or("?").to_string();
                out.push(Finding::error(
                    "TST-002",
                    &id,
                    format!("exemple « {id} » invalide : {err}"),
                ));
                continue;
            }
        };
        if !ids.insert(example.id.clone()) {
            out.push(Finding::error(
                "VAL-001",
                &example.id,
                format!("nom d'exemple en double : {}", example.id),
            ));
        }
        check_example(project, name, io, trigger, &example, &mut out);
    }
    out
}

fn check_example(
    project: &Project,
    name: &str,
    io: Io<'_>,
    trigger: Option<&Trigger>,
    example: &Example,
    out: &mut Vec<Finding>,
) {
    let id = &example.id;
    for (component, value) in example.given.iter().chain(&example.expect) {
        check_values(project, example, component, value, false, out);
    }
    for (resource, value) in example
        .given_resources
        .iter()
        .chain(&example.expect_resources)
    {
        check_values(project, example, resource, value, true, out);
    }

    // TST-004 : l'entrée doit être sélectionnée par le système.
    let entity = example.entity();
    if !io.selects(&entity) {
        let present = |c: &String| entity.iter().any(|(n, enabled)| n == c && *enabled);
        let mut reasons: Vec<String> = io
            .required()
            .filter(|c| !present(c))
            .map(|c| format!("faute de {}", quoted(c)))
            .collect();
        reasons.extend(
            io.excluded()
                .filter(|c| present(c))
                .map(|c| format!("à cause de {}", quoted(c))),
        );
        out.push(Finding::error(
            "TST-004",
            id,
            format!(
                "{} ne sélectionne pas l'entrée de l'exemple « {id} », {}",
                quoted(name),
                reasons.join(", ")
            ),
        ));
    }
    if let Some(Trigger::Lifecycle {
        lifecycle: Lifecycle::Add | Lifecycle::Insert,
        component,
    }) = trigger
        && !example.given.contains_key(component)
    {
        out.push(Finding::error(
            "TST-004",
            id,
            format!(
                "l'exemple « {id} » de {} doit contenir {}, dont l'ajout déclenche l'observateur",
                quoted(name),
                quoted(component)
            ),
        ));
    }
    for resource in example.given_resources.keys() {
        if !io.resources.contains_key(resource) {
            out.push(Finding::error(
                "TST-002",
                id,
                format!(
                    "l'exemple « {id} » donne la ressource {}, que {} n'utilise pas",
                    quoted(resource),
                    quoted(name)
                ),
            ));
        }
    }

    // TST-012 : un exemple incomplet est signalé, et sa sortie n'est pas vérifiée.
    if example.incomplete {
        out.push(Finding::warning(
            "TST-012",
            id,
            format!(
                "l'exemple « {id} » de {} est à compléter : sa sortie attendue reste à écrire",
                quoted(name)
            ),
        ));
        return;
    }

    // TST-005 : la sortie ne change que ce que le contrat déclare.
    let undeclared = |component: &str| {
        format!(
            "{} n'écrit, n'ajoute ni ne retire {} (exemple « {id} »)",
            quoted(name),
            quoted(component)
        )
    };
    for (component, value) in &example.expect {
        match example.given.get(component) {
            Some(before) => {
                let changed = typed(project, component, before) != typed(project, component, value);
                if changed && io.components.get(component) != Some(&Access::Write) {
                    out.push(Finding::error("TST-005", id, undeclared(component)));
                }
            }
            None => {
                if !io.structural.added().any(|c| c == component) {
                    out.push(Finding::error("TST-005", id, undeclared(component)));
                }
            }
        }
    }
    for component in &example.expect_absent {
        let removed = io.structural.removed().any(|c| c == component) || io.structural.despawns();
        if example.given.contains_key(component) && !removed {
            out.push(Finding::error("TST-005", id, undeclared(component)));
        }
    }
    for (list, enable) in [
        (&example.expect_enabled, true),
        (&example.expect_disabled, false),
    ] {
        for component in list {
            let declared = io.toggles.get(component).is_some_and(|t| match t {
                Toggle::Enable | Toggle::MayEnable => enable,
                Toggle::Disable | Toggle::MayDisable => !enable,
            });
            if !declared {
                let verb = if enable {
                    "n'active pas"
                } else {
                    "ne désactive pas"
                };
                out.push(Finding::error(
                    "TST-005",
                    id,
                    format!(
                        "{} {verb} {} (exemple « {id} »)",
                        quoted(name),
                        quoted(component)
                    ),
                ));
            }
        }
    }
    for channel in example.expect_messages.keys() {
        if !io.messages.get(channel).is_some_and(|a| a.emits()) {
            out.push(Finding::error(
                "TST-005",
                id,
                format!(
                    "{} n'émet pas sur {} (exemple « {id} »)",
                    quoted(name),
                    quoted(channel)
                ),
            ));
        }
    }
    for (resource, value) in &example.expect_resources {
        let changed = example.given_resources.get(resource).is_none_or(|before| {
            typed(project, resource, before) != typed(project, resource, value)
        });
        if changed && io.resources.get(resource) != Some(&ResourceAccess::Write) {
            out.push(Finding::error(
                "TST-005",
                id,
                format!(
                    "{} n'écrit pas la ressource {} (exemple « {id} »)",
                    quoted(name),
                    quoted(resource)
                ),
            ));
        }
    }
    for archetype in example.expect_spawned.iter().flatten() {
        if !io.structural.spawned().any(|a| a == archetype) {
            out.push(Finding::error(
                "TST-005",
                id,
                format!(
                    "{} ne crée pas d'entité {} (exemple « {id} »)",
                    quoted(name),
                    quoted(archetype)
                ),
            ));
        }
    }
    if example.expect_despawned && !io.structural.despawns() {
        out.push(Finding::error(
            "TST-005",
            id,
            format!(
                "{} ne détruit pas d'entité (exemple « {id} »)",
                quoted(name)
            ),
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Field, System};

    fn project() -> Project {
        let mut p = Project::new("t");
        let health = p.ensure_component("Health");
        health.fields.insert(
            "current".into(),
            serde_yaml::from_str::<Field>("{ type: f32, min: 0, max: 1000 }").unwrap(),
        );
        p.ensure_component("Hit").fields.insert(
            "amount".into(),
            serde_yaml::from_str("{ type: f32 }").unwrap(),
        );
        p.ensure_component("Dead");
        let s = p.ensure_system("Damage");
        s.components.insert("Health".into(), Access::Write);
        s.components.insert("Hit".into(), Access::Read);
        s.structural.remove.push("Hit".into());
        s.structural.may_add.push("Dead".into());
        p
    }

    fn findings(p: &Project, yaml: &str) -> Vec<Finding> {
        let system: &System = p.system("Damage").unwrap();
        let examples: Vec<Value> = serde_yaml::from_str(yaml).unwrap();
        check_examples(p, "Damage", system.io(), None, &examples)
    }

    #[test]
    fn tst_002_un_exemple_coherent_ne_produit_rien() {
        let p = project();
        let f = findings(
            &p,
            "- id: a\n  given: { Health: { current: 100 }, Hit: { amount: 30 } }\n  expect: { Health: { current: 70 }, Dead: {} }\n  expect_absent: [Hit]\n",
        );
        assert!(f.is_empty(), "{f:#?}");
    }

    #[test]
    fn tst_002_valeurs_et_champs_controles() {
        let p = project();
        let f = findings(
            &p,
            "- id: a\n  given: { Health: { current: 2000, bogus: 1 }, Hit: { amount: 1 } }\n",
        );
        assert_eq!(f.len(), 2, "{f:#?}");
        assert!(f.iter().all(|x| x.code == "TST-002"));
    }

    #[test]
    fn tst_002_les_decimaux_et_entiers_se_comparent_selon_le_schema() {
        let p = project();
        let f = findings(
            &p,
            "- id: a\n  given: { Health: { current: 100 }, Hit: { amount: 0 } }\n  expect: { Health: { current: 100.0 } }\n",
        );
        assert!(
            f.is_empty(),
            "100 et 100.0 sont la même valeur pour un f32 : {f:#?}"
        );
    }

    #[test]
    fn tst_004_entree_non_selectionnee() {
        let p = project();
        let f = findings(&p, "- id: a\n  given: { Health: { current: 1 } }\n");
        assert_eq!(f[0].code, "TST-004");
        assert!(
            f[0].message.contains("faute de « Hit »"),
            "{}",
            f[0].message
        );
    }

    #[test]
    fn tst_005_sorties_non_declarees() {
        let p = project();
        let f = findings(
            &p,
            "- id: a\n  given: { Health: { current: 1 }, Hit: { amount: 1 } }\n  expect: { Hit: { amount: 2 } }\n  expect_messages: { Boom: [{}] }\n  expect_despawned: true\n",
        );
        let codes: Vec<&str> = f.iter().map(|x| x.code).collect();
        assert_eq!(codes, ["TST-005", "TST-005", "TST-005"], "{f:#?}");
    }

    #[test]
    fn tst_012_un_exemple_incomplet_est_signale_sans_verifier_sa_sortie() {
        let p = project();
        let f = findings(
            &p,
            "- id: a\n  incomplete: true\n  given: { Health: { current: 0 }, Hit: { amount: 1 } }\n",
        );
        assert_eq!(f.len(), 1);
        assert_eq!((f[0].code, f[0].error), ("TST-012", false));
    }

    #[test]
    fn etats_avant_apres() {
        let p = project();
        let e = Example::parse(&serde_yaml::from_str("id: a\ngiven: { Health: { current: 100 }, Hit: { amount: 30 } }\nexpect: { Health: { current: 70 } }\nexpect_absent: [Hit]\n").unwrap()).unwrap();
        let (before, after) = states(&p, &e);
        assert_eq!(before["Health"]["current"], Json::from(100.0));
        assert_eq!(after["Health"]["current"], Json::from(70.0));
        assert!(after.get("Hit").is_none());
    }
}
