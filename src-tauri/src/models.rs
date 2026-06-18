use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Track {
    pub id: i64,
    pub path: String,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub album_artist: String,
    pub genre: String,
    pub year: Option<i32>,
    pub track_number: Option<i32>,
    pub duration_ms: i64,
    pub file_hash: String,
    pub fingerprint: Option<String>,
    pub art_path: Option<String>,
    pub added_at: i64,
    pub favorite: bool,
    pub play_count: i64,
    pub last_played_at: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcquireTool {
    pub id: String,
    pub name: String,
    pub path: String,
    pub version: String,
    pub installed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcquireCommand {
    pub tool_id: String,
    pub label: String,
    pub command: String,
    pub output_dir: String,
    pub import_hint: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Playlist {
    pub id: i64,
    pub name: String,
    pub track_count: i64,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtistGroup {
    pub name: String,
    pub track_count: i64,
    pub album_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlbumGroup {
    pub name: String,
    pub artist: String,
    pub track_count: i64,
    pub art_path: Option<String>,
    pub year: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportProgress {
    pub active: bool,
    pub phase: String,
    pub current_file: String,
    pub processed: usize,
    pub total: usize,
    pub imported: usize,
    pub skipped_duplicates: usize,
    pub tagged: usize,
    pub errors: Vec<String>,
}

impl Default for ImportProgress {
    fn default() -> Self {
        Self {
            active: false,
            phase: String::new(),
            current_file: String::new(),
            processed: 0,
            total: 0,
            imported: 0,
            skipped_duplicates: 0,
            tagged: 0,
            errors: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub acoustid_api_key: Option<String>,
    pub music_folders: Vec<String>,
    pub download_folder: Option<String>,
    pub auto_import_downloads: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            acoustid_api_key: None,
            music_folders: Vec::new(),
            download_folder: None,
            auto_import_downloads: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportResult {
    pub imported: usize,
    pub skipped_duplicates: usize,
    pub tagged: usize,
    pub errors: Vec<String>,
}
