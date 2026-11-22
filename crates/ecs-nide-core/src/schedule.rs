//! Calcul du plan d'exécution (exigences ORD).

use std::collections::{BTreeSet, HashMap, VecDeque};
use std::fs;
use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::model::{
    Access, FORMAT_VERSION, Io, Lifecycle, Observer, Project, ResourceAccess, System, Trigger,
};
use crate::profile::EngineProfile;

pub const PLAN_FILE: &str = "schedule.plan.yaml";

/// Une donnée au sens de l'ordonnancement : composant, ressource ou canal de messages.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Data {
    Component(String),
    Resource(String),
    Channel(String),
}

impl Data {
    pub fn label(&self) -> String {
        match self {
            Data::Component(name) => name.clone(),
            Data::Resource(name) => format!("ressource {name}"),
            Data::Channel(name) => format!("canal {name}"),
        }
    }
}

/// Ce qu'un système lit, écrit, exige et exclut (définitions de `exigences/ORD-ordonnancement.md`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AccessSet {
    pub reads: BTreeSet<Data>,
    pub writes: BTreeSet<Data>,
    pub required: BTreeSet<String>,
    pub excluded: BTreeSet<String>,
}

impl AccessSet {
    pub fn of(project: &Project, io: Io<'_>) -> Self {
        let mut set = AccessSet::default();
        for (name, access) in io.components {
            let data = Data::Component(name.clone());
            match access {
                Access::Read | Access::Optional => {
                    set.reads.insert(data);
                }
                Access::Write => {
                    set.writes.insert(data);
                }
                Access::With | Access::Without => {
                    // La sélection dépend de l'état d'un composant activable : c'est une lecture.
                    if project.is_toggleable(name) {
                        set.reads.insert(data);
                    }
                }
            }
        }
        set.required = io.required().cloned().collect();
        set.excluded = io.excluded().cloned().collect();
        for name in io.changed {
            set.reads.insert(Data::Component(name.clone()));
        }
        for components in io.via.values() {
            for name in components.keys() {
                set.reads.insert(Data::Component(name.clone()));
            }
        }
        for name in io.toggles.keys() {
            set.writes.insert(Data::Component(name.clone()));
        }
        for (name, access) in io.resources {
            let data = Data::Resource(name.clone());
            match access {
                ResourceAccess::Read => set.reads.insert(data),
                ResourceAccess::Write => set.writes.insert(data),
            };
        }
        for (name, access) in io.messages {
            let data = Data::Channel(name.clone());
            if access.emits() {
                set.writes.insert(data);
            } else {
                set.reads.insert(data);
            }
        }
        set
    }

    /// Les données qui mettent les deux systèmes en conflit (vide s'il n'y en a pas).
    pub fn conflicts_with(&self, other: &Self) -> BTreeSet<Data> {
        let mut data = BTreeSet::new();
        for d in &self.writes {
            if other.reads.contains(d) || other.writes.contains(d) {
                data.insert(d.clone());
            }
        }
        for d in &other.writes {
            if self.reads.contains(d) {
                data.insert(d.clone());
            }
        }
        data
    }

    /// Vrai si aucune entité ne peut être sélectionnée par les deux systèmes.
    pub fn is_disjoint_from(&self, other: &Self) -> bool {
        !self.required.is_disjoint(&other.excluded) || !self.excluded.is_disjoint(&other.required)
    }
}

/// Un système placé dans une étape.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PlannedSystem {
    pub name: String,
    pub phase: Option<String>,
    pub every: u32,
}

/// Une étape : des systèmes exécutés en parallèle, et les observateurs qu'ils peuvent déclencher.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Stage {
    pub stage: usize,
    pub systems: Vec<PlannedSystem>,
    pub observers: Vec<String>,
}

/// Deux systèmes en conflit sans contrainte d'ordre entre eux (ORD-007).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Ambiguity {
    pub first: String,
    pub second: String,
    pub data: Vec<String>,
}

