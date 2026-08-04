# Build the Stream Deck plugin and produce installer artifacts under dist/.
#
# Outputs (under repo dist/):
#   com.red.eminence.dygma.battery.streamDeckPlugin  (preferred; via streamdeck CLI)
#   com.red.eminence.dygma.battery.sdPlugin.zip      (fallback zip of the .sdPlugin folder)
#
# Optional env:
#   PLUGIN_VERSION  e.g. 0.1.0 or 0.1.0.0  (default: read from tag GITHUB_REF_NAME or manifest)

[CmdletBinding()]
param(
  [string]$Version = $env:PLUGIN_VERSION
)

$ErrorActionPreference = 'Stop'
$pluginRoot = Resolve-Path (Join-Path $PSScriptRoot '..')
$repoRoot = Resolve-Path (Join-Path $pluginRoot '..')
$sdPluginName = 'com.red.eminence.dygma.battery.sdPlugin'
$sdPluginDir = Join-Path $pluginRoot $sdPluginName
$distDir = Join-Path $repoRoot 'dist'
$uuid = 'com.red.eminence.dygma.battery'

function Find-VcVars64 {
  $vswhere = Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio\Installer\vswhere.exe'
  if (Test-Path $vswhere) {
    $found = & $vswhere -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -find '**/vcvars64.bat' 2>$null
    if ($found) { return ($found | Select-Object -First 1) }
  }
  $candidates = @(
    'C:\Program Files (x86)\Microsoft Visual Studio\18\BuildTools\VC\Auxiliary\Build\vcvars64.bat',
    'C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat',
    'C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Auxiliary\Build\vcvars64.bat',
    'C:\Program Files\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat'
  )
  foreach ($c in $candidates) { if (Test-Path $c) { return $c } }
  return $null
}

function Resolve-PluginVersion([string]$inputVersion) {
  if (-not $inputVersion -and $env:GITHUB_REF_NAME -match '^v') {
    $inputVersion = $env:GITHUB_REF_NAME
  }
  if (-not $inputVersion) {
    $manifest = Get-Content (Join-Path $sdPluginDir 'manifest.json') -Raw | ConvertFrom-Json
    return $manifest.Version
  }
  $v = $inputVersion.TrimStart('v')
  $parts = $v.Split('.')
  while ($parts.Count -lt 4) { $parts += '0' }
  return ($parts[0..3] -join '.')
}

$pluginVersion = Resolve-PluginVersion $Version
Write-Host "Plugin version: $pluginVersion"

# Bump manifest Version for this build (string replace keeps JSON shape intact)
$manifestPath = Join-Path $sdPluginDir 'manifest.json'
$manifestRaw = Get-Content $manifestPath -Raw
if ($manifestRaw -notmatch '"Version"\s*:\s*"[^"]+"') {
  throw 'manifest.json missing Version field'
}
$manifestRaw = [regex]::Replace(
  $manifestRaw,
  '"Version"\s*:\s*"[^"]+"',
  "`"Version`": `"$pluginVersion`"",
  1
)
# Write UTF-8 without BOM (Stream Deck is picky about BOM sometimes)
[System.IO.File]::WriteAllText($manifestPath, $manifestRaw)
Write-Host "Updated manifest Version -> $pluginVersion"

Write-Host 'Generating icons...'
& (Join-Path $PSScriptRoot 'gen-icons.ps1')

Write-Host 'Building release...'
$vcvars = Find-VcVars64
if (-not $vcvars) {
  # GitHub windows-latest usually has VS; still try cargo directly if vswhere fails
  Write-Warning 'vcvars64.bat not found; trying cargo build --release directly'
  Push-Location $pluginRoot
  try {
    cargo build --release
    if ($LASTEXITCODE -ne 0) { throw "cargo build failed ($LASTEXITCODE)" }
  } finally {
    Pop-Location
  }
} else {
  $pluginPath = $pluginRoot.Path
  cmd /c "`"$vcvars`" && cd /d `"$pluginPath`" && cargo build --release"
  if ($LASTEXITCODE -ne 0) { throw "cargo build failed ($LASTEXITCODE)" }
}

