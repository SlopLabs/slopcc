# AGENTS.md — AI Agent Onboarding

You are an AI agent working on slopcc, a C11-compliant C compiler written in Rust.
Read this file before doing anything.

## Project Overview

slopcc compiles C source code (C89 through C11) to x86_64 machine code. It is built
entirely by AI agents. No human writes code. Humans provide architectural direction only.

The compiler is built incrementally — one phase at a time. Each phase lives in its own
Cargo workspace crate under `crates/`.

## Repository Layout

```
slopcc/
├── slopcc/                # Binary crate. The compiler executable.
├── crates/
│   ├── slopcc-arena/      # Bump arena allocator. Core memory infrastructure.
│   ├── slopcc-common/     # Shared types used across all crates.
│   └── slopcc-lex/        # Lexer / tokenizer.
│   (more crates added as phases are implemented)
├── tests/
│   ├── fixtures/          # C source files for integration testing
│   └── harness/           # Test runner infrastructure
├── Cargo.toml             # Workspace root
├── README.md
└── AGENTS.md              # You are here
```

Executables live at the workspace root level. Library crates live under `crates/`.

New crates are added only when that compiler phase is actively being built. Do not
create placeholder crates for future phases.

## Compiler Pipeline (planned)

```
Source → Preprocessor → Lexer → Parser → Sema → IR → Optimizer → Codegen(x86_64) → Assembly
```

Each arrow is a crate boundary. Data flows through well-typed interfaces.

## Rules for AI Agents

### Code Quality

- Rust 2021 edition, stable toolchain only.
- `unsafe` is encouraged when it provides meaningful speed or simplicity gains.
  Every `unsafe` block requires a `// SAFETY:` comment explaining the invariant.
- No `unwrap()` in library crates unless the invariant is documented and proven.
- Error types use `thiserror`. No stringly-typed errors. No `Box<dyn Error>` in
  public APIs.
- All public types and functions get doc comments explaining what, not how.
- Use `#[must_use]` on functions that return values that shouldn't be silently dropped.

### Unsafe Patterns (encouraged)

These patterns are explicitly encouraged throughout the codebase:

- **`&'static` lifetime lying** — arena-allocated data returns `&'static T` references.
  We know the arena outlives all references to its contents within a compilation pass.
  Lying to the borrow checker here avoids lifetime parameter pollution across the
  entire codebase.
- **`ManuallyDrop`** — explicit ownership control. Values moved into arenas never have
  their destructors run; the arena bulk-frees raw bytes on drop.
- **`MaybeUninit`** — type-safe uninitialized memory for arena chunk storage. Avoids
  unnecessary zeroing of memory that will be written before it is read.
- **Raw pointer arithmetic** — arena internals, codegen emission buffers, anywhere
  the abstraction cost of safe wrappers exceeds the safety benefit.
- **`NonNull`** — preferred over `*mut T` for pointers known to be non-null.

The rule is simple: if `unsafe` makes the code faster or simpler and the invariant
is provable, use it. Document the invariant with `// SAFETY:`. Do not reach for
safe-but-slow alternatives out of fear.

### Architecture

- One crate per compiler phase. Dependencies flow strictly forward (lexer does not
  depend on parser).
- No dependency cycles. The workspace Cargo.toml enforces this.
- Shared types (Span, FileId, Diagnostics) live in `slopcc-common`. Do not duplicate
  them in other crates.
- Arena allocation for AST and IR nodes to avoid per-node heap allocation.

### Adding a New Compiler Phase

1. Create `crates/slopcc-<phase>/` with `Cargo.toml` and `src/lib.rs`.
2. Add it to the workspace `members` list in the root `Cargo.toml`.
3. Add it to the workspace `[workspace.dependencies]` table.
4. Wire it into the `slopcc` binary crate.
5. Update this file's repository layout section.

### Testing — Test-Driven Development

