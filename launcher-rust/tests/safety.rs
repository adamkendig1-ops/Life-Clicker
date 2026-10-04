use life_clicker_launcher::{
    self_update, server, storage,
    update::{self, Manifest, Package},
};
use serde_json::json;
use std::fs;
use tempfile::TempDir;

fn fixture() -> TempDir {
    let t = TempDir::new().unwrap();
    fs::create_dir(t.path().join("game")).unwrap();
    fs::write(t.path().join("game/index.html"), b"old exact game").unwrap();
    fs::write(
        t.path().join("game/version.json"),
        br#"{"version":"7.0.7","saveVersion":12}"#,
    )
    .unwrap();
    t
}
fn package() -> Package {
    Package {
        version: "7.0.8".into(),
        save_version: 12,
        app_html:
            "<!doctype html><html><script>const VERSION='7.0.8',SAVE_VERSION=12;</script></html>"
                .into(),
        notes: vec![],
        launcher_exe_base64: String::new(),
        launcher_sha256: String::new(),
    }
}
fn snapshot() -> serde_json::Value {
    json!({"data":{"LC4_PROFILES":"[]","LC4_SAVE_fixture":"{\"saveVersion\":12}"}})
}
fn pe() -> Vec<u8> {
    let mut b = vec![0; 2048];
    b[..2].copy_from_slice(b"MZ");
    b[60..64].copy_from_slice(&128u32.to_le_bytes());
    b[128..132].copy_from_slice(b"PE\0\0");
    b
}

