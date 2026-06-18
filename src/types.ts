export interface Track {
  id: number;
  path: string;
  title: string;
  artist: string;
  album: string;
  album_artist: string;
  genre: string;
  year: number | null;
  track_number: number | null;
  duration_ms: number;
  file_hash: string;
  fingerprint: string | null;
  art_path: string | null;
  added_at: number;
  favorite: boolean;
  play_count: number;
  last_played_at: number | null;
}

export interface Playlist {
  id: number;
  name: string;
  track_count: number;
  created_at: number;
  updated_at: number;
}

export interface ArtistGroup {
  name: string;
  track_count: number;
  album_count: number;
}

export interface AlbumGroup {
  name: string;
  artist: string;
  track_count: number;
  art_path: string | null;
  year: number | null;
}

export interface ImportProgress {
  active: boolean;
  phase: string;
  current_file: string;
  processed: number;
  total: number;
  imported: number;
  skipped_duplicates: number;
  tagged: number;
  errors: string[];
}

export interface AppConfig {
  acoustid_api_key: string | null;
  music_folders: string[];
  download_folder: string | null;
  auto_import_downloads: boolean;
}

export interface AcquireTool {
  id: string;
  name: string;
  path: string;
  version: string;
  installed: boolean;
}

export interface AcquireCommand {
  tool_id: string;
  label: string;
  command: string;
  output_dir: string;
  import_hint: string;
}

export type View =
  | { kind: "library" }
  | { kind: "albums" }
  | { kind: "artists" }
  | { kind: "artist"; name: string }
  | { kind: "album"; name: string; artist: string }
  | { kind: "favorites" }
  | { kind: "recent" }
  | { kind: "top" }
  | { kind: "playlist"; id: number; name: string }
  | { kind: "import" }
  | { kind: "acquire" }
  | { kind: "settings" };

export type RepeatMode = "off" | "all" | "one";
