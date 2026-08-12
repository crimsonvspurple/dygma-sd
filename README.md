# Dygma Battery for Stream Deck

Native **Stream Deck plugin** (Rust) that shows left and right wireless battery levels from a Dygma Neuron on a key.

<p align="center">
  <img src="docs/preview.svg" alt="Stream Deck key previews: charging bolts with empty bars, mid charge, low, and bars-only" width="720" />
</p>

**Author:** Eminence · **UUID:** `red.eminence.dygma.battery` · **Stream Deck:** 7.3+ (SDKVersion 3)

---

## About the Plugin and Features

Reads Focus `wireless.battery.*` over the Neuron USB serial port and paints dynamic **SVG key art**.

| Board | Status |
|-------|--------|
| **Defy** | Verified |
| **Raise 2** (wireless) | Beta |
| **Sonsei** (wireless) | Beta |

| Platform | Status |
|----------|--------|
| **Windows** | Primary / verified |
| **macOS** | Beta |
| **Linux** | Experimental binary only (no official Stream Deck app) |

### Features

- Action **Dygma Battery → Dygma Battery**
- **Dual vertical bars** (5 blocks): left column left-aligned, right column right-aligned; blocks taper narrower at the bottom
- **Charge colors:** green (high) → lime → yellow → orange → red (≤20%)
- **Optional numbers** under each bar (property inspector; no `%` sign)
- **Center Dygma mark** between the columns
- **Charging:** yellow bolt + empty bar outlines only (no filled level or number while that side is charging — fuel-gauge % is unreliable on cable charge)
- **Device picker** when multiple Neurons are connected (per-key; auto = first available)
- **Auto-poll** (default 60 s, range 15–600)
- **Key press** forces `wireless.battery.forceRead` and refresh

### Stack

| Piece | Tech |
|-------|------|
| Focus / battery | [`dygma_focus` fork](https://github.com/crimsonvspurple/dygma-focus) |
| Stream Deck protocol | [`streamdeck-rs`](https://crates.io/crates/streamdeck-rs) |
| Runtime | Native binary launched by Stream Deck |

Marketplace listing assets live under [`marketplace/`](marketplace/) (same key SVG as the plugin).

---

## Requirements

| Item | Notes |
|------|--------|
| **Neuron USB** | Focus serial to the PC (not pure Bluetooth) |
| **Sides on RF** | Wireless link to Neuron (not side USB cables for a reliable level read) |
| **Bazecor** | **Closed** while this plugin owns the serial port |
| **Stream Deck** | Official app **7.3+** (Windows primary; macOS beta) |
| **Build (from source)** | Rust stable; Windows MSVC/`link.exe`; Linux `pkg-config` + `libudev-dev` |

Serial ports typically: Windows `COMx`, macOS `/dev/cu.usbmodem…`, Linux `/dev/ttyACM0` (user in `dialout` or udev).

---

## Install

### From GitHub Release (Recommended)

1. Open the latest [Release](https://github.com/crimsonvspurple/dygma-sd/releases).
2. Download **`red.eminence.dygma.battery.streamDeckPlugin`**.
3. Double-click to install into Stream Deck.
4. Add **Dygma Battery → Dygma Battery** to a key.
5. **Close Bazecor** while the plugin uses the Neuron.

Releases are multi-OS packages from GitHub Actions on tags (`v1.5.1`, …).

### From Source

**Windows**

```powershell
# One-time if link.exe is missing
winget install Microsoft.VisualStudio.BuildTools `
  --override "--wait --passive --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended"

cd plugin
.\scripts\install.ps1
# → %APPDATA%\Elgato\StreamDeck\Plugins\red.eminence.dygma.battery.sdPlugin
```

Local pack (Windows binary):

```powershell
.\plugin\scripts\pack.ps1
# → dist/*.streamDeckPlugin and dist/*.sdPlugin.zip
```

**macOS / Linux**

```bash
# Linux: sudo apt-get install -y pkg-config libudev-dev
cd plugin
chmod +x scripts/install.sh
./scripts/install.sh
```

macOS path:  
`~/Library/Application Support/com.elgato.StreamDeck/Plugins/`

---

## Development Info

### Project Layout

```text
dygma-defy-battery-instructions.md   # Focus API notes
docs/preview.svg                     # Key art strip (from plugin SVG)
marketplace/                         # Elgato listing media + product photos
plugin/
  Cargo.toml                         # lib + bins (plugin, gen-marketplace)
  src/
    lib.rs                           # crate root
    main.rs                          # Stream Deck entry, --self-test
    plugin.rs                        # per-key state, PI, polling
    battery.rs                       # Focus list/read
    visual.rs                        # SVG key art (source of truth)
    error.rs
    bin/gen_marketplace.rs           # marketplace PNG generator
  examples/gen_preview.rs            # docs/preview.svg generator
  red.eminence.dygma.battery.sdPlugin/
    manifest.json
    ui/property-inspector.html
    imgs/
  scripts/
    install.ps1 / install.sh
    pack.ps1
    gen-icons.ps1
.github/workflows/release.yml        # tag v* → multi-OS release
```

### Build, Test, Self-Test

```bash
cd plugin
cargo test
cargo run --release -- --self-test   # list devices + read battery (no Stream Deck)
```

### Marketplace Assets

Key tiles use **`visual::render_levels_svg_body`** (same as the live plugin):

```bash
cargo run --manifest-path plugin/Cargo.toml --features gen-marketplace --bin gen-marketplace
# optional: --out DIR
```

Regenerate the README preview strip:

```bash
cargo run --manifest-path plugin/Cargo.toml --example gen_preview
```

### Publish a Release

```bash
git tag -a v1.5.1 -m "v1.5.1"
git push origin v1.5.1
```

Builds Windows + macOS + Linux binaries and attaches `.streamDeckPlugin` to the GitHub Release. Manifest version is four segments (e.g. `1.5.1.0`).

### Focus Commands (Reference)

```text
wireless.battery.forceRead     # then wait ~2 s
wireless.battery.left.level
wireless.battery.right.level
wireless.battery.left.status
wireless.battery.right.status
```

See [`dygma-defy-battery-instructions.md`](dygma-defy-battery-instructions.md) for protocol details.

---

## Caveats

- **One process per serial port** — close Bazecor while the plugin is active.
- **Pure Bluetooth** (Neuron under the left half, no USB to PC) has **no** Focus port → no readings.
- **Cable charging:** status may show charging (bolt), but percentage is often wrong or missing; the UI hides filled bars and numbers for that side.
- **Status codes** vary by firmware; the bolt treats **1** and **2** as charging (Defy FW 2.2.1 often uses `1`).
- **Raise 2 / Sonsei / macOS** are **beta** (same Focus APIs as Defy; less field testing).
- Focus crate: [crimsonvspurple/dygma-focus](https://github.com/crimsonvspurple/dygma-focus) (fork; unused `windows` dep removed for Unix builds).

---

## Disclaimer

This is an **unofficial** community plugin by **Eminence**. It is **not** an official Dygma Lab product and is **not** made by Elgato / Corsair.

The **Dygma** name and logo are used **with permission from Dygma** to identify compatible keyboards.

The software is provided **as-is**, **without warranty or guarantee** of any kind — including fitness for a particular purpose, uninterrupted operation, or that it will not interfere with Bazecor, firmware, or hardware. **Use at your own risk.**
