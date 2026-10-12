$ErrorActionPreference='Stop'
$reviewDir='D:/autostack/auto-lang/docs/plans/attachments/745/review-r4'
$worktree='D:/autostack/.wt/lang-745/auto-lang'
cargo build --locked --manifest-path experimental/ac-core/Cargo.toml --message-format=json > (Join-Path $reviewDir 'cargo-artifacts.jsonl')
if($LASTEXITCODE -ne 0){throw 'Cargo artifact binding failed'}
$artifacts=@(Get-Content (Join-Path $reviewDir 'cargo-artifacts.jsonl')|ForEach-Object{$_|ConvertFrom-Json}|Where-Object{$_.reason -eq 'compiler-artifact' -and $_.target.name -eq 'auto_ac_prototype' -and $_.target.kind -contains 'lib' -and -not $_.profile.test})
if($artifacts.Count -ne 1){throw 'Expected exactly one current library artifact'}
$libs=@($artifacts[0].filenames|Where-Object{$_.EndsWith('.rlib')})
if($libs.Count -ne 1){throw 'Expected one exact current rlib'}
$lib=$libs[0]
$nativeArgs=@()
foreach($record in @(Get-Content (Join-Path $reviewDir 'cargo-artifacts.jsonl')|ForEach-Object{$_|ConvertFrom-Json}|Where-Object{$_.reason -eq 'build-script-executed'})){
 foreach($nativePath in $record.linked_paths){$nativeArgs+=@('-L',$nativePath)}
}
$buildDir=Join-Path $worktree 'experimental/ac-core/target/review-r4'
New-Item -ItemType Directory -Force -Path $buildDir|Out-Null
$rs=Join-Path $buildDir 'bundle_probe.rs';$exe=Join-Path $buildDir 'bundle_probe.exe'
Copy-Item -LiteralPath (Join-Path $reviewDir 'bundle-probe.rs.txt') -Destination $rs
rustc --edition=2021 --crate-name ac745_review_bundle $rs --extern "auto_ac_prototype=$lib" -L "dependency=$(Join-Path $worktree 'experimental/ac-core/target/debug/deps')" @nativeArgs -o $exe
if($LASTEXITCODE -ne 0){throw 'Bundle probe rustc failed'}
$psi=[Diagnostics.ProcessStartInfo]::new();$psi.FileName=$exe;$psi.WorkingDirectory=$worktree;$psi.UseShellExecute=$false;$psi.RedirectStandardOutput=$true;$psi.RedirectStandardError=$true
$p=[Diagnostics.Process]::Start($psi);$out=$p.StandardOutput.ReadToEndAsync();$err=$p.StandardError.ReadToEndAsync()
if(-not $p.WaitForExit(10000)){$p.Kill($true);$p.WaitForExit();throw 'Bundle review harness deadline'}
if(-not $out.Wait(1000) -or -not $err.Wait(1000)){throw 'Bundle output deadline'}
$result=[ordered]@{exact_rlib=$lib;rlib_sha256=(Get-FileHash $lib -Algorithm SHA256).Hash.ToLowerInvariant();binary_sha256=(Get-FileHash $exe -Algorithm SHA256).Hash.ToLowerInvariant();exit=$p.ExitCode;stdout=$out.Result;stderr=$err.Result}
$result|ConvertTo-Json -Depth 4|Set-Content (Join-Path $reviewDir 'bundle-results.json') -Encoding utf8
$result|ConvertTo-Json -Depth 4|Write-Output
if($p.ExitCode -ne 0){throw 'Bundle control failed'}
