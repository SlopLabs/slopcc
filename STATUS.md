# STATUS

## Current State
- Preprocessing-token lexer (`slopcc-lex`) implemented and tested (23 unit tests).
- CLI foundation, shared source/span/diagnostic primitives in place.
- Total: 52 passing tests across workspace.

## Next Up
- [ ] P1: implement preprocessor (`slopcc-pp`) — `#define`, `#include`, `#if`/`#ifdef`/`#ifndef`, macro expansion
- [ ] P2: add location remapping for preprocessor line markers (`#line` / `# <line> <file>`)
- [ ] P3: pp-token → C token conversion (keyword recognition, numeric literal validation)

## Feature Tickets

### LEX — Lexer enhancements (post-preprocessor)
- [ ] LEX-1: trigraph replacement (translation phase 1)
- [ ] LEX-2: line splicing / backslash-newline continuation (translation phase 2)
- [ ] LEX-3: pp-token → C token conversion (keyword recognition, numeric literal validation)
- [ ] LEX-4: diagnostic emission for lexer errors (unterminated strings, bad chars)
- [ ] LEX-5: fixture-driven lexer regression tests in `tests/fixtures/`

### PP — Preprocessor
- [ ] PP-1: `#define` object-like macros + macro expansion
- [ ] PP-2: `#define` function-like macros with parameters
- [ ] PP-3: `#` stringification and `##` token pasting
- [ ] PP-4: `#include` with header search paths
- [ ] PP-5: conditional compilation (`#if`, `#ifdef`, `#ifndef`, `#elif`, `#else`, `#endif`)
- [ ] PP-6: constant expression evaluation for `#if`
- [ ] PP-7: `#line`, `#error`, `#pragma`
- [ ] PP-8: variadic macros (`__VA_ARGS__`)
- [ ] PP-9: predefined macros (`__FILE__`, `__LINE__`, `__DATE__`, `__TIME__`, etc.)

### PARSE — Parser
- [ ] PARSE-1: create `slopcc-ast` crate with AST node types
- [ ] PARSE-2: create `slopcc-parse` crate — recursive descent parser
- [ ] PARSE-3: expression parsing with operator precedence
- [ ] PARSE-4: declaration parsing (variables, functions, types)
- [ ] PARSE-5: statement parsing (if/else, for, while, switch, return, goto)

### SEMA — Semantic analysis
- [ ] SEMA-1: type checking
- [ ] SEMA-2: scope/symbol resolution
- [ ] SEMA-3: implicit conversions (integer promotions, usual arithmetic conversions)

### CODEGEN — LLVM IR generation
- [ ] CODEGEN-1: integrate `inkwell` (LLVM bindings)
- [ ] CODEGEN-2: emit LLVM IR for basic expressions and functions
- [ ] CODEGEN-3: emit LLVM IR for control flow

### DRIVER — CLI and pipeline
- [ ] DRIVER-1: pipeline orchestration (lex → preprocess → parse → sema → codegen)
- [ ] DRIVER-2: external linker invocation

## Deferred (Too Hard / Later)
- [ ] Full C11 `_Generic`, `_Atomic`, `_Alignas`/`_Alignof` — blocked on basic type system
- [ ] Inline assembly (`asm`/`__asm__`) — blocked on codegen
- [ ] VLA support — blocked on sema + codegen
- [ ] Self-hosting — long-term goal, not a near-term constraint

## Last Updated
- 2026-02-15 by opencode-agent in main
