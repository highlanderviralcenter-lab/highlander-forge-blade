# Build otimizado para release
$ErrorActionPreference = "Stop"

Write-Host "=== Highlander Forge Blade — Build Release ===" -ForegroundColor Cyan

# Verificar toolchain
rustc --version
cargo --version

# Build TUI
Write-Host "`nBuilding TUI..." -ForegroundColor Yellow
cargo build --release --features tui

# Strip e info
$bin = "targetelease\hfb.exe"
if (Test-Path $bin) {
    $size = (Get-Item $bin).Length
    Write-Host "`nBinario: $bin" -ForegroundColor Green
    Write-Host "Tamanho: $([math]::Round($size/1MB, 2)) MB" -ForegroundColor Green
} else {
    Write-Error "Build falhou!"
}
