# JaLM Effect System v0

This document defines the MVP effect system for JaLM: how effects are declared, composed, and enforced by the compiler. The system is intentionally simple, explicit, and deterministic for agent use.

## Goals
- Make side effects explicit at function boundaries.
- Allow local reasoning: effects used in a function are a subset of its declared effect set.
- Provide a minimal effect vocabulary for web apps.

## Non-goals
- Effect polymorphism or higher-order effect variables.
- Implicit effect inference across module boundaries.
- Capability-based security beyond the declared set (host policy is separate).

## Effect Set (MVP)
Effects are named and finite in v0:
- `io`   — stdout/stderr, stdin, logging, console.
- `net`  — sockets, HTTP client/server, networking.
- `fs`   — filesystem read/write, path inspection.
- `time` — wall clock, timers, sleep.
- `rand` — randomness or entropy.
- `ffi`  — any host import not covered above.

## Syntax
Effect sets appear after the return type (or directly after parameters if no return type):

```
fn read_config() -> string !{fs, io} { ... }
fn now() -> i64 !{time} { ... }
fn pure_add(a: i64, b: i64) -> i64 { a + b }
```

The absence of `!{...}` means the function is **pure** (empty effect set).

## Core Rules
1. **Declaration required**: Any function that performs an effect must declare it.
2. **Subset rule**: The set of effects used in a function body must be a subset of its declared effect set.
3. **Call rule**: Calling a function requires the caller to include all effects declared by the callee.
4. **Pure default**: Functions without a `!{...}` declaration are treated as pure.
5. **No effect subtyping**: Effects are orthogonal; `net` does not imply `io` or `fs`.
6. **No effect inference**: Effects are never inferred across function boundaries; the declared set is authoritative.

## Where Effects Arise
An effect is considered “used” when:
- Calling any function annotated with that effect.
- Using a built-in or stdlib operation marked with that effect.
- Calling a host import (always `ffi`, unless specified otherwise).

## Async, Tasks, and Concurrency
- `async fn` uses the same declared effect set rules as sync functions.
- `await` itself introduces no effects.
- `spawn expr` is allowed only if the current function declares all effects required by the spawned computation.
- Effects are checked at the **call site** where `spawn` is performed (not at join time).
- The effects of an `async fn` are associated with its body; `await`ing a `Task<T>` does not introduce new effects beyond those already required by the task's creation.

## Higher-Order Functions
- A function value carries its declared effect set.
- Calling a function value requires the caller to declare the function's effects (same as a direct call).

## Standard Library Requirements (MVP)
The following are required annotations:
- `net::listen` / `http::serve` / `http::client`: `!{net}`.
- `fs::read`, `fs::write`: `!{fs}`.
- `time::now`, `time::sleep`: `!{time}`.
- `rand::bytes`, `rand::u64`: `!{rand}`.
- `log::*`: `!{io}`.
- Any raw host call: `!{ffi}`.

## Error Propagation and Effects
The `?` operator does not add effects by itself. Effects are determined solely by calls performed to produce the `Result` value.

## Examples
```
fn load_config(path: string) -> Result<string, IoError> !{fs, io} {
  let data = fs::read(path)?;
  log::info("loaded config");
  Ok(data)
}

fn handle(req: Request) -> Result<Response, HttpError> !{net} {
  if req.path == "/health" { Ok(Response::text("ok")) }
  else { Ok(Response::text("not found")) }
}

async fn tick_loop() -> () !{time, io} {
  loop {
    time::sleep(1000)?;
    log::info("tick");
  }
}
```

## Diagnostics (MVP)
- **Undeclared effect**: call requires `net` but caller has `!{io}`.
- **Unexpected effect in pure function**: effect used but no `!{...}`.
- **Unknown effect name**: effect not in the v0 set.
- **Missing ffi**: host import used without `!{ffi}`.

## Success Criteria
- All effectful stdlib functions are annotated.
- The compiler rejects any undeclared effect usage.
- The demo web app in `SPEC_MVP.md` can be annotated with `!{net, io, time}` (or less) and type-checks.
- Effect errors are deterministic and reference the call site that requires the effect.
