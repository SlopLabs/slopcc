# slopcc-lex

Tokenizer crate for C source input.

## Purpose

Defines the single tokenizer stage for slopcc. It scans source bytes and emits a
general token stream suitable as the input boundary for preprocessing and later parser
token cleanup.

## Points of Interest

- `src/lib.rs` — crate entry point (still intentionally empty).
- Tokenization model: one tokenizer type (`Tokenizer`) for all source scanning.
- Pipeline intent: source -> tokenizer output -> preprocessor -> parser-facing token stream.

## Public API

Not implemented yet. Planned surface:

- `Tokenizer` — source scanner entry point.
- `Token` — token value with location metadata.
- `TokenKind` — token category enum.
- Supporting span/diagnostic integration via `slopcc-common` types.

## Dependencies

- `slopcc-common` — shared source location and diagnostics primitives.

## Status

Scaffolded only. No tokenizer logic has been implemented yet.
