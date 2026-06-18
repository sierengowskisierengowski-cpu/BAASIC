#!/usr/bin/env bash
# BAASIC Media Player — one-command installer for Arch Linux
# Installs missing deps one-by-one; never aborts the whole run on a single package failure.
set -uo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
INSTALL_BIN="${INSTALL_BIN:-$HOME/.local/bin/baasic-media-player}"
DESKTOP_FILE="$HOME/.local/share/applications/baasic-media-player.desktop"
ICON_DIR="$HOME/.local/share/icons/hicolor"
MUSIC_DIR="$HOME/Music/BAASIC"
GET_SCRIPT="$HOME/.local/bin/baasic-get"
LOG_FILE="${BAASIC_INSTALL_LOG:-$HOME/.cache/baasic-install.log}"

mkdir -p "$(dirname "$LOG_FILE")"
: >"$LOG_FILE"

log()  { echo "$@" | tee -a "$LOG_FILE"; }
warn() { echo "  ⚠ $*" | tee -a "$LOG_FILE"; }
ok()   { echo "  ✓ $*" | tee -a "$LOG_FILE"; }
fail() { echo "  ✗ $*" | tee -a "$LOG_FILE"; }

FAILED_REQUIRED=()
FAILED_OPTIONAL=()

have_pkg() {
  pacman -Qi "$1" &>/dev/null
}

have_cmd() {
  command -v "$1" &>/dev/null
}

# Install one pacman package; try alternate names; never kills the whole script.
install_pacman_pkg() {
  local pkg="$1"
  local required="${2:-required}" # required | optional
  shift 2 || true
  local alternates=("$@")

  if have_pkg "$pkg"; then
    ok "$pkg already installed"
    return 0
  fi

  local candidates=("$pkg" "${alternates[@]}")
  for candidate in "${candidates[@]}"; do
    [ -z "$candidate" ] && continue
    if ! pacman -Si "$candidate" &>/dev/null; then
      continue
    fi
    log "  → Installing $candidate..."
    if sudo pacman -S --needed --noconfirm "$candidate" >>"$LOG_FILE" 2>&1; then
      ok "$candidate installed"
      return 0
    fi
    warn "$candidate failed — trying next option if any"
  done

  if [ "$required" = "required" ]; then
    FAILED_REQUIRED+=("$pkg")
    fail "REQUIRED package missing: $pkg"
  else
    FAILED_OPTIONAL+=("$pkg")
    warn "optional package skipped: $pkg"
  fi
  return 1
}

ensure_pnpm() {
  if have_cmd pnpm; then
    ok "pnpm $(pnpm --version)"
    return 0
  fi
  install_pacman_pkg pnpm optional || true
  if have_cmd pnpm; then return 0; fi
  if have_cmd npm; then
    log "  → Installing pnpm via npm..."
    npm install -g pnpm >>"$LOG_FILE" 2>&1 && ok "pnpm installed via npm" && return 0
  fi
  if have_cmd corepack; then
    corepack enable >>"$LOG_FILE" 2>&1
    corepack prepare pnpm@latest --activate >>"$LOG_FILE" 2>&1 && ok "pnpm via corepack" && return 0
  fi
  FAILED_REQUIRED+=("pnpm")
  fail "pnpm not available"
  return 1
}

ensure_rust() {
  if have_cmd cargo && have_cmd rustc; then
    ok "rust $(rustc --version | awk '{print $2}')"
    return 0
  fi
  install_pacman_pkg rust required
  have_cmd cargo
}

echo "╔══════════════════════════════════════╗"
echo "║   BAASIC Media Player — Installer    ║"
echo "╚══════════════════════════════════════╝"
echo "Log: $LOG_FILE"
echo

# ── 1. Sync package database ───────────────────────────────────────
log "→ Syncing package database..."
sudo pacman -Sy --noconfirm >>"$LOG_FILE" 2>&1 || warn "pacman sync had issues (continuing)"

# ── 2. System dependencies (one at a time, never batch-fail) ───────
log "→ Installing system dependencies (auto, non-interactive)..."

# REQUIRED for building the Tauri app on Arch
install_pacman_pkg base-devel   required
install_pacman_pkg gtk3         required
install_pacman_pkg openssl      required
install_pacman_pkg curl         required
install_pacman_pkg wget         required
install_pacman_pkg file           optional
install_pacman_pkg librsvg      optional
# Arch extra: webkit2gtk is deprecated — ONLY use webkit2gtk-4.1 (libsoup3)
install_pacman_pkg webkit2gtk-4.1 required webkit2gtk-4.1-docs

ensure_rust || true
ensure_pnpm || true

# REQUIRED / strongly recommended for BAASIC features
install_pacman_pkg chromaprint    required   # fpcalc fingerprinting
install_pacman_pkg ffmpeg         required
install_pacman_pkg imagemagick    optional   # icon generation

