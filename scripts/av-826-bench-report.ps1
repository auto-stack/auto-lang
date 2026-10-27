# PLAN-826 T-06 parent-side raw-sample validator, wire-size/compression
# report and distribution summary. It never drops outliers or failed rows
# and never promises a speed percentage. Compression uses ONE algorithm
# and parameter set (.NET GZipStream, optimal buffer) for BOTH wire
# generations — same-params by construction.
[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)][string]$RawDir,
    [Parameter(Mandatory = $true)][string]$OutFile
)
$ErrorActionPreference = 'Stop'
$engines = @('av-reference', 'av-loaded', 'v8-js-ignition', 'v8-wasm-liftoff', 'v8-wasm-turbofan')
$commonWorkloads = @('arithmetic', 'short-b1', 'short-b8', 'short-b64', 'short-b256', 'short-b1024', 'long1024', 'recursive32')
$wireWorkloads = @('many-small-functions', 'short-consts', 'large-indices', 'debug-present', 'debug-stripped')
$allAvWorkloads = $commonWorkloads + $wireWorkloads

function Get-GzipBytes {
    param([byte[]]$Bytes)
    $ms = New-Object System.IO.MemoryStream
    $gz = New-Object System.IO.Compression.GZipStream($ms, [System.IO.Compression.CompressionLevel]::Optimal, $true)
    $gz.Write($Bytes, 0, $Bytes.Length)
    $gz.Dispose()
    $ms.ToArray()
}

$avPath = Join-Path $RawDir 'av826.jsonl'
if (-not (Test-Path $avPath)) { throw "missing AV raw file $avPath" }
$avRows = @(Get-Content -LiteralPath $avPath -Encoding UTF8 | Where-Object { $_.Trim().Length -gt 0 } | ForEach-Object { $_ | ConvertFrom-Json })
$avEngines = @($avRows | Where-Object { $_.engine -ne 'wire-size' } | Select-Object -ExpandProperty engine -Unique)
foreach ($e in $avEngines) {
    if ($engines -notcontains $e) { throw "invalid AV engine identity $e" }
}
# Engine rows: 13 workloads x 2 engines x 43 rows.
foreach ($engine in @('av-reference', 'av-loaded')) {
    foreach ($workload in $allAvWorkloads) {
        $set = @($avRows | Where-Object { $_.engine -eq $engine -and $_.workload -eq $workload })
        if ($set.Count -ne 43) { throw "av/${engine}/${workload}: expected 43 rows, got $($set.Count)" }
        foreach ($row in $set) {
            if ($row.expected -ne $row.value) { throw "av/${engine}/${workload} sample $($row.sample): expected $($row.expected), got $($row.value)" }
            if ($row.wire -ne 'compact') { throw "av/${engine}/${workload}: engine rows must ride the compact wire" }
        }
        foreach ($phase in @('load', 'first', 'warmup', 'hot', 'process-peak')) {
            $expectedCount = switch ($phase) { 'warmup' { 10 } 'hot' { 30 } default { 1 } }
            $count = @($set | Where-Object { $_.phase -eq $phase }).Count
            if ($count -ne $expectedCount) { throw "av/${engine}/${workload}/${phase}: expected $expectedCount rows, got $count" }
        }
        $peak = @($set | Where-Object { $_.phase -eq 'process-peak' })[0].private_bytes_peak
        if ($null -eq $peak -or [uint64]$peak -eq 0) { throw "av/${engine}/${workload}: missing process peak" }
        if ($engine -eq 'av-loaded') {
            $hot = @($set | Where-Object { $_.phase -eq 'hot' })[0]
            if ([uint64]$hot.loaded_pc_lookups -eq 0) { throw "av-loaded/${workload}: loaded path not engaged" }
            if ([uint64]$hot.reference_pc_lookups -ne 0) { throw "av-loaded/${workload}: reference lookups leaked" }
        }
    }
}

# V8/Wasm engine files: the COMMON load only (the frozen 825 comparator
# face; the wire-density groups are AV-side).
$v8files = @('js.jsonl', 'liftoff.jsonl', 'turbofan.jsonl')
$v8rows = @()
$v8ids = @()
foreach ($name in $v8files) {
    $path = Join-Path $RawDir $name
    if (-not (Test-Path $path)) { throw "missing raw engine file $name" }
    $fileRows = @(Get-Content -LiteralPath $path -Encoding UTF8 | Where-Object { $_.Trim().Length -gt 0 } | ForEach-Object { $_ | ConvertFrom-Json })
    if ($fileRows.Count -ne 344) { throw "${name}: expected 344 rows per engine, got $($fileRows.Count)" }
    $v8rows += $fileRows
    $v8ids += [pscustomobject]@{ path = $path; sha256 = (Get-FileHash $path -Algorithm SHA256).Hash.ToLower(); rows = $fileRows.Count }
}
foreach ($engine in @('v8-js-ignition', 'v8-wasm-liftoff', 'v8-wasm-turbofan')) {
    foreach ($workload in $commonWorkloads) {
        $set = @($v8rows | Where-Object { $_.engine -eq $engine -and $_.workload -eq $workload })
        if ($set.Count -ne 43) { throw "${engine}/${workload}: expected 43 rows, got $($set.Count)" }
        foreach ($row in $set) {
            if ($row.expected -ne $row.value) { throw "${engine}/${workload} sample $($row.sample): expected $($row.expected), got $($row.value)" }
        }
    }
}

