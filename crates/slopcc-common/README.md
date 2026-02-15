# slopcc-common

Shared types used across all compiler crates.

## Purpose

Single canonical location for types that multiple crates need: source locations,
file identifiers, diagnostics, error reporting. Prevents duplication across the
compiler pipeline.

## Points of Interest

- `src/lib.rs` — `Span`, `BytePos`, and `FileId` types.

## Public API

```rust
type BytePos = u32;                       // Byte offset (supports files up to 4 GiB)

Span::new(lo, hi) -> Span                 // Byte range [lo, hi)
Span::at(pos) -> Span                     // Zero-width span
Span::len(&self) -> u32                   // Length in bytes
Span::is_empty(&self) -> bool             // Zero-width check
Span::merge(self, other) -> Span          // Union of two spans
Span::as_str(&self, src: &[u8]) -> &[u8]  // Extract source text

FileId::new(id: u32) -> FileId            // Create file identifier
FileId::as_u32(self) -> u32               // Raw index
```

Planned (not yet implemented):
- `SourceMap` — maps FileId + offset to file/line/column
- `Diagnostic` — compiler error/warning with source location

## Dependencies

None.

## Status

`Span`, `BytePos`, and `FileId` implemented with 6 unit tests.
`SourceMap` and `Diagnostic` deferred until needed by preprocessor or parser.
