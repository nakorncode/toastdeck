param(
    [ValidateSet("Debug", "Release")]
    [string] $Configuration = "Debug"
)

$ErrorActionPreference = "Stop"

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$repoRoot = Split-Path -Parent $scriptDir
$projectPath = Join-Path $repoRoot "ToastDeckA.csproj"

$runningApps = Get-Process -Name "ToastDeckA" -ErrorAction SilentlyContinue
if ($runningApps) {
    Write-Host "Stopping running ToastDeck-A process..."
    $runningApps | Stop-Process -Force
}

Write-Host "Building ToastDeck-A ($Configuration)..."
dotnet build $projectPath --configuration $Configuration
