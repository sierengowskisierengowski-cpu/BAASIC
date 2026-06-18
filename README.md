# BAASIC Media Player

A local-first desktop music player with smart library management, duplicate detection, audio fingerprinting, and a built-in download command builder.

![BAASIC](src-tauri/icons/256x256.png)

## One-command install (Arch Linux)

```bash
git clone https://github.com/sierengowskisierengowski-cpu/BAASIC.git
cd BAASIC
git pull   # get latest installer fixes
chmod +x install.sh
./install.sh
```

The installer auto-installs every missing dependency **one at a time** — it won't stop on the old `webkit2gtk` package. On Arch it uses `webkit2gtk-4.1`. Optional packages (like `yt-dlp`) failing won't abort the whole run.

## Download music in one command

After install:

```bash
baasic-get "Artist - Song Title"
baasic-get "Album Name" album
baasic-get "Artist Name" artist
```

This downloads via `yt-dlp`, then auto-imports into your library with dedup + fingerprinting.

## Features

| Feature | Description |
|---------|-------------|
| **Playback** | Play/pause, next/prev, seek, shuffle, repeat, volume |
| **Library** | Artists, albums, search, favorites, recently played |
| **Playlists** | Create, save, add/remove tracks |
| **Smart import** | SHA-256 dedup + Chromaprint fingerprint dedup |
| **Auto-tagging** | AcoustID + MusicBrainz for blank/unknown files |
| **Album art** | Downloaded from Cover Art Archive |
| **Get Music** | In-app terminal command builder for yt-dlp/spotdl |
| **Auto-import** | Scans download folder on startup |

## Manual dev run

```bash
pnpm install
pnpm desktop:dev
```

## Settings

- **AcoustID API key** — free at [acoustid.org](https://acoustid.org/new-application) for auto-identifying untagged songs
- **Download folder** — defaults to `~/Music/BAASIC`
- **Auto-import** — on by default; scans download folder each time you open the app

## Data locations

| Path | Purpose |
|------|---------|
| `~/.local/share/baasic-media-player/library.db` | Music library database |
| `~/.local/share/baasic-media-player/art/` | Album artwork cache |
| `~/.config/baasic-media-player/config.json` | App settings |
| `~/Music/BAASIC/` | Default download folder |

## CLI import

```bash
baasic-media-player --import-folder ~/Music/BAASIC
```

## Requirements

- Arch Linux (or similar with `pacman`)
- `pnpm`, `rustc`, `cargo`
- `chromaprint` (fpcalc), `ffmpeg`, `webkit2gtk-4.1`

## License

MIT
