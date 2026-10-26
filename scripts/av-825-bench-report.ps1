# PLAN-825 T-06 parent-side raw-sample validator and distribution report.
# It never drops outliers or failed rows and never promises a speed percentage.
[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)][string]$RawDir,
    [Parameter(Mandatory = $true)][string]$OutFile
)
$ErrorActionPreference = 'Stop'
$engines = @('av-reference', 'av-loaded', 'v8-js-ignition', 'v8-wasm-liftoff', 'v8-wasm-turbofan')
$workloads = @('arithmetic', 'short-b1', 'short-b8', 'short-b64', 'short-b256', 'short-b1024', 'long1024', 'recursive32')
$files = Get-ChildItem -LiteralPath $RawDir -Filter '*.jsonl' -File
if ($files.Count -ne 4) { throw "expected exactly four raw engine files, got $($files.Count)" }
$rows = @()
$fileIds = @()
foreach ($file in $files) {
    $fileRows = @(Get-Content -LiteralPath $file.FullName -Encoding UTF8 | Where-Object { $_.Trim().Length -gt 0 } | ForEach-Object { $_ | ConvertFrom-Json })
    if ($fileRows.Count -ne 344 -and $fileRows.Count -ne 688) { throw "$($file.Name): expected 344 rows per engine, got $($fileRows.Count)" }
    $engineRows = @($fileRows | Select-Object -ExpandProperty engine -Unique)
    if ($engineRows.Count -lt 1 -or $engineRows.Count -gt 2 -or @($engineRows | Where-Object { $engines -notcontains $_ }).Count -ne 0) { throw "$($file.Name): invalid engine identity $engineRows" }
    $rows += $fileRows
    $fileIds += [pscustomobject]@{ path = $file.FullName; sha256 = (Get-FileHash $file.FullName -Algorithm SHA256).Hash.ToLower(); rows = $fileRows.Count }
}
if ($rows.Count -ne 1720) { throw "expected 1720 total rows, got $($rows.Count)" }
$groups = @()
foreach ($engine in $engines) {
    foreach (${workload} in $workloads) {
        $set = @($rows | Where-Object { $_.engine -eq $engine -and $_.workload -eq ${workload} })
        if ($set.Count -ne 43) { throw "${engine}/${workload}: expected 43 rows, got $($set.Count)" }
        foreach ($row in $set) {
            if ($row.expected -ne $row.value) { throw "${engine}/${workload} sample $($row.sample): expected $($row.expected), got $($row.value)" }
        }
        foreach ($phase in @('load', 'first', 'warmup', 'hot', 'process-peak')) {
            $expectedCount = switch ($phase) { 'warmup' { 10 } 'hot' { 30 } default { 1 } }
            $count = @($set | Where-Object { $_.phase -eq $phase }).Count
            if ($count -ne $expectedCount) { throw "${engine}/${workload}/${phase}: expected $expectedCount rows, got $count" }
        }
        $peak = @($set | Where-Object { $_.phase -eq 'process-peak' })[0].private_bytes_peak
        if ($null -eq $peak -or [uint64]$peak -eq 0) { throw "${engine}/${workload}: missing process peak" }
        $hot = @($set | Where-Object { $_.phase -eq 'hot' } | ForEach-Object { [double]$_.ms } | Sort-Object)
        $mean = ($hot | Measure-Object -Average).Average
        $variance = 0.0
        foreach ($ms in $hot) { $variance += ($ms - $mean) * ($ms - $mean) }
        $variance /= $hot.Count
        $groups += [pscustomobject]@{
            engine = $engine; workload = ${workload}; samples = $hot.Count
            p50_ms = $hot[14]; p95_ms = $hot[28]; p99_ms = $hot[29]
            min_ms = $hot[0]; max_ms = $hot[29]; mean_ms = $mean; variance_ms2 = $variance
            process_private_bytes_peak = [uint64]$peak
        }
    }
}
$summary = [pscustomobject]@{
    schema = 'av.runtime.825.bench/1'
    generated_at = (Get-Date).ToUniversalTime().ToString('o')
    warmup_samples = 10; measured_samples = 30; total_rows = $rows.Count
    engines = $engines; workloads = $workloads; raw_files = $fileIds; groups = $groups
}
$parent = Split-Path -Parent $OutFile
if ($parent) { New-Item -ItemType Directory -Force -Path $parent | Out-Null }
$summary | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $OutFile -Encoding UTF8
Write-Host "av825 benchmark: $($groups.Count) groups, $($rows.Count) raw rows validated -> $OutFile"
