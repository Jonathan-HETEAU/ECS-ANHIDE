//! Le contrat d'un système ou d'un observateur : tout ce qu'une IA doit savoir pour l'implémenter
//! (TST-013, TST-016).

use std::collections::BTreeSet;

use indexmap::IndexMap;
use serde::Serialize;
use serde_yaml::Value;

use crate::model::{
    Archetype, ChannelAccess, Component, ComponentAccesses, Event, Io, MessageChannel, Project,
    Relationship, Resource, ResourceAccess, Structural, Toggle, Trigger, TriggerAccess,
    ViaAccesses,
};
use crate::profile::EngineProfile;
use crate::schedule::observers_triggered;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ContractKind {
    System,
    Observer,
}

/// Les schémas des seuls éléments qu'utilise le système ou l'observateur.
#[derive(Debug, Clone, PartialEq, Serialize, Default)]
pub struct Schemas {
    pub components: Vec<Component>,
    pub resources: Vec<Resource>,
    pub messages: Vec<MessageChannel>,
    pub events: Vec<Event>,
    pub relationships: Vec<Relationship>,
    pub archetypes: Vec<Archetype>,
}

/// Le moteur cible et son skill.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct EngineInfo {
    pub profile: EngineProfile,
    pub skill: Option<String>,
}

/// Le contrat autonome d'un système ou d'un observateur.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Contract {
    pub kind: ContractKind,
    pub name: String,
    pub description: String,
    pub trigger: Option<Trigger>,
    pub phase: Option<String>,
    pub every: u32,
    pub per_entity: bool,
    pub components: ComponentAccesses,
    pub changed: Vec<String>,
    pub resources: IndexMap<String, ResourceAccess>,
    pub messages: IndexMap<String, ChannelAccess>,
    pub triggers: IndexMap<String, TriggerAccess>,
    pub via: ViaAccesses,
    pub structural: Structural,
    pub toggles: IndexMap<String, Toggle>,
    pub before: Vec<String>,
    pub after: Vec<String>,
    /// Observateurs que le système peut déclencher, directement ou en cascade.
    pub triggered_observers: Vec<String>,
    pub schemas: Schemas,
    pub spec: String,
    pub examples: Vec<Value>,
    pub properties: Vec<Value>,
    pub engine: EngineInfo,
}

impl Contract {
    pub fn to_yaml(&self) -> String {
        serde_yaml::to_string(self).expect("un contrat est toujours sérialisable")
    }

    pub fn component_schema(&self, name: &str) -> Option<&Component> {
        self.schemas.components.iter().find(|c| c.name == name)
    }
}

/// Le contrat du système ou de l'observateur `name`, pour le profil de moteur donné.
pub fn contract(project: &Project, name: &str, profile: &EngineProfile) -> Option<Contract> {
    let engine = EngineInfo {
        profile: profile.clone(),
        skill: project
            .manifest
            .engine
            .as_ref()
            .map(|engine| format!("skills/{engine}/SKILL.md")),
    };
    if let Some(s) = project.system(name) {
        let io = s.io();
        return Some(Contract {
            kind: ContractKind::System,
            name: s.name.clone(),
            description: s.description.clone(),
            trigger: None,
            phase: s.phase.clone(),
            every: s.every,
            per_entity: s.per_entity,
            components: s.components.clone(),
            changed: s.changed.clone(),
            resources: s.resources.clone(),
            messages: s.messages.clone(),
            triggers: s.triggers.clone(),
            via: s.via.clone(),
            structural: s.structural.clone(),
            toggles: s.toggles.clone(),
            before: s.before.clone(),
            after: s.after.clone(),
            triggered_observers: observers_triggered(project, io),
            schemas: schemas(project, io, None),
            spec: s.spec.clone(),
            examples: s.examples.clone(),
            properties: s.properties.clone(),
            engine,
        });
    }
    let o = project.observer(name)?;
    let io = o.io();
    Some(Contract {
        kind: ContractKind::Observer,
        name: o.name.clone(),
        description: o.description.clone(),
        trigger: Some(o.on.clone()),
        phase: None,
        every: 1,
        per_entity: false,
        components: o.components.clone(),
        changed: o.changed.clone(),
        resources: o.resources.clone(),
        messages: o.messages.clone(),
        triggers: o.triggers.clone(),
        via: o.via.clone(),
        structural: o.structural.clone(),
        toggles: o.toggles.clone(),
        before: Vec::new(),
        after: Vec::new(),
        triggered_observers: observers_triggered(project, io),
        schemas: schemas(project, io, Some(&o.on)),
        spec: o.spec.clone(),
        examples: o.examples.clone(),
        properties: o.properties.clone(),
        engine,
    })
}

fn schemas(project: &Project, io: Io<'_>, trigger: Option<&Trigger>) -> Schemas {
    let mut components: BTreeSet<String> = io.mentioned_components().into_iter().cloned().collect();
    let archetypes: Vec<Archetype> = io
        .structural
        .spawned()
        .filter_map(|name| project.archetypes.iter().find(|a| &a.name == name))
        .cloned()
        .collect();
    for archetype in &archetypes {
        components.extend(archetype.components.iter().cloned());
    }
    let mut events: BTreeSet<String> = io.triggers.keys().cloned().collect();
    match trigger {
        Some(Trigger::Lifecycle { component, .. }) => {
            components.insert(component.clone());
        }
        Some(Trigger::Event { event }) => {
            events.insert(event.clone());
        }
        None => {}
    }
    Schemas {
        components: project
            .components
            .iter()
            .filter(|c| components.contains(&c.name))
            .cloned()
            .collect(),
        resources: project
            .resources
            .iter()
            .filter(|r| io.resources.contains_key(&r.name))
            .cloned()
            .collect(),
        messages: project
            .messages
            .iter()
            .filter(|m| io.messages.contains_key(&m.name))
            .cloned()
            .collect(),
        events: project
            .events
            .iter()
            .filter(|e| events.contains(&e.name))
            .cloned()
            .collect(),
        relationships: project
            .relationships
            .iter()
            .filter(|r| components.contains(&r.name) || components.contains(&r.target_component))
            .cloned()
            .collect(),
        archetypes,
    }
}
