use std::collections::BTreeMap;

/// Clés reconnues sur les lignes de traçabilité d'un scénario (`Exigences : ORD-001, ORD-002`).
pub const TAG_KEYS: [&str; 3] = ["Exigences", "Exemples", "Propriétés"];

/// Un tableau Markdown placé sous une étape.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Table {
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

impl Table {
    /// Index d'une colonne d'après son en-tête.
    pub fn column(&self, header: &str) -> Option<usize> {
        self.headers.iter().position(|h| h == header)
    }

    /// Valeur d'une cellule d'après le numéro de ligne et l'en-tête de colonne.
    pub fn cell(&self, row: usize, header: &str) -> Option<&str> {
        let col = self.column(header)?;
        self.rows.get(row)?.get(col).map(String::as_str)
    }
}

/// Une étape exécutable : une puce `* Phrase naturelle`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Step {
    pub text: String,
    pub line: usize,
    /// Valeurs écrites entre guillemets droits, dans l'ordre.
    pub params: Vec<String>,
    pub table: Option<Table>,
}

impl Step {
    /// Forme normalisée de la phrase, qui sert de clé pour trouver la fonction de l'étape.
    ///
    /// Chaque valeur entre guillemets devient `{}`, et une suite de valeurs séparées par des
    /// virgules devient `{…}`. Ainsi `L'ordre est "A", "B", "C"` donne `L'ordre est {…}`.
    pub fn pattern(&self) -> String {
        normalize_pattern(&self.text)
    }

    /// La valeur entre guillemets numéro `index` (à partir de 0).
    pub fn param(&self, index: usize) -> Result<&str, String> {
        self.params.get(index).map(String::as_str).ok_or_else(|| {
            format!(
                "l'étape « {} » n'a pas de paramètre n°{}",
                self.text,
                index + 1
            )
        })
    }

    /// Le tableau de l'étape, ou une erreur explicite s'il manque.
    pub fn table(&self) -> Result<&Table, String> {
        self.table
            .as_ref()
            .ok_or_else(|| format!("l'étape « {} » attend un tableau", self.text))
    }
}

/// Un scénario : un titre `## …` qui énonce une règle, suivi de ses étapes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Scenario {
    pub title: String,
    pub line: usize,
    /// Lignes de traçabilité, par clé (`Exigences`, `Exemples`, `Propriétés`).
    pub tags: BTreeMap<String, Vec<String>>,
    pub description: String,
    pub steps: Vec<Step>,
}

impl Scenario {
    /// Valeurs d'une ligne de traçabilité (vide si absente).
    pub fn tag(&self, key: &str) -> &[String] {
        self.tags.get(key).map(Vec::as_slice).unwrap_or(&[])
    }
}

/// Une spécification : le contenu d'un fichier `*.spec.md`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Spec {
    pub title: String,
    pub description: String,
    pub scenarios: Vec<Scenario>,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("ligne {line} : {message}")]
pub struct ParseError {
    pub line: usize,
    pub message: String,
}

fn error(line: usize, message: impl Into<String>) -> ParseError {
    ParseError {
        line,
        message: message.into(),
    }
}

