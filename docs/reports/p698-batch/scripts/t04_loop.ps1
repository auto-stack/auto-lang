# PLAN-698 T-04c live loop: ui_desktop (embedded rqhost daemon) + 003 exe desktop-endpoint adoption.
$ErrorActionPreference = "Continue"
$lang = "D:/autostack/.wt/lang-698/auto-lang"
$save = "D:/autostack/.wt/lang-698/scratch-w1"

# 1. desktop host with embedded daemon (windowed mode to avoid fullscreen hijack)
$deskLog = "$save/t04_desktop.log"
$env:AUTO_DESKTOP_RQHOST = "1"
$desk = Start-Process -FilePath "cargo" -ArgumentList "run","-p","auto-lang","--features","ui-iced","--example","ui_desktop" `
    -WorkingDirectory $lang -RedirectStandardError $deskLog -RedirectStandardOutput "$save/t04_desktop_out.log" `
    -PassThru -WindowStyle Hidden
Write-Output "[host] desktop pid=$($desk.Id)"
$pipe = $null
$deadline = (Get-Date).AddSeconds(120)
while ((Get-Date) -lt $deadline) {
    Start-Sleep -Seconds 2
    $line = Select-String -Path $deskLog -Pattern "rqhost daemon spawned at (\S+)" | Select-Object -Last 1
    if ($line) { $pipe = $line.Matches[0].Groups[1].Value; break }
}
if (-not $pipe) { Write-Output "RESULT: daemon embedding line never appeared"; Get-Content $deskLog -Tail 10; Stop-Process -Id $desk.Id -Force -ErrorAction SilentlyContinue; exit 2 }
Write-Output "[host] endpoint pipe = $pipe"

# wait for the desktop window itself (iced loop up)
Start-Sleep -Seconds 15
$alive = Get-Process -Id $desk.Id -ErrorAction SilentlyContinue
Write-Output "[host] alive after window wait: $($null -ne $alive)"

# 2. launch 003 exe against the desktop endpoint
$convLog = "$save/t04_converter.log"
$conv = Start-Process -FilePath "$lang/target/debug/converter.exe" `
    -ArgumentList "--render-mode","desktop","--desktop-endpoint",$pipe `
    -RedirectStandardError $convLog -RedirectStandardOutput "$save/t04_converter_out.log" `
    -PassThru
Write-Output "[client] converter pid=$($conv.Id)"
Start-Sleep -Seconds 10
$convAlive = Get-Process -Id $conv.Id -ErrorAction SilentlyContinue
Write-Output "[client] alive after 10s: $($null -ne $convAlive)"

# 3. screenshot both windows (screen-level capture as evidence)
Add-Type -AssemblyName System.Windows.Forms,System.Drawing
$bmp = New-Object System.Drawing.Bitmap([System.Windows.Forms.Screen]::PrimaryScreen.Bounds.Width, [System.Windows.Forms.Screen]::PrimaryScreen.Bounds.Height)
$g = [System.Drawing.Graphics]::FromImage($bmp)
$g.CopyFromScreen(0,0,0,0,$bmp.Size)
$bmp.Save("$save/t04_both_windows.png",[System.Drawing.Imaging.ImageFormat]::Png)
$g.Dispose(); $bmp.Dispose()
Write-Output "[evidence] screen capture saved"

# 4. input probe: move cursor across the converter window region (hover events)
Add-Type @"
using System;
using System.Runtime.InteropServices;
public class M {
    [DllImport("user32.dll")] public static extern bool SetCursorPos(int x, int y);
}
"@
$w = [System.Windows.Forms.Screen]::PrimaryScreen.Bounds.Width
$h = [System.Windows.Forms.Screen]::PrimaryScreen.Bounds.Height
foreach ($i in 1..20) { [M]::SetCursorPos([int]($w*0.3 + $i*8), [int]($h*0.4 + ($i % 5)*12)) | Out-Null; Start-Sleep -Milliseconds 60 }
Write-Output "[input] cursor sweep done"

# 5. close the converter (graceful WM_CLOSE via taskkill graceful is not; use window close message)
# use PowerShell: send Alt+F4 to foreground after activating converter window is complex; use taskkill (tree) and observe reclaim
Stop-Process -Id $conv.Id -Force -ErrorAction SilentlyContinue
Start-Sleep -Seconds 6
$convAfter = Get-Process -Id $conv.Id -ErrorAction SilentlyContinue
Write-Output "[reclaim] converter after kill: $($null -ne $convAfter)"

# 6. stop desktop host
Stop-Process -Id $desk.Id -Force -ErrorAction SilentlyContinue
Start-Sleep -Seconds 2
Write-Output "=== desktop log tail ==="
Get-Content $deskLog -Tail 20
Write-Output "=== converter log tail ==="
Get-Content $convLog -Tail 20
