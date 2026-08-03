# Generate minimal PNG icons for the Stream Deck plugin (no external deps).
# Stream Deck wants extension-less paths in the manifest; files are name.png and name@2x.png.

$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Drawing

$root = Join-Path $PSScriptRoot '..\com.red.eminence.dygma.battery.sdPlugin\imgs'
New-Item -ItemType Directory -Force -Path $root | Out-Null

function New-SolidPng {
  param(
    [string]$Path,
    [int]$Size,
    [System.Drawing.Color]$Bg,
    [System.Drawing.Color]$Fg,
    [string]$Glyph = $null
  )
  $bmp = New-Object System.Drawing.Bitmap $Size, $Size
  $g = [System.Drawing.Graphics]::FromImage($bmp)
  $g.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::AntiAlias
  $g.Clear($Bg)

  if ($Glyph) {
    $fontSize = [math]::Max(8, [int]($Size * 0.45))
    $font = New-Object System.Drawing.Font 'Segoe UI', $fontSize, ([System.Drawing.FontStyle]::Bold)
    $brush = New-Object System.Drawing.SolidBrush $Fg
    $sf = New-Object System.Drawing.StringFormat
    $sf.Alignment = [System.Drawing.StringAlignment]::Center
    $sf.LineAlignment = [System.Drawing.StringAlignment]::Center
    $rect = New-Object System.Drawing.RectangleF 0, 0, $Size, $Size
    $g.DrawString($Glyph, $font, $brush, $rect, $sf)
    $font.Dispose()
    $brush.Dispose()
  } else {
    # battery silhouette
    $pen = New-Object System.Drawing.Pen $Fg, ([math]::Max(1, $Size / 18))
    $brush = New-Object System.Drawing.SolidBrush $Fg
    $m = [int]($Size * 0.18)
    $bodyW = $Size - 2 * $m
    $bodyH = [int]($Size * 0.42)
    $bodyY = [int](($Size - $bodyH) / 2)
    $g.DrawRectangle($pen, $m, $bodyY, $bodyW - [int]($Size * 0.08), $bodyH)
    $tipW = [int]($Size * 0.08)
    $tipH = [int]($bodyH * 0.45)
    $g.FillRectangle($brush, $m + $bodyW - $tipW, $bodyY + [int](($bodyH - $tipH) / 2), $tipW, $tipH)
    # fill level ~60%
    $fill = New-Object System.Drawing.SolidBrush ([System.Drawing.Color]::FromArgb(255, 80, 200, 120))
    $inner = [int]($Size * 0.06)
    $fillW = [int](($bodyW - $tipW - 2 * $inner) * 0.6)
    $g.FillRectangle($fill, $m + $inner, $bodyY + $inner, $fillW, $bodyH - 2 * $inner)
    $pen.Dispose(); $brush.Dispose(); $fill.Dispose()
  }

  $g.Dispose()
  $dir = Split-Path $Path -Parent
  if (-not (Test-Path $dir)) { New-Item -ItemType Directory -Force -Path $dir | Out-Null }
  $bmp.Save($Path, [System.Drawing.Imaging.ImageFormat]::Png)
  $bmp.Dispose()
}

$transparent = [System.Drawing.Color]::FromArgb(0, 0, 0, 0)
$dark = [System.Drawing.Color]::FromArgb(255, 30, 32, 40)
$white = [System.Drawing.Color]::White

# Plugin preference icons (256 / 512)
New-SolidPng (Join-Path $root 'plugin.png') 256 $dark $white
New-SolidPng (Join-Path $root 'plugin@2x.png') 512 $dark $white

# Category list icons (28 / 56) — monochrome white on transparent
New-SolidPng (Join-Path $root 'category.png') 28 $transparent $white
New-SolidPng (Join-Path $root 'category@2x.png') 56 $transparent $white

# Action list icons (20 / 40) — monochrome white on transparent
New-SolidPng (Join-Path $root 'action.png') 20 $transparent $white
New-SolidPng (Join-Path $root 'action@2x.png') 40 $transparent $white

# Key state images (72 / 144)
New-SolidPng (Join-Path $root 'key.png') 72 $dark $white
New-SolidPng (Join-Path $root 'key@2x.png') 144 $dark $white

Write-Host "Icons written to $root"
