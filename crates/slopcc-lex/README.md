# slopcc-lex

Lexer / tokenizer for C source code.

## Purpose

Converts a byte stream of C source into a token stream. Handles all C89/C99/C11
token types including keywords, identifiers, numeric and string literals, operators,
and punctuation.

## Points of Interest

- `src/lib.rs` — currently empty, awaiting implementation.

## Public API

Not yet implemented. Planned:
- `Token` type with span information
- `TokenKind` enum covering all C token types
- `Lexer` iterator that yields tokens from source bytes

## Dependencies

- `slopcc-common` — for `Span`, `FileId`, diagnostics

## Status

Scaffolded. No lexer logic implemented yet.
