# slopcc-arena

Bump arena allocator. Core memory infrastructure for the entire compiler.

## Purpose

Provides fast, bulk-deallocated memory for AST nodes, IR nodes, interned strings,
and anything else that lives for the duration of a compilation pass. Returns
`&'static T` references — we lie to the borrow checker because the arena outlives
all references within a pass.

## Points of Interest

- `src/lib.rs` — the entire implementation. Single file, intentionally simple.
- `Chunk` — raw memory blocks allocated via the global allocator (mimalloc).
  Uses `NonNull<MaybeUninit<u8>>` for type-safe uninitialized storage.
- `ManuallyDrop` — values moved into the arena never have destructors run.
  The arena bulk-frees raw bytes on drop.
- Thread-safe via `Mutex<ArenaInner>`. Designed for concurrent use from day one.
- Oversized allocations (larger than a single chunk) panic. Keep it simple.

## Public API

```rust
Arena::new() -> Arena                           // 8 KiB chunks (default)
Arena::with_chunk_size(usize) -> Arena          // custom chunk size
Arena::alloc<T>(value: T) -> &'static T         // allocate a single value
Arena::alloc_str(s: &str) -> &'static str       // allocate a string copy
Arena::alloc_slice<T: Copy>(&[T]) -> &'static [T]  // allocate a slice copy
```

## Dependencies

None (uses only std). Allocates through the global allocator, which is mimalloc
when used from the slopcc binary.

## Status

Implemented and tested. 15 unit tests covering allocation, alignment, threading,
edge cases (ZST, empty slices, oversized rejection), and unicode strings.
