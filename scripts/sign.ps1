# Assinatura de codigo (requer certificado)
$ErrorActionPreference = "Stop"

$cert = Get-ChildItem Cert:\CurrentUser\My -CodeSigningCert | Select-Object -First 1
if (-not $cert) {
    Write-Error "Certificado de assinatura nao encontrado!"
}

$bin = "target\release\hfb.exe"
Set-AuthenticodeSignature -FilePath $bin -Certificate $cert -TimestampServer "http://timestamp.digicert.com"

Write-Host "Assinatura concluida!" -ForegroundColor Green
