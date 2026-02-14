# slopcc

A C11-compliant C compiler built entirely by AI. Part of the [SlopLabs](https://github.com/SlopLabs) ecosystem.

## What is this?

slopcc is a from-scratch C compiler targeting full C11 compliance, built incrementally
by AI agents with human-provided architectural direction. No human writes code — humans
provide operative insight, design decisions, and course corrections. AI does the rest.

slopcc is part of SlopLabs, a collection of tools built exclusively by AI — including
slopos, a POSIX-compliant operating system. A long-term goal is running slopcc natively
on slopos.

## Goals

- **Full C11 compliance** — C89, C99, and C11 semantics with all their quirks
- **Complete preprocessor** — `#include`, `#define`, conditionals, macro expansion,
  stringification, token pasting, variadic macros, `_Pragma`
- **LLVM backend** — we emit LLVM IR via `inkwell`, LLVM handles optimization and
  machine code generation. This gives us every target (x86_64, i386/16-bit, AArch64,
  RISC-V) and all optimization levels for free.
- **Inline assembly** — GCC-style `asm`/`__asm__` with AT&T syntax and constraints
- **GCC-compatible CLI** — drop-in replacement for `gcc` in most build systems
- **Accurate diagnostics** — GCC/Clang-quality error messages with source locations
- **Linux kernel compilation** — a north-star goal that exercises every dark corner

## Non-Goals (for now)

- Self-hosting (eventual goal, not a constraint during development)
- C++ support
- Implementing our own linker (we dispatch to `lld`/`ld`/`gold` via `-fuse-ld=`)

## Language: Rust

| Concern         | Why Rust                                                              |
|-----------------|-----------------------------------------------------------------------|
| Portability     | Tier-1 support for Linux, macOS, Windows. Cross-compiles easily.      |
| Speed           | Zero-cost abstractions, no GC. Comparable to C/C++.                   |
| Readability     | `enum` + `match` = natural AST/token/IR representation.               |
| Maintainability | Type system is the integration contract between AI sessions.          |
| Memory          | No GC. Arena allocators are idiomatic. Fine-grained control.          |
| AI authorship   | Strong types prevent integration bugs across independent AI sessions. |

## Architecture

Cargo workspace, one crate per compiler phase. Crates are added only when that phase
is actively being built.

```
Source (.c/.h) → Preprocessor → Lexer → Parser → Sema → LLVM IR → [LLVM] → Object Code → [Linker]
```

We implement everything left of `[LLVM]`. LLVM handles optimization, machine code
generation, and assembly. The linker is an external tool invoked as a subprocess.

### Current Crates

| Crate | Purpose |
|-------|---------|
| `slopcc` | Binary — CLI entry point, pipeline orchestration |
| `slopcc-arena` | Bump arena allocator — core memory infrastructure |
| `slopcc-common` | Shared types: Span, SourceMap, Diagnostics, FileId |
| `slopcc-lex` | Tokenizer for C source code |

## C Standard Compliance Strategy

1. **Phase 1**: C89/C90 core — basic types, control flow, functions, structs, unions,
   enums, pointers, arrays, string literals.
2. **Phase 2**: C99 — `_Bool`, designated initializers, compound literals, VLAs,
   `restrict`, `inline`, `//` comments, mixed declarations, variadic macros.
3. **Phase 3**: C11 — `_Alignas`/`_Alignof`, `_Atomic`, `_Generic`, `_Noreturn`,
   `_Static_assert`, `_Thread_local`, anonymous structs/unions.

## Building

```sh
cargo build
```

## Testing

```sh
cargo test
```

## Current Status

Project scaffolding. Next step: implement the lexer.

## License

Apache-2.0
