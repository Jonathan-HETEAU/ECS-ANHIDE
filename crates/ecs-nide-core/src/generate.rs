//! Génération de données de test pour une propriété (TST-007 à TST-010).
//!
//! - Les entrées sont sélectionnées par le système et respectent les types, les contraintes des
//!   schémas et le domaine de la propriété (TST-007).
//! - Une même graine donne exactement les mêmes données, sur toutes les plateformes (TST-008) :
//!   le générateur pseudo-aléatoire est implémenté ici, sans dépendance.
//! - Les premières entrées parcourent les valeurs limites de chaque champ (TST-009).
//! - Les données s'exportent en JSON, selon un schéma publié (TST-010).

use indexmap::IndexMap;
use serde::Serialize;
use serde_json::{Value as Json, json};

use crate::model::{Field, Project};
use crate::properties::{Domain, Property};

/// Le schéma JSON publié des données de test (TST-010).
pub const TEST_DATA_SCHEMA: &str = include_str!("../schemas/test-data.schema.json");

/// Une entrée générée : les composants de l'entité et les ressources lues.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Input {
    pub components: IndexMap<String, Json>,
    pub resources: IndexMap<String, Json>,
}

/// Un jeu de données de test pour une propriété.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Dataset {
    pub format: u32,
    pub system: String,
    pub property: String,
    pub seed: u64,
    pub inputs: Vec<Input>,
}

impl Dataset {
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).expect("un jeu de données est toujours sérialisable")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum GenerateError {
    #[error("système ou observateur inconnu : {0}")]
    UnknownActor(String),
    #[error("propriété inconnue : {0}")]
    UnknownProperty(String),
    #[error("propriété invalide : {0}")]
    InvalidProperty(String),
}

/// SplitMix64 : petit générateur déterministe, identique sur toutes les plateformes.
struct Rng(u64);

impl Rng {
    fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Un décimal dans [0, 1).
    fn unit(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }

    /// Un entier dans [low, high].
    fn int(&mut self, low: i64, high: i64) -> i64 {
        if high <= low {
            return low;
        }
        let span = (high - low) as u64 + 1;
        low + (self.next_u64() % span) as i64
    }
}

/// Bornes par défaut d'un champ numérique sans contrainte.
const DEFAULT_SPAN: f64 = 1000.0;

/// Les contraintes effectives d'un champ : celles du schéma, remplacées par le domaine.
struct Spec<'a> {
    ty: &'a str,
    min: Option<f64>,
    max: Option<f64>,
    values: Option<Vec<Json>>,
    max_len: Option<u64>,
}

impl<'a> Spec<'a> {
    fn new(field: &'a Field, domain: Option<&Domain>) -> Self {
        let number = |v: Option<&serde_yaml::Value>| v.and_then(serde_yaml::Value::as_f64);
        let values = |v: Option<&Vec<serde_yaml::Value>>| {
            v.map(|list| {
                list.iter()
                    .filter_map(|x| serde_json::to_value(x).ok())
                    .collect()
            })
        };
        let field_values = field
            .values
            .as_ref()
            .map(|list| list.iter().map(|s| json!(s)).collect());
        Spec {
            ty: field.ty.as_str(),
            min: domain
                .and_then(|d| number(d.min.as_ref()))
                .or(number(field.min.as_ref())),
            max: domain
                .and_then(|d| number(d.max.as_ref()))
                .or(number(field.max.as_ref())),
            values: domain
                .and_then(|d| values(d.values.as_ref()))
                .or(field_values),
            max_len: field.max_len,
        }
    }

    fn is_integer(&self) -> bool {
        matches!(
            self.ty,
            "i8" | "i16" | "i32" | "i64" | "u8" | "u16" | "u32" | "u64"
        )
    }

    /// Bornes numériques effectives.
    fn range(&self) -> (f64, f64) {
        let default_min = if self.ty.starts_with('u') {
            0.0
        } else {
            -DEFAULT_SPAN
        };
        let min = self.min.unwrap_or(default_min);
        let max = self.max.unwrap_or(min.max(0.0) + DEFAULT_SPAN);
        (min, max.max(min))
    }

