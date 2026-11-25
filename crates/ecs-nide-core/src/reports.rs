//! Import des résultats de tests exécutés à l'extérieur (TST-011, TST-014, TST-015) et
//! conversion d'un contre-exemple en exemple (TST-012).
//!
//! - **CTRF** : chaque test porte, dans `extra.ecsnide`, le système, le type de test, l'identifiant
//!   et, pour une propriété en échec, le contre-exemple (schéma publié :
//!   `schemas/ctrf-extension.schema.json`).
//! - **JUnit XML** : le nom du test suit la convention `ecsnide::<Système>::example::<id>` ou
//!   `ecsnide::<Système>::property::<id>`.

use serde::{Deserialize, Serialize};
use serde_json::Value as Json;

use crate::diagnostic::Diagnostic;
use crate::examples::Example;
use crate::model::Project;

/// Le schéma JSON publié des champs d'extension CTRF d'ECS NIDE (TST-011).
pub const CTRF_EXTENSION_SCHEMA: &str = include_str!("../schemas/ctrf-extension.schema.json");

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TestKind {
    Example,
    Property,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TestStatus {
    Passed,
    Failed,
    Skipped,
    Other,
}

/// Un résultat rattaché à un exemple ou à une propriété du projet.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TestResult {
    pub name: String,
    pub system: String,
    pub kind: TestKind,
    pub id: String,
    pub status: TestStatus,
    pub message: Option<String>,
    /// L'entrée qui viole la propriété, par champ : `{ "Health.current": 0 }`.
    pub counterexample: Option<Json>,
    pub seed: Option<u64>,
}

/// Les résultats d'un rapport importé.
#[derive(Debug, Clone, PartialEq, Default, Serialize)]
pub struct ImportedResults {
    pub results: Vec<TestResult>,
    /// Les noms des tests sans correspondance dans le projet (TST-015).
    pub unmatched: Vec<String>,
    pub diagnostics: Vec<Diagnostic>,
}

impl ImportedResults {
    /// Nombre de tests réussis et échoués d'un système.
    pub fn summary(&self, system: &str) -> (usize, usize) {
        let of = |status| {
            self.results
                .iter()
                .filter(|r| r.system == system && r.status == status)
                .count()
        };
        (of(TestStatus::Passed), of(TestStatus::Failed))
    }

