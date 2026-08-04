#!/usr/bin/env bash
# Build and install the Stream Deck plugin on macOS (or Linux community hosts).
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PLUGIN_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
SD_NAME="com.red.eminence.dygma.battery.sdPlugin"
SD_DIR="$PLUGIN_ROOT/$SD_NAME"

if [[ "$(uname -s)" == "Darwin" ]]; then
  DEST="${HOME}/Library/Application Support/com.elgato.StreamDeck/Plugins/${SD_NAME}"
elif [[ "$(uname -s)" == "Linux" ]]; then
  # Official Elgato app is not on Linux; common community path / local use.
  DEST="${STREAMDECK_PLUGINS_DIR:-${HOME}/.local/share/StreamDeck/Plugins}/${SD_NAME}"
else
  echo "Unsupported OS: $(uname -s). Use install.ps1 on Windows." >&2
  exit 1
fi

echo "Building release..."
(
  cd "$PLUGIN_ROOT"
  cargo build --release
)

BIN_DIR="$SD_DIR/bin"
mkdir -p "$BIN_DIR"
EXE="$PLUGIN_ROOT/target/release/dygma-sd-plugin"
if [[ ! -f "$EXE" ]]; then
  echo "Missing $EXE" >&2
  exit 1
fi

if [[ "$(uname -s)" == "Darwin" ]]; then
  cp -f "$EXE" "$BIN_DIR/dygma-sd-plugin-mac"
  chmod +x "$BIN_DIR/dygma-sd-plugin-mac"
else
  cp -f "$EXE" "$BIN_DIR/dygma-sd-plugin-linux"
  chmod +x "$BIN_DIR/dygma-sd-plugin-linux"
fi

mkdir -p "$(dirname "$DEST")"
rm -rf "$DEST"
cp -R "$SD_DIR" "$DEST"
echo "Installed -> $DEST"
echo "Restart Stream Deck (if applicable). Close Bazecor while the plugin owns the serial port."
