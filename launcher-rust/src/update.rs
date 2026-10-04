use crate::storage::{self, Result};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{fs, path::Path, time::Duration};

pub const VERSION: &str = "3.0.0";
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Package {
    pub version: String,
    pub save_version: u32,
    pub app_html: String,
    #[serde(default)]
    pub notes: Vec<String>,
    #[serde(default)]
    pub launcher_exe_base64: String,
    #[serde(default)]
    pub launcher_sha256: String,
}
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Manifest {
    pub game_version: String,
    pub save_version: u32,
    pub minimum_launcher_version: String,
    pub update_url: String,
    pub sha256: String,
    pub size: u64,
    pub mandatory: bool,
    pub release_notes: Vec<String>,
    pub launcher_version: String,
    pub launcher_url: String,
    pub launcher_sha256: String,
}
pub fn greater(a: &str, b: &str) -> bool {
    fn parts(s: &str) -> [u64; 4] {
        let mut out = [0; 4];
        for (i, p) in s
            .trim()
            .trim_start_matches('v')
            .split('-')
            .next()
            .unwrap_or("")
            .split('.')
            .take(4)
            .enumerate()
        {
            out[i] = p.parse().unwrap_or(0);
        }
        out
    }
    parts(a) > parts(b)
}
pub fn validate_package(p: &Package) -> Result<()> {
    if p.version.trim().is_empty() || p.save_version == 0 {
        return Err("Package version and saveVersion are required".into());
    }
    let h = p.app_html.trim().to_lowercase();
    if !h.starts_with("<!doctype html")
        || !h.contains("<html")
        || !h.contains("</html>")
        || !h.contains("<script")
        || !h.contains("</script>")
        || h.contains('\0')
    {
        return Err("appHtml must be a complete HTML document with game script".into());
    }
    let version_marker = format!("VERSION='{}'", p.version);
    let html_save_version = p
        .app_html
        .split_once("SAVE_VERSION=")
        .and_then(|(_, rest)| {
            rest.chars()
                .take_while(char::is_ascii_digit)
                .collect::<String>()
                .parse::<u32>()
                .ok()
        });
    if !p.app_html.contains(&version_marker) || html_save_version != Some(p.save_version) {
        return Err("Game HTML version/saveVersion differs from package metadata".into());
    }
    Ok(())
}
pub fn verify(bytes: &[u8], size: u64, expected: &str) -> Result<()> {
    if size > 0 && bytes.len() as u64 != size {
        return Err(format!(
            "Update size mismatch: got {}, expected {size}",
            bytes.len()
        ));
    }
    if expected.len() != 64
        || !expected.bytes().all(|b| b.is_ascii_hexdigit())
        || !storage::hash(bytes).eq_ignore_ascii_case(expected)
    {
        return Err("SHA-256 verification failed (a valid hash is required)".into());
    }
    Ok(())
}
pub fn decode(bytes: &[u8], m: &Manifest) -> Result<Package> {
    verify(bytes, m.size, &m.sha256)?;
    let p: Package =
        serde_json::from_slice(bytes).map_err(|e| format!("Invalid update JSON: {e}"))?;
    validate_package(&p)?;
    if p.version != m.game_version || p.save_version != m.save_version {
        return Err("Package version/saveVersion does not match manifest".into());
    }
    Ok(p)
}
pub async fn fetch(raw: &str, limit: usize) -> Result<Vec<u8>> {
    let mut url = reqwest::Url::parse(raw).map_err(|e| e.to_string())?;
    if url.scheme() != "https" || url.host_str().is_none() {
        return Err("Remote URL must use HTTPS".into());
    }
    url.query_pairs_mut()
        .append_pair("lc_nonce", &storage::stamp());
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(20))
        .connect_timeout(Duration::from_secs(8))
        .redirect(reqwest::redirect::Policy::custom(|a| {
            if a.previous().len() > 6 || a.url().scheme() != "https" {
                a.error("Redirect must remain HTTPS, with at most six redirects")
            } else {
                a.follow()
            }
        }))
        .build()
        .map_err(|e| e.to_string())?;
    let mut response = client
        .get(url)
        .header("Cache-Control", "no-cache, no-store, max-age=0")
        .header("Pragma", "no-cache")
        .header("User-Agent", format!("LifeClickerLauncher/{VERSION}"))
        .send()
        .await
        .map_err(|e| format!("Download: {e}"))?
        .error_for_status()
        .map_err(|e| format!("Download: {e}"))?;
    if response.content_length().is_some_and(|n| n > limit as u64) {
        return Err("Remote file too large".into());
    }
    let mut out = Vec::new();
    while let Some(chunk) = response.chunk().await.map_err(|e| e.to_string())? {
        if out.len() + chunk.len() > limit {
            return Err("Remote file too large".into());
        }
        out.extend_from_slice(&chunk);
    }
    Ok(out)
}

