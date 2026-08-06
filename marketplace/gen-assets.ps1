# Generate Elgato Marketplace media (app icon, thumbnail, gallery).
# Requires: plugin/assets/dygma-logo.png
# Output: marketplace/*.png

$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Drawing

$root = Resolve-Path (Join-Path $PSScriptRoot '..')
$out = Join-Path $root 'marketplace'
New-Item -ItemType Directory -Force -Path $out | Out-Null
$logoPath = Join-Path $root 'plugin\assets\dygma-logo.png'
if (-not (Test-Path $logoPath)) { throw "Missing logo: $logoPath" }
$logo = [System.Drawing.Image]::FromFile($logoPath)

function Save-Png([System.Drawing.Bitmap]$bmp, [string]$path) {
  $bmp.Save($path, [System.Drawing.Imaging.ImageFormat]::Png)
  Write-Host "Wrote $path ($($bmp.Width)x$($bmp.Height))"
}

function New-RoundedRectPath([float]$x, [float]$y, [float]$w, [float]$h, [float]$r) {
  $path = New-Object System.Drawing.Drawing2D.GraphicsPath
  $d = [math]::Max(1, $r * 2)
  $path.AddArc($x, $y, $d, $d, 180, 90)
  $path.AddArc($x + $w - $d, $y, $d, $d, 270, 90)
  $path.AddArc($x + $w - $d, $y + $h - $d, $d, $d, 0, 90)
  $path.AddArc($x, $y + $h - $d, $d, $d, 90, 90)
  $path.CloseFigure()
  return $path
}

function Get-Blocks([int]$pct) {
  if ($pct -le 0) { return 0 }
  if ($pct -le 20) { return 1 }
  if ($pct -le 40) { return 2 }
  if ($pct -le 60) { return 3 }
  if ($pct -le 80) { return 4 }
  return 5
}

function Get-BlockColor([int]$b) {
  switch ($b) {
    5 { [System.Drawing.Color]::FromArgb(255, 34, 197, 94) }
    4 { [System.Drawing.Color]::FromArgb(255, 163, 230, 53) }
    3 { [System.Drawing.Color]::FromArgb(255, 234, 179, 8) }
    2 { [System.Drawing.Color]::FromArgb(255, 249, 115, 22) }
    1 { [System.Drawing.Color]::FromArgb(255, 239, 68, 68) }
    default { [System.Drawing.Color]::FromArgb(255, 63, 63, 70) }
  }
}

function Draw-KeyArt(
  [System.Drawing.Graphics]$g,
  [float]$ox, [float]$oy, [float]$scale,
  [int]$leftPct, [int]$rightPct,
  [bool]$leftChg, [bool]$rightChg, [bool]$showPct
) {
  $VIEW = 72.0
  $state = $g.Save()
  $g.TranslateTransform($ox, $oy)
  $g.ScaleTransform($scale, $scale)
  $bg = New-Object System.Drawing.SolidBrush ([System.Drawing.Color]::FromArgb(255, 20, 20, 24))
  $g.FillPath($bg, (New-RoundedRectPath 0 0 72 72 8))
  $bg.Dispose()

  $padX = 6.0; $BLOCKS = 5
  $padTop = if ($leftChg -or $rightChg) { 14.0 } else { 8.0 }
  $padBottom = if ($showPct) { 16.0 } else { 8.0 }
  $gap = 10.0
  $colW = ($VIEW - $padX * 2 - $gap) / 2
  $stackH = $VIEW - $padTop - $padBottom
  $blockGap = 2.0
  $blockH = ($stackH - $blockGap * ($BLOCKS - 1)) / $BLOCKS
  $empty = [System.Drawing.Color]::FromArgb(255, 42, 42, 48)

  $cols = @(
    @{ x = $padX; blocks = (Get-Blocks $leftPct); align = 'L'; chg = $leftChg; pct = $leftPct },
    @{ x = $padX + $colW + $gap; blocks = (Get-Blocks $rightPct); align = 'R'; chg = $rightChg; pct = $rightPct }
  )
  foreach ($c in $cols) {
    $fill = Get-BlockColor $c.blocks
    for ($i = 0; $i -lt $BLOCKS; $i++) {
      $fromBottom = $i
      $y = $padTop + ($BLOCKS - 1 - $i) * ($blockH + $blockGap)
      $t = $fromBottom / 4.0
      $w = $colW * (0.42 + (1.0 - 0.42) * $t)
      $x = if ($c.align -eq 'L') { [float]$c.x } else { [float]($c.x + ($colW - $w)) }
      $color = if ($fromBottom -lt $c.blocks) { $fill } else { $empty }
      $br = New-Object System.Drawing.SolidBrush $color
      $g.FillPath($br, (New-RoundedRectPath $x $y $w $blockH 2))
      $br.Dispose()
    }
    if ($c.chg) {
      $cx = $c.x + $colW / 2
      $cy = $padTop - 6.5
      $pts = @(
        (New-Object System.Drawing.PointF ($cx + 1.4), ($cy - 5)),
        (New-Object System.Drawing.PointF ($cx - 2.5), ($cy + 0.4)),
        (New-Object System.Drawing.PointF ($cx - 0.4), ($cy + 0.4)),
        (New-Object System.Drawing.PointF ($cx - 1.8), ($cy + 5)),
        (New-Object System.Drawing.PointF ($cx + 2.5), ($cy - 0.1)),
        (New-Object System.Drawing.PointF ($cx + 0.6), ($cy - 0.1))
      )
      $yb = New-Object System.Drawing.SolidBrush ([System.Drawing.Color]::FromArgb(255, 253, 224, 71))
      $g.FillPolygon($yb, $pts)
      $yb.Dispose()
    }
  }
  if ($showPct) {
    $font = New-Object System.Drawing.Font 'Segoe UI', 9, ([System.Drawing.FontStyle]::Bold)
    $tb = New-Object System.Drawing.SolidBrush ([System.Drawing.Color]::FromArgb(255, 228, 228, 231))
    $sf = New-Object System.Drawing.StringFormat
    $sf.Alignment = [System.Drawing.StringAlignment]::Center
    $g.DrawString("$leftPct%", $font, $tb, ($padX + $colW / 2), ($VIEW - 12), $sf)
    $g.DrawString("$rightPct%", $font, $tb, ($padX + $colW + $gap + $colW / 2), ($VIEW - 12), $sf)
    $font.Dispose(); $tb.Dispose()
  }
  $g.Restore($state)
}

