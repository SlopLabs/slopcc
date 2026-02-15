use slopcc_common::{
  BytePos,
  Span,
};

use crate::{
  cursor::Cursor,
  token::{
    Token,
    TokenKind,
  },
};

/// Preprocessing-token lexer for C source bytes.
pub struct Lexer<'src> {
  cursor: Cursor<'src>,
  src: &'src [u8],
}

impl<'src> Lexer<'src> {
  /// Creates a lexer over `src` bytes.
  #[must_use]
  pub fn new(src: &'src [u8]) -> Self {
    Self {
      cursor: Cursor::new(src),
      src,
    }
  }

  /// Returns the next preprocessing token.
  #[must_use]
  pub fn next_token(&mut self) -> Token {
    if self.cursor.is_eof() {
      let pos = self.byte_pos(self.cursor.pos());
      return Token::new(TokenKind::Eof, Span::at(pos));
    }

    let byte = self.cursor.peek().unwrap_or_default();

    if is_whitespace_no_newline(byte) {
      return self.whitespace();
    }

    if byte == b'\n' {
      let start = self.cursor.pos();
      let _ = self.cursor.advance();
      return self.make_token(start, TokenKind::Newline);
    }

    if byte == b'/' {
      if self.cursor.peek_next() == Some(b'/') {
        return self.line_comment();
      }
      if self.cursor.peek_next() == Some(b'*') {
        return self.block_comment();
      }
    }

    if byte.is_ascii_digit()
      || (byte == b'.' && self.cursor.peek_next().is_some_and(|b| b.is_ascii_digit()))
    {
      return self.pp_number();
    }

    if matches!(byte, b'L' | b'u' | b'U') {
      return self.ident_or_string_prefix();
    }

    if is_ident_start(byte) {
      return self.ident();
    }

    if byte == b'"' {
      let _ = self.cursor.advance();
      return self.string_literal(0);
    }

    if byte == b'\'' {
      let _ = self.cursor.advance();
      return self.char_const(0);
    }

    let start = self.cursor.pos();
    let first = self.cursor.advance().unwrap_or_default();
    self.punctuator(start, first)
  }

  /// Tokenizes all input and appends a terminal `Eof` token.
  #[must_use]
  pub fn tokenize(src: &'src [u8]) -> Vec<Token> {
    let mut lexer = Self::new(src);
    let mut out = Vec::new();
    loop {
      let token = lexer.next_token();
      out.push(token);
      if token.kind == TokenKind::Eof {
        break;
      }
    }
    out
  }

  /// Lexes a header-name token in include context.
  #[must_use]
  pub fn lex_header_name(&mut self) -> Token {
    let start = self.cursor.pos();
    match self.cursor.peek() {
      Some(b'<') => {
        let _ = self.cursor.advance();
        while let Some(byte) = self.cursor.peek() {
          if byte == b'>' {
            let _ = self.cursor.advance();
            return self.make_token(start, TokenKind::HeaderName);
          }
          if byte == b'\n' {
            break;
          }
          let _ = self.cursor.advance();
        }
        self.make_token(start, TokenKind::Unknown)
      }
      Some(b'"') => {
        let _ = self.cursor.advance();
        while let Some(byte) = self.cursor.peek() {
          if byte == b'"' {
            let _ = self.cursor.advance();
            return self.make_token(start, TokenKind::HeaderName);
          }
          if byte == b'\n' {
            break;
          }
          let _ = self.cursor.advance();
        }
        self.make_token(start, TokenKind::Unknown)
      }
      Some(_) => {
        let _ = self.cursor.advance();
        self.make_token(start, TokenKind::Unknown)
      }
      None => Token::new(TokenKind::Eof, Span::at(self.byte_pos(start))),
    }
  }

  fn whitespace(&mut self) -> Token {
    let start = self.cursor.pos();
    self.cursor.eat_while(is_whitespace_no_newline);
    self.make_token(start, TokenKind::Whitespace)
  }

