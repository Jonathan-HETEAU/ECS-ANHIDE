//! Les tests par propriétés : une règle CEL sur l'état avant (`before`) et après (`after`) le
//! passage du système, valable pour toutes les entrées d'un domaine (TST-003, TST-006).

use std::collections::BTreeSet;

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_json::Value as Json;
use serde_yaml::Value;

use crate::examples::{Example, Finding, states};
use crate::model::{Access, Io, Project};

/// Bornes ou valeurs autorisées d'un champ, pour une propriété.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Domain {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub values: Option<Vec<Value>>,
}

/// Une propriété, telle qu'elle s'écrit dans `properties.yaml` (TST-003).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Property {
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Domaine des entrées, par champ : `Hit.amount: { min: 0, max: 100 }`.
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub domain: IndexMap<String, Domain>,
    /// La règle CEL, qui doit valoir `true`.
    pub ensure: String,
}

impl Property {
    pub fn parse(value: &Value) -> Result<Property, String> {
        serde_yaml::from_value(value.clone()).map_err(|e| e.to_string())
    }

    /// Vrai si l'entrée décrite par `before` appartient au domaine de la propriété.
    pub fn in_domain(&self, before: &Json) -> bool {
        self.domain.iter().all(|(path, domain)| {
            let Some((element, field)) = path.split_once('.') else {
                return true;
            };
            let Some(value) = before.get(element).and_then(|e| e.get(field)) else {
                return true;
            };
            let number = value.as_f64();
            let below = domain
                .min
                .as_ref()
                .and_then(Value::as_f64)
                .zip(number)
                .is_some_and(|(min, n)| n < min);
            let above = domain
                .max
                .as_ref()
                .and_then(Value::as_f64)
                .zip(number)
                .is_some_and(|(max, n)| n > max);
            let outside = domain.values.as_ref().is_some_and(|values| {
                !values
                    .iter()
                    .any(|v| serde_json::to_value(v).ok().as_ref() == Some(value))
            });
            !(below || above || outside)
        })
    }
}

/// Résultat de l'évaluation d'une règle sur un état avant / après.
#[derive(Debug, Clone, PartialEq)]
pub enum Verdict {
    Holds,
    Violated,
    /// La règle cite une donnée absente de cet état : elle ne s'applique pas.
    NotApplicable,
    Error(String),
}

/// Compile une règle CEL.
pub fn compile(rule: &str) -> Result<cel::Program, String> {
    cel::Program::compile(rule).map_err(|e| {
        e.to_string()
            .lines()
            .next()
            .unwrap_or_default()
            .trim_start_matches("ERROR: <input>:")
            .to_string()
    })
}

/// Évalue une règle sur un état avant / après.
pub fn evaluate(program: &cel::Program, before: &Json, after: &Json) -> Verdict {
    let mut context = cel::Context::default();
    let bind = |context: &mut cel::Context, name: &str, value: &Json| {
        cel::to_value(value)
            .map_err(|e| e.to_string())
            .and_then(|v| context.add_variable(name, v).map_err(|e| e.to_string()))
    };
    if let Err(e) =
        bind(&mut context, "before", before).and_then(|_| bind(&mut context, "after", after))
    {
        return Verdict::Error(e);
    }
    match program.execute(&context) {
        Ok(cel::Value::Bool(true)) => Verdict::Holds,
        Ok(cel::Value::Bool(false)) => Verdict::Violated,
        Ok(other) => Verdict::Error(format!("la règle doit renvoyer un booléen, pas {other:?}")),
        Err(cel::ExecutionError::NoSuchKey(_)) => Verdict::NotApplicable,
        Err(e) => Verdict::Error(e.to_string()),
    }
}

