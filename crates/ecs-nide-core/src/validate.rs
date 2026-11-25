//! Validation du modèle (exigences VAL, et diagnostics d'ordonnancement ORD-006, ORD-007, ORD-017).
//!
//! Chaque diagnostic porte le code de l'exigence qui le définit, l'élément concerné, le fichier et
//! la ligne (VAL-006). Le fichier est celui où l'élément est enregistré ; la ligne est celle de la
//! première mention du nom en cause dans ce fichier, tel qu'ECS NIDE l'écrit.

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use serde::Serialize;

use crate::diagnostic::Diagnostic;
use crate::edit::contains_word;
use crate::examples::{Finding, check_examples};
use crate::layout;
use crate::load::{EXAMPLES_FILE, MANIFEST_FILE, PROPERTIES_FILE, SCHEDULE_FILE, SPEC_FILE};
use crate::model::{Fields, Io, Lifecycle, Project, Trigger};
use crate::profile::EngineProfile;
use crate::properties::check_properties;
use crate::schedule::{PlanError, candidate_components, compute_plan};

/// Au-delà, le calcul des archétypes atteignables s'arrête (garde-fou contre l'explosion
/// combinatoire).
const REACHABLE_LIMIT: usize = 10_000;

/// Valide le projet pour le profil de moteur donné.
pub fn validate(project: &Project, profile: &EngineProfile) -> Vec<Diagnostic> {
    let mut v = Validator {
        project,
        profile,
        out: Vec::new(),
    };
    v.duplicates();
    v.field_types();
    v.references();
    v.specs();
    v.phases();
    v.io_rules();
    v.provided();
    v.reachable();
    v.unused();
    v.tests_present();
    v.tests();
    v.channels();
    v.events_observed();
    v.observers_triggerable();
    v.profile_support();
    v.plan();
    v.out
}

/// Un élément du modèle : son nom, son fichier et son contenu tel qu'il est enregistré.
struct Located {
    name: String,
    file: PathBuf,
    text: String,
}

fn located<T: Serialize>(name: &str, file: PathBuf, value: &T) -> Located {
    Located {
        name: name.to_string(),
        file,
        text: serde_yaml::to_string(value).unwrap_or_default(),
    }
}

/// Un système ou un observateur, vu de façon uniforme.
struct Actor<'a> {
    io: Io<'a>,
    loc: Located,
    is_system: bool,
    enabled: bool,
}

struct Validator<'a> {
    project: &'a Project,
    profile: &'a EngineProfile,
    out: Vec<Diagnostic>,
}

