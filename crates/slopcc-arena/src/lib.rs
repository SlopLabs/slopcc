use std::{
  alloc::Layout,
  mem::{
    ManuallyDrop,
    MaybeUninit,
  },
  ptr::{
    self,
    NonNull,
  },
  sync::Mutex,
};

use crate::boxed::ArenaBox;

pub mod boxed;
pub mod prelude;

const DEFAULT_CHUNK_SIZE: usize = 8 * 1024;

struct Chunk {
  storage: NonNull<MaybeUninit<u8>>,
  capacity: usize,
  cursor: usize,
}

impl Chunk {
  fn new(capacity: usize) -> Self {
    let layout = Layout::array::<u8>(capacity).expect("chunk layout overflow");
    // SAFETY: layout is non-zero size (capacity > 0, enforced by Arena constructors)
    let ptr = unsafe { std::alloc::alloc(layout) };
    let storage = NonNull::new(ptr.cast::<MaybeUninit<u8>>())
      .unwrap_or_else(|| std::alloc::handle_alloc_error(layout));
    Self {
      storage,
      capacity,
      cursor: 0,
    }
  }

  fn try_alloc(&mut self, layout: Layout) -> Option<NonNull<u8>> {
    let aligned = self.cursor.checked_add(layout.align() - 1)? & !(layout.align() - 1);
    let end = aligned.checked_add(layout.size())?;
    if end > self.capacity {
      return None;
    }
    self.cursor = end;
    // SAFETY: aligned is within [0, capacity), storage is valid for capacity bytes
    unsafe {
      Some(NonNull::new_unchecked(
        self.storage.as_ptr().add(aligned).cast::<u8>(),
      ))
    }
  }
}

impl Drop for Chunk {
  fn drop(&mut self) {
    let layout = Layout::array::<u8>(self.capacity).expect("chunk layout overflow");
    // SAFETY: self.storage was allocated with this exact layout in Chunk::new
    unsafe { std::alloc::dealloc(self.storage.as_ptr().cast(), layout) }
  }
}

struct ArenaInner {
  chunks: Vec<Chunk>,
  chunk_size: usize,
}

pub struct Arena {
  inner: Mutex<ArenaInner>,
}

// SAFETY: All access to ArenaInner goes through the Mutex.
// Raw pointers in Chunk are exclusively owned by the Arena.
unsafe impl Send for Arena {}
unsafe impl Sync for Arena {}

impl Arena {
  #[must_use]
  pub fn new() -> Self {
    Self::with_chunk_size(DEFAULT_CHUNK_SIZE)
  }

  #[must_use]
  pub fn with_chunk_size(chunk_size: usize) -> Self {
    assert!(chunk_size > 0, "chunk size must be positive");
    Self {
      inner: Mutex::new(ArenaInner {
        chunks: vec![Chunk::new(chunk_size)],
        chunk_size,
      }),
    }
  }

  pub fn alloc<T>(&self, value: T) -> &'static T {
    let layout = Layout::new::<T>();

    if layout.size() == 0 {
      std::mem::forget(value);
      // SAFETY: ZST needs no actual memory. NonNull::dangling() provides a
      // validly-aligned, non-null pointer that will never be read from.
      return unsafe { &*NonNull::<T>::dangling().as_ptr() };
    }

    let ptr = self.alloc_raw(layout);
    let value = ManuallyDrop::new(value);

    // SAFETY: ptr is valid, aligned for T, and exclusively owned by this arena.
    // ManuallyDrop prevents double-drop: bytes are copied into arena memory,
    // and the value's destructor will never run. The arena bulk-frees raw bytes
    // on drop without running destructors on stored values.
    unsafe {
      ptr::copy_nonoverlapping(&*value as *const T, ptr.as_ptr().cast::<T>(), 1);
      &*ptr.as_ptr().cast::<T>()
    }
  }

  #[must_use]
  pub fn alloc_box<T>(&self, value: T) -> ArenaBox<T> {
    ArenaBox::new(self.alloc(value))
  }

  pub fn alloc_str(&self, s: &str) -> &'static str {
    if s.is_empty() {
      return "";
    }
    let bytes = self.alloc_slice(s.as_bytes());
    // SAFETY: input was valid UTF-8, bytes were copied verbatim
    unsafe { std::str::from_utf8_unchecked(bytes) }
  }

  pub fn alloc_slice<T: Copy>(&self, slice: &[T]) -> &'static [T] {
    if slice.is_empty() {
      return &[];
    }

    let layout = Layout::array::<T>(slice.len()).expect("slice layout overflow");
    let ptr = self.alloc_raw(layout);

    // SAFETY: ptr is valid, aligned for T, has room for slice.len() elements.
    // T: Copy so no drop concerns.
    unsafe {
      ptr::copy_nonoverlapping(slice.as_ptr(), ptr.as_ptr().cast::<T>(), slice.len());
      std::slice::from_raw_parts(ptr.as_ptr().cast::<T>(), slice.len())
    }
  }

  fn alloc_raw(&self, layout: Layout) -> NonNull<u8> {
    let mut inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());

    assert!(
      layout.size() <= inner.chunk_size,
      "allocation of {} bytes exceeds chunk size of {} bytes",
      layout.size(),
      inner.chunk_size,
    );

    if let Some(ptr) = inner.chunks.last_mut().unwrap().try_alloc(layout) {
      return ptr;
    }

    let chunk_size = inner.chunk_size;
    inner.chunks.push(Chunk::new(chunk_size));
    inner
      .chunks
      .last_mut()
      .unwrap()
      .try_alloc(layout)
      .expect("fresh chunk must fit allocation within chunk_size")
  }
}

