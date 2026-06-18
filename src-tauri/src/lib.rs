mod acquire;
mod db;
mod fingerprint;
mod metadata;
mod models;
mod scanner;

use acquire::{build_commands, default_download_dir, detect_tools};
use db::Database;
use metadata::{config_path, data_dir};
use models::*;
use scanner::import_folder;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

struct AppState {
    db: Database,
    config: Mutex<AppConfig>,
    import_progress: Arc<Mutex<ImportProgress>>,
}

fn now_ts() -> i64 {
    chrono::Utc::now().timestamp()
}

fn load_config() -> AppConfig {
    let path = config_path();
    if path.exists() {
        if let Ok(text) = std::fs::read_to_string(&path) {
            if let Ok(cfg) = serde_json::from_str(&text) {
                return cfg;
            }
        }
    }
    AppConfig::default()
}

fn save_config(cfg: &AppConfig) -> Result<(), String> {
    let path = config_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let text = serde_json::to_string_pretty(cfg).map_err(|e| e.to_string())?;
    std::fs::write(path, text).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_tracks(state: tauri::State<AppState>) -> Result<Vec<Track>, String> {
    state.db.all_tracks().map_err(|e| e.to_string())
}

#[tauri::command]
fn search_tracks(state: tauri::State<AppState>, query: String) -> Result<Vec<Track>, String> {
    state.db.search_tracks(&query).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_favorites(state: tauri::State<AppState>) -> Result<Vec<Track>, String> {
    state.db.favorites().map_err(|e| e.to_string())
}

#[tauri::command]
fn get_recently_played(state: tauri::State<AppState>, limit: i64) -> Result<Vec<Track>, String> {
    state.db.recently_played(limit).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_most_played(state: tauri::State<AppState>, limit: i64) -> Result<Vec<Track>, String> {
    state.db.most_played(limit).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_artists(state: tauri::State<AppState>) -> Result<Vec<ArtistGroup>, String> {
    state
        .db
        .artists()
        .map(|rows| {
            rows.into_iter()
                .map(|(name, track_count, album_count)| ArtistGroup {
                    name,
                    track_count,
                    album_count,
                })
                .collect()
        })
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn get_albums(state: tauri::State<AppState>) -> Result<Vec<AlbumGroup>, String> {
    state
        .db
        .albums()
        .map(|rows| {
            rows.into_iter()
                .map(|(name, artist, track_count, art_path, year)| AlbumGroup {
                    name,
                    artist,
                    track_count,
                    art_path,
                    year,
                })
                .collect()
        })
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn get_tracks_by_album(
    state: tauri::State<AppState>,
    album: String,
    artist: String,
) -> Result<Vec<Track>, String> {
    state
        .db
        .tracks_by_album(&album, &artist)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn get_tracks_by_artist(state: tauri::State<AppState>, artist: String) -> Result<Vec<Track>, String> {
    state.db.tracks_by_artist(&artist).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_playlists(state: tauri::State<AppState>) -> Result<Vec<Playlist>, String> {
    state
        .db
        .playlists()
        .map(|rows| {
            rows.into_iter()
                .map(|(id, name, track_count, created_at, updated_at)| Playlist {
                    id,
                    name,
                    track_count,
                    created_at,
                    updated_at,
                })
                .collect()
        })
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn get_playlist_tracks(state: tauri::State<AppState>, playlist_id: i64) -> Result<Vec<Track>, String> {
    state.db.playlist_tracks(playlist_id).map_err(|e| e.to_string())
}

#[tauri::command]
fn create_playlist(state: tauri::State<AppState>, name: String) -> Result<i64, String> {
    state.db.create_playlist(&name, now_ts()).map_err(|e| e.to_string())
}

#[tauri::command]
fn rename_playlist(state: tauri::State<AppState>, id: i64, name: String) -> Result<(), String> {
    state.db.rename_playlist(id, &name, now_ts()).map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_playlist(state: tauri::State<AppState>, id: i64) -> Result<(), String> {
    state.db.delete_playlist(id).map_err(|e| e.to_string())
}

#[tauri::command]
fn add_to_playlist(
    state: tauri::State<AppState>,
    playlist_id: i64,
    track_id: i64,
) -> Result<(), String> {
    state
        .db
        .add_to_playlist(playlist_id, track_id, now_ts())
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn remove_from_playlist(
    state: tauri::State<AppState>,
    playlist_id: i64,
    track_id: i64,
) -> Result<(), String> {
    state
        .db
        .remove_from_playlist(playlist_id, track_id, now_ts())
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn set_favorite(state: tauri::State<AppState>, id: i64, favorite: bool) -> Result<(), String> {
    state.db.set_favorite(id, favorite).map_err(|e| e.to_string())
}

#[tauri::command]
fn record_play(state: tauri::State<AppState>, id: i64) -> Result<(), String> {
    state.db.record_play(id, now_ts()).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_stats(state: tauri::State<AppState>) -> Result<(i64, i64, i64, i64), String> {
    state.db.stats().map_err(|e| e.to_string())
}

#[tauri::command]
fn get_import_progress(state: tauri::State<AppState>) -> Result<ImportProgress, String> {
    Ok(state.import_progress.lock().unwrap().clone())
}

#[tauri::command]
fn import_music_folder(
    state: tauri::State<AppState>,
    folder: String,
) -> Result<ImportResult, String> {
    let config = state.config.lock().unwrap().clone();
    let result = import_folder(
        &state.db,
        PathBuf::from(&folder).as_path(),
        &config,
        state.import_progress.clone(),
    );
    Ok(result)
}

#[tauri::command]
fn get_config(state: tauri::State<AppState>) -> Result<AppConfig, String> {
    Ok(state.config.lock().unwrap().clone())
}

#[tauri::command]
fn save_app_config(state: tauri::State<AppState>, config: AppConfig) -> Result<(), String> {
    save_config(&config)?;
    *state.config.lock().unwrap() = config;
    Ok(())
}

#[tauri::command]
fn get_acquire_tools() -> Vec<AcquireTool> {
    detect_tools()
}

#[tauri::command]
fn get_acquire_commands(
    query: String,
    mode: String,
    output_dir: Option<String>,
) -> Vec<AcquireCommand> {
    build_commands(&query, &mode, output_dir.as_deref())
}

#[tauri::command]
fn get_default_download_dir() -> String {
    default_download_dir().to_string_lossy().to_string()
}

#[tauri::command]
fn pick_folder(title: Option<String>) -> Option<String> {
    let mut dialog = rfd::FileDialog::new();
    if let Some(t) = title {
        dialog = dialog.set_title(&t);
    }
    dialog.pick_folder().map(|p| p.to_string_lossy().to_string())
}

pub fn cli_import(folder: &str) -> Result<ImportResult, String> {
    let db_path = data_dir().join("library.db");
    let db = Database::open(&db_path).map_err(|e| e.to_string())?;
    let config = load_config();
    let progress = Arc::new(Mutex::new(ImportProgress::default()));
    Ok(import_folder(&db, std::path::Path::new(folder), &config, progress))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let db_path = data_dir().join("library.db");
    let db = Database::open(&db_path).expect("failed to open database");

    tauri::Builder::default()
        .manage(AppState {
            db,
            config: Mutex::new(load_config()),
            import_progress: Arc::new(Mutex::new(ImportProgress::default())),
        })
        .invoke_handler(tauri::generate_handler![
            get_tracks,
            search_tracks,
            get_favorites,
            get_recently_played,
            get_most_played,
            get_artists,
            get_albums,
            get_tracks_by_album,
            get_tracks_by_artist,
            get_playlists,
            get_playlist_tracks,
            create_playlist,
            rename_playlist,
            delete_playlist,
            add_to_playlist,
            remove_from_playlist,
            set_favorite,
            record_play,
            get_stats,
            get_import_progress,
            import_music_folder,
            get_config,
            save_app_config,
            get_acquire_tools,
            get_acquire_commands,
            get_default_download_dir,
            pick_folder,
        ])
        .run(tauri::generate_context!())
        .expect("error while running BAASIC Media Player");
}
