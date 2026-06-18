import { useCallback, useEffect, useState } from "react";
import { convertFileSrc } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import {
  Download,
  FolderOpen,
  Heart,
  Library,
  ListMusic,
  Mic2,
  Music2,
  Search,
  Settings,
  Star,
  Clock,
  TrendingUp,
} from "lucide-react";
import { usePlayer } from "./hooks/usePlayer";
import { api, formatDuration } from "./lib/api";
import type { AlbumGroup, AppConfig, Playlist, Track, View } from "./types";
import { PlayerBar } from "./components/PlayerBar";
import { TrackList } from "./components/TrackList";
import { AlbumGrid } from "./components/AlbumGrid";
import { AcquirePanel } from "./components/AcquirePanel";
import { ImportPanel } from "./components/ImportPanel";
import { SettingsPanel } from "./components/SettingsPanel";

export default function App() {
  const player = usePlayer();
  const [view, setView] = useState<View>({ kind: "library" });
  const [tracks, setTracks] = useState<Track[]>([]);
  const [albums, setAlbums] = useState<AlbumGroup[]>([]);
  const [playlists, setPlaylists] = useState<Playlist[]>([]);
  const [search, setSearch] = useState("");
  const [stats, setStats] = useState<[number, number, number, number]>([0, 0, 0, 0]);
  const [config, setConfig] = useState<AppConfig | null>(null);
  const [newPlaylist, setNewPlaylist] = useState("");

  const refresh = useCallback(async () => {
    const [t, a, p, s] = await Promise.all([
      api.getTracks(),
      api.getAlbums(),
      api.getPlaylists(),
      api.getStats(),
    ]);
    setTracks(t);
    setAlbums(a);
    setPlaylists(p);
    setStats(s);
  }, []);

  const loadViewTracks = useCallback(async (): Promise<Track[]> => {
    if (search.trim()) return api.searchTracks(search.trim());
    switch (view.kind) {
      case "library":
        return api.getTracks();
      case "favorites":
        return api.getFavorites();
      case "recent":
        return api.getRecentlyPlayed();
      case "top":
        return api.getMostPlayed();
      case "artist":
        return api.getTracksByArtist(view.name);
      case "album":
        return api.getTracksByAlbum(view.name, view.artist);
      case "playlist":
        return api.getPlaylistTracks(view.id);
      default:
        return api.getTracks();
    }
  }, [view, search]);

  const [viewTracks, setViewTracks] = useState<Track[]>([]);

  useEffect(() => {
    void refresh();
    void api.getConfig().then(async (cfg) => {
      setConfig(cfg);
      if (cfg.auto_import_downloads) {
        const folder = cfg.download_folder ?? (await api.getDefaultDownloadDir());
        if (folder) {
          await api.importMusicFolder(folder);
          await refresh();
        }
      }
    });
  }, [refresh]);

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement) return;
      if (e.code === "Space") {
        e.preventDefault();
        player.togglePlay();
      } else if (e.code === "ArrowRight") {
        player.next();
      } else if (e.code === "ArrowLeft") {
        player.prev();
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [player]);

  useEffect(() => {
    void loadViewTracks().then(setViewTracks);
  }, [loadViewTracks, tracks]);

  const onPlayTrack = (track: Track, list = viewTracks) => {
    const idx = list.findIndex((t) => t.id === track.id);
    player.playTracks(list, Math.max(0, idx));
  };

  const onToggleFavorite = async (track: Track) => {
    await api.setFavorite(track.id, !track.favorite);
    await refresh();
  };

  const onCreatePlaylist = async () => {
    const name = newPlaylist.trim();
    if (!name) return;
    await api.createPlaylist(name);
    setNewPlaylist("");
    await refresh();
  };

  const onImportFolder = async () => {
    const selected = await open({ directory: true, multiple: false });
    if (!selected) return;
    await api.importMusicFolder(selected as string);
    await refresh();
  };

  const title = (() => {
    switch (view.kind) {
      case "library":
        return "All Music";
      case "albums":
        return "Albums";
      case "artists":
        return "Artists";
      case "artist":
        return view.name;
      case "album":
        return view.name;
      case "favorites":
        return "Favorites";
      case "recent":
        return "Recently Played";
      case "top":
        return "Most Played";
      case "playlist":
        return view.name;
      case "import":
        return "Import Music";
      case "acquire":
        return "Get Music";
      case "settings":
        return "Settings";
    }
  })();

  const artSrc = player.currentTrack?.art_path
    ? convertFileSrc(player.currentTrack.art_path)
    : null;

  return (
    <div className="flex h-full flex-col">
      <div className="flex min-h-0 flex-1">
        <aside className="flex w-60 shrink-0 flex-col border-r border-baasic-border bg-baasic-panel">
          <div className="border-b border-baasic-border px-4 py-5">
            <div className="flex items-center gap-3">
              <div className="flex h-10 w-10 items-center justify-center rounded-xl bg-gradient-to-br from-baasic-gold to-baasic-gold-dim text-lg font-black text-black">
                B
              </div>
              <div>
                <div className="text-sm font-bold tracking-wide">BAASIC</div>
                <div className="text-xs text-baasic-muted">Media Player</div>
              </div>
            </div>
          </div>

          <nav className="flex-1 space-y-1 overflow-y-auto p-3">
            <NavBtn icon={Library} label="Library" active={view.kind === "library"} onClick={() => setView({ kind: "library" })} />
            <NavBtn icon={Music2} label="Albums" active={view.kind === "albums"} onClick={() => setView({ kind: "albums" })} />
            <NavBtn icon={Mic2} label="Artists" active={view.kind === "artists"} onClick={() => setView({ kind: "artists" })} />
            <NavBtn icon={Heart} label="Favorites" active={view.kind === "favorites"} onClick={() => setView({ kind: "favorites" })} />
            <NavBtn icon={Clock} label="Recent" active={view.kind === "recent"} onClick={() => setView({ kind: "recent" })} />
            <NavBtn icon={TrendingUp} label="Top Played" active={view.kind === "top"} onClick={() => setView({ kind: "top" })} />

            <div className="pt-3 pb-1 text-[10px] font-semibold uppercase tracking-widest text-baasic-muted">
              Playlists
            </div>
            {playlists.map((pl) => (
              <NavBtn
                key={pl.id}
                icon={ListMusic}
                label={pl.name}
                sub={`${pl.track_count}`}
                active={view.kind === "playlist" && view.id === pl.id}
                onClick={() => setView({ kind: "playlist", id: pl.id, name: pl.name })}
              />
            ))}
            <div className="flex gap-1 pt-1">
              <input
                value={newPlaylist}
                onChange={(e) => setNewPlaylist(e.target.value)}
                onKeyDown={(e) => e.key === "Enter" && void onCreatePlaylist()}
                placeholder="New playlist"
                className="min-w-0 flex-1 rounded-lg border border-baasic-border bg-baasic-panel-2 px-2 py-1.5 text-xs outline-none focus:border-baasic-gold"
              />
              <button
                onClick={() => void onCreatePlaylist()}
                className="rounded-lg bg-baasic-gold px-2 text-xs font-bold text-black"
              >
                +
              </button>
            </div>

            <div className="pt-3 pb-1 text-[10px] font-semibold uppercase tracking-widest text-baasic-muted">
              Tools
            </div>
            <NavBtn icon={FolderOpen} label="Import" active={view.kind === "import"} onClick={() => setView({ kind: "import" })} />
            <NavBtn icon={Download} label="Get Music" active={view.kind === "acquire"} onClick={() => setView({ kind: "acquire" })} />
            <NavBtn icon={Settings} label="Settings" active={view.kind === "settings"} onClick={() => setView({ kind: "settings" })} />
          </nav>

          <div className="border-t border-baasic-border p-3 text-xs text-baasic-muted">
            <div>{stats[0]} tracks · {stats[1]} albums · {stats[2]} artists</div>
            <div className="mt-1 flex items-center gap-1 text-baasic-gold">
              <Star size={12} /> {stats[3]} favorites
            </div>
          </div>
        </aside>

        <main className="flex min-w-0 flex-1 flex-col">
          <header className="flex items-center gap-3 border-b border-baasic-border px-6 py-4">
            <h1 className="text-xl font-semibold">{title}</h1>
            {!["import", "acquire", "settings", "albums", "artists"].includes(view.kind) && (
              <div className="relative ml-auto w-72">
                <Search className="absolute top-2.5 left-3 text-baasic-muted" size={16} />
                <input
                  value={search}
                  onChange={(e) => setSearch(e.target.value)}
                  placeholder="Search songs, artists, albums..."
                  className="w-full rounded-full border border-baasic-border bg-baasic-panel-2 py-2 pr-3 pl-9 text-sm outline-none focus:border-baasic-gold"
                />
              </div>
            )}
            {view.kind === "library" && (
              <button
                onClick={() => void onImportFolder()}
                className="ml-auto rounded-full border border-baasic-border px-4 py-2 text-sm hover:border-baasic-gold"
              >
                Import Folder
              </button>
            )}
          </header>

          <div className="min-h-0 flex-1 overflow-y-auto p-6">
            {view.kind === "import" && <ImportPanel onDone={refresh} />}
            {view.kind === "acquire" && <AcquirePanel onImportFolder={onImportFolder} />}
            {view.kind === "settings" && config && (
              <SettingsPanel
                config={config}
                onSave={async (c) => {
                  await api.saveConfig(c);
                  setConfig(c);
                }}
              />
            )}
            {view.kind === "albums" && (
              <AlbumGrid
                albums={albums}
                onSelect={(album) => setView({ kind: "album", name: album.name, artist: album.artist })}
              />
            )}
            {view.kind === "artists" && <ArtistsView onSelect={(name) => setView({ kind: "artist", name })} />}
            {!["import", "acquire", "settings", "albums", "artists"].includes(view.kind) && (
              <TrackList
                tracks={viewTracks}
                currentId={player.currentTrack?.id ?? null}
                playlists={playlists}
                onPlay={onPlayTrack}
                onToggleFavorite={onToggleFavorite}
                onAddToPlaylist={async (playlistId, trackId) => {
                  await api.addToPlaylist(playlistId, trackId);
                  await refresh();
                }}
              />
            )}
          </div>
        </main>

        {player.currentTrack && (
          <aside className="hidden w-72 shrink-0 border-l border-baasic-border bg-baasic-panel p-4 xl:block">
            <div className="text-xs font-semibold uppercase tracking-widest text-baasic-muted">Now Playing</div>
            <div className="mt-4 aspect-square overflow-hidden rounded-2xl bg-baasic-panel-2">
              {artSrc ? (
                <img src={artSrc} alt="" className="h-full w-full object-cover" />
              ) : (
                <div className="flex h-full items-center justify-center text-5xl font-black text-baasic-gold">B</div>
              )}
            </div>
            <div className="mt-4 text-lg font-semibold">{player.currentTrack.title}</div>
            <div className="text-baasic-muted">{player.currentTrack.artist}</div>
            <div className="mt-1 text-sm text-baasic-muted">{player.currentTrack.album}</div>
            <div className="mt-2 text-xs text-baasic-muted">
              {formatDuration(player.currentTrack.duration_ms)}
              {player.currentTrack.genre ? ` · ${player.currentTrack.genre}` : ""}
            </div>
          </aside>
        )}
      </div>

      <PlayerBar player={player} />
    </div>
  );
}

