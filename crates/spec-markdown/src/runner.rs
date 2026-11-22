use std::collections::BTreeMap;
use std::panic::{AssertUnwindSafe, catch_unwind};

use crate::parser::{Scenario, Step, normalize_pattern};

/// Fonction associée à une phrase d'étape.
pub type StepFn<W> = fn(&mut W, &Step) -> Result<(), String>;

/// Une étape sans fonction associée.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UndefinedStep {
    pub line: usize,
    pub pattern: String,
}

/// Résultat de l'exécution d'un scénario.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    Passed,
    /// Au moins une étape n'a pas de fonction : le scénario n'est pas exécuté.
    Undefined(Vec<UndefinedStep>),
    Failed {
        line: usize,
        step: String,
        message: String,
    },
}

/// Le registre des étapes : phrase normalisée → fonction.
pub struct Registry<W> {
    steps: BTreeMap<String, StepFn<W>>,
}

impl<W> Default for Registry<W> {
    fn default() -> Self {
        Self {
            steps: BTreeMap::new(),
        }
    }
}

impl<W> Registry<W> {
    pub fn new() -> Self {
        Self::default()
    }

    /// Associe une phrase à une fonction. La phrase s'écrit avec `{}` pour une valeur entre
    /// guillemets et `{…}` pour une liste de valeurs séparées par des virgules.
    ///
    /// # Panics
    /// Si la phrase est déjà enregistrée.
    pub fn step(&mut self, pattern: &str, f: StepFn<W>) -> &mut Self {
        let key = normalize_pattern(pattern);
        assert!(
            self.steps.insert(key.clone(), f).is_none(),
            "étape enregistrée deux fois : {key}"
        );
        self
    }

    pub fn len(&self) -> usize {
        self.steps.len()
    }

    pub fn is_empty(&self) -> bool {
        self.steps.is_empty()
    }

    pub fn lookup(&self, step: &Step) -> Option<StepFn<W>> {
        self.steps.get(&step.pattern()).copied()
    }

    /// Exécute un scénario dans le monde donné. Une panique dans une étape est un échec.
    pub fn run(&self, scenario: &Scenario, world: &mut W) -> Outcome {
        let undefined: Vec<UndefinedStep> = scenario
            .steps
            .iter()
            .filter(|step| self.lookup(step).is_none())
            .map(|step| UndefinedStep {
                line: step.line,
                pattern: step.pattern(),
            })
            .collect();
        if !undefined.is_empty() {
            return Outcome::Undefined(undefined);
        }
        for step in &scenario.steps {
            let f = self.lookup(step).expect("étape vérifiée ci-dessus");
            let result = catch_unwind(AssertUnwindSafe(|| f(world, step)));
            let message = match result {
                Ok(Ok(())) => continue,
                Ok(Err(message)) => message,
                Err(panic) => panic
                    .downcast_ref::<String>()
                    .cloned()
                    .or_else(|| panic.downcast_ref::<&str>().map(|s| s.to_string()))
                    .unwrap_or_else(|| "panique".to_string()),
            };
            return Outcome::Failed {
                line: step.line,
                step: step.text.clone(),
                message,
            };
        }
        Outcome::Passed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse;

    #[derive(Default)]
    struct Counter {
        value: i64,
    }

    fn registry() -> Registry<Counter> {
        let mut r = Registry::new();
        r.step("Ajouter {}", |w: &mut Counter, s| {
            w.value += s.param(0)?.parse::<i64>().map_err(|e| e.to_string())?;
            Ok(())
        });
        r.step("Ajouter chacun de {…}", |w: &mut Counter, s| {
            for p in &s.params {
                w.value += p.parse::<i64>().map_err(|e| e.to_string())?;
            }
            Ok(())
        });
        r.step("La valeur est {}", |w: &mut Counter, s| {
            let expected: i64 = s.param(0)?.parse().map_err(|e| format!("{e}"))?;
            if w.value == expected {
                Ok(())
            } else {
                Err(format!("valeur {} au lieu de {expected}", w.value))
            }
        });
        r.step("Paniquer", |_: &mut Counter, _| panic!("boum"));
        r
    }

    fn scenario(body: &str) -> Scenario {
        parse(&format!("# T\n## S\n{body}"))
            .unwrap()
            .scenarios
            .remove(0)
    }

    #[test]
    fn scenario_reussi() {
        let sc = scenario(
            "* Ajouter \"2\"\n* Ajouter chacun de \"1\", \"2\", \"3\"\n* La valeur est \"8\"\n",
        );
        assert_eq!(
            registry().run(&sc, &mut Counter::default()),
            Outcome::Passed
        );
    }

    #[test]
    fn scenario_en_echec() {
        let sc = scenario("* Ajouter \"2\"\n* La valeur est \"3\"\n");
        match registry().run(&sc, &mut Counter::default()) {
            Outcome::Failed { line, message, .. } => {
                assert_eq!(line, 4);
                assert_eq!(message, "valeur 2 au lieu de 3");
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn une_panique_est_un_echec() {
        let sc = scenario("* Paniquer\n");
        assert!(matches!(
            registry().run(&sc, &mut Counter::default()),
            Outcome::Failed { message, .. } if message == "boum"
        ));
    }

    #[test]
    fn une_etape_inconnue_empeche_l_execution() {
        let sc = scenario("* Ajouter \"2\"\n* Multiplier par \"3\"\n");
        let mut counter = Counter::default();
        match registry().run(&sc, &mut counter) {
            Outcome::Undefined(steps) => {
                assert_eq!(
                    steps,
                    [UndefinedStep {
                        line: 4,
                        pattern: "Multiplier par {}".into()
                    }]
                );
            }
            other => panic!("{other:?}"),
        }
        assert_eq!(counter.value, 0, "aucune étape ne doit avoir été exécutée");
    }

    #[test]
    #[should_panic(expected = "étape enregistrée deux fois")]
    fn une_phrase_ne_s_enregistre_qu_une_fois() {
        let mut r = registry();
        r.step("Ajouter {}", |_: &mut Counter, _| Ok(()));
    }
}
