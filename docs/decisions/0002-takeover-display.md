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
  exit or on `SIGTERM`/`SIGINT`. A detached watchdog restores them even if the
  process dies without unwinding.

## Consequences

- The binary depends on private vendor symbols, which can change in any
  firmware update. Supported firmware has to be tracked and the symbols checked
  on every release.
- Stepanova cannot run side by side with the stock UI. Switching between them
  means stopping one and starting the other.
- Firmware drift is real. Going from 3.27 to 3.29 (OS 5 to OS 6, Qt 6.8 to
  6.10) did two things. It removed the `EPContentType` parameter from
  `swapBuffers`, so both signatures are now resolved. It also added a second
  lock file (`/tmp/epd.lock`) that the stopped `xochitl` leaves behind and that
  makes the engine abort. CI smoke-tests each supported firmware in the
  emulator to catch this kind of change.
- The vendor engine aborts the process on initialization failure, so a
  detached watchdog restarts `xochitl` whenever Stepanova exits, however it
  exits.
- Verified in the emulator on 3.27.1.0 and 3.29.1.0: drawing works, and software latency
  from pen event to refresh submission is about 5 ms at p50 and 9 ms at p95.
  The panel's own latency can only be measured on hardware.
