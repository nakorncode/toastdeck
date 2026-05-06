param(
    [ValidateSet("win-x64")]
    [string] $Runtime = "win-x64",

    [ValidateSet("Release", "Debug")]
    [string] $Configuration = "Release"
)

$ErrorActionPreference = "Stop"

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$repoRoot = Split-Path -Parent $scriptDir
$projectPath = Join-Path $repoRoot "ToastDesk.csproj"
$publishDir = Join-Path $repoRoot "artifacts\publish\$Runtime"
$releaseDir = Join-Path $repoRoot "artifacts\release"
$zipPath = Join-Path $releaseDir "ToastDesk-$Runtime.zip"

$runningApps = Get-Process -Name "ToastDesk" -ErrorAction SilentlyContinue
if ($runningApps) {
    Write-Host "Stopping running ToastDesk process..."
    $runningApps | Stop-Process -Force
}

if (Test-Path $publishDir) {
    Remove-Item -LiteralPath $publishDir -Recurse -Force
}

New-Item -ItemType Directory -Force -Path $publishDir, $releaseDir | Out-Null

dotnet publish $projectPath `
    --configuration $Configuration `
    --runtime $Runtime `
    --self-contained true `
    --output $publishDir `
    -p:PublishSingleFile=true `
    -p:IncludeNativeLibrariesForSelfExtract=true `
    -p:DebugType=None `
    -p:DebugSymbols=false

if (Test-Path $zipPath) {
    Remove-Item -LiteralPath $zipPath -Force
}

Compress-Archive -Path (Join-Path $publishDir "*") -DestinationPath $zipPath
Write-Host "Release package created: $zipPath"
