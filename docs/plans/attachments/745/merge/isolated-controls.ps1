$ErrorActionPreference = 'Stop'
$repo = 'D:/autostack/auto-lang'
$receipt = Join-Path $repo 'docs/plans/attachments/745/merge'
if (@(Get-Process | Where-Object { $_.ProcessName -match '^(cargo|cargo-nextest|nextest)$' }).Count) { throw 'Batch still active' }
$tests = @('stage3_memory_baseline_n1_3_5','plan707_client_sse_poll_close_error_surface')
$rows = @()
foreach ($test in $tests) {
    $psi = [Diagnostics.ProcessStartInfo]::new()
    $psi.FileName='cargo'; $psi.WorkingDirectory=$repo; $psi.UseShellExecute=$false
    $psi.RedirectStandardOutput=$true; $psi.RedirectStandardError=$true
    foreach ($a in @('tf',$test,'--jobs','1','--no-fail-fast')) { $psi.ArgumentList.Add($a) }
    $sw=[Diagnostics.Stopwatch]::StartNew(); $p=[Diagnostics.Process]::Start($psi)
    $o=$p.StandardOutput.ReadToEndAsync(); $e=$p.StandardError.ReadToEndAsync()
    $timeout=!$p.WaitForExit(1200000)
    if ($timeout) { $p.Kill($true); $null=$p.WaitForExit(5000) }
    if (!$o.Wait(10000) -or !$e.Wait(10000)) { throw 'Output collection failed' }
    [IO.File]::WriteAllText((Join-Path $receipt ($test+'.stdout.txt')),$o.Result)
    [IO.File]::WriteAllText((Join-Path $receipt ($test+'.stderr.txt')),$e.Result)
    $rows += [ordered]@{test=$test;command='cargo tf '+$test+' --jobs 1 --no-fail-fast';working_directory=$repo;exit=$p.ExitCode;timed_out=$timeout;seconds=$sw.Elapsed.TotalSeconds}
    $rows|ConvertTo-Json|Set-Content (Join-Path $receipt 'isolated-controls.json') -Encoding utf8
    Write-Output "$test exit=$($p.ExitCode) seconds=$([math]::Round($sw.Elapsed.TotalSeconds,1))"
}
