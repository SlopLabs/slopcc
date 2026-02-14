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
Source → Preprocessor → Lexer → Parser → Sema → LLVM IR → [LLVM] → Object Code → [Linker]
```

Each arrow before `[LLVM]` is a crate boundary we implement. LLVM handles optimization,
machine code generation, and assembly via `inkwell` (safe Rust bindings to LLVM).

We do NOT implement our own IR, optimizer, or machine code backend. LLVM gives us:
- All optimization levels (-O0 through -O3) for free
- Every target architecture (x86_64, i386/16-bit, AArch64, RISC-V, etc.)
- Assembly parsing and encoding (MC layer)
- 16-bit code generation (required for Linux kernel boot code)

### Linker Strategy

slopcc does not implement a linker. It dispatches to an external linker:
- Default: system linker (`cc`)
- Configurable via `-fuse-ld=lld`, `-fuse-ld=gold`, `-fuse-ld=bfd`, etc.
- The driver invokes the linker as a subprocess after codegen

### CLI Interface

GCC-compatible flags. Users should be able to substitute `slopcc` for `gcc` in most
build systems without changes:
- `-c`, `-S`, `-E`, `-o <file>`
- `-O0`, `-O1`, `-O2`, `-O3`, `-Os`, `-Oz`
- `-std=c89`, `-std=c99`, `-std=c11`, `-std=gnu11`
- `-I<dir>`, `-D<macro>`, `-U<macro>`
- `-W...`, `-Wall`, `-Werror`, `-Wno-...`
- `-fuse-ld=<linker>`
- `-v`, `--version`, `-###` (dry-run)

### Inline Assembly

GCC-style extended `asm` and `__asm__` with AT&T syntax:
```c
asm volatile ("movl %1, %%eax\n\t"
              "addl %2, %%eax\n\t"
              "movl %%eax, %0"
              : "=r" (result)
              : "r" (a), "r" (b)
              : "%eax");
```
We parse the asm statement, validate constraints, and emit it as LLVM inline assembly.

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

### Per-Crate README.md (MANDATORY)

Every crate has a `README.md` at its root (`crates/slopcc-<name>/README.md` or
`slopcc/README.md` for the binary). This is the first thing an AI agent reads when
touching a crate.

Each crate README must contain:
1. **Purpose** — what this crate does, in 1-2 sentences.
2. **Points of Interest** — key files, important types, non-obvious design decisions.
3. **Public API** — summary of the main entry points and how to use them.
4. **Dependencies** — which other slopcc crates this depends on and why.
5. **Status** — what is implemented, what is not yet.

**Before pushing any commit, update the README of every crate you touched.** This is
non-negotiable. If you changed a public API, the README must reflect it. If you added
a new module, the README must list it. Stale documentation is worse than no documentation
because it actively misleads the next agent.

### Session Status Tracking (STATUS.md) (MANDATORY)

`STATUS.md` at repo root is the single source of truth for cross-session continuity.
Every agent must read it first and update it during handoff.

Required structure:

```md
# STATUS

## Current State
- One-line truth of current project state

## Next Up
- [ ] P1: highest-priority next task
- [ ] P2: second-priority next task
- [ ] P3: third-priority next task

## Deferred (Too Hard / Later)
- [ ] Item — reason blocked now; unblock condition

## Last Updated
- YYYY-MM-DD by <agent> in <branch>
```

Usage protocol (strict):

1. Before work: read `STATUS.md` and align scope with `Next Up`.
2. While working: keep at most three actionable items in `Next Up`.
3. When blocked or too costly now: move item to `Deferred` with both:
   - reason blocked now
   - clear unblock condition (what must become true to resume)
4. At end of session: update `Current State` to reflect factual progress, not intent.
5. Never delete deferred items silently; either complete them or keep them deferred.
6. If priorities change, reorder `Next Up` by execution order.

Interpretation rules:

- `Current State` answers: "Where are we right now?"
- `Next Up` answers: "What should be worked on next?"
- `Deferred` answers: "What is known but intentionally postponed?"
- If `STATUS.md` conflicts with stale README status text, trust `STATUS.md` first and
  then update affected READMEs in the same session.

### Conventions

- Commit messages: imperative mood, concise. `add token types for C keywords` not
  `Added token types for C keywords`.
- Branch per feature when the change is large.
- No commented-out code in committed files.
- No TODO comments without a linked issue or concrete plan.

### Multi-Agent Git Workflow

When coordinating multiple subagents in parallel, prefer `git subtree`-based intake by
starting each subagent from the same current commit and merging results back into one
integration branch managed by the master agent.

Role model:

- Project maintainer (human): sets direction, reviews PR, merges on GitHub.
- Master agent: subtree maintainer/integrator; orchestrates subagents and performs final integration.
- Subagents: scoped developers working in isolated subtree clones from the same base commit.

Recommended flow:

1. Snapshot the current commit as the integration base.
2. Create one subtree clone/worktree per subagent from that exact base commit.
3. Let each subagent implement only its scoped change in its subtree clone.
4. Merge each subtree result back into the integration branch in small steps.
5. Resolve conflicts manually in the integration branch (never by dropping one side).
6. Push the integration result to a `feature/*` or `fix/*` branch and open a PR.

Guardrails:

- Non-code/docs-only changes may be pushed directly to `main`/`master`.
- Code changes must not be pushed directly to `main`/`master`; use `feature/*` or `fix/*` branches.
- Keep remote history non-destructive (no force-push to protected branches).
- Master agent is responsible for final integration and conflict resolution.
- Prefer manual GitHub merge after review over direct local fast-forwarding.
- If subtree is not practical for a small change, document why and use normal branch
  merge while preserving one-scope-per-branch discipline.

Incremental delivery policy (mandatory):

- Avoid massive PRs. Split feature work into small, reviewable commits.
- Push working branches regularly so progress is recoverable and others can continue.
- Prefer vertical slices (test + implementation + docs) over large horizontal rewrites.
- Keep each commit rollback-safe: if reverted, the branch should still build or fail in a clearly isolated way.
- If a feature is unfinished, push partial progress behind clear scope boundaries and update `STATUS.md` with exact next steps.

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

### Preventing Code Duplication (CRITICAL)

This project is built by multiple AI agents across many sessions. The #1 failure mode
is agents losing overview of the codebase and reimplementing things that already exist.

**Before writing ANY new function, type, or module:**
1. Search the codebase for existing implementations. Use grep, find, or your tools.
2. Check `slopcc-common` — shared types belong there, not duplicated per-crate.
3. Check `slopcc-arena` — all arena allocation goes through this crate.
4. Read doc comments on public APIs of crates you depend on.

**If you find existing code that almost does what you need:**
- Extend it. Do not create a parallel implementation.
- If the existing API needs changes, change it and update all callers.

**Canonical locations (one implementation, one place):**

| Concern | Lives in | Never duplicate in |
|---------|----------|--------------------|
| Span, FileId, SourceMap | `slopcc-common` | Any other crate |
| Diagnostics, error reporting | `slopcc-common` | Any other crate |
| Arena allocation | `slopcc-arena` | Any other crate |
| Token types | `slopcc-lex` | Parser or preprocessor |
| AST node types | `slopcc-ast` (when created) | Sema or codegen |

**If you are unsure whether something already exists, search first.** Five minutes of
searching beats five hours of debugging a duplicate implementation that subtly diverges.

### What Not To Do

- Do not create crates for phases that don't exist yet.
- Do not add external dependencies without justification — but when justified, use them.
- Do not pin old versions. Always check `cargo info` for latest.
- Do not refactor while fixing a bug. Fix first, refactor separately.
- Do not suppress warnings or errors. Fix them.
- Do not write code that "works for now" — write it correctly or leave a clear
  interface for the next agent to implement.
- Do not reimplement functionality that exists elsewhere in the codebase.
  Search before you write.

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
