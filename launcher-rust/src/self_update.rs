use crate::{
    storage::{self, Result},
    update,
};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::Duration,
};

pub fn validate(bytes: &[u8], expected: &str) -> Result<()> {
    update::verify(bytes, 0, expected)?;
    if bytes.len() < 1024 || !bytes.starts_with(b"MZ") {
        return Err("Launcher payload is not a Windows executable".into());
    }
    let offset = bytes
        .get(60..64)
        .map(|b| u32::from_le_bytes(b.try_into().unwrap()) as usize)
        .unwrap_or(usize::MAX);
    if bytes.get(offset..offset.saturating_add(4)) != Some(b"PE\0\0") {
        return Err("Launcher PE signature missing".into());
    }
    Ok(())
}
pub fn stage(root: &Path, bytes: &[u8], expected: &str) -> Result<()> {
    validate(bytes, expected)?;
    // Hash first: interruption can only leave a mismatching stage, never a trusted partial exe.
    storage::atomic_write(
        &root.join("LifeClicker.next.exe.sha256"),
        expected.as_bytes(),
    )?;
    storage::atomic_write(&root.join("LifeClicker.next.exe"), bytes)
}
pub fn staged(root: &Path) -> Result<Option<PathBuf>> {
    for name in ["LifeClicker.next.exe", "LifeClickerLauncher.next.exe"] {
        let path = root.join(name);
        if path.exists() {
            let hash_path = root.join(format!("{name}.sha256"));
            let hash = String::from_utf8(storage::read(&hash_path)?).map_err(|e| e.to_string())?;
            validate(&storage::read(&path)?, hash.trim())?;
            return Ok(Some(path));
        }
    }
    Ok(None)
}
pub fn replace_staged(root: &Path, current: &Path, fail: bool) -> Result<()> {
    let next = staged(root)?.ok_or("No valid staged launcher")?;
    let previous = root.join("LifeClicker.previous.exe");
    if current.exists() {
        storage::atomic_write(&previous, &storage::read(current)?)?;
    }
    if fail {
        return Err("Injected replacement failure; current executable preserved".into());
    }
    if let Err(e) = storage::replace(&next, current) {
        if !current.exists() && previous.exists() {
            storage::atomic_write(current, &storage::read(&previous)?)?;
        }
        return Err(e);
    }
    let _ = fs::remove_file(next.with_file_name(format!(
        "{}.sha256",
        next.file_name().unwrap().to_string_lossy()
    )));
    Ok(())
}
pub fn launch_helper(root: &Path, current: &Path, no_browser: bool) -> Result<bool> {
    if staged(root)?.is_none() {
        return Ok(false);
    }
    let helper = std::env::temp_dir().join(format!("LifeClicker-helper-{}.exe", storage::stamp()));
    fs::copy(current, &helper).map_err(|e| e.to_string())?;
    let mut command = hidden_command(&helper);
    command.arg("--apply-self-update").arg(root).arg(current);
    if no_browser {
        command.arg("--no-browser");
    }
    command.spawn().map_err(|e| e.to_string())?;
    Ok(true)
}
pub fn helper(root: &Path, current: &Path, no_browser: bool) -> Result<()> {
    let mut last = String::new();
    for _ in 0..100 {
        match replace_staged(root, current, false) {
            Ok(()) => {
                let self_path = std::env::current_exe().map_err(|e| e.to_string())?;
                let mut command = hidden_command(current);
                command
                    .arg("--cleanup-helper")
                    .arg(self_path)
                    .arg("--root")
                    .arg(root);
                if no_browser {
                    command.arg("--no-browser");
                }
                if let Err(e) = command.spawn() {
                    storage::atomic_write(
                        current,
                        &storage::read(&root.join("LifeClicker.previous.exe"))?,
                    )?;
                    return Err(format!("Restart failed: {e}; previous executable restored"));
                }
                return Ok(());
            }
            Err(e) => last = e,
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    Err(format!(
        "Launcher replacement failed; current/previous copies retained: {last}"
    ))
}
pub fn cleanup(path: &Path) {
    if path.parent() != Some(std::env::temp_dir().as_path())
        || !path
            .file_name()
            .is_some_and(|n| n.to_string_lossy().starts_with("LifeClicker-helper-"))
    {
        return;
    }
    for _ in 0..30 {
        if fs::remove_file(path).is_ok() {
            break;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}
pub fn hidden_command(program: impl AsRef<std::ffi::OsStr>) -> Command {
    let mut c = Command::new(program);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        c.creation_flags(0x08000000);
    }
    c
}
