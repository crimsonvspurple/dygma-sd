# Elgato Marketplace listing (Maker Console)

Copy/paste into [Maker Console](https://maker.elgato.com/). Update after each release.

## Product identity

| Field | Value |
|-------|--------|
| **Product type** | Stream Deck plugin |
| **Name** | Dygma Battery |
| **Author / organization** | Eminence |
| **Plugin UUID** | `red.eminence.dygma.battery` (must not change after publish) |
| **Price** | Free |
| **OS** | Windows 10+ (primary); **macOS 10.15+ beta**; Linux binary experimental only (no official Stream Deck app) |
| **Stream Deck** | **7.3+** |
| **SDKVersion** | **3** (latest; enables Marketplace DRM) |
| **DRM** | **Yes** — set in Maker Console when uploading a package with SDKVersion 3 + MinimumVersion ≥ 6.9 (we use 7.3) |
| **Category (in Stream Deck)** | Dygma Battery |
| **Action** | Dygma Battery |

## Short description (first ~250 characters — SEO)

Show left and right wireless battery levels from a Dygma Neuron on a Stream Deck key. Verified on Defy; Raise 2 and Sonsei wireless support is beta. Windows is the primary platform; macOS is beta. Neuron USB + RF sides required (not pure Bluetooth). Close Bazecor while the plugin owns the serial port. Unofficial community plugin by Eminence; Dygma logo used with permission.

## Full description (≤1500 characters)

**Dygma Battery** puts your wireless Dygma keyboard’s left and right battery levels on a Stream Deck key.

**Compatible boards** (Focus `wireless.battery.*` over Neuron USB):

- Dygma **Defy** — verified
- Dygma **Raise 2** (wireless) — **beta**
- Dygma **Sonsei** (wireless) — **beta**

**Features**

- Dual vertical bars for left and right halves
- Color by charge (green → red)
- Optional numeric labels (no % sign)
- Charging: bolt + empty outlines only (level is unreliable while charging)
- Device picker when multiple Neurons are connected
- Auto-poll (default 60s) and key-press force refresh

**Requirements**

- Neuron connected to the PC by **USB** (pure Bluetooth mode is not supported)
- Keyboard halves linked to the Neuron over **RF** (not side USB cables for battery read)
- **Bazecor closed** while this plugin uses the serial port
- Stream Deck software **7.3+** (**Windows** primary; **macOS beta**)
- Manifest: `SDKVersion` **3**, `Software.MinimumVersion` **7.3** (DRM-eligible package)

**Platform status**

- **Windows** — primary, best tested
- **macOS** — **beta**
- **Raise 2 / Sonsei** wireless — **beta** (same Focus battery APIs as Defy; less field testing)

**Notes**

This is an **unofficial** community plugin by **Eminence**. It is not an official Dygma Lab product. The Dygma logo is used with permission from Dygma. Provided as-is, without warranty; use at your own risk.

Source and updates: https://github.com/crimsonvspurple/dygma-sd

## Media files (this folder)

| File | Spec | Use |
|------|------|-----|
| `app-icon-288.png` | 288×288 PNG | Marketplace app icon |
| `thumbnail-1920x960.png` | 1920×960 PNG | Product thumbnail |
| `gallery-01-key-art.png` | 1920×960 PNG | Gallery 1 |
| `gallery-02-setup.png` | 1920×960 PNG | Gallery 2 |
| `gallery-03-boards.png` | 1920×960 PNG | Gallery 3 |

Regenerate:

```powershell
powershell -ExecutionPolicy Bypass -File marketplace/gen-assets.ps1
```

## Demo video (you record)

Hardware integrations need a short demo. Suggested script (~30–60s):

1. Stream Deck + Neuron USB + wireless halves; Bazecor closed.
2. Add **Dygma Battery** action; show bars update.
3. Optional: toggle % in property inspector.
4. Optional: plug charger → bolt appears; filled level hides while charging.
5. Optional: force refresh by pressing the key.

Export **1920×1080 MP4**, under size limits in Maker Console.

## Package to upload

From a full multi-OS build (GitHub Release) or local pack:

```text
red.eminence.dygma.battery.streamDeckPlugin
```

Prefer the artifact from CI release `v*` after `streamdeck validate` / `pack` once the CLI is available.

## Submission checklist

- [ ] Maker Console org **Eminence** created; Maker Agreement signed
- [ ] Listing name/description/media filled
- [ ] Demo video attached
- [ ] Upload latest `.streamDeckPlugin`
- [ ] Confirm OS listing: Windows primary, macOS beta (accurate)
- [ ] Support / source link: GitHub repo
- [ ] Confirm **DRM protection = Yes** (requires SDKVersion 3 + MinVersion ≥ 6.9; this build uses 7.3)
- [ ] Optional: uncheck “publish automatically” until you’ve smoke-tested the DRM-processed build from Versions tab
- [ ] Submit for review (expect **4–10 business days**)

## Contact

- Maker review: maker@elgato.com  
- Makers Discord: https://discord.gg/4rTB7cYzyj  
- Project: https://github.com/crimsonvspurple/dygma-sd  