  fn line_comment(&mut self) -> Token {
    let start = self.cursor.pos();
    let _ = self.cursor.advance();
    let _ = self.cursor.advance();
    self.cursor.eat_while(|byte| byte != b'\n');
    self.make_token(start, TokenKind::Comment)
  }

  fn block_comment(&mut self) -> Token {
    let start = self.cursor.pos();
    let _ = self.cursor.advance();
    let _ = self.cursor.advance();
    while let Some(byte) = self.cursor.advance() {
      if byte == b'*' && self.cursor.eat(b'/') {
        break;
      }
    }
    self.make_token(start, TokenKind::Comment)
  }

  fn ident_or_string_prefix(&mut self) -> Token {
    let start = self.cursor.pos();
    let first = self.cursor.advance().unwrap_or_default();

    match first {
      b'L' | b'U' => {
        if self.cursor.eat(b'"') {
          return self.string_literal(1);
        }
        if self.cursor.eat(b'\'') {
          return self.char_const(1);
        }
      }
      b'u' => {
        if self.cursor.eat(b'8') {
          if self.cursor.eat(b'"') {
            return self.string_literal(2);
          }
          self.cursor.eat_while(is_ident_continue);
          return self.make_token(start, TokenKind::Ident);
        }

        if self.cursor.eat(b'"') {
          return self.string_literal(1);
        }
        if self.cursor.eat(b'\'') {
          return self.char_const(1);
        }
      }
      _ => {}
    }

    self.cursor.eat_while(is_ident_continue);
    self.make_token(start, TokenKind::Ident)
  }

  fn ident(&mut self) -> Token {
    let start = self.cursor.pos();
    let _ = self.cursor.advance();
    self.cursor.eat_while(is_ident_continue);
    self.make_token(start, TokenKind::Ident)
  }

  fn pp_number(&mut self) -> Token {
    let start = self.cursor.pos();

    if self.cursor.peek() == Some(b'.') {
      let _ = self.cursor.advance();
    } else {
      let _ = self.cursor.advance();
    }

    loop {
      match self.cursor.peek() {
        Some(byte @ (b'e' | b'E' | b'p' | b'P'))
          if self
            .cursor
            .peek_next()
            .is_some_and(|next| next == b'+' || next == b'-') =>
        {
          let _ = self.cursor.advance();
          let _ = self.cursor.advance();
          let _ = byte;
        }
        Some(byte) if byte.is_ascii_digit() || is_ident_nondigit(byte) || byte == b'.' => {
          let _ = self.cursor.advance();
        }
        _ => break,
      }
    }

    self.make_token(start, TokenKind::PpNumber)
  }

  fn string_literal(&mut self, prefix_len: u32) -> Token {
    let start = self.cursor.pos().saturating_sub(prefix_len as usize + 1);

    while let Some(byte) = self.cursor.advance() {
      match byte {
        b'\\' => {
          let _ = self.cursor.advance();
        }
        b'"' => {
          return self.make_token(start, TokenKind::StringLiteral);
        }
        b'\n' => {
          return self.make_token(start, TokenKind::Unknown);
        }
        _ => {}
      }
    }

    self.make_token(start, TokenKind::Unknown)
  }

  fn char_const(&mut self, prefix_len: u32) -> Token {
    let start = self.cursor.pos().saturating_sub(prefix_len as usize + 1);

    while let Some(byte) = self.cursor.advance() {
      match byte {
        b'\\' => {
          let _ = self.cursor.advance();
        }
        b'\'' => {
          return self.make_token(start, TokenKind::CharConst);
        }
        b'\n' => {
          return self.make_token(start, TokenKind::Unknown);
        }
        _ => {}
      }
    }

    self.make_token(start, TokenKind::Unknown)
  }

