use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File},
    io::Write,
    path::Path,
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

pub type Result<T> = std::result::Result<T, String>;
static SEQUENCE: AtomicU64 = AtomicU64::new(0);
pub fn stamp() -> String {
    format!(
        "{:020}_{:06}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis(),
        SEQUENCE.fetch_add(1, Ordering::Relaxed)
    )
}
pub fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
pub fn read(path: &Path) -> Result<Vec<u8>> {
    fs::read(path).map_err(|e| format!("Read {}: {e}", path.display()))
}
pub fn write_sync(path: &Path, bytes: &[u8]) -> Result<()> {
    let mut f = File::create(path).map_err(|e| format!("Create {}: {e}", path.display()))?;
    f.write_all(bytes)
        .and_then(|_| f.sync_all())
        .map_err(|e| format!("Write {}: {e}", path.display()))
}
pub fn replace(from: &Path, to: &Path) -> Result<()> {
    fs::rename(from, to).map_err(|e| format!("Replace {}: {e}", to.display()))
}
pub fn atomic_write(path: &Path, bytes: &[u8]) -> Result<()> {
    let temp = path.with_extension(format!("{}.tmp", stamp()));
    let result = (|| {
        write_sync(&temp, bytes)?;
        replace(&temp, path)
    })();
    if result.is_err() {
        let _ = fs::remove_file(temp);
    }
    result
}
pub fn prune(dir: &Path, prefix: &str, suffix: &str, keep: usize) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    let mut files: Vec<_> = entries
        .flatten()
        .filter(|e| {
            e.file_type().is_ok_and(|t| t.is_file())
                && e.file_name().to_string_lossy().starts_with(prefix)
                && e.file_name().to_string_lossy().ends_with(suffix)
        })
        .collect();
    files.sort_by_key(|e| {
        std::cmp::Reverse((e.metadata().and_then(|m| m.modified()).ok(), e.file_name()))
    });
    for entry in files.into_iter().skip(keep.max(1)) {
        let _ = fs::remove_file(entry.path());
    }
}
pub fn backup(data_root: &Path, snapshot: &Value) -> Result<String> {
    if !snapshot.get("data").is_some_and(|v| {
        v.as_object()
            .is_some_and(|m| m.values().all(Value::is_string))
    }) {
        return Err(
            "Save snapshot must contain a data object of localStorage string values".into(),
        );
    }
    let dir = data_root.join("backups");
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let name = format!("saves_{}.json", stamp());
    write_sync(
        &dir.join(&name),
        &serde_json::to_vec(snapshot).map_err(|e| e.to_string())?,
    )?;
    prune(&dir, "saves_", ".json", 25);
    Ok(name)
}
pub fn backups(data_root: &Path) -> Value {
    let mut entries = Vec::new();
    if let Ok(files) = fs::read_dir(data_root.join("backups")) {
        for f in files.flatten() {
            let name = f.file_name().to_string_lossy().into_owned();
            if name.starts_with("saves_")
                && name.ends_with(".json")
                && let Ok(m) = f.metadata()
                && m.is_file()
            {
                entries.push(json!({"name":name,"size":m.len(),"modified":m.modified().ok().and_then(|t| t.duration_since(UNIX_EPOCH).ok()).map(|d|d.as_secs().to_string()).unwrap_or_default()}));
            }
        }
    }
    entries.sort_by(|a, b| b["name"].as_str().cmp(&a["name"].as_str()));
    json!({"backups":entries})
}
pub fn backup_path(data_root: &Path, name: &str) -> Result<std::path::PathBuf> {
    if !name.starts_with("saves_")
        || !name.ends_with(".json")
        || name.contains(['/', '\\', ':'])
        || name.contains("..")
    {
        return Err("Invalid backup name".into());
    }
    Ok(data_root.join("backups").join(name))
}
