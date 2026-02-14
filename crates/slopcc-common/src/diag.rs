use crate::span::Span;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Severity {
  Error,
  Warning,
  Note,
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Diagnostic {
  pub severity: Severity,
  pub message: String,
  pub span: Option<Span>,
}

#[derive(Default, Clone, Eq, PartialEq, Debug)]
pub struct Diagnostics {
  items: Vec<Diagnostic>,
}

impl Diagnostics {
  #[must_use]
  pub fn new() -> Self {
    Self { items: Vec::new() }
  }

  pub fn push(&mut self, diagnostic: Diagnostic) {
    self.items.push(diagnostic);
  }

  #[must_use]
  pub fn has_errors(&self) -> bool {
    self
      .items
      .iter()
      .any(|diagnostic| diagnostic.severity == Severity::Error)
  }

  pub fn iter(&self) -> impl Iterator<Item = &Diagnostic> {
    self.items.iter()
  }

  #[must_use]
  pub fn len(&self) -> usize {
    self.items.len()
  }

  #[must_use]
  pub fn is_empty(&self) -> bool {
    self.items.is_empty()
  }
}

#[cfg(test)]
mod tests {
  use super::{
    Diagnostic,
    Diagnostics,
    Severity,
  };

  #[test]
  fn has_errors_tracks_error_severity() {
    let mut diagnostics = Diagnostics::new();
    diagnostics.push(Diagnostic {
      severity: Severity::Warning,
      message: String::from("warn"),
      span: None,
    });
    assert!(!diagnostics.has_errors());

    diagnostics.push(Diagnostic {
      severity: Severity::Error,
      message: String::from("err"),
      span: None,
    });
    assert!(diagnostics.has_errors());
  }
}
