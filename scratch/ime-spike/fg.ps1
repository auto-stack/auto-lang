Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
using System.Text;
public class Win32 {
  [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
  [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr hWnd);
  [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr hWnd, int nCmdShow);
  [DllImport("user32.dll")] public static extern bool IsIconic(IntPtr hWnd);
  [DllImport("user32.dll")] public static extern int GetWindowText(IntPtr hWnd, StringBuilder sb, int max);
  [DllImport("user32.dll")] public static extern void keybd_event(byte bVk, byte bScan, uint dwFlags, UIntPtr dwExtraInfo);
}
'@
$spike = (Get-Process ime-spike -ErrorAction Stop).MainWindowHandle
# Alt 键解锁前台锁，再置前
[Win32]::keybd_event(0x12, 0, 0, [UIntPtr]::Zero)
[Win32]::keybd_event(0x12, 0, 2, [UIntPtr]::Zero)
if ([Win32]::IsIconic($spike)) { [Win32]::ShowWindow($spike, 9) | Out-Null }
[Win32]::SetForegroundWindow($spike) | Out-Null
Start-Sleep -Milliseconds 400
$fg = [Win32]::GetForegroundWindow()
$sb = New-Object System.Text.StringBuilder 256
[Win32]::GetWindowText($fg, $sb, 256) | Out-Null
Write-Output ("foreground='" + $sb.ToString() + "' spike_handle=" + $spike + " match=" + ($fg -eq $spike))
