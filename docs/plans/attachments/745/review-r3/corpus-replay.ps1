$ErrorActionPreference='Stop'
$r='D:/autostack/auto-lang/docs/plans/attachments/745/review-r3'
$helper=(Get-Content (Join-Path $r 'probes.ps1') -Raw).Split('$probes=@(')[0];Invoke-Expression $helper
$data=Get-Content 'D:/autostack/auto-lang/docs/plans/attachments/745/corpus.json' -Raw|ConvertFrom-Json
$support=Join-Path $repo 'experimental/ac-core/test-support/target/debug/ac_trace_support.lib'
$rows=@()
foreach($c in $data.cases){
 $d=Join-Path $r ('corpus/'+$c.id);New-Item -ItemType Directory -Force $d|Out-Null
 $src=Join-Path $d 'input.at';[IO.File]::WriteAllText($src,$c.source,[Text.UTF8Encoding]::new($false))
 $extra=@();foreach($cap in $c.capabilities){$extra+=@('--capability',$cap)}
 $check=Invoke-ReviewProcess $bin (@('check-source',$src,'--source-profile',$profile)+$extra)
 $build=$null;$run=$null;$hashes=@();$ok=$false
 if($c.kind -eq 'native'){
  $exe=Join-Path $d 'app.exe';if($c.support_lib){$extra+=@('--support-lib',$support)}
  $build=Invoke-ReviewProcess $bin (@('build-source',$src,'--source-profile',$profile,'--entry','main','--output',$exe)+$extra)
  if($build.exit -eq 0){$run=Invoke-ReviewProcess $exe @();foreach($f in @($exe,(Join-Path $d 'app.obj'),(Join-Path $d 'app.ac-link.txt'))){$hashes+=@{name=[IO.Path]::GetFileName($f);sha256=(Get-FileHash $f).Hash.ToLower()}}}
  $ok=($check.exit -eq 0 -and $build.exit -eq 0 -and $run.exit -eq $c.exit -and $run.stderr -ceq $c.stderr)
 } else {$marker='error['+$c.stage+'/'+$c.code+']';$ok=($check.exit -eq 1 -and $check.stderr.Contains($marker))}
 $row=@{id=$c.id;kind=$c.kind;ok=$ok;check=$check;build=$build;run=$run;artifact_hashes=$hashes;input_sha256=(Get-FileHash $src).Hash.ToLower()};$rows+=$row
 $row|ConvertTo-Json -Depth 9|Set-Content (Join-Path $d 'result.json')
}
$rows|ConvertTo-Json -Depth 9|Set-Content (Join-Path $r 'corpus-results.json')
Write-Output ('independent corpus '+@($rows|Where-Object ok).Count+'/'+$rows.Count+' pass')
if(@($rows|Where-Object {-not $_.ok}).Count){$rows|Where-Object {-not $_.ok}|ConvertTo-Json -Depth 5;exit 1}
