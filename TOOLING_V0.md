# JaLM Tooling (jalmt) V0

This document describes the `jalmt` CLI surface for the MVP.

## Commands
- `jalmt parse <file>`: parse and print JSON errors.
- `jalmt fmt <file>`: format file in place.
- `jalmt check <file>`: type + effect check, output JSON diagnostics.
- `jalmt new <name> [--dir <path>]`: create a new project.
- `jalmt build [--dir <path>]`: parse + check `src/main.jalm`.
- `jalmt test [--dir <path>]`: parse + check all `tests/*.jalm`.
- `jalmt run [--dir <path>]`: parse + check `src/main.jalm` (runtime TBD).

## Project Layout
`jalmt new` creates:
```
<name>/
  jalm.toml
  jalm.lock
  src/
    main.jalm
  tests/
    basic.jalm
```

## Deterministic Builds
`jalm.lock` is a placeholder for deterministic builds. In v0 it is static
and must exist for tools that expect a lockfile.

## Notes
- `build`, `test`, and `run` currently only validate parse + checks.
- Execution will be wired once the WASM runtime + host ABI are available.
