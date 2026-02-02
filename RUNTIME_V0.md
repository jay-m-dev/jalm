# JaLM Runtime (WASM) V0

This document describes the minimal runtime surface needed for the standard library.

## Goals
- Provide a simple allocator for WASM linear memory.
- Provide panic/trap hooks for unrecoverable errors.
- Provide byte-oriented helpers for string/bytes primitives.

## Memory Model
- Linear memory with a bump allocator.
- 8-byte alignment.
- No deallocation in V0 (free is a no-op).
- WASM builds grow memory as needed with `memory.grow`.

## Exported ABI (V0)
These symbols are exported by the runtime module:

- `jalm_alloc(size: usize) -> *mut u8`
- `jalm_realloc(ptr: *mut u8, old_size: usize, new_size: usize) -> *mut u8`
- `jalm_free(ptr: *mut u8, size: usize)` (no-op in V0)
- `jalm_bytes_alloc(len: usize) -> *mut u8`
- `jalm_bytes_clone(src: *const u8, len: usize) -> *mut u8`
- `jalm_memcpy(dst: *mut u8, src: *const u8, len: usize) -> *mut u8`
- `jalm_memset(dst: *mut u8, value: u8, len: usize) -> *mut u8`
- `jalm_panic(code: u32) -> !` (traps in WASM)

## String/Bytes Basics
Strings and byte slices are represented as `(ptr, len)` pairs at the ABI boundary.
The runtime only provides allocation and copy helpers. UTF-8 validation and higher
level string APIs live in the standard library.

## Limitations
- No GC, no free list, and no compaction.
- `jalm_realloc` always allocates + copies.
- `jalm_memcpy` is non-overlapping; use memmove semantics at higher levels.

## Benchmarks
Run the allocator micro-bench:

```bash
cd jalmc
cargo bench -p jalm_runtime -- bench_alloc
```
