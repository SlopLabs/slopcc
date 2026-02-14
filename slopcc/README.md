# slopcc

The compiler binary. CLI entry point and pipeline orchestration.

## Purpose

Parses command-line arguments (GCC-compatible flags), drives the compilation
pipeline from preprocessing through codegen, and dispatches to an external
linker.

## Points of Interest

- `src/main.rs` — process entry point and exit code handling.
- `src/cli.rs` — initial GCC-shaped argument parser.
- `src/driver.rs` — driver boundary for source loading and phase dispatch.
- GCC-compatible CLI flags: `-c`, `-S`, `-E`, `-o`, `-O`, `-std=`, `-I`, `-D`,
  `-W`, `-fuse-ld=`, `-v`, `--version`, `-###`

## Public API

N/A — this is a binary crate, not a library.

## Dependencies

- `slopcc-common` — shared types
- `slopcc-lex` — tokenizer
- `mimalloc` — global allocator

Additional compiler phase crates will be added as they are implemented.

## Status

Initial CLI parsing and driver scaffolding implemented. Binary now parses a core
flag subset, loads input sources, and reports tokenizer phase as not implemented.
