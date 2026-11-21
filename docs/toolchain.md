# Toolchain

How to build for, emulate, and deploy to the reMarkable Paper Pro. Facts here
were collected as of 2026-11-20; device software moves quickly, so re-check
versions before relying on them.

## Target device

| Property       | Value                                                      |
| -------------- | ---------------------------------------------------------- |
| Device         | reMarkable Paper Pro, codename `ferrari`                   |
| SoC            | NXP i.MX 8M Mini, 4x Cortex-A53 (aarch64)                  |
| Display        | 11.8" color e-ink, 1620x2160 portrait                      |
| OS             | reMarkable OS ("Codex"), Yocto-based Linux with glibc      |
| Stock UI       | `xochitl`, Qt 6 / Qt Quick, started by systemd             |
| Rust target    | `aarch64-unknown-linux-gnu`                                |

Verified on software 3.27.1.0 (in the emulator, below): OS `5.7.121
(scarthgap)`, glibc 2.39, Qt 6.8.2, and the platform plugin
`/usr/lib/plugins/platforms/libepaper.so`. The shell is BusyBox, so some
GNU tools (such as `ldd`) are missing.

The Paper Pro Move (`chiappa`, 954x1696) is a sibling device and not a target
yet.

## Cross-compiling

The default path needs no reMarkable SDK:

```sh
brew install zig                  # or your platform's package manager
cargo install cargo-zigbuild
cargo device                      # alias defined in .cargo/config.toml
```

The output is `target/aarch64-unknown-linux-gnu/release/stepanova`. The alias
targets glibc 2.35 as a floor, which is older than the device's glibc (2.39)
and therefore safe. CI also builds this binary and uploads it as an artifact. This works for pure-Rust code and for C code built from source.

### Official SDK (only when linking device libraries)

If Stepanova ever links against libraries that exist only on the device (for
example Qt 6 with the `epaper` platform plugin), use reMarkable's official SDK,
which provides a cross-compiler and sysroot. The SDK only runs on Linux, so on
macOS use a Linux container. Since OS 3.27 there are aarch64-host builds, which
run natively in an arm64 container on Apple Silicon. Download links and the
mapping of SDK versions to OS versions are at
<https://developer.remarkable.com/links>. Use the SDK that matches the device's
OS version.

## Emulator

reMarkable publishes no emulator. [`rmemu`](https://github.com/aetherino/rmemu)
(unofficial, MIT) runs the stock Paper Pro root filesystem in an arm64 Docker
container with a shim faking the e-ink controller, pen, touch, and buttons. It
provides a browser simulator and a CLI for screenshots and input injection
(including pen strokes with pressure), which suits agent-driven testing.

Requirements: Apple Silicon or arm64 Linux, Docker, Node 20+. Intel hosts are
not supported.

```sh
git clone https://github.com/aetherino/rmemu ../rmemu
cd ../rmemu
bin/rmemu setup 3.27.1.0      # downloads firmware (~200 MB), builds the image
bin/rmemu start 3.27.1.0
bin/rmemu open                # simulator at http://127.0.0.1:7777
bin/rmemu push ../stepanova/target/aarch64-unknown-linux-gnu/release/stepanova /home/root/stepanova
bin/rmemu ssh /home/root/stepanova
```

Caveats:

- It is a young, single-maintainer project. Treat it as a convenience, not a
  ground truth; the real device is the final arbiter.
- E-ink refresh behavior and colors are approximated, not exact.
- Wi-Fi, cloud sync, Bluetooth, and the Type Folio keyboard are not emulated.
- Firmware images are reMarkable's property. Never commit or redistribute them
  (they are gitignored).

## Running on the device

1. Enable developer mode: *Settings → General → Paper Tablet → Software →
   Advanced → Developer Mode*. **This factory-resets the device**, shows a
   warning on every boot, and can only be undone through recovery mode. Sync
   your documents first.
2. Find the root password under *Settings → General → Help → About →
   Copyrights and Licenses*.
3. Connect over USB and copy the binary:

   ```sh
   scp target/aarch64-unknown-linux-gnu/release/stepanova root@10.11.99.1:/home/root/
   ssh root@10.11.99.1 /home/root/stepanova
   ```

Notes:

- The root filesystem is read-only. Keep everything under `/home/root`.
- The Paper Pro is encrypted, and `xochitl` is needed at boot to unlock it.
  Never disable the `xochitl` service; stopping it temporarily
  (`systemctl stop xochitl`) is fine.

## Display and input

Stepanova stops `xochitl` and drives reMarkable's e-paper engine directly; see
[decision 0002](decisions/0002-takeover-display.md). The device binary handles
the takeover itself. To try it in the emulator:

```sh
cargo device
cd ../rmemu
bin/rmemu push ../stepanova/target/aarch64-unknown-linux-gnu/release/stepanova /home/root/stepanova
bin/rmemu ssh '(nohup /home/root/stepanova > /tmp/stepanova.log 2>&1 &)'
bin/rmemu draw 300,500 900,450 1200,700 --pressure 0.7
bin/rmemu screenshot shot.png --scale 0.25
bin/rmemu power                   # exits Stepanova and restarts xochitl
bin/rmemu ssh 'cat /tmp/stepanova.log'
```

On a device, run it the same way over SSH. Exit with the power button or a
five-finger tap. If the process is killed hard, bring the stock UI back with
`ssh root@10.11.99.1 systemctl start xochitl`.

Input device numbering differs between the emulator (pen on `event1`) and the
device (pen on `event2`), so devices are always found by name.
