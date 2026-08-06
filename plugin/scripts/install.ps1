# Build release plugin and install into Stream Deck Plugins folder.
# Restarts Stream Deck so it picks up the new plugin.

[CmdletBinding()]
param(
  [switch]$NoRestart,
  [switch]$SkipBuild
)

$ErrorActionPreference = 'Stop'
$pluginRoot = Resolve-Path (Join-Path $PSScriptRoot '..')
$sdPluginName = 'red.eminence.dygma.battery.sdPlugin'
$srcPlugin = Join-Path $pluginRoot $sdPluginName
$destRoot = Join-Path $env:APPDATA 'Elgato\StreamDeck\Plugins'
$dest = Join-Path $destRoot $sdPluginName

function Find-VcVars64 {
  $vswhere = Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio\Installer\vswhere.exe'
  if (Test-Path $vswhere) {
    $found = & $vswhere -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -find '**/vcvars64.bat' 2>$null
    if ($found) { return ($found | Select-Object -First 1) }
  }
  $candidates = @(
    'C:\Program Files (x86)\Microsoft Visual Studio\18\BuildTools\VC\Auxiliary\Build\vcvars64.bat',
    'C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat',
    'C:\Program Files\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat'
  )
  foreach ($c in $candidates) { if (Test-Path $c) { return $c } }
  return $null
}

if (-not $SkipBuild) {
  Write-Host 'Generating icons...'
  & (Join-Path $PSScriptRoot 'gen-icons.ps1')

  Write-Host 'Building release...'
  $vcvars = Find-VcVars64
  if (-not $vcvars) {
    throw @"
MSVC build tools not found (need link.exe + Windows SDK).
Install Visual Studio Build Tools with the C++ workload, e.g.:

  winget install Microsoft.VisualStudio.BuildTools --override "--wait --passive --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended"

Rust/cargo alone is not enough for native Windows binaries.
"@
  }

  $pluginPath = $pluginRoot.Path
  cmd /c "`"$vcvars`" && cd /d `"$pluginPath`" && cargo build --release"
  if ($LASTEXITCODE -ne 0) { throw "cargo build failed ($LASTEXITCODE)" }
}

$exe = Join-Path $pluginRoot 'target\release\dygma-sd-plugin.exe'
if (-not (Test-Path $exe)) { throw "Missing binary: $exe" }

$binDir = Join-Path $srcPlugin 'bin'
New-Item -ItemType Directory -Force -Path $binDir | Out-Null
Copy-Item -Force $exe (Join-Path $binDir 'dygma-sd-plugin.exe')

if (-not $NoRestart) {
  Write-Host 'Stopping Stream Deck (if running)...'
  Get-Process -Name 'StreamDeck' -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
  Start-Sleep -Seconds 2
}

if (Test-Path $dest) {
  Write-Host "Removing old install: $dest"
  Remove-Item -Recurse -Force $dest
}

Write-Host "Installing to $dest"
New-Item -ItemType Directory -Force -Path $destRoot | Out-Null
Copy-Item -Recurse -Force $srcPlugin $dest

if (-not $NoRestart) {
  $sdExe = @(
    "${env:ProgramFiles}\Elgato\StreamDeck\StreamDeck.exe",
    "${env:ProgramFiles(x86)}\Elgato\StreamDeck\StreamDeck.exe"
  ) | Where-Object { Test-Path $_ } | Select-Object -First 1

  if ($sdExe) {
    Write-Host "Starting Stream Deck: $sdExe"
    Start-Process $sdExe
  } else {
    Write-Warning 'StreamDeck.exe not found; start Stream Deck manually.'
  }
}

Write-Host "Done. Add action from category Dygma Battery -> Dygma Battery."
Write-Host "Close Bazecor while the plugin is reading COM."
