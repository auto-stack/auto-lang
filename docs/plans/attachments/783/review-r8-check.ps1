$ErrorActionPreference='Stop'
$tree='D:/autostack/.wt/lang-783/auto-lang'
$report=Get-Content "$tree/docs/reports/plan783/perf-report.md" -Raw
$json=Get-Content "$tree/docs/reports/plan783/perf.json" -Raw | ConvertFrom-Json
if(-not $report.StartsWith('# PLAN-783 ')){throw 'Heading is not standalone at start'}
if($report.Contains('||')){throw 'Concatenated table fragment remains'}
foreach($s in $json.scenarios){
    $rows=@(($report -split "`n") | Where-Object {$_ -match ('^\| '+[regex]::Escape($s.name)+' \|')})
    if($rows.Count -ne 1){throw "Expected one row for $($s.name)"}
    $parts=$rows[0].Split('|')
    $fields=@('p50_ms','p95_ms','p99_ms')
    for($i=0;$i -lt 3;$i++){
        $actual=[int]($parts[$i+3] -replace '[^0-9]','')
        $expected=[int][Math]::Round($s.stats.($fields[$i]))
        if($actual -ne $expected){throw "Value mismatch: $($s.name) $($fields[$i])"}
    }
    if($parts[6].Trim() -ne "$($s.ok)/$($s.stats.n)"){throw 'Semantic count mismatch'}
    Write-Output "TABLE $($s.name): three percentiles + $($s.ok)/$($s.stats.n) PASS"
}
$required=@('21 趟 GC','73 个 chunk','69 个发生','21趟/73块删/69重叠','查询重叠(69/73)','fresh P50≈246ms','P50≈187ms','P50≈217ms','P99≈503ms','P95≈258ms','P95≈206ms','高约')
foreach($phrase in $required){if(-not $report.Contains($phrase)){throw "Missing expected narrative: $phrase"}}
if($report -match '\b(516|707|774|497|510|364|386|347|387|437|453|398|400|346|327|232)\b'){throw 'Stale R7 numeric fragment remains'}
$oldHash=(Get-FileHash "$PSScriptRoot/review-r7-perf.json" -Algorithm SHA256).Hash
$newHash=(Get-FileHash "$tree/docs/reports/plan783/perf.json" -Algorithm SHA256).Hash
if($oldHash -ne $newHash){throw 'Frozen JSON changed'}
Write-Output 'Heading, narrative, GC counts, old-fragment removal and frozen JSON identity: PASS'