impl Default for Arena {
  fn default() -> Self {
    Self::new()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn alloc_single_value() {
    let arena = Arena::new();
    let x = arena.alloc(42u64);
    assert_eq!(*x, 42);
  }

  #[test]
  fn alloc_multiple_values_stay_valid() {
    let arena = Arena::new();
    let a = arena.alloc(1u32);
    let b = arena.alloc(2u32);
    let c = arena.alloc(3u32);
    assert_eq!(*a, 1);
    assert_eq!(*b, 2);
    assert_eq!(*c, 3);
  }

  #[test]
  fn alloc_heterogeneous_types() {
    let arena = Arena::new();
    let x = arena.alloc(42u8);
    let y = arena.alloc(1000u64);
    let z = arena.alloc(true);
    assert_eq!(*x, 42);
    assert_eq!(*y, 1000);
    assert!(*z);
  }

  #[test]
  fn alloc_str_roundtrip() {
    let arena = Arena::new();
    let s = arena.alloc_str("hello world");
    assert_eq!(s, "hello world");
  }

  #[test]
  fn alloc_empty_str() {
    let arena = Arena::new();
    let s = arena.alloc_str("");
    assert_eq!(s, "");
  }

  #[test]
  fn alloc_slice_roundtrip() {
    let arena = Arena::new();
    let s = arena.alloc_slice(&[1u32, 2, 3, 4, 5]);
    assert_eq!(s, &[1, 2, 3, 4, 5]);
  }

  #[test]
  fn alloc_empty_slice() {
    let arena = Arena::new();
    let s = arena.alloc_slice::<u32>(&[]);
    assert!(s.is_empty());
  }

  #[test]
  fn alloc_zst() {
    let arena = Arena::with_chunk_size(64);
    let x = arena.alloc(());
    let y = arena.alloc(());
    assert_eq!(*x, ());
    assert_eq!(*y, ());
  }

  #[test]
  fn alloc_spans_multiple_chunks() {
    let arena = Arena::with_chunk_size(64);
    let mut refs = Vec::new();
    for i in 0..100u64 {
      refs.push(arena.alloc(i));
    }
    for (i, r) in refs.iter().enumerate() {
      assert_eq!(**r, i as u64);
    }
  }

  #[test]
  #[should_panic(expected = "exceeds chunk size")]
  fn oversized_alloc_panics() {
    let arena = Arena::with_chunk_size(64);
    arena.alloc([0u8; 128]);
  }

  #[test]
  #[should_panic(expected = "exceeds chunk size")]
  fn oversized_slice_panics() {
    let arena = Arena::with_chunk_size(32);
    arena.alloc_slice(&[0u64; 128]);
  }

  #[test]
  fn alignment_u64() {
    let arena = Arena::new();
    let _a = arena.alloc(1u8);
    let b = arena.alloc(2u64);
    assert_eq!((b as *const u64 as usize) % std::mem::align_of::<u64>(), 0);
  }

  #[test]
  fn alignment_u128() {
    let arena = Arena::new();
    let _pad = arena.alloc(1u8);
    let val = arena.alloc(42u128);
    assert_eq!(*val, 42);
    assert_eq!(
      (val as *const u128 as usize) % std::mem::align_of::<u128>(),
      0
    );
  }

  #[test]
  fn concurrent_allocations() {
    use std::{
      sync::Arc,
      thread,
    };

    let arena = Arc::new(Arena::new());
    let mut handles = Vec::new();

    for t in 0..8u64 {
      let arena = Arc::clone(&arena);
      handles.push(thread::spawn(move || {
        let mut refs = Vec::new();
        for i in 0..100u64 {
          refs.push(arena.alloc(t * 1000 + i));
        }
        for (i, r) in refs.iter().enumerate() {
          assert_eq!(**r, t * 1000 + i as u64);
        }
      }));
    }

    for h in handles {
      h.join().unwrap();
    }
  }

  #[test]
  fn str_with_unicode() {
    let arena = Arena::new();
    let s = arena.alloc_str("hello \u{1F980} crab");
    assert_eq!(s, "hello \u{1F980} crab");
  }

  #[test]
  fn alloc_box_wraps_arena_reference() {
    let arena = Arena::new();
    let value = arena.alloc_box(77u32);
    assert_eq!(*value, 77);
    assert_eq!(*value.as_ref(), 77);
  }
}
