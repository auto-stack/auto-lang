$ErrorActionPreference='Stop'
$reviewDir='D:/autostack/auto-lang/docs/plans/attachments/745/review-r2'
$worktree='D:/autostack/.wt/lang-745/auto-lang'
$helperPath=Join-Path $worktree 'experimental/ac-core/tests/common/mod.rs'
$fullHelper=[IO.File]::ReadAllText($helperPath)
$start=$fullHelper.IndexOf('/// Outcome of a deadline-guarded process run')
if($start -lt 0){throw 'Exact helper seam not found'}
$helperSource=$fullHelper.Substring($start)
$mainSource=[IO.File]::ReadAllText((Join-Path $reviewDir 'helper-probe-main.rs.txt'))
$source=$helperSource+"`n"+$mainSource
[IO.File]::WriteAllText((Join-Path $reviewDir 'helper-probe-exact.rs.txt'),$source,[Text.UTF8Encoding]::new($false))
$buildDir=Join-Path $worktree 'experimental/ac-core/target/review-r2'
New-Item -ItemType Directory -Force -Path $buildDir|Out-Null
$rs=Join-Path $buildDir 'helper_probe.rs';$exe=Join-Path $buildDir 'helper_probe.exe'
[IO.File]::WriteAllText($rs,$source,[Text.UTF8Encoding]::new($false))
rustc --edition=2021 --crate-name ac745_review_helper $rs -o $exe
if($LASTEXITCODE -ne 0){throw 'Helper probe rustc failed'}
$psi=[Diagnostics.ProcessStartInfo]::new();$psi.FileName=$exe;$psi.WorkingDirectory=$worktree;$psi.UseShellExecute=$false;$psi.RedirectStandardOutput=$true;$psi.RedirectStandardError=$true
$p=[Diagnostics.Process]::Start($psi);$out=$p.StandardOutput.ReadToEndAsync();$err=$p.StandardError.ReadToEndAsync()
if(-not $p.WaitForExit(12000)){$p.Kill($true);$p.WaitForExit();throw 'Review harness exceeded outer 12s deadline'}
if(-not $out.Wait(1000) -or -not $err.Wait(1000)){throw 'Review output collection deadline'}
$result=[ordered]@{helper_path=$helperPath;helper_sha256=(Get-FileHash $helperPath -Algorithm SHA256).Hash.ToLowerInvariant();exact_harness_sha256=(Get-FileHash $rs -Algorithm SHA256).Hash.ToLowerInvariant();binary_sha256=(Get-FileHash $exe -Algorithm SHA256).Hash.ToLowerInvariant();exit=$p.ExitCode;stdout=$out.Result;stderr=$err.Result}
$result|ConvertTo-Json -Depth 4|Set-Content (Join-Path $reviewDir 'helper-results.json') -Encoding utf8
$result|ConvertTo-Json -Depth 4|Write-Output
