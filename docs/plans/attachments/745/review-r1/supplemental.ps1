$ErrorActionPreference='Stop'
$reviewRoot='D:/autostack/auto-lang/docs/plans/attachments/745/review-r1'
$helper=(Get-Content (Join-Path $reviewRoot 'probes.ps1') -Raw).Split('$probes=@(')[0]
Invoke-Expression $helper
$results=@()
$txn=Join-Path $reviewRoot 'transaction'
New-Item -ItemType Directory -Force -Path $txn|Out-Null
$exe=Join-Path $txn 'app.exe'
$files=@($exe,(Join-Path $txn 'app.obj'),(Join-Path $txn 'app.ac-link.txt'))
foreach($file in $files){[IO.File]::WriteAllText($file,'SENTINEL-'+[IO.Path]::GetExtension($file))}
$before=@($files|ForEach-Object{(Get-FileHash -LiteralPath $_ -Algorithm SHA256).Hash})
foreach($case in @(@{id='bad-source';source='fn main() int { return unknown }';extra=@()},@{id='bad-link';source='fn main() int { return 3 }';extra=@('--support-lib',(Join-Path $txn 'absent.lib'))},@{id='wrong-entry';source='fn main(x int) int { return x }';extra=@()})){
 $src=Join-Path $txn ($case.id+'.at'); [IO.File]::WriteAllText($src,$case.source)
 $run=Invoke-ReviewProcess $bin (@('build-source',$src,'--source-profile',$profile,'--entry','main','--output',$exe)+$case.extra)
 $after=@($files|ForEach-Object{(Get-FileHash -LiteralPath $_ -Algorithm SHA256).Hash})
 $same=($before -join ',') -eq ($after -join ',')
 $results+=@{id=$case.id;run=$run;sentinel_preserved=$same;sha256_before=$before;sha256_after=$after}
 if($run.exit -ne 1 -or -not $same){throw "transaction control failed: $($case.id)"}
}
$traceDir=Join-Path $reviewRoot 'large-trace'
New-Item -ItemType Directory -Force -Path $traceDir|Out-Null
$src=Join-Path $traceDir 'input.at'; $exe=Join-Path $traceDir 'app.exe'
[IO.File]::WriteAllText($src,'fn main() int { var i = 0; while i < 20000 { let observed = mark_a(); i = i + 1 }; return 0 }')
$build=Invoke-ReviewProcess $bin @('build-source',$src,'--source-profile',$profile,'--entry','main','--output',$exe,'--capability','hir.test.trace.v1','--support-lib',(Join-Path $repo 'experimental/ac-core/test-support/target/debug/ac_trace_support.lib'))
if($build.exit -ne 0){throw $build.stderr}
$timer=[Diagnostics.Stopwatch]::StartNew(); $run=Invoke-ReviewProcess $exe @(); $timer.Stop()
if($run.exit -ne 0 -or $run.stderr.Length -ne 20000){throw 'valid trace control failed'}
$results+=@{id='large-trace-drained-control';exit=$run.exit;stderr_length=$run.stderr.Length;duration_ms=$timer.ElapsedMilliseconds}
$psi=[Diagnostics.ProcessStartInfo]::new(); $psi.FileName=$exe; $psi.UseShellExecute=$false; $psi.RedirectStandardOutput=$true; $psi.RedirectStandardError=$true
$p=[Diagnostics.Process]::Start($psi)
$timer=[Diagnostics.Stopwatch]::StartNew()
while(-not $p.HasExited -and $timer.ElapsedMilliseconds -lt 2000){Start-Sleep -Milliseconds 20}
$blocked=-not $p.HasExited
if($blocked){$p.Kill();$p.WaitForExit(10000)|Out-Null}
$out=$p.StandardOutput.ReadToEnd(); $err=$p.StandardError.ReadToEnd()
$results+=@{id='large-trace-poll-before-drain';deadline_ms=2000;blocked_before_drain=$blocked;stderr_length=$err.Length;note='Same poll-before-drain ordering as source_native::run_deadlined; shortened deadline, not a claim that the actual 60s test was run.'}
$results|ConvertTo-Json -Depth 8|Set-Content (Join-Path $reviewRoot 'supplemental-results.json') -Encoding utf8
$results|ConvertTo-Json -Depth 8|Write-Output