// The journal is durable before either replacement. Recovery restores BOTH files.
// Two separate paths cannot be committed in one filesystem rename.
pub fn recover(root: &Path) -> Result<bool> {
    let tx = root.join("userdata/update-transaction");
    if !tx.join("pending").exists() {
        return Ok(false);
    }
    let html = storage::read(&tx.join("index.html"))?;
    let meta = storage::read(&tx.join("version.json"))?;
    let game_path = root.join("game/index.html");
    let meta_path = root.join("game/version.json");
    if storage::read(&game_path).ok().as_deref() != Some(html.as_slice()) {
        storage::atomic_write(&game_path, &html)?;
    }
    if storage::read(&meta_path).ok().as_deref() != Some(meta.as_slice()) {
        storage::atomic_write(&meta_path, &meta)?;
    }
    fs::remove_file(tx.join("pending"))
        .map_err(|e| format!("Rollback restored files but could not clear journal: {e}"))?;
    Ok(true)
}
pub fn install(root: &Path, p: &Package, snapshot: &Value) -> Result<Value> {
    install_inner(root, p, snapshot, None)
}
pub fn install_inner(
    root: &Path,
    p: &Package,
    snapshot: &Value,
    fail_at: Option<&str>,
) -> Result<Value> {
    validate_package(p)?;
    recover(root)?;
    let game = root.join("game/index.html");
    let meta = root.join("game/version.json");
    let old = storage::read(&game)?;
    let old_meta = storage::read(&meta)?;
    let backup_name = storage::backup(&root.join("userdata"), snapshot)?;
    let dir = root.join("userdata/game-backups");
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let id = storage::stamp();
    storage::write_sync(&dir.join(format!("index_{id}.html")), &old)?;
    let tx = root.join("userdata/update-transaction");
    fs::create_dir_all(&tx).map_err(|e| e.to_string())?;
    storage::write_sync(&tx.join("index.html"), &old)?;
    storage::write_sync(&tx.join("version.json"), &old_meta)?;
    let new_html = root.join("game/index.html.new");
    let new_meta = root.join("game/version.json.new");
    storage::write_sync(&new_html, p.app_html.as_bytes())?;
    if storage::read(&new_html)? != p.app_html.as_bytes() {
        return Err("Written game verification failed; current game intact".into());
    }
    let metadata = json!({"version":p.version,"saveVersion":p.save_version,"launcherVersion":VERSION,"product":"Life Clicker"});
    let metadata_bytes = serde_json::to_vec_pretty(&metadata).map_err(|e| e.to_string())?;
    storage::write_sync(&new_meta, &metadata_bytes)?;
    if storage::read(&new_meta)? != metadata_bytes {
        return Err("Written version metadata verification failed; current game intact".into());
    }
    storage::write_sync(&tx.join("pending"), b"rollback required")?;
    let commit: Result<()> = (|| {
        if fail_at == Some("before") {
            return Err("Injected failure before replacement".into());
        }
        storage::replace(&new_html, &game)?;
        if fail_at == Some("metadata") {
            return Err("Injected metadata replacement failure".into());
        }
        storage::replace(&new_meta, &meta)?;
        fs::remove_file(tx.join("pending")).map_err(|e| format!("Commit journal: {e}"))?;
        Ok(())
    })();
    if let Err(e) = commit {
        return match recover(root) {
            Ok(_) => Err(format!(
                "{e}; automatic rollback restored game and version metadata"
            )),
            Err(r) => Err(format!(
                "{e}; rollback failed: {r}. Recovery copies retained in {}",
                tx.display()
            )),
        };
    }
    storage::prune(&dir, "index_", ".html", 12);
    Ok(
        json!({"ok":true,"version":p.version,"gameInstalled":true,"launcherStaged":false,"backup":backup_name}),
    )
}
