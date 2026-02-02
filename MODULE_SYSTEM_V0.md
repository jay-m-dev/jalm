# JaLM Module System & Project Layout v0

This document defines the MVP module system, directory layout, import resolution, and visibility rules for JaLM.

## Goals
- Simple, predictable, file-based module layout.
- Explicit imports with no hidden global namespace.
- Clear public/private boundaries for agent readability.

## Non-goals
- Multi-package workspaces or dependency registries (handled later).
- Conditional compilation or feature flags.
- Cyclic module graphs with implicit ordering.

## Project Layout (MVP)
```
my_app/
  jalm.toml
  src/
    main.jalm
    lib.jalm
    foo.jalm
    foo/
      bar.jalm
```

- `jalm.toml` defines the package name.
- `src/main.jalm` is the executable entry.
- `src/lib.jalm` is the library root (if present).

## Modules and Files
- Each `.jalm` file defines exactly one module.
- The module name is the file stem.
- `mod name;` in a file declares a submodule named `name`.
- Submodule resolution:
  - `mod foo;` resolves to `src/foo.jalm` or `src/foo/mod.jalm` (if directory).
  - `mod foo;` inside `src/foo.jalm` resolves to `src/foo/bar.jalm` or `src/foo/bar/mod.jalm`.

## Import Resolution (`use`)
- Absolute paths start at the package root module:
  - `use crate::foo::bar;`
- `crate::` is the canonical root path.
- Relative paths start at the current module:
  - `use self::bar;` (explicitly relative)
- `use super::baz;` refers to the parent module.
- Aliasing: `use foo::bar as baz;`.

## Visibility
- Items are **private by default**.
- Use `pub` to export functions, structs, enums, and constants.
- Private items are accessible only within the defining module.
- Child modules cannot access private items of parent modules (no "friend" access).

## Module Namespace
- The module path is the canonical namespace for items.
- Two items with the same name in different modules are distinct.
- `use` brings names into local scope; name conflicts must be resolved with aliasing.
- Module imports must be acyclic (no circular `use` graphs) in v0.

## Main vs Library
- `main.jalm` can access the library root via `use crate::...`.
- `lib.jalm` exports library public APIs.
- The root module path is `crate`.

## Diagnostics (MVP)
- **Unknown module**: `mod foo;` with no matching file.
- **Unknown import**: `use foo::bar;` when `foo::bar` is not public.
- **Visibility error**: access to a non-`pub` item outside its module.
- **Duplicate module**: both `foo.jalm` and `foo/mod.jalm` exist.

## Examples
```jalm
// src/lib.jalm
mod foo;

pub fn version() -> string { "0.1.0" }

// src/foo.jalm
pub struct User { id: i64; name: string; }

// src/main.jalm
use crate::foo::User;
use crate::version;

fn main() -> () {
  let u = User { id: 1, name: "A" };
  log::info(version());
}
```

## Success Criteria
- Module graph is deterministically resolved from files on disk.
- `pub`/private access is enforced at compile time.
- Import resolution is unambiguous and stable across platforms.