function New-Banner([string]$title, [string]$subtitle, [scriptblock]$extra) {
  $bmp = New-Object System.Drawing.Bitmap 1920, 960
  $g = [System.Drawing.Graphics]::FromImage($bmp)
  $g.SmoothingMode = 'HighQuality'
  $g.InterpolationMode = 'HighQualityBicubic'
  $g.TextRenderingHint = 'ClearTypeGridFit'
  $brush = New-Object System.Drawing.Drawing2D.LinearGradientBrush (
    (New-Object System.Drawing.Point 0, 0),
    (New-Object System.Drawing.Point 1920, 960),
    [System.Drawing.Color]::FromArgb(255, 12, 12, 16),
    [System.Drawing.Color]::FromArgb(255, 28, 20, 36)
  )
  $g.FillRectangle($brush, 0, 0, 1920, 960)
  $brush.Dispose()
  $g.FillRectangle((New-Object System.Drawing.SolidBrush ([System.Drawing.Color]::FromArgb(255, 244, 63, 39))), 0, 0, 12, 960)
  $g.DrawImage($script:logo, 80, 80, 160, 160)
  $titleFont = New-Object System.Drawing.Font 'Segoe UI', 54, ([System.Drawing.FontStyle]::Bold)
  $subFont = New-Object System.Drawing.Font 'Segoe UI', 22
  $white = New-Object System.Drawing.SolidBrush ([System.Drawing.Color]::White)
  $muted = New-Object System.Drawing.SolidBrush ([System.Drawing.Color]::FromArgb(255, 161, 161, 170))
  $g.DrawString($title, $titleFont, $white, 280, 100)
  $g.DrawString($subtitle, $subFont, $muted, 280, 190)
  & $extra $g
  $titleFont.Dispose(); $subFont.Dispose(); $white.Dispose(); $muted.Dispose()
  $g.Dispose()
  return $bmp
}

# App icon 288x288
$app = New-Object System.Drawing.Bitmap 288, 288
$ag = [System.Drawing.Graphics]::FromImage($app)
$ag.SmoothingMode = 'HighQuality'; $ag.InterpolationMode = 'HighQualityBicubic'
$ag.Clear([System.Drawing.Color]::FromArgb(255, 18, 18, 22))
$logoSize = 200
$ag.DrawImage($logo, [int]((288 - $logoSize) / 2), [int]((288 - $logoSize) / 2 - 8), $logoSize, $logoSize)
$ag.Dispose()
Save-Png $app (Join-Path $out 'app-icon-288.png')
$app.Dispose()

