# Plan 509 evidence helper: list visible top-level window titles (and hwnd),
# optionally filtered by substring. Usage:
#   powershell -ExecutionPolicy Bypass -File list_windows.ps1 [-Match Smithay]
param([string]$Match = "")

Add-Type @"
using System;
using System.Text;
using System.Collections.Generic;
using System.Runtime.InteropServices;
public class WinEnum {
    public delegate bool EnumCb(IntPtr hWnd, IntPtr lParam);
    [DllImport("user32.dll")] public static extern bool EnumWindows(EnumCb cb, IntPtr p);
    [DllImport("user32.dll")] public static extern int GetWindowText(IntPtr h, StringBuilder s, int n);
    [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr h);
    public static List<string> Titles = new List<string>();
    public static bool Cb(IntPtr h, IntPtr p) {
        var sb = new StringBuilder(256);
        GetWindowText(h, sb, 256);
        string t = sb.ToString();
        if (t.Length > 0 && IsWindowVisible(h)) Titles.Add(h.ToInt64().ToString() + "|" + t);
        return true;
    }
    public static List<string> Get() {
        Titles.Clear();
        EnumWindows(Cb, IntPtr.Zero);
        return Titles;
    }
}
"@
foreach ($line in [WinEnum]::Get()) {
    if ($Match -eq "" -or $line -like "*$Match*") { Write-Output $line }
}
