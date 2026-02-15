# slopcc-lex

Preprocessing-token lexer for C source input.

## Purpose

Scans source bytes and emits preprocessing tokens (C11 §6.4). The output feeds
directly into the preprocessor. After preprocessing, pp-tokens are converted to
final C tokens (keyword recognition, numeric validation, etc.) — that conversion
is not part of this crate.

## Points of Interest

- `src/token.rs` — `Token` struct and `TokenKind` enum covering the full C11
  preprocessing token set: pp-numbers, string/char literals, identifiers,
  all punctuators, whitespace, newlines, comments, header names.
- `src/cursor.rs` — low-level byte cursor with peek/advance/eat operations.
- `src/lexer.rs` — main lexer engine. Handles string/char prefix disambiguation
  (L, u, U, u8), greedy pp-number scanning with exponent signs, multi-byte
  punctuator disambiguation, and separate header-name lexing for `#include`.
- `src/lib.rs` — module wiring and public re-exports.
- No keyword recognition — all identifier-like tokens are `Ident`.
- No numeric validation — `PpNumber` is intentionally loose per C11 §6.4.8.
- Newlines are distinct from whitespace (preprocessor is line-oriented).

## Public API

```rust
Lexer::new(src: &[u8], file: FileId) -> Lexer
Lexer::next_token(&mut self) -> Token
Lexer::tokenize(src: &[u8], file: FileId) -> Vec<Token>
Lexer::lex_header_name(&mut self) -> Token

Token { kind: TokenKind, span: Span }
Token::new(kind, span) -> Token
```

`TokenKind` variants: `PpNumber`, `CharConst`, `StringLiteral`, `Ident`,
`HeaderName`, `Hash`, `HashHash`, `LParen`, `RParen`, `LBracket`, `RBracket`,
`LBrace`, `RBrace`, `Comma`, `Semi`, `Colon`, `Ellipsis`, `Dot`, `Arrow`,
`Plus`, `Minus`, `Star`, `Slash`, `Percent`, `PlusPlus`, `MinusMinus`,
`Eq`, `Ne`, `Lt`, `Gt`, `Le`, `Ge`, `And`, `Or`, `Not`, `Amp`, `Pipe`,
`Caret`, `Tilde`, `Shl`, `Shr`, `Assign`, `PlusAssign`, `MinusAssign`,
`StarAssign`, `SlashAssign`, `PercentAssign`, `AmpAssign`, `PipeAssign`,
`CaretAssign`, `ShlAssign`, `ShrAssign`, `Question`, `Whitespace`, `Newline`,
`Comment`, `Eof`, `Unknown`.

## Dependencies

- `slopcc-common` — `Span`, `FileId` for source location tracking.

## Status

Implemented with 23 unit tests. Covers:
- Whitespace and newline handling (including `\r\n`)
- Line comments (`//`) and block comments (`/* */`), including unterminated
- Identifiers with string/char prefix fallback (L, u, U, u8)
- Greedy pp-number scanning with exponent signs (e/E/p/P ±)
- String literals and char constants with all prefix variants and escape sequences
- All C11 punctuators with multi-byte disambiguation
- Header name lexing (`<...>` and `"..."`)
- Unknown byte and empty input handling

Not yet implemented (deferred to future phases):
- Trigraph replacement (translation phase 1)
- Line splicing / backslash-newline (translation phase 2)
- Keyword recognition (post-preprocessing conversion)
- Numeric literal validation (post-preprocessing conversion)
- Diagnostic emission for lexer errors
