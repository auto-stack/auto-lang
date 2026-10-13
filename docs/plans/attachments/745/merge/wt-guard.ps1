param([Parameter(Mandatory=$true)][string]$Path)
$ErrorActionPreference = 'Stop'
try {
    $resolved = [IO.Path]::GetFullPath($Path)
    if ($resolved.TrimEnd('\','/') -eq [IO.Path]::GetPathRoot($resolved).TrimEnd('\','/')) { throw 'Refuse drive root' }
    $item = Get-Item -LiteralPath $resolved -Force
    if (!$item.PSIsContainer) { throw 'Expected directory' }
    $queue = [Collections.Generic.Stack[string]]::new()
    $queue.Push($resolved)
    $found = [Collections.Generic.List[string]]::new()
    while ($queue.Count) {
        $next = $queue.Pop()
        $node = Get-Item -LiteralPath $next -Force
        if ($node.Attributes -band [IO.FileAttributes]::ReparsePoint) { $found.Add($node.FullName); continue }
        foreach ($child in (Get-ChildItem -LiteralPath $next -Force)) {
            if ($child.Attributes -band [IO.FileAttributes]::ReparsePoint) { $found.Add($child.FullName) }
            elseif ($child.PSIsContainer) { $queue.Push($child.FullName) }
        }
    }
    if ($found.Count) { $found | Write-Output; Write-Output 'BLOCKED: reparse points found'; exit 1 }
    Write-Output "clean: $resolved (no reparse points)"
    exit 0
} catch { Write-Output "BLOCKED: $($_.Exception.Message)"; exit 2 }
