param(
  [string]$DatabaseUrl,
  [string]$OutputPath = ".\\backups\\aeroledger-$(Get-Date -Format yyyyMMdd-HHmmss).sql"
)

if (-not $DatabaseUrl) {
  throw "Provide -DatabaseUrl"
}

New-Item -ItemType Directory -Force -Path (Split-Path $OutputPath) | Out-Null
pg_dump $DatabaseUrl --file $OutputPath
Write-Host "Backup written to $OutputPath"
