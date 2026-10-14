param([string]$Repo = 'D:/autostack/.wt/lang-770/auto-lang')
$ErrorActionPreference = 'Stop'
$deps = Join-Path $Repo 'experimental/ac-core/target/debug/deps'
$nativeLib = Join-Path $env:USERPROFILE '.cargo/registry/src/mirrors.aliyun.com-0671735e7cc7f5e7/windows_x86_64_msvc-0.52.6/lib'
$out = Join-Path $Repo 'experimental/ac-core/target/review-770-r8/emit-probes.exe'
& rustc --edition=2021 (Join-Path $PSScriptRoot 'emit-probes.rs') --extern "auto_ac_prototype=$deps/libauto_ac_prototype-902627b1f8bf81b3.rlib" -L "dependency=$deps" -L "native=$nativeLib" -o $out
if ($LASTEXITCODE -ne 0) { throw 'API probe compile failed' }
& $out $PSScriptRoot
if ($LASTEXITCODE -ne 0) { throw 'API probe execution failed' }
