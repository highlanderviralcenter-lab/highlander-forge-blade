# Script de instalacao alpha.2 — Highlander Forge Blade
# Execute na raiz do projeto: .\INSTALAR.ps1
$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $MyInvocation.MyCommand.Path

function Backup($path) {
    if (Test-Path $path) {
        Copy-Item $path "$path.bak" -Force
        Write-Host "  Backup: $path.bak" -ForegroundColor DarkGray
    }
}

function Install($src, $dst) {
    $full = Join-Path $root $dst
    $dir  = Split-Path $full -Parent
    if (!(Test-Path $dir)) { New-Item -ItemType Directory -Path $dir -Force | Out-Null }
    Backup $full
    Copy-Item $src $full -Force
    Write-Host "  OK: $dst" -ForegroundColor Green
}

$files = Split-Path -Parent $MyInvocation.MyCommand.Path

Write-Host "`n=== Highlander Forge Blade — Instalando alpha.2 ===" -ForegroundColor Cyan

Install "$files\src_app_messages.rs"               "src\app\messages.rs"
Install "$files\src_app_state.rs"                  "src\app\state.rs"
Install "$files\src_ui_ratatui_mod.rs"             "src\ui\ratatui\mod.rs"
Install "$files\src_ui_ratatui_app.rs"             "src\ui\ratatui\app.rs"
Install "$files\src_ui_ratatui_views_mod.rs"       "src\ui\ratatui\views\mod.rs"
Install "$files\src_ui_ratatui_views_menu.rs"      "src\ui\ratatui\views\menu.rs"
Install "$files\src_ui_ratatui_views_progress.rs"  "src\ui\ratatui\views\progress.rs"
Install "$files\src_ui_ratatui_views_summary.rs"   "src\ui\ratatui\views\summary.rs"
Install "$files\src_ui_ratatui_views_confirm.rs"   "src\ui\ratatui\views\confirm.rs"
Install "$files\src_ui_ratatui_views_report.rs"    "src\ui\ratatui\views\report.rs"
Install "$files\src_ui_ratatui_views_logs.rs"      "src\ui\ratatui\views\logs.rs"
Install "$files\src_ui_ratatui_views_detailed.rs"  "src\ui\ratatui\views\detailed.rs"
Install "$files\core_audit.rs"                     "src\core\audit.rs"
Install "$files\main_rs.rs"                        "src\main.rs"

Write-Host "`nTodos os arquivos instalados!" -ForegroundColor Cyan
Write-Host "Compilando..." -ForegroundColor Yellow
Set-Location $root
cargo build --features tui 2>&1

if ($LASTEXITCODE -eq 0) {
    Write-Host "`nBUILD OK! Rodando..." -ForegroundColor Green
    .\target\debug\hfb.exe
} else {
    Write-Host "`nBuild falhou. Verifique os erros acima." -ForegroundColor Red
}
