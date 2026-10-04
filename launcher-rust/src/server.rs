use crate::{
    self_update,
    storage::{self, Result},
    update::{self, Manifest, Package},
};
use axum::{
    Json, Router,
    body::Bytes,
    extract::{DefaultBodyLimit, Query, State},
    http::{HeaderMap, StatusCode, header},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use base64::Engine;
use serde_json::{Value, json};
use std::{path::PathBuf, sync::Arc};
use tokio::sync::{Mutex, Notify};

pub struct App {
    pub root: PathBuf,
    pub dev: bool,
    pub shutdown: Notify,
    online: Mutex<Option<(Value, Option<Manifest>)>>,
    mutation: Mutex<()>,
    snapshot: Mutex<Option<Value>>,
}
impl App {
    pub fn new(root: PathBuf, dev: bool) -> Arc<Self> {
        Arc::new(Self {
            root,
            dev,
            shutdown: Notify::new(),
            online: Mutex::new(None),
            mutation: Mutex::new(()),
            snapshot: Mutex::new(None),
        })
    }
    pub fn data(&self) -> PathBuf {
        self.root
            .join(if self.dev { "dev-userdata" } else { "userdata" })
    }
    pub fn ui(&self) -> PathBuf {
        let clean = self.root.join("launcher-ui/launcher.html");
        if clean.exists() {
            clean
        } else {
            self.root.join("launcher.html")
        }
    }
    pub fn version(&self) -> Value {
        let mut v: Value = storage::read(&self.root.join("game/version.json"))
            .ok()
            .and_then(|b| serde_json::from_slice(&b).ok())
            .filter(Value::is_object)
            .unwrap_or(json!({"version":"0.0.0","saveVersion":0}));
        v["launcherVersion"] = json!(update::VERSION);
        v["product"] = json!("Life Clicker");
        v["devMode"] = json!(self.dev);
        v
    }
    async fn check(&self, force: bool) -> (Value, Option<Manifest>) {
        let mut cache = self.online.lock().await;
        if !force && let Some(c) = cache.as_ref() {
            return c.clone();
        }
        let result=async {
            if self.dev {return Ok((json!({"configured":false,"devMode":true,"message":"DEV MODE: online installation disabled"}),None));}
            let config:Value=serde_json::from_slice(&storage::read(&self.root.join("launcher-config.json"))?).map_err(|e|e.to_string())?;
            let url=config["manifestUrl"].as_str().unwrap_or("");
            if url.is_empty() {return Ok((json!({"configured":false}),None));}
            let m:Manifest=serde_json::from_slice(&update::fetch(url,2<<20).await?).map_err(|e|format!("Invalid manifest: {e}"))?;
            let cur=self.version();let ga=update::greater(&m.game_version,cur["version"].as_str().unwrap_or("0"));let la=update::greater(&m.launcher_version,update::VERSION);
            let r=json!({"configured":true,"checkedAt":storage::stamp(),"channel":config["channel"].as_str().unwrap_or("stable"),"manifestUrl":url,"available":ga||la,"gameAvailable":ga,"launcherAvailable":la,"launcherTooOld":update::greater(&m.minimum_launcher_version,update::VERSION),"currentGameVersion":cur["version"],"currentLauncherVersion":update::VERSION,"gameVersion":m.game_version,"saveVersion":m.save_version,"launcherVersion":m.launcher_version,"mandatory":m.mandatory,"releaseNotes":m.release_notes});
            Ok::<_,String>((r,Some(m)))
        }.await.unwrap_or_else(|e|(json!({"configured":true,"error":e}),None));
        *cache = Some(result.clone());
        result
    }
}
type S = State<Arc<App>>;
fn response(r: Result<Value>) -> Response {
    match r {
        Ok(v) => Json(v).into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, Json(json!({"error":e}))).into_response(),
    }
}
fn parse(b: &[u8]) -> Result<Value> {
    serde_json::from_slice(b).map_err(|e| format!("Invalid JSON: {e}"))
}
async fn version(State(s): S) -> Json<Value> {
    let _lock = s.mutation.lock().await;
    Json(s.version())
}
async fn online(
    State(s): S,
    Query(q): Query<std::collections::HashMap<String, String>>,
) -> Json<Value> {
    Json(s.check(q.get("force").is_some_and(|s| s == "1")).await.0)
}
async fn backup(State(s): S, b: Bytes) -> Response {
    let _lock = s.mutation.lock().await;
    let r = (|| {
        let v = parse(&b)?;
        let name = storage::backup(&s.data(), &v)?;
        Ok((v, name))
    })();
    match r {
        Ok((v, name)) => {
            *s.snapshot.lock().await = Some(v);
            response(Ok(json!({"ok":true,"name":name})))
        }
        Err(e) => response(Err(e)),
    }
}
async fn backups(State(s): S) -> Json<Value> {
    Json(storage::backups(&s.data()))
}
async fn get_backup(
    State(s): S,
    Query(q): Query<std::collections::HashMap<String, String>>,
) -> Response {
    response((|| {
        parse(&storage::read(&storage::backup_path(
            &s.data(),
            q.get("name").map(String::as_str).unwrap_or(""),
        )?)?)
    })())
}
async fn snapshot(s: &App, v: &Value) -> Result<Value> {
    if let Some(v) = v.get("saveSnapshot") {
        Ok(v.clone())
    } else {
        s.snapshot
            .lock()
            .await
            .clone()
            .ok_or("Create a browser save snapshot before installing an update".into())
    }
}
fn embedded_launcher(p: &Package) -> Result<Option<Vec<u8>>> {
    if p.launcher_exe_base64.is_empty() {
        return Ok(None);
    }
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(&p.launcher_exe_base64)
        .map_err(|e| e.to_string())?;
    self_update::validate(&bytes, &p.launcher_sha256)?;
    Ok(Some(bytes))
}
async fn apply(State(s): S, b: Bytes) -> Response {
    if s.dev {
        return response(Err(
            "DEV MODE: installation is disabled; edit repository source instead".into(),
        ));
    }
    let _lock = s.mutation.lock().await;
    let result = async {
        let v = parse(&b)?;
        let p: Package = serde_json::from_value(v.clone()).map_err(|e| e.to_string())?;
        update::validate_package(&p)?;
        let launcher = embedded_launcher(&p)?;
        let snap = snapshot(&s, &v).await?;
        let mut r = update::install(&s.root, &p, &snap)?;
        if let Some(bytes) = launcher {
            match self_update::stage(&s.root, &bytes, &p.launcher_sha256) {
                Ok(()) => r["launcherStaged"] = json!(true),
                Err(e) => r["launcherStageError"] = json!(e),
            }
        }
        *s.online.lock().await = None;
        Ok(r)
    }
    .await;
    response(result)
}
async fn install_online(State(s): S, b: Bytes) -> Response {
    if s.dev {
        return response(Err("DEV MODE: online installation disabled".into()));
    }
    let _lock = s.mutation.lock().await;
    let result=async {
        let v=if b.is_empty(){json!({})}else{parse(&b)?};
        let (status,manifest)=s.check(true).await;
        if let Some(e)=status["error"].as_str(){return Err(e.to_string());}
        if status["available"]!=true{return Ok(json!({"ok":true,"gameInstalled":false,"launcherStaged":false,"message":"already up to date"}));}
        let m=manifest.ok_or("Online manifest unavailable")?;
        let p=if status["gameAvailable"]==true{Some(update::decode(&update::fetch(&m.update_url,64<<20).await?,&m)?)}else{None};
        let mut launcher=if let Some(p)=p.as_ref(){embedded_launcher(p)?.map(|b|(b,p.launcher_sha256.clone()))}else{None};
        if status["launcherAvailable"]==true {
            let bytes=update::fetch(&m.launcher_url,32<<20).await?;self_update::validate(&bytes,&m.launcher_sha256)?;launcher=Some((bytes,m.launcher_sha256.clone()));
        }
        let snap=snapshot(&s,&v).await?;
        let mut r=if let Some(p)=p{update::install(&s.root,&p,&snap)?}else{storage::backup(&s.data(),&snap)?;json!({"ok":true,"gameInstalled":false,"launcherStaged":false})};
        if let Some((bytes,hash))=launcher{match self_update::stage(&s.root,&bytes,&hash){Ok(())=>r["launcherStaged"]=json!(true),Err(e)=>r["launcherStageError"]=json!(e)}}
        *s.online.lock().await=None;Ok(r)
    }.await;
    response(result)
}
async fn integrity(State(s): S) -> Json<Value> {
    fn info(p: PathBuf) -> Value {
        match storage::read(&p) {
            Ok(b) => json!({"exists":true,"bytes":b.len(),"sha256":storage::hash(&b)}),
            Err(_) => json!({"exists":false,"bytes":0,"sha256":""}),
        }
    }
    let game = info(s.root.join("game/index.html"));
    let launcher = info(s.ui());
    Json(
        json!({"ok":game["exists"]==true&&launcher["exists"]==true,"game":game,"launcher":launcher}),
    )
}
async fn open_folder(
    State(s): S,
    Query(q): Query<std::collections::HashMap<String, String>>,
) -> Response {
    let path = if q.get("target").is_some_and(|s| s == "userdata") {
        s.data()
    } else {
        s.root.clone()
    };
    response(
        std::fs::create_dir_all(&path)
            .and_then(|_| {
                self_update::hidden_command("explorer.exe")
                    .arg(path)
                    .spawn()
            })
            .map(|_| json!({"ok":true}))
            .map_err(|e| e.to_string()),
    )
}
async fn shutdown(State(s): S) -> Json<Value> {
    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_millis(150)).await;
        s.shutdown.notify_one();
    });
    Json(json!({"ok":true}))
}
async fn dev_info(State(s): S) -> Response {
    if !s.dev {
        return StatusCode::NOT_FOUND.into_response();
    }
    let branch = self_update::hidden_command("git")
        .arg("-C")
        .arg(&s.root)
        .args(["branch", "--show-current"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string());
    response(Ok(
        json!({"devMode":true,"gameSource":s.root.join("game/index.html"),"launcherSource":s.ui(),"gitBranch":branch,"launcherVersion":update::VERSION}),
    ))
}
async fn static_file(State(s): S, uri: axum::http::Uri) -> Response {
    let _lock = s.mutation.lock().await;
    // Only published assets are served; executable, config, journals and backups are private.
    let (path, kind) = match uri.path() {
        "/" | "/launcher.html" => (s.ui(), "text/html; charset=utf-8"),
        "/launcher.js" => (
            s.root.join("launcher-ui/launcher.js"),
            "text/javascript; charset=utf-8",
        ),
        "/game/index.html" => (s.root.join("game/index.html"), "text/html; charset=utf-8"),
        "/game/version.json" => (
            s.root.join("game/version.json"),
            "application/json; charset=utf-8",
        ),
        _ => return StatusCode::NOT_FOUND.into_response(),
    };
    match storage::read(&path) {
        Ok(b) => ([(header::CONTENT_TYPE, kind)], b).into_response(),
        Err(e) => (StatusCode::NOT_FOUND, e).into_response(),
    }
}
async fn guard(
    State(s): S,
    headers: HeaderMap,
    request: axum::extract::Request,
    next: Next,
) -> Response {
    let expected = if s.dev {
        "127.0.0.1:8766"
    } else {
        "127.0.0.1:8765"
    };
    if headers.get(header::HOST).is_some_and(|h| h != expected)
        || headers
            .get(header::ORIGIN)
            .is_some_and(|h| h != format!("http://{expected}").as_str())
    {
        return (StatusCode::FORBIDDEN, "Origin/Host does not match launcher").into_response();
    }
    let mut r = next.run(request).await;
    r.headers_mut().insert(
        header::CACHE_CONTROL,
        "no-store, no-cache, must-revalidate, max-age=0"
            .parse()
            .unwrap(),
    );
    r.headers_mut()
        .insert(header::PRAGMA, "no-cache".parse().unwrap());
    r.headers_mut()
        .insert(header::EXPIRES, "0".parse().unwrap());
    r
}
pub fn router(s: Arc<App>) -> Router {
    Router::new()
        .route("/api/version", get(version))
        .route("/api/online-update", get(online))
        .route("/api/install-online-update", post(install_online))
        .route("/api/apply-update", post(apply))
        .route("/api/backup-saves", post(backup))
        .route("/api/backups", get(backups))
        .route("/api/backup", get(get_backup))
        .route("/api/integrity", get(integrity))
        .route("/api/open-folder", post(open_folder))
        .route("/api/shutdown", post(shutdown))
        .route("/api/dev", get(dev_info))
        .fallback(get(static_file))
        .layer(DefaultBodyLimit::max(64 << 20))
        .layer(middleware::from_fn_with_state(s.clone(), guard))
        .with_state(s)
}
pub async fn existing(addr: &str) -> bool {
    let Ok(client) = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(2))
        .build()
    else {
        return false;
    };
    match client
        .get(format!("http://{addr}/api/version"))
        .send()
        .await
    {
        Ok(r) => r
            .json::<Value>()
            .await
            .is_ok_and(|v| v["product"] == "Life Clicker" && v["launcherVersion"].is_string()),
        Err(_) => false,
    }
}
