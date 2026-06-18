use crate::models::{AcquireCommand, AcquireTool};
use std::path::PathBuf;
use std::process::Command;

pub fn default_download_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("Music")
        .join("BAASIC")
}

pub fn detect_tools() -> Vec<AcquireTool> {
    vec![
        probe_tool("yt-dlp", "yt-dlp", &["--version"]),
        probe_tool("spotdl", "spotdl", &["--version"]),
    ]
}

fn probe_tool(id: &str, bin: &str, version_args: &[&str]) -> AcquireTool {
    match Command::new(bin).args(version_args).output() {
        Ok(output) if output.status.success() => {
            let version = String::from_utf8_lossy(&output.stdout)
                .lines()
                .next()
                .unwrap_or("installed")
                .trim()
                .to_string();
            let path = which_path(bin).unwrap_or_else(|| bin.into());
            AcquireTool {
                id: id.into(),
                name: bin.into(),
                path: path.to_string_lossy().to_string(),
                version,
                installed: true,
            }
        }
        _ => AcquireTool {
            id: id.into(),
            name: bin.into(),
            path: String::new(),
            version: String::new(),
            installed: false,
        },
    }
}

fn which_path(bin: &str) -> Option<PathBuf> {
    std::env::var_os("PATH").and_then(|paths| {
        std::env::split_paths(&paths).find_map(|dir| {
            let candidate = dir.join(bin);
            if candidate.is_file() {
                Some(candidate)
            } else {
                None
            }
        })
    })
}

pub fn build_commands(
    query: &str,
    mode: &str,
    output_dir: Option<&str>,
) -> Vec<AcquireCommand> {
    let query = query.trim();
    if query.is_empty() {
        return Vec::new();
    }

    let out = output_dir
        .map(PathBuf::from)
        .unwrap_or_else(default_download_dir);
    let out_str = out.to_string_lossy().to_string();
    let out_template = format!("{}/%(artist|Unknown)s - %(title)s.%(ext)s", out_str);
    let escaped_query = shell_escape(query);

    let mut commands = Vec::new();
    let tools = detect_tools();

    if let Some(yt) = tools.iter().find(|t| t.id == "yt-dlp" && t.installed) {
        let search = match mode {
            "album" => format!("ytsearch10:{query} full album"),
            "artist" => format!("ytsearch15:{query}"),
            _ => format!("ytsearch1:{query}"),
        };

        commands.push(AcquireCommand {
            tool_id: yt.id.clone(),
            label: format!("Download with yt-dlp ({mode})"),
            command: format!(
                "mkdir -p {out_str:?} && {bin} {search:?} -x --audio-format mp3 --audio-quality 0 --embed-thumbnail --add-metadata -o {out_template:?}",
                bin = yt.path,
                search = search,
            ),
            output_dir: out_str.clone(),
            import_hint: format!(
                "After download finishes, in BAASIC go to Import → Add Folder → select {out_str:?}"
            ),
        });

        commands.push(AcquireCommand {
            tool_id: yt.id.clone(),
            label: "Download from URL with yt-dlp".to_string(),
            command: format!(
                "mkdir -p {out_str:?} && {bin} {url:?} -x --audio-format mp3 --audio-quality 0 --embed-thumbnail --add-metadata -o {out_template:?}",
                bin = yt.path,
                url = query,
            ),
            output_dir: out_str.clone(),
            import_hint: format!("Import folder: {out_str:?}"),
        });
    }

    if let Some(spot) = tools.iter().find(|t| t.id == "spotdl" && t.installed) {
        commands.push(AcquireCommand {
            tool_id: spot.id.clone(),
            label: format!("Download with spotdl ({mode})"),
            command: format!(
                "mkdir -p {out_str:?} && cd {out_str:?} && {bin} download {escaped_query}",
                bin = spot.path,
            ),
            output_dir: out_str.clone(),
            import_hint: format!("Import folder: {out_str:?}"),
        });
    }

    if commands.is_empty() {
        commands.push(AcquireCommand {
            tool_id: "manual".into(),
            label: "Install yt-dlp first".to_string(),
            command: "sudo pacman -S yt-dlp ffmpeg".to_string(),
            output_dir: out_str.clone(),
            import_hint: "Install yt-dlp, run a download command, then import the BAASIC music folder.".into(),
        });
    }

    commands
}

fn shell_escape(s: &str) -> String {
    if s.chars().any(|c| c.is_whitespace() || c == '"' || c == '\'' || c == '\\') {
        format!("'{s}'")
    } else {
        s.to_string()
    }
}
