# Dygma Defy Battery Level – Programmatic Access Instructions

## Goal
Read the battery percentage of both halves of a wireless Dygma Defy (or Raise 2) **while the sides are running wirelessly** (no USB cables attached to the halves). This is the same condition under which Bazecor displays the battery levels.

## Architecture Summary

| Mode              | Neuron connection to PC | Sides connection          | Battery levels readable via Focus? | Notes |
|-------------------|-------------------------|---------------------------|------------------------------------|-------|
| RF (2.4 GHz)      | USB                     | RF to Neuron              | **Yes**                            | Primary use case |
| Wired             | USB                     | USB cables to Neuron      | Limited (often only “charging”)    | Hardware limitation of fuel-gauge chip |
| Bluetooth         | None (Neuron docked under left half) | RF between halves + BT to PC | No                                 | Focus serial port not available |

**Required setup for programmatic reading**
- Neuron plugged into the computer via USB.
- Both keyboard halves powered on and communicating with the Neuron over RF (no side cables).
- Bazecor closed (or not holding the serial port).

## Focus Serial API

Communication is a simple text protocol over the Neuron’s USB CDC serial interface.

### Port identification
- Linux: typically `/dev/ttyACM0` (or the next available ACM device belonging to the Dygma Neuron)
- macOS / Windows: the corresponding COM / cu.usbmodem device

### Protocol rules
1. Send a command string followed by `\n`.
2. Read lines until a line containing only `.` is received (end-of-response marker).
3. The useful payload is the line(s) before the `.`.

### Battery-related commands

```
wireless.battery.forceRead          # Force Neuron to query both sides (recommended first)
wireless.battery.left.level         # Returns 0–100
wireless.battery.right.level        # Returns 0–100
wireless.battery.left.status        # Status code (see below)
wireless.battery.right.status       # Status code
wireless.battery.savingMode         # Get/set energy-saving mode (0/1)
```

**Status codes (approximate)**
- 1 = discharging / normal
- 2 = charging
- 3 = fully charged
- 4 = fault / read error
- 5 = physically disconnected from RF board

### Recommended query sequence
```
wireless.battery.forceRead
<wait 1–2 seconds>
wireless.battery.left.level
wireless.battery.right.level
```

## Existing Tools & Libraries

### Ready-to-use CLI
- **dygma-indicator** (Go)  
  https://github.com/coolapso/dygma-indicator  
  - Designed for exactly this RF scenario.  
  - Outputs JSON suitable for Waybar / scripts:  
    `{"text":"L:50% R:70%","tooltip":"...","percentage":50}`  
  - Install via AUR, `go install`, or release binaries.

### Libraries
- **Rust**: `dygma_focus` crate  
  High-level methods:  
  `wireless_battery_level_left_get()`, `wireless_battery_level_right_get()`,  
  `wireless_battery_force_read()`, status getters, etc.
- Any language with serial support (Python + pyserial, Node, etc.) can implement the protocol directly.

### Official documentation
- Focus API reference: https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md
- Bazecor source (how the official app does it): https://github.com/Dygmalab/Bazecor

## Important Caveats
- Pure Bluetooth mode (Neuron under the left half, no USB to PC) does **not** expose the Focus serial port → programmatic battery reading is unavailable.
- While a side is charging over a physical cable, the percentage is frequently hidden (hardware limitation). Levels are reliable only when the side is running on battery over RF.
- Readings can occasionally become stale. Calling `forceRead` and waiting, or power-cycling the affected side, refreshes them.
- Keep firmware reasonably current; battery reporting has improved across 1.2.x → 2.x releases.
- Only one process can hold the serial port at a time. Close Bazecor before querying.

## Minimal Usage Patterns

### One-shot CLI (recommended starting point)
```bash
dygma-indicator
```

### Python sketch (pyserial)
```python
import serial, time

ser = serial.Serial('/dev/ttyACM0', 115200, timeout=1)
ser.write(b'wireless.battery.forceRead\n')
time.sleep(1.5)
ser.write(b'wireless.battery.left.level\n')
# read until '.' then parse the number
# same for right side
ser.close()
```

### Rust (using dygma_focus)
```rust
let mut focus = Focus::new_first_available()?;
focus.wireless_battery_force_read()?;
std::thread::sleep(Duration::from_secs(2));
let left  = focus.wireless_battery_level_left_get()?;
let right = focus.wireless_battery_level_right_get()?;
```

## Visual fallback (no software needed)
Assign the “Battery Level” key in Bazecor (Wireless submenu). Holding it lights the inner-column LEDs:
- 3 green = 70–100 %
- 2 green = 40–70 %
- 1 green = 10–40 %
- 1 red   = < 10 %
- Pulsing green = charging

## Summary for automation
To programmatically obtain battery levels of a wireless Dygma Defy without attaching cables to the sides:

1. Ensure Neuron is USB-connected and sides are in RF mode.
2. Open the Neuron’s serial port.
3. Send `wireless.battery.forceRead`, wait ~1–2 s.
4. Query `wireless.battery.left.level` and `wireless.battery.right.level`.
5. Parse the numeric responses.

This matches the exact condition under which Bazecor displays live battery percentages.