  fn punctuator(&mut self, start: usize, first: u8) -> Token {
    let kind = match first {
      b'#' => {
        if self.cursor.eat(b'#') {
          TokenKind::HashHash
        } else {
          TokenKind::Hash
        }
      }
      b'(' => TokenKind::LParen,
      b')' => TokenKind::RParen,
      b'[' => TokenKind::LBracket,
      b']' => TokenKind::RBracket,
      b'{' => TokenKind::LBrace,
      b'}' => TokenKind::RBrace,
      b',' => TokenKind::Comma,
      b';' => TokenKind::Semi,
      b':' => TokenKind::Colon,
      b'.' => {
        if self.cursor.eat(b'.') && self.cursor.eat(b'.') {
          TokenKind::Ellipsis
        } else {
          TokenKind::Dot
        }
      }
      b'?' => TokenKind::Question,
      b'+' => {
        if self.cursor.eat(b'+') {
          TokenKind::PlusPlus
        } else if self.cursor.eat(b'=') {
          TokenKind::PlusAssign
        } else {
          TokenKind::Plus
        }
      }
      b'-' => {
        if self.cursor.eat(b'-') {
          TokenKind::MinusMinus
        } else if self.cursor.eat(b'>') {
          TokenKind::Arrow
        } else if self.cursor.eat(b'=') {
          TokenKind::MinusAssign
        } else {
          TokenKind::Minus
        }
      }
      b'*' => {
        if self.cursor.eat(b'=') {
          TokenKind::StarAssign
        } else {
          TokenKind::Star
        }
      }
      b'/' => {
        if self.cursor.eat(b'=') {
          TokenKind::SlashAssign
        } else {
          TokenKind::Slash
        }
      }
      b'%' => {
        if self.cursor.eat(b'=') {
          TokenKind::PercentAssign
        } else {
          TokenKind::Percent
        }
      }
      b'=' => {
        if self.cursor.eat(b'=') {
          TokenKind::Eq
        } else {
          TokenKind::Assign
        }
      }
      b'!' => {
        if self.cursor.eat(b'=') {
          TokenKind::Ne
        } else {
          TokenKind::Not
        }
      }
      b'<' => {
        if self.cursor.eat(b'<') {
          if self.cursor.eat(b'=') {
            TokenKind::ShlAssign
          } else {
            TokenKind::Shl
          }
        } else if self.cursor.eat(b'=') {
          TokenKind::Le
        } else {
          TokenKind::Lt
        }
      }
      b'>' => {
        if self.cursor.eat(b'>') {
          if self.cursor.eat(b'=') {
            TokenKind::ShrAssign
          } else {
            TokenKind::Shr
          }
        } else if self.cursor.eat(b'=') {
          TokenKind::Ge
        } else {
          TokenKind::Gt
        }
      }
      b'&' => {
        if self.cursor.eat(b'&') {
          TokenKind::And
        } else if self.cursor.eat(b'=') {
          TokenKind::AmpAssign
        } else {
          TokenKind::Amp
        }
      }
      b'|' => {
        if self.cursor.eat(b'|') {
          TokenKind::Or
        } else if self.cursor.eat(b'=') {
          TokenKind::PipeAssign
        } else {
          TokenKind::Pipe
        }
      }
      b'^' => {
        if self.cursor.eat(b'=') {
          TokenKind::CaretAssign
        } else {
          TokenKind::Caret
        }
      }
      b'~' => TokenKind::Tilde,
      _ => TokenKind::Unknown,
    };

    self.make_token(start, kind)
  }

  fn make_token(&self, start: usize, kind: TokenKind) -> Token {
    debug_assert!(self.cursor.pos() <= self.src.len());
    Token::new(
      kind,
      Span::new(self.byte_pos(start), self.byte_pos(self.cursor.pos())),
    )
  }

  fn byte_pos(&self, pos: usize) -> BytePos {
    debug_assert!(pos <= BytePos::MAX as usize);
    pos as BytePos
  }

  #[cfg(test)]
  fn slice(&self, token: Token) -> &'src [u8] {
    token.span.as_str(self.src)
  }
}

