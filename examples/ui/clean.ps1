<#
.SYNOPSIS
    Deletes 'gen' and 'build' directories from all example subdirectories in examples/ui.

.DESCRIPTION
    Iterates over all immediate subdirectories in examples/ui and recursively removes
    any 'gen' and 'build' folders found within them to clean up generated code and build artifacts.

.EXAMPLE
    .\clean.ps1
    Cleans all gen and build directories.

.EXAMPLE
    .\clean.ps1 -WhatIf
    .\clean.ps1 -DryRun
    Previews which directories would be deleted without actually removing them.
#>

[CmdletBinding(SupportsShouldProcess = $true)]
param(
    [Alias("n")]
    [switch]$DryRun
)

if ($DryRun) {
    $WhatIfPreference = $true
}

$ScriptDir = if ($PSScriptRoot) { $PSScriptRoot } else { (Get-Location).Path }

function Remove-DirectoryRobustly {
    param([string]$Path)

    if (-not (Test-Path -LiteralPath $Path)) {
        return
    }

    # Attempt 1: Standard Remove-Item
    try {
        Remove-Item -LiteralPath $Path -Recurse -Force -ErrorAction Stop
        return
    }
    catch {
        # Attempt 2: Clear read-only attributes and retry
        try {
            Get-ChildItem -LiteralPath $Path -Recurse -Force -ErrorAction SilentlyContinue | ForEach-Object {
                if ($_.Attributes -band [System.IO.FileAttributes]::ReadOnly) {
                    $_.Attributes = [System.IO.FileAttributes]::Normal
                }
            }
            Remove-Item -LiteralPath $Path -Recurse -Force -ErrorAction Stop
            return
        }
        catch {
            # Attempt 3: cmd rd /s /q
            cmd.exe /c "rd /s /q `"$Path`"" 2>&1 | Out-Null
            if (-not (Test-Path -LiteralPath $Path)) {
                return
            }
            throw $_
        }
    }
}

Write-Host "Scanning example directories in '$ScriptDir'..." -ForegroundColor Cyan

$exampleDirs = Get-ChildItem -Path $ScriptDir -Directory
$targetNames = @('gen', 'build')
$foundCount = 0
$deletedCount = 0
$failedCount = 0

foreach ($dir in $exampleDirs) {
    foreach ($target in $targetNames) {
        $targetPath = Join-Path -Path $dir.FullName -ChildPath $target
        if (Test-Path -LiteralPath $targetPath) {
            $foundCount++
            $relativeDir = Join-Path -Path $dir.Name -ChildPath $target
            if ($PSCmdlet.ShouldProcess($targetPath, "Remove-Item -Recurse -Force")) {
                try {
                    Remove-DirectoryRobustly -Path $targetPath
                    Write-Host "  [DELETED] $relativeDir" -ForegroundColor Green
                    $deletedCount++
                }
                catch {
                    Write-Warning "  [LOCKED/FAILED] $relativeDir : $($_.Exception.Message)"
                    $failedCount++
                }
            }
            else {
                # WhatIf / DryRun mode
                $deletedCount++
            }
        }
    }
}

if ($foundCount -eq 0) {
    Write-Host "No 'gen' or 'build' directories found to clean." -ForegroundColor Yellow
}
else {
    if ($WhatIfPreference) {
        Write-Host "Dry run completed: $deletedCount directory/directories would be deleted." -ForegroundColor Cyan
    }
    elseif ($failedCount -gt 0) {
        Write-Host "Completed with warnings: $deletedCount cleaned, $failedCount failed (possibly in use by running processes)." -ForegroundColor Yellow
    }
    else {
        Write-Host "Done! Cleaned $deletedCount directory/directories." -ForegroundColor Green
    }
}
