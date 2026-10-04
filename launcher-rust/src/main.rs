use life_clicker_launcher::{self_update, server, storage::Result, update};
use std::path::PathBuf;

fn open(url: &str) -> Result<()> {
    self_update::hidden_command("cmd")
        .args(["/c", "start", "", url])
        .spawn()
        .map(|_| ())
        .map_err(|e| format!("Browser launch: {e}"))
}
#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("Life Clicker: {e}");
        std::process::exit(1);
    }
}
async fn run() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.first().is_some_and(|s| s == "--apply-self-update") {
        if args.len() != 3 && !(args.len() == 4 && args[3] == "--no-browser") {
            return Err("Invalid helper arguments".into());
        }
        return self_update::helper(
            std::path::Path::new(&args[1]),
            std::path::Path::new(&args[2]),
            args.len() == 4,
        );
    }
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let mut dev = false;
    let mut no_browser = false;
    let mut root = None;
    let mut recover_launcher = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--dev" => dev = true,
            "--no-browser" => no_browser = true,
            "--recover-launcher" => recover_launcher = true,
            "--root" => {
                i += 1;
                root = Some(PathBuf::from(args.get(i).ok_or("--root requires a path")?));
            }
            "--cleanup-helper" => {
                i += 1;
                self_update::cleanup(std::path::Path::new(
                    args.get(i).ok_or("Missing helper path")?,
                ));
            }
            "--help" => {
                println!(
                    "LifeClicker [--dev] [--root PATH] [--no-browser] [--recover-launcher]\nProduction: 127.0.0.1:8765; development: 127.0.0.1:8766"
                );
                return Ok(());
            }
            other => return Err(format!("Unknown argument {other}")),
        }
        i += 1;
    }
    let root = root
        .unwrap_or_else(|| {
            if dev {
                PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                    .parent()
                    .unwrap()
                    .to_path_buf()
            } else {
                exe.parent().unwrap().to_path_buf()
            }
        })
        .canonicalize()
        .map_err(|e| format!("Installation root: {e}"))?;
    if recover_launcher {
        if dev {
            return Err("Launcher recovery is disabled in dev mode".into());
        }
        let bytes =
            std::fs::read(root.join("LifeClicker.previous.exe")).map_err(|e| e.to_string())?;
        self_update::stage(&root, &bytes, &life_clicker_launcher::storage::hash(&bytes))?;
    }
    let addr = if dev {
        "127.0.0.1:8766"
    } else {
        "127.0.0.1:8765"
    };
    let url = format!("http://{addr}/launcher.html");
    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(l) => l,
        Err(e) => {
            if server::existing(addr).await {
                println!("Existing Life Clicker instance at {addr}");
                if !no_browser {
                    open(&url)?;
                }
                return Ok(());
            }
            return Err(format!(
                "Cannot bind {addr}: {e}. Another application occupies the required save origin; close it and retry. No alternate port was used."
            ));
        }
    };
    if !dev {
        if update::recover(&root)? {
            eprintln!("Recovered interrupted game update from journal");
        }
        match self_update::launch_helper(&root, &exe, no_browser) {
            Ok(true) => return Ok(()),
            Ok(false) => {}
            Err(e) => eprintln!("Staged launcher rejected; continuing current launcher: {e}"),
        }
    }
    let app = server::App::new(root, dev);
    for p in [
        app.root.join("game/index.html"),
        app.root.join("game/version.json"),
        app.ui(),
        app.root.join("launcher-ui/launcher.js"),
    ] {
        if !p.is_file() {
            return Err(format!(
                "Required installation file missing: {}",
                p.display()
            ));
        }
    }
    println!(
        "Life Clicker Launcher {} {} ready at {url}",
        update::VERSION,
        if dev { "DEV MODE" } else { "production" }
    );
    if !no_browser {
        let url = url.clone();
        tokio::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_millis(300)).await;
            if let Err(e) = open(&url) {
                eprintln!("{e}");
            }
        });
    }
    let shutdown = app.clone();
    axum::serve(listener, server::router(app))
        .with_graceful_shutdown(async move { shutdown.shutdown.notified().await })
        .await
        .map_err(|e| e.to_string())
}
