# Style Guide

These rules apply to all Rust code in the repository. Formatting itself is
enforced by [`rustfmt.toml`](../rustfmt.toml); run `cargo +nightly fmt`, since
several options (such as braces on the next line) are nightly-only.

## 1. Module layout

Each module with content has its own file. Do not create files that only
declare submodules:

```rust
// Avoid: module0.rs or module0/mod.rs containing only this.
pub mod module1;
pub mod module2;
```

Instead, declare such "category" modules inline in their parent with a module
block:

```rust
pub mod module0 {
    pub mod module1;
    pub mod module2;
}
```

## 2. Sections

Group the items in a file under uppercase banner comments, in this order where
present:

```rust
/* MODULES */
/* CONSTANTS */
/* TYPES */            // type aliases and simple enums
/* DEFINITIONS */      // structs and enums carrying data
/* INTERFACES */       // traits
/* IMPLEMENTATIONS */
/* TESTS */
```

Binaries may add sections such as `/* PROGRAM ENTRY */`. More specific names
(`/* UTILITY MODULES */`, `/* STRUCTURAL INTERFACES */`) are fine when they
help.

## 3. Imports

One item per `use` statement, in three blocks separated by blank lines:
external crates, then `std`, then `crate`/`super`.

```rust
use anyhow::Context;
use anyhow::Result;

use std::path::Path;

use crate::document::Selection;
```

## 4. Documentation

Every module opens with an inner doc comment headed by its name:

```rust
//! # Selections Module
//!
//! What this module provides and why it exists.
```

All `pub` items are documented with `///` comments. Do not write doctests; put
tests in `#[cfg(test)]` modules instead.

## 5. Errors

Binaries (`device`, `server`) use `anyhow` and attach a full-sentence
`.context(...)` to fallible calls at boundaries. Library code in `core` returns
typed errors when callers need to distinguish failure cases.

## 6. Containment

Changes should be specific and self-contained. A pull request should generally
not touch several top-level modules at once, and all of its changes should be
logically related.

## 7. Testing

Every module with non-trivial logic has a `#[cfg(test)] mod test` submodule.
Interfaces and simple glue need no tests, but any code that a Border Collie
could not understand within ten seconds of looking at it should be tested.

## 8. Platform boundaries

`stepanova-core` must build and test on any host: no device hardware, display,
or network access. Device-specific code belongs in `stepanova-device`, and
server-specific code in `stepanova-server`.
