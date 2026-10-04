# Life Clicker development

Game source is `game/index.html`; its initial UTF-8 bytes exactly match the
7.0.7 package's decoded `appHtml`. Save Version remains 12. Do not change game
mechanics as part of launcher maintenance. `launcher-ui/launcher.html` and
`launcher-ui/launcher.js` are readable launcher sources; no new base64 embeds.

From the repository root (Rust 1.98+, Windows x64 MSVC and Node.js required):

The Windows target configuration statically links the C runtime so the release
EXE does not depend on a separately installed `VCRUNTIME140.dll`. Compiler and
Node.js tools are only needed for development, not for the assembled installation.

```powershell
node tools/validate.mjs
node tools/ui-test.mjs
cargo fmt --check
cargo test --locked
cargo clippy --locked --all-targets -- -D warnings
cargo build --locked --release
.\target\release\LifeClicker.exe --dev --root .
# Or build and launch:
.\tools\dev.ps1
```

Dev mode serves the repository source directly on **127.0.0.1:8766**. Reload the
game after editing. That different origin deliberately isolates browser saves
from production **127.0.0.1:8765**. Dev snapshots use `dev-userdata`, and both
update installation routes and launcher self-replacement are disabled. The
read-only `/api/dev` endpoint reports source paths, branch and launcher version.

Production uses files beside the executable. It never falls back to another
port. A recognized existing Life Clicker server is reused; an unrelated occupied
port produces an error. `--root PATH --no-browser` is available for fixture tests.
Do not point production tests at a real installation. Do not open a browser on
port 8765 during tests: an existing browser profile may contain real saves.

Run the Windows smoke suite after building, with both ports initially free:

```powershell
node tools/smoke.mjs
node tools/self-update-smoke.mjs
# If production 8765 is occupied, validate dev processes separately (8766 must be free):
node tools/dev-process-smoke.mjs
```

It creates temporary installations, launches no browser, uses only synthetic
snapshots, and retains fixture files for inspection. Rust tests also use temporary
directories and inject replacement failures. `tools/ui-test.mjs` executes the
actual launcher JS with a browser-API fixture; it is not a full browser test.

For final release validation of an assembled folder or independently extracted
ZIP, use `node tools/release-candidate-check.mjs ABSOLUTE_INSTALL_FOLDER`.
It starts that folder's EXE with the installation as its working directory and
without `--root`; validates served files against the installation; runs the
served launcher JS against the live API and HTTPS update feed with a browser-
storage access trap; checks manual updates and second-instance behavior; then
shuts down and verifies that every installation file is unchanged. Port 8765
must be free and outbound HTTPS must be available. `--serve` allows a visual
check in a separate test browser instead; stop it through the launcher Exit
button. Never open the game or real browser profile during release validation.

`node tools/api-contract-smoke.mjs ABSOLUTE_INSTALL_FOLDER` copies a complete
installation into another temporary directory and explicitly exercises every
launcher-used API with synthetic snapshots. It also opens the fixture's data
folder in Windows Explorer. The online install endpoint is tested only when
the live feed reports no update, so it must return an up-to-date no-op.

After creating the release ZIP, run `node tools/validate-zip.mjs PATH_TO_ZIP`.
The checker independently decompresses each entry and validates CRC-32, byte
size and the exact eight-file runtime inventory. Documentation/specifications
remain in the repository and are not runtime installation dependencies.

## Release preparation (no publication)

For a future game release, edit the source version constant and corresponding
`game/version.json`, keeping the save version unless the game schema changes.
Create a JSON array of release-note strings, then:

```powershell
node tools/package.mjs 7.0.7 12 notes.json dist/candidate.lcupdate
node tools/assemble.mjs target/release/LifeClicker.exe dist/LifeClicker-3.0.0
```

Packaging validates UTF-8, required files and game/launcher JS syntax. It emits
deterministic UTF-8 JSON without timestamps, reports exact byte size and SHA-256,
refuses an existing output and refuses direct output into `stable`. Assembly
copies source assets and the built executable into a reviewable install folder.
Neither tool edits `stable/latest.json`, replaces the stable executable, commits,
pushes, or publishes. The Rust Windows workflow uploads candidate artifacts only.
The original Go source and workflows remain available as rollback history.

## Compatibility and recovery

Browser storage keys and game migration code are unchanged. The UI submits a
snapshot with the install request. The backend validates the complete package
and any executable payload before writing the snapshot, backing up code, and
replacing files. The legacy backup-then-install API sequence is also supported.
The server cannot read browser localStorage itself. Backup JSON preserves string
values; only the explicit browser Restore action restores browser data.

The transaction journal holds old HTML and metadata, and is marked pending
before replacement. Each file replacement is atomic; two paths cannot be renamed
in one atomic filesystem operation. Failures restore both files. A crash before
the journal commit causes next-start recovery. The recovery copy is never pruned.
Game backup retention is 12; save snapshot retention is 25.

SHA-256 is mandatory for online payloads and staged executables. Local game JSON
has no external trusted hash; it is parsed and validated before replacement.
Runtime HTML validation checks complete document/script boundaries and version
markers; JavaScript syntax is a packaging gate, not a bundled JS engine. Legacy
embedded launcher payloads now need `launcherSha256`. Unhashed legacy next.exe
files are rejected with a diagnostic and the working launcher continues.

Self-update uses a temporary copy of the executable as a helper. It verifies
the staged PE/hash, preserves `LifeClicker.previous.exe`, retries atomic
replacement while the old process exits, restarts the new executable and cleans
up the helper. On failure it preserves the current executable and recovery copy.
SHA-256 provides integrity relative to the feed, not publisher authentication.

The first Go-to-Rust migration needs the complete assembled installation, including
`launcher-ui/launcher.js`; publishing only the Rust EXE through the old launcher
feed is insufficient to deploy those UI assets. Subsequent binary self-updates
retain the installed UI assets. A stable rollout must account for this explicitly.

The old launcher reported `mandatory` and `launcherTooOld` without enforcing
them; the Rust implementation preserves that policy. Mutating routes are POST,
Host/Origin are checked, and only public assets are served. Private backups and
executables are not exposed through generic static file serving.

Dependencies: axum/tokio for local HTTP and async coordination, reqwest/rustls for
bounded HTTPS downloads, serde/serde_json for existing JSON contracts, sha2 for
integrity, base64 for legacy embedded launcher payloads. `tempfile` is test-only.
`Cargo.lock` is committed for repeatable dependency resolution.