# OPTIONAL — Get Music helper; install continues if this fails
install_pacman_pkg yt-dlp         optional

# ── 3. Build (continues even if optional deps failed) ─────────────
if [ ${#FAILED_REQUIRED[@]} -gt 0 ]; then
  echo
  fail "Missing required packages: ${FAILED_REQUIRED[*]}"
  fail "Fix with: sudo pacman -S ${FAILED_REQUIRED[*]}"
  fail "Then re-run: ./install.sh"
  echo "  (Build skipped until required packages are installed.)"
  exit 1
fi

log "→ Building BAASIC..."
cd "$ROOT"

log "  → pnpm install..."
if ! pnpm install >>"$LOG_FILE" 2>&1; then
  pnpm install --no-frozen-lockfile >>"$LOG_FILE" 2>&1 || {
    fail "pnpm install failed — see $LOG_FILE"
    exit 1
  }
fi
pnpm approve-builds esbuild >>"$LOG_FILE" 2>&1 || true

if [ -f scripts/generate-icons.sh ]; then
  log "  → Generating icons..."
  bash scripts/generate-icons.sh >>"$LOG_FILE" 2>&1 || warn "icon generation failed (using existing icons)"
fi

log "  → Building frontend..."
pnpm build >>"$LOG_FILE" 2>&1 || {
  fail "frontend build failed — see $LOG_FILE"
  exit 1
}

log "  → Building backend (this can take a few minutes)..."
cd src-tauri
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$ROOT/src-tauri/target}"
if ! cargo build --release >>"$LOG_FILE" 2>&1; then
  fail "cargo build failed — see $LOG_FILE"
  tail -20 "$LOG_FILE"
  exit 1
fi
cd "$ROOT"

BINARY="$CARGO_TARGET_DIR/release/baasic-media-player"
if [ ! -f "$BINARY" ]; then
  BINARY="$ROOT/src-tauri/target/release/baasic-media-player"
fi
if [ ! -f "$BINARY" ]; then
  fail "Build finished but binary not found"
  exit 1
fi
ok "Built $BINARY"

# ── 4. Install binary + helper scripts ────────────────────────────
log "→ Installing to $INSTALL_BIN..."
mkdir -p "$(dirname "$INSTALL_BIN")" "$MUSIC_DIR"
install -m 755 "$BINARY" "$INSTALL_BIN"
ok "Installed binary"

cat > "$GET_SCRIPT" << 'SCRIPT'
#!/usr/bin/env bash
set -euo pipefail
QUERY="${1:?Usage: baasic-get \"Artist - Song\"}"
MODE="${2:-song}"
OUT="${BAASIC_MUSIC_DIR:-$HOME/Music/BAASIC}"
mkdir -p "$OUT"
if ! command -v yt-dlp &>/dev/null; then
  echo "yt-dlp not installed. Run: sudo pacman -S yt-dlp"
  echo "Or use Get Music inside BAASIC to copy a download command."
  exit 1
fi
case "$MODE" in
  album)  SEARCH="ytsearch10:${QUERY} full album" ;;
  artist) SEARCH="ytsearch15:${QUERY}" ;;
  *)      SEARCH="ytsearch1:${QUERY}" ;;
esac
yt-dlp "$SEARCH" -x --audio-format mp3 --audio-quality 0 \
  --embed-thumbnail --add-metadata \
  -o "${OUT}/%(artist|Unknown)s - %(title)s.%(ext)s"
echo "→ Importing into BAASIC library..."
baasic-media-player --import-folder "$OUT"
echo "Done! Open BAASIC Media Player from your app menu."
SCRIPT
chmod +x "$GET_SCRIPT"
ok "Installed baasic-get helper"

# ── 5. Desktop launcher + icon ────────────────────────────────────
log "→ Installing desktop launcher..."
mkdir -p "$ICON_DIR/256x256/apps" "$ICON_DIR/32x32/apps"
if [ -f "$ROOT/src-tauri/icons/256x256.png" ]; then
  cp "$ROOT/src-tauri/icons/256x256.png" "$ICON_DIR/256x256/apps/baasic-media-player.png"
  cp "$ROOT/src-tauri/icons/32x32.png" "$ICON_DIR/32x32/apps/baasic-media-player.png"
fi

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
ok "Desktop launcher installed"

# ── 6. Default config ─────────────────────────────────────────────
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
echo "  Download + import:"
echo "    baasic-get \"Artist - Song\""
echo
echo "  Music folder: $MUSIC_DIR"
echo "  Install log:  $LOG_FILE"
if [ ${#FAILED_OPTIONAL[@]} -gt 0 ]; then
  echo
  warn "Optional packages not installed: ${FAILED_OPTIONAL[*]}"
  warn "Install later: sudo pacman -S ${FAILED_OPTIONAL[*]}"
fi
echo
