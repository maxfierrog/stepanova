# 0002: Drive the panel directly ("takeover")

Date: 2026-11-21. Status: accepted.

## Context

Stepanova is a writing tool, so pen-to-ink latency is the deciding property
of its display path. The [platform survey](../research/2026-11-20-platform-survey.md)
found three routes: an official Qt Quick app, a window inside the stock UI via
AppLoad and QTFB, or stopping the stock UI and driving reMarkable's e-paper
engine (`libqsgepaper.so`, class `EPFramebuffer`) directly. Only the last one
gives the application the vendor's pen waveform with nothing in between. The
existing implementation of it, Quill, is a C++ adapter that needs the
reMarkable SDK to build.

## Decision

- Stepanova takes over the panel. While it runs, `xochitl` is stopped and
  Stepanova owns the display and input.
- The takeover is implemented natively in Rust (`crates/device/src/vendor.rs`),
  following the technique documented by Quill (MIT, clean-room):
  - export the `QImage` pixel constructor to capture the framebuffer while the
    engine starts;
  - resolve `EPFramebuffer::instance` and `EPFramebuffer::swapBuffers` with
    `dlsym`;
  - keep a `QCoreApplication` alive for the life of the process.
  Everything is resolved at runtime, so no vendor or Qt code is linked or
  redistributed, and no SDK is needed (`cargo device` still builds it).
- Input is read from evdev. Devices are found by name (`EVIOCGNAME`) and axis
  ranges are read with `EVIOCGABS`, because device numbering and ranges differ
  between the device and the emulator.
- A `Session` guard stops `xochitl`, holds a wake lock, and restores both on
  exit or on `SIGTERM`/`SIGINT`.

## Consequences

- The binary depends on private vendor symbols, which can change in any
  firmware update. Supported firmware has to be tracked and the symbols checked
  on every release.
- Stepanova cannot run side by side with the stock UI. Switching between them
  means stopping one and starting the other.
- If the process is killed with `SIGKILL`, `xochitl` stays stopped until it is
  restarted over SSH.
- Verified in the emulator on 3.27.1.0: drawing works, and software latency
  from pen event to refresh submission is about 5 ms at p50 and 9 ms at p95.
  The panel's own latency can only be measured on hardware.
