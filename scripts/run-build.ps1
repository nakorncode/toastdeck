param(
    [ValidateSet("Debug", "Release")]
    [string] $Configuration = "Debug"
)

$ErrorActionPreference = "Stop"

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$repoRoot = Split-Path -Parent $scriptDir
$projectPath = Join-Path $repoRoot "ToastDesk.csproj"

$runningApps = Get-Process -Name "ToastDesk" -ErrorAction SilentlyContinue
if ($runningApps) {
    Write-Host "Stopping running ToastDesk process..."
    $runningApps | Stop-Process -Force
}

Write-Host "Building ToastDesk ($Configuration)..."
dotnet build $projectPath --configuration $Configuration
