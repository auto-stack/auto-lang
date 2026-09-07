# Plan 580 T6 integration matrix driver.
# Usage: powershell -File form.ps1 <A|B|C>
#   A: PATH has ninja (normal) -> ninja host, build succeeds
#   B: PATH stripped of ninja dir (n2 in ~\.cargo\bin) -> n2 host, same artifacts
#   C: broken util.c -> build runner fails, exit code 1 propagates as Err
param([Parameter(Mandatory=$true)][ValidateSet('A','B','C')][string]$Form)

$msvc = 'C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Tools\MSVC\14.43.34808'
$sdk  = 'C:\Program Files (x86)\Windows Kits\10'
$auto = 'D:\autostack\.wt\lang-580\auto-lang\target\debug\auto.exe'
$tpl  = 'D:\autostack\.wt\lang-580\_probe\t6app-template'

# MSVC toolchain env (cl/link/lib/ml64 all in the same Hostx64\x64 dir)
$env:PATH = "$msvc\bin\Hostx64\x64;" + $env:PATH
$env:INCLUDE = "$msvc\include;$sdk\Include\10.0.26100.0\ucrt;$sdk\Include\10.0.26100.0\um;$sdk\Include\10.0.26100.0\shared"
$env:LIB = "$msvc\lib\x64;$sdk\Lib\10.0.26100.0\ucrt\x64;$sdk\Lib\10.0.26100.0\um\x64"

if ($Form -eq 'B') {
    $env:PATH = ($env:PATH -split ';' | Where-Object { $_ -and $_.TrimEnd('\') -ne 'D:\soft\bin' }) -join ';'
}

$proj = "D:\autostack\.wt\lang-580\_probe\t6app-$Form"
if (Test-Path $proj) { Remove-Item -Recurse -Force $proj }
Copy-Item -Recurse $tpl $proj

if ($Form -eq 'C') {
    Set-Content -Path "$proj\src\util.c" -Value 'int add(int a, int b) { return a + ; }'
}

Write-Output ("=== host visibility ===")
Write-Output ("ninja: " + ((where.exe ninja 2>$null | Select-Object -First 1) -join ''))
Write-Output ("n2:    " + ((where.exe n2 2>$null | Select-Object -First 1) -join ''))
Write-Output ("cl:    " + ((where.exe cl 2>$null | Select-Object -First 1) -join ''))

Set-Location $proj
& $auto build 2>&1 | Tee-Object -FilePath "D:\autostack\.wt\lang-580\_probe\form-$Form.log"
$code = $LASTEXITCODE
Write-Output "AUTO_EXIT=$code"
exit $code
