#
# MASTerm PowerShell Setup Script
# Run: . .\setup-powershell.ps1
#

$ProfilePath = $PROFILE.CurrentUserAllHosts
$ProfileDir = Split-Path -Parent $ProfilePath

Write-Host "Setting up MASTerm for PowerShell..." -ForegroundColor Cyan

# Create profile directory if needed
if (-not (Test-Path $ProfileDir)) {
    New-Item -ItemType Directory -Path $ProfileDir -Force | Out-Null
}

# Create profile if needed
if (-not (Test-Path $ProfilePath)) {
    New-Item -ItemType File -Path $ProfilePath -Force | Out-Null
}

# Check if already configured
$content = Get-Content $ProfilePath -Raw -ErrorAction SilentlyContinue
if ($content -notmatch "masterm init") {
    $initBlock = @"

# Cargo/Rust PATH
`$env:PATH = "`$env:USERPROFILE\.cargo\bin;`$env:PATH"

# MASTerm - Master your Terminal
Invoke-Expression (& masterm init powershell)
"@
    Add-Content -Path $ProfilePath -Value $initBlock
    Write-Host "✓ Added MASTerm to $ProfilePath" -ForegroundColor Green
} else {
    Write-Host "⚠ PowerShell already configured" -ForegroundColor Yellow
}

Write-Host ""
Write-Host "To apply changes, run: . `$PROFILE" -ForegroundColor White
Write-Host "Or restart your terminal." -ForegroundColor White
