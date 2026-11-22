use std::fmt;
use std::path::PathBuf;

use serde::Serialize;

/// Niveau d'un diagnostic (VAL-006).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Level {
    Error,
    Warning,
}

/// Un diagnostic : niveau, message, élément du modèle, fichier et ligne (VAL-006).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Diagnostic {
    pub level: Level,
    /// Identifiant de l'exigence qui définit ce diagnostic (`VAL-001`, `ORD-007`…).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub element: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<usize>,
}

impl Diagnostic {
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            level: Level::Error,
            code: None,
            message: message.into(),
            element: None,
            file: None,
            line: None,
        }
    }

    pub fn warning(message: impl Into<String>) -> Self {
        Self {
            level: Level::Warning,
            ..Self::error(message)
        }
    }

    pub fn code(mut self, code: &str) -> Self {
        self.code = Some(code.to_string());
        self
    }

    pub fn element(mut self, element: impl Into<String>) -> Self {
        self.element = Some(element.into());
        self
    }

    pub fn at(mut self, file: impl Into<PathBuf>, line: Option<usize>) -> Self {
        self.file = Some(file.into());
        self.line = line;
        self
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let level = match self.level {
            Level::Error => "erreur",
            Level::Warning => "avertissement",
        };
        write!(f, "{level}")?;
        if let Some(file) = &self.file {
            write!(f, " [{}", file.display())?;
            if let Some(line) = self.line {
                write!(f, ":{line}")?;
            }
            write!(f, "]")?;
        }
        write!(f, " : {}", self.message)
    }
}
