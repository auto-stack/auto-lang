#requires -Version 7.0
param([Parameter(Mandatory=$true)][string]$RepoRoot)
$ErrorActionPreference='Stop'
$RepoRoot=(Resolve-Path -LiteralPath $RepoRoot).Path
$bin=Join-Path $RepoRoot 'experimental/ac-core/target/debug/auto-ac-prototype.exe'
if(!(Test-Path -LiteralPath $bin)){throw 'Run cargo build --manifest-path experimental/ac-core/Cargo.toml --locked first'}
$work=Join-Path ([System.IO.Path]::GetTempPath()) ('p741-quality-'+[guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $work | Out-Null
function Invoke-Probe([string]$Program,[string[]]$Arguments) {
  $psi=[System.Diagnostics.ProcessStartInfo]::new()
  $psi.FileName=$Program
  $psi.UseShellExecute=$false
  $psi.CreateNoWindow=$true
  $psi.RedirectStandardOutput=$true
  $psi.RedirectStandardError=$true
  foreach($argument in $Arguments){$psi.ArgumentList.Add($argument)}
  $process=[System.Diagnostics.Process]::Start($psi)
  $stdout=$process.StandardOutput.ReadToEndAsync()
  $stderr=$process.StandardError.ReadToEndAsync()
  if(!$process.WaitForExit(15000)){$process.Kill($true);$process.WaitForExit();throw "Probe exceeded 15s: $Program"}
  $result=@{exit=$process.ExitCode;stdout=$stdout.Result;stderr=$stderr.Result}
  $process.Dispose()
  return $result
}
$cases=@()
foreach($name in @('bool-local','bool-return','binding-valid','binding-invalid','shared-body','loop-return')){
  $inputFile=Join-Path $PSScriptRoot ('fixtures/'+$name+'.atom')
  $entryId=if($name -eq 'shared-body'){'d_entry'}else{'de'}
  $cases+=@{case=$name;check=(Invoke-Probe $bin @('check',$inputFile));build=(Invoke-Probe $bin @('build',$inputFile,'--entry',$entryId,'--output',(Join-Path $work ($name+'.exe'))))}
}
$out=Join-Path $work 'transaction.exe'
$oldInput=Join-Path $RepoRoot 'experimental/ac-core/fixtures/native/add-2-3.atom'
$newInput=Join-Path $RepoRoot 'experimental/ac-core/fixtures/native/add-neg2-3.atom'
$initial=Invoke-Probe $bin @('build',$oldInput,'--entry','d_entry','--output',$out)
if($initial.exit -ne 0){throw 'Initial transaction build failed'}
$beforeExe=(Get-FileHash -LiteralPath $out).Hash
$objPath=[System.IO.Path]::ChangeExtension($out,'obj')
$beforeObj=(Get-FileHash -LiteralPath $objPath).Hash
$receiptPath=[System.IO.Path]::ChangeExtension($out,'ac-link.txt')
Remove-Item -LiteralPath $receiptPath
New-Item -ItemType Directory -Path $receiptPath | Out-Null
$failure=Invoke-Probe $bin @('build',$newInput,'--entry','d_entry','--output',$out)
$atomic=@{build=$failure;exeChanged=($beforeExe -ne (Get-FileHash -LiteralPath $out).Hash);objChanged=($beforeObj -ne (Get-FileHash -LiteralPath $objPath).Hash);native=(Invoke-Probe $out @())}
$readme=Invoke-Probe $bin @('ac-probe','check',(Join-Path $RepoRoot 'docs/design/strategy/hir-examples/01-add.atom'))
$result=@{repo=$RepoRoot;output=$work;cases=$cases;atomic=$atomic;readme=$readme}
$result | ConvertTo-Json -Depth 7 | Set-Content -LiteralPath (Join-Path $work 'reproduction.json') -Encoding utf8
$result | ConvertTo-Json -Depth 7
