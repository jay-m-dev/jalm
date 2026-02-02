# JaLM Structured Concurrency v0

This document defines the MVP structured concurrency model for JaLM: scopes, spawning, cancellation, timeouts, join semantics, and request handling patterns. The model is designed for deterministic reasoning by agents.

## Goals
- Ensure tasks never outlive their parent scope.
- Provide predictable cancellation propagation.
- Make error handling and aggregation explicit.
- Support per-request scopes for web servers.

## Non-goals
- Detached background tasks (no "fire-and-forget" in v0).
- Actor systems or message passing beyond basic channels.
- Scheduling guarantees or real-time constraints.

## Core Concepts
### Task Scope
- `scope { ... }` creates a **task scope**.
- All tasks spawned inside a scope must complete before the scope exits.
- A scope returns `Result<T, TaskError>` where `T` is the type of its final expression.
- `scope { ... }` is implicitly equivalent to `try { ... }` in that `TaskError` propagates to the caller unless handled.

### Task Handle
- `spawn expr` evaluates `expr` in a new task and returns a `Task<T>` handle.
- `expr` must be an `async` computation that yields `T`.
- Tasks are **structured**: they are tied to the nearest enclosing `scope`.
- `Task<T>` is **scope-bound**: it cannot be stored in a struct that outlives the scope or returned from the scope.

### Join
- `join task` waits for completion and returns `Result<T, TaskError>`.
- Join is **explicit**; tasks are not auto-joined except at scope exit.

## Cancellation Model
- Each scope has a cancellation token.
- Cancelling a scope cancels all child tasks recursively.
- Cancellation propagates from parent to children, not vice versa.
- When a scope is cancelled:
  - All child tasks receive cancellation.
  - The scope waits for children to observe cancellation and exit.
  - The scope returns a `TaskError::Cancelled` unless explicitly handled.

## Timeouts
- `timeout(dur, task)` returns `Result<T, TaskError>`.
- On timeout, the task is cancelled and the result is `TaskError::Timeout`.
- Timeouts are required to be deterministic: the timer source is `time` effect.

## Error Aggregation
- If multiple child tasks fail, errors are **collected**.
- `join_all(tasks)` returns `Result<Vec<T>, TaskError>` where `TaskError` can represent:
  - `Cancelled`
  - `Timeout`
  - `Panic`
  - `Many(Vec<TaskError>)` for aggregation
- `scope` exit aggregates unjoined child failures into a single `TaskError` if they occur.

## Effects and Concurrency
- `spawn` does not change the effect set; the enclosing function must already declare all effects used by the spawned task.
- `timeout` requires `!{time}`.

## Request Handling Pattern (Web Server)
- Each incoming request runs inside a fresh `scope`.
- The request scope is cancelled when:
  - The client disconnects.
  - The handler returns a response.
- Example pattern:

```
async fn handle(req: Request) -> Result<Response, HttpError> !{net, time, io} {
  scope {
    let task = spawn async {
      let data = fetch_user(req.user_id)?;
      Ok(data)
    };

    let data = join task?;
    Ok(Response::json(data))
  }
}
```

## Diagnostics (MVP)
- **Spawn outside scope**: `spawn` used without an enclosing `scope`.
- **Escaping task**: returning or storing a `Task<T>` beyond its scope.
- **Missing time effect**: `timeout` used without `!{time}`.
- **Unjoined task on scope exit**: task still running when scope exits.

## Success Criteria
- Task lifetimes are enforced by the compiler (no escapes).
- Cancellation propagates on request end.
- `timeout` deterministically cancels and returns `TaskError::Timeout`.
- Errors from child tasks are surfaced in a deterministic, aggregated form.
