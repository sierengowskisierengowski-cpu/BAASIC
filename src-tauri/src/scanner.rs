use crate::db::{Database, InsertTrack};
use crate::fingerprint::compute_fingerprint;
use crate::metadata::{identify_track, read_metadata, write_tags, art_dir};
use crate::models::{AppConfig, ImportProgress, ImportResult};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use walkdir::WalkDir;

const AUDIO_EXTENSIONS: &[&str] = &["mp3", "flac", "m4a", "ogg", "opus", "wav", "aac", "wma"];

pub fn collect_audio_files(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for entry in WalkDir::new(root).follow_links(false).into_iter().flatten() {
        let path = entry.path();
        if path.is_file() && is_audio(path) {
            files.push(path.to_path_buf());
        }
    }
    files.sort();
    files
}

fn is_audio(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| AUDIO_EXTENSIONS.contains(&e.to_ascii_lowercase().as_str()))
        .unwrap_or(false)
}

pub fn file_hash(path: &Path) -> Result<String, String> {
    let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
    let hash = Sha256::digest(&bytes);
    Ok(hex::encode(hash))
}

pub fn import_folder(
    db: &Database,
    folder: &Path,
    config: &AppConfig,
    progress: Arc<Mutex<ImportProgress>>,
) -> ImportResult {
    let files = collect_audio_files(folder);
    let total = files.len();
    let now = chrono::Utc::now().timestamp();

    {
        let mut p = progress.lock().unwrap();
        p.active = true;
        p.phase = "Scanning".into();
        p.total = total;
        p.processed = 0;
        p.imported = 0;
        p.skipped_duplicates = 0;
        p.tagged = 0;
        p.errors.clear();
    }

    let mut result = ImportResult {
        imported: 0,
        skipped_duplicates: 0,
        tagged: 0,
        errors: Vec::new(),
    };

    let art_directory = art_dir();

    for path in files {
        let path_str = path.to_string_lossy().to_string();
        {
            let mut p = progress.lock().unwrap();
            p.current_file = path_str.clone();
            p.phase = "Processing".into();
        }

        if db.path_exists(&path_str).unwrap_or(false) {
            result.skipped_duplicates += 1;
            bump_progress(&progress, result.imported, result.skipped_duplicates, result.tagged);
            continue;
        }

        let hash = match file_hash(&path) {
            Ok(h) => h,
            Err(e) => {
                result.errors.push(format!("{path_str}: hash failed: {e}"));
                bump_progress(&progress, result.imported, result.skipped_duplicates, result.tagged);
                continue;
            }
        };

        if db.hash_exists(&hash).unwrap_or(false) {
            result.skipped_duplicates += 1;
            bump_progress(&progress, result.imported, result.skipped_duplicates, result.tagged);
            continue;
        }

        let mut meta = match read_metadata(&path) {
            Ok(m) => m,
            Err(e) => {
                result.errors.push(format!("{path_str}: metadata failed: {e}"));
                bump_progress(&progress, result.imported, result.skipped_duplicates, result.tagged);
                continue;
            }
        };

        let mut fingerprint = None;
        let mut acoustid = None;
        let mut recording_id = None;
        let mut release_id = None;
        let mut art_path = None;

        if let Ok(fp) = compute_fingerprint(&path) {
            if db.fingerprint_exists(&fp.fingerprint).unwrap_or(false) {
                result.skipped_duplicates += 1;
                bump_progress(&progress, result.imported, result.skipped_duplicates, result.tagged);
                continue;
            }
            fingerprint = Some(fp.fingerprint.clone());

            if meta.needs_identification {
                if let Some(key) = config.acoustid_api_key.as_deref().filter(|k| !k.is_empty()) {
                    {
                        let mut p = progress.lock().unwrap();
                        p.phase = "Fingerprinting".into();
                    }
                    let hint = hash.chars().take(12).collect::<String>();
                    if let Ok(id) = identify_track(
                        &fp.fingerprint,
                        fp.duration_secs,
                        key,
                        &art_directory,
                        &hint,
                    ) {
                        let _ = write_tags(&path, &id);
                        meta.title = id.title;
                        meta.artist = id.artist;
                        meta.album = id.album;
                        meta.year = id.year;
                        acoustid = id.acoustid;
                        recording_id = id.recording_id;
                        release_id = id.release_id;
                        art_path = id.art_path;
                        result.tagged += 1;
                    }
                }
            }
        }

        let file_size = std::fs::metadata(&path).map(|m| m.len() as i64).unwrap_or(0);

        let insert = InsertTrack {
            path: path_str,
            title: meta.title,
            artist: meta.artist,
            album: meta.album,
            album_artist: meta.album_artist,
            genre: meta.genre,
            year: meta.year,
            track_number: meta.track_number,
            disc_number: meta.disc_number,
            duration_ms: meta.duration_ms,
            file_hash: hash,
            fingerprint,
            acoustid,
            musicbrainz_recording_id: recording_id,
            musicbrainz_release_id: release_id,
            art_path,
            file_size,
            added_at: now,
        };

        match db.insert_track(&insert) {
            Ok(_) => result.imported += 1,
            Err(e) => result.errors.push(format!("insert failed: {e}")),
        }

        bump_progress(&progress, result.imported, result.skipped_duplicates, result.tagged);
    }

    {
        let mut p = progress.lock().unwrap();
        p.active = false;
        p.phase = "Done".into();
        p.processed = total;
    }

    result
}

fn bump_progress(
    progress: &Arc<Mutex<ImportProgress>>,
    imported: usize,
    skipped: usize,
    tagged: usize,
) {
    let mut p = progress.lock().unwrap();
    p.processed += 1;
    p.imported = imported;
    p.skipped_duplicates = skipped;
    p.tagged = tagged;
}
