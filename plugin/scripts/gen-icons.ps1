# Generate Stream Deck plugin icons from the Dygma brand mark.
# Stream Deck wants extension-less paths in the manifest; files are name.png and name@2x.png.
#
# Logo source: plugin/assets/dygma-logo.png (Bazecor public logo.svg / logo.png).

$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Drawing

$scriptDir = $PSScriptRoot
$logoPath = Join-Path $scriptDir '..\assets\dygma-logo.png'
$root = Join-Path $scriptDir '..\com.red.eminence.dygma.battery.sdPlugin\imgs'
New-Item -ItemType Directory -Force -Path $root | Out-Null

if (-not (Test-Path $logoPath)) {
  throw "Missing Dygma logo at $logoPath"
}

$logo = [System.Drawing.Image]::FromFile((Resolve-Path $logoPath))

function Save-Png {
  param([System.Drawing.Bitmap]$Bmp, [string]$Path)
  $dir = Split-Path $Path -Parent
  if (-not (Test-Path $dir)) { New-Item -ItemType Directory -Force -Path $dir | Out-Null }
  $Bmp.Save($Path, [System.Drawing.Imaging.ImageFormat]::Png)
}

function New-LogoIcon {
  param(
    [string]$Path,
    [int]$Size,
    [System.Drawing.Color]$Bg,
    [double]$LogoFrac = 0.72,
    [bool]$MonochromeWhite = $false
  )

  $bmp = New-Object System.Drawing.Bitmap $Size, $Size
  $g = [System.Drawing.Graphics]::FromImage($bmp)
  $g.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::HighQuality
  $g.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
  $g.PixelOffsetMode = [System.Drawing.Drawing2D.PixelOffsetMode]::HighQuality
  $g.CompositingQuality = [System.Drawing.Drawing2D.CompositingQuality]::HighQuality
  $g.Clear($Bg)

  $logoPx = [math]::Max(8, [int]([math]::Round($Size * $LogoFrac)))
  $ox = [int](($Size - $logoPx) / 2)
  $oy = [int](($Size - $logoPx) / 2)

  if ($MonochromeWhite) {
    # Draw logo into a temp bitmap, then lift luminance → white with alpha.
    $tmp = New-Object System.Drawing.Bitmap $logoPx, $logoPx
    $tg = [System.Drawing.Graphics]::FromImage($tmp)
    $tg.SmoothingMode = $g.SmoothingMode
    $tg.InterpolationMode = $g.InterpolationMode
    $tg.Clear([System.Drawing.Color]::Transparent)
    $tg.DrawImage($script:logo, 0, 0, $logoPx, $logoPx)
    $tg.Dispose()

    for ($y = 0; $y -lt $logoPx; $y++) {
      for ($x = 0; $x -lt $logoPx; $x++) {
        $c = $tmp.GetPixel($x, $y)
        if ($c.A -lt 8) { continue }
        # Opaque brand pixels → solid white; soft edges keep alpha
        $a = [byte][math]::Min(255, [int]($c.A * 1.0))
        # Boost low-alpha anti-alias from colored edges
        $lum = (0.299 * $c.R + 0.587 * $c.G + 0.114 * $c.B) / 255.0
        # Center hole is transparent (white bg in source PNG → treat near-white as empty)
        if ($lum -gt 0.92 -and $c.A -gt 200) {
          # hole / background in source art
          continue
        }
        $bmp.SetPixel($ox + $x, $oy + $y, [System.Drawing.Color]::FromArgb($a, 255, 255, 255))
      }
    }
    $tmp.Dispose()
  } else {
    $g.DrawImage($script:logo, $ox, $oy, $logoPx, $logoPx)
  }

  $g.Dispose()
  Save-Png $bmp $Path
  $bmp.Dispose()
}

$transparent = [System.Drawing.Color]::FromArgb(0, 0, 0, 0)
$dark = [System.Drawing.Color]::FromArgb(255, 20, 20, 24)

# Plugin preference icons (256 / 512) — full-color mark on dark
New-LogoIcon (Join-Path $root 'plugin.png') 256 $dark 0.78 $false
New-LogoIcon (Join-Path $root 'plugin@2x.png') 512 $dark 0.78 $false

# Category list icons (28 / 56) — monochrome white on transparent
New-LogoIcon (Join-Path $root 'category.png') 28 $transparent 0.88 $true
New-LogoIcon (Join-Path $root 'category@2x.png') 56 $transparent 0.88 $true

# Action list icons (20 / 40) — monochrome white on transparent
New-LogoIcon (Join-Path $root 'action.png') 20 $transparent 0.90 $true
New-LogoIcon (Join-Path $root 'action@2x.png') 40 $transparent 0.90 $true

# Default key state images (72 / 144) — full-color mark on dark (shown until live SVG)
New-LogoIcon (Join-Path $root 'key.png') 72 $dark 0.70 $false
New-LogoIcon (Join-Path $root 'key@2x.png') 144 $dark 0.70 $false

$logo.Dispose()
Write-Host "Icons written to $root (Dygma logo)"
