<#
.SYNOPSIS
    Highlander Forge Blade - Mapeamento Completo com Blake3 + Relatorio
.DESCRIPTION
    Mapeia recursivamente C:\highlander-forge-blade, calcula hashes Blake3,
    le conteudo dos arquivos, gera arquivo unico de texto, JSON e HTML.
.NOTES
    Requer: PowerShell 5.1+ ou PowerShell 7.x
    Recomendado: rhash ou b3sum instalado. Se nao houver, usa Get-FileHash (SHA256) como fallback.
#>

param(
    [string]$SourcePath = "C:\highlander-forge-blade",
    [string]$OutputDir = "C:\temp\hfb_mapeamento"
)

# ============================================================================
# CONFIGURACAO
# ============================================================================
$ErrorActionPreference = "Stop"
$ProgressPreference = "Continue"

# Cria pasta de saida
New-Item -ItemType Directory -Force -Path $OutputDir | Out-Null

$ArquivoUnico = Join-Path $OutputDir "hfb_conteudo_completo.txt"
$ArquivoJSON  = Join-Path $OutputDir "hfb_mapeamento.json"
$ArquivoHTML  = Join-Path $OutputDir "hfb_relatorio.html"

# Extensoes de texto que serao lidas
$TextoExtensoes = @(
    '.rs','.toml','.md','.txt','.ps1','.json','.yaml','.yml','.xml',
    '.html','.css','.js','.sh','.bat','.cmd','.ini','.cfg','.conf'
)

# Extensoes binarias a excluir da leitura de conteudo
$BinarioExtensoes = @('.exe','.dll','.bin','.png','.jpg','.jpeg','.gif','.ico','.zip','.7z','.tar','.gz')

# Pastas a excluir
$PastasExcluir = @('target','.git','node_modules','.vs','.vscode','dist','build')

# ============================================================================
# FUNCOES
# ============================================================================

function Get-Blake3Hash {
    param([string]$FilePath)

    # Tenta b3sum primeiro
    $b3sum = Get-Command b3sum -ErrorAction SilentlyContinue
    if ($b3sum) {
        $result = & b3sum --no-names "$FilePath" 2>$null
        if ($result) { return $result.Trim() }
    }

    # Tenta rhash
    $rhash = Get-Command rhash -ErrorAction SilentlyContinue
    if ($rhash) {
        $result = & rhash --blake3 "$FilePath" 2>$null
        if ($result) { 
            # rhash retorna "hash  filepath", pegamos so a hash
            return ($result -split '\s+')[0].Trim() 
        }
    }

    # Fallback: SHA256 do PowerShell (nao e Blake3, mas e o que temos)
    $hash = (Get-FileHash -Algorithm SHA256 -Path $FilePath).Hash
    return "SHA256:$hash"
}

function Test-ArquivoTexto {
    param([string]$Ext)
    return $TextoExtensoes -contains $Ext.ToLower()
}

function Test-ArquivoBinario {
    param([string]$Ext)
    return $BinarioExtensoes -contains $Ext.ToLower()
}

function Get-ConteudoSeguro {
    param([string]$FilePath, [string]$Ext)

    if (Test-ArquivoBinario $Ext) {
        return "[CONTEUDO BINARIO - NAO LIDO]"
    }

    if (-not (Test-ArquivoTexto $Ext)) {
        return "[EXTENSAO DESCONHECIDA - NAO LIDO]"
    }

    try {
        # Tenta UTF-8 primeiro
        $bytes = [System.IO.File]::ReadAllBytes($FilePath)

        # Se arquivo muito grande (>500KB), trunca
        if ($bytes.Length -gt 512000) {
            $bytes = $bytes[0..511999]
            $conteudo = [System.Text.Encoding]::UTF8.GetString($bytes)
            return $conteudo + "`n[... ARQUIVO TRUNCADO - MAIOR QUE 500KB]"
        }

        $conteudo = [System.Text.Encoding]::UTF8.GetString($bytes)
        return $conteudo
    }
    catch {
        return "[ERRO AO LER: $($_.Exception.Message)]"
    }
}

# ============================================================================
# MAPEAMENTO
# ============================================================================

Write-Host "`n========================================" -ForegroundColor Cyan
Write-Host "  Highlander Forge Blade - Mapeamento" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "Origem:  $SourcePath" -ForegroundColor Gray
Write-Host "Destino: $OutputDir" -ForegroundColor Gray
Write-Host ""

if (-not (Test-Path $SourcePath)) {
    Write-Error "Pasta origem nao encontrada: $SourcePath"
    exit 1
}