impl Validator<'_> {
    fn push(&mut self, diagnostic: Diagnostic, code: &str, at: &Located, needle: &str) {
        let line = at
            .text
            .lines()
            .position(|l| contains_word(l, needle))
            .map_or(1, |i| i + 1);
        self.out.push(
            diagnostic
                .code(code)
                .element(at.name.clone())
                .at(at.file.clone(), Some(line)),
        );
    }

    fn error(&mut self, code: &str, at: &Located, needle: &str, message: String) {
        self.push(Diagnostic::error(message), code, at, needle);
    }

    fn warning(&mut self, code: &str, at: &Located, needle: &str, message: String) {
        self.push(Diagnostic::warning(message), code, at, needle);
    }

    fn actors(&self) -> Vec<Actor<'_>> {
        let p = self.project;
        let systems = p.systems.iter().map(|s| Actor {
            io: s.io(),
            loc: located(&s.name, layout::system_file(&s.name), s),
            is_system: true,
            enabled: s.enabled,
        });
        let observers = p.observers.iter().map(|o| Actor {
            io: o.io(),
            loc: located(&o.name, layout::observer_file(&o.name), o),
            is_system: false,
            enabled: true,
        });
        systems.chain(observers).collect()
    }

    fn schedule_loc(&self) -> Located {
        located(SCHEDULE_FILE, SCHEDULE_FILE.into(), &self.project.schedule)
    }

    fn manifest_loc(&self) -> Located {
        located(
            &self.project.manifest.name,
            MANIFEST_FILE.into(),
            &self.project.manifest,
        )
    }

    /// Composants connus : déclarés, ou composants source et cible d'une relation.
    fn known_components(&self) -> BTreeSet<&str> {
        let p = self.project;
        p.components
            .iter()
            .map(|c| c.name.as_str())
            .chain(p.relationships.iter().map(|r| r.name.as_str()))
            .chain(p.relationships.iter().map(|r| r.target_component.as_str()))
            .collect()
    }

    // --- VAL-001 : noms en double -------------------------------------------------------------
    fn duplicates(&mut self) {
        let p = self.project;
        let mut check = |kind: &str, items: Vec<Located>| {
            let mut seen = BTreeSet::new();
            for item in items {
                if !seen.insert(item.name.clone()) {
                    let name = item.name.clone();
                    self.error(
                        "VAL-001",
                        &item,
                        &name,
                        format!("nom {kind} en double : {name}"),
                    );
                }
            }
        };
        check(
            "de composant",
            p.components
                .iter()
                .map(|c| located(&c.name, layout::component_file(&c.name), c))
                .collect(),
        );
        check(
            "de ressource",
            p.resources
                .iter()
                .map(|r| located(&r.name, layout::resource_file(&r.name), r))
                .collect(),
        );
        check(
            "de canal de messages",
            p.messages
                .iter()
                .map(|m| located(&m.name, layout::message_file(&m.name), m))
                .collect(),
        );
        check(
            "d'événement",
            p.events
                .iter()
                .map(|e| located(&e.name, layout::event_file(&e.name), e))
                .collect(),
        );
        check(
            "de relation",
            p.relationships
                .iter()
                .map(|r| located(&r.name, layout::relationship_file(&r.name), r))
                .collect(),
        );
        check(
            "d'archétype",
            p.archetypes
                .iter()
                .map(|a| located(&a.name, layout::archetype_file(&a.name), a))
                .collect(),
        );
        check(
            "de système",
            p.systems
                .iter()
                .map(|s| located(&s.name, layout::system_file(&s.name), s))
                .collect(),
        );
        check(
            "d'observateur",
            p.observers
                .iter()
                .map(|o| located(&o.name, layout::observer_file(&o.name), o))
                .collect(),
        );
    }

    // --- MOD-002, MOD-003 : types et valeurs par défaut ----------------------------------------
    fn field_types(&mut self) {
        let p = self.project;
        let mut owners: Vec<(Located, &Fields)> = Vec::new();
        owners.extend(p.components.iter().map(|c| {
            (
                located(&c.name, layout::component_file(&c.name), c),
                &c.fields,
            )
        }));
        owners.extend(p.resources.iter().map(|r| {
            (
                located(&r.name, layout::resource_file(&r.name), r),
                &r.fields,
            )
        }));
        owners.extend(p.messages.iter().map(|m| {
            (
                located(&m.name, layout::message_file(&m.name), m),
                &m.fields,
            )
        }));
        owners.extend(
            p.events
                .iter()
                .map(|e| (located(&e.name, layout::event_file(&e.name), e), &e.fields)),
        );
        owners.extend(p.relationships.iter().map(|r| {
            (
                located(&r.name, layout::relationship_file(&r.name), r),
                &r.fields,
            )
        }));
        for (loc, fields) in owners {
            for (field, def) in fields {
                if !crate::model::is_known_type(&def.ty) {
                    let message = format!("type inconnu : {} (champ {}.{field})", def.ty, loc.name);
                    self.error("MOD-002", &loc, field, message);
                } else if let Some(default) = &def.default
                    && let Err(reason) = def.check(default)
                {
                    let message = format!(
                        "valeur par défaut invalide pour {}.{field} : {reason}",
                        loc.name
                    );
                    self.error("MOD-003", &loc, field, message);
                }
            }
        }
    }

    // --- VAL-001 : références inconnues -------------------------------------------------------
    fn references(&mut self) {
        let p = self.project;
        let known = self.known_components();
        let resources: BTreeSet<&str> = p.resources.iter().map(|r| r.name.as_str()).collect();
        let channels: BTreeSet<&str> = p.messages.iter().map(|m| m.name.as_str()).collect();
        let events: BTreeSet<&str> = p.events.iter().map(|e| e.name.as_str()).collect();
        let relationships: BTreeSet<&str> =
            p.relationships.iter().map(|r| r.name.as_str()).collect();
        let archetypes: BTreeSet<&str> = p.archetypes.iter().map(|a| a.name.as_str()).collect();
        let systems: BTreeSet<&str> = p.systems.iter().map(|s| s.name.as_str()).collect();
        let observers: BTreeSet<&str> = p.observers.iter().map(|o| o.name.as_str()).collect();
        let mut found: Vec<(Located, String, String)> = Vec::new();

        for actor in self.actors() {
            let io = actor.io;
            let mut unknown = |kind: &str, name: &String| {
                found.push((
                    Located {
                        name: actor.loc.name.clone(),
                        file: actor.loc.file.clone(),
                        text: actor.loc.text.clone(),
                    },
                    name.clone(),
                    format!("{kind} : {name}"),
                ));
            };
            for c in io.mentioned_components() {
                if !known.contains(c.as_str()) {
                    unknown("composant inconnu", c);
                }
            }
            for r in io
                .resources
                .keys()
                .filter(|r| !resources.contains(r.as_str()))
            {
                unknown("ressource inconnue", r);
            }
            for m in io
                .messages
                .keys()
                .filter(|m| !channels.contains(m.as_str()))
            {
                unknown("canal de messages inconnu", m);
            }
            for e in io.triggers.keys().filter(|e| !events.contains(e.as_str())) {
                unknown("événement inconnu", e);
            }
            for r in io
                .via
                .keys()
                .filter(|r| !relationships.contains(r.as_str()))
            {
                unknown("relation inconnue", r);
            }
            for a in io
                .structural
                .spawned()
                .filter(|a| !archetypes.contains(a.as_str()))
            {
                unknown("archétype inconnu", a);
            }
        }
        for s in &p.systems {
            let loc = || located(&s.name, layout::system_file(&s.name), s);
            if let Some(phase) = &s.phase
                && !p.schedule.phases.contains(phase)
            {
                found.push((loc(), phase.clone(), format!("phase inconnue : {phase}")));
            }
            for other in s.before.iter().chain(&s.after) {
                if !systems.contains(other.as_str()) {
                    found.push((loc(), other.clone(), format!("système inconnu : {other}")));
                }
            }
        }
        for o in &p.observers {
            let loc = || located(&o.name, layout::observer_file(&o.name), o);
            match &o.on {
                Trigger::Event { event } if !events.contains(event.as_str()) => {
                    found.push((loc(), event.clone(), format!("événement inconnu : {event}")));
                }
                Trigger::Lifecycle { component, .. } if !known.contains(component.as_str()) => {
                    found.push((
                        loc(),
                        component.clone(),
                        format!("composant inconnu : {component}"),
                    ));
                }
                _ => {}
            }
        }
        for a in &p.archetypes {
            let loc = || located(&a.name, layout::archetype_file(&a.name), a);
            for c in a.components.iter().filter(|c| !known.contains(c.as_str())) {
                found.push((loc(), c.clone(), format!("composant inconnu : {c}")));
            }
            for related in &a.related {
                if !relationships.contains(related.relationship.as_str()) {
                    let r = &related.relationship;
                    found.push((loc(), r.clone(), format!("relation inconnue : {r}")));
                }
                if !archetypes.contains(related.archetype.as_str()) {
                    let x = &related.archetype;
                    found.push((loc(), x.clone(), format!("archétype inconnu : {x}")));
                }
            }
        }
        for e in &p.events {
            if let Some(r) = &e.propagate
                && !relationships.contains(r.as_str())
            {
                let loc = located(&e.name, layout::event_file(&e.name), e);
                found.push((loc, r.clone(), format!("relation inconnue : {r}")));
            }
        }
        let schedule = self.schedule_loc();
        for name in &p.schedule.order {
            if !systems.contains(name.as_str()) {
                found.push((
                    self.schedule_loc(),
                    name.clone(),
                    format!("système inconnu : {name}"),
                ));
            }
        }
        for name in &p.schedule.observers {
            if !observers.contains(name.as_str()) {
                found.push((
                    self.schedule_loc(),
                    name.clone(),
                    format!("observateur inconnu : {name}"),
                ));
            }
        }
        drop(schedule);
        for (loc, needle, message) in found {
            self.error("VAL-001", &loc, &needle, message);
        }
    }

    // --- TST-001, VAL-001 : spécifications et renvois vers les exemples et propriétés ---------
    fn specs(&mut self) {
        let p = self.project;
        let mut items: Vec<SpecItem<'_>> = Vec::new();
        for s in &p.systems {
            items.push((
                s.name.clone(),
                layout::system_dir(&s.name).join(SPEC_FILE),
                &s.spec,
                &s.examples,
                &s.properties,
            ));
        }
        for o in &p.observers {
            items.push((
                o.name.clone(),
                layout::observer_dir(&o.name).join(SPEC_FILE),
                &o.spec,
                &o.examples,
                &o.properties,
            ));
        }
        for (name, file, spec, examples, properties) in items {
            if spec.trim().is_empty() {
                continue;
            }
            let ids = |list: &[serde_yaml::Value]| -> BTreeSet<String> {
                list.iter()
                    .filter_map(|v| v["id"].as_str().map(String::from))
                    .collect()
            };
            let (example_ids, property_ids) = (ids(examples), ids(properties));
            match spec_markdown::parse(spec) {
                Err(err) => self.out.push(
                    Diagnostic::error(format!("spécification invalide : {}", err.message))
                        .code("TST-001")
                        .element(name.clone())
                        .at(file.clone(), Some(err.line)),
                ),
                Ok(parsed) => {
                    for scenario in &parsed.scenarios {
                        let refs = scenario
                            .tag("Exemples")
                            .iter()
                            .filter(|id| !example_ids.contains(*id))
                            .map(|id| format!("exemple inconnu : {id}"))
                            .chain(
                                scenario
                                    .tag("Propriétés")
                                    .iter()
                                    .filter(|id| !property_ids.contains(*id))
                                    .map(|id| format!("propriété inconnue : {id}")),
                            )
                            .collect::<Vec<_>>();
                        for message in refs {
                            self.out.push(
                                Diagnostic::error(message)
                                    .code("VAL-001")
                                    .element(name.clone())
                                    .at(file.clone(), Some(scenario.line)),
                            );
                        }
                    }
                }
            }
        }
    }

    // --- VAL-008 : phases ---------------------------------------------------------------------
    fn phases(&mut self) {
        let p = self.project;
        if p.schedule.phases.is_empty() {
            return;
        }
        for s in p.systems.iter().filter(|s| s.phase.is_none()) {
            let loc = located(&s.name, layout::system_file(&s.name), s);
            let message = format!("« {} » n'est rattaché à aucune phase", s.name);
            self.error("VAL-008", &loc, "name", message);
        }
    }

    // --- VAL-009, VAL-010, VAL-016 : règles sur les accès -------------------------------------
    fn io_rules(&mut self) {
        let mut found: Vec<(&str, Located, String, String)> = Vec::new();
        for actor in self.actors() {
            let io = actor.io;
            let required: BTreeSet<&String> = io.required().collect();
            let name = &actor.loc.name;
            for component in io.toggles.keys() {
                if self
                    .project
                    .component(component)
                    .is_some_and(|c| !c.toggleable)
                {
                    found.push(("VAL-009", clone_loc(&actor.loc), component.clone(), format!(
                        "« {component} » n'est pas activable : « {name} » ne peut ni l'activer ni le désactiver"
                    )));
                }
            }
            for component in io.changed.iter().filter(|c| !required.contains(c)) {
                found.push((
                    "VAL-010",
                    clone_loc(&actor.loc),
                    component.clone(),
                    format!(
                        "« {name} » filtre les changements de « {component} » sans le sélectionner"
                    ),
                ));
            }
            for relationship in io.via.keys().filter(|r| !required.contains(r)) {
                found.push(("VAL-016", clone_loc(&actor.loc), relationship.clone(), format!(
                    "« {name} » doit sélectionner « {relationship} » pour lire via cette relation"
                )));
            }
        }
        for (code, loc, needle, message) in found {
            self.error(code, &loc, &needle, message);
        }
    }

    /// Composants qu'une entité peut recevoir : archétypes, ajouts, créations, et composants cible
    /// des relations ainsi fournies.
    fn provided_components(&self) -> BTreeSet<String> {
        let p = self.project;
        let mut provided: BTreeSet<String> = p
            .archetypes
            .iter()
            .flat_map(|a| a.components.iter().cloned())
            .collect();
        for actor in self.actors() {
            provided.extend(actor.io.structural.added().cloned());
            for name in actor.io.structural.spawned() {
                if let Some(a) = p.archetypes.iter().find(|a| &a.name == name) {
                    provided.extend(a.components.iter().cloned());
                }
            }
        }
        for r in &p.relationships {
            if provided.contains(&r.name) {
                provided.insert(r.target_component.clone());
            }
        }
        provided
    }

    // --- VAL-002 : composant exigé mais jamais fourni -----------------------------------------
    fn provided(&mut self) {
        let provided = self.provided_components();
        let mut found = Vec::new();
        for actor in self.actors().into_iter().filter(|a| a.is_system) {
            for component in actor.io.required().filter(|c| !provided.contains(*c)) {
                let message = format!(
                    "« {component} » est lu par « {} » mais n'est jamais fourni",
                    actor.loc.name
                );
                found.push((clone_loc(&actor.loc), component.clone(), message));
            }
        }
        for (loc, needle, message) in found {
            self.warning("VAL-002", &loc, &needle, message);
        }
    }

    // --- VAL-003 : système qui ne sélectionne aucun archétype atteignable ---------------------
    fn reachable(&mut self) {
        let actors = self.actors();
        let selects = |io: &Io<'_>, set: &BTreeSet<String>| {
            io.required().all(|c| set.contains(c)) && !io.excluded().any(|c| set.contains(c))
        };
        let mut sets: BTreeSet<BTreeSet<String>> = self
            .project
            .archetypes
            .iter()
            .map(|a| a.components.iter().cloned().collect())
            .collect();
        let mut queue: Vec<BTreeSet<String>> = sets.iter().cloned().collect();
        while let Some(set) = queue.pop() {
            for actor in actors.iter().filter(|a| a.enabled) {
                if !selects(&actor.io, &set) {
                    continue;
                }
                let mut next = set.clone();
                next.extend(actor.io.structural.added().cloned());
                for removed in actor.io.structural.removed() {
                    next.remove(removed);
                }
                if sets.len() < REACHABLE_LIMIT && sets.insert(next.clone()) {
                    queue.push(next);
                }
            }
        }
        let mut found = Vec::new();
        for actor in actors.iter().filter(|a| a.is_system && a.enabled) {
            if !sets.iter().any(|set| selects(&actor.io, set)) {
                let message = format!(
                    "« {} » ne sélectionne aucun archétype atteignable",
                    actor.loc.name
                );
                found.push((clone_loc(&actor.loc), message));
            }
        }
        for (loc, message) in found {
            self.warning("VAL-003", &loc, "name", message);
        }
    }

    // --- VAL-004 : éléments inutilisés --------------------------------------------------------
    fn unused(&mut self) {
        let p = self.project;
        let actors = self.actors();
        let mut used_components: BTreeSet<String> = BTreeSet::new();
        let mut used_resources = BTreeSet::new();
        let mut used_channels = BTreeSet::new();
        let mut used_events = BTreeSet::new();
        for actor in &actors {
            used_components.extend(actor.io.mentioned_components().into_iter().cloned());
            used_resources.extend(actor.io.resources.keys().cloned());
            used_channels.extend(actor.io.messages.keys().cloned());
            used_events.extend(actor.io.triggers.keys().cloned());
        }
        for o in &p.observers {
            match &o.on {
                Trigger::Lifecycle { component, .. } => {
                    used_components.insert(component.clone());
                }
                Trigger::Event { event } => {
                    used_events.insert(event.clone());
                }
            }
        }
        for a in &p.archetypes {
            used_components.extend(a.components.iter().cloned());
            used_components.extend(a.related.iter().map(|r| r.relationship.clone()));
        }
        for r in &p.relationships {
            used_components.insert(r.target_component.clone());
        }
        for e in &p.events {
            if let Some(r) = &e.propagate {
                used_components.insert(r.clone());
            }
        }
        drop(actors);
        let mut found = Vec::new();
        for c in p
            .components
            .iter()
            .filter(|c| !used_components.contains(&c.name))
        {
            found.push((
                located(&c.name, layout::component_file(&c.name), c),
                format!("« {} » est inutilisé", c.name),
            ));
        }
        for r in p
            .resources
            .iter()
            .filter(|r| !used_resources.contains(&r.name))
        {
            found.push((
                located(&r.name, layout::resource_file(&r.name), r),
                format!("la ressource « {} » est inutilisée", r.name),
            ));
        }
        for m in p
            .messages
            .iter()
            .filter(|m| !used_channels.contains(&m.name))
        {
            found.push((
                located(&m.name, layout::message_file(&m.name), m),
                format!("le canal « {} » est inutilisé", m.name),
            ));
        }
        for e in p.events.iter().filter(|e| !used_events.contains(&e.name)) {
            found.push((
                located(&e.name, layout::event_file(&e.name), e),
                format!("l'événement « {} » est inutilisé", e.name),
            ));
        }
        for r in p
            .relationships
            .iter()
            .filter(|r| !used_components.contains(&r.name))
        {
            found.push((
                located(&r.name, layout::relationship_file(&r.name), r),
                format!("la relation « {} » est inutilisée", r.name),
            ));
        }
        for (loc, message) in found {
            self.warning("VAL-004", &loc, "name", message);
        }
    }

    // --- VAL-005 : spécification et tests présents --------------------------------------------
    fn tests_present(&mut self) {
        let p = self.project;
        let mut found = Vec::new();
        for s in &p.systems {
            let loc = || located(&s.name, layout::system_file(&s.name), s);
            if s.spec.trim().is_empty() {
                found.push((loc(), format!("« {} » n'a pas de spécification", s.name)));
            }
            if s.examples.is_empty() && s.properties.is_empty() {
                found.push((loc(), format!("« {} » n'a aucun test", s.name)));
            }
        }
        for o in &p.observers {
            let loc = || located(&o.name, layout::observer_file(&o.name), o);
            if o.spec.trim().is_empty() {
                found.push((loc(), format!("« {} » n'a pas de spécification", o.name)));
            }
            if o.examples.is_empty() && o.properties.is_empty() {
                found.push((loc(), format!("« {} » n'a aucun test", o.name)));
            }
        }
        for (loc, message) in found {
            self.warning("VAL-005", &loc, "name", message);
        }
    }

    // --- TST-002 à TST-006, TST-012, TST-016 : exemples et propriétés -------------------------
    fn tests(&mut self) {
        let p = self.project;
        let mut found: Vec<(Located, Finding)> = Vec::new();
        let mut actors: Vec<TestedActor<'_>> = Vec::new();
        for s in &p.systems {
            actors.push((
                s.name.clone(),
                layout::system_dir(&s.name),
                s.io(),
                None,
                &s.examples,
                &s.properties,
            ));
        }
        for o in &p.observers {
            actors.push((
                o.name.clone(),
                layout::observer_dir(&o.name),
                o.io(),
                Some(&o.on),
                &o.examples,
                &o.properties,
            ));
        }
        for (name, dir, io, trigger, examples, properties) in actors {
            let examples_loc = located(&name, dir.join(EXAMPLES_FILE), &examples.to_vec());
            for finding in check_examples(p, &name, io, trigger, examples) {
                found.push((clone_loc(&examples_loc), finding));
            }
            let properties_loc = located(&name, dir.join(PROPERTIES_FILE), &properties.to_vec());
            for finding in check_properties(p, &name, io, properties, examples) {
                found.push((clone_loc(&properties_loc), finding));
            }
        }
        for (loc, finding) in found {
            if finding.error {
                self.error(finding.code, &loc, &finding.needle, finding.message);
            } else {
                self.warning(finding.code, &loc, &finding.needle, finding.message);
            }
        }
    }

    // --- VAL-012 : canaux sans émetteur ou sans lecteur ---------------------------------------
    fn channels(&mut self) {
        let mut emitted: BTreeMap<String, Located> = BTreeMap::new();
        let mut read: BTreeMap<String, Located> = BTreeMap::new();
        for actor in self.actors() {
            for (channel, access) in actor.io.messages {
                let target = if access.emits() {
                    &mut emitted
                } else {
                    &mut read
                };
                target
                    .entry(channel.clone())
                    .or_insert_with(|| clone_loc(&actor.loc));
            }
        }
        for (channel, loc) in &read {
            if !emitted.contains_key(channel) {
                self.warning(
                    "VAL-012",
                    loc,
                    channel,
                    format!("le canal « {channel} » est lu mais jamais émis"),
                );
            }
        }
        for (channel, loc) in &emitted {
            if !read.contains_key(channel) {
                self.warning(
                    "VAL-012",
                    loc,
                    channel,
                    format!("le canal « {channel} » est émis mais jamais lu"),
                );
            }
        }
    }

    // --- VAL-015 : événement déclenché mais jamais observé ------------------------------------
    fn events_observed(&mut self) {
        let observed: BTreeSet<&String> = self
            .project
            .observers
            .iter()
            .filter_map(|o| match &o.on {
                Trigger::Event { event } => Some(event),
                Trigger::Lifecycle { .. } => None,
            })
            .collect();
        let mut found: BTreeMap<String, Located> = BTreeMap::new();
        for actor in self.actors() {
            for event in actor.io.triggers.keys().filter(|e| !observed.contains(e)) {
                found
                    .entry(event.clone())
                    .or_insert_with(|| clone_loc(&actor.loc));
            }
        }
        for (event, loc) in found {
            self.warning(
                "VAL-015",
                &loc,
                &event,
                format!("l'événement « {event} » est déclenché mais jamais observé"),
            );
        }
    }

    // --- VAL-014 : observateur que rien ne peut déclencher ------------------------------------
    fn observers_triggerable(&mut self) {
        let p = self.project;
        let provided = self.provided_components();
        let mut removed: BTreeSet<String> = BTreeSet::new();
        let mut despawned: BTreeSet<String> = BTreeSet::new();
        for actor in self.actors() {
            removed.extend(actor.io.structural.removed().cloned());
            if actor.io.structural.despawns() {
                despawned.extend(candidate_components(p, actor.io));
            }
        }
        let mut found = Vec::new();
        for o in &p.observers {
            let triggerable = match &o.on {
                Trigger::Event { event } => p
                    .systems
                    .iter()
                    .map(|s| s.io())
                    .chain(
                        p.observers
                            .iter()
                            .filter(|other| other.name != o.name)
                            .map(|x| x.io()),
                    )
                    .any(|io| io.triggers.contains_key(event)),
                Trigger::Lifecycle {
                    lifecycle,
                    component,
                } => match lifecycle {
                    Lifecycle::Add | Lifecycle::Insert => provided.contains(component),
                    Lifecycle::Replace | Lifecycle::Remove => {
                        removed.contains(component) || despawned.contains(component)
                    }
                    Lifecycle::Despawn => despawned.contains(component),
                },
            };
            if !triggerable {
                found.push(located(&o.name, layout::observer_file(&o.name), o));
            }
        }
        for loc in found {
            let message = format!("« {} » ne peut jamais être déclenché", loc.name);
            self.warning("VAL-014", &loc, "on", message);
        }
    }

    // --- VAL-013 : concepts non pris en charge par le moteur cible ----------------------------
    fn profile_support(&mut self) {
        let p = self.project;
        let profile = self.profile;
        let actors = self.actors();
        let mut unsupported = Vec::new();
        if !profile.toggleable_components && p.components.iter().any(|c| c.toggleable) {
            unsupported.push("les composants activables");
        }
        if !profile.change_filters && actors.iter().any(|a| !a.io.changed.is_empty()) {
            unsupported.push("les filtres de changement");
        }
        if !profile.system_frequency && p.systems.iter().any(|s| s.every > 1) {
            unsupported.push("la fréquence des systèmes");
        }
        if !profile.observers && !p.observers.is_empty() {
            unsupported.push("les observateurs");
        }
        if !profile.relationships && !p.relationships.is_empty() {
            unsupported.push("les relations");
        }
        drop(actors);
        let loc = self.manifest_loc();
        for concept in unsupported {
            let message = format!(
                "{concept} ne sont pas pris en charge par le moteur cible « {name} » : voir skills/{name}/SKILL.md",
                name = profile.name
            );
            self.warning("VAL-013", &loc, "name", message);
        }
    }

    // --- ORD-006, ORD-007, ORD-017 : plan d'exécution -----------------------------------------
    fn plan(&mut self) {
        let schedule = self.schedule_loc();
        match compute_plan(self.project, self.profile) {
            Err(PlanError::OrderCycle(cycle)) => {
                let message = format!(
                    "les contraintes d'ordre forment un cycle : {}",
                    cycle.join(" → ")
                );
                self.error("ORD-006", &schedule, &cycle[0], message);
            }
            Ok(plan) => {
                for ambiguity in &plan.ambiguities {
                    let message = format!(
                        "« {} » et « {} » sont en conflit sur {} sans contrainte d'ordre : « {} » passe en premier selon l'ordre de référence",
                        ambiguity.first,
                        ambiguity.second,
                        ambiguity.data.join(", "),
                        ambiguity.first
                    );
                    self.warning("ORD-007", &schedule, &ambiguity.second, message);
                }
                for cycle in &plan.observer_cycles {
                    let message = format!(
                        "cascade d'observateurs circulaire possible : {}",
                        cycle.join(" → ")
                    );
                    self.warning("ORD-017", &schedule, &cycle[0], message);
                }
            }
        }
    }
}

