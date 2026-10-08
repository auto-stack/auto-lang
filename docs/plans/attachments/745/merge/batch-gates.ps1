$ErrorActionPreference = 'Stop'
$repo = 'D:/autostack/auto-lang'
$receipt = Join-Path $repo 'docs/plans/attachments/745/merge'
$active = @(Get-Process | Where-Object { $_.ProcessName -match '^(cargo|cargo-nextest|nextest)$' })
if ($active.Count) { throw 'Another cargo/nextest instance is active; batch must be single instance' }
$landed = Get-Content (Join-Path $receipt 'landed.json') -Raw | ConvertFrom-Json
if ((git -C $repo rev-parse HEAD).Trim() -ne $landed.delivery_commit) { throw 'Main drift before batch' }
$gates = @(
    @{id='main-check';exe='cargo';args=@('check','-p','auto-lang');timeout=1200000},
    @{id='main-inventory';exe='python';args=@('scripts/acc_inventory.py','--root','.','--output','docs/reports/743-acc-hir-contract','--check','--require-decisions');timeout=60000},
    @{id='batch-tf';exe='cargo';args=@('tf','--no-fail-fast');timeout=7200000},
    @{id='batch-tt';exe='cargo';args=@('tt','--no-fail-fast');timeout=3600000},
    @{id='batch-tb';exe='cargo';args=@('tb','--no-fail-fast');timeout=3600000}
)
$results = @()
foreach ($g in $gates) {
    $psi = [Diagnostics.ProcessStartInfo]::new()
    $psi.FileName = $g.exe; $psi.WorkingDirectory = $repo
    $psi.UseShellExecute = $false; $psi.RedirectStandardOutput = $true; $psi.RedirectStandardError = $true
    foreach ($a in $g.args) { $psi.ArgumentList.Add($a) }
    $sw = [Diagnostics.Stopwatch]::StartNew()
    $p = [Diagnostics.Process]::Start($psi)
    $o = $p.StandardOutput.ReadToEndAsync(); $e = $p.StandardError.ReadToEndAsync()
    $timedOut = !$p.WaitForExit($g.timeout)
    if ($timedOut) { $p.Kill($true); $null = $p.WaitForExit(5000) }
    if (!$o.Wait(10000) -or !$e.Wait(10000)) { throw 'Batch output collection failed' }
    [IO.File]::WriteAllText((Join-Path $receipt ($g.id+'.stdout.txt')), $o.Result)
    [IO.File]::WriteAllText((Join-Path $receipt ($g.id+'.stderr.txt')), $e.Result)
    $results += [ordered]@{id=$g.id;working_directory=$repo;head=$landed.delivery_commit;command=$g.exe;args=$g.args;exit=$p.ExitCode;timed_out=$timedOut;seconds=$sw.Elapsed.TotalSeconds}
    $results | ConvertTo-Json -Depth 6 | Set-Content (Join-Path $receipt 'batch-gates.json') -Encoding utf8
    Write-Output "$($g.id) exit=$($p.ExitCode) seconds=$([math]::Round($sw.Elapsed.TotalSeconds,1))"
    if ($timedOut -or ($g.id -in @('main-check','main-inventory') -and $p.ExitCode)) { throw "Required integration prerequisite failed: $($g.id)" }
}
