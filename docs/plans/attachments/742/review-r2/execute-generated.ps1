$ErrorActionPreference = 'Stop'
$evidence = $PSScriptRoot
$scratch = 'D:/autostack/tmp/p742-review-r2/projects'
$results = @()
foreach ($case in @('control','list_literal','module_literal','brace_literal','borrow_literal','escape_literal','module_control','scope_shadow','void_control','void_branch','option_variable','file_control','file_module')) {
    $project = Join-Path $scratch $case
    $source = Join-Path $project "rust-workspace/$case-back/src/api.rs"
    if (-not(Test-Path -LiteralPath $source)) { $results += @{case=$case;generation='missing artifact'}; continue }
    $generated = Get-Content -Raw -LiteralPath $source
    $handler = $generated.Substring($generated.IndexOf('pub async fn echo'))
    $caseDir = Join-Path $evidence "cases/$case"
    New-Item -ItemType Directory -Force -Path $caseDir | Out-Null
    Copy-Item -LiteralPath $source -Destination (Join-Path $caseDir 'api.rs')
    Copy-Item -LiteralPath (Join-Path $project 'src/back/api.at') -Destination (Join-Path $caseDir 'api.at')
    Copy-Item -LiteralPath (Join-Path $project 'src/back/repository.at') -Destination (Join-Path $caseDir 'repository.at')
    $prefix = @'
#![allow(unused)]
use std::{future::Future, sync::Arc, task::{Context,Poll,Wake,Waker}};
#[derive(Debug,Clone)] pub struct Probe { pub text: String }
pub struct State<T>(T);
pub struct Path<T>(T);
pub type Db = ();
pub struct JsonResponse<T>(pub T);
#[derive(Debug)] pub struct StatusCode;
impl StatusCode { const OK: Self=Self; const NOT_FOUND:Self=Self; }
mod repository { pub fn ping() -> i64 { 7 } pub fn take(v: &str) -> i64 { v.len() as i64 } pub fn path() -> String { "readme.txt".to_string() } }
mod axum { pub mod http { pub struct Method; pub struct HeaderMap; } pub mod response { #[derive(Debug)] pub struct Response(pub String); } }
mod auto_lang { pub mod a2r_std { pub mod http { pub struct FileResponse(pub String); pub fn file_response(_: &str, path: &str, _: &str) -> FileResponse { FileResponse(path.to_string()) } } } }
async fn __file_reply(d: auto_lang::a2r_std::http::FileResponse, _: &axum::http::Method, _: &axum::http::HeaderMap) -> axum::response::Response { axum::response::Response(d.0) }
struct Noop;
impl Wake for Noop { fn wake(self: Arc<Self>) {} }
fn ready<T>(f: impl Future<Output=T>) -> T {
    let w = Waker::from(Arc::new(Noop));
    let mut c = Context::from_waker(&w);
    let mut f = std::pin::pin!(f);
    match f.as_mut().poll(&mut c) { Poll::Ready(v)=>v, Poll::Pending=>panic!("unexpectedly pending") }
}
'@
    $call = if ($case.StartsWith('file_')) { 'fn main() { println!("{:?}",ready(echo(State(()),axum::http::Method,axum::http::HeaderMap)).0); }' } elseif ($case.StartsWith('void_')) { 'fn main() { println!("{:?}",ready(echo(State(())))); }' } elseif($case -eq 'option_variable') { 'fn main() { for id in [1,-1] { match ready(echo(Path(id),State(()))) { Ok(v)=>println!("{id}:{}",v.0.text),Err(_)=>println!("{id}:not_found") } } }' } else { 'fn main() { println!("{:?}",ready(echo(State(()))).0.text); }' }
    $runner = Join-Path $caseDir 'runner.rs'
    [IO.File]::WriteAllText($runner, $prefix + "`n" + $handler + "`n" + $call)
    $exe = Join-Path $project 'runner.exe'
    rustc --edition=2021 $runner -o $exe *> (Join-Path $caseDir 'compile.log')
    $compileExit = $LASTEXITCODE
    $actual = $null
    if ($compileExit -eq 0) { $actual = (& $exe | Out-String).Trim() }
    $results += [ordered]@{case=$case;compile_exit=$compileExit;actual=$actual;source_sha256=(Get-FileHash -LiteralPath (Join-Path $caseDir 'api.at') -Algorithm SHA256).Hash;generated_sha256=(Get-FileHash -LiteralPath $source -Algorithm SHA256).Hash}
}
$results | ConvertTo-Json | Set-Content (Join-Path $evidence 'generated-results.json')
Get-Content (Join-Path $evidence 'generated-results.json')
