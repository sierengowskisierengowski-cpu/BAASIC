use serde::Deserialize;
use std::path::Path;
use std::process::Command;

#[derive(Debug, Deserialize)]
struct FpcalcJson {
    duration: f64,
    fingerprint: String,
}

pub struct AudioFingerprint {
    pub duration_secs: f64,
    pub fingerprint: String,
}

pub fn compute_fingerprint(path: &Path) -> Result<AudioFingerprint, String> {
    let output = Command::new("fpcalc")
        .args(["-json", "-length", "120"])
        .arg(path)
        .output()
        .map_err(|e| format!("fpcalc not found or failed: {e}"))?;

    if !output.status.success() {
        return Err(format!(
            "fpcalc failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let parsed: FpcalcJson = serde_json::from_slice(&output.stdout)
        .map_err(|e| format!("failed to parse fpcalc output: {e}"))?;

    Ok(AudioFingerprint {
        duration_secs: parsed.duration,
        fingerprint: parsed.fingerprint,
    })
}
