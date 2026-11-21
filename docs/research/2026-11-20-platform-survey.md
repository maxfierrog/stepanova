# Paper Pro platform survey

Survey of official documentation, community projects, and prior art relevant
to building Stepanova on the reMarkable Paper Pro, as of 2026-11-20. Claims
about third-party projects come from their READMEs and source, not from
running them, unless stated.

## 1. What reMarkable officially provides

Source: [developer.remarkable.com](https://developer.remarkable.com/documentation)
(eight pages, last updated 2026-09-24).

- **SDK.** One Yocto SDK per device and OS version (cross-compiler, sysroot,
  headers). Linux hosts only; x86_64 and, since OS 3.27, aarch64. The latest
  Paper Pro (`ferrari`) SDK is for 3.29.1.0. Index:
  [developer.remarkable.com/links](https://developer.remarkable.com/links).
- **Developer mode.** Required for SSH. It factory-resets the device, shows a
  permanent boot warning, and keeps disk encryption. Turning it off requires
  the recovery tool (`rm_recover`).
- **GUI apps.** The only documented route is a Qt Quick app (no Qt Widgets)
  started with `QT_QUICK_BACKEND=epaper ./app -platform epaper`, with `xochitl`
  stopped. Touch works out of the box. Marker input is "more involved" and is
  **not documented**. Refresh and waveform control, partial updates, color,
  pressure, and tilt are **not documented** either.
- **xochitl** (the stock UI) is proprietary, with no compatibility guarantees
  between versions. It must stay installed and enabled, because it unlocks the
  encrypted disk at boot. Its document store is
  `/home/root/.local/share/remarkable/xochitl`, a set of UUID-named files with
  one binary file per page. The page format (`.rm` v6) is undocumented.
- **No cloud API docs, no app store, no developer program, and no statement
  about third-party apps.** Every page disclaims support for modified software.
- **Open source:** kernel
  ([linux-imx-rm](https://github.com/reMarkable/linux-imx-rm)), U-Boot, the
  `epaper` QPA plugin source
  ([epaper-qpa](https://github.com/reMarkable/epaper-qpa), touch and keyboard
  only, no pen handler), and two BSD-2 Qt examples
  ([remarkable-developer-examples](https://github.com/reMarkable/remarkable-developer-examples)).

The company [blog](https://remarkable.com/blog) has no developer content. Its
engineering posts stress writing feel above all (friction, sound, and latency;
Paper Pro cut ink response time by 40%) and a "calm", distraction-free product.
Integrations (Google Drive, Dropbox, OneDrive) are first-party and curated.

## 2. Display stack

Verified in the emulator (software 3.27.1.0): Qt 6.8.2, glibc 2.39, and
`/usr/lib/plugins/platforms/libepaper.so`.

The panel is driven by `libqsgepaper.so` (class `EPFramebuffer`), a userspace
"software TCON" that computes waveforms and flips dumb buffers on atomic-KMS DRM
(`/dev/dri/card0`). The framebuffer is 1620x2160 with a stride of 6528, in
RGB32 order. There is no fbdev to write to. Measured latency to sync
completion for a 300x300 rect
([rmpp-display-notes](https://github.com/MaximeRivest/rmpp-display-notes)):
Animation 304 ms, Pen 372 ms, Mono 634 ms, UI 630 ms, Content 1086 ms. Swaps
are non-blocking, and disjoint rects refresh concurrently.

Every third-party app uses one of three routes:

| Route | How it draws | xochitl | Used by | Main risks |
| ----- | ------------ | ------- | ------- | ---------- |
| **Official Qt Quick** | Qt scene graph through the `epaper` QPA and `libqsgepaper` | Stopped | reMarkable's examples | Rust↔Qt bindings and the SDK needed; pen handling undocumented; no documented refresh control |
| **AppLoad + QTFB** | Shared-memory RGBA8888 buffer plus socket messages; xochitl refreshes the panel | Running (app is a window inside it) | KOReader (official since 2025.08), riddle (windowed build), paperterm | Requires xovi (LD_PRELOAD hooks, LGPL) and AppLoad (GPL-3.0); breaks on firmware updates; only coarse refresh modes (`UFAST`, `FAST`, `ANIMATE`, `CONTENT`, `UI`); fast modes lose color |
| **Takeover (quill)** | Drives `EPFramebuffer` directly through a C ABI (`quill_init`, `quill_buffer`, `quill_swap_*`) | Stopped | riddle (takeover build), Muse, badapple | Interposes on vendor symbols (fragile across firmware); tested on 3.27.3.0 only; one commit; **disputed license** (see below) |

[Quill](https://github.com/MaximeRivest/quill) says it is an MIT clean-room
implementation. [Muse](https://github.com/Rkcr7/Muse) says its copy of quill
derives from the GPL-3.0 `epfb-re`. That must be resolved before Stepanova
reuses any of it.

Operational lessons from takeover apps:

- Hold a wake lock (`/sys/power/wake_lock`) while `xochitl` is stopped. Without
  it, the device autosuspends, and on resume a second `xochitl` starts on top of
  the app.
- Never `systemctl mask xochitl`; it drops SSH sessions on this firmware.
- Always provide an escape (five-finger tap or power key) and restore `xochitl`
  on exit, including on signals.

KOReader's lesson for in-xochitl apps: let xochitl own suspend and wake, don't
grab input, and send periodic full refreshes to clear ghosting.

## 3. Input

All projects read evdev directly. No library or API wraps pen input.

| Node | Device | Ranges |
| ---- | ------ | ------ |
| `/dev/input/event0` | Power button | |
| `/dev/input/event1` | Hall sensors (Folio) | |
| `/dev/input/event2` | Pen ("Elan marker") | X 0–11180, Y 0–15340, pressure 0–4096 |
| `/dev/input/event3` | Touch (multitouch) | X 0–2064, Y 0–2832 |

Discover devices by name under `/sys/class/input/eventN/device/name` rather
than by number. Pen coordinates scale to the screen without axis swaps or
mirroring (unlike rM1/rM2). Tilt (`ABS_TILT_*`) is used by no project and is
unverified. Exclusive access is `EVIOCGRAB`. Injected events must be paced, or
they get dropped. Detect the model from `/sys/devices/soc0/machine`
(`reMarkable Ferrari`, `Chiappa`, or `Tatsu`).

## 4. Document formats and sync

- `.rm` v6 (since late 2022) is reverse-engineered:
  [rmscene](https://github.com/ricklupton/rmscene) (Python, the reference),
  [rmc](https://github.com/ricklupton/rmc) (to SVG, PDF, or Markdown),
  [rmconvert](https://github.com/YakBarber/rmconvert) (Rust, both directions),
  [librm_lines](https://github.com/RedTTGMoss/librm_lines) (C++). Pre-v6 tools
  such as `lines-are-rusty` do not apply.
- Cloud: [rmapi](https://github.com/ddvk/rmapi) (Go client) and
  [rmfakecloud](https://github.com/ddvk/rmfakecloud) (self-hosted replacement
  for the cloud). These matter if Stepanova ever imports from or interoperates
  with xochitl notebooks.
- Agent-facing: [remarkable-mcp](https://github.com/SamMorrowDrums/remarkable-mcp)
  exposes documents to AI assistants over SSH or the cloud.

## 5. Prior art in AI on the device

[riddle](https://github.com/MaximeRivest/riddle) (MIT, Rust, about 1.9k stars)
and its GPL fork [Muse](https://github.com/Rkcr7/Muse) are the closest
products:

- You write; after about 2.8 s idle, the page is sent as a PNG to an
  OpenAI-compatible vision model. The reply is drawn back as skeletonized
  handwriting.
- First ink appears about 1 s after commit.
- Context is a list of recent page images. Neither does on-device recognition,
  structured documents, or links between documents.

Stepanova's distinguishing idea is addressable selections with recursive
provenance, used both as links and as prompt context. No surveyed project does
this.

## 6. Tooling

- **Emulator:** [rmemu](https://github.com/aetherino/rmemu) (see
  [toolchain](../toolchain.md)). It runs stock firmware with a hardware shim, so
  in principle all three display routes can be tried in it. Its e-ink timing is
  approximate, so latency must be measured on the device.
- **Installers:** [remagic](https://github.com/MaximeRivest/remagic) (developer
  mode → xovi + AppLoad, plus an app catalog) and
  [Vellum](https://github.com/vellum-dev/vellum) /
  [reManager](https://github.com/rmitchellscott/reManager). Toltec is rM1/rM2
  only.
- **Rust:** [libremarkable](https://github.com/canselcik/libremarkable) targets
  rM1/rM2 framebuffers and does not apply to the Paper Pro. No maintained Rust
  crate covers the Paper Pro display or input.
- **Community docs:** [remarkable.guide](https://remarkable.guide),
  [awesome-reMarkable](https://github.com/reHackable/awesome-reMarkable).

## 7. Implications for Stepanova

1. **Pen-to-ink latency is the deciding metric** for the display route, and it
   can only be measured on a real device. reMarkable's own priorities (writing
   feel, latency) point the same way.
2. **All three routes come down to the same primitives:** an RGB buffer,
   dirty-rect swaps with a refresh-mode hint, and raw evdev input. Hiding the
   route behind a small `Surface` / `Input` interface in the device crate lets
   it be chosen, or switched, later without touching the rest of the app.
3. **QTFB is the cheapest first experiment.** It is pure Rust with no Qt, the
   device stays usable, and KOReader proves it works. Takeover is the latency
   comparison. Official Qt is the route most compatible with reMarkable's own
   stack.
4. **Licensing:** reading evdev, and talking to QTFB through its socket
   protocol as a separate process, are unlikely to bring in copyleft terms
   (not legal advice). Linking quill or vendoring GPL code would; see
   [decision 0001](../decisions/0001-license-and-cla.md).
