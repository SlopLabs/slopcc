pub(crate) struct Cursor<'src> {
  bytes: &'src [u8],
  pos: usize,
}

impl<'src> Cursor<'src> {
  pub(crate) const fn new(src: &'src [u8]) -> Self {
    Self { bytes: src, pos: 0 }
  }

  pub(crate) const fn pos(&self) -> usize {
    self.pos
  }

  pub(crate) fn is_eof(&self) -> bool {
    self.pos >= self.bytes.len()
  }

  pub(crate) fn peek(&self) -> Option<u8> {
    self.bytes.get(self.pos).copied()
  }

  pub(crate) fn peek_next(&self) -> Option<u8> {
    self.bytes.get(self.pos + 1).copied()
  }

  pub(crate) fn advance(&mut self) -> Option<u8> {
    let byte = self.peek()?;
    self.pos += 1;
    Some(byte)
  }

  pub(crate) fn eat(&mut self, byte: u8) -> bool {
    if self.peek() == Some(byte) {
      self.pos += 1;
      true
    } else {
      false
    }
  }

  pub(crate) fn eat_while(&mut self, mut pred: impl FnMut(u8) -> bool) {
    while let Some(byte) = self.peek() {
      if !pred(byte) {
        break;
      }
      self.pos += 1;
    }
  }
}

#[cfg(test)]
mod tests {
  use super::Cursor;

  #[test]
  fn peek_advance_and_eat() {
    let mut cursor = Cursor::new(b"ab");
    assert_eq!(cursor.peek(), Some(b'a'));
    assert_eq!(cursor.peek_next(), Some(b'b'));
    assert_eq!(cursor.advance(), Some(b'a'));
    assert!(cursor.eat(b'b'));
    assert_eq!(cursor.advance(), None);
  }

  #[test]
  fn eat_while_consumes_matching_prefix() {
    let mut cursor = Cursor::new(b"123abc");
    cursor.eat_while(|b| b.is_ascii_digit());
    assert_eq!(cursor.pos(), 3);
    assert_eq!(cursor.peek(), Some(b'a'));
  }

  #[test]
  fn eof_behavior() {
    let mut cursor = Cursor::new(b"");
    assert!(cursor.is_eof());
    assert_eq!(cursor.peek(), None);
    assert_eq!(cursor.peek_next(), None);
    assert_eq!(cursor.advance(), None);
    assert!(!cursor.eat(b'x'));
  }
}
