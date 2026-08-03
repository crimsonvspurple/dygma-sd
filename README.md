# dygma-sd — Dygma Defy battery on Stream Deck

Native **Stream Deck plugin** (Rust) that reads wireless battery levels from a Dygma Defy Neuron over Focus serial and shows them on a key.

| Piece | Tech |
|-------|------|
| Focus / battery | [`dygma_focus`](https://crates.io/crates/dygma_focus) |
| Stream Deck protocol | [`streamdeck-rs`](https://crates.io/crates/streamdeck-rs) |
| Runtime | Native `.exe` launched by Stream Deck |

**Author (plugin):** Eminence · **UUID:** `com.red.eminence.dygma.battery`

## Requirements

| Item | Notes |
|------|--------|
| Neuron USB | COM port (e.g. **COM4**, `VID_35EF` / `PID_0012`) |
| Sides | RF to Neuron (not side USB cables) |
| Bazecor | **Closed** while the plugin owns the serial port |
| Stream Deck | 6.4+ |
| Build | Rust (MSVC) + C++ Build Tools / Windows SDK (`link.exe`) |

Bluetooth-only mode (Neuron under left half, no USB) does **not** expose Focus serial.

## Features

- Action **Dygma → Defy Battery**
- **SVG key art:** two vertical 5-block bars (L left-aligned, R right-aligned), bottom blocks narrower
- Colors by charge: green (high) → lime → yellow → orange → red (≤20%)
- Optional **% text** under each bar (property inspector)
- **Charging bolt** when Focus status is charging (`2`)
- Auto-poll (default **60 s**, min 15 / max 600)
- **Key press** forces `wireless.battery.forceRead` + refresh

## Build & install (Windows)

### One-time: C++ build tools

```powershell
winget install Microsoft.VisualStudio.BuildTools `
  --override "--wait --passive --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended"
```

### Install into Stream Deck

```powershell
cd plugin
.\scripts\install.ps1
```

This generates icons, builds release, copies  
`com.red.eminence.dygma.battery.sdPlugin` →  
`%APPDATA%\Elgato\StreamDeck\Plugins\`, and restarts Stream Deck.

Then drag **Defy Battery** from the **Dygma** category onto a key.

### Self-test (no Stream Deck)

```powershell
cd plugin
cargo run --release -- --self-test
```

### Unit tests

```powershell
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
    plugin.rs     # state, KeyTitle, SD handlers
    battery.rs    # dygma_focus wrapper + tests
    visual.rs     # SVG key art
    error.rs      # PluginError
  com.red.eminence.dygma.battery.sdPlugin/
    manifest.json
    ui/property-inspector.html
    imgs/
  scripts/
    gen-icons.ps1
    install.ps1
```

## Focus protocol (reference)

```
wireless.battery.forceRead     # then wait ~2s
wireless.battery.left.level
wireless.battery.right.level
```

See `dygma-defy-battery-instructions.md` for details and status-code caveats.

## Caveats

- **One process owns the COM port** — close Bazecor while the plugin is active.
- Status codes may not match older docs on some firmware (e.g. `0` observed on 2.2.1).
- Wired charging sides often hide %.
