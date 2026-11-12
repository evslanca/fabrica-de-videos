# Fast incremental development launch for Fabrica de Videos (Concat)
$ErrorActionPreference = "Stop"
Push-Location (Join-Path $PSScriptRoot "..\src")
try {
    cargo dev
} finally {
    Pop-Location
}
