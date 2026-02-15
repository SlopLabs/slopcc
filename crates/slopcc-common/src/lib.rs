/// Byte position in source code. Supports files up to 4 GiB.
pub type BytePos = u32;

/// A contiguous byte range `[lo, hi)` in source code.
///
/// Spans are file-agnostic — pair with [`FileId`] when you need to identify
/// which file a span belongs to.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Span {
  pub lo: BytePos,
  pub hi: BytePos,
}

impl Span {
  /// Create a span covering `[lo, hi)`.
  #[must_use]
  pub const fn new(lo: BytePos, hi: BytePos) -> Self {
    Self { lo, hi }
  }

  /// A zero-width span at the given position.
  #[must_use]
  pub const fn at(pos: BytePos) -> Self {
    Self { lo: pos, hi: pos }
  }

  /// Length in bytes.
  #[must_use]
  pub const fn len(&self) -> u32 {
    self.hi - self.lo
  }

  /// Whether this span is zero-width.
  #[must_use]
  pub const fn is_empty(&self) -> bool {
    self.lo == self.hi
  }

  /// Merge two spans into one covering both. The spans need not be adjacent
  /// or ordered.
  #[must_use]
  pub const fn merge(self, other: Self) -> Self {
    Self {
      lo: if self.lo < other.lo {
        self.lo
      } else {
        other.lo
      },
      hi: if self.hi > other.hi {
        self.hi
      } else {
        other.hi
      },
    }
  }

  /// Extract the source text this span refers to from `src`.
  #[must_use]
  pub fn as_str<'a>(&self, src: &'a [u8]) -> &'a [u8] {
    &src[self.lo as usize..self.hi as usize]
  }
}

/// Opaque identifier for a source file in the compilation.
///
/// Indexes into whatever source-map structure the driver maintains.
/// Cheap to copy and compare.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FileId(pub u32);

impl FileId {
  /// Create a new file identifier.
  #[must_use]
  pub const fn new(id: u32) -> Self {
    Self(id)
  }

  /// The raw index.
  #[must_use]
  pub const fn as_u32(self) -> u32 {
    self.0
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn span_new() {
    let s = Span::new(10, 20);
    assert_eq!(s.lo, 10);
    assert_eq!(s.hi, 20);
    assert_eq!(s.len(), 10);
    assert!(!s.is_empty());
  }

  #[test]
  fn span_at() {
    let s = Span::at(5);
    assert_eq!(s.lo, 5);
    assert_eq!(s.hi, 5);
    assert!(s.is_empty());
    assert_eq!(s.len(), 0);
  }

  #[test]
  fn span_merge() {
    let a = Span::new(5, 10);
    let b = Span::new(8, 15);
    let m = a.merge(b);
    assert_eq!(m.lo, 5);
    assert_eq!(m.hi, 15);
  }

  #[test]
  fn span_merge_reversed() {
    let a = Span::new(8, 15);
    let b = Span::new(5, 10);
    let m = a.merge(b);
    assert_eq!(m.lo, 5);
    assert_eq!(m.hi, 15);
  }

  #[test]
  fn span_as_str() {
    let src = b"int main() {}";
    let s = Span::new(4, 8);
    assert_eq!(s.as_str(src), b"main");
  }

  #[test]
  fn file_id_roundtrip() {
    let id = FileId::new(42);
    assert_eq!(id.as_u32(), 42);
    assert_eq!(id, FileId(42));
  }
}
