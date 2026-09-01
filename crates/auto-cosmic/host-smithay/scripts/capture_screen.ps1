# Plan 509 T3/T4 evidence: capture the full Windows virtual screen (WSLg
# windows render as native Windows windows) to PNG. Usage:
#   powershell -ExecutionPolicy Bypass -File capture_screen.ps1 -Out <png>
param(
    [Parameter(Mandatory = $true)][string]$Out
)
Add-Type -AssemblyName System.Windows.Forms
Add-Type -AssemblyName System.Drawing
$bounds = [System.Windows.Forms.SystemInformation]::VirtualScreen
$bmp = New-Object System.Drawing.Bitmap($bounds.Width, $bounds.Height)
$gfx = [System.Drawing.Graphics]::FromImage($bmp)
$gfx.CopyFromScreen($bounds.X, $bounds.Y, 0, 0, $bmp.Size)
$dir = Split-Path -Parent $Out
if ($dir -and -not (Test-Path $dir)) { New-Item -ItemType Directory -Force -Path $dir | Out-Null }
$bmp.Save($Out, [System.Drawing.Imaging.ImageFormat]::Png)
$gfx.Dispose(); $bmp.Dispose()
Write-Output "saved $Out ($($bounds.Width)x$($bounds.Height))"
