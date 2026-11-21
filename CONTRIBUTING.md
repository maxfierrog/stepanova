# Contributing

Anyone is welcome to contribute. Before starting on anything substantial, open
an issue or get in touch with a maintainer so your effort is well-placed.

## Before your first contribution

Read and agree to the [Contributor License Agreement](CLA.md). The pull request
template asks you to confirm this; a pull request cannot be merged without it.

## Workflow

Stepanova uses trunk-based development. `main` is the only long-lived branch
and must always build and pass CI.

1. Branch off `main` with a short-lived branch named `<kind>/<topic>`:

   | Kind     | For                                                        |
   | -------- | ---------------------------------------------------------- |
   | `feat/`  | New functionality.                                         |
   | `fix/`   | Bug fixes.                                                 |
   | `docs/`  | Documentation only.                                        |
   | `chore/` | Tooling, CI, dependencies, refactors without behavior change. |
   | `exp/`   | Throwaway experiments (e.g. probing device behavior).      |

   ```sh
   git switch main && git pull
   git switch -c feat/selection-anchors
   ```

2. Commit with short imperative subjects ("Add selection anchors", not "Added
   selection anchors").

3. Push and open a pull request into `main`. CI must pass.

4. Pull requests are **squash-merged**, and the branch is deleted afterwards.
   The pull request title becomes the commit subject on `main`, so make it a
   good one.

`exp/` branches are never merged as-is. When an experiment produces something
worth keeping, write it up (code on a `feat/` branch, findings in
[`docs/decisions`](docs/decisions) or [`docs`](docs)) and delete the
experiment branch.

Releases are tags of the form `vMAJOR.MINOR.PATCH` on `main`, following
semantic versioning.

## Standards

- Code follows the [style guide](docs/style.md). Format with
  `cargo +nightly fmt` (the formatter configuration uses nightly-only options).
- Keep changes self-contained and small; under roughly 250 changed lines is a
  good target.
- Before pushing, run what CI runs:

  ```sh
  cargo +nightly fmt --all -- --check
  cargo clippy --workspace --all-targets -- -D warnings
  cargo test --workspace
  cargo device
  ```

## Documentation

- **API documentation** lives in rustdoc comments. Every `pub` item is
  documented; the workspace warns on missing docs.
- **Guides** (building, toolchain, architecture) live as Markdown in
  [`docs`](docs), versioned with the code. There is no wiki.
- **Decisions** that shape the project are recorded as short, numbered records
  in [`docs/decisions`](docs/decisions). Add one when you make a choice that a
  future contributor would otherwise have to rediscover.