/// Lit une spécification au format Spec Markdown.
pub fn parse(source: &str) -> Result<Spec, ParseError> {
    let mut title: Option<String> = None;
    let mut description = String::new();
    let mut scenarios: Vec<Scenario> = Vec::new();
    let mut in_code_block = false;
    // Vrai tant que des lignes de tableau peuvent encore s'attacher à la dernière étape.
    let mut table_may_follow = false;
    // Vrai si la ligne non vide précédente était une ligne de tableau.
    let mut previous_was_table_row = false;

    for (index, raw) in source.lines().enumerate() {
        let line_no = index + 1;
        let trimmed = raw.trim();

        if trimmed.starts_with("```") {
            in_code_block = !in_code_block;
            table_may_follow = false;
            previous_was_table_row = false;
            continue;
        }
        if in_code_block {
            continue;
        }
        if trimmed.is_empty() {
            continue;
        }

        if trimmed.starts_with('|') {
            let cells = split_row(trimmed);
            if is_separator(&cells) {
                previous_was_table_row = true;
                continue;
            }
            let step = scenarios
                .last_mut()
                .and_then(|s| s.steps.last_mut())
                .filter(|_| table_may_follow || previous_was_table_row)
                .ok_or_else(|| error(line_no, "tableau sans étape juste au-dessus"))?;
            match &mut step.table {
                None => {
                    step.table = Some(Table {
                        headers: cells,
                        rows: Vec::new(),
                    })
                }
                Some(table) if previous_was_table_row => {
                    if cells.len() != table.headers.len() {
                        return Err(error(
                            line_no,
                            format!(
                                "la ligne de tableau a {} cellules au lieu de {}",
                                cells.len(),
                                table.headers.len()
                            ),
                        ));
                    }
                    table.rows.push(cells);
                }
                Some(_) => return Err(error(line_no, "une étape ne peut porter qu'un tableau")),
            }
            previous_was_table_row = true;
            continue;
        }
        previous_was_table_row = false;

        if let Some(text) = trimmed.strip_prefix("## ") {
            table_may_follow = false;
            scenarios.push(Scenario {
                title: text.trim().to_string(),
                line: line_no,
                tags: BTreeMap::new(),
                description: String::new(),
                steps: Vec::new(),
            });
            continue;
        }
        if let Some(text) = trimmed.strip_prefix("# ") {
            if title.is_some() {
                return Err(error(
                    line_no,
                    "une spécification n'a qu'un titre de niveau 1",
                ));
            }
            if !scenarios.is_empty() {
                return Err(error(
                    line_no,
                    "le titre de niveau 1 doit précéder les scénarios",
                ));
            }
            title = Some(text.trim().to_string());
            continue;
        }
        if let Some(text) = trimmed.strip_prefix("* ") {
            let scenario = scenarios
                .last_mut()
                .ok_or_else(|| error(line_no, "étape en dehors de tout scénario"))?;
            scenario.steps.push(Step {
                text: text.trim().to_string(),
                line: line_no,
                params: extract_params(text),
                table: None,
            });
            table_may_follow = true;
            continue;
        }
        table_may_follow = false;

        if let Some(scenario) = scenarios.last_mut() {
            if let Some((key, values)) = parse_tag_line(trimmed) {
                if !scenario.steps.is_empty() {
                    return Err(error(
                        line_no,
                        format!("la ligne « {key} : … » doit précéder les étapes"),
                    ));
                }
                scenario.tags.entry(key).or_default().extend(values);
                continue;
            }
            push_line(&mut scenario.description, trimmed);
        } else {
            push_line(&mut description, trimmed);
        }
    }

    if in_code_block {
        return Err(error(source.lines().count(), "bloc de code non refermé"));
    }
    let title = title.ok_or_else(|| error(1, "titre de niveau 1 (`# …`) manquant"))?;
    Ok(Spec {
        title,
        description,
        scenarios,
    })
}

fn push_line(buffer: &mut String, line: &str) {
    if !buffer.is_empty() {
        buffer.push('\n');
    }
    buffer.push_str(line);
}

fn parse_tag_line(line: &str) -> Option<(String, Vec<String>)> {
    let (key, values) = line.split_once(" : ")?;
    let key = key.trim();
    if !TAG_KEYS.contains(&key) {
        return None;
    }
    let values = values
        .split(',')
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(String::from)
        .collect();
    Some((key.to_string(), values))
}

fn split_row(line: &str) -> Vec<String> {
    let inner = line.trim().trim_start_matches('|').trim_end_matches('|');
    inner
        .split('|')
        .map(|cell| cell.trim().to_string())
        .collect()
}

fn is_separator(cells: &[String]) -> bool {
    !cells.is_empty()
        && cells.iter().all(|c| {
            let c = c.trim_matches(':');
            !c.is_empty() && c.chars().all(|ch| ch == '-')
        })
}

fn extract_params(text: &str) -> Vec<String> {
    let mut params = Vec::new();
    let mut rest = text;
    while let Some(start) = rest.find('"') {
        let after = &rest[start + 1..];
        match after.find('"') {
            Some(end) => {
                params.push(after[..end].to_string());
                rest = &after[end + 1..];
            }
            None => break,
        }
    }
    params
}

