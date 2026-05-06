param(
    [ValidateSet("win-x64")]
    [string] $Runtime = "win-x64",

    [ValidateSet("Release", "Debug")]
    [string] $Configuration = "Release",

    [string] $ProductVersion,

    [switch] $SkipInstallers,

    [switch] $SkipSetupExe,

    [switch] $SkipMsi
)

$ErrorActionPreference = "Stop"

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$repoRoot = Split-Path -Parent $scriptDir
$projectPath = Join-Path $repoRoot "ToastDesk.csproj"
$publishDir = Join-Path $repoRoot "artifacts\publish\$Runtime"
$releaseDir = Join-Path $repoRoot "artifacts\release"
$installerDir = Join-Path $repoRoot "artifacts\installer"
$zipPath = Join-Path $releaseDir "ToastDesk-Portable-$Runtime.zip"
$setupExePath = Join-Path $releaseDir "ToastDesk-Setup-$Runtime.exe"
$setupMsiPath = Join-Path $releaseDir "ToastDesk-Setup-$Runtime.msi"
$checksumPath = Join-Path $releaseDir "ToastDesk-SHA256SUMS.txt"
$appIconPath = Join-Path $repoRoot "assets\icons\ToastDesk.ico"
$innoScriptPath = Join-Path $repoRoot "installer\ToastDesk.iss"

function Get-ProductVersion {
    if (-not [string]::IsNullOrWhiteSpace($ProductVersion)) {
        return $ProductVersion.TrimStart("v")
    }

    if ($env:GITHUB_REF_TYPE -eq "tag" -and -not [string]::IsNullOrWhiteSpace($env:GITHUB_REF_NAME)) {
        return $env:GITHUB_REF_NAME.TrimStart("v")
    }

    $gitTag = git -C $repoRoot describe --tags --exact-match 2>$null
    if (-not [string]::IsNullOrWhiteSpace($gitTag)) {
        return $gitTag.TrimStart("v")
    }

    [xml] $projectXml = Get-Content $projectPath
    return $projectXml.Project.PropertyGroup.Version
}

function Get-FileVersion([string] $version) {
    $parts = $version.Split(".")
    while ($parts.Count -lt 4) {
        $parts += "0"
    }

    return ($parts | Select-Object -First 4) -join "."
}

function Get-IsccPath {
    $command = Get-Command "ISCC.exe" -ErrorAction SilentlyContinue
    if ($command) {
        return $command.Source
    }

    $knownPaths = @(
        "${env:ProgramFiles(x86)}\Inno Setup 6\ISCC.exe",
        "$env:ProgramFiles\Inno Setup 6\ISCC.exe"
    )

    foreach ($path in $knownPaths) {
        if (Test-Path $path) {
            return $path
        }
    }

    return $null
}

function ConvertTo-WixSafeId([string] $value) {
    $safe = $value -replace "[^A-Za-z0-9_\.]", "_"
    if ($safe -notmatch "^[A-Za-z_]") {
        $safe = "Id_$safe"
    }

    return $safe
}

function Escape-Xml([string] $value) {
    return [System.Security.SecurityElement]::Escape($value)
}

