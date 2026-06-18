#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
ICON_DIR="$ROOT/src-tauri/icons"
SRC="$ICON_DIR/icon.svg"
mkdir -p "$ICON_DIR"

for size in 32 128 256 512 1024; do
  magick -background none "$SRC" -resize "${size}x${size}" "$ICON_DIR/${size}x${size}.png"
done

cp "$ICON_DIR/256x256.png" "$ICON_DIR/128x128@2x.png"
cp "$ICON_DIR/512x512.png" "$ICON_DIR/icon.png"

# ICO and ICNS via magick
magick "$ICON_DIR/256x256.png" "$ICON_DIR/32x32.png" "$ICON_DIR/128x128.png" "$ICON_DIR/icon.ico"
magick "$ICON_DIR/512x512.png" "$ICON_DIR/icon.icns" 2>/dev/null || cp "$ICON_DIR/512x512.png" "$ICON_DIR/icon.icns"

echo "Icons generated in $ICON_DIR"