$exe = Join-Path $pluginRoot 'target\release\dygma-sd-plugin.exe'
if (-not (Test-Path $exe)) { throw "Missing binary: $exe" }

$binDir = Join-Path $sdPluginDir 'bin'
New-Item -ItemType Directory -Force -Path $binDir | Out-Null
Copy-Item -Force $exe (Join-Path $binDir 'dygma-sd-plugin.exe')
Write-Host "Staged $exe -> bin/dygma-sd-plugin.exe"
Write-Host 'Note: multi-OS release CI also ships bin/dygma-sd-plugin-mac and bin/dygma-sd-plugin-linux.'

# Clean dist
if (Test-Path $distDir) { Remove-Item -Recurse -Force $distDir }
New-Item -ItemType Directory -Force -Path $distDir | Out-Null

# Prefer official packer
$packed = $false
$streamdeck = Get-Command streamdeck -ErrorAction SilentlyContinue
if ($streamdeck) {
  Write-Host 'Validating plugin...'
  & streamdeck validate $sdPluginDir
  if ($LASTEXITCODE -ne 0) {
    Write-Warning "streamdeck validate failed ($LASTEXITCODE); continuing with zip fallback"
  } else {
    Write-Host 'Packing with streamdeck CLI...'
    Push-Location $distDir
    try {
      # pack outputs next to cwd or beside plugin depending on CLI version
      & streamdeck pack $sdPluginDir --output $distDir
      if ($LASTEXITCODE -eq 0) { $packed = $true }
      else { Write-Warning "streamdeck pack failed ($LASTEXITCODE)" }
    } finally {
      Pop-Location
    }
  }
} else {
  Write-Warning 'streamdeck CLI not found; using zip package only'
}

# Always also emit a zip of the .sdPlugin tree (works as portable install / backup)
$zipPath = Join-Path $distDir "$uuid.sdPlugin.zip"
if (Test-Path $zipPath) { Remove-Item -Force $zipPath }
Compress-Archive -Path $sdPluginDir -DestinationPath $zipPath -Force
Write-Host "Wrote $zipPath"

# If streamdeck pack left the artifact elsewhere, collect it
Get-ChildItem -Path $pluginRoot, $distDir, $repoRoot -Filter '*.streamDeckPlugin' -Recurse -ErrorAction SilentlyContinue |
  ForEach-Object {
    $dest = Join-Path $distDir $_.Name
    if ($_.FullName -ne $dest) {
      Copy-Item -Force $_.FullName $dest
      Write-Host "Collected $($_.Name) -> dist/"
    }
    $packed = $true
  }

# Fallback: rename a zip to .streamDeckPlugin (Elgato installer format is zip-based)
$sdp = Join-Path $distDir "$uuid.streamDeckPlugin"
if (-not (Test-Path $sdp)) {
  # Official pack may produce com.red.eminence.dygma.battery.streamDeckPlugin
  $existing = Get-ChildItem $distDir -Filter '*.streamDeckPlugin' -ErrorAction SilentlyContinue | Select-Object -First 1
  if (-not $existing) {
    Write-Host 'Creating .streamDeckPlugin from sdPlugin folder zip...'
    # Structure: zip root must contain the .sdPlugin directory
    $stage = Join-Path $distDir '_stage'
    New-Item -ItemType Directory -Force -Path $stage | Out-Null
    Copy-Item -Recurse -Force $sdPluginDir (Join-Path $stage $sdPluginName)
    $tmpZip = Join-Path $distDir '_plugin.zip'
    if (Test-Path $tmpZip) { Remove-Item -Force $tmpZip }
    Compress-Archive -Path (Join-Path $stage $sdPluginName) -DestinationPath $tmpZip -Force
    Move-Item -Force $tmpZip $sdp
    Remove-Item -Recurse -Force $stage
    Write-Host "Wrote $sdp"
  }
}

Write-Host 'Artifacts:'
Get-ChildItem $distDir | ForEach-Object { Write-Host "  $($_.Name)  ($([math]::Round($_.Length/1KB,1)) KB)" }
Write-Host 'Done.'
