# Windows build set for EnZIME Reader: standalone .exe, NSIS installer, MSI and MSIX,
# all in one run and all bearing one new version. Same scheme as build_all.sh
# (BUILD_CONVENTIONS): MAJOR from version.txt, MINOR bumped on every invocation,
# BUILD = epoch-minutes % 100000 zero-padded to five digits. No two builds share a version.
# Run from a Windows boot on an up-to-date, clean checkout:
#     powershell -ExecutionPolicy Bypass -File build_all_windows.ps1
# Optional signing: ENZIME_WINDOWS_CERTIFICATE_THUMBPRINT + ENZIME_WINDOWS_TIMESTAMP_URL.
# MSIX identity: ENZIME_MS_STORE_PUBLISHER (required, e.g. "CN=..."),
#                ENZIME_MS_STORE_IDENTITY_NAME (default: the Tauri identifier).
[CmdletBinding()]
param()

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
if ([Environment]::OSVersion.Platform -ne [PlatformID]::Win32NT) {
    throw 'Native Windows is required; on Linux run build_all.sh.'
}
$Root = $PSScriptRoot
Set-Location $Root

function Read-Json([string]$Path) {
    try { return Get-Content -LiteralPath $Path -Raw | ConvertFrom-Json }
    catch { throw ('Missing or invalid JSON: ' + $Path) }
}
function Write-Json([string]$Path, $Value) {
    [IO.File]::WriteAllText($Path, (($Value | ConvertTo-Json -Depth 8) + [Environment]::NewLine), [Text.UTF8Encoding]::new($false))
}
function Invoke-Checked([string]$Tool, [string[]]$ToolArguments) {
    & $Tool @ToolArguments
    if ($LASTEXITCODE -ne 0) { throw ('Tool failed: ' + (Split-Path -Leaf $Tool) + ' ' + ($ToolArguments -join ' ')) }
}
function Find-SdkTool([string]$Name) {
    $Command = Get-Command $Name -ErrorAction SilentlyContinue
    if ($Command) { return $Command.Source }
    $Kits = Join-Path ${env:ProgramFiles(x86)} 'Windows Kits\10\bin'
    if (Test-Path -LiteralPath $Kits) {
        foreach ($Directory in @(Get-ChildItem -LiteralPath $Kits -Directory | Sort-Object Name -Descending)) {
            $Candidate = Join-Path $Directory.FullName ('x64\' + $Name)
            if (Test-Path -LiteralPath $Candidate) { return $Candidate }
        }
    }
    throw ('Install the Windows SDK tool: ' + $Name)
}
function Get-Record([string]$Id, [string]$Path, [bool]$Signed) {
    $Item = Get-Item -LiteralPath $Path
    return @{ id = $Id; filename = $Item.Name; size_bytes = $Item.Length
        sha256 = (Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash.ToLowerInvariant()
        signing = $(if ($Signed) { 'verified' } else { 'unsigned' }) }
}

function Set-FirstMatch([string]$Path, [string]$Pattern, [string]$Replacement) {
    $Text = [IO.File]::ReadAllText($Path)
    $Regex = New-Object Text.RegularExpressions.Regex($Pattern, [Text.RegularExpressions.RegexOptions]::Multiline)
    if (-not $Regex.IsMatch($Text)) { throw ('No version field found in ' + $Path) }
    [IO.File]::WriteAllText($Path, $Regex.Replace($Text, $Replacement, 1), [Text.UTF8Encoding]::new($false))
}

# --- new version, stamped before anything is built -----------------------------------
$Dirty = (& git status --porcelain -uno | Out-String).Trim()
if ($LASTEXITCODE -ne 0 -or $Dirty) { throw 'Tracked files differ from HEAD; commit or discard them first.' }
$SourceHead = (& git rev-parse HEAD | Out-String).Trim()
$Current = (Get-Content -LiteralPath (Join-Path $Root 'version.txt') -Raw).Trim()
if ($Current -notmatch '^([0-9]+)\.([0-9]+)\.([0-9]+)$') { throw 'Invalid version in version.txt.' }
$Major = [uint32]$Matches[1]
$Minor = [uint32]$Matches[2] + 1
$BuildNumber = [int]([DateTimeOffset]::UtcNow.ToUnixTimeSeconds() / 60) % 100000
$Version = '{0}.{1}.{2:D5}' -f $Major, $Minor, $BuildNumber
# Cargo, npm and Tauri parse SemVer, which forbids a leading zero; they take BUILD unpadded.
$Semver = '{0}.{1}.{2}' -f $Major, $Minor, $BuildNumber
$VersionCode = $Major * 100000 + $Minor
if ($Major -eq 0 -or $Major -gt 255 -or $Minor -gt 255) {
    throw 'Windows installer fields allow MAJOR 1..255 and MINOR 0..255; the version is not truncated.'
}
# Windows Installer and MSIX cap their internal version fields at 255.255.65535, so those
# two fields carry MAJOR.MINOR; filenames and the app itself carry the full version.
$MsiVersion = '{0}.{1}.0' -f $Major, $Minor
$MsixVersion = '{0}.{1}.0.0' -f $Major, $Minor
Write-Host ('Stamping version: ' + $Version + '  (versionCode: ' + $VersionCode + ')')
$Old = Read-Json (Join-Path $Root 'version.json')
[IO.File]::WriteAllText((Join-Path $Root 'version.txt'), $Version + "`n", [Text.UTF8Encoding]::new($false))
Write-Json (Join-Path $Root 'version.json') ([ordered]@{ version = $Version; semver = $Semver
    versionBase = ('{0}.{1}' -f $Major, $Minor); buildNumber = ('{0:D5}' -f $BuildNumber); versionCode = $VersionCode
    buildDate = (Get-Date).ToString('yyyy-MM-ddTHH:mm:sszzz'); productName = $Old.productName
    internalName = $Old.internalName; packageName = $Old.packageName })
[IO.File]::WriteAllText((Join-Path $Root 'release.lock'), ("MAJOR=$Major`nMINOR=$Minor`nBUILD_NUM=$BuildNumber`n"), [Text.UTF8Encoding]::new($false))
foreach ($Json in @('package.json', 'apps\reader\package.json', 'apps\reader\src-tauri\tauri.conf.json')) {
    Set-FirstMatch (Join-Path $Root $Json) '"version":\s*"[^"]*"' ('"version": "' + $Semver + '"')
}
foreach ($Toml in @('Cargo.toml', 'apps\reader\src-tauri\Cargo.toml')) {
    $TomlPath = Join-Path $Root $Toml
    if ([IO.File]::ReadAllText($TomlPath) -match '(?m)^version *= *"[^"]*"') {
        Set-FirstMatch $TomlPath '^version *= *"[^"]*"' ('version = "' + $Semver + '"')
    }
}
[IO.File]::WriteAllText((Join-Path $Root 'apps\reader\src-tauri\gen\android\tauri.properties'),
    ("tauri.android.versionCode=$VersionCode`ntauri.android.versionName=$Version`n"), [Text.UTF8Encoding]::new($false))
Invoke-Checked 'cargo' @('metadata', '--format-version', '1', '--quiet') | Out-Null   # refreshes Cargo.lock for the stamp
$TauriConfig = Read-Json (Join-Path $Root 'apps\reader\src-tauri\tauri.conf.json')
if ($TauriConfig.version -cne $Semver) { throw 'tauri.conf.json was not stamped.' }
$Prefix = 'enzime-reader-v' + $Version
$Dist = Join-Path $Root 'dist'
$ExeOut = Join-Path $Dist ($Prefix + '-windows-x64.exe')
$NsisOut = Join-Path $Dist ($Prefix + '-windows-x64-setup.exe')
$MsiOut = Join-Path $Dist ($Prefix + '-windows-x64.msi')
$MsixOut = Join-Path $Dist ($Prefix + '-windows-x64.msix')
$ReceiptOut = Join-Path $Dist ($Prefix + '-windows-build-manifest.json')

# --- signing (used when configured) --------------------------------------------------
$Thumbprint = $env:ENZIME_WINDOWS_CERTIFICATE_THUMBPRINT
$TimestampUrl = $env:ENZIME_WINDOWS_TIMESTAMP_URL
$Signing = -not [string]::IsNullOrWhiteSpace($Thumbprint)
$Certificate = $null
if ($Signing) {
    if ([string]::IsNullOrWhiteSpace($TimestampUrl)) { throw 'ENZIME_WINDOWS_TIMESTAMP_URL is required with a certificate.' }
    $Certificate = Get-Item -LiteralPath ('Cert:\CurrentUser\My\' + $Thumbprint)
    $SignTool = Find-SdkTool 'signtool.exe'
}
$Publisher = $env:ENZIME_MS_STORE_PUBLISHER
if ([string]::IsNullOrWhiteSpace($Publisher) -and $Certificate) { $Publisher = $Certificate.Subject }
if ([string]::IsNullOrWhiteSpace($Publisher)) { throw 'Set ENZIME_MS_STORE_PUBLISHER (the Partner Center publisher, "CN=...").' }
$IdentityName = $env:ENZIME_MS_STORE_IDENTITY_NAME
if ([string]::IsNullOrWhiteSpace($IdentityName)) { $IdentityName = [string]$TauriConfig.identifier }

# --- standalone .exe, NSIS installer and MSI in one Tauri build ----------------------
$Attempt = (Get-Date).ToUniversalTime().ToString('yyyyMMddTHHmmssfffZ')
$Target = [IO.Path]::GetFullPath((Join-Path $Root 'target\windows-native'))
$AttemptDir = Join-Path $Target ($Prefix + '-' + $Attempt)
New-Item -ItemType Directory -Path $AttemptDir -Force | Out-Null
$env:CARGO_TARGET_DIR = $Target
$Windows = @{ wix = @{ version = $MsiVersion } }
if ($Signing) {
    $Windows.certificateThumbprint = $Thumbprint
    $Windows.timestampUrl = $TimestampUrl
    $Windows.tsp = $true
}
$ConfigPath = Join-Path $AttemptDir 'windows-config.json'
Write-Json $ConfigPath @{ bundle = @{ targets = @('nsis', 'msi'); windows = $Windows } }
Invoke-Checked 'pnpm' @('install', '--frozen-lockfile')
$BuildStarted = [DateTime]::UtcNow
Push-Location (Join-Path $Root 'apps\reader')
try {
    Invoke-Checked 'cargo' @('tauri', 'build', '--target', 'x86_64-pc-windows-msvc', '--bundles', 'nsis,msi', '--config', $ConfigPath, '--ci', '--', '--locked')
} finally { Pop-Location }
$Release = Join-Path $Target 'x86_64-pc-windows-msvc\release'
$Exes = @(Get-ChildItem -LiteralPath $Release -Filter '*.exe' | Where-Object { $_.LastWriteTimeUtc -ge $BuildStarted })
if ($Exes.Count -ne 1) { throw 'Expected exactly one release executable from this run.' }
$ExePath = $Exes[0].FullName
$Msis = @(Get-ChildItem -LiteralPath (Join-Path $Release 'bundle\msi') -Filter '*.msi' | Where-Object { $_.LastWriteTimeUtc -ge $BuildStarted })
if ($Msis.Count -ne 1) { throw 'Expected exactly one MSI from this run.' }
$Setups = @(Get-ChildItem -LiteralPath (Join-Path $Release 'bundle\nsis') -Filter '*.exe' | Where-Object { $_.LastWriteTimeUtc -ge $BuildStarted })
if ($Setups.Count -ne 1) { throw 'Expected exactly one NSIS installer from this run.' }

# --- MSIX ----------------------------------------------------------------------------
$Stage = Join-Path $AttemptDir 'msix'
New-Item -ItemType Directory -Path (Join-Path $Stage 'Assets') -Force | Out-Null
Copy-Item -LiteralPath $ExePath -Destination $Stage
Copy-Item -LiteralPath (Join-Path $Root 'version.json') -Destination $Stage
foreach ($Logo in @('StoreLogo', 'Square44x44Logo', 'Square150x150Logo', 'Square310x310Logo', 'Square71x71Logo')) {
    Copy-Item -LiteralPath (Join-Path $Root ('apps\reader\src-tauri\icons\' + $Logo + '.png')) -Destination (Join-Path $Stage 'Assets')
}
$Product = [Security.SecurityElement]::Escape([string]$TauriConfig.productName)
$PublisherName = [Security.SecurityElement]::Escape([string]$TauriConfig.bundle.publisher)
$AppxManifest = @"
<?xml version="1.0" encoding="utf-8"?>
<Package xmlns="http://schemas.microsoft.com/appx/manifest/foundation/windows10"
         xmlns:uap="http://schemas.microsoft.com/appx/manifest/uap/windows10"
         xmlns:rescap="http://schemas.microsoft.com/appx/manifest/foundation/windows10/restrictedcapabilities"
         IgnorableNamespaces="uap rescap">
  <Identity Name="$([Security.SecurityElement]::Escape($IdentityName))" Publisher="$([Security.SecurityElement]::Escape($Publisher))" Version="$MsixVersion" ProcessorArchitecture="x64" />
  <Properties>
    <DisplayName>$Product</DisplayName>
    <PublisherDisplayName>$PublisherName</PublisherDisplayName>
    <Logo>Assets\StoreLogo.png</Logo>
  </Properties>
  <Dependencies>
    <TargetDeviceFamily Name="Windows.Desktop" MinVersion="10.0.17763.0" MaxVersionTested="10.0.22621.0" />
  </Dependencies>
  <Resources><Resource Language="en-us" /></Resources>
  <Applications>
    <Application Id="EnZIMEReader" Executable="$($Exes[0].Name)" EntryPoint="Windows.FullTrustApplication">
      <uap:VisualElements DisplayName="$Product" Description="$Product $Version" BackgroundColor="transparent"
                          Square150x150Logo="Assets\Square150x150Logo.png" Square44x44Logo="Assets\Square44x44Logo.png">
        <uap:DefaultTile Square310x310Logo="Assets\Square310x310Logo.png" Square71x71Logo="Assets\Square71x71Logo.png" />
      </uap:VisualElements>
    </Application>
  </Applications>
  <Capabilities>
    <Capability Name="internetClient" />
    <rescap:Capability Name="runFullTrust" />
  </Capabilities>
</Package>
"@
[IO.File]::WriteAllText((Join-Path $Stage 'AppxManifest.xml'), $AppxManifest, [Text.UTF8Encoding]::new($false))
$MakeAppx = Find-SdkTool 'makeappx.exe'
$Package = Join-Path $AttemptDir ($Prefix + '-windows-x64.msix')
Invoke-Checked $MakeAppx @('pack', '/d', $Stage, '/p', $Package)
if ($Signing) {
    Invoke-Checked $SignTool @('sign', '/sha1', $Thumbprint, '/fd', 'SHA256', '/tr', $TimestampUrl, '/td', 'SHA256', $Package)
    foreach ($Signed in @($ExePath, $Setups[0].FullName, $Msis[0].FullName, $Package)) { Invoke-Checked $SignTool @('verify', '/pa', $Signed) }
}

# --- publish into dist with the versioned names, then commit and push ----------------
Copy-Item -LiteralPath $ExePath -Destination $ExeOut
Copy-Item -LiteralPath $Setups[0].FullName -Destination $NsisOut
Copy-Item -LiteralPath $Msis[0].FullName -Destination $MsiOut
Copy-Item -LiteralPath $Package -Destination $MsixOut
$Receipt = @{ schema = 'mba.robin.enzime.windows-build-set.v1'; version = $Version; version_code = $VersionCode
    source_commit = $SourceHead; msi_version = $MsiVersion; msix_version = $MsixVersion
    built_at = (Get-Date).ToUniversalTime().ToString('o'); host = $env:COMPUTERNAME
    artifacts = @((Get-Record 'windows-exe' $ExeOut $Signing), (Get-Record 'windows-nsis' $NsisOut $Signing),
                  (Get-Record 'windows-msi' $MsiOut $Signing), (Get-Record 'windows-msix' $MsixOut $Signing)) }
Write-Json $ReceiptOut $Receipt
Write-Json (Join-Path $Dist 'current-windows-build-manifest.json') $Receipt
Invoke-Checked 'git' @('add', '-u')
Invoke-Checked 'git' @('add', '--', $ExeOut, $NsisOut, $MsiOut, $MsixOut, $ReceiptOut, (Join-Path $Dist 'current-windows-build-manifest.json'))
Invoke-Checked 'git' @('commit', '-m', ('v' + $Version + ': Windows build set (exe, NSIS, MSI, MSIX)'))
Invoke-Checked 'git' @('push', 'origin', 'HEAD')
Write-Host ('Built, committed and pushed the Windows set v' + $Version + $(if ($Signing) { ' (signed).' } else { ' (unsigned: no certificate configured).' }))
