param(
    [ValidateSet("Debug", "Release")]
    [string] $Configuration = "Debug"
)

$ErrorActionPreference = "Stop"

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$repoRoot = Split-Path -Parent $scriptDir
$buildScript = Join-Path $scriptDir "run-build.ps1"
$targetFramework = "net8.0-windows10.0.19041.0"
$exePath = Join-Path $repoRoot "bin\$Configuration\$targetFramework\ToastDesk.exe"

& $buildScript -Configuration $Configuration

if (-not (Test-Path $exePath)) {
    throw "Built executable was not found: $exePath"
}

Write-Host "Starting ToastDesk..."
Start-Process -FilePath $exePath -WorkingDirectory $repoRoot
