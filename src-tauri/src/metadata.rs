use lofty::file::{AudioFile, TaggedFileExt};
use lofty::probe::Probe;
use lofty::tag::Accessor;
use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct FileMetadata {
    pub title: String,
    pub artist: String,
    pub album: String,
    pub album_artist: String,
    pub genre: String,
    pub year: Option<i32>,
    pub track_number: Option<i32>,
    pub disc_number: Option<i32>,
    pub duration_ms: i64,
    pub needs_identification: bool,
}

pub fn read_metadata(path: &Path) -> Result<FileMetadata, String> {
    let tagged_file = Probe::open(path)
        .map_err(|e| format!("probe failed: {e}"))?
        .read()
        .map_err(|e| format!("read failed: {e}"))?;

    let duration_ms = tagged_file
        .properties()
        .duration()
        .as_millis() as i64;

    let tag = tagged_file.primary_tag().or_else(|| tagged_file.first_tag());

    let (title, artist, album, album_artist, genre, year, track_number, disc_number) =
        if let Some(tag) = tag {
            (
                tag.title().map(|s| s.to_string()).unwrap_or_default(),
                tag.artist().map(|s| s.to_string()).unwrap_or_default(),
                tag.album().map(|s| s.to_string()).unwrap_or_default(),
                tag.get_string(&lofty::tag::ItemKey::AlbumArtist)
                    .map(|s| s.to_string())
                    .unwrap_or_default(),
                tag.genre().map(|s| s.to_string()).unwrap_or_default(),
                tag.year().map(|y| y as i32),
                tag.track().map(|t| t as i32),
                tag.disk().map(|d| d as i32),
            )
        } else {
            (
                String::new(),
                String::new(),
                String::new(),
                String::new(),
                String::new(),
                None,
                None,
                None,
            )
        };

    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Unknown Title")
        .to_string();

    let title = if title.trim().is_empty() {
        stem
    } else {
        title
    };

    let artist = if artist.trim().is_empty() {
        "Unknown Artist".to_string()
    } else {
        artist
    };

    let album = if album.trim().is_empty() {
        "Unknown Album".to_string()
    } else {
        album
    };

    let needs_identification =
        artist == "Unknown Artist" || album == "Unknown Album" || tag.is_none();

    Ok(FileMetadata {
        title,
        artist,
        album,
        album_artist,
        genre,
        year,
        track_number,
        disc_number,
        duration_ms,
        needs_identification,
    })
}

#[derive(Debug, Clone)]
pub struct IdentifiedMetadata {
    pub title: String,
    pub artist: String,
    pub album: String,
    pub year: Option<i32>,
    pub acoustid: Option<String>,
    pub recording_id: Option<String>,
    pub release_id: Option<String>,
    pub art_path: Option<String>,
}

#[derive(Deserialize)]
struct AcoustidResponse {
    results: Option<Vec<AcoustidResult>>,
}

#[derive(Deserialize)]
struct AcoustidResult {
    id: Option<String>,
    score: Option<f64>,
    recordings: Option<Vec<AcoustidRecording>>,
}

#[derive(Deserialize)]
struct AcoustidRecording {
    id: Option<String>,
    title: Option<String>,
    artists: Option<Vec<AcoustidArtist>>,
    releases: Option<Vec<AcoustidRelease>>,
}

#[derive(Deserialize)]
struct AcoustidArtist {
    name: Option<String>,
}

#[derive(Deserialize)]
struct AcoustidRelease {
    id: Option<String>,
    title: Option<String>,
    date: Option<AcoustidDate>,
}

#[derive(Deserialize)]
struct AcoustidDate {
    year: Option<i32>,
}

#[derive(Deserialize)]
struct CoverArtArchive {
    images: Option<Vec<CoverImage>>,
}

#[derive(Deserialize)]
struct CoverImage {
    front: Option<bool>,
    image: Option<String>,
    thumbnails: Option<CoverThumbs>,
}

#[derive(Deserialize)]
struct CoverThumbs {
    large: Option<String>,
}

