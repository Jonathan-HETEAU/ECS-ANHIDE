//! Les concepts du modèle déclaratif, tels qu'ils s'écrivent dans les fichiers YAML du projet.

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_yaml::Value;

/// Version du format des fichiers de projet gérée par cette version d'ECS NIDE (PRJ-009).
pub const FORMAT_VERSION: u32 = 1;

fn is_false(b: &bool) -> bool {
    !*b
}
fn is_true(b: &bool) -> bool {
    *b
}
fn yes() -> bool {
    true
}
fn one() -> u32 {
    1
}
fn is_one(n: &u32) -> bool {
    *n == 1
}
fn is_default<T: Default + PartialEq>(value: &T) -> bool {
    *value == T::default()
}
fn default_language() -> String {
    "en".to_string()
}

/// `ecs-nide.yaml` : l'identité du projet.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub format: u32,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub engine: Option<String>,
    /// Langue des fichiers générés dans le projet (PRJ-010).
    #[serde(default = "default_language")]
    pub language: String,
}

/// `schedule.yaml` : phases, ordre de référence des systèmes et ordre de déclaration des
/// observateurs.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Schedule {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub phases: Vec<String>,
    #[serde(default)]
    pub order: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub observers: Vec<String>,
}

/// Un champ typé (MOD-002, MOD-003).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Field {
    #[serde(rename = "type")]
    pub ty: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub values: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_len: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default: Option<Value>,
}

pub type Fields = IndexMap<String, Field>;

/// Indication de stockage d'un composant (MOD-016).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Storage {
    #[default]
    Dense,
    Sparse,
}

/// Un composant (MOD-001, MOD-004, MOD-013, MOD-016).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Component {
    pub name: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub description: String,
    #[serde(default, skip_serializing_if = "is_false")]
    pub toggleable: bool,
    #[serde(default, skip_serializing_if = "is_default")]
    pub storage: Storage,
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub fields: Fields,
}

impl Component {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: String::new(),
            toggleable: false,
            storage: Storage::Dense,
            fields: Fields::new(),
        }
    }

    /// Un composant sans champ est un marqueur (MOD-004).
    pub fn is_marker(&self) -> bool {
        self.fields.is_empty()
    }
}

/// Une ressource (MOD-005).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Resource {
    pub name: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub description: String,
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub fields: Fields,
}

/// Un canal de messages (MOD-017).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MessageChannel {
    pub name: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub description: String,
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub fields: Fields,
}

/// Un événement (MOD-022, MOD-030).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Event {
    pub name: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub description: String,
    /// Vrai pour un événement d'entité.
    #[serde(default, skip_serializing_if = "is_false")]
    pub entity: bool,
    /// Relation le long de laquelle l'événement d'entité se propage.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub propagate: Option<String>,
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub fields: Fields,
}

/// Une relation (MOD-026, MOD-027). Son nom est celui de son composant source.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Relationship {
    pub name: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub description: String,
    /// Composant tenu à jour sur l'entité cible, qui liste les entités sources.
    pub target_component: String,
    #[serde(default, skip_serializing_if = "is_false")]
    pub linked_spawn: bool,
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub fields: Fields,
}

/// Entités liées créées avec chaque entité d'un archétype (MOD-029).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RelatedSpawn {
    pub relationship: String,
    pub archetype: String,
    pub count: u32,
}

/// Un archétype (MOD-010, MOD-029).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Archetype {
    pub name: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub description: String,
    pub components: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub estimated_count: Option<u64>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub related: Vec<RelatedSpawn>,
}

/// Accès d'un système à un composant (MOD-006).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Access {
    Read,
    Write,
    Optional,
    With,
    Without,
}

/// Accès à une ressource (MOD-007).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourceAccess {
    Read,
    Write,
}

/// Accès à un canal de messages (MOD-018).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChannelAccess {
    Emit,
    MayEmit,
    Read,
}

impl ChannelAccess {
    pub fn emits(self) -> bool {
        matches!(self, Self::Emit | Self::MayEmit)
    }
}

/// Déclenchement d'un événement (MOD-023).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TriggerAccess {
    Trigger,
    MayTrigger,
}

/// Activation ou désactivation d'un composant activable (MOD-014).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Toggle {
    Enable,
    Disable,
    MayEnable,
    MayDisable,
}

/// Accès aux composants de la cible d'une relation (MOD-028) : lecture seule.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ViaAccess {
    Read,
}

/// Changements structurels, certains ou possibles (MOD-008).
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Structural {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub add: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub may_add: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub remove: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub may_remove: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub spawn: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub may_spawn: Vec<String>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub despawn: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub may_despawn: bool,
}

impl Structural {
    pub fn is_empty(&self) -> bool {
        *self == Self::default()
    }

    /// Composants ajoutés, certains ou possibles.
    pub fn added(&self) -> impl Iterator<Item = &String> {
        self.add.iter().chain(&self.may_add)
    }

