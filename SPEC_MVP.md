# JaLM MVP Spec (1-page)

## Purpose
Define the minimum viable JaLM language + runtime surface area to build and test a small web service, and the success criteria for declaring the MVP “done.”

## MVP Feature Scope
### 1) Syntax subset
- **Modules**: single package, `mod` declarations, `use` imports, file-based module layout.
- **Functions**: `fn name(params) -> Type` with optional `async`.
- **Control flow**: `if/else`, `match`, `for` over ranges/iterables.
- **Data**: `struct`, `enum` (algebraic), literals (int, float, bool, string, bytes), arrays/vecs, maps.
- **Errors**: `Result<T, E>`, `?` propagation, `panic` (discouraged in app code).
- **Effects annotation**: `fn foo() -> T !{net, io}` syntax (exact tokens tbd but required conceptually).

### 2) Types
- **Primitives**: `i32`, `i64`, `f64`, `bool`, `string`, `bytes`.
- **Core ADTs**: `Option<T>`, `Result<T, E>`.
- **Structs/enums**: nominal types with pattern matching.
- **Generics**: only for `Option`/`Result` and container types; no higher-kinded types.
- **Type checking**: local inference for literals and `let`; explicit annotations required at public fn boundaries.

### 3) Effects system
- **Effect set**: `io`, `net`, `fs`, `time`, `rand`, `ffi`.
- **Rules**:
  - Effects are explicit on functions that perform them.
  - Effect sets compose upward through calls.
  - Pure functions have empty effect set.
  - Compiler rejects missing or undeclared effects.

### 4) Structured concurrency
- **Scope model**: `scope { ... }` creates a task scope; tasks cannot outlive their scope.
- **Spawning**: `spawn` within scope returns a task handle.
- **Join**: `join` awaits completion; errors are aggregated.
- **Cancellation**: cancellation propagates from parent to children; per-request cancellation tokens exist.
- **Timeouts**: `timeout(dur, task)` cancels and returns error on expiration.

### 5) Web server MVP
- **HTTP types**: `Request`, `Response`, `Status`, `Headers`.
- **Router**: path + method matching with typed handler signature.
- **Middleware**: chainable, runs before/after handlers.
- **JSON**: encode/decode for structs/enums.
- **Runtime**: WASM host provides socket/http bindings; effects gate all net I/O.

## Non-goals (MVP excludes)
- Optimizing compiler or advanced performance tuning.
- Advanced generics (traits/typeclasses), macros, reflection, metaprogramming.
- Full standard library coverage beyond web + JSON + collections basics.
- Cross-package dependency resolution or registry.
- Production-ready WASM runtime/security hardening.
- Full IDE/LSP feature set beyond basic diagnostics.

## Demo App Target (acceptance example)
Build a demo JaLM web app with:
- **Routes**:
  - `GET /health` returns `"ok"`.
  - `GET /hello` returns `"hello"`.
  - `POST /json` accepts JSON body `{ "name": string }` and returns `{ "greeting": string }`.
- **Middleware**:
  - Adds `x-request-id` header if missing.
  - Logs method/path + status code.
- **Concurrency**:
  - Per-request scope; JSON route spawns a small async task (e.g., formatting) and joins.
- **Tests**:
  - Unit tests for JSON encode/decode.
  - Integration tests for all three routes and middleware header.
  - Failure test for invalid JSON → `400`.

## Success Criteria
- The demo app compiles and runs on the JaLM MVP toolchain.
- All demo tests pass locally.
- Effect checker prevents undeclared `net/io/fs/time/rand/ffi` usage.
- Structured concurrency rules prevent task escapes and ensure cancellation on request end.
- Documentation for MVP scope and demo behavior matches implementation.
