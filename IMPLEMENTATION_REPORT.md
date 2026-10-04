# Rust Launcher 3.0.0 implementation report

Status: **NOT READY to replace the stable Go launcher yet.** Implementation and
the available local checks are complete, but the actual production-port server
smoke test is blocked by the user's running Go launcher on 127.0.0.1:8765.
Do not publish this candidate as an EXE-only upgrade: its initial deployment
also requires the new launcher UI assets.

## Repository and protected state

- Repository: adamkendig1-ops/Life-Clicker.
- Working branch: rust-launcher-3.0.0.
- Main remains at 3bd68a67162887e4d1847a90a94699e1d97fc7a3.
- Stable manifest, all stable binaries/packages, launcher-config.json, original
  Go launcher and original workflows remain unchanged.
- No merge, push, publication or real production-save modification occurred.
- Developer and automated tests used isolated storage and synthetic snapshots.

## Added source and architecture

- `game/index.html`, `game/version.json`: editable 7.0.7 source, Save Version 12.
- `launcher-ui/launcher.html`, `launcher-ui/launcher.js`: recovered layout with
  readable JavaScript and fixed initialization/error handling.
- Root Cargo workspace and lockfile; `launcher-rust/` contains the binary,
  server, filesystem storage, game updater, self-update helper and safety tests.
- `tools/`: UTF-8/JS validation, browser-API fixture tests, deterministic package
  generation, install assembly, dev launcher and process/self-update smoke tests.
- `DEVELOPMENT.md`, `README_FIRST.txt`: development, packaging and recovery steps.
- `.github/workflows/build-rust-launcher.yml`: Windows MSVC candidate-only CI.
- `.gitignore`, `.gitattributes`: exclude build/test outputs and preserve game bytes.

Axum/tokio serves the loopback API; reqwest/rustls downloads bounded HTTPS
payloads; serde preserves JSON field names; sha2 verifies payload integrity.
Base64 supports existing embedded executable payloads, now requiring a hash.
Normal production binds only 127.0.0.1:8765. Dev binds only 127.0.0.1:8766,
serves repository source, uses dev-userdata and disables both update routes.
The existing default-browser storage model and game save/migration code remain.

Updates validate package metadata, HTML boundaries/version markers, byte size
and SHA-256 before applying. Browser snapshots are supplied with the install
request, with the legacy backup-then-install sequence also supported. The backend
cannot read localStorage itself. It saves snapshots, backs up HTML, writes and
verifies staged files, then replaces HTML and metadata under a mutation lock.
A durable pending journal permits rollback of both files on failure or startup
after an interrupted update. Save backups retain 25; game backups retain 12.

Self-update validates the staged PE and SHA-256, copies the current executable
to a previous version, uses a temporary helper to replace the executable after
exit, restarts, and cleans up the helper. Current/previous files remain available
when replacement fails. A recovery flag and manual missing-executable procedure
are documented. Dev mode never runs self-replacement.

## UI repair

The old base64-decoded integrity display contained a corrupted string/ternary
boundary immediately before `'missing'`, causing the entire script to fail
parsing. It also called `toLocalString()` rather than `toLocaleString()`.
The new `describe(item)` function uses a valid conditional expression and
`item.bytes.toLocaleString()`; integrity output uses textContent.

Startup now displays Game 7.0.7 / Launcher 3.0.0 / Save Version 12, including at
narrow widths. Initialization failure displays the actual error. Tests establish
one background check, explicit manual checks and no automatic installation.
The dev browser visibly showed DEV MODE and loaded the 7.0.7 profile screen.

## Verification results

| Check | Result |
| --- | --- |
| Rust release compilation on this Windows MSVC machine | PASS |
| cargo test --locked | PASS: 15 tests, zero failures |
| cargo clippy --locked --all-targets -- -D warnings | PASS |
| cargo fmt --check | PASS |
| Launcher and game JS syntax, UTF-8 and required files | PASS |
| UI initialization, failure display, one check, no auto-install | PASS |
| Actual dev process on 127.0.0.1:8766 | PASS |
| Dev assets, version, integrity, info and no-cache headers | PASS |
| Dev update denial and isolated synthetic backups | PASS |
| Actual dev clean shutdown | PASS |
| Occupied dev port: unrelated service and recognized launcher | PASS |
| Rust recognition of existing Go launcher on production 8765 | PASS |
| Rust serving on actual production 127.0.0.1:8765 | BLOCKED |
| Production API install/backup tests on ephemeral fixture listener | PASS |
| Required numeric version comparisons | PASS |
| Bad SHA, wrong size, malformed JSON, mismatched version | PASS |
| Invalid package leaves installation unchanged | PASS |
| Injected replacement failures restore HTML and metadata | PASS |
| Actual Windows locked metadata triggers HTML rollback | PASS |
| Interrupted journal recovery; recovery failure retains copies | PASS |
| Save and game backup retention | PASS |
| Self-update staging, tamper rejection and failure preservation | PASS |
| Real isolated helper replacement, restart and helper cleanup | PASS |
| Deterministic package generation | PASS: two outputs have identical hashes |
| GitHub-hosted CI execution | Not run; workflow added, nothing pushed |

The production-port blocker is Go Launcher 2.1.1 serving game 7.0.6. It was
queried read-only and left running to avoid interrupting the user's session.
After the user saves and closes that launcher, run `node tools/smoke.mjs` with
both fixed ports free. The script refuses occupied ports and never opens a
browser; it creates temporary fixture installations and synthetic saves.

## Game and artifact integrity

The tracked game's 296,054 UTF-8 bytes exactly equal the decoded `appHtml` in
the existing 7.0.7 package. No line-ending change or gameplay refactor occurred.
This preserves the existing Owner Investment, business blockers/assets/OPEX,
career repair, Click Mastery, crime mastery and migration code unchanged.

Game SHA-256:
`4fb88d3e6f09770aadcaf7e267798a5dfd28e77b7a3f6bd8bee3c179cea1e390`

Final executable (4,817,920 bytes):
`dist/LifeClicker-3.0.0/LifeClicker.exe`

Executable SHA-256:
`5c0143f2853c2f29b1fbe5b8476a33e3a7bb8105d22141c4c536b118b5c3fb7a`

The installation folder also includes game HTML/metadata, launcher HTML/JS,
launcher-config.json, README_FIRST.txt and the executable hash sidecar.
Build output is also at `target/release/LifeClicker.exe`.

## Remaining limitations and release recommendation

1. Complete the actual production-port smoke test after the existing Go process
   is closed safely. The fixed production address has not changed.
2. The first migration needs the full install folder. The old Go EXE-only updater
   cannot deploy the new separate UI JavaScript asset. Do not change the stable
   launcher manifest until a rollout method is accepted and tested.
3. HTTPS transport behavior is implemented, but live end-to-end remote release
   installation was not exercised. Payload validation has deterministic tests.
4. Runtime HTML validation is structural/version validation. Full JavaScript
   syntax checking runs in packaging/CI; the launcher does not bundle a JS engine.
5. Replacement uses two atomic file renames with a recovery journal, not a single
   multi-file filesystem transaction. Sudden power-loss durability has not been
   physically tested. SHA-256 verifies feed integrity, not publisher identity.

Recommendation: keep this as a reviewable candidate on the Rust branch, retain
Go 2.1.1 as stable, and complete the outstanding production/rollout validation
before declaring it ready for replacement.
