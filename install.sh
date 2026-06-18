#!/usr/bin/env bash
# BAASIC Media Player — one-command installer for Arch Linux
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
INSTALL_BIN="${INSTALL_BIN:-$HOME/.local/bin/baasic-media-player}"
DESKTOP_FILE="$HOME/.local/share/applications/baasic-media-player.desktop"
ICON_DIR="$HOME/.local/share/icons/hicolor"
MUSIC_DIR="$HOME/Music/BAASIC"
GET_SCRIPT="$HOME/.local/bin/baasic-get"

echo "╔══════════════════════════════════════╗"
echo "║   BAASIC Media Player — Installer    ║"
echo "╚══════════════════════════════════════╝"
echo

# ── 1. System dependencies ──────────────────────────────────────────
echo "→ Checking system dependencies..."
MISSING=()
for pkg in pnpm rust chromaprint ffmpeg yt-dlp imagemagick webkit2gtk base-devel; do
  if ! pacman -Qi "$pkg" &>/dev/null; then
    MISSING+=("$pkg")
  fi
done

if [ ${#MISSING[@]} -gt 0 ]; then
  echo "  Installing: ${MISSING[*]}"
  sudo pacman -S --needed --noconfirm "${MISSING[@]}"
else
  echo "  All system packages present."
fi

# ── 2. Build ──────────────────────────────────────────────────────
echo "→ Building BAASIC..."
cd "$ROOT"
pnpm install --frozen-lockfile 2>/dev/null || pnpm install
pnpm approve-builds esbuild 2>/dev/null || true
bash scripts/generate-icons.sh
pnpm build
cd src-tauri
cargo build --release
cd "$ROOT"

BINARY="$ROOT/src-tauri/target/release/baasic-media-player"
if [ ! -f "$BINARY" ]; then
  echo "ERROR: Build failed — binary not found at $BINARY"
  exit 1
fi

# ── 3. Install binary + helper scripts ────────────────────────────
echo "→ Installing to $INSTALL_BIN..."
mkdir -p "$(dirname "$INSTALL_BIN")" "$MUSIC_DIR"

install -m 755 "$BINARY" "$INSTALL_BIN"

cat > "$GET_SCRIPT" << 'SCRIPT'
#!/usr/bin/env bash
# Download music then auto-import into BAASIC (dedup + fingerprint + tag)
set -euo pipefail
QUERY="${1:?Usage: baasic-get \"Artist - Song\"}"
MODE="${2:-song}"
OUT="${BAASIC_MUSIC_DIR:-$HOME/Music/BAASIC}"
mkdir -p "$OUT"

if command -v yt-dlp &>/dev/null; then
  case "$MODE" in
    album)  SEARCH="ytsearch10:${QUERY} full album" ;;
    artist) SEARCH="ytsearch15:${QUERY}" ;;
    *)      SEARCH="ytsearch1:${QUERY}" ;;
  esac
  yt-dlp "$SEARCH" -x --audio-format mp3 --audio-quality 0 \
    --embed-thumbnail --add-metadata \
    -o "${OUT}/%(artist|Unknown)s - %(title)s.%(ext)s"
else
  echo "yt-dlp not found. Install: sudo pacman -S yt-dlp"
  exit 1
fi

echo "→ Importing into BAASIC library..."
baasic-media-player --import-folder "$OUT"
echo "Done! Open BAASIC Media Player from your app menu."
SCRIPT
chmod +x "$GET_SCRIPT"

# ── 4. Desktop launcher + icon ────────────────────────────────────
echo "→ Installing desktop launcher..."
mkdir -p "$ICON_DIR/256x256/apps" "$ICON_DIR/32x32/apps"
cp "$ROOT/src-tauri/icons/256x256.png" "$ICON_DIR/256x256/apps/baasic-media-player.png"
cp "$ROOT/src-tauri/icons/32x32.png" "$ICON_DIR/32x32/apps/baasic-media-player.png"

cat > "$DESKTOP_FILE" << DESKTOP
[Desktop Entry]
Type=Application
Name=BAASIC Media Player
Comment=Local music library with smart import and fingerprinting
Exec=${INSTALL_BIN}
Icon=baasic-media-player
Terminal=false
Categories=Audio;Music;Player;
StartupWMClass=baasic-media-player
DESKTOP

update-desktop-database "$HOME/.local/share/applications" 2>/dev/null || true

# ── 5. Default config ─────────────────────────────────────────────
CONFIG_DIR="$HOME/.config/baasic-media-player"
mkdir -p "$CONFIG_DIR"
if [ ! -f "$CONFIG_DIR/config.json" ]; then
  cat > "$CONFIG_DIR/config.json" << JSON
{
  "acoustid_api_key": null,
  "music_folders": [],
  "download_folder": "${MUSIC_DIR}",
  "auto_import_downloads": true
}
JSON
fi

echo
echo "╔══════════════════════════════════════╗"
echo "║         Installation complete!        ║"
echo "╚══════════════════════════════════════╝"
echo
echo "  Launch:  BAASIC Media Player (app menu)"
echo "           or: baasic-media-player"
echo
echo "  Download + import in one command:"
echo "    baasic-get \"Radiohead - Creep\""
echo "    baasic-get \"Pink Floyd\" artist"
echo
echo "  Music folder: $MUSIC_DIR"
echo "  Optional: add AcoustID key in Settings for auto-tagging"
echo