pub fn identify_track(
    fingerprint: &str,
    duration_secs: f64,
    api_key: &str,
    art_dir: &Path,
    track_id_hint: &str,
) -> Result<IdentifiedMetadata, String> {
    let client = reqwest::blocking::Client::builder()
        .user_agent("BAASIC-Media-Player/1.0 (https://github.com/local)")
        .build()
        .map_err(|e| e.to_string())?;

    let body = format!(
        "client={}&meta=recordings+releasegroups+compress&duration={}&fingerprint={}",
        urlencoding::encode(api_key),
        duration_secs.round() as i64,
        urlencoding::encode(fingerprint),
    );

    let response: AcoustidResponse = client
        .post("https://api.acoustid.org/v2/lookup")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .map_err(|e| format!("AcoustID request failed: {e}"))?
        .json()
        .map_err(|e| format!("AcoustID parse failed: {e}"))?;

    let best = response
        .results
        .unwrap_or_default()
        .into_iter()
        .filter(|r| r.score.unwrap_or(0.0) > 0.5)
        .max_by(|a, b| {
            a.score
                .unwrap_or(0.0)
                .partial_cmp(&b.score.unwrap_or(0.0))
                .unwrap_or(std::cmp::Ordering::Equal)
        });

    let Some(result) = best else {
        return Err("no AcoustID match found".into());
    };

    let recording = result
        .recordings
        .as_ref()
        .and_then(|r| r.first())
        .ok_or("no recording in AcoustID result")?;

    let title = recording
        .title
        .clone()
        .unwrap_or_else(|| "Unknown Title".into());

    let artist = recording
        .artists
        .as_ref()
        .and_then(|a| a.first())
        .and_then(|a| a.name.clone())
        .unwrap_or_else(|| "Unknown Artist".into());

    let release = recording.releases.as_ref().and_then(|r| r.first());

    let album = release
        .and_then(|r| r.title.clone())
        .unwrap_or_else(|| "Unknown Album".into());

    let year = release.and_then(|r| r.date.as_ref().and_then(|d| d.year));
    let release_id = release.and_then(|r| r.id.clone());
    let recording_id = recording.id.clone();

    let art_path = release_id
        .as_ref()
        .and_then(|id| download_cover_art(&client, id, art_dir, track_id_hint).ok());

    Ok(IdentifiedMetadata {
        title,
        artist,
        album,
        year,
        acoustid: result.id.clone(),
        recording_id,
        release_id,
        art_path,
    })
}

fn download_cover_art(
    client: &reqwest::blocking::Client,
    release_id: &str,
    art_dir: &Path,
    track_id_hint: &str,
) -> Result<String, String> {
    std::fs::create_dir_all(art_dir).map_err(|e| e.to_string())?;

    let url = format!("https://coverartarchive.org/release/{release_id}");
    let archive: CoverArtArchive = client
        .get(&url)
        .send()
        .map_err(|e| e.to_string())?
        .json()
        .map_err(|e| e.to_string())?;

    let image_url = archive
        .images
        .unwrap_or_default()
        .into_iter()
        .find(|img| img.front.unwrap_or(false))
        .and_then(|img| img.thumbnails.and_then(|t| t.large).or(img.image))
        .ok_or("no cover art found")?;

    let bytes = client
        .get(&image_url)
        .send()
        .map_err(|e| e.to_string())?
        .bytes()
        .map_err(|e| e.to_string())?;

    let path = art_dir.join(format!("{track_id_hint}.jpg"));
    std::fs::write(&path, &bytes).map_err(|e| e.to_string())?;
    Ok(path.to_string_lossy().to_string())
}

pub fn write_tags(path: &Path, meta: &IdentifiedMetadata) -> Result<(), String> {
    use lofty::tag::{Tag, TagType};
    use lofty::probe::Probe;
    use lofty::config::WriteOptions;

    let mut tagged_file = Probe::open(path)
        .map_err(|e| e.to_string())?
        .read()
        .map_err(|e| e.to_string())?;

    let mut tag = tagged_file
        .primary_tag()
        .cloned()
        .unwrap_or_else(|| Tag::new(TagType::Id3v2));

    tag.set_title(meta.title.clone());
    tag.set_artist(meta.artist.clone());
    tag.set_album(meta.album.clone());
    if let Some(year) = meta.year {
        tag.set_year(year as u32);
    }

    tagged_file.insert_tag(tag);
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .open(path)
        .map_err(|e| e.to_string())?;
    tagged_file
        .save_to(&mut file, WriteOptions::default())
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn data_dir() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("baasic-media-player")
}

pub fn art_dir() -> PathBuf {
    data_dir().join("art")
}

pub fn config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("baasic-media-player")
        .join("config.json")
}
