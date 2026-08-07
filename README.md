# dygma-sd — Dygma wireless battery on Stream Deck

Native **Stream Deck plugin** (Rust) that reads wireless battery levels from a Dygma Neuron over Focus serial and shows them on a key.

<p align="center">
  <img src="docs/preview.svg" alt="Stream Deck key previews: dual battery bars with charge colors, optional %, and charging bolts" width="720" />
</p>

Works with **wireless split** boards that expose Focus `wireless.battery.*` (Neuron USB + RF sides). **Defy** is verified; **Raise 2** and **Sonsei** wireless support is **beta**. Pure Bluetooth mode is **not** supported (no Focus serial without Neuron USB).

| Piece | Tech |
|-------|------|
| Focus / battery | [`dygma_focus` fork](https://github.com/crimsonvspurple/dygma-focus) (crates.io 0.5.x also builds on Unix; fork drops unused `windows` dep) |
| Stream Deck protocol | [`streamdeck-rs`](https://crates.io/crates/streamdeck-rs) |
| Runtime | Native binary launched by Stream Deck |

**Plugin author:** Eminence · **UUID:** `red.eminence.dygma.battery`

> **Disclaimer:** This is an **unofficial** community plugin. It is **not** an official Dygma Lab product. The **Dygma logo is used with permission**. Provided **as-is, with no warranty or guarantee**. **Use at your own risk.**

## Platforms

| OS | Stream Deck host | Notes |
|----|------------------|--------|
| **Windows** | Official Elgato app **7.0+** | Primary / verified |
| **macOS** | Official Elgato app **7.0+** | **Beta** — binary `dygma-sd-plugin-mac` |
| **Linux** | No official Elgato app | Experimental binary for `--self-test` / community hosts |

Serial ports: Windows `COM4`, macOS `/dev/cu.usbmodem…`, Linux `/dev/ttyACM0` (add user to `dialout` or use udev).

## Requirements

| Item | Notes |
|------|--------|
| Neuron USB | Focus serial (e.g. VID `35EF`; Defy verified, Raise 2 / Sonsei beta) |
| Sides | RF to Neuron (not side USB cables) |
| Bazecor | **Closed** while the plugin owns the serial port |
| Stream Deck | **7.0+** (SDKVersion 3; Marketplace DRM-ready) |
| Build | Rust stable; Windows needs MSVC/link; Linux needs `pkg-config` + `libudev-dev` |

Bluetooth-only (Neuron under left half, no USB) does **not** expose Focus serial.

## Features

- Action **Dygma Battery → Dygma Battery**
- **SVG key art:** two vertical 5-block bars (L left-aligned, R right-aligned; bottom blocks narrower)
- Colors by charge: green (high) → lime → yellow → orange → red (≤20%)
- Tiny **Dygma mark** centered at the bottom of the key (between L/R %)
- Optional **% text** under each bar (property inspector)
- **Charging bolt** when Focus status is **1 or 2** (Defy FW 2.2.1 uses `1` while charging; older docs list `2`)
- **Device picker** when multiple Neurons are on USB (per-key setting; auto = first available)
- Auto-poll (default **60 s**, min 15 / max 600)
- **Key press** forces `wireless.battery.forceRead` + refresh

## Marketplace

Elgato Marketplace submission materials live in [`marketplace/`](marketplace/):

| Asset | File |
|-------|------|
| Listing copy + checklist | [`marketplace/LISTING.md`](marketplace/LISTING.md) |
| App icon (288×288) | `marketplace/app-icon-288.png` |
| Thumbnail / galleries | `marketplace/thumbnail-*.png`, `gallery-*.png` |

Regenerate media: `powershell -ExecutionPolicy Bypass -File marketplace/gen-assets.ps1`

Submit via [Maker Console](https://maker.elgato.com/) (org **Eminence**). Demo video of a real Neuron + wireless halves is required for hardware integrations.

## Install (end users)

### From GitHub Release (recommended)

1. Open the latest [Release](https://github.com/crimsonvspurple/dygma-sd/releases).
2. Download **`red.eminence.dygma.battery.streamDeckPlugin`**.
3. Double-click to install into Stream Deck.
4. Add **Dygma Battery → Dygma Battery** to a key.
5. **Close Bazecor** while the plugin owns the Neuron serial port.

Releases are multi-OS packages built by GitHub Actions on version tags (`v1.2.0`, …).

### From source (developers)

#### Windows

```powershell
# One-time: C++ build tools if link.exe is missing
winget install Microsoft.VisualStudio.BuildTools `
  --override "--wait --passive --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended"

cd plugin
.\scripts\install.ps1
# → %APPDATA%\Elgato\StreamDeck\Plugins\red.eminence.dygma.battery.sdPlugin
```

Local pack (Windows binary only):

```powershell
.\plugin\scripts\pack.ps1
# → dist/*.streamDeckPlugin and dist/*.sdPlugin.zip
```

#### macOS / Linux

```bash
# Linux: sudo apt-get install -y pkg-config libudev-dev
cd plugin
chmod +x scripts/install.sh
./scripts/install.sh
```

macOS install path:  
`~/Library/Application Support/com.elgato.StreamDeck/Plugins/`

#### Publish a GitHub Release via Actions

```bash
git tag v1.2.0
git push origin v1.2.0
```

Builds **Windows + macOS + Linux** binaries, packs one multi-OS `.streamDeckPlugin`.

### Self-test (no Stream Deck)

Lists Focus devices, then reads battery from the first available Neuron:

```bash
cd plugin
cargo run --release -- --self-test
```

### Unit tests

```bash
cd plugin
cargo test
```

## Project layout

```text
dygma-defy-battery-instructions.md   # Focus API notes
plugin/
  Cargo.toml
  src/
    main.rs       # entry, --self-test, select! event loop
    plugin.rs     # per-key state, device selection, SD handlers
    battery.rs    # dygma_focus list/read + tests
    visual.rs     # SVG key art
    error.rs      # PluginError
  red.eminence.dygma.battery.sdPlugin/
    manifest.json
    ui/property-inspector.html
    imgs/
  scripts/
    gen-icons.ps1
    install.ps1
    pack.ps1          # build + package for GitHub Release
.github/workflows/
  release.yml         # tag v* → build → GitHub Release
```

## Focus protocol (reference)

```
wireless.battery.forceRead     # then wait ~2s
wireless.battery.left.level
wireless.battery.right.level
wireless.battery.left.status
wireless.battery.right.status
```

See `dygma-defy-battery-instructions.md` for details and caveats.

## Caveats

- **One process owns each serial port** — close Bazecor while the plugin is active.
- Focus dependency: [crimsonvspurple/dygma-focus](https://github.com/crimsonvspurple/dygma-focus) (fork of `dygma_focus`; unused `windows` crate removed).
- Status codes vary by firmware; charging bolt accepts `1` and `2`.
- Wired charging sides often hide reliable % (hardware fuel-gauge limit).
- Pure Bluetooth mode is not supported (no Focus serial).

## Disclaimer & trademark

This project is an **unofficial** community plugin. It is **not** an official Dygma Lab product and is **not** made by Elgato / Corsair.

The **Dygma** name and logo are used **with permission from Dygma** to identify compatible keyboards.

The software is provided **as-is**, **without warranty or guarantee** of any kind — including fitness for a particular purpose, uninterrupted operation, or that it will not interfere with Bazecor, firmware, or hardware. **Use at your own risk.**