    /// Composants retirés, certains ou possibles.
    pub fn removed(&self) -> impl Iterator<Item = &String> {
        self.remove.iter().chain(&self.may_remove)
    }

    /// Archétypes créés, certains ou possibles.
    pub fn spawned(&self) -> impl Iterator<Item = &String> {
        self.spawn.iter().chain(&self.may_spawn)
    }

    pub fn despawns(&self) -> bool {
        self.despawn || self.may_despawn
    }
}

/// Événement de cycle de vie d'un composant (MOD-024).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Lifecycle {
    Add,
    Insert,
    Replace,
    Remove,
    Despawn,
}

/// Déclencheur d'un observateur (MOD-024).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Trigger {
    Lifecycle {
        lifecycle: Lifecycle,
        component: String,
    },
    Event {
        event: String,
    },
}

pub type ComponentAccesses = IndexMap<String, Access>;
pub type ViaAccesses = IndexMap<String, IndexMap<String, ViaAccess>>;

/// Un système (MOD-006 à MOD-008, MOD-014, MOD-015, MOD-018 à MOD-021, MOD-023, MOD-028).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct System {
    pub name: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub description: String,
    #[serde(default = "yes", skip_serializing_if = "is_true")]
    pub enabled: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub phase: Option<String>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub per_entity: bool,
    #[serde(default = "one", skip_serializing_if = "is_one")]
    pub every: u32,
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub components: ComponentAccesses,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub changed: Vec<String>,
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub resources: IndexMap<String, ResourceAccess>,
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub messages: IndexMap<String, ChannelAccess>,
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub triggers: IndexMap<String, TriggerAccess>,
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub via: ViaAccesses,
    #[serde(default, skip_serializing_if = "Structural::is_empty")]
    pub structural: Structural,
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub toggles: IndexMap<String, Toggle>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub before: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub after: Vec<String>,
    /// Contenu de `spec.md`.
    #[serde(skip)]
    pub spec: String,
}

impl System {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: String::new(),
            enabled: true,
            phase: None,
            per_entity: false,
            every: 1,
            components: ComponentAccesses::new(),
            changed: Vec::new(),
            resources: IndexMap::new(),
            messages: IndexMap::new(),
            triggers: IndexMap::new(),
            via: ViaAccesses::new(),
            structural: Structural::default(),
            toggles: IndexMap::new(),
            before: Vec::new(),
            after: Vec::new(),
            spec: String::new(),
        }
    }

    pub fn io(&self) -> Io<'_> {
        Io {
            components: &self.components,
            changed: &self.changed,
            resources: &self.resources,
            messages: &self.messages,
            triggers: &self.triggers,
            via: &self.via,
            structural: &self.structural,
            toggles: &self.toggles,
        }
    }
}

/// Un observateur (MOD-024, MOD-025) : comme un système, plus un déclencheur.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Observer {
    pub name: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub description: String,
    pub on: Trigger,
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub components: ComponentAccesses,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub changed: Vec<String>,
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub resources: IndexMap<String, ResourceAccess>,
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub messages: IndexMap<String, ChannelAccess>,
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub triggers: IndexMap<String, TriggerAccess>,
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub via: ViaAccesses,
    #[serde(default, skip_serializing_if = "Structural::is_empty")]
    pub structural: Structural,
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub toggles: IndexMap<String, Toggle>,
    /// Contenu de `spec.md`.
    #[serde(skip)]
    pub spec: String,
}

impl Observer {
    pub fn new(name: impl Into<String>, on: Trigger) -> Self {
        Self {
            name: name.into(),
            description: String::new(),
            on,
            components: ComponentAccesses::new(),
            changed: Vec::new(),
            resources: IndexMap::new(),
            messages: IndexMap::new(),
            triggers: IndexMap::new(),
            via: ViaAccesses::new(),
            structural: Structural::default(),
            toggles: IndexMap::new(),
            spec: String::new(),
        }
    }

    pub fn io(&self) -> Io<'_> {
        Io {
            components: &self.components,
            changed: &self.changed,
            resources: &self.resources,
            messages: &self.messages,
            triggers: &self.triggers,
            via: &self.via,
            structural: &self.structural,
            toggles: &self.toggles,
        }
    }
}

/// Vue commune sur les accès et les effets d'un système ou d'un observateur.
#[derive(Debug, Clone, Copy)]
pub struct Io<'a> {
    pub components: &'a ComponentAccesses,
    pub changed: &'a [String],
    pub resources: &'a IndexMap<String, ResourceAccess>,
    pub messages: &'a IndexMap<String, ChannelAccess>,
    pub triggers: &'a IndexMap<String, TriggerAccess>,
    pub via: &'a ViaAccesses,
    pub structural: &'a Structural,
    pub toggles: &'a IndexMap<String, Toggle>,
}