    pub fn find(&self, system: &str, kind: TestKind, id: &str) -> Option<&TestResult> {
        self.results
            .iter()
            .find(|r| r.system == system && r.kind == kind && r.id == id)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ImportError {
    #[error("rapport CTRF invalide : {0}")]
    Ctrf(String),
    #[error("rapport JUnit XML invalide : {0}")]
    Junit(String),
}

/// `ecsnide::Damage::example::basique` → (`Damage`, exemple, `basique`).
pub fn parse_test_name(name: &str) -> Option<(String, TestKind, String)> {
    let rest = &name[name.find("ecsnide::")? + "ecsnide::".len()..];
    let mut parts = rest.splitn(3, "::");
    let system = parts.next()?.to_string();
    let kind = match parts.next()? {
        "example" => TestKind::Example,
        "property" => TestKind::Property,
        _ => return None,
    };
    let id = parts.next()?.to_string();
    (!system.is_empty() && !id.is_empty()).then_some((system, kind, id))
}

/// Vrai si le projet contient cet exemple ou cette propriété.
fn exists(project: &Project, system: &str, kind: TestKind, id: &str) -> bool {
    let lists = project
        .system(system)
        .map(|s| (&s.examples, &s.properties))
        .or_else(|| {
            project
                .observer(system)
                .map(|o| (&o.examples, &o.properties))
        });
    let Some((examples, properties)) = lists else {
        return false;
    };
    let list = match kind {
        TestKind::Example => examples,
        TestKind::Property => properties,
    };
    list.iter().any(|v| v["id"].as_str() == Some(id))
}

struct Raw {
    name: String,
    status: TestStatus,
    message: Option<String>,
    target: Option<(String, TestKind, String)>,
    counterexample: Option<Json>,
    seed: Option<u64>,
}

fn collect(project: &Project, raws: Vec<Raw>) -> ImportedResults {
    let mut imported = ImportedResults::default();
    for raw in raws {
        let target = raw.target.or_else(|| parse_test_name(&raw.name));
        match target {
            Some((system, kind, id)) if exists(project, &system, kind, &id) => {
                imported.results.push(TestResult {
                    name: raw.name,
                    system,
                    kind,
                    id,
                    status: raw.status,
                    message: raw.message,
                    counterexample: raw.counterexample,
                    seed: raw.seed,
                })
            }
            _ => {
                imported.diagnostics.push(
                    Diagnostic::warning(format!(
                        "résultat sans correspondance dans le projet : {}",
                        raw.name
                    ))
                    .code("TST-015")
                    .element(raw.name.clone()),
                );
                imported.unmatched.push(raw.name);
            }
        }
    }
    imported
}

/// Importe un rapport au format CTRF (TST-011).
pub fn import_ctrf(project: &Project, text: &str) -> Result<ImportedResults, ImportError> {
    let root: Json = serde_json::from_str(text).map_err(|e| ImportError::Ctrf(e.to_string()))?;
    let tests = root["results"]["tests"]
        .as_array()
        .ok_or_else(|| ImportError::Ctrf("« results.tests » est absent".into()))?;
    let raws = tests
        .iter()
        .map(|test| {
            let extra = &test["extra"]["ecsnide"];
            let kind = match extra["kind"].as_str() {
                Some("example") => Some(TestKind::Example),
                Some("property") => Some(TestKind::Property),
                _ => None,
            };
            let target = match (extra["system"].as_str(), kind, extra["id"].as_str()) {
                (Some(system), Some(kind), Some(id)) => {
                    Some((system.to_string(), kind, id.to_string()))
                }
                _ => None,
            };
            Raw {
                name: test["name"].as_str().unwrap_or_default().to_string(),
                status: match test["status"].as_str() {
                    Some("passed") => TestStatus::Passed,
                    Some("failed") => TestStatus::Failed,
                    Some("skipped") | Some("pending") => TestStatus::Skipped,
                    _ => TestStatus::Other,
                },
                message: test["message"].as_str().map(String::from),
                target,
                counterexample: extra.get("counterexample").cloned(),
                seed: extra["seed"].as_u64(),
            }
        })
        .collect();
    Ok(collect(project, raws))
}

/// Importe un rapport au format JUnit XML, rattaché par la convention de nommage (TST-014).
pub fn import_junit(project: &Project, text: &str) -> Result<ImportedResults, ImportError> {
    let document =
        roxmltree::Document::parse(text).map_err(|e| ImportError::Junit(e.to_string()))?;
    let raws = document
        .descendants()
        .filter(|n| n.has_tag_name("testcase"))
        .map(|case| {
            let name = case.attribute("name").unwrap_or_default();
            let full = match case.attribute("classname") {
                Some(class) if parse_test_name(name).is_none() => format!("{class}::{name}"),
                _ => name.to_string(),
            };
            let failure = case
                .children()
                .find(|c| c.has_tag_name("failure") || c.has_tag_name("error"));
            let skipped = case.children().any(|c| c.has_tag_name("skipped"));
            Raw {
                name: full,
                status: if failure.is_some() {
                    TestStatus::Failed
                } else if skipped {
                    TestStatus::Skipped
                } else {
                    TestStatus::Passed
                },
                message: failure
                    .and_then(|f| f.attribute("message"))
                    .map(String::from),
                target: None,
                counterexample: None,
                seed: None,
            }
        })
        .collect();
    Ok(collect(project, raws))
}

/// Importe un rapport, au format CTRF ou JUnit XML selon son contenu.
pub fn import_report(project: &Project, text: &str) -> Result<ImportedResults, ImportError> {
    if text.trim_start().starts_with('<') {
        import_junit(project, text)
    } else {
        import_ctrf(project, text)
    }
}

/// Convertit le contre-exemple d'une propriété en échec en un nouvel exemple du système,
/// pré-rempli avec l'entrée connue et marqué comme incomplet (TST-012). Renvoie son identifiant.
pub fn add_counterexample(project: &mut Project, result: &TestResult) -> Result<String, String> {
    let counterexample = result
        .counterexample
        .as_ref()
        .and_then(Json::as_object)
        .ok_or("ce résultat n'a pas de contre-exemple")?;
    let resources: Vec<String> = project.resources.iter().map(|r| r.name.clone()).collect();
    let mut example = Example {
        id: String::new(),
        description: Some(format!(
            "Contre-exemple de la propriété « {} », à compléter.",
            result.id
        )),
        incomplete: true,
        ..Example::default()
    };
    for (path, value) in counterexample {
        let (element, field) = path
            .split_once('.')
            .ok_or_else(|| format!("chemin de champ invalide : {path}"))?;
        let target = if resources.iter().any(|r| r == element) {
            &mut example.given_resources
        } else {
            &mut example.given
        };
        let entry = target
            .entry(element.to_string())
            .or_insert_with(|| serde_yaml::Value::Mapping(Default::default()));
        if let serde_yaml::Value::Mapping(map) = entry {
            let value = serde_yaml::to_value(value).map_err(|e| e.to_string())?;
            map.insert(serde_yaml::Value::from(field), value);
        }
    }
    let examples = match project.system_mut(&result.system) {
        Some(system) => &mut system.examples,
        None => {
            &mut project
                .observers
                .iter_mut()
                .find(|o| o.name == result.system)
                .ok_or_else(|| format!("système inconnu : {}", result.system))?
                .examples
        }
    };
    let base = format!("contre-exemple-{}", result.id);
    let mut id = base.clone();
    let mut n = 1;
    while examples
        .iter()
        .any(|e| e["id"].as_str() == Some(id.as_str()))
    {
        n += 1;
        id = format!("{base}-{n}");
    }
    example.id = id.clone();
    examples.push(example.to_value());
    Ok(id)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn project() -> Project {
        let mut p = Project::new("t");
        let s = p.ensure_system("Damage");
        s.examples = serde_yaml::from_str("- id: basique\n").unwrap();
        s.properties = serde_yaml::from_str("- id: sante-positive\n  ensure: 'true'\n").unwrap();
        p
    }

    #[test]
    fn tst_014_convention_de_nommage() {
        assert_eq!(
            parse_test_name("tests::ecsnide::Damage::example::basique"),
            Some(("Damage".into(), TestKind::Example, "basique".into()))
        );
        assert_eq!(parse_test_name("ecsnide::Damage::other::x"), None);
        assert_eq!(parse_test_name("damage_basique"), None);
    }

    #[test]
    fn tst_014_import_junit() {
        let xml = r#"<testsuites><testsuite name="s">
            <testcase classname="game" name="ecsnide::Damage::example::basique"/>
            <testcase classname="game::ecsnide::Damage::property" name="sante-positive"><failure message="boom"/></testcase>
            <testcase name="autre"/>
        </testsuite></testsuites>"#;
        let r = import_junit(&project(), xml).unwrap();
        assert_eq!(r.summary("Damage"), (1, 1));
        assert_eq!(r.unmatched, ["autre"]);
        assert_eq!(
            r.find("Damage", TestKind::Property, "sante-positive")
                .unwrap()
                .message
                .as_deref(),
            Some("boom")
        );
    }

    #[test]
    fn tst_011_import_ctrf_avec_champs_d_extension() {
        let ctrf = r#"{"results": {"tool": {"name": "cargo"}, "tests": [
            {"name": "basique", "status": "passed", "extra": {"ecsnide": {"system": "Damage", "kind": "example", "id": "basique"}}},
            {"name": "p", "status": "failed", "extra": {"ecsnide": {"system": "Damage", "kind": "property", "id": "sante-positive",
                "counterexample": {"Health.current": 0, "Hit.amount": 1}, "seed": 7}}}
        ]}}"#;
        let r = import_ctrf(&project(), ctrf).unwrap();
        assert_eq!(r.summary("Damage"), (1, 1));
        let failed = r
            .find("Damage", TestKind::Property, "sante-positive")
            .unwrap();
        assert_eq!(failed.seed, Some(7));
        assert!(failed.counterexample.is_some());
    }

    #[test]
    fn tst_012_contre_exemple_vers_exemple_incomplet() {
        let mut p = project();
        let result = TestResult {
            name: "p".into(),
            system: "Damage".into(),
            kind: TestKind::Property,
            id: "sante-positive".into(),
            status: TestStatus::Failed,
            message: None,
            counterexample: Some(serde_json::json!({"Health.current": 0, "Hit.amount": 1})),
            seed: None,
        };
        let id = add_counterexample(&mut p, &result).unwrap();
        assert_eq!(id, "contre-exemple-sante-positive");
        let example = Example::parse(p.system("Damage").unwrap().examples.last().unwrap()).unwrap();
        assert!(example.incomplete);
        assert_eq!(
            example.given["Health"]["current"],
            serde_yaml::Value::from(0)
        );
        assert_eq!(
            add_counterexample(&mut p, &result).unwrap(),
            "contre-exemple-sante-positive-2"
        );
    }
}
