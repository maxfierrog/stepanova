# Stepanova: agent notes

- Read [CONTRIBUTING.md](CONTRIBUTING.md) (workflow, branching) and
  [docs/style.md](docs/style.md) (code style) before changing code.
- Format with `cargo +nightly fmt`; the stable formatter ignores most of
  `rustfmt.toml`.
- Device builds: `cargo device`. Emulator and deployment:
  [docs/toolchain.md](docs/toolchain.md). The emulator lives in `../rmemu` and is
  driven with `bin/rmemu` (screenshots, taps, pen strokes, ssh, push).
- The design is built incrementally from device experiments. Do not design
  ahead of evidence; record settled choices in `docs/decisions`.
- Never commit reMarkable firmware, SDKs, or files copied off a device.
