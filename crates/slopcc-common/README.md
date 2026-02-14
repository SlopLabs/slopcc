# slopcc-common

Shared types used across all compiler crates.

## Purpose

Single canonical location for types that multiple crates need: source locations,
file identifiers, diagnostics, error reporting. Prevents duplication across the
compiler pipeline.

Memory model goal: prefer borrowed/slice APIs where possible and avoid owned `String`/
`Vec` usage unless ownership boundaries require it.

## Points of Interest

- `src/lib.rs` — crate module exports.
- `src/prelude.rs` — canonical re-exports for downstream crates.
- `src/source.rs` — `FileId`, `SourceFile`, `SourceMap`, line/column resolution.
- `src/span.rs` — half-open byte-range `Span`.
- `src/diag.rs` — diagnostic severity and collection types.

## Public API

Current API surface:
- `Span` — half-open byte range `[start, end)` with `FileId`
- `FileId` — opaque source file identifier
- `SourceMap` — owns source bytes and resolves byte offsets to line/column
- `ResolvedSpan` — resolved source name + line/column + length
- `Diagnostic`, `Diagnostics`, `Severity` — compiler diagnostic primitives
- `prelude` module — central re-exports for consumers

## Dependencies

None.

## Status

Initial source/span/diagnostic foundations implemented with unit tests.
