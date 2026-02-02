# JaLM MIR v0 (Mid-level IR)

This document defines JaLM MIR v0: a typed, lowered IR designed for straightforward WASM codegen. MIR is SSA-like at the block level (block parameters), uses explicit control flow, and carries effect metadata.

## Goals
- Enable direct, deterministic WASM codegen.
- Make control flow explicit (CFG with blocks and terminators).
- Preserve types and effect requirements at each call site.
- Keep the instruction set small and stable.

## Non-goals
- Aggressive optimizations (CSE, LICM, inlining) in v0.
- Full SSA or register allocation complexity.
- High-level language constructs (pattern matching, async) without lowering.

## MIR Overview
### Module
A module contains:
- Type definitions (struct/enum layouts).
- Function signatures (name, params, result, effects).
- Function bodies (blocks + instructions).

### Function
```
fn <name>(p0: T0, p1: T1, ...) -> T !{effects}
{
  blocks...
}
```

Each function body is a set of basic blocks:
- One entry block.
- Blocks have parameters (phi-like).
- Each block ends with a terminator.

### Basic Block
```
block <id>(b0: T0, b1: T1, ...) {
  v0 = ...
  v1 = ...
  terminator ...
}
```

### Values
- Values are SSA temporaries: `v0`, `v1`, ...
- All values have explicit types.

## Types
- Primitive: `i32`, `i64`, `f64`, `bool`, `string`, `bytes`, `unit`.
- Aggregate: `struct S`, `enum E` (lowered to tagged unions).
- Pointers/references: not exposed in MIR (lowered to WASM locals/linear memory ops).

## Effects Metadata
Each function signature includes a declared effect set. MIR uses that metadata only for validation: call sites require the callee effects to be included in the caller's effect set.

## Instructions (v0)
### Constants
- `const_i32`, `const_i64`, `const_f64`, `const_bool`, `const_string`, `const_bytes`, `const_unit`.

### Locals
- `local_get <name>`
- `local_set <name> <value>`

### Arithmetic / Logic
- `add`, `sub`, `mul`, `div`, `rem`
- `eq`, `neq`, `lt`, `lte`, `gt`, `gte`
- `and`, `or`, `not`

### Struct / Enum
- `struct_new <S> (fields...)`
- `struct_get <S>.<field> <value>`
- `enum_new <E>::<Variant> (payload...)`
- `enum_tag <E> <value>`
- `enum_payload <E>::<Variant> <value>`

### Calls
- `call <fn> (args...)`
- `call_import <name> (args...)`

### Memory (WASM)
- `load_i32`, `load_i64`, `load_f64`, `load_bytes`
- `store_i32`, `store_i64`, `store_f64`, `store_bytes`
- `alloc <size>`

## Terminators
- `return <value>`
- `br <block> (args...)`
- `br_if <cond> <then_block> (args...) <else_block> (args...)`
- `trap <message>`

## Lowering Rules (Sketch)
- `if` becomes `br_if` to then/else blocks.
- `match` lowers to `enum_tag` + conditional branches.
- `let` binds to SSA values (or `local_set` if needed).
- `?` becomes `match` on `Result` with early return.

## Layout Notes
- Enums are represented as a tag + payload:\n  - Tag type: `i32`.\n  - Payload is a union sized to the largest variant payload.\n- Struct layout is field-ordered with alignment rules matching WASM linear memory conventions.

## Validation Rules (MVP)
- Block arguments must match the target block’s parameter arity and types.\n- Every value is defined before use.\n- Terminators are the last instruction in a block.\n- `call` argument types must match callee signature.\n- `return` value type must match function result.

## WASM Lowering Conventions
- MIR values map to WASM locals.\n- Aggregates are stored in linear memory; `struct_new` returns a pointer.\n- `load_*`/`store_*` use byte offsets aligned to the target type.

## Example
Source:
```
fn add(a: i64, b: i64) -> i64 { a + b }
```

MIR:
```
fn add(p0: i64, p1: i64) -> i64 !{} {
  block0() {
    v0 = add p0 p1
    return v0
  }
}
```

## Success Criteria
- Every construct in `GRAMMAR_V0.md` has a deterministic MIR lowering path.
- MIR is sufficient to generate WASM for the MVP demo.
- Effect sets are validated at MIR call sites.
