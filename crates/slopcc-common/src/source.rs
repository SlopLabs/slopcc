use std::path::{
  Path,
  PathBuf,
};

use crate::span::Span;

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct FileId(u32);

impl FileId {
  #[must_use]
  pub fn as_u32(self) -> u32 {
    self.0
  }

  #[must_use]
  pub fn new_for_tests(raw: u32) -> Self {
    Self(raw)
  }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct LineCol {
  pub line: u32,
  pub column: u32,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum SourceName<'a> {
  Path(&'a Path),
  Stdin,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct ResolvedSpan<'a> {
  pub source_name: SourceName<'a>,
  pub line: u32,
  pub column: u32,
  pub length: u32,
}

pub struct SourceFile {
  id: FileId,
  path: Option<PathBuf>,
  bytes: Box<[u8]>,
  line_starts: Box<[u32]>,
}

impl SourceFile {
  #[must_use]
  pub fn id(&self) -> FileId {
    self.id
  }

  #[must_use]
  pub fn path(&self) -> Option<&Path> {
    self.path.as_deref()
  }

  #[must_use]
  pub fn bytes(&self) -> &[u8] {
    &self.bytes
  }

  #[must_use]
  pub fn line_col(&self, byte_offset: u32) -> LineCol {
    if self.bytes.is_empty() {
      return LineCol { line: 1, column: 1 };
    }

    let max_offset = u32::try_from(self.bytes.len()).unwrap_or(u32::MAX);
    let clamped = byte_offset.min(max_offset);

    let line_index = match self.line_starts.binary_search(&clamped) {
      Ok(idx) => idx,
      Err(idx) => idx.saturating_sub(1),
    };

    let line_start = self.line_starts[line_index];
    let line = match u32::try_from(line_index) {
      Ok(raw) => raw.saturating_add(1),
      Err(_) => u32::MAX,
    };
    let column = clamped.saturating_sub(line_start).saturating_add(1);

    LineCol { line, column }
  }
}

pub struct SourceMap {
  files: Vec<SourceFile>,
}

impl Default for SourceMap {
  fn default() -> Self {
    Self::new()
  }
}

impl SourceMap {
  #[must_use]
  pub fn new() -> Self {
    Self { files: Vec::new() }
  }

  pub fn add_file(&mut self, path: PathBuf, bytes: Vec<u8>) -> FileId {
    self.add_internal(Some(path), bytes)
  }

  pub fn add_stdin(&mut self, bytes: Vec<u8>) -> FileId {
    self.add_internal(None, bytes)
  }

  pub fn add_file_from_path(&mut self, path: &Path) -> Result<FileId, SourceError> {
    let bytes = std::fs::read(path).map_err(|source| SourceError::ReadFile {
      path: path.to_path_buf(),
      source,
    })?;
    Ok(self.add_file(path.to_path_buf(), bytes))
  }

  #[must_use]
  pub fn file(&self, id: FileId) -> &SourceFile {
    let idx = match usize::try_from(id.0) {
      Ok(v) => v,
      Err(_) => panic!("invalid file id {}", id.0),
    };
    &self.files[idx]
  }

  #[must_use]
  pub fn resolve_span(&self, span: Span) -> ResolvedSpan<'_> {
    let file = self.file(span.file());
    let loc = file.line_col(span.start());
    let source_name = match file.path() {
      Some(path) => SourceName::Path(path),
      None => SourceName::Stdin,
    };

    ResolvedSpan {
      source_name,
      line: loc.line,
      column: loc.column,
      length: span.len(),
    }
  }

  fn add_internal(&mut self, path: Option<PathBuf>, bytes: Vec<u8>) -> FileId {
    let next = match u32::try_from(self.files.len()) {
      Ok(raw) => raw,
      Err(_) => panic!("too many source files"),
    };

    let id = FileId(next);
    let line_starts = compute_line_starts(&bytes);

    self.files.push(SourceFile {
      id,
      path,
      bytes: bytes.into_boxed_slice(),
      line_starts: line_starts.into_boxed_slice(),
    });

    id
  }
}

#[derive(thiserror::Error, Debug)]
pub enum SourceError {
  #[error("failed to read source file '{path}': {source}")]
  ReadFile {
    path: PathBuf,
    source: std::io::Error,
  },
}

fn compute_line_starts(bytes: &[u8]) -> Vec<u32> {
  let mut starts = vec![0];

  for (idx, byte) in bytes.iter().enumerate() {
    if *byte != b'\n' {
      continue;
    }

    let next = match u32::try_from(idx.saturating_add(1)) {
      Ok(v) => v,
      Err(_) => break,
    };
    starts.push(next);
  }

  starts
}

#[cfg(test)]
mod tests {
  use super::{
    SourceMap,
    SourceName,
  };
  use crate::span::Span;

  #[test]
  fn line_col_resolves_start_of_file() {
    let mut map = SourceMap::new();
    let file = map.add_stdin(b"abc\nxy".to_vec());
    let loc = map.file(file).line_col(0);
    assert_eq!(loc.line, 1);
    assert_eq!(loc.column, 1);
  }

  #[test]
  fn line_col_resolves_after_newline() {
    let mut map = SourceMap::new();
    let file = map.add_stdin(b"ab\ncd\n".to_vec());
    let loc = map.file(file).line_col(3);
    assert_eq!(loc.line, 2);
    assert_eq!(loc.column, 1);
  }

  #[test]
  fn line_col_handles_missing_final_newline() {
    let mut map = SourceMap::new();
    let file = map.add_stdin(b"ab\ncd".to_vec());
    let loc = map.file(file).line_col(4);
    assert_eq!(loc.line, 2);
    assert_eq!(loc.column, 2);
  }

  #[test]
  fn line_col_tolerates_crlf_as_byte_offsets() {
    let mut map = SourceMap::new();
    let file = map.add_stdin(b"a\r\nb".to_vec());
    let loc = map.file(file).line_col(3);
    assert_eq!(loc.line, 2);
    assert_eq!(loc.column, 1);
  }

  #[test]
  fn resolve_span_uses_source_name_and_location() {
    let mut map = SourceMap::new();
    let file = map.add_stdin(b"abc\ndef".to_vec());
    let span = Span::new(file, 4, 7);
    let resolved = map.resolve_span(span);
    assert_eq!(resolved.source_name, SourceName::Stdin);
    assert_eq!(resolved.line, 2);
    assert_eq!(resolved.column, 1);
    assert_eq!(resolved.length, 3);
  }
}