# Wire-size table: both generations of every workload + same-algorithm
# compressed sizes from the written payload files.
$sizes = @()
foreach ($workload in $allAvWorkloads) {
    $rowByWire = @{}
    foreach ($wire in @('legacy', 'compact')) {
        $rows = @($avRows | Where-Object { $_.engine -eq 'wire-size' -and $_.workload -eq $workload -and $_.wire -eq $wire })
        if ($rows.Count -ne 1) { throw "wire-size/${workload}/${wire}: expected exactly one size row, got $($rows.Count)" }
        $rowByWire[$wire] = $rows[0]
    }
    $compactRaw = [uint64]$rowByWire['compact'].raw_bytes
    $legacyRaw = [uint64]$rowByWire['legacy'].raw_bytes
    if ($compactRaw -ge $legacyRaw) {
        throw "wire-size/${workload}: compact ${compactRaw} must be smaller than legacy ${legacyRaw}"
    }
    $payloads = @{}
    foreach ($wire in @('legacy', 'compact')) {
        $path = Join-Path $RawDir ("{0}.{1}.bin" -f $workload, $wire)
        if (-not (Test-Path $path)) { throw "missing payload file $path" }
        $bytes = [System.IO.File]::ReadAllBytes($path)
        if ($bytes.Length -ne [int]$rowByWire[$wire].raw_bytes) {
            throw "payload ${path} length $($bytes.Length) != size row $($rowByWire[$wire].raw_bytes)"
        }
        $payloads[$wire] = $bytes
    }
    $sizes += [pscustomobject]@{
        workload = $workload
        legacy_raw = $legacyRaw
        compact_raw = $compactRaw
        legacy_gzip = (Get-GzipBytes $payloads['legacy']).Length
        compact_gzip = (Get-GzipBytes $payloads['compact']).Length
        compact_ratio = [math]::Round($compactRaw / $legacyRaw, 4)
    }
}

# Distribution groups over the hot samples (no outlier dropping).
$groups = @()
foreach ($engine in $engines) {
    foreach ($workload in ($commonWorkloads + $wireWorkloads)) {
        $pool = if ($engine -like 'av-*') { $avRows } else { $v8rows }
        $set = @($pool | Where-Object { $_.engine -eq $engine -and $_.workload -eq $workload })
        if ($set.Count -eq 0) { continue }
        $hot = @($set | Where-Object { $_.phase -eq 'hot' } | ForEach-Object { [double]$_.ms } | Sort-Object)
        if ($hot.Count -ne 30) { throw "${engine}/${workload}: expected 30 hot rows, got $($hot.Count)" }
        $mean = ($hot | Measure-Object -Average).Average
        $variance = 0.0
        foreach ($ms in $hot) { $variance += ($ms - $mean) * ($ms - $mean) }
        $variance /= $hot.Count
        $peak = @($set | Where-Object { $_.phase -eq 'process-peak' })[0].private_bytes_peak
        $groups += [pscustomobject]@{
            engine = $engine; workload = $workload; samples = $hot.Count
            p50_ms = $hot[14]; p95_ms = $hot[28]; p99_ms = $hot[29]
            min_ms = $hot[0]; max_ms = $hot[29]; mean_ms = $mean; variance_ms2 = $variance
            process_private_bytes_peak = [uint64]$peak
        }
    }
}

$totalRows = $avRows.Count + $v8rows.Count
$summary = [pscustomobject]@{
    schema = 'av.wire.826.bench/1'
    generated_at = (Get-Date).ToUniversalTime().ToString('o')
    warmup_samples = 10; measured_samples = 30
    total_rows = $totalRows
    engines = $engines
    common_workloads = $commonWorkloads
    wire_workloads = $wireWorkloads
    compression = 'gzip/CLR-Optimal (both generations, same parameters)'
    raw_files = $v8ids + @([pscustomobject]@{
        path = $avPath; sha256 = (Get-FileHash $avPath -Algorithm SHA256).Hash.ToLower(); rows = $avRows.Count
    })
    wire_sizes = $sizes
    groups = $groups
}
$parent = Split-Path -Parent $OutFile
if ($parent) { New-Item -ItemType Directory -Force -Path $parent | Out-Null }
$summary | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $OutFile -Encoding UTF8
Write-Host "av826 benchmark: $($groups.Count) groups, $($sizes.Count) wire-size rows, $totalRows raw rows validated -> $OutFile"
