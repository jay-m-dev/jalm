# JaLM Type System v0

This document specifies the MVP type system sufficient to type-check JaLM v0 programs (web app + JSON + middleware). It is intentionally minimal and designed to be implementable before advanced features.

## Goals
- Provide a small, explicit, and predictable type system.
- Support structs, enums, pattern matching, and standard containers.
- Enable local type inference while keeping public APIs explicit.
- Make `Option`/`Result` and error propagation well-typed.

## Agent-Oriented Design Principles
- Favor **determinism** over convenience: the same program always yields the same types without global inference.
- Prefer **explicitness at boundaries**: public APIs and cross-module calls must be fully typed.
- Avoid implicit conversions: numeric and container types do not coerce automatically.
- Keep the surface area **small and teachable** for agents (limited features, clear rules).

## Non-goals
- Higher-kinded types, traits/typeclasses, or user-defined generic parameters.
- Subtyping or implicit numeric widening.
- Polymorphic recursion or global type inference.
- Overload resolution beyond built-in operator typing.

## Core Types
### Primitives
- `i32`, `i64`, `f64`, `bool`, `string`, `bytes`.
- Unit type: `()` (implicit when a function or block has no final expression).

### Standard Algebraic Types
- `Option<T>`: `Some(T)` | `None`.
- `Result<T, E>`: `Ok(T)` | `Err(E)`.

### Containers (stdlib types)
- `Vec<T>`: growable list.
- `Map<K, V>`: key/value map.

### User-Defined Types
- **Structs**: nominal product types with named fields.
- **Enums**: nominal sum types with variants (tuple-like or unit).
- **Type aliases**: not in v0.
- **User generics**: not in v0 (only built-in generic containers).

## Type Equality
- Nominal: `struct` and `enum` names define unique types.
- Two types are equal if they are structurally identical and all named types resolve to the same definition.

## Literals and Defaults
- Integer literals are untyped until constrained by context; if unconstrained in a simple `let`, they default to `i64`.
- Float literals are untyped until constrained by context; if unconstrained in a simple `let`, they default to `f64`.
- If a numeric literal remains ambiguous (e.g., appears in different branches with no type anchor), it is a type error and requires an explicit annotation or `as` cast.
- At **module boundaries** (public functions or exported constants), numeric literals must be type-anchored by an annotation or cast even if a default exists.
- String literals are `string`; byte string literals are `bytes`.
- `true`/`false` are `bool`.

## Function Types
- Function signature: `fn (T1, T2, ...) -> T` with optional effect set (see effects spec).
- `async fn` returns an implicit `Task<T>` (stdlib type), and `await` yields `T`.
- Public functions must have explicit parameter and return types.

## Type Inference (Local)
- `let` bindings infer their type from the initializer when no annotation is provided.
- Inference is **local and expression-based**; no whole-program inference.
- Type annotations are required for:
  - Public functions.
  - Any `let` without initializer.
  - `match` arms if the arm expression alone is ambiguous (e.g., numeric literal).
- Inference does not perform implicit numeric widening or coercion.

## Operators
All operators are **monomorphic** and type-checked by fixed rules:

- Arithmetic: `+ - * / %` require numeric operands of the same type; result is that type.
- Comparison: `< <= > >=` require numeric operands of the same type; result is `bool`.
- Equality: `== !=` require operands of the same type; result is `bool`.
- Logical: `&& || !` require `bool` operands; result is `bool`.
- Bitwise: `& | ^ ~ << >>` require integer operands of the same type; result is that type.
- Range: `..` and `..=` require numeric operands of the same type; result is `Range<T>` (stdlib type).
- Null-coalescing: `a ?? b` requires `a: Option<T>` and `b: T`; result is `T` (syntax sugar for `a.unwrap_or(b)`).
- Conditional: `cond ? a : b` requires `cond: bool` and `a`/`b` same type; result is that type.
- Assignment: `=` requires LHS/RHS same type. Compound assignments follow corresponding operator rules.

Note: `??` is defined only for `Option<T>` in v0 to avoid silently discarding errors from `Result<T, E>`.

## Blocks and Control Flow
- Block type is the type of its final expression (or `()` if none).
- `if` expression requires a `bool` condition; both branches must have the same type.
- `for` loops evaluate to `()`.
- `break`/`continue` are only valid in loops; `break expr` requires the loop to accept a result type (not supported in v0), so `break expr` is a type error in v0.

## Pattern Matching
- `match` scrutinee type must be known.
- Each arm pattern must be compatible with the scrutinee type.
- All arms must return the same type.
- Exhaustiveness is required for `enum` and `bool`. For numeric and string types, a default `_` arm is required.
- Patterns:
  - Identifier binds a value of the matched type.
  - `_` matches any value and binds nothing.
  - Struct pattern requires all listed fields to exist and match field types.
  - Enum pattern must match a known variant; tuple arity must match variant payload types.

## Calls and Member Access
- Function calls must supply arguments that exactly match parameter types.
- `await` is valid only inside `async fn`.
- Field access requires the base to be a struct with that field.
- Indexing requires `Vec<T>` or `Map<K, V>` (or a stdlib-defined indexable type).

## Casts (`as`)
- `expr as T` is an explicit cast.
- v0 supports numeric casts among `i32`, `i64`, and `f64`.
- All other casts are a type error (add a library conversion instead).

## Error Propagation (`?`)
- `expr?` is valid if `expr` has type `Result<T, E>`.
- The containing function must return `Result<U, E>` for some `U`.
- `expr?` has type `T`.

## Type Rule Summary (MVP Checklist)
1. Public function params/returns are fully typed.
2. No implicit numeric widening or container coercion.
3. Branching (`if`, `match`) produces a single unified type.
4. `Option<T>` handled via `??` or `match`; `Result<T, E>` handled via `?` or `match`.
5. `await` only in `async fn`; yields the inner `T`.
6. Operator operands must be same type (except `??` which is `Option<T> ?? T`).
7. Struct field access and enum patterns must match declared definitions.
8. Unconstrained numeric literals at public boundaries are an error.

## Standard Library Type Requirements (MVP)
These type signatures are required for the MVP to type-check the demo web app:
- JSON: `fn decode<T>(bytes) -> Result<T, JsonError>` and `fn encode<T>(T) -> Result<bytes, JsonError>`.
- HTTP: `Request`, `Response`, `Status`, `Headers` types and handler signature `fn(Request) -> Result<Response, HttpError>`.

## Diagnostics (MVP)
Type checker errors should include:
- Expected vs actual type.
- Span of error location.
- Diagnostic code (e.g., `E0003` for type mismatch).
- A short fix hint when possible (e.g., “add `as i64`” or “add explicit return type”).

## Examples
```jalm
fn add(a: i64, b: i64) -> i64 { a + b }

fn parse_user(body: bytes) -> Result<User, JsonError> {
  let user = json::decode<User>(body)?;
  Ok(user)
}

fn handle(req: Request) -> Result<Response, HttpError> {
  if req.path == "/health" { Ok(Response::text("ok")) }
  else { Ok(Response::text("not found")) }
}

pub fn timeout_ms() -> i64 {
  5000 as i64
}
```

## Success Criteria
- All core constructs in `GRAMMAR_V0.md` are type-checkable with the rules above.
- Demo web app described in `SPEC_MVP.md` type-checks without additional type system features.
- Type errors are consistent and deterministic.