/// Le plan d'exécution d'une boucle, tel qu'il est exporté dans `schedule.plan.yaml` (ORD-011).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Plan {
    pub format: u32,
    pub profile: EngineProfile,
    /// L'ordre résolu : contraintes d'ordre explicites et implicites, phases, ordre de référence.
    pub resolved_order: Vec<String>,
    pub stages: Vec<Stage>,
    pub ambiguities: Vec<Ambiguity>,
    /// Cascades circulaires possibles d'observateurs, le premier observateur répété à la fin
    /// (ORD-017).
    pub observer_cycles: Vec<Vec<String>>,
}

impl Plan {
    /// Numéro (à partir de 1) de l'étape qui contient le système.
    pub fn stage_of(&self, system: &str) -> Option<usize> {
        self.stages
            .iter()
            .find(|stage| stage.systems.iter().any(|s| s.name == system))
            .map(|stage| stage.stage)
    }

    pub fn has_ambiguity(&self, a: &str, b: &str) -> bool {
        self.ambiguities
            .iter()
            .any(|amb| (amb.first == a && amb.second == b) || (amb.first == b && amb.second == a))
    }

    pub fn to_yaml(&self) -> String {
        serde_yaml::to_string(self).expect("un plan est toujours sérialisable")
    }

    /// Écrit le plan dans `schedule.plan.yaml` (ORD-011).
    pub fn write(&self, project_dir: &Path) -> std::io::Result<PathBuf> {
        let path = project_dir.join(PLAN_FILE);
        fs::write(&path, self.to_yaml())?;
        Ok(path)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum PlanError {
    /// Les contraintes d'ordre forment un cycle ; le premier système est répété à la fin (ORD-006).
    #[error("les contraintes d'ordre forment un cycle : {}", .0.join(" → "))]
    OrderCycle(Vec<String>),
}

/// Calcule le plan d'exécution du projet pour le profil de moteur donné.
pub fn compute_plan(project: &Project, profile: &EngineProfile) -> Result<Plan, PlanError> {
    let systems: Vec<&System> = project
        .systems_in_reference_order()
        .into_iter()
        .filter(|s| s.enabled)
        .collect();
    let n = systems.len();
    let index: HashMap<&str, usize> = systems
        .iter()
        .enumerate()
        .map(|(i, s)| (s.name.as_str(), i))
        .collect();
    let accesses: Vec<AccessSet> = systems
        .iter()
        .map(|s| AccessSet::of(project, s.io()))
        .collect();

    // Contraintes d'ordre : explicites, implicites (messages, ORD-015) et phases (ORD-012).
    let mut edges: BTreeSet<(usize, usize)> = BTreeSet::new();
    for (i, system) in systems.iter().enumerate() {
        for other in &system.before {
            if let Some(&j) = index.get(other.as_str()) {
                edges.insert((i, j));
            }
        }
        for other in &system.after {
            if let Some(&j) = index.get(other.as_str()) {
                edges.insert((j, i));
            }
        }
    }
    for (i, emitter) in systems.iter().enumerate() {
        for (channel, access) in &emitter.messages {
            if !access.emits() {
                continue;
            }
            for (j, reader) in systems.iter().enumerate() {
                if i != j && reader.messages.get(channel).is_some_and(|a| !a.emits()) {
                    edges.insert((i, j));
                }
            }
        }
    }
    let phase_rank: Vec<Option<usize>> = systems
        .iter()
        .map(|s| {
            s.phase
                .as_ref()
                .and_then(|p| project.schedule.phases.iter().position(|q| q == p))
        })
        .collect();
    for i in 0..n {
        for j in 0..n {
            if let (Some(a), Some(b)) = (phase_rank[i], phase_rank[j])
                && a < b
            {
                edges.insert((i, j));
            }
        }
    }
    let mut successors = vec![Vec::new(); n];
    let mut predecessors = vec![Vec::new(); n];
    for &(a, b) in &edges {
        successors[a].push(b);
        predecessors[b].push(a);
    }

    // Ordre résolu : tri topologique, l'ordre de référence départageant (ORD-004).
    let mut indegree: Vec<usize> = predecessors.iter().map(Vec::len).collect();
    let mut ready: BTreeSet<usize> = (0..n).filter(|&i| indegree[i] == 0).collect();
    let mut order = Vec::with_capacity(n);
    while let Some(i) = ready.pop_first() {
        order.push(i);
        for &j in &successors[i] {
            indegree[j] -= 1;
            if indegree[j] == 0 {
                ready.insert(j);
            }
        }
    }
    if order.len() < n {
        let remaining: BTreeSet<usize> = (0..n).filter(|&i| indegree[i] > 0).collect();
        let cycle = find_cycle(&remaining, &predecessors);
        let mut names: Vec<String> = cycle.iter().map(|&i| systems[i].name.clone()).collect();
        names.push(names[0].clone());
        return Err(PlanError::OrderCycle(names));
    }

    let conflict = |a: usize, b: usize| -> BTreeSet<Data> {
        if profile.disjoint_sets && accesses[a].is_disjoint_from(&accesses[b]) {
            return BTreeSet::new();
        }
        accesses[a].conflicts_with(&accesses[b])
    };

    // Placement : première étape compatible (ORD-002 à ORD-005, ORD-012 à ORD-014).
    let mut stage_of = vec![0usize; n];
    for (k, &i) in order.iter().enumerate() {
        stage_of[i] = if profile.inter_system_parallelism {
            let after_predecessors = predecessors[i].iter().map(|&p| stage_of[p] + 1);
            let after_conflicts = order[..k]
                .iter()
                .filter(|&&q| !conflict(q, i).is_empty())
                .map(|&q| stage_of[q] + 1);
            after_predecessors.chain(after_conflicts).max().unwrap_or(1)
        } else {
            k + 1
        };
    }

    // Ambiguïtés : conflit sans chemin de contraintes entre les deux systèmes (ORD-007).
    let reach: Vec<BTreeSet<usize>> = (0..n).map(|i| reachable(i, &successors)).collect();
    let mut ambiguities = Vec::new();
    for (k, &a) in order.iter().enumerate() {
        for &b in &order[k + 1..] {
            let data = conflict(a, b);
            if !data.is_empty() && !reach[a].contains(&b) && !reach[b].contains(&a) {
                ambiguities.push(Ambiguity {
                    first: systems[a].name.clone(),
                    second: systems[b].name.clone(),
                    data: data.iter().map(Data::label).collect(),
                });
            }
        }
    }

    // Observateurs déclenchés par chaque étape, directement ou en cascade (ORD-016).
    let observers = project.observers_in_declaration_order();
    let observer_graph: Vec<Vec<usize>> = observers
        .iter()
        .map(|o| direct_observers(project, &observers, o.io()))
        .collect();

    let stage_count = stage_of.iter().copied().max().unwrap_or(0);
    let stages = (1..=stage_count)
        .map(|number| {
            let members: Vec<usize> = order
                .iter()
                .copied()
                .filter(|&i| stage_of[i] == number)
                .collect();
            let mut triggered: Vec<usize> = Vec::new();
            for &i in &members {
                let direct = direct_observers(project, &observers, systems[i].io());
                for o in cascade(direct, &observer_graph) {
                    if !triggered.contains(&o) {
                        triggered.push(o);
                    }
                }
            }
            Stage {
                stage: number,
                systems: members
                    .iter()
                    .map(|&i| PlannedSystem {
                        name: systems[i].name.clone(),
                        phase: systems[i].phase.clone(),
                        every: systems[i].every,
                    })
                    .collect(),
                observers: triggered
                    .iter()
                    .map(|&o| observers[o].name.clone())
                    .collect(),
            }
        })
        .collect();

    let observer_cycles = observer_cycles(&observer_graph)
        .into_iter()
        .map(|cycle| {
            let mut names: Vec<String> = cycle.iter().map(|&o| observers[o].name.clone()).collect();
            names.push(names[0].clone());
            names
        })
        .collect();

    Ok(Plan {
        format: FORMAT_VERSION,
        profile: profile.clone(),
        resolved_order: order.iter().map(|&i| systems[i].name.clone()).collect(),
        stages,
        ambiguities,
        observer_cycles,
    })
}

/// Trouve un cycle parmi des nœuds qui ont tous au moins un prédécesseur dans l'ensemble, en
/// remontant les prédécesseurs. Le cycle commence par son nœud de plus petit index.
fn find_cycle(remaining: &BTreeSet<usize>, predecessors: &[Vec<usize>]) -> Vec<usize> {
    let start = *remaining.first().expect("un cycle a au moins un nœud");
    let mut path = vec![start];
    let mut current = start;
    loop {
        let previous = *predecessors[current]
            .iter()
            .filter(|p| remaining.contains(p))
            .min()
            .expect("chaque nœud restant a un prédécesseur restant");
        if let Some(pos) = path.iter().position(|&x| x == previous) {
            let mut cycle: Vec<usize> = path[pos..].to_vec();
            cycle.reverse();
            let min_pos = cycle
                .iter()
                .enumerate()
                .min_by_key(|&(_, &x)| x)
                .map(|(i, _)| i)
                .unwrap();
            cycle.rotate_left(min_pos);
            return cycle;
        }
        path.push(previous);
        current = previous;
    }
}

fn reachable(start: usize, successors: &[Vec<usize>]) -> BTreeSet<usize> {
    let mut seen = BTreeSet::new();
    let mut queue: VecDeque<usize> = successors[start].iter().copied().collect();
    while let Some(i) = queue.pop_front() {
        if seen.insert(i) {
            queue.extend(successors[i].iter().copied());
        }
    }
    seen
}

/// Composants que peut porter une entité sélectionnée : ses composants exigés et optionnels, et
/// ceux des archétypes déclarés qu'elle peut être.
pub(crate) fn candidate_components(project: &Project, io: Io<'_>) -> BTreeSet<String> {
    let required: BTreeSet<&String> = io.required().collect();
    let excluded: BTreeSet<&String> = io.excluded().collect();
    let mut set: BTreeSet<String> = io
        .components
        .iter()
        .filter(|(_, a)| !matches!(a, Access::Without))
        .map(|(name, _)| name.clone())
        .collect();
    for archetype in &project.archetypes {
        let has_all = required.iter().all(|c| archetype.components.contains(c));
        let has_none = !excluded.iter().any(|c| archetype.components.contains(c));
        if has_all && has_none {
            set.extend(archetype.components.iter().cloned());
        }
    }
    set
}

/// Observateurs (index dans `observers`) que des accès et effets peuvent déclencher directement.
fn direct_observers(project: &Project, observers: &[&Observer], io: Io<'_>) -> Vec<usize> {
    let target_of = |component: &String| {
        project
            .relationship(component)
            .map(|r| r.target_component.clone())
    };

    let mut added: BTreeSet<String> = io.structural.added().cloned().collect();
    for name in io.structural.spawned() {
        if let Some(archetype) = project.archetypes.iter().find(|a| &a.name == name) {
            added.extend(archetype.components.iter().cloned());
        }
    }
    let added_targets: Vec<String> = added.iter().filter_map(target_of).collect();
    added.extend(added_targets);

    let mut removed: BTreeSet<String> = io.structural.removed().cloned().collect();
    let removed_targets: Vec<String> = removed.iter().filter_map(target_of).collect();
    removed.extend(removed_targets);

    let despawned = if io.structural.despawns() {
        candidate_components(project, io)
    } else {
        BTreeSet::new()
    };

    observers
        .iter()
        .enumerate()
        .filter(|(_, observer)| match &observer.on {
            Trigger::Event { event } => io.triggers.contains_key(event),
            Trigger::Lifecycle {
                lifecycle,
                component,
            } => match lifecycle {
                Lifecycle::Add | Lifecycle::Insert => added.contains(component),
                Lifecycle::Replace | Lifecycle::Remove => {
                    removed.contains(component) || despawned.contains(component)
                }
                Lifecycle::Despawn => despawned.contains(component),
            },
        })
        .map(|(i, _)| i)
        .collect()
}

/// Les observateurs que des accès et effets peuvent déclencher, directement ou en cascade, dans
/// l'ordre de la cascade (ORD-016).
pub fn observers_triggered(project: &Project, io: Io<'_>) -> Vec<String> {
    let observers = project.observers_in_declaration_order();
    let graph: Vec<Vec<usize>> = observers
        .iter()
        .map(|o| direct_observers(project, &observers, o.io()))
        .collect();
    cascade(direct_observers(project, &observers, io), &graph)
        .into_iter()
        .map(|o| observers[o].name.clone())
        .collect()
}

/// Parcours en largeur de la cascade d'observateurs à partir des déclenchements directs.
fn cascade(direct: Vec<usize>, graph: &[Vec<usize>]) -> Vec<usize> {
    let mut result = Vec::new();
    let mut queue: VecDeque<usize> = direct.into();
    while let Some(o) = queue.pop_front() {
        if !result.contains(&o) {
            result.push(o);
            queue.extend(graph[o].iter().copied());
        }
    }
    result
}

/// Cycles élémentaires du graphe des observateurs, chacun commençant par son plus petit index.
fn observer_cycles(graph: &[Vec<usize>]) -> Vec<Vec<usize>> {
    let mut cycles: BTreeSet<Vec<usize>> = BTreeSet::new();
    for start in 0..graph.len() {
        // Plus court chemin de `start` vers lui-même.
        let mut parent: HashMap<usize, usize> = HashMap::new();
        let mut queue: VecDeque<usize> = VecDeque::new();
        for &next in &graph[start] {
            parent.entry(next).or_insert(start);
            queue.push_back(next);
        }
        let mut found = false;
        while let Some(node) = queue.pop_front() {
            if node == start {
                found = true;
                break;
            }
            for &next in &graph[node] {
                if let std::collections::hash_map::Entry::Vacant(e) = parent.entry(next) {
                    e.insert(node);
                    queue.push_back(next);
                }
            }
        }
        if !found {
            continue;
        }
        let mut cycle = vec![start];
        let mut node = parent[&start];
        while node != start {
            cycle.push(node);
            node = parent[&node];
        }
        cycle[1..].reverse();
        let min_pos = cycle
            .iter()
            .enumerate()
            .min_by_key(|&(_, &x)| x)
            .map(|(i, _)| i)
            .unwrap();
        cycle.rotate_left(min_pos);
        cycles.insert(cycle);
    }
    cycles.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Access, ChannelAccess, Toggle};

    fn plan(project: &Project) -> Plan {
        compute_plan(project, &EngineProfile::generic()).unwrap()
    }

    #[test]
    fn ord_002_deux_ecritures_du_meme_composant_sont_separees() {
        let mut p = Project::new("t");
        p.ensure_system("A")
            .components
            .insert("Position".into(), Access::Write);
        p.ensure_system("B")
            .components
            .insert("Position".into(), Access::Write);
        let plan = plan(&p);
        assert_ne!(plan.stage_of("A"), plan.stage_of("B"));
    }

    #[test]
    fn ord_003_une_lecture_optionnelle_est_une_lecture() {
        let mut p = Project::new("t");
        p.ensure_system("A")
            .components
            .insert("Target".into(), Access::Write);
        p.ensure_system("B")
            .components
            .insert("Target".into(), Access::Optional);
        let plan = plan(&p);
        assert_eq!(plan.stage_of("A"), Some(1));
        assert_eq!(plan.stage_of("B"), Some(2));
    }

    #[test]
    fn ord_003_un_filtre_sur_un_composant_activable_est_une_lecture() {
        let mut p = Project::new("t");
        p.ensure_component("Stunned").toggleable = true;
        p.ensure_system("Timer")
            .toggles
            .insert("Stunned".into(), Toggle::MayDisable);
        p.ensure_system("Act")
            .components
            .insert("Stunned".into(), Access::Without);
        assert_eq!(plan(&p).stage_of("Act"), Some(2));
    }

    #[test]
    fn ord_003_un_filtre_sur_un_composant_non_activable_n_est_pas_une_lecture() {
        let mut p = Project::new("t");
        p.ensure_system("Tagger")
            .structural
            .add
            .push("Enemy".into());
        p.ensure_system("Act")
            .components
            .insert("Enemy".into(), Access::Without);
        assert_eq!(plan(&p).stages.len(), 1);
    }

    #[test]
    fn ord_006_le_cycle_commence_par_le_premier_systeme_de_reference() {
        let mut p = Project::new("t");
        p.ensure_system("A").after.push("C".into());
        p.ensure_system("B").after.push("A".into());
        p.ensure_system("C").after.push("B".into());
        p.ensure_system("D").after.push("C".into());
        assert_eq!(
            compute_plan(&p, &EngineProfile::generic()).unwrap_err(),
            PlanError::OrderCycle(vec!["A".into(), "B".into(), "C".into(), "A".into()])
        );
    }

    #[test]
    fn ord_007_pas_d_ambiguite_quand_un_chemin_de_contraintes_existe() {
        let mut p = Project::new("t");
        p.ensure_system("A")
            .components
            .insert("X".into(), Access::Write);
        p.ensure_system("M").before.push("B".into());
        p.ensure_system("B")
            .components
            .insert("X".into(), Access::Write);
        p.system_mut("A").unwrap().before.push("M".into());
        assert!(plan(&p).ambiguities.is_empty());
    }

    #[test]
    fn ord_015_un_emetteur_passe_avant_ses_lecteurs_meme_declare_apres() {
        let mut p = Project::new("t");
        p.ensure_system("Reader")
            .messages
            .insert("Hits".into(), ChannelAccess::Read);
        p.ensure_system("Writer")
            .messages
            .insert("Hits".into(), ChannelAccess::MayEmit);
        let plan = plan(&p);
        assert_eq!(plan.resolved_order, ["Writer", "Reader"]);
        assert!(plan.ambiguities.is_empty());
    }

    #[test]
    fn ord_013_systemes_disjoints_par_lecture_et_absence() {
        let mut p = Project::new("t");
        let a = p.ensure_system("A");
        a.components.insert("Position".into(), Access::Write);
        a.components.insert("Player".into(), Access::Read);
        let b = p.ensure_system("B");
        b.components.insert("Position".into(), Access::Write);
        b.components.insert("Player".into(), Access::Without);
        let profile = EngineProfile {
            disjoint_sets: true,
            ..EngineProfile::generic()
        };
        assert_eq!(compute_plan(&p, &profile).unwrap().stages.len(), 1);
    }

    #[test]
    fn ord_016_un_despawn_declenche_les_observateurs_de_ses_composants() {
        let mut p = Project::new("t");
        let death = p.ensure_system("Death");
        death.components.insert("Dead".into(), Access::With);
        death.structural.despawn = true;
        p.ensure_observer(
            "OnEnemyGone",
            Trigger::Lifecycle {
                lifecycle: Lifecycle::Despawn,
                component: "Enemy".into(),
            },
        );
        p.archetypes.push(crate::model::Archetype {
            name: "Asteroid".into(),
            description: String::new(),
            components: vec!["Enemy".into(), "Dead".into()],
            estimated_count: None,
            related: Vec::new(),
        });
        assert_eq!(plan(&p).stages[0].observers, ["OnEnemyGone"]);
    }

    #[test]
    fn ord_010_un_systeme_desactive_est_exclu() {
        let mut p = Project::new("t");
        p.ensure_system("A");
        p.ensure_system("B").enabled = false;
        let plan = plan(&p);
        assert_eq!(plan.stage_of("B"), None);
        assert_eq!(plan.resolved_order, ["A"]);
    }

    #[test]
    fn ord_011_le_plan_s_exporte_en_yaml() {
        let mut p = Project::new("t");
        p.ensure_system("A");
        let yaml = plan(&p).to_yaml();
        for key in [
            "format:",
            "profile:",
            "stages:",
            "phase:",
            "every:",
            "ambiguities:",
        ] {
            assert!(yaml.contains(key), "{key} absent de :\n{yaml}");
        }
    }
}
