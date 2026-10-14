$ErrorActionPreference = 'Stop'
$evidence = $PSScriptRoot
$scratch = 'D:/autostack/tmp/p742-review-r1/projects'
$results = @()
foreach ($case in @('control','list_literal','module_literal','brace_literal')) {
    $source = Join-Path $scratch "$case/rust-workspace/$case-back/src/api.rs"
    $generated = Get-Content -Raw -LiteralPath $source
    $handler = $generated.Substring($generated.IndexOf('pub async fn echo'))
    $caseDir = Join-Path $evidence "cases/$case"
    New-Item -ItemType Directory -Force -Path $caseDir | Out-Null
    Copy-Item -LiteralPath $source -Destination (Join-Path $caseDir 'api.rs')
    Copy-Item -LiteralPath (Join-Path $scratch "$case/src/back/api.at") -Destination (Join-Path $caseDir 'api.at')
    Copy-Item -LiteralPath (Join-Path $scratch "$case/src/back/repository.at") -Destination (Join-Path $caseDir 'repository.at')
    $prefix = @'
#![allow(unused)]
use std::{future::Future, sync::Arc, task::{Context,Poll,Wake,Waker}};
#[derive(Debug)] struct Probe { text: String }
struct State<T>(T);
type Db = ();
struct JsonResponse<T>(T);
struct Noop;
impl Wake for Noop { fn wake(self: Arc<Self>) {} }
fn main() {
    let w = Waker::from(Arc::new(Noop));
    let mut c = Context::from_waker(&w);
    let mut f = std::pin::pin!(echo(State(())));
    match f.as_mut().poll(&mut c) {
        Poll::Ready(v) => println!("{:?}", v.0.text),
        Poll::Pending => panic!("synchronous handler unexpectedly pending"),
    }
}
'@
    $runner = Join-Path $caseDir 'runner.rs'
    [IO.File]::WriteAllText($runner, $prefix + "`n" + $handler)
    $exe = Join-Path $scratch "$case/runner.exe"
    rustc --edition=2021 $runner -o $exe *> (Join-Path $caseDir 'compile.log')
    $compileExit = $LASTEXITCODE
    $actual = $null
    if ($compileExit -eq 0) { $actual = (& $exe | Out-String).Trim() }
    $results += [ordered]@{case=$case; compile_exit=$compileExit; actual=$actual; generated_sha256=(Get-FileHash -LiteralPath $source -Algorithm SHA256).Hash}
}
$results | ConvertTo-Json | Set-Content (Join-Path $evidence 'generated-results.json')
Get-Content (Join-Path $evidence 'generated-results.json')
