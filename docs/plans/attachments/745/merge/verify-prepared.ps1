$ErrorActionPreference = 'Stop'
$repo = 'D:/autostack/.wt/lang-745/auto-lang'
$receipt = Join-Path $repo 'docs/plans/attachments/745/merge'
$psi = [Diagnostics.ProcessStartInfo]::new()
$psi.FileName = 'powershell'
$psi.WorkingDirectory = $repo
$psi.UseShellExecute = $false
$psi.RedirectStandardOutput = $true
$psi.RedirectStandardError = $true
$null = $psi.Environment.Remove('PSModulePath')
foreach ($a in @('-NoProfile','-ExecutionPolicy','Bypass','-File','scripts/verify-ac-source-745.ps1')) { $psi.ArgumentList.Add($a) }
$sw = [Diagnostics.Stopwatch]::StartNew()
$p = [Diagnostics.Process]::Start($psi)
$o = $p.StandardOutput.ReadToEndAsync(); $e = $p.StandardError.ReadToEndAsync()
$timeout = !$p.WaitForExit(1200000)
if ($timeout) { $p.Kill($true); $null = $p.WaitForExit(5000) }
if (!$o.Wait(10000) -or !$e.Wait(10000)) { throw 'Output collection failed' }
[IO.File]::WriteAllText((Join-Path $receipt 'verify745-prepared.stdout.txt'), $o.Result)
[IO.File]::WriteAllText((Join-Path $receipt 'verify745-prepared.stderr.txt'), $e.Result)
[ordered]@{id='verify745-prepared';worktree=$repo;command='powershell -NoProfile -ExecutionPolicy Bypass -File scripts/verify-ac-source-745.ps1';exit=$p.ExitCode;timed_out=$timeout;seconds=$sw.Elapsed.TotalSeconds} | ConvertTo-Json | Set-Content (Join-Path $receipt 'verify745-prepared.json') -Encoding utf8
if ($timeout -or $p.ExitCode) { throw 'Prepared source gate failed' }
"Prepared verify745 passed in $([math]::Round($sw.Elapsed.TotalSeconds,1))s"