/// Les données citées par une règle : `X` pour chaque `before.X` ou `after.X`.
pub fn referenced_data(rule: &str) -> BTreeSet<String> {
    let mut found = BTreeSet::new();
    for prefix in ["before.", "after."] {
        let mut rest = rule;
        while let Some(pos) = rest.find(prefix) {
            let boundary = rest[..pos]
                .chars()
                .next_back()
                .is_none_or(|c| !(c.is_alphanumeric() || c == '_' || c == '.'));
            let tail = &rest[pos + prefix.len()..];
            let name: String = tail
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            if boundary && !name.is_empty() {
                found.insert(name);
            }
            rest = tail;
        }
    }
    found
}

/// Les données qu'une règle peut citer : composants et ressources auxquels le système accède,
/// et composants qu'il ajoute ou retire.
pub fn accessible_data(io: Io<'_>) -> BTreeSet<String> {
    let mut names: BTreeSet<String> = io
        .components
        .iter()
        .filter(|(_, access)| **access != Access::Without)
        .map(|(name, _)| name.clone())
        .collect();
    names.extend(io.structural.added().cloned());
    names.extend(io.structural.removed().cloned());
    names.extend(io.resources.keys().cloned());
    names
}

/// Vérifie les propriétés d'un système ou d'un observateur (TST-003), puis leur accord avec ses
/// exemples (TST-006).
pub fn check_properties(
    project: &Project,
    name: &str,
    io: Io<'_>,
    properties: &[Value],
    examples: &[Value],
) -> Vec<Finding> {
    let mut out = Vec::new();
    let accessible = accessible_data(io);
    let examples: Vec<Example> = examples
        .iter()
        .filter_map(|v| Example::parse(v).ok())
        .filter(|e| !e.incomplete)
        .collect();
    let mut ids = BTreeSet::new();
    for value in properties {
        let property = match Property::parse(value) {
            Ok(property) => property,
            Err(err) => {
                let id = value["id"].as_str().unwrap_or("?").to_string();
                out.push(error(
                    "TST-003",
                    &id,
                    format!("propriété « {id} » invalide : {err}"),
                ));
                continue;
            }
        };
        let id = &property.id;
        if !ids.insert(id.clone()) {
            out.push(error(
                "VAL-001",
                id,
                format!("nom de propriété en double : {id}"),
            ));
        }
        let program = match compile(&property.ensure) {
            Ok(program) => program,
            Err(err) => {
                out.push(error(
                    "TST-003",
                    id,
                    format!("la règle de « {id} » n'est pas une expression CEL valide : {err}"),
                ));
                continue;
            }
        };
        for variable in program.references().variables() {
            if variable != "before" && variable != "after" {
                out.push(error(
                    "TST-003",
                    id,
                    format!(
                        "la règle de « {id} » cite la variable inconnue « {variable} » : seules before et after existent"
                    ),
                ));
            }
        }
        for data in referenced_data(&property.ensure) {
            if !accessible.contains(&data) {
                out.push(error(
                    "TST-003",
                    id,
                    format!("la règle de « {id} » cite « {data} », qui n'est pas accessible par « {name} »"),
                ));
            }
        }
        for path in property.domain.keys() {
            let known = path.split_once('.').is_some_and(|(element, field)| {
                accessible.contains(element)
                    && project
                        .component(element)
                        .map(|c| c.fields.contains_key(field))
                        .or_else(|| {
                            project
                                .resources
                                .iter()
                                .find(|r| r.name == element)
                                .map(|r| r.fields.contains_key(field))
                        })
                        .unwrap_or(false)
            });
            if !known {
                out.push(error(
                    "TST-003",
                    id,
                    format!(
                        "le domaine de « {id} » cite le champ inconnu ou inaccessible « {path} »"
                    ),
                ));
            }
        }

        // TST-006 : chaque exemple complet doit respecter la propriété.
        for example in &examples {
            let (before, after) = states(project, example);
            if !property.in_domain(&before) {
                continue;
            }
            match evaluate(&program, &before, &after) {
                Verdict::Holds | Verdict::NotApplicable => {}
                Verdict::Violated => out.push(error(
                    "TST-006",
                    id,
                    format!(
                        "l'exemple « {} » contredit la propriété « {id} » de « {name} »",
                        example.id
                    ),
                )),
                Verdict::Error(err) => out.push(Finding {
                    code: "TST-006",
                    error: false,
                    needle: id.clone(),
                    message: format!(
                        "la propriété « {id} » ne peut pas être évaluée sur l'exemple « {} » : {err}",
                        example.id
                    ),
                }),
            }
        }
    }
    out
}