fn is_whitespace_no_newline(byte: u8) -> bool {
  matches!(byte, b' ' | b'\t' | b'\r' | 0x0B | 0x0C)
}

fn is_ident_start(byte: u8) -> bool {
  byte.is_ascii_alphabetic() || byte == b'_'
}

fn is_ident_nondigit(byte: u8) -> bool {
  is_ident_start(byte)
}

fn is_ident_continue(byte: u8) -> bool {
  byte.is_ascii_alphanumeric() || byte == b'_'
}

#[cfg(test)]
mod tests {
  use super::Lexer;
  use crate::TokenKind;
  use slopcc_common::Span;

  fn kinds(src: &[u8]) -> Vec<TokenKind> {
    Lexer::tokenize(src)
      .into_iter()
      .map(|token| token.kind)
      .collect()
  }

  #[test]
  fn lexes_whitespace_with_span() {
    let mut lexer = Lexer::new(b" \t\r\x0B\x0C");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::Whitespace);
    assert_eq!(token.span, Span::new(0, 5));
    assert_eq!(lexer.next_token().kind, TokenKind::Eof);
  }

  #[test]
  fn handles_newline_and_crlf() {
    let tokens = Lexer::tokenize(b"\n\r\n");
    let kinds: Vec<_> = tokens.iter().map(|token| token.kind).collect();
    assert_eq!(
      kinds,
      vec![
        TokenKind::Newline,
        TokenKind::Whitespace,
        TokenKind::Newline,
        TokenKind::Eof,
      ]
    );
    assert_eq!(tokens[0].span, Span::new(0, 1));
    assert_eq!(tokens[1].span, Span::new(1, 2));
    assert_eq!(tokens[2].span, Span::new(2, 3));
  }

  #[test]
  fn lexes_comments() {
    let tokens = Lexer::tokenize(b"// x\n/* y */");
    let kinds: Vec<_> = tokens.iter().map(|token| token.kind).collect();
    assert_eq!(
      kinds,
      vec![
        TokenKind::Comment,
        TokenKind::Newline,
        TokenKind::Comment,
        TokenKind::Eof,
      ]
    );
  }

  #[test]
  fn lexes_unterminated_block_comment_as_comment() {
    let tokens = Lexer::tokenize(b"/* not closed");
    assert_eq!(tokens[0].kind, TokenKind::Comment);
    assert_eq!(tokens[0].span, Span::new(0, 13));
    assert_eq!(tokens[1].kind, TokenKind::Eof);
  }

  #[test]
  fn lexes_identifiers() {
    let tokens = Lexer::tokenize(b"foo _bar x123");
    let kinds: Vec<_> = tokens.iter().map(|token| token.kind).collect();
    assert_eq!(
      kinds,
      vec![
        TokenKind::Ident,
        TokenKind::Whitespace,
        TokenKind::Ident,
        TokenKind::Whitespace,
        TokenKind::Ident,
        TokenKind::Eof,
      ]
    );
  }

  #[test]
  fn lexes_pp_numbers() {
    let src = b"42 3.14 0xFF 1e10 0x1p+3 .5 1.0f 100ULL";
    let tokens = Lexer::tokenize(src);
    assert_eq!(tokens[0].kind, TokenKind::PpNumber);
    assert_eq!(tokens[2].kind, TokenKind::PpNumber);
    assert_eq!(tokens[4].kind, TokenKind::PpNumber);
    assert_eq!(tokens[6].kind, TokenKind::PpNumber);
    assert_eq!(tokens[8].kind, TokenKind::PpNumber);
    assert_eq!(tokens[10].kind, TokenKind::PpNumber);
    assert_eq!(tokens[12].kind, TokenKind::PpNumber);
    assert_eq!(tokens[14].kind, TokenKind::PpNumber);
  }

  #[test]
  fn lexes_string_literals_and_prefixes() {
    let src = b"\"hello\" \"with \\\"escape\\\"\" L\"wide\" u8\"utf8\" u\"utf16\" U\"utf32\" \"\"";
    let tokens = Lexer::tokenize(src);
    assert_eq!(tokens[0].kind, TokenKind::StringLiteral);
    assert_eq!(tokens[2].kind, TokenKind::StringLiteral);
    assert_eq!(tokens[4].kind, TokenKind::StringLiteral);
    assert_eq!(tokens[6].kind, TokenKind::StringLiteral);
    assert_eq!(tokens[8].kind, TokenKind::StringLiteral);
    assert_eq!(tokens[10].kind, TokenKind::StringLiteral);
    assert_eq!(tokens[12].kind, TokenKind::StringLiteral);
  }

  #[test]
  fn lexes_char_constants_and_prefixes() {
    let src = b"'a' '\\n' L'x' u'y' U'z'";
    let tokens = Lexer::tokenize(src);
    assert_eq!(tokens[0].kind, TokenKind::CharConst);
    assert_eq!(tokens[2].kind, TokenKind::CharConst);
    assert_eq!(tokens[4].kind, TokenKind::CharConst);
    assert_eq!(tokens[6].kind, TokenKind::CharConst);
    assert_eq!(tokens[8].kind, TokenKind::CharConst);
  }

  #[test]
  fn lexes_all_punctuators() {
    let src = b"# ## ( ) [ ] { } , ; : ... . -> + - * / % ++ -- == != < > <= >= && || ! & | ^ ~ << >> = += -= *= /= %= &= |= ^= <<= >>= ?";
    let expected = vec![
      TokenKind::Hash,
      TokenKind::HashHash,
      TokenKind::LParen,
      TokenKind::RParen,
      TokenKind::LBracket,
      TokenKind::RBracket,
      TokenKind::LBrace,
      TokenKind::RBrace,
      TokenKind::Comma,
      TokenKind::Semi,
      TokenKind::Colon,
      TokenKind::Ellipsis,
      TokenKind::Dot,
      TokenKind::Arrow,
      TokenKind::Plus,
      TokenKind::Minus,
      TokenKind::Star,
      TokenKind::Slash,
      TokenKind::Percent,
      TokenKind::PlusPlus,
      TokenKind::MinusMinus,
      TokenKind::Eq,
      TokenKind::Ne,
      TokenKind::Lt,
      TokenKind::Gt,
      TokenKind::Le,
      TokenKind::Ge,
      TokenKind::And,
      TokenKind::Or,
      TokenKind::Not,
      TokenKind::Amp,
      TokenKind::Pipe,
      TokenKind::Caret,
      TokenKind::Tilde,
      TokenKind::Shl,
      TokenKind::Shr,
      TokenKind::Assign,
      TokenKind::PlusAssign,
      TokenKind::MinusAssign,
      TokenKind::StarAssign,
      TokenKind::SlashAssign,
      TokenKind::PercentAssign,
      TokenKind::AmpAssign,
      TokenKind::PipeAssign,
      TokenKind::CaretAssign,
      TokenKind::ShlAssign,
      TokenKind::ShrAssign,
      TokenKind::Question,
      TokenKind::Eof,
    ];
    let filtered: Vec<_> = kinds(src)
      .into_iter()
      .filter(|kind| *kind != TokenKind::Whitespace)
      .collect();
    assert_eq!(filtered, expected);
  }

  #[test]
  fn disambiguates_multi_byte_punctuators() {
    assert_eq!(
      kinds(b"++ + +"),
      vec![
        TokenKind::PlusPlus,
        TokenKind::Whitespace,
        TokenKind::Plus,
        TokenKind::Whitespace,
        TokenKind::Plus,
        TokenKind::Eof,
      ]
    );
    assert_eq!(
      kinds(b"<< < < <<= -> ..."),
      vec![
        TokenKind::Shl,
        TokenKind::Whitespace,
        TokenKind::Lt,
        TokenKind::Whitespace,
        TokenKind::Lt,
        TokenKind::Whitespace,
        TokenKind::ShlAssign,
        TokenKind::Whitespace,
        TokenKind::Arrow,
        TokenKind::Whitespace,
        TokenKind::Ellipsis,
        TokenKind::Eof,
      ]
    );
  }

  #[test]
  fn lexes_header_names() {
    let mut angle = Lexer::new(b"<stdio.h>");
    assert_eq!(angle.lex_header_name().kind, TokenKind::HeaderName);

    let mut quote = Lexer::new(b"\"myheader.h\"");
    assert_eq!(quote.lex_header_name().kind, TokenKind::HeaderName);
  }

  #[test]
  fn lexes_full_stream_with_spans() {
    let src = b"int main() { return 0; }";
    let tokens = Lexer::tokenize(src);
    let kinds: Vec<_> = tokens.iter().map(|token| token.kind).collect();
    assert_eq!(
      kinds,
      vec![
        TokenKind::Ident,
        TokenKind::Whitespace,
        TokenKind::Ident,
        TokenKind::LParen,
        TokenKind::RParen,
        TokenKind::Whitespace,
        TokenKind::LBrace,
        TokenKind::Whitespace,
        TokenKind::Ident,
        TokenKind::Whitespace,
        TokenKind::PpNumber,
        TokenKind::Semi,
        TokenKind::Whitespace,
        TokenKind::RBrace,
        TokenKind::Eof,
      ]
    );
    assert_eq!(tokens[0].span, Span::new(0, 3));
    assert_eq!(tokens[2].span, Span::new(4, 8));
    assert_eq!(tokens[10].span, Span::new(20, 21));
  }

  #[test]
  fn lexes_unknown_bytes() {
    assert_eq!(
      kinds(b"@$"),
      vec![TokenKind::Unknown, TokenKind::Unknown, TokenKind::Eof]
    );
  }

  #[test]
  fn lexes_empty_input() {
    assert_eq!(kinds(b""), vec![TokenKind::Eof]);
  }

  #[test]
  fn lexes_define_like_line() {
    let tokens = Lexer::tokenize(b"#define FOO 42\n");
    let kinds: Vec<_> = tokens.iter().map(|token| token.kind).collect();
    assert_eq!(
      kinds,
      vec![
        TokenKind::Hash,
        TokenKind::Ident,
        TokenKind::Whitespace,
        TokenKind::Ident,
        TokenKind::Whitespace,
        TokenKind::PpNumber,
        TokenKind::Newline,
        TokenKind::Eof,
      ]
    );
  }

  #[test]
  fn prefixes_fall_back_to_ident_when_not_literal_prefix() {
    assert_eq!(
      kinds(b"u8ident Ufoo Lbar uabc"),
      vec![
        TokenKind::Ident,
        TokenKind::Whitespace,
        TokenKind::Ident,
        TokenKind::Whitespace,
        TokenKind::Ident,
        TokenKind::Whitespace,
        TokenKind::Ident,
        TokenKind::Eof,
      ]
    );
  }

  #[test]
  fn unknown_for_unterminated_string_and_char() {
    assert_eq!(kinds(b"\"abc"), vec![TokenKind::Unknown, TokenKind::Eof]);
    assert_eq!(kinds(b"'x"), vec![TokenKind::Unknown, TokenKind::Eof]);
  }

  #[test]
  fn pp_numbers_greedy_sign_exponents() {
    let mut lexer = Lexer::new(b"0x1p+3 1e-2");
    let first = lexer.next_token();
    assert_eq!(first.kind, TokenKind::PpNumber);
    assert_eq!(lexer.slice(first), b"0x1p+3");

    let _ = lexer.next_token();

    let second = lexer.next_token();
    assert_eq!(second.kind, TokenKind::PpNumber);
    assert_eq!(lexer.slice(second), b"1e-2");
  }
}
