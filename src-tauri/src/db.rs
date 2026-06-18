use crate::models::Track;
use rusqlite::{params, Connection, OptionalExtension};
use std::path::PathBuf;
use std::sync::Mutex;

pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    pub fn open(path: &PathBuf) -> rusqlite::Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        let conn = Connection::open(path)?;
        let db = Self { conn: Mutex::new(conn) };
        db.migrate()?;
        Ok(db)
    }

    fn migrate(&self) -> rusqlite::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS tracks (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                path TEXT UNIQUE NOT NULL,
                title TEXT NOT NULL DEFAULT 'Unknown Title',
                artist TEXT NOT NULL DEFAULT 'Unknown Artist',
                album TEXT NOT NULL DEFAULT 'Unknown Album',
                album_artist TEXT NOT NULL DEFAULT '',
                genre TEXT NOT NULL DEFAULT '',
                year INTEGER,
                track_number INTEGER,
                disc_number INTEGER,
                duration_ms INTEGER NOT NULL DEFAULT 0,
                file_hash TEXT NOT NULL DEFAULT '',
                fingerprint TEXT,
                acoustid TEXT,
                musicbrainz_recording_id TEXT,
                musicbrainz_release_id TEXT,
                art_path TEXT,
                file_size INTEGER NOT NULL DEFAULT 0,
                added_at INTEGER NOT NULL DEFAULT 0,
                favorite INTEGER NOT NULL DEFAULT 0,
                play_count INTEGER NOT NULL DEFAULT 0,
                last_played_at INTEGER
            );
            CREATE INDEX IF NOT EXISTS idx_tracks_hash ON tracks(file_hash);
            CREATE INDEX IF NOT EXISTS idx_tracks_fingerprint ON tracks(fingerprint);
            CREATE INDEX IF NOT EXISTS idx_tracks_artist ON tracks(artist);
            CREATE INDEX IF NOT EXISTS idx_tracks_album ON tracks(album);
            CREATE INDEX IF NOT EXISTS idx_tracks_favorite ON tracks(favorite);

            CREATE TABLE IF NOT EXISTS playlists (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS playlist_tracks (
                playlist_id INTEGER NOT NULL,
                track_id INTEGER NOT NULL,
                position INTEGER NOT NULL,
                PRIMARY KEY (playlist_id, track_id),
                FOREIGN KEY (playlist_id) REFERENCES playlists(id) ON DELETE CASCADE,
                FOREIGN KEY (track_id) REFERENCES tracks(id) ON DELETE CASCADE
            );
            ",
        )?;
        Ok(())
    }

    pub fn all_tracks(&self) -> rusqlite::Result<Vec<Track>> {
        self.query_tracks(
            "SELECT id, path, title, artist, album, album_artist, genre, year, track_number,
                    duration_ms, file_hash, fingerprint, art_path, added_at, favorite, play_count, last_played_at
             FROM tracks ORDER BY artist COLLATE NOCASE, album COLLATE NOCASE, track_number, title",
            params![],
        )
    }

    pub fn search_tracks(&self, query: &str) -> rusqlite::Result<Vec<Track>> {
        let like = format!("%{query}%");
        self.query_tracks(
            "SELECT id, path, title, artist, album, album_artist, genre, year, track_number,
                    duration_ms, file_hash, fingerprint, art_path, added_at, favorite, play_count, last_played_at
             FROM tracks
             WHERE title LIKE ?1 OR artist LIKE ?1 OR album LIKE ?1 OR genre LIKE ?1
             ORDER BY artist COLLATE NOCASE, album COLLATE NOCASE, track_number",
            params![like],
        )
    }

    pub fn favorites(&self) -> rusqlite::Result<Vec<Track>> {
        self.query_tracks(
            "SELECT id, path, title, artist, album, album_artist, genre, year, track_number,
                    duration_ms, file_hash, fingerprint, art_path, added_at, favorite, play_count, last_played_at
             FROM tracks WHERE favorite = 1
             ORDER BY artist COLLATE NOCASE, album COLLATE NOCASE, track_number",
            params![],
        )
    }

    pub fn recently_played(&self, limit: i64) -> rusqlite::Result<Vec<Track>> {
        self.query_tracks(
            "SELECT id, path, title, artist, album, album_artist, genre, year, track_number,
                    duration_ms, file_hash, fingerprint, art_path, added_at, favorite, play_count, last_played_at
             FROM tracks WHERE last_played_at IS NOT NULL
             ORDER BY last_played_at DESC LIMIT ?1",
            params![limit],
        )
    }

    pub fn most_played(&self, limit: i64) -> rusqlite::Result<Vec<Track>> {
        self.query_tracks(
            "SELECT id, path, title, artist, album, album_artist, genre, year, track_number,
                    duration_ms, file_hash, fingerprint, art_path, added_at, favorite, play_count, last_played_at
             FROM tracks WHERE play_count > 0
             ORDER BY play_count DESC, last_played_at DESC LIMIT ?1",
            params![limit],
        )
    }

    pub fn tracks_by_album(&self, album: &str, artist: &str) -> rusqlite::Result<Vec<Track>> {
        self.query_tracks(
            "SELECT id, path, title, artist, album, album_artist, genre, year, track_number,
                    duration_ms, file_hash, fingerprint, art_path, added_at, favorite, play_count, last_played_at
             FROM tracks WHERE album = ?1 AND artist = ?2
             ORDER BY disc_number, track_number, title",
            params![album, artist],
        )
    }

    pub fn tracks_by_artist(&self, artist: &str) -> rusqlite::Result<Vec<Track>> {
        self.query_tracks(
            "SELECT id, path, title, artist, album, album_artist, genre, year, track_number,
                    duration_ms, file_hash, fingerprint, art_path, added_at, favorite, play_count, last_played_at
             FROM tracks WHERE artist = ?1
             ORDER BY album COLLATE NOCASE, track_number, title",
            params![artist],
        )
    }

    pub fn playlist_tracks(&self, playlist_id: i64) -> rusqlite::Result<Vec<Track>> {
        self.query_tracks(
            "SELECT t.id, t.path, t.title, t.artist, t.album, t.album_artist, t.genre, t.year,
                    t.track_number, t.duration_ms, t.file_hash, t.fingerprint, t.art_path, t.added_at,
                    t.favorite, t.play_count, t.last_played_at
             FROM tracks t
             JOIN playlist_tracks pt ON pt.track_id = t.id
             WHERE pt.playlist_id = ?1 ORDER BY pt.position",
            params![playlist_id],
        )
    }

    fn query_tracks(
        &self,
        sql: &str,
        params: impl rusqlite::Params,
    ) -> rusqlite::Result<Vec<Track>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(sql)?;
        let rows = stmt.query_map(params, row_to_track)?;
        rows.collect()
    }

    pub fn hash_exists(&self, hash: &str) -> rusqlite::Result<bool> {
        let conn = self.conn.lock().unwrap();
        Ok(conn
            .query_row(
                "SELECT 1 FROM tracks WHERE file_hash = ?1 LIMIT 1",
                params![hash],
                |_| Ok(true),
            )
            .optional()?
            .unwrap_or(false))
    }

    pub fn fingerprint_exists(&self, fingerprint: &str) -> rusqlite::Result<bool> {
        let conn = self.conn.lock().unwrap();
        Ok(conn
            .query_row(
                "SELECT 1 FROM tracks WHERE fingerprint = ?1 LIMIT 1",
                params![fingerprint],
                |_| Ok(true),
            )
            .optional()?
            .unwrap_or(false))
    }

    pub fn path_exists(&self, path: &str) -> rusqlite::Result<bool> {
        let conn = self.conn.lock().unwrap();
        Ok(conn
            .query_row(
                "SELECT 1 FROM tracks WHERE path = ?1 LIMIT 1",
                params![path],
                |_| Ok(true),
            )
            .optional()?
            .unwrap_or(false))
    }

    pub fn insert_track(&self, track: &InsertTrack) -> rusqlite::Result<i64> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO tracks (path, title, artist, album, album_artist, genre, year,
                track_number, disc_number, duration_ms, file_hash, fingerprint, acoustid,
                musicbrainz_recording_id, musicbrainz_release_id, art_path, file_size, added_at)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18)",
            params![
                track.path, track.title, track.artist, track.album, track.album_artist,
                track.genre, track.year, track.track_number, track.disc_number, track.duration_ms,
                track.file_hash, track.fingerprint, track.acoustid, track.musicbrainz_recording_id,
                track.musicbrainz_release_id, track.art_path, track.file_size, track.added_at,
            ],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn set_favorite(&self, id: i64, favorite: bool) -> rusqlite::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE tracks SET favorite = ?2 WHERE id = ?1",
            params![id, if favorite { 1 } else { 0 }],
        )?;
        Ok(())
    }

    pub fn record_play(&self, id: i64, now: i64) -> rusqlite::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE tracks SET play_count = play_count + 1, last_played_at = ?2 WHERE id = ?1",
            params![id, now],
        )?;
        Ok(())
    }

    pub fn artists(&self) -> rusqlite::Result<Vec<(String, i64, i64)>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT artist, COUNT(*), COUNT(DISTINCT album)
             FROM tracks GROUP BY artist ORDER BY artist COLLATE NOCASE",
        )?;
        let rows: Vec<(String, i64, i64)> = stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))?
            .collect::<Result<_, _>>()?;
        Ok(rows)
    }

    pub fn albums(&self) -> rusqlite::Result<Vec<(String, String, i64, Option<String>, Option<i32>)>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT album, artist, COUNT(*), MIN(art_path), MIN(year)
             FROM tracks GROUP BY album, artist
             ORDER BY artist COLLATE NOCASE, album COLLATE NOCASE",
        )?;
        let rows: Vec<(String, String, i64, Option<String>, Option<i32>)> = stmt
            .query_map([], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?))
            })?
            .collect::<Result<_, _>>()?;
        Ok(rows)
    }

    pub fn playlists(&self) -> rusqlite::Result<Vec<(i64, String, i64, i64, i64)>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT p.id, p.name,
                    (SELECT COUNT(*) FROM playlist_tracks pt WHERE pt.playlist_id = p.id),
                    p.created_at, p.updated_at
             FROM playlists p ORDER BY p.updated_at DESC",
        )?;
        let rows: Vec<(i64, String, i64, i64, i64)> = stmt
            .query_map([], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?))
            })?
            .collect::<Result<_, _>>()?;
        Ok(rows)
    }

    pub fn create_playlist(&self, name: &str, now: i64) -> rusqlite::Result<i64> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO playlists (name, created_at, updated_at) VALUES (?1, ?2, ?3)",
            params![name, now, now],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn rename_playlist(&self, id: i64, name: &str, now: i64) -> rusqlite::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE playlists SET name = ?2, updated_at = ?3 WHERE id = ?1",
            params![id, name, now],
        )?;
        Ok(())
    }

    pub fn delete_playlist(&self, id: i64) -> rusqlite::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM playlist_tracks WHERE playlist_id = ?1", params![id])?;
        conn.execute("DELETE FROM playlists WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn add_to_playlist(&self, playlist_id: i64, track_id: i64, now: i64) -> rusqlite::Result<()> {
        let conn = self.conn.lock().unwrap();
        let pos: i64 = conn.query_row(
            "SELECT COALESCE(MAX(position), -1) + 1 FROM playlist_tracks WHERE playlist_id = ?1",
            params![playlist_id],
            |row| row.get(0),
        )?;
        conn.execute(
            "INSERT OR IGNORE INTO playlist_tracks (playlist_id, track_id, position) VALUES (?1, ?2, ?3)",
            params![playlist_id, track_id, pos],
        )?;
        conn.execute(
            "UPDATE playlists SET updated_at = ?2 WHERE id = ?1",
            params![playlist_id, now],
        )?;
        Ok(())
    }

    pub fn remove_from_playlist(&self, playlist_id: i64, track_id: i64, now: i64) -> rusqlite::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "DELETE FROM playlist_tracks WHERE playlist_id = ?1 AND track_id = ?2",
            params![playlist_id, track_id],
        )?;
        conn.execute(
            "UPDATE playlists SET updated_at = ?2 WHERE id = ?1",
            params![playlist_id, now],
        )?;
        Ok(())
    }

    pub fn stats(&self) -> rusqlite::Result<(i64, i64, i64, i64)> {
        let conn = self.conn.lock().unwrap();
        let tracks: i64 = conn.query_row("SELECT COUNT(*) FROM tracks", [], |r| r.get(0))?;
        let albums: i64 = conn.query_row(
            "SELECT COUNT(DISTINCT album || '|' || artist) FROM tracks",
            [],
            |r| r.get(0),
        )?;
        let artists: i64 = conn.query_row("SELECT COUNT(DISTINCT artist) FROM tracks", [], |r| r.get(0))?;
        let favorites: i64 = conn.query_row("SELECT COUNT(*) FROM tracks WHERE favorite = 1", [], |r| r.get(0))?;
        Ok((tracks, albums, artists, favorites))
    }
}