function New-ToastDeskMsi([string] $version) {
    $wixCommand = Get-Command "wix" -ErrorAction SilentlyContinue
    if (-not $wixCommand) {
        throw "WiX CLI was not found. Install it with: dotnet tool install --global wix"
    }

    $wxsPath = Join-Path $installerDir "ToastDesk.generated.wxs"
    $componentRefs = New-Object System.Collections.Generic.List[string]
    $fileComponents = New-Object System.Collections.Generic.List[string]
    $index = 0

    Get-ChildItem -Path $publishDir -File -Recurse | Sort-Object FullName | ForEach-Object {
        $relativePath = $_.FullName.Substring($publishDir.TrimEnd("\").Length + 1)
        $relativeDirectory = Split-Path -Parent $relativePath
        $directoryId = switch ($relativeDirectory) {
            "" { "INSTALLFOLDER" }
            "assets" { "AssetsFolder" }
            "assets\icons" { "AssetsIconsFolder" }
            "assets\sounds" { "AssetsSoundsFolder" }
            default { throw "Unhandled MSI publish directory: $relativeDirectory" }
        }

        $componentId = ConvertTo-WixSafeId "cmp_$index"
        $fileId = ConvertTo-WixSafeId "fil_$index"
        $sourcePath = Escape-Xml $_.FullName
        $name = Escape-Xml $_.Name

        $fileComponents.Add("    <DirectoryRef Id=""$directoryId"">
      <Component Id=""$componentId"" Guid=""*"">
        <File Id=""$fileId"" Source=""$sourcePath"" Name=""$name"" KeyPath=""yes"" />
      </Component>
    </DirectoryRef>") | Out-Null
        $componentRefs.Add("      <ComponentRef Id=""$componentId"" />") | Out-Null
        $index++
    }

    $escapedIconPath = Escape-Xml $appIconPath
    $escapedVersion = Escape-Xml $version
    $fileComponentXml = $fileComponents -join [Environment]::NewLine
    $componentRefXml = $componentRefs -join [Environment]::NewLine

    $wxs = @"
<Wix xmlns="http://wixtoolset.org/schemas/v4/wxs">
  <Package Name="ToastDesk" Manufacturer="NakornCode" Version="$escapedVersion" UpgradeCode="{2F8F8BF4-5B80-495B-B7F0-E571B477AFEF}" Scope="perMachine">
    <MajorUpgrade DowngradeErrorMessage="A newer version of ToastDesk is already installed." />
    <MediaTemplate EmbedCab="yes" />
    <Icon Id="ToastDeskIcon" SourceFile="$escapedIconPath" />
    <Property Id="ARPPRODUCTICON" Value="ToastDeskIcon" />

    <StandardDirectory Id="ProgramFiles64Folder">
      <Directory Id="INSTALLFOLDER" Name="ToastDesk">
        <Directory Id="AssetsFolder" Name="assets">
          <Directory Id="AssetsIconsFolder" Name="icons" />
          <Directory Id="AssetsSoundsFolder" Name="sounds" />
        </Directory>
      </Directory>
    </StandardDirectory>

    <StandardDirectory Id="ProgramMenuFolder">
      <Directory Id="ApplicationProgramsFolder" Name="ToastDesk" />
    </StandardDirectory>

$fileComponentXml
    <DirectoryRef Id="ApplicationProgramsFolder">
      <Component Id="ApplicationShortcut" Guid="*">
        <Shortcut Id="ApplicationStartMenuShortcut" Name="ToastDesk" Description="Persistent Windows notification overlay" Target="[INSTALLFOLDER]ToastDesk.exe" WorkingDirectory="INSTALLFOLDER" Icon="ToastDeskIcon" />
        <RemoveFolder Id="ApplicationProgramsFolder" On="uninstall" />
        <RegistryValue Root="HKCU" Key="Software\NakornCode\ToastDesk" Name="installed" Type="integer" Value="1" KeyPath="yes" />
      </Component>
    </DirectoryRef>

    <Feature Id="MainFeature" Title="ToastDesk" Level="1">
$componentRefXml
      <ComponentRef Id="ApplicationShortcut" />
    </Feature>
  </Package>
</Wix>
"@

    Set-Content -Path $wxsPath -Value $wxs -Encoding UTF8
    & $wixCommand.Source build $wxsPath -arch x64 -pdbtype none -out $setupMsiPath
}

$resolvedVersion = Get-ProductVersion
$fileVersion = Get-FileVersion $resolvedVersion

$runningApps = Get-Process -Name "ToastDesk" -ErrorAction SilentlyContinue
if ($runningApps) {
    Write-Host "Stopping running ToastDesk process..."
    $runningApps | Stop-Process -Force
}

if (Test-Path $publishDir) {
    Remove-Item -LiteralPath $publishDir -Recurse -Force
}

New-Item -ItemType Directory -Force -Path $publishDir, $releaseDir, $installerDir | Out-Null

Get-ChildItem -Path $releaseDir -File -Filter "ToastDesk-*" -ErrorAction SilentlyContinue | Remove-Item -Force
Get-ChildItem -Path $installerDir -File -Filter "ToastDesk-*" -ErrorAction SilentlyContinue | Remove-Item -Force

dotnet publish $projectPath `
    --configuration $Configuration `
    --runtime $Runtime `
    --self-contained true `
    --output $publishDir `
    -p:PublishSingleFile=true `
    -p:IncludeNativeLibrariesForSelfExtract=true `
    -p:DebugType=None `
    -p:DebugSymbols=false `
    -p:Version=$resolvedVersion `
    -p:FileVersion=$fileVersion `
    -p:AssemblyVersion=$fileVersion `
    -p:InformationalVersion=$resolvedVersion

if (Test-Path $zipPath) {
    Remove-Item -LiteralPath $zipPath -Force
}

Compress-Archive -Path (Join-Path $publishDir "*") -DestinationPath $zipPath

if (-not $SkipInstallers) {
    if (-not $SkipSetupExe) {
        $isccPath = Get-IsccPath
        if (-not $isccPath) {
            throw "Inno Setup compiler was not found. Install Inno Setup 6 or run with -SkipInstallers."
        }

        & $isccPath "/DMyAppVersion=$resolvedVersion" $innoScriptPath

        $builtSetupExe = Get-ChildItem -Path $installerDir -File -Filter "ToastDesk-Setup-*.exe" |
            Sort-Object LastWriteTime -Descending |
            Select-Object -First 1

        if (-not $builtSetupExe) {
            throw "Inno Setup did not create a setup executable."
        }

        Move-Item -LiteralPath $builtSetupExe.FullName -Destination $setupExePath -Force
    }

    if (-not $SkipMsi) {
        New-ToastDeskMsi $resolvedVersion
    }
}

$packages = Get-ChildItem -Path $releaseDir -File | Where-Object { $_.Extension -in ".zip", ".exe", ".msi" } | Sort-Object Name
$hashLines = foreach ($package in $packages) {
    $hash = Get-FileHash -Path $package.FullName -Algorithm SHA256
    "$($hash.Hash.ToLowerInvariant())  $($package.Name)"
}

Set-Content -Path $checksumPath -Value $hashLines -Encoding ASCII

Write-Host "Release assets created:"
Get-ChildItem -Path $releaseDir -File | Sort-Object Name | ForEach-Object {
    Write-Host " - $($_.FullName)"
}
