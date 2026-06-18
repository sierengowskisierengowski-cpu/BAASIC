import { invoke } from "@tauri-apps/api/core";
import type {
  AcquireCommand,
  AcquireTool,
  AlbumGroup,
  AppConfig,
  ArtistGroup,
  ImportProgress,
  Playlist,
  Track,
} from "../types";

export const api = {
  getTracks: () => invoke<Track[]>("get_tracks"),
  searchTracks: (query: string) => invoke<Track[]>("search_tracks", { query }),
  getFavorites: () => invoke<Track[]>("get_favorites"),
  getRecentlyPlayed: (limit = 50) => invoke<Track[]>("get_recently_played", { limit }),
  getMostPlayed: (limit = 50) => invoke<Track[]>("get_most_played", { limit }),
  getArtists: () => invoke<ArtistGroup[]>("get_artists"),
  getAlbums: () => invoke<AlbumGroup[]>("get_albums"),
  getTracksByAlbum: (album: string, artist: string) =>
    invoke<Track[]>("get_tracks_by_album", { album, artist }),
  getTracksByArtist: (artist: string) => invoke<Track[]>("get_tracks_by_artist", { artist }),
  getPlaylists: () => invoke<Playlist[]>("get_playlists"),
  getPlaylistTracks: (playlistId: number) =>
    invoke<Track[]>("get_playlist_tracks", { playlistId }),
  createPlaylist: (name: string) => invoke<number>("create_playlist", { name }),
  renamePlaylist: (id: number, name: string) => invoke("rename_playlist", { id, name }),
  deletePlaylist: (id: number) => invoke("delete_playlist", { id }),
  addToPlaylist: (playlistId: number, trackId: number) =>
    invoke("add_to_playlist", { playlistId, trackId }),
  removeFromPlaylist: (playlistId: number, trackId: number) =>
    invoke("remove_from_playlist", { playlistId, trackId }),
  setFavorite: (id: number, favorite: boolean) => invoke("set_favorite", { id, favorite }),
  recordPlay: (id: number) => invoke("record_play", { id }),
  getStats: () => invoke<[number, number, number, number]>("get_stats"),
  getImportProgress: () => invoke<ImportProgress>("get_import_progress"),
  importMusicFolder: (folder: string) => invoke("import_music_folder", { folder }),
  getConfig: () => invoke<AppConfig>("get_config"),
  saveConfig: (config: AppConfig) => invoke("save_app_config", { config }),
  getAcquireTools: () => invoke<AcquireTool[]>("get_acquire_tools"),
  getAcquireCommands: (query: string, mode: string, outputDir?: string) =>
    invoke<AcquireCommand[]>("get_acquire_commands", { query, mode, outputDir }),
  getDefaultDownloadDir: () => invoke<string>("get_default_download_dir"),
  pickFolder: (title?: string) => invoke<string | null>("pick_folder", { title }),
};

export function formatDuration(ms: number): string {
  if (!ms || ms < 0) return "0:00";
  const total = Math.floor(ms / 1000);
  const h = Math.floor(total / 3600);
  const m = Math.floor((total % 3600) / 60);
  const s = total % 60;
  if (h > 0) return `${h}:${String(m).padStart(2, "0")}:${String(s).padStart(2, "0")}`;
  return `${m}:${String(s).padStart(2, "0")}`;
}