    fn number(&self, n: f64) -> Json {
        if self.is_integer() {
            json!(n.round() as i64)
        } else {
            // Trois décimales : lisible, et reproductible d'une plateforme à l'autre.
            let (min, max) = self.range();
            json!(((n * 1000.0).round() / 1000.0).clamp(min, max))
        }
    }

    /// Les valeurs limites du champ (TST-009).
    fn boundaries(&self) -> Vec<Json> {
        if let Some(values) = &self.values {
            return values.clone();
        }
        match self.ty {
            "bool" => vec![json!(false), json!(true)],
            "string" => vec![json!("")],
            "entity" => vec![json!(0)],
            "vec2" | "vec3" | "vec4" | "quat" => vec![self.vector(|_| 0.0)],
            ty if ty.starts_with("list<") => vec![json!([])],
            _ => {
                let (min, max) = self.range();
                let mut values = Vec::new();
                for candidate in [
                    Some(min),
                    Some(max),
                    (min <= 0.0 && 0.0 <= max).then_some(0.0),
                ] {
                    let value = candidate.map(|n| self.number(n));
                    if let Some(v) = value
                        && !values.contains(&v)
                    {
                        values.push(v);
                    }
                }
                values
            }
        }
    }

    fn vector(&self, mut component: impl FnMut(usize) -> f64) -> Json {
        let axes: &[&str] = match self.ty {
            "vec2" => &["x", "y"],
            "vec3" => &["x", "y", "z"],
            _ => &["x", "y", "z", "w"],
        };
        let mut map = serde_json::Map::new();
        for (i, axis) in axes.iter().enumerate() {
            map.insert(
                (*axis).into(),
                json!(((component(i) * 1000.0).round()) / 1000.0),
            );
        }
        Json::Object(map)
    }

    fn random(&self, rng: &mut Rng) -> Json {
        if let Some(values) = &self.values
            && !values.is_empty()
        {
            return values[rng.int(0, values.len() as i64 - 1) as usize].clone();
        }
        match self.ty {
            "bool" => json!(rng.next_u64() & 1 == 1),
            "string" => {
                let max = self.max_len.unwrap_or(8).min(16) as i64;
                let len = rng.int(0, max);
                json!(
                    (0..len)
                        .map(|_| (b'a' + rng.int(0, 25) as u8) as char)
                        .collect::<String>()
                )
            }
            "entity" => json!(rng.int(0, 1_000_000)),
            "vec2" | "vec3" | "vec4" | "quat" => {
                self.vector(|_| -DEFAULT_SPAN + 2.0 * DEFAULT_SPAN * rng.unit())
            }
            ty if ty.starts_with("list<") => json!([]),
            _ => {
                let (min, max) = self.range();
                if self.is_integer() {
                    json!(rng.int(min.ceil() as i64, max.floor() as i64))
                } else {
                    self.number(min + (max - min) * rng.unit())
                }
            }
        }
    }
}

/// Un élément à générer : ressource ou composant, son nom, et ses champs avec leurs contraintes.
type Slot<'a> = (bool, String, Vec<(String, Spec<'a>)>);

