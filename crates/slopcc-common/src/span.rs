use crate::source::FileId;

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct Span {
  file: FileId,
  start: u32,
  end: u32,
}

impl Span {
  #[must_use]
  pub fn new(file: FileId, start: u32, end: u32) -> Self {
    assert!(start <= end, "span start must be <= end");
    Self { file, start, end }
  }

  #[must_use]
  pub fn at(file: FileId, offset: u32) -> Self {
    Self {
      file,
      start: offset,
      end: offset,
    }
  }

  #[must_use]
  pub fn file(self) -> FileId {
    self.file
  }

  #[must_use]
  pub fn start(self) -> u32 {
    self.start
  }

  #[must_use]
  pub fn end(self) -> u32 {
    self.end
  }

  #[must_use]
  pub fn len(self) -> u32 {
    self.end - self.start
  }

  #[must_use]
  pub fn is_empty(self) -> bool {
    self.start == self.end
  }
}

#[cfg(test)]
mod tests {
  use super::Span;
  use crate::source::FileId;

  #[test]
  fn span_len_is_half_open() {
    let span = Span::new(FileId::new_for_tests(0), 2, 5);
    assert_eq!(span.len(), 3);
  }

  #[test]
  fn zero_length_span_is_empty() {
    let span = Span::at(FileId::new_for_tests(1), 12);
    assert!(span.is_empty());
  }
}