**Write tests first.** Every feature starts with a failing test. Implement until it passes.
This is non-negotiable — it's how multiple AI agents maintain confidence in each other's work.

#### Test Layers

1. **Unit tests** (`#[cfg(test)] mod tests`) — every crate, every module. Test the
   smallest meaningful behavior. Write these *before* implementing the function.
2. **Integration tests** (`tests/fixtures/`) — C source files with expected output.
   Add a `.c` file and its expected result *before* making the compiler handle it.
3. **Bash test harness** (`tests/harness/`) — shell scripts that exercise the compiled
   binary end-to-end. These are encouraged for anything `cargo test` can't easily cover:
   exit codes, signal handling, multi-file compilation, linker integration, comparison
   against gcc/clang output. Write these as plain `#!/bin/bash` scripts.
4. **Regression tests** — every bug fix gets a test that reproduces the bug *first*,
   then the fix makes it pass.

#### Test-First Workflow

```
1. Write a test that describes the desired behavior
2. Run it — confirm it fails
3. Implement the minimal code to make it pass
4. Refactor if needed (tests still pass)
5. Repeat
```

Do not write implementation code without a corresponding test. If you're unsure what
to test, that means the interface isn't clear yet — define it first.

#### Beyond Cargo

`cargo test` is the baseline but not the ceiling. Use whatever tools make sense:
- Bash scripts for pipeline / integration testing
- `diff` against gcc/clang output for conformance checking
- Custom test runners in `tests/harness/` for batch fixture testing
- Any tool that improves confidence in correctness

### Conventions

- Commit messages: imperative mood, concise. `add token types for C keywords` not
  `Added token types for C keywords`.
- Branch per feature when the change is large.
- No commented-out code in committed files.
- No TODO comments without a linked issue or concrete plan.

### Dependencies

- **Always use latest versions.** Before adding a dependency, run `cargo info <crate>`
  to check the current version, features, and license. Then use `cargo add <crate>` to
  add it — this always pulls the latest release.
- Never hardcode stale version numbers from memory. Your training data is outdated.
  `cargo info` is the source of truth.
- Dev-dependencies and build-dependencies are encouraged when they improve testing,
  code generation, or developer experience.
- Prefer std where possible, but do not reinvent well-maintained crates.

#### What to use external crates for

Utility and ergonomics crates are encouraged:
- **CLI**: `clap` for argument parsing
- **Error handling**: `thiserror`, `anyhow`
- **Derive macros**: `getset`, `typed-builder`, etc.
- **Testing**: `pretty_assertions`, `insta`, `criterion`, etc.
- **Allocators**: `mimalloc` (already the global allocator)

#### What NEVER to use external crates for

Never use a crate that does core compiler work for you. The following are strictly
forbidden:
- C parser crates, C lexer crates, C preprocessor crates
- AST libraries for C
- Any crate that would replace a compiler phase we are building

The entire point of slopcc is to implement the compiler. Using a `c-parser` crate
would defeat the purpose. If it's a compiler phase, we write it ourselves.

### What Not To Do

- Do not create crates for phases that don't exist yet.
- Do not add external dependencies without justification — but when justified, use them.
- Do not pin old versions. Always check `cargo info` for latest.
- Do not refactor while fixing a bug. Fix first, refactor separately.
- Do not suppress warnings or errors. Fix them.
- Do not write code that "works for now" — write it correctly or leave a clear
  interface for the next agent to implement.

## Current Status

The project is in initial scaffolding. The lexer is the first phase to implement.

## Target Architecture

Primary: x86_64 (Linux ELF, System V ABI).
Future: AArch64, RISC-V (not yet started, do not scaffold).

## C Standard References

When implementing, refer to:
- ISO/IEC 9899:1990 (C90)
- ISO/IEC 9899:1999 (C99)
- ISO/IEC 9899:2011 (C11)
- The C standard draft N1570 (freely available C11 draft)

Behavior must match the standard. When the standard is ambiguous, match GCC behavior
and document the choice.
