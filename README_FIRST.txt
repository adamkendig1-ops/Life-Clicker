Life Clicker Rust Launcher 3.0.0 — candidate build

Game 7.0.7 / Save Version 12

Keep this entire folder together. Close older launchers before starting
LifeClicker.exe. The launcher uses http://127.0.0.1:8765 in your default browser.
Use the SAME browser profile as before to access existing browser-local saves.

The launcher checks for updates once when its page starts. Installation requires
Download & Install or an explicit local package selection. Save snapshots and
game-code recovery copies are stored under userdata beside the executable.

Developer mode: LifeClicker.exe --dev --root PATH_TO_REPOSITORY
It uses 127.0.0.1:8766, separate browser storage and dev-userdata backups.
Updates and launcher replacement are disabled in developer mode.

If game installation is interrupted, next production startup restores the
pending transaction before serving the game. Do not delete recovery files.
For launcher recovery, run LifeClicker.exe --recover-launcher to stage the
previous executable. If LifeClicker.exe is missing, copy LifeClicker.previous.exe
to LifeClicker.exe first, then launch it. Keep the previous copy until verified.

This candidate is not yet published to the stable update feed.