fn error(code: &'static str, needle: &str, message: String) -> Finding {
    Finding {
        code,
        error: true,
        needle: needle.to_string(),
        message,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn project() -> Project {
        let mut p = Project::new("t");
        p.ensure_component("Health").fields.insert(
            "current".into(),
            serde_yaml::from_str("{ type: f32, min: 0 }").unwrap(),
        );
        p.ensure_component("Hit").fields.insert(
            "amount".into(),
            serde_yaml::from_str("{ type: f32 }").unwrap(),
        );
        let s = p.ensure_system("Damage");
        s.components.insert("Health".into(), Access::Write);
        s.components.insert("Hit".into(), Access::Read);
        s.structural.remove.push("Hit".into());
        p
    }

    fn check(p: &Project, properties: &str, examples: &str) -> Vec<Finding> {
        let io = p.system("Damage").unwrap().io();
        let properties: Vec<Value> = serde_yaml::from_str(properties).unwrap();
        let examples: Vec<Value> = serde_yaml::from_str(examples).unwrap();
        check_properties(p, "Damage", io, &properties, &examples)
    }

    const EXAMPLES: &str = "- id: basique\n  given: { Health: { current: 100 }, Hit: { amount: 30 } }\n  expect: { Health: { current: 70 } }\n  expect_absent: [Hit]\n- id: soin\n  given: { Health: { current: 50 }, Hit: { amount: 1 } }\n  expect: { Health: { current: 80 } }\n";

    #[test]
    fn tst_006_un_exemple_qui_contredit_une_propriete_est_signale() {
        let p = project();
        let f = check(
            &p,
            "- id: ne-remonte-pas\n  ensure: after.Health.current <= before.Health.current\n",
            EXAMPLES,
        );
        assert_eq!(f.len(), 1, "{f:#?}");
        assert_eq!(f[0].code, "TST-006");
        assert!(f[0].message.contains("« soin »"));
    }

    #[test]
    fn tst_006_un_exemple_hors_domaine_n_est_pas_evalue() {
        let p = project();
        let f = check(
            &p,
            "- id: p\n  domain: { Hit.amount: { min: 10 } }\n  ensure: after.Health.current <= before.Health.current\n",
            EXAMPLES,
        );
        assert!(f.is_empty(), "{f:#?}");
    }

    #[test]
    fn tst_006_une_donnee_absente_rend_la_propriete_inapplicable() {
        let p = project();
        let f = check(&p, "- id: p\n  ensure: after.Hit.amount >= 0.0\n", EXAMPLES);
        assert!(f.is_empty(), "{f:#?}");
    }

    #[test]
    fn tst_003_regle_invalide_ou_donnees_inaccessibles() {
        let p = project();
        let f = check(
            &p,
            "- id: a\n  ensure: 'after.Health.current <='\n- id: b\n  ensure: after.Velocity.x > 0.0 && speed > 1.0\n- id: c\n  domain: { Mana.level: { min: 0 } }\n  ensure: 'true'\n",
            "[]",
        );
        let codes: Vec<&str> = f.iter().map(|x| x.code).collect();
        assert_eq!(codes, ["TST-003"; 4], "{f:#?}");
    }

    #[test]
    fn references_des_regles() {
        assert_eq!(
            referenced_data("after.Health.current <= before.Health.current && after.Dead.x"),
            BTreeSet::from(["Health".to_string(), "Dead".to_string()])
        );
        assert!(referenced_data("myafter.Health").is_empty());
    }
}
