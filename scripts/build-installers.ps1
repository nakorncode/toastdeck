param(
    [Parameter(Mandatory = $true)]
    [string]$ProductVersion,
    [string]$NsisDir,
    [string]$WixDir
)

# Builds ToastDesk v3 and its NSIS and MSI installers.
# $ProductVersion is the release tag, for example 3.0.0-pre.1. The MSI needs a plain x.y.z version,
# so the numeric part is used there and in the NSIS version string.

$ErrorActionPreference = "Stop"

$root = Split-Path -Parent $PSScriptRoot
$native = Join-Path $root "native"
$packaging = Join-Path $root "packaging\windows"
$stage = Join-Path $root "artifacts\stage"
$out = Join-Path $root "artifacts\release"

$numeric = ($ProductVersion -split "-")[0]
if ($numeric -notmatch '^\d+\.\d+\.\d+$') {
    throw "ProductVersion must start with x.y.z, got '$ProductVersion'"
}

function Find-Tool([string]$name, [string[]]$folders) {
    $command = Get-Command $name -ErrorAction SilentlyContinue
    if ($command) { return $command.Source }
    foreach ($folder in $folders) {
        if ($folder -and (Test-Path (Join-Path $folder $name))) {
            return (Join-Path $folder $name)
        }
    }
    throw "$name was not found. Install it or pass the folder that contains it."
}

$makensis = Find-Tool "makensis.exe" @(
    $NsisDir,
    "${env:ProgramFiles(x86)}\NSIS",
    "$env:ProgramFiles\NSIS",
    "$env:LOCALAPPDATA\tauri\NSIS"
)
$candle = Find-Tool "candle.exe" @(
    $WixDir,
    "${env:ProgramFiles(x86)}\WiX Toolset v3.11\bin",
    "$env:LOCALAPPDATA\tauri\WixTools314"
)
$light = Find-Tool "light.exe" @(
    $WixDir,
    "${env:ProgramFiles(x86)}\WiX Toolset v3.11\bin",
    "$env:LOCALAPPDATA\tauri\WixTools314"
)

Push-Location $native
try {
    cargo build --release
    if ($LASTEXITCODE -ne 0) { throw "cargo build failed" }
} finally {
    Pop-Location
}

if (Test-Path $stage) { Remove-Item -Recurse -Force $stage }
New-Item -ItemType Directory -Path (Join-Path $stage "sounds") | Out-Null
Copy-Item (Join-Path $native "target\release\ToastDeskV3.exe") $stage
Copy-Item (Join-Path $root "assets\sounds\AOSP-LICENSE.md") $stage
Copy-Item (Join-Path $root "assets\sounds\*.wav") (Join-Path $stage "sounds")

if (Test-Path $out) { Remove-Item -Recurse -Force $out }
New-Item -ItemType Directory -Path $out | Out-Null

$setup = Join-Path $out "ToastDesk-v3-$ProductVersion-Setup-win-x64.exe"
& $makensis "/DVERSION=$numeric" "/DSRCDIR=$stage" "/DOUTFILE=$setup" (Join-Path $packaging "toastdesk-v3.nsi")
if ($LASTEXITCODE -ne 0) { throw "makensis failed" }

$obj = Join-Path $stage "toastdesk-v3.wixobj"
$msi = Join-Path $out "ToastDesk-v3-$ProductVersion-Setup-win-x64.msi"
& $candle -nologo "-dVersion=$numeric" "-dStageDir=$stage" -out $obj (Join-Path $packaging "toastdesk-v3.wxs")
if ($LASTEXITCODE -ne 0) { throw "candle failed" }
& $light -nologo -sice:ICE91 -out $msi $obj
if ($LASTEXITCODE -ne 0) { throw "light failed" }

Get-ChildItem $out | ForEach-Object { "{0}  {1:N1} KB" -f $_.Name, ($_.Length / 1KB) }
