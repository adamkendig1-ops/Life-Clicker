$ErrorActionPreference = 'Stop'
$repoRoot = Split-Path -Parent $PSScriptRoot
Push-Location $repoRoot
try {
    cargo build --release
    if ($LASTEXITCODE -ne 0) { throw 'Rust build failed' }
    & (Join-Path $repoRoot 'target/release/LifeClicker.exe') --dev --root $repoRoot
} finally {
    Pop-Location
}
