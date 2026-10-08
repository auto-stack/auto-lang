$ErrorActionPreference='Stop'
$r='D:/autostack/auto-lang/docs/plans/attachments/745/review-r7'
$helper=(Get-Content (Join-Path $r 'probes.ps1') -Raw).Split('$probes=@(')[0];Invoke-Expression $helper
$dir=Join-Path $r 'extra-probes/M21-unclosed-source-sentinel';New-Item -ItemType Directory -Force $dir|Out-Null
$src=Join-Path $dir 'input.at';[IO.File]::WriteAllText($src,'fn f(x int) int { return x*10 } fn main() int { return f(1+2 }')
$names=@('app.exe','app.obj','app.ac-link.txt');foreach($n in $names){[IO.File]::WriteAllText((Join-Path $dir $n),('SENTINEL-'+$n))}
$before=@();foreach($n in $names){$before+=@{name=$n;sha256=(Get-FileHash (Join-Path $dir $n)).Hash.ToLower()}}
$exe=Join-Path $dir 'app.exe';$check=Invoke-ReviewProcess $bin @('check-source',$src,'--source-profile',$profile)
$build=Invoke-ReviewProcess $bin @('build-source',$src,'--source-profile',$profile,'--entry','main','--output',$exe)
$run=$null;if($build.exit -eq 0){$run=Invoke-ReviewProcess $exe @()}
$after=@();foreach($n in $names){$after+=@{name=$n;sha256=(Get-FileHash (Join-Path $dir $n)).Hash.ToLower()}}
$result=@{id='M21-unclosed-source-sentinel';check=$check;build=$build;run=$run;input_sha256=(Get-FileHash $src).Hash.ToLower();before=$before;after=$after;sentinel_preserved=(@(Compare-Object @($before.sha256) @($after.sha256)).Count -eq 0)}
$result|ConvertTo-Json -Depth 7|Set-Content (Join-Path $dir 'result.json') -Encoding utf8
$result|ConvertTo-Json -Depth 7|Set-Content (Join-Path $r 'malformed-sentinel-results.json') -Encoding utf8
Write-Output ('check='+$check.exit+' build='+$build.exit+' run='+$run.exit+' sentinel_preserved='+$result.sentinel_preserved)