function NavBtn({
  icon: Icon,
  label,
  sub,
  active,
  onClick,
}: {
  icon: React.ComponentType<{ size?: number }>;
  label: string;
  sub?: string;
  active: boolean;
  onClick: () => void;
}) {
  return (
    <button
      onClick={onClick}
      className={`flex w-full items-center gap-2 rounded-lg px-3 py-2 text-left text-sm transition ${
        active ? "bg-baasic-gold/15 text-baasic-gold" : "text-baasic-text hover:bg-baasic-panel-2"
      }`}
    >
      <Icon size={16} />
      <span className="flex-1 truncate">{label}</span>
      {sub && <span className="text-xs text-baasic-muted">{sub}</span>}
    </button>
  );
}

function ArtistsView({ onSelect }: { onSelect: (name: string) => void }) {
  const [artists, setArtists] = useState<{ name: string; track_count: number; album_count: number }[]>([]);
  useEffect(() => {
    void api.getArtists().then(setArtists);
  }, []);
  return (
    <div className="grid grid-cols-2 gap-3 md:grid-cols-3 lg:grid-cols-4">
      {artists.map((a) => (
        <button
          key={a.name}
          onClick={() => onSelect(a.name)}
          className="rounded-xl border border-baasic-border bg-baasic-panel-2 p-4 text-left hover:border-baasic-gold"
        >
          <div className="mb-3 flex h-12 w-12 items-center justify-center rounded-full bg-baasic-gold/20 text-lg font-bold text-baasic-gold">
            {a.name.charAt(0).toUpperCase()}
          </div>
          <div className="truncate font-medium">{a.name}</div>
          <div className="text-xs text-baasic-muted">
            {a.track_count} tracks · {a.album_count} albums
          </div>
        </button>
      ))}
    </div>
  );
}