pub struct InsertTrack {
    pub path: String,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub album_artist: String,
    pub genre: String,
    pub year: Option<i32>,
    pub track_number: Option<i32>,
    pub disc_number: Option<i32>,
    pub duration_ms: i64,
    pub file_hash: String,
    pub fingerprint: Option<String>,
    pub acoustid: Option<String>,
    pub musicbrainz_recording_id: Option<String>,
    pub musicbrainz_release_id: Option<String>,
    pub art_path: Option<String>,
    pub file_size: i64,
    pub added_at: i64,
}

fn row_to_track(row: &rusqlite::Row) -> rusqlite::Result<Track> {
    Ok(Track {
        id: row.get(0)?,
        path: row.get(1)?,
        title: row.get(2)?,
        artist: row.get(3)?,
        album: row.get(4)?,
        album_artist: row.get(5)?,
        genre: row.get(6)?,
        year: row.get(7)?,
        track_number: row.get(8)?,
        duration_ms: row.get(9)?,
        file_hash: row.get(10)?,
        fingerprint: row.get(11)?,
        art_path: row.get(12)?,
        added_at: row.get(13)?,
        favorite: row.get::<_, i64>(14)? != 0,
        play_count: row.get(15)?,
        last_played_at: row.get(16)?,
    })
}
