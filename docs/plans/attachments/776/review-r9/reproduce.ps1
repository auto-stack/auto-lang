$ErrorActionPreference = 'Stop'
$reviewRoot = $PSScriptRoot
$treeRoot = 'D:/autostack/.wt/lang-776/auto-lang'
Push-Location -LiteralPath $treeRoot
try {
    & cargo build -p auto-lang --lib *> "$reviewRoot/build-library.log"
    if ($LASTEXITCODE -ne 0) { throw 'library build failed' }
    $library = Get-ChildItem target/debug/deps -Filter 'libauto_lang*.rlib' | Sort-Object LastWriteTime -Descending | Select-Object -First 1
    $nativeArgs = @()
    Get-ChildItem 'C:/Users/zhaop/.cargo/registry/src' -Directory | ForEach-Object { Get-ChildItem $_.FullName -Directory -Filter 'windows_x86_64_msvc-*' } | ForEach-Object { $nativeArgs += @('-L', ('native=' + $_.FullName + '/lib')) }
    Get-ChildItem target/debug/build -Directory | ForEach-Object { $nativeRoot = Join-Path $_.FullName 'out'; if (Test-Path -LiteralPath $nativeRoot) { $nativeArgs += @('-L', ('native=' + $nativeRoot)) } }
    & rustc "$reviewRoot/probe.rs" --edition=2021 --extern "auto_lang=$($library.FullName)" -L dependency=target/debug/deps @nativeArgs --out-dir "$reviewRoot" -C linker=rust-lld.exe -C link-arg=/STACK:33554432 2> "$reviewRoot/build.log"
    if ($LASTEXITCODE -ne 0) { throw 'probe compilation failed' }
    "HEAD=$(git rev-parse HEAD) library=$($library.Name) SHA256=$((Get-FileHash $library.FullName -Algorithm SHA256).Hash)" | Set-Content "$reviewRoot/baseline.txt"
    foreach ($mode in @('native','shared-native','identity','direct','local','helper','internal','plain','legacy-int','helper-int','future-number','future-number-int','future-branch','future-helper','future-number-async','future-array','future-record','future-closure')) {
        & "$reviewRoot/probe.exe" $mode > "$reviewRoot/$mode.log" 2>&1
        if ($LASTEXITCODE -ne 0) { throw "probe execution failed: $mode" }
    }
    # Logs capture observed behavior; successful invocation is not acceptance pass.
} finally { Pop-Location }
