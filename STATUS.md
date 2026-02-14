# STATUS

## Current State
- Initial CLI foundation and shared source/span/diagnostic primitives are implemented.
- `slopcc` now parses a core GCC-shaped flag subset and loads input sources.
- `slopcc-common` now provides `FileId`, `Span` (offset + length via half-open range), `SourceMap`, and diagnostics.

## Next Up
- [ ] P1: implement `Tokenizer` skeleton in `crates/slopcc-lex` over byte offsets only
- [ ] P2: define token taxonomy for preprocessor-first tokenization (`pp-token` compatible)
- [ ] P3: add location remapping strategy for preprocessor line markers (`#line` / `# <line> <file>`)

## Deferred (Too Hard / Later)
- [ ] Full preprocessor macro expansion — blocked on stable tokenizer output and directive handling model

## Last Updated
- 2026-02-15 by opencode-agent in feature/cli-source-foundation
