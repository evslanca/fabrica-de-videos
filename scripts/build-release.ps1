# Release build and workspace test verification for Fabrica de Videos (Concat)
$ErrorActionPreference = "Stop"
Push-Location (Join-Path $PSScriptRoot "..\src")
try {
    Write-Host "Running workspace unit tests..." -ForegroundColor Cyan
    cargo test --workspace
    Write-Host "Building release binary..." -ForegroundColor Cyan
    cargo build --release --bin concat
    Write-Host "Done: target\release\concat.exe" -ForegroundColor Green
} finally {
    Pop-Location
}