#[test]
fn versions() {
    for (a, b, expected) in [
        ("7.0.7", "7.0.6", true),
        ("7.0.6", "7.0.7", false),
        ("7.0.7", "7.0.7", false),
        ("7.0.10", "7.0.9", true),
        ("7.1.0", "7.0.99", true),
        ("10.0.0", "9.99.99", true),
        ("v7.0.8", "7.0.7", true),
    ] {
        assert_eq!(update::greater(a, b), expected, "{a} > {b}");
    }
}
#[test]
fn validates_download() {
    let b = serde_json::to_vec(&package()).unwrap();
    let mut m = Manifest {
        game_version: "7.0.8".into(),
        save_version: 12,
        sha256: storage::hash(&b),
        size: b.len() as u64,
        ..Default::default()
    };
    assert!(update::decode(&b, &m).is_ok());
    m.size += 1;
    assert!(update::decode(&b, &m).unwrap_err().contains("size"));
    m.size -= 1;
    m.sha256 = "a".repeat(64);
    assert!(update::decode(&b, &m).unwrap_err().contains("SHA"));
    m.sha256 = storage::hash(&b);
    m.game_version = "7.0.9".into();
    assert!(update::decode(&b, &m).unwrap_err().contains("manifest"));
    m.sha256 = storage::hash(b"{");
    m.size = 1;
    assert!(update::decode(b"{", &m).unwrap_err().contains("JSON"));
}
#[test]
fn rejects_missing_fields_and_invalid_html() {
    assert!(serde_json::from_value::<Package>(json!({"version":"7.0.8"})).is_err());
    let mut p = package();
    p.app_html = "broken".into();
    assert!(update::validate_package(&p).is_err());
    p = package();
    p.save_version = 13;
    assert!(update::validate_package(&p).is_err());
}
#[test]
fn rollback_on_replacement_failures() {
    for stage in ["before", "metadata"] {
        let t = fixture();
        let old = fs::read(t.path().join("game/index.html")).unwrap();
        let meta = fs::read(t.path().join("game/version.json")).unwrap();
        let error =
            update::install_inner(t.path(), &package(), &snapshot(), Some(stage)).unwrap_err();
        assert!(error.contains("automatic rollback restored"));
        assert_eq!(fs::read(t.path().join("game/index.html")).unwrap(), old);
        assert_eq!(fs::read(t.path().join("game/version.json")).unwrap(), meta);
        assert!(
            !t.path()
                .join("userdata/update-transaction/pending")
                .exists()
        );
    }
}
#[test]
fn invalid_package_never_changes_installation() {
    let t = fixture();
    let mut p = package();
    p.app_html.clear();
    assert!(update::install(t.path(), &p, &snapshot()).is_err());
    assert_eq!(
        fs::read(t.path().join("game/index.html")).unwrap(),
        b"old exact game"
    );
    assert!(!t.path().join("userdata").exists());
}
#[test]
fn update_commit_and_retention() {
    let t = fixture();
    for _ in 0..28 {
        update::install(t.path(), &package(), &snapshot()).unwrap();
    }
    assert_eq!(
        fs::read_to_string(t.path().join("game/index.html")).unwrap(),
        package().app_html
    );
    assert_eq!(
        fs::read_dir(t.path().join("userdata/game-backups"))
            .unwrap()
            .count(),
        12
    );
    assert_eq!(
        fs::read_dir(t.path().join("userdata/backups"))
            .unwrap()
            .count(),
        25
    );
    let list = storage::backups(&t.path().join("userdata"));
    let name = list["backups"][0]["name"].as_str().unwrap();
    let bytes =
        storage::read(&storage::backup_path(&t.path().join("userdata"), name).unwrap()).unwrap();
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&bytes).unwrap(),
        snapshot()
    );
}
#[test]
fn interrupted_update_recovered() {
    let t = fixture();
    let tx = t.path().join("userdata/update-transaction");
    fs::create_dir_all(&tx).unwrap();
    fs::write(tx.join("index.html"), b"recovery html").unwrap();
    fs::write(tx.join("version.json"), b"recovery metadata").unwrap();
    fs::write(tx.join("pending"), b"1").unwrap();
    assert!(update::recover(t.path()).unwrap());
    assert_eq!(
        fs::read(t.path().join("game/index.html")).unwrap(),
        b"recovery html"
    );
    assert!(!update::recover(t.path()).unwrap());
}
#[test]
fn failed_recovery_retains_only_recoverable_copy() {
    let t = fixture();
    let tx = t.path().join("userdata/update-transaction");
    fs::create_dir_all(&tx).unwrap();
    fs::write(tx.join("index.html"), b"only recovery html").unwrap();
    fs::write(tx.join("pending"), b"1").unwrap();
    assert!(update::recover(t.path()).is_err());
    assert!(tx.join("pending").exists());
    assert_eq!(
        fs::read(tx.join("index.html")).unwrap(),
        b"only recovery html"
    );
    assert_eq!(
        fs::read(t.path().join("game/index.html")).unwrap(),
        b"old exact game"
    );
}
#[test]
fn self_update_stage_verify_replace_and_recover() {
    let t = fixture();
    let exe = t.path().join("LifeClicker.exe");
    fs::write(&exe, b"old working executable").unwrap();
    let b = pe();
    self_update::stage(t.path(), &b, &storage::hash(&b)).unwrap();
    assert!(self_update::replace_staged(t.path(), &exe, true).is_err());
    assert_eq!(fs::read(&exe).unwrap(), b"old working executable");
    self_update::replace_staged(t.path(), &exe, false).unwrap();
    assert_eq!(fs::read(&exe).unwrap(), b);
    assert_eq!(
        fs::read(t.path().join("LifeClicker.previous.exe")).unwrap(),
        b"old working executable"
    );
    assert!(!t.path().join("LifeClicker.next.exe").exists());
}
#[test]
fn staged_tampering_rejected() {
    let t = fixture();
    let b = pe();
    assert!(self_update::stage(t.path(), &b, &"0".repeat(64)).is_err());
    self_update::stage(t.path(), &b, &storage::hash(&b)).unwrap();
    fs::write(t.path().join("LifeClicker.next.exe"), b"tampered").unwrap();
    assert!(self_update::staged(t.path()).is_err());
}
#[test]
fn snapshots_and_paths_validated() {
    let t = fixture();
    assert!(storage::backup(t.path(), &json!({"data":{"LC4_PROFILES":[]}})).is_err());
    for name in [
        "../saves_x.json",
        "saves_..\\escape.json",
        "saves_x.json:stream",
    ] {
        assert!(storage::backup_path(t.path(), name).is_err());
    }
}

