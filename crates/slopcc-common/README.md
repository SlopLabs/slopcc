# slopcc-common

Shared types used across all compiler crates.

## Purpose

Single canonical location for types that multiple crates need: source locations,
file identifiers, diagnostics, error reporting. Prevents duplication across the
compiler pipeline.

## Points of Interest

- `src/lib.rs` — currently empty, awaiting first shared types.

## Public API

Not yet implemented. Planned types:
- `Span` — byte range in source code
- `FileId` — identifier for a source file
- `SourceMap` — maps FileId + offset to file/line/column
- `Diagnostic` — compiler error/warning with source location

## Dependencies

None.

## Status

Scaffolded. No types implemented yet.