/// Normalise une phrase d'étape (voir [`Step::pattern`]).
pub(crate) fn normalize_pattern(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text.trim();
    while let Some(start) = rest.find('"') {
        let after = &rest[start + 1..];
        match after.find('"') {
            Some(end) => {
                out.push_str(&rest[..start]);
                out.push_str("{}");
                rest = &after[end + 1..];
            }
            None => break,
        }
    }
    out.push_str(rest);
    loop {
        let mut collapsed = out.clone();
        for pair in ["{}, {}", "{…}, {}", "{}, {…}", "{…}, {…}"] {
            collapsed = collapsed.replace(pair, "{…}");
        }
        if collapsed == out {
            return out;
        }
        out = collapsed;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"# Ordonnancement

Texte de contexte.

## Deux systèmes qui écrivent le même composant ne sont jamais dans la même étape
Exigences : ORD-002, ORD-003
* Le système "Movement" écrit "Position"
* Le plan est :

  | étape | systèmes      |
  |-------|---------------|
  | 1     | Input, Audio  |
  | 2     | Movement      |

* Calculer le plan d'exécution

## Second scénario
Exemples : basique
* L'ordre de référence est "A", "B", "C"

```markdown
## Ceci n'est pas un scénario
* ni une étape
```
"#;

    #[test]
    fn parse_titre_contexte_scenarios_et_etapes() {
        let spec = parse(SAMPLE).unwrap();
        assert_eq!(spec.title, "Ordonnancement");
        assert_eq!(spec.description, "Texte de contexte.");
        assert_eq!(spec.scenarios.len(), 2);

        let first = &spec.scenarios[0];
        assert_eq!(first.line, 5);
        assert_eq!(first.tag("Exigences"), ["ORD-002", "ORD-003"]);
        assert_eq!(first.steps.len(), 3);
        assert_eq!(first.steps[0].params, ["Movement", "Position"]);
        assert_eq!(first.steps[0].line, 7);

        let table = first.steps[1].table.as_ref().unwrap();
        assert_eq!(table.headers, ["étape", "systèmes"]);
        assert_eq!(table.rows.len(), 2);
        assert_eq!(table.cell(0, "systèmes"), Some("Input, Audio"));
        assert!(first.steps[2].table.is_none());

        assert_eq!(spec.scenarios[1].tag("Exemples"), ["basique"]);
    }

    #[test]
    fn les_blocs_de_code_sont_ignores() {
        let spec = parse(SAMPLE).unwrap();
        assert_eq!(spec.scenarios[1].steps.len(), 1);
    }

    #[test]
    fn normalisation_des_phrases() {
        assert_eq!(
            normalize_pattern(r#"Le système "Movement" écrit "Position""#),
            "Le système {} écrit {}"
        );
        assert_eq!(
            normalize_pattern(r#"L'ordre de référence est "A", "B", "C", "D""#),
            "L'ordre de référence est {…}"
        );
        for count in 2..=7 {
            let values: Vec<String> = (0..count).map(|i| format!("\"v{i}\"")).collect();
            assert_eq!(
                normalize_pattern(&format!("Liste {}", values.join(", "))),
                "Liste {…}"
            );
        }
        assert_eq!(
            normalize_pattern(r#"Les fichiers "a", "b", "c" et "d""#),
            "Les fichiers {…} et {}"
        );
        assert_eq!(
            normalize_pattern(r#""Movement" et "Knockback" sont dans des étapes différentes"#),
            "{} et {} sont dans des étapes différentes"
        );
        assert_eq!(normalize_pattern("Calculer le plan"), "Calculer le plan");
    }

    #[test]
    fn erreurs_de_structure() {
        assert!(parse("## Scénario sans titre\n").is_err());
        assert_eq!(parse("# T\n* étape orpheline\n").unwrap_err().line, 2);
        assert!(parse("# T\n## S\n| a |\n").is_err());
        assert!(parse("# T\n## S\n* x\nExigences : ORD-001\n").is_err());
        assert!(parse("# A\n# B\n").is_err());
        assert!(parse("# T\n```\n").is_err());
        let err = parse("# T\n## S\n* x\n\n  | a | b |\n  | 1 |\n").unwrap_err();
        assert_eq!(err.line, 6);
    }
}
