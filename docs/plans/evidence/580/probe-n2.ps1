# Plan 580 T3 evidence: resolve_ninja_host() under PATH without ninja (n2 present)
$bin = 'D:\soft\bin'
$env:PATH = ($env:PATH -split ';' | Where-Object { $_ -and $_.TrimEnd('\') -ne $bin }) -join ';'
Set-Location 'D:\autostack\.wt\lang-580\auto-lang'
cargo nextest run -p auto-man probe_resolve_logs_host --run-ignored only --success-output immediate