$thumb = New-Banner 'Dygma Battery' 'Wireless left / right battery on Stream Deck  ·  Defy verified · Raise 2 / Sonsei beta · macOS beta' {
  param($g)
  Draw-KeyArt $g 280 360 4.2 100 40 $true $true $true
  Draw-KeyArt $g 620 360 4.2 72 55 $false $false $true
  Draw-KeyArt $g 960 360 4.2 18 8 $false $false $true
  Draw-KeyArt $g 1300 360 4.2 90 90 $false $false $false
  $f = New-Object System.Drawing.Font 'Segoe UI', 16
  $m = New-Object System.Drawing.SolidBrush ([System.Drawing.Color]::FromArgb(255, 113, 113, 122))
  $g.DrawString('Charging + %     Mid charge     Low     Bars only', $f, $m, 280, 720)
  $f.Dispose(); $m.Dispose()
  $tag = New-Object System.Drawing.Font 'Segoe UI', 14, ([System.Drawing.FontStyle]::Bold)
  $tb = New-Object System.Drawing.SolidBrush ([System.Drawing.Color]::FromArgb(255, 244, 63, 39))
  $g.DrawString('by Eminence  ·  Unofficial community plugin  ·  Logo used with permission', $tag, $tb, 80, 880)
  $tag.Dispose(); $tb.Dispose()
}
Save-Png $thumb (Join-Path $out 'thumbnail-1920x960.png')
$thumb.Dispose()

$g1 = New-Banner 'Live key art' 'Dual bars · charge colors · optional % · charging bolts · Dygma mark' {
  param($g)
  Draw-KeyArt $g 520 320 6.5 100 40 $true $true $true
  Draw-KeyArt $g 1100 320 6.5 55 75 $false $true $true
}
Save-Png $g1 (Join-Path $out 'gallery-01-key-art.png')
$g1.Dispose()

$g2 = New-Banner 'How it works' 'Neuron USB + RF sides  ·  Focus serial  ·  Close Bazecor while reading' {
  param($g)
  $boxFont = New-Object System.Drawing.Font 'Segoe UI', 20, ([System.Drawing.FontStyle]::Bold)
  $bodyFont = New-Object System.Drawing.Font 'Segoe UI', 16
  $white = New-Object System.Drawing.SolidBrush ([System.Drawing.Color]::White)
  $muted = New-Object System.Drawing.SolidBrush ([System.Drawing.Color]::FromArgb(255, 180, 180, 190))
  $card = New-Object System.Drawing.SolidBrush ([System.Drawing.Color]::FromArgb(255, 28, 28, 36))
  $items = @(
    @{ t = '1. Neuron on USB'; d = 'Focus serial (not pure Bluetooth mode)' },
    @{ t = '2. Halves on RF'; d = 'Wireless battery fuel-gauge over RF to Neuron' },
    @{ t = '3. Close Bazecor'; d = 'One process owns the serial port at a time' },
    @{ t = '4. Drop on a key'; d = 'Auto-poll; press key to force refresh' }
  )
  $x = 120; $y = 340
  foreach ($it in $items) {
    $g.FillPath($card, (New-RoundedRectPath $x $y 400 200 16))
    $g.DrawString($it.t, $boxFont, $white, ($x + 24), ($y + 40))
    $g.DrawString($it.d, $bodyFont, $muted, (New-Object System.Drawing.RectangleF ($x + 24), ($y + 90), 350, 80))
    $x += 440
  }
  $boxFont.Dispose(); $bodyFont.Dispose(); $white.Dispose(); $muted.Dispose(); $card.Dispose()
}
Save-Png $g2 (Join-Path $out 'gallery-02-setup.png')
$g2.Dispose()

$g3 = New-Banner 'Supported boards' 'Any wireless Dygma with Focus wireless.battery.* over Neuron USB' {
  param($g)
  $card = New-Object System.Drawing.SolidBrush ([System.Drawing.Color]::FromArgb(255, 28, 28, 36))
  $titleF = New-Object System.Drawing.Font 'Segoe UI', 28, ([System.Drawing.FontStyle]::Bold)
  $bodyF = New-Object System.Drawing.Font 'Segoe UI', 16
  $white = New-Object System.Drawing.SolidBrush ([System.Drawing.Color]::White)
  $muted = New-Object System.Drawing.SolidBrush ([System.Drawing.Color]::FromArgb(255, 161, 161, 170))
  $boards = @(
    @{ n = 'Defy'; d = 'Columnar wireless'; s = 'Verified' },
    @{ n = 'Raise 2'; d = 'Row-staggered wireless'; s = 'Beta' },
    @{ n = 'Sonsei'; d = 'Low-profile wireless'; s = 'Beta' }
  )
  $x = 200
  foreach ($b in $boards) {
    $g.FillPath($card, (New-RoundedRectPath $x 360 480 280 20))
    $g.DrawImage($script:logo, ($x + 40), 400, 90, 90)
    $g.DrawString($b.n, $titleF, $white, ($x + 160), 420)
    $g.DrawString($b.d, $bodyF, $muted, ($x + 160), 480)
    $g.DrawString("$($b.s)  ·  Windows primary  ·  macOS beta", $bodyF, $muted, ($x + 160), 540)
    $x += 520
  }
  $card.Dispose(); $titleF.Dispose(); $bodyF.Dispose(); $white.Dispose(); $muted.Dispose()
}
Save-Png $g3 (Join-Path $out 'gallery-03-boards.png')
$g3.Dispose()

$logo.Dispose()
Write-Host 'Marketplace assets ready in marketplace/'
