param(
    [Parameter(Mandatory = $true)]
    [string]$ProductVersion
)

$ErrorActionPreference = "Stop"

$root = Split-Path -Parent $PSScriptRoot
$out = Join-Path $root "artifacts\release"
$bundleRoot = Join-Path $root "src-tauri\target\release\bundle"

if (Test-Path $out) {
    Remove-Item -Recurse -Force $out
}
New-Item -ItemType Directory -Path $out | Out-Null

$nsis = Get-ChildItem -Path (Join-Path $bundleRoot "nsis") -Filter "*-setup.exe" -ErrorAction SilentlyContinue |
    Select-Object -First 1
$msi = Get-ChildItem -Path (Join-Path $bundleRoot "msi") -Filter "*.msi" -ErrorAction SilentlyContinue |
    Select-Object -First 1

if (-not $nsis) {
    throw "NSIS setup.exe not found under $bundleRoot\nsis"
}
if (-not $msi) {
    throw "MSI not found under $bundleRoot\msi"
}

$setupExe = Join-Path $out "ToastDesk-$ProductVersion-Setup-win-x64.exe"
$setupMsi = Join-Path $out "ToastDesk-$ProductVersion-Setup-win-x64.msi"
Copy-Item $nsis.FullName $setupExe
Copy-Item $msi.FullName $setupMsi

$exe = Join-Path $root "src-tauri\target\release\ToastDesk.exe"
if (-not (Test-Path $exe)) {
    $exe = Join-Path $root "src-tauri\target\release\toastdesk.exe"
}
if (Test-Path $exe) {
    $stage = Join-Path $out "portable-stage"
    New-Item -ItemType Directory -Path (Join-Path $stage "assets\sounds") | Out-Null
    Copy-Item $exe (Join-Path $stage "ToastDesk.exe")
    Copy-Item (Join-Path $root "LICENSE") (Join-Path $stage "LICENSE")
    Copy-Item (Join-Path $root "assets\sounds\*.wav") (Join-Path $stage "assets\sounds")
    $upSounds = Join-Path $root "src-tauri\target\release\_up_\assets\sounds"
    if (Test-Path $upSounds) {
        New-Item -ItemType Directory -Path (Join-Path $stage "_up_\assets\sounds") | Out-Null
        Copy-Item (Join-Path $upSounds "*.wav") (Join-Path $stage "_up_\assets\sounds")
    }
    Compress-Archive -Path (Join-Path $stage "*") -DestinationPath (Join-Path $out "ToastDesk-$ProductVersion-Portable-win-x64.zip") -CompressionLevel Optimal
    Remove-Item -Recurse -Force $stage
}

Get-ChildItem $out -File | ForEach-Object {
    $hash = (Get-FileHash $_.FullName -Algorithm SHA256).Hash.ToLowerInvariant()
    "{0}  {1}" -f $hash, $_.Name
} | Set-Content -Encoding ascii (Join-Path $out "ToastDesk-SHA256SUMS.txt")

Get-ChildItem $out -File | Select-Object Name, Length
