param([string]$Keys = "", [int]$GapMs = 90, [string]$Click = "")

Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
public class Win32 {
  [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
  [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr hWnd);
  [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr hWnd, int nCmdShow);
  [DllImport("user32.dll")] public static extern bool IsIconic(IntPtr hWnd);
  [DllImport("user32.dll")] public static extern void keybd_event(byte bVk, byte bScan, uint dwFlags, UIntPtr dwExtraInfo);
  [DllImport("user32.dll")] public static extern bool SetCursorPos(int x, int y);
  [DllImport("user32.dll")] public static extern void mouse_event(uint dwFlags, uint dx, uint dy, uint dwData, UIntPtr dwExtraInfo);
}
'@

$vk = @{
  'a'=0x41;'b'=0x42;'c'=0x43;'d'=0x44;'e'=0x45;'f'=0x46;'g'=0x47;'h'=0x48;'i'=0x49;
  'j'=0x4A;'k'=0x4B;'l'=0x4C;'m'=0x4D;'n'=0x4E;'o'=0x4F;'p'=0x50;'q'=0x51;'r'=0x52;
  's'=0x53;'t'=0x54;'u'=0x55;'v'=0x56;'w'=0x57;'x'=0x58;'y'=0x59;'z'=0x5A;
  '1'=0x31;'2'=0x32;'3'=0x33;'4'=0x34;'5'=0x35;'6'=0x36;'7'=0x37;'8'=0x38;'9'=0x39;'0'=0x30;
  'space'=0x20;'enter'=0x0D;'tab'=0x09;'esc'=0x1B;'shift'=0x10;'ctrl'=0x11;'alt'=0x12;
  'bs'=0x08;'del'=0x2E;
  'f1'=0x70;'f2'=0x71;'f3'=0x72;'f4'=0x73;'f5'=0x74;'f6'=0x75;'f7'=0x76;'f8'=0x77;
  'f9'=0x78;'f10'=0x79;'f11'=0x7A;'f12'=0x7B;
}

$spike = (Get-Process ime-spike -ErrorAction Stop).MainWindowHandle
[Win32]::keybd_event(0x12, 0, 0, [UIntPtr]::Zero)
[Win32]::keybd_event(0x12, 0, 2, [UIntPtr]::Zero)
if ([Win32]::IsIconic($spike)) { [Win32]::ShowWindow($spike, 9) | Out-Null }
[Win32]::SetForegroundWindow($spike) | Out-Null
Start-Sleep -Milliseconds 350
$fg = [Win32]::GetForegroundWindow()
if ($fg -ne $spike) { Write-Output "FOREGROUND_FAIL handle=$fg expect=$spike"; exit 1 }

foreach ($tok in ($Keys -split ',')) {
  $t = $tok.Trim().ToLower()
  if (-not $t) { continue }
  if ($t -eq 'wait') { Start-Sleep -Milliseconds 400; continue }
  if (-not $vk.ContainsKey($t)) { Write-Output "UNKNOWN_KEY:$t"; continue }
  $code = $vk[$t]
  [Win32]::keybd_event([byte]$code, 0, 0, [UIntPtr]::Zero)
  [Win32]::keybd_event([byte]$code, 0, 2, [UIntPtr]::Zero)
  Start-Sleep -Milliseconds $GapMs
}
if ($Click -ne "") {
  $parts = $Click -split ','
  [Win32]::SetCursorPos([int]$parts[0], [int]$parts[1]) | Out-Null
  Start-Sleep -Milliseconds 120
  [Win32]::mouse_event(0x0002, 0, 0, 0, [UIntPtr]::Zero)
  [Win32]::mouse_event(0x0004, 0, 0, 0, [UIntPtr]::Zero)
  Start-Sleep -Milliseconds 300
  Write-Output "CLICKED $Click"
}
Write-Output "OK keys=[$Keys]"
