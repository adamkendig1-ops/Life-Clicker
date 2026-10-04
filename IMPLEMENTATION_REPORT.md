# Rust Launcher 3.0.0 implementation report

Status: **READY as a complete-folder release candidate.** Final Windows validation
completed on October 4, 2026, including the actual production port and live update
checks. The final ZIP was extracted to a separate temporary folder and tested
with that installation as the executable's working directory, without `--root`
or any repository assets. Initial deployment requires the whole ZIP, not an
EXE-only upgrade. Stable publication and merging remain unauthorized.

## Repository and protected state

- Repository: adamkendig1-ops/Life-Clicker.
- Working branch: rust-launcher-3.0.0.
- Main remains at 3bd68a67162887e4d1847a90a94699e1d97fc7a3.
- Stable manifest, all stable binaries/packages, launcher-config.json, original
  Go launcher and original workflows remain unchanged.
- No merge, stable publication or real production-save inspection/modification occurred.
- Developer and automated tests used isolated storage and synthetic snapshots.
- Master game specifications are retained under `docs/specs/` in documentation-only
  commit `1386e625fe1b53ac2dcb39251516be3e36b3a072`; the working tree was clean
  immediately after that commit. No launcher implementation was repeated.

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
- `.cargo/config.toml`: static Windows C runtime linkage; dependency inspection
  confirms no VCRUNTIME140.dll, MSVCP or dynamic UCRT dependency.

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
| Rust serving on actual production 127.0.0.1:8765 | PASS: complete temporary installation and separately extracted final ZIP |
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
| Live HTTPS startup and forced manual update checks | PASS: both installations; up to date; no installation requested |
| Real production browser UI and Exit button | PASS: versions visible, no DEV MODE, update dialog, no browser errors, clean shutdown |
| Separate ZIP extraction without repository dependencies | PASS |
| Every launcher-used API, including open-folder and explicit no-op online install | PASS: separate fixture copy, synthetic saves only |
| ZIP CRC-32, entry sizes, decompression and exact file list | PASS: all 8 files |
| ZIP checker negative control | PASS: intentionally corrupted CRC rejected |
| Static Windows runtime dependency inspection | PASS: Windows system DLLs only |
| GitHub-hosted CI execution | Separate from these completed local gates; branch push can trigger candidate-only CI |

The earlier production-port blocker was resolved when the user closed Go 2.1.1.
The full `tools/smoke.mjs` suite passed on both fixed ports. The full-installation
checker also passed against the final standalone candidate and separate ZIP
extraction, including the real HTTPS stable feed. It runs served launcher JS
with a storage-access trap, checks there is exactly one startup update request,
forces a manual recheck, verifies no install/backup calls occurred and confirms
no userdata directory or changed installation files remain after shutdown.
The Codex test browser additionally confirmed visible production version labels,
the manual "up to date" dialog and the Exit button. The game/profile UI was not
opened; game HTML was fetched and compared byte-for-byte through HTTP instead.
The user's actual browser saves were neither inspected nor modified.

## Game and artifact integrity

The tracked game's 296,054 UTF-8 bytes exactly equal the decoded `appHtml` in
the existing 7.0.7 package. No line-ending change or gameplay refactor occurred.
This preserves the existing Owner Investment, business blockers/assets/OPEX,
career repair, Click Mastery, crime mastery and migration code unchanged.

Game SHA-256:
`4fb88d3e6f09770aadcaf7e267798a5dfd28e77b7a3f6bd8bee3c179cea1e390`

Final executable (4,926,976 bytes):
`dist/LifeClicker-3.0.0/LifeClicker.exe`

Executable SHA-256:
`c3a861e28e51dc5753b6aeabf78c2a88877850979ddca7201a18e768624221d0`

Final ZIP (2,252,553 bytes):
`dist/Life_Clicker_7.0.7_Rust_Launcher_3.0.0_Release_Candidate.zip`

ZIP SHA-256:
`1983964c82b89665dfdf005439cce3dab5f310439927b06411009be95991e1b5`

Exact files at the ZIP root and subdirectories:

```text
LifeClicker.exe
LifeClicker.exe.sha256
launcher-config.json
README_FIRST.txt
game/index.html
game/version.json
launcher-ui/launcher.html
launcher-ui/launcher.js
```

The installation folder also includes game HTML/metadata, launcher HTML/JS,
launcher-config.json, README_FIRST.txt and the executable hash sidecar.
Build output is also at `target/release/LifeClicker.exe`.

## Remaining limitations and release recommendation

1. The first migration needs the full install folder. The old Go EXE-only updater
   cannot deploy the new separate UI JavaScript asset. Do not change the stable
   launcher manifest until a rollout method is accepted and tested.
2. Live HTTPS startup/manual checks passed. Installing a future remote release
   was not exercised because the current stable feed reports up to date; no
   update was installed without approval. Package installation, bad hashes/sizes,
   rollback and self-replacement are covered by local fixture tests.
3. Runtime HTML validation is structural/version validation. Full JavaScript
   syntax checking runs in packaging/CI; the launcher does not bundle a JS engine.
4. Replacement uses two atomic file renames with a recovery journal, not a single
   multi-file filesystem transaction. Sudden power-loss durability has not been
   physically tested. SHA-256 verifies feed integrity, not publisher identity.

Recommendation: READY for review/use as the complete-folder release candidate.
Push only the Rust branch as authorized. Retain Go 2.1.1 and the existing stable
manifest until stable rollout is separately authorized; do not merge this branch.
