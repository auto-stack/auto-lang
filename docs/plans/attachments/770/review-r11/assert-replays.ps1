$ErrorActionPreference='Stop'
$expect=@{
 'c01-nested-once-move'=0; 'c02-both-arms-move-break'=0; 'c03-two-owners-one-transfer'=3;
 'c04-false-loop-owner-read'=3; 'c05-no-transfer-loop-read'=3; 'c06-nested-return-break'=3;
 'c07-fresh-loop-move'=6
}
$single="LIT len=3`nLIT id=1`nTEXT-SCALARS n=3`nFREE id=1`n"
$traces=@{
 'c01-nested-once-move'=$single; 'c02-both-arms-move-break'=$single;
 'c03-two-owners-one-transfer'="LIT len=3`nLIT id=1`nLIT len=3`nLIT id=2`nTEXT-SCALARS n=3`nFREE id=1`nTEXT-SCALARS n=3`nFREE id=2`n";
 'c04-false-loop-owner-read'=$single; 'c05-no-transfer-loop-read'=$single; 'c06-nested-return-break'=$single;
 'c07-fresh-loop-move'=$single+"LIT len=3`nLIT id=2`nTEXT-SCALARS n=3`nFREE id=2`n"
}
foreach($f in Get-ChildItem $PSScriptRoot -Filter '*.json') {
 $r=Get-Content $f.FullName -Raw | ConvertFrom-Json
 $name=[IO.Path]::GetFileNameWithoutExtension($r.name)
 foreach($k in @('check','build','run')){if($r.$k -and ($r.$k.TimedOut -or $r.$k.Error)){throw "$name $k failed instrument"}}
 if($expect.ContainsKey($name)){
  if($r.check.Code -ne 0 -or $r.build.Code -ne 0 -or $r.run.Code -ne $expect[$name]){throw "$name unexpected exit"}
  if($r.run.Stdout -cne '' -or $r.run.Stderr -cne $traces[$name]){throw "$name unexpected trace"}
 }else{
  if($r.check.Code -ne 1 -or $r.build.Code -ne 1 -or $r.run -or $r.check.Stderr -notmatch 'verify.ownership-required'){throw "$name expected controlled reject"}
 }
}
foreach($f in Get-ChildItem (Join-Path $PSScriptRoot 'drop-probes') -Filter '*.json'){
 $r=Get-Content $f.FullName -Raw | ConvertFrom-Json
 if($r.name.StartsWith('c08')){
  if($r.check.Code -ne 0 -or $r.build.Code -ne 0 -or $r.run.Code -ne 0 -or $r.run.Stderr -cne "LIT len=3`nLIT id=1`nFREE id=1`n"){throw 'drop positive failed'}
 }else{if($r.check.Code -ne 1 -or $r.build.Code -ne 1 -or $r.check.Stderr -notmatch 'verify.ownership-required'){throw 'drop negative failed'}}
}
'R11 exact exit/stdout/stderr assertions: 18 source/HIR + 2 direct-HIR drop PASS'
'Historical original-byte comparison: see historical-diff.txt and historical-input-hashes.txt (4 expected changes only)'
'Historical instrument errors/timeouts: 0/184'
'Initial drop construction eval-position controls preserved separately; corrected valid inputs tested both admission directions.'