$ErrorActionPreference='Stop'
$repo='D:/autostack/.wt/lang-745/auto-lang'
$receipt='D:/autostack/auto-lang/docs/plans/attachments/745/review-r6'
$gates=@(
 @{id='ac-check';exe='cargo';args=@('check','--locked','--all-targets','--manifest-path','experimental/ac-core/Cargo.toml')},
 @{id='ac-fmt';exe='cargo';args=@('fmt','--manifest-path','experimental/ac-core/Cargo.toml','--','--check')},
 @{id='ac-test';exe='cargo';args=@('test','--locked','--manifest-path','experimental/ac-core/Cargo.toml')},
 @{id='verify741';exe='powershell';args=@('-NoProfile','-ExecutionPolicy','Bypass','-File','scripts/verify-ac-741.ps1','-SkipMainGates')},
 @{id='verify745';exe='powershell';args=@('-NoProfile','-ExecutionPolicy','Bypass','-File','scripts/verify-ac-source-745.ps1')},
 @{id='inventory';exe='python';args=@('scripts/acc_inventory.py','--root','.','--output','docs/reports/743-acc-hir-contract','--check','--require-decisions')},
 @{id='root-check';exe='cargo';args=@('check','-p','auto-lang')},
 @{id='root-t';exe='cargo';args=@('t','--no-fail-fast')},
 @{id='root-tv';exe='cargo';args=@('tv','--no-fail-fast')}
)
$results=@()
foreach($g in $gates){
 $psi=[Diagnostics.ProcessStartInfo]::new();$psi.FileName=$g.exe;$psi.WorkingDirectory=$repo;$psi.UseShellExecute=$false;$psi.RedirectStandardOutput=$true;$psi.RedirectStandardError=$true
 # Native pwsh invocation sanitizes PSModulePath when launching Windows
 # PowerShell; ProcessStartInfo does not. Remove the inherited PS7 path.
 if($g.exe -eq 'powershell'){$psi.Environment.Remove('PSModulePath')|Out-Null}
 foreach($a in $g.args){$psi.ArgumentList.Add($a)}
 $sw=[Diagnostics.Stopwatch]::StartNew();$p=[Diagnostics.Process]::Start($psi);$o=$p.StandardOutput.ReadToEndAsync();$e=$p.StandardError.ReadToEndAsync()
 $timeout=-not $p.WaitForExit(1200000);if($timeout){$p.Kill($true);$p.WaitForExit()}
 if(-not $o.Wait(10000) -or -not $e.Wait(10000)){throw 'review gate collection timeout'}
 [IO.File]::WriteAllText((Join-Path $receipt ($g.id+'.stdout.txt')),$o.Result);[IO.File]::WriteAllText((Join-Path $receipt ($g.id+'.stderr.txt')),$e.Result)
 $row=[ordered]@{id=$g.id;command=$g.exe;args=$g.args;exit=$p.ExitCode;timed_out=$timeout;seconds=$sw.Elapsed.TotalSeconds};$results+=$row
 $results|ConvertTo-Json -Depth 5|Set-Content (Join-Path $receipt 'gates.json')
 Write-Output ($g.id+' exit='+$p.ExitCode+' seconds='+[math]::Round($sw.Elapsed.TotalSeconds,1))
}
