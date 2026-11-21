# Stepanova

Stepanova is a note-taking and knowledge application for the reMarkable Paper
Pro that unifies knowledge creation (writing, drawing, markup) with organized
information retrieval through agentic AI.

Its core idea is the **selection**: any region of any document can be linked
from any other document, where it renders inline and stays clickable. Prompts
work the same way. Every selection included in a prompt carries a reference to
its source document and its location inside it, recursively, so the agent sees
exactly what you see.

The project is in its earliest stage. The design is developed incrementally
through experimentation on the device, and recorded in
[`docs/decisions`](docs/decisions).

## Layout

| Path             | Crate              | Purpose                                                 |
| ---------------- | ------------------ | ------------------------------------------------------- |
| `crates/core`    | `stepanova-core`   | Platform-independent model shared by device and server. |
| `crates/device`  | `stepanova-device` | Client running on the Paper Pro (binary `stepanova`).   |
| `crates/server`  | `stepanova-server` | Self-hosted server: agent harness, rendering, storage.  |

## Building

```sh
cargo build                   # host build of the whole workspace
cargo test                    # run tests
cargo device                  # cross-compile the device client for the Paper Pro
```

Cross-compilation, the emulator, and deploying to a device are covered in
[`docs/toolchain.md`](docs/toolchain.md).

## Contributing

See [`CONTRIBUTING.md`](CONTRIBUTING.md). Contributions require agreeing to the
[Contributor License Agreement](CLA.md).

## License

Stepanova is licensed under the [GNU Affero General Public License v3.0 or
later](LICENSE).

Stepanova is an independent project and is not affiliated with or endorsed by
reMarkable AS. "reMarkable" and "reMarkable Paper Pro" are trademarks of
reMarkable AS.