#[cfg(windows)]
#[test]
fn windows_locked_metadata_rolls_back_html() {
    use std::os::windows::fs::OpenOptionsExt;
    let t = fixture();
    let lock = fs::OpenOptions::new()
        .read(true)
        .share_mode(1)
        .open(t.path().join("game/version.json"))
        .unwrap();
    let error = update::install(t.path(), &package(), &snapshot()).unwrap_err();
    assert!(error.contains("automatic rollback restored"), "{error}");
    assert_eq!(
        fs::read(t.path().join("game/index.html")).unwrap(),
        b"old exact game"
    );
    assert!(
        !t.path()
            .join("userdata/update-transaction/pending")
            .exists()
    );
    drop(lock);
}
#[tokio::test]
async fn insecure_feed_rejected() {
    assert!(
        update::fetch("http://example.com/update", 100)
            .await
            .unwrap_err()
            .contains("HTTPS")
    );
}
#[tokio::test]
async fn dev_server_isolation_and_contract() {
    let t = fixture();
    fs::create_dir(t.path().join("launcher-ui")).unwrap();
    fs::write(t.path().join("launcher-ui/launcher.html"), "launcher").unwrap();
    let app = server::App::new(t.path().to_path_buf(), true);
    let shutdown = app.clone();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let task = tokio::spawn(async move {
        axum::serve(listener, server::router(app))
            .with_graceful_shutdown(async move { shutdown.shutdown.notified().await })
            .await
            .unwrap();
    });
    let client = reqwest::Client::new();
    let url = format!("http://{addr}");
    let r = client
        .get(format!("{url}/api/version"))
        .header("Host", "127.0.0.1:8766")
        .send()
        .await
        .unwrap();
    assert_eq!(
        r.headers()["Cache-Control"],
        "no-store, no-cache, must-revalidate, max-age=0"
    );
    assert_eq!(
        r.json::<serde_json::Value>().await.unwrap()["devMode"],
        true
    );
    for endpoint in ["/api/apply-update", "/api/install-online-update"] {
        assert!(
            !client
                .post(format!("{url}{endpoint}"))
                .header("Host", "127.0.0.1:8766")
                .json(&package())
                .send()
                .await
                .unwrap()
                .status()
                .is_success()
        );
    }
    let r = client
        .post(format!("{url}/api/backup-saves"))
        .header("Host", "127.0.0.1:8766")
        .json(&snapshot())
        .send()
        .await
        .unwrap();
    assert!(r.status().is_success());
    assert!(t.path().join("dev-userdata/backups").exists());
    assert!(!t.path().join("userdata").exists());
    assert_eq!(
        client
            .get(format!("{url}/api/version"))
            .header("Host", "localhost:8766")
            .send()
            .await
            .unwrap()
            .status(),
        403
    );
    assert_eq!(
        client
            .get(format!("{url}/userdata/backups/anything"))
            .header("Host", "127.0.0.1:8766")
            .send()
            .await
            .unwrap()
            .status(),
        404
    );
    client
        .post(format!("{url}/api/shutdown"))
        .header("Host", "127.0.0.1:8766")
        .send()
        .await
        .unwrap();
    task.await.unwrap();
}

#[tokio::test]
async fn production_http_updates_with_fixture_saves() {
    // Ephemeral transport for unit tests only; Host/Origin remain production values.
    let t = fixture();
    let app = server::App::new(t.path().to_path_buf(), false);
    let shutdown = app.clone();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let task = tokio::spawn(async move {
        axum::serve(listener, server::router(app))
            .with_graceful_shutdown(async move { shutdown.shutdown.notified().await })
            .await
            .unwrap();
    });
    let client = reqwest::Client::new();
    let url = format!("http://{addr}");
    let response = client
        .post(format!("{url}/api/apply-update"))
        .header("Host", "127.0.0.1:8765")
        .body("{")
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 400);
    assert_eq!(
        fs::read(t.path().join("game/index.html")).unwrap(),
        b"old exact game"
    );
    let response = client
        .post(format!("{url}/api/apply-update"))
        .header("Host", "127.0.0.1:8765")
        .json(&package())
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 400); // No browser snapshot, no install.
    let mut payload = serde_json::to_value(package()).unwrap();
    payload["saveSnapshot"] = snapshot();
    let response = client
        .post(format!("{url}/api/apply-update"))
        .header("Host", "127.0.0.1:8765")
        .json(&payload)
        .send()
        .await
        .unwrap();
    assert!(
        response.status().is_success(),
        "{}",
        response.text().await.unwrap()
    );
    assert_eq!(
        fs::read_to_string(t.path().join("game/index.html")).unwrap(),
        package().app_html
    );
    let r = client
        .get(format!("{url}/api/backups"))
        .header("Host", "127.0.0.1:8765")
        .send()
        .await
        .unwrap()
        .json::<serde_json::Value>()
        .await
        .unwrap();
    assert_eq!(r["backups"].as_array().unwrap().len(), 1);
    client
        .post(format!("{url}/api/shutdown"))
        .header("Host", "127.0.0.1:8765")
        .send()
        .await
        .unwrap();
    task.await.unwrap();
}