impl Io<'_> {
    /// Composants dont la présence est exigée : `read`, `write` et `with` (MOD-009).
    pub fn required(&self) -> impl Iterator<Item = &String> {
        self.components
            .iter()
            .filter(|(_, a)| matches!(a, Access::Read | Access::Write | Access::With))
            .map(|(name, _)| name)
    }

    /// Composants dont l'absence est exigée : `without` (MOD-009).
    pub fn excluded(&self) -> impl Iterator<Item = &String> {
        self.components
            .iter()
            .filter(|(_, a)| **a == Access::Without)
            .map(|(name, _)| name)
    }
}

/// Un projet ECS NIDE complet.
#[derive(Debug, Clone, PartialEq)]
pub struct Project {
    pub manifest: Manifest,
    pub schedule: Schedule,
    pub components: Vec<Component>,
    pub resources: Vec<Resource>,
    pub messages: Vec<MessageChannel>,
    pub events: Vec<Event>,
    pub relationships: Vec<Relationship>,
    pub archetypes: Vec<Archetype>,
    pub systems: Vec<System>,
    pub observers: Vec<Observer>,
}

impl Project {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            manifest: Manifest {
                format: FORMAT_VERSION,
                name: name.into(),
                engine: None,
                language: default_language(),
            },
            schedule: Schedule::default(),
            components: Vec::new(),
            resources: Vec::new(),
            messages: Vec::new(),
            events: Vec::new(),
            relationships: Vec::new(),
            archetypes: Vec::new(),
            systems: Vec::new(),
            observers: Vec::new(),
        }
    }

    pub fn component(&self, name: &str) -> Option<&Component> {
        self.components.iter().find(|c| c.name == name)
    }

    pub fn is_toggleable(&self, component: &str) -> bool {
        self.component(component).is_some_and(|c| c.toggleable)
    }

    pub fn relationship(&self, name: &str) -> Option<&Relationship> {
        self.relationships.iter().find(|r| r.name == name)
    }

    pub fn system(&self, name: &str) -> Option<&System> {
        self.systems.iter().find(|s| s.name == name)
    }

    pub fn system_mut(&mut self, name: &str) -> Option<&mut System> {
        self.systems.iter_mut().find(|s| s.name == name)
    }

    pub fn observer(&self, name: &str) -> Option<&Observer> {
        self.observers.iter().find(|o| o.name == name)
    }

    /// Le composant `name`, créé s'il n'existe pas.
    pub fn ensure_component(&mut self, name: &str) -> &mut Component {
        let index = match self.components.iter().position(|c| c.name == name) {
            Some(i) => i,
            None => {
                self.components.push(Component::new(name));
                self.components.len() - 1
            }
        };
        &mut self.components[index]
    }

    /// Le système `name`, créé s'il n'existe pas et ajouté à la fin de l'ordre de référence.
    pub fn ensure_system(&mut self, name: &str) -> &mut System {
        let index = match self.systems.iter().position(|s| s.name == name) {
            Some(i) => i,
            None => {
                self.systems.push(System::new(name));
                self.systems.len() - 1
            }
        };
        if !self.schedule.order.iter().any(|n| n == name) {
            self.schedule.order.push(name.to_string());
        }
        &mut self.systems[index]
    }

    /// L'observateur `name`, créé avec le déclencheur `on` s'il n'existe pas, et ajouté à la fin
    /// de l'ordre de déclaration des observateurs.
    pub fn ensure_observer(&mut self, name: &str, on: Trigger) -> &mut Observer {
        let index = match self.observers.iter().position(|o| o.name == name) {
            Some(i) => i,
            None => {
                self.observers.push(Observer::new(name, on));
                self.observers.len() - 1
            }
        };
        if !self.schedule.observers.iter().any(|n| n == name) {
            self.schedule.observers.push(name.to_string());
        }
        &mut self.observers[index]
    }

    /// Les systèmes dans l'ordre de référence : ceux de `schedule.yaml`, puis les autres par
    /// ordre alphabétique.
    pub fn systems_in_reference_order(&self) -> Vec<&System> {
        ordered(&self.systems, &self.schedule.order, |s| &s.name)
    }

    /// Les observateurs dans l'ordre de déclaration : ceux de `schedule.yaml`, puis les autres
    /// par ordre alphabétique.
    pub fn observers_in_declaration_order(&self) -> Vec<&Observer> {
        ordered(&self.observers, &self.schedule.observers, |o| &o.name)
    }
}

fn ordered<'a, T>(items: &'a [T], order: &[String], name: fn(&T) -> &String) -> Vec<&'a T> {
    let mut result: Vec<&T> = order
        .iter()
        .filter_map(|n| items.iter().find(|item| name(item) == n))
        .collect();
    let mut rest: Vec<&T> = items
        .iter()
        .filter(|item| !order.contains(name(item)))
        .collect();
    rest.sort_by(|a, b| name(a).cmp(name(b)));
    result.extend(rest);
    result
}
