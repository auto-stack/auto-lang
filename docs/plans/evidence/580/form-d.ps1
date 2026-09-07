# Plan 580 T6 form D: regenerate unified-demo build.ninja under the same
# env class as golden generation (no MSVC dir on PATH), diff vs golden.
$auto = 'D:\autostack\.wt\lang-580\auto-lang\target\debug\auto.exe'
$src  = 'D:\autostack\.wt\lang-580\auto-lang\examples\unified-demo'
$dst  = 'D:\autostack\.wt\lang-580\_probe\unified-regen'
$golden = 'D:\autostack\auto-lang\examples\unified-demo\build\build.ninja'

if (Test-Path $dst) { Remove-Item -Recurse -Force $dst }
New-Item -ItemType Directory -Path $dst | Out-Null
Copy-Item "$src\pac.at" $dst
Copy-Item -Recurse "$src\front" $dst

# native pipeline trigger (the only path that writes build/build.ninja;
# golden itself was produced through the native generator)
(Get-Content "$dst\pac.at") -replace 'scene: "ui"', 'scene: "c"' | Set-Content "$dst\pac.at"

# golden baseline
New-Item -ItemType Directory -Path "$dst\build" | Out-Null
Copy-Item $golden "$dst\build\build.ninja"
Copy-Item $golden "D:\autostack\.wt\lang-580\_probe\golden-build.ninja"

Write-Output ("cl visible:  " + ((where.exe cl 2>$null | Select-Object -First 1) -join ''))
Write-Output ("link visible: " + ((where.exe link 2>$null | Select-Object -First 1) -join ''))

Set-Location $dst
& $auto build 2>&1 | Out-File -FilePath "D:\autostack\.wt\lang-580\_probe\form-D.log" -Encoding utf8
Write-Output "AUTO_EXIT=$LASTEXITCODE"
