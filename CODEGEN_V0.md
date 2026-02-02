# JaLM WASM Codegen (V0)

This document describes the minimum-viable WASM code generator and its limits.

## Supported (V0)
- Top-level `fn` items.
- `let` bindings with explicit type.
- `return` statements.
- Expression statements.
- Function calls.
- Binary operators: `+ - * / == != < <= > >=`.
- `if` expressions as statement-like control flow.
- Literals: `i64`, `true`, `false`.

## Not Yet Supported (V0)
- `struct`, `enum`, and pattern matching beyond `if`.
- `match` codegen.
- Heap allocation, references, or strings.
- Multiple return types, non-`i64` params/returns.
- Modules/imports at codegen time.

## Execution
The tests in `jalmc/crates/jalm_codegen/tests/codegen_smoke.rs` use `wasmtime`
to compile and run the generated WASM and verify basic execution.

## Performance Checks
Run the basic micro-benchmark:

```bash
cd jalmc
cargo bench -p jalm_codegen -- bench_compile_to_wasm_small
```

## Notes
- The codegen currently emits a minimal WASM module with exported `main`.
- Errors are collected and returned as diagnostics instead of panicking.