/// Un système ou un observateur dont on vérifie les tests : nom, dossier, accès, déclencheur,
/// exemples et propriétés.
type TestedActor<'a> = (
    String,
    PathBuf,
    Io<'a>,
    Option<&'a Trigger>,
    &'a [serde_yaml::Value],
    &'a [serde_yaml::Value],
);

/// Une spécification à vérifier : élément, fichier, texte, exemples et propriétés.
type SpecItem<'a> = (
    String,
    PathBuf,
    &'a str,
    &'a [serde_yaml::Value],
    &'a [serde_yaml::Value],
);

fn clone_loc(loc: &Located) -> Located {
    Located {
        name: loc.name.clone(),
        file: loc.file.clone(),
        text: loc.text.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Access, Archetype};

    fn codes(project: &Project) -> Vec<String> {
        validate(project, &EngineProfile::generic())
            .into_iter()
            .filter_map(|d| d.code)
            .collect()
    }

    #[test]
    fn un_projet_vide_est_valide() {
        assert!(validate(&Project::new("t"), &EngineProfile::generic()).is_empty());
    }

    #[test]
    fn val_006_chaque_diagnostic_est_localise() {
        let mut p = Project::new("t");
        p.ensure_system("Magic")
            .components
            .insert("Mana".into(), Access::Read);
        let diagnostics = validate(&p, &EngineProfile::generic());
        let unknown = diagnostics
            .iter()
            .find(|d| d.message == "composant inconnu : Mana")
            .expect("VAL-001 attendu");
        assert_eq!(
            unknown.file.as_deref(),
            Some(std::path::Path::new("systems/magic/system.yaml"))
        );
        assert_eq!(unknown.line, Some(3), "ligne de « Mana: read »");
        assert!(
            diagnostics
                .iter()
                .all(|d| d.code.is_some() && d.element.is_some() && d.line.is_some())
        );
    }

    #[test]
    fn val_003_atteignable_par_ajouts_successifs() {
        let mut p = Project::new("t");
        p.archetypes.push(Archetype {
            name: "Rock".into(),
            description: String::new(),
            components: vec!["Health".into()],
            estimated_count: None,
            related: Vec::new(),
        });
        let a = p.ensure_system("Hitter");
        a.components.insert("Health".into(), Access::With);
        a.structural.may_add.push("Hit".into());
        let b = p.ensure_system("Damage");
        b.components.insert("Hit".into(), Access::Read);
        b.structural.may_add.push("Dead".into());
        p.ensure_system("Death")
            .components
            .insert("Dead".into(), Access::With);
        assert!(
            !codes(&p).contains(&"VAL-003".to_string()),
            "{:?}",
            codes(&p)
        );
    }

    #[test]
    fn mod_003_une_valeur_par_defaut_hors_contraintes_est_une_erreur() {
        let mut p = Project::new("t");
        let c = p.ensure_component("Health");
        c.fields.insert(
            "current".into(),
            serde_yaml::from_str("{ type: f32, max: 10, default: 100 }").unwrap(),
        );
        assert!(codes(&p).contains(&"MOD-003".to_string()));
    }
}
