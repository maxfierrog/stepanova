# Stepanova: agent notes

- Read [CONTRIBUTING.md](CONTRIBUTING.md) (workflow, branching) and
  [style.md](style.md) (mandatory code style) before changing code.
- Format with stable `cargo fmt` (it skips the unstable options in
  `rustfmt.toml`, by design).
- Device builds: `cargo device`. Emulator and deployment:
  [docs/toolchain.md](docs/toolchain.md). The emulator lives in `../rmemu` and is
  driven with `bin/rmemu` (screenshots, taps, pen strokes, ssh, push).
- The design is built incrementally from device experiments. Do not design
  ahead of evidence. Design rationale is not kept in this repository; `docs`
  is only for people working on the code.
- Never commit reMarkable firmware, SDKs, or files copied off a device.