/// Génère `count` entrées pour la propriété `property` du système ou de l'observateur `actor`.
pub fn generate(
    project: &Project,
    actor: &str,
    property: &str,
    count: usize,
    seed: u64,
) -> Result<Dataset, GenerateError> {
    let (io, properties) = match (project.system(actor), project.observer(actor)) {
        (Some(s), _) => (s.io(), &s.properties),
        (None, Some(o)) => (o.io(), &o.properties),
        _ => return Err(GenerateError::UnknownActor(actor.to_string())),
    };
    let value = properties
        .iter()
        .find(|p| p["id"].as_str() == Some(property))
        .ok_or_else(|| GenerateError::UnknownProperty(property.to_string()))?;
    let property = Property::parse(value).map_err(GenerateError::InvalidProperty)?;

    // Les champs à générer : composants exigés par le système, puis ressources qu'il lit.
    let mut slots: Vec<Slot<'_>> = Vec::new();
    for name in io.required() {
        let fields = project
            .component(name)
            .map(|c| {
                c.fields
                    .iter()
                    .map(|(f, field)| {
                        let domain = property.domain.get(&format!("{name}.{f}"));
                        (f.clone(), Spec::new(field, domain))
                    })
                    .collect()
            })
            .unwrap_or_default();
        slots.push((false, name.clone(), fields));
    }
    for name in io.resources.keys() {
        if let Some(resource) = project.resources.iter().find(|r| &r.name == name) {
            let fields = resource
                .fields
                .iter()
                .map(|(f, field)| {
                    let domain = property.domain.get(&format!("{name}.{f}"));
                    (f.clone(), Spec::new(field, domain))
                })
                .collect();
            slots.push((true, name.clone(), fields));
        }
    }

    // Les valeurs limites : une entrée par valeur limite de chaque champ.
    let boundaries: Vec<(usize, usize, Json)> = slots
        .iter()
        .enumerate()
        .flat_map(|(s, (_, _, fields))| {
            fields.iter().enumerate().flat_map(move |(f, (_, spec))| {
                spec.boundaries().into_iter().map(move |v| (s, f, v))
            })
        })
        .collect();

    let mut rng = Rng(seed);
    let inputs = (0..count)
        .map(|i| {
            let forced = boundaries.get(i);
            let mut input = Input {
                components: IndexMap::new(),
                resources: IndexMap::new(),
            };
            for (s, (is_resource, name, fields)) in slots.iter().enumerate() {
                let mut map = serde_json::Map::new();
                for (f, (field, spec)) in fields.iter().enumerate() {
                    let value = match forced {
                        Some((fs, ff, v)) if *fs == s && *ff == f => v.clone(),
                        _ => spec.random(&mut rng),
                    };
                    map.insert(field.clone(), value);
                }
                let target = if *is_resource {
                    &mut input.resources
                } else {
                    &mut input.components
                };
                target.insert(name.clone(), Json::Object(map));
            }
            input
        })
        .collect();

    Ok(Dataset {
        format: crate::model::FORMAT_VERSION,
        system: actor.to_string(),
        property: property.id,
        seed,
        inputs,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Access;

    fn project() -> Project {
        let mut p = Project::new("t");
        p.ensure_component("Health").fields.insert(
            "current".into(),
            serde_yaml::from_str("{ type: f32, min: 0, max: 1000 }").unwrap(),
        );
        p.ensure_component("Hit").fields.insert(
            "amount".into(),
            serde_yaml::from_str("{ type: u32 }").unwrap(),
        );
        p.ensure_component("Enemy");
        let s = p.ensure_system("Damage");
        s.components.insert("Health".into(), Access::Write);
        s.components.insert("Hit".into(), Access::Read);
        s.components.insert("Enemy".into(), Access::With);
        s.properties = serde_yaml::from_str(
            "- id: p\n  domain: { Hit.amount: { max: 50 } }\n  ensure: after.Health.current >= 0.0\n",
        )
        .unwrap();
        p
    }

    #[test]
    fn tst_007_009_bornes_domaine_et_valeurs_limites() {
        let data = generate(&project(), "Damage", "p", 200, 1).unwrap();
        let currents: Vec<f64> = data
            .inputs
            .iter()
            .map(|i| i.components["Health"]["current"].as_f64().unwrap())
            .collect();
        let amounts: Vec<i64> = data
            .inputs
            .iter()
            .map(|i| i.components["Hit"]["amount"].as_i64().unwrap())
            .collect();
        assert!(currents.iter().all(|c| (0.0..=1000.0).contains(c)));
        assert!(
            amounts.iter().all(|a| (0..=50).contains(a)),
            "le domaine borne Hit.amount"
        );
        for limit in [0.0, 1000.0] {
            assert!(currents.contains(&limit), "{limit} absent");
        }
        for limit in [0, 50] {
            assert!(amounts.contains(&limit), "{limit} absent");
        }
        assert!(
            data.inputs
                .iter()
                .all(|i| i.components["Enemy"] == json!({}))
        );
    }

    #[test]
    fn tst_008_meme_graine_memes_donnees() {
        let p = project();
        assert_eq!(
            generate(&p, "Damage", "p", 50, 42),
            generate(&p, "Damage", "p", 50, 42)
        );
        assert_ne!(
            generate(&p, "Damage", "p", 50, 42),
            generate(&p, "Damage", "p", 50, 43)
        );
    }

    #[test]
    fn tst_010_le_schema_publie_est_du_json() {
        let schema: Json = serde_json::from_str(TEST_DATA_SCHEMA).unwrap();
        assert_eq!(schema["type"], "object");
    }
}