$files = Get-ChildItem -Path $SourcePath -Recurse -File | Where-Object {
    $rel = $_.FullName.Substring($SourcePath.Length).TrimStart('\')
    $partes = $rel -split '\\'
    $pastaPai = if ($partes.Count -gt 1) { $partes[0] } else { "" }
    $pastaPai -notin $PastasExcluir -and $_.Name -ne 'Cargo.lock'
}

Write-Host "Arquivos encontrados: $($files.Count)" -ForegroundColor Green
Write-Host "Iniciando processamento...`n" -ForegroundColor Yellow

# Array para JSON
$jsonData = @{
    projeto = "highlander-forge-blade"
    gerado_em = (Get-Date -Format "yyyy-MM-dd HH:mm:ss")
    origem = $SourcePath
    total_arquivos = $files.Count
    arquivos = @()
}

# HTML Header
$htmlHeader = @"
<!DOCTYPE html>
<html lang="pt-BR">
<head>
    <meta charset="UTF-8">
    <title>Highlander Forge Blade - Relatorio de Mapeamento</title>
    <style>
        body { font-family: 'Segoe UI', Consolas, monospace; background: #0d1117; color: #c9d1d9; margin: 20px; }
        h1 { color: #58a6ff; border-bottom: 2px solid #30363d; padding-bottom: 10px; }
        h2 { color: #7ee787; margin-top: 30px; }
        .stats { background: #161b22; border: 1px solid #30363d; padding: 15px; border-radius: 8px; margin: 20px 0; }
        .stats span { display: inline-block; margin-right: 30px; }
        .stats .label { color: #8b949e; }
        .stats .value { color: #58a6ff; font-weight: bold; }
        table { width: 100%; border-collapse: collapse; margin-top: 20px; }
        th { background: #21262d; color: #58a6ff; padding: 12px; text-align: left; border: 1px solid #30363d; }
        td { padding: 10px; border: 1px solid #30363d; }
        tr:nth-child(even) { background: #161b22; }
        tr:hover { background: #1c2128; }
        .hash { font-family: Consolas, monospace; color: #d2a8ff; font-size: 0.85em; }
        .path { color: #7ee787; }
        .size { color: #ffa657; text-align: right; }
        .tipo-txt { color: #58a6ff; }
        .tipo-bin { color: #f85149; }
        .conteudo-box { background: #0d1117; border: 1px solid #30363d; padding: 15px; margin-top: 10px; border-radius: 6px; max-height: 400px; overflow: auto; white-space: pre-wrap; font-family: Consolas, monospace; font-size: 0.9em; }
        .btn { display: inline-block; padding: 8px 16px; background: #238636; color: white; text-decoration: none; border-radius: 6px; margin: 5px; }
        .btn:hover { background: #2ea043; }
        .nav { position: sticky; top: 0; background: #0d1117; padding: 10px 0; border-bottom: 1px solid #30363d; z-index: 100; }
    </style>
</head>
<body>
    <h1>🔧 Highlander Forge Blade - Relatorio de Mapeamento</h1>
    <div class="stats">
        <span><span class="label">Projeto:</span> <span class="value">highlander-forge-blade</span></span>
        <span><span class="label">Gerado:</span> <span class="value">$(Get-Date -Format "yyyy-MM-dd HH:mm:ss")</span></span>
        <span><span class="label">Total Arquivos:</span> <span class="value">$($files.Count)</span></span>
    </div>
    <div class="nav">
        <a href="#tabela" class="btn">Tabela de Arquivos</a>
        <a href="#conteudo" class="btn">Conteudo dos Arquivos</a>
    </div>
    <h2 id="tabela">📋 Tabela de Arquivos</h2>
    <table>
        <tr>
            <th>#</th>
            <th>Caminho Relativo</th>
            <th>Extensao</th>
            <th>Tamanho</th>
            <th>Hash (Blake3/SHA256)</th>
            <th>Tipo</th>
        </tr>
"@

$htmlRows = ""
$htmlConteudo = "<h2 id='conteudo'>📄 Conteudo dos Arquivos de Texto</h2>`n"

# Arquivo unico de texto
"HIGHLANDER FORGE BLADE - CONTEUDO COMPLETO`n" | Out-File -FilePath $ArquivoUnico -Encoding UTF8
"Gerado em: $(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')`n" | Out-File -Append -FilePath $ArquivoUnico -Encoding UTF8
"Origem: $SourcePath`n" | Out-File -Append -FilePath $ArquivoUnico -Encoding UTF8
"================================================================================`n`n" | Out-File -Append -FilePath $ArquivoUnico -Encoding UTF8

$contador = 0
$totalTexto = 0
$totalBinario = 0

foreach ($file in $files) {
    $contador++
    $relPath = $file.FullName.Substring($SourcePath.Length).TrimStart('\')
    $ext = $file.Extension
    $isTexto = Test-ArquivoTexto $ext
    $isBinario = Test-ArquivoBinario $ext

    Write-Progress -Activity "Processando arquivos" -Status $relPath -PercentComplete (($contador / $files.Count) * 100)

    # Hash
    try { $hash = Get-Blake3Hash $file.FullName }
    catch { $hash = "ERRO" }

    # Conteudo
    $conteudo = Get-ConteudoSeguro -FilePath $file.FullName -Ext $ext

    # JSON entry
    $entry = @{
        indice = $contador
        caminho_relativo = $relPath
        caminho_absoluto = $file.FullName
        extensao = $ext
        tamanho_bytes = $file.Length
        tamanho_formatado = switch ($file.Length) {
            { $_ -gt 1MB } { "{0:N2} MB" -f ($_/1MB); break }
            { $_ -gt 1KB } { "{0:N2} KB" -f ($_/1KB); break }
            default { "$_ bytes" }
        }
        hash = $hash
        tipo = if ($isTexto) { "texto" } elseif ($isBinario) { "binario" } else { "desconhecido" }
        conteudo = if ($isTexto) { $conteudo } else { "[OMITIDO - ARQUIVO BINARIO]" }
        modificado = $file.LastWriteTime.ToString("yyyy-MM-dd HH:mm:ss")
    }
    $jsonData.arquivos += $entry

    # HTML Tabela
    $tipoClass = if ($isTexto) { "tipo-txt" } else { "tipo-bin" }
    $tipoLabel = if ($isTexto) { "TXT" } elseif ($isBinario) { "BIN" } else { "?" }
    $htmlRows += "<tr><td>$contador</td><td class='path'>$([System.Web.HttpUtility]::HtmlEncode($relPath))</td><td>$ext</td><td class='size'>$($entry.tamanho_formatado)</td><td class='hash'>$hash</td><td class='$tipoClass'>$tipoLabel</td></tr>`n"

    # HTML Conteudo (so texto)
    if ($isTexto) {
        $totalTexto++
        $htmlConteudo += "<div style='margin-top:20px;'><h3 style='color:#58a6ff;'>📄 $([System.Web.HttpUtility]::HtmlEncode($relPath)) <span style='color:#8b949e;font-size:0.8em;'>($($entry.tamanho_formatado))</span></h3>`n"
        $htmlConteudo += "<div class='conteudo-box'>$([System.Web.HttpUtility]::HtmlEncode($conteudo))</div></div>`n"
    } else {
        $totalBinario++
    }

    # Arquivo unico de texto
    $sep = "`n================================================================================`nARQUIVO [$contador/$($files.Count)]: $relPath`nTamanho: $($entry.tamanho_formatado) | Hash: $hash | Tipo: $($entry.tipo)`n================================================================================`n"
    $sep | Out-File -Append -FilePath $ArquivoUnico -Encoding UTF8
    $conteudo | Out-File -Append -FilePath $ArquivoUnico -Encoding UTF8
    "`n" | Out-File -Append -FilePath $ArquivoUnico -Encoding UTF8
}

Write-Progress -Activity "Processando arquivos" -Completed

# ============================================================================
# SALVAR JSON
# ============================================================================
$jsonData | ConvertTo-Json -Depth 10 | Out-File -FilePath $ArquivoJSON -Encoding UTF8

# ============================================================================
# FINALIZAR HTML
# ============================================================================
$htmlFooter = @"
    </table>
    $htmlConteudo
    <div style="margin-top:40px; padding:20px; border-top:2px solid #30363d; color:#8b949e; text-align:center;">
        <p>Highlander Forge Blade v3.0.0-alpha.1 | Relatorio gerado por mapeamento.ps1</p>
    </div>
</body>
</html>
"@

$htmlCompleto = $htmlHeader + $htmlRows + $htmlFooter
$htmlCompleto | Out-File -FilePath $ArquivoHTML -Encoding UTF8

# ============================================================================
# RESUMO
# ============================================================================
$tamanhoTotal = ($files | Measure-Object -Property Length -Sum).Sum

Write-Host "`n========================================" -ForegroundColor Green
Write-Host "  MAPEAMENTO CONCLUIDO!" -ForegroundColor Green
Write-Host "========================================" -ForegroundColor Green
Write-Host ""
Write-Host "📁 Arquivos processados: $($files.Count)" -ForegroundColor White
Write-Host "📄 Arquivos de texto:    $totalTexto" -ForegroundColor Cyan
Write-Host "💾 Arquivos binarios:   $totalBinario" -ForegroundColor Magenta
Write-Host "📦 Tamanho total:        $([math]::Round($tamanhoTotal/1MB, 2)) MB" -ForegroundColor Yellow
Write-Host ""
Write-Host "📋 Arquivos gerados:" -ForegroundColor Green
Write-Host "   1. $ArquivoUnico" -ForegroundColor Gray
Write-Host "   2. $ArquivoJSON" -ForegroundColor Gray
Write-Host "   3. $ArquivoHTML" -ForegroundColor Gray
Write-Host ""
Write-Host "Abra o HTML no navegador para visualizar o relatorio completo." -ForegroundColor Cyan
Write-Host ""

# Abre a pasta de saida
Start-Process explorer.exe -ArgumentList $OutputDir
