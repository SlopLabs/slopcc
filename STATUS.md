# STATUS

## Current State
- Workspace scaffolding is in place; lexer is the first active implementation phase.

## Next Up
- [ ] P1: define and test complete C token taxonomy in `crates/slopcc-lex`
- [ ] P2: implement lexing for whitespace/comments/identifiers/keywords with spans
- [ ] P3: add fixture-driven lexer regression tests in `tests/fixtures`

## Deferred (Too Hard / Later)
- [ ] Full preprocessor macro expansion — blocked on stable lexer token stream and diagnostics shape

## Last Updated
- 2026-02-14 by opencode-agent in current branch